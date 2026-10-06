//! Specification-derived synthetic map expectations; no existing golden changed.
use oh_data::map::{EdgeKind, ProvinceKind, load_map};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "oh-map-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("maps/testland")).unwrap();
        fs::create_dir_all(path.join("common")).unwrap();
        for file in [
            "provinces.png",
            "provinces.csv",
            "adjacency_overrides.csv",
            "states.toml",
            "regions.toml",
        ] {
            fs::copy(
                pack().join("maps/testland").join(file),
                path.join("maps/testland").join(file),
            )
            .unwrap();
        }
        for file in ["terrain.toml", "resources.toml", "buildings.toml"] {
            fs::copy(
                pack().join("common").join(file),
                path.join("common").join(file),
            )
            .unwrap();
        }
        Self(path)
    }
    fn write(&self, file: &str, text: &str) {
        fs::write(self.0.join(file), text).unwrap();
    }
    fn replace(&self, file: &str, from: &str, to: &str) {
        let text = fs::read_to_string(self.0.join(file)).unwrap();
        assert!(
            text.contains(from),
            "fixture replacement must exist: {from}"
        );
        self.write(file, &text.replace(from, to));
    }
    fn error(&self, file: &str, message: &str) {
        let err = load_map(&self.0, "testland").unwrap_err();
        assert!(err.path.ends_with(file), "{err}");
        assert!(err.line > 0 && err.column > 0, "{err}");
        assert!(err.message.contains(message), "{err}");
    }
    fn pixels(&self, indices: &[u16]) {
        let colors = [
            [200, 40, 40],
            [40, 200, 40],
            [40, 40, 200],
            [200, 200, 40],
            [40, 160, 200],
            [120, 80, 200],
        ];
        let bytes: Vec<_> = indices.iter().flat_map(|&i| colors[i as usize]).collect();
        self.png(png::ColorType::Rgb, &bytes);
    }
    fn png(&self, color: png::ColorType, bytes: &[u8]) {
        let file = fs::File::create(self.0.join("maps/testland/provinces.png")).unwrap();
        let mut encoder = png::Encoder::new(file, 8, 6);
        encoder.set_color(color);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(bytes)
            .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn pack() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland")
}

#[test]
fn req_map_01_rgb_definition_and_u16_index() {
    let map = load_map(pack(), "testland").unwrap();
    assert_eq!((map.width, map.height), (8, 6));
    assert_eq!(
        map.provinces.iter().map(|p| p.id).collect::<Vec<_>>(),
        [10, 20, 30, 40, 50, 60]
    );
    assert_eq!(map.provinces[0].kind, ProvinceKind::Land);
    assert_eq!(map.provinces[4].kind, ProvinceKind::Sea);
    assert_eq!(map.provinces[5].kind, ProvinceKind::Lake);
    assert_eq!(map.provinces[0].terrain, "plains");
    assert_eq!(
        map.index,
        [
            0, 0, 1, 1, 4, 4, 4, 4, 0, 0, 1, 1, 4, 4, 4, 4, 2, 2, 3, 3, 4, 4, 5, 5, 2, 2, 3, 3, 4,
            4, 5, 5, 2, 2, 3, 3, 4, 4, 4, 4, 2, 2, 3, 3, 4, 4, 4, 4
        ]
    );
    assert_eq!(
        map.index_le_bytes(),
        map.index
            .iter()
            .flat_map(|i| i.to_le_bytes())
            .collect::<Vec<_>>()
    );
    assert_eq!(map.province_index(30), Some(2));
    assert_eq!(map.province_index(999), None);
    assert!(map.warnings.is_empty());
}

#[test]
fn req_map_02_sorted_four_neighbor_adjacency_golden_and_overrides() {
    let f = Fixture::new();
    f.write("maps/testland/adjacency_overrides.csv", "a,b,kind\n");
    let map = load_map(&f.0, "testland").unwrap();
    // Rectangles: diagonal-only 10/40 and 20/30 MUST NOT form edges.
    assert_eq!(
        map.edges
            .iter()
            .map(|e| (e.a, e.b, e.kind))
            .collect::<Vec<_>>(),
        [
            (10, 20, EdgeKind::Normal),
            (10, 30, EdgeKind::Normal),
            (20, 40, EdgeKind::Normal),
            (20, 50, EdgeKind::Normal),
            (30, 40, EdgeKind::Normal),
            (40, 50, EdgeKind::Normal),
            (50, 60, EdgeKind::Normal),
        ]
    );
    f.write(
        "maps/testland/adjacency_overrides.csv",
        "a,b,kind\n20,10,river_small\n30,40,river_large\n10,40,strait\n40,50,impassable\n",
    );
    let map = load_map(&f.0, "testland").unwrap();
    assert_eq!(
        map.edges
            .iter()
            .map(|e| (e.a, e.b, e.kind))
            .collect::<Vec<_>>(),
        [
            (10, 20, EdgeKind::RiverSmall),
            (10, 30, EdgeKind::Normal),
            (10, 40, EdgeKind::Strait),
            (20, 40, EdgeKind::Normal),
            (20, 50, EdgeKind::Normal),
            (30, 40, EdgeKind::RiverLarge),
            (40, 50, EdgeKind::Impassable),
            (50, 60, EdgeKind::Normal),
        ]
    );
    assert_eq!(map.edge(20, 10).unwrap(), map.edge(10, 20).unwrap());
    assert_eq!(map.edge(10, 20).unwrap().distance_km.to_bits(), 4_i64 << 32);
    assert_eq!(map.edge(10, 30).unwrap().distance_km.to_bits(), 6_i64 << 32);
    assert_eq!(map, load_map(&f.0, "testland").unwrap());
}

#[test]
fn req_map_03_states_and_references() {
    let map = load_map(pack(), "testland").unwrap();
    assert_eq!(map.states.len(), 2);
    assert_eq!(map.state_for_province(10), Some(1));
    assert_eq!(map.state_for_province(30), Some(2));
    assert_eq!(map.state_for_province(50), None);
    assert_eq!(map.states[0].population, 1000);
    assert_eq!(map.states[0].resources["steel"], 2);
    assert_eq!(map.states[0].buildings["industry"], 1);
    assert_eq!(map.states[0].infrastructure, 1);
    for (from, to, message) in [
        (
            "provinces = [10, 20]",
            "provinces = [10]",
            "exactly one state",
        ),
        (
            "provinces = [30, 40]",
            "provinces = [10, 30, 40]",
            "multiple states",
        ),
        ("provinces = [10, 20]", "provinces = [10, 20, 50]", "land"),
        (
            "provinces = [10, 20]",
            "provinces = [10, 20, 999]",
            "unknown province",
        ),
        ("population = 1000", "population = -1", "population"),
        ("steel = 2", "steel = -1", "resource amount"),
        ("steel = 2", "unknown = 2", "unknown resource"),
        ("industry = 1", "unknown = 1", "unknown building"),
        ("industry = 1", "industry = -1", "building level"),
        (
            "infrastructure = 1",
            "infrastructure = -1",
            "infrastructure",
        ),
    ] {
        let f = Fixture::new();
        f.replace("maps/testland/states.toml", from, to);
        f.error("states.toml", message);
    }
}

#[test]
fn req_map_09_victory_point_definition_and_invalid_references() {
    let map = load_map(pack(), "testland").unwrap();
    assert_eq!(map.victory_points.len(), 2);
    assert_eq!(
        (map.victory_points[0].province, map.victory_points[0].points),
        (10, 3)
    );
    assert_eq!(map.victory_points[0].name_key, "vp_testland_north");
    for (from, to, message) in [
        ("province = 10", "province = 999", "unknown province"),
        ("province = 30", "province = 10", "duplicate victory point"),
        ("points = 3", "points = 0", "positive"),
        ("vp_testland_north", "", "name_key"),
    ] {
        let f = Fixture::new();
        f.replace("maps/testland/states.toml", from, to);
        f.error("states.toml", message);
    }
    // 01 §4.1 says VP belongs to a province, without a land-only restriction.
    // Keep that rule open for data-driven eras rather than inventing a ban.
    let f = Fixture::new();
    f.replace(
        "maps/testland/states.toml",
        "province = 10",
        "province = 50",
    );
    assert_eq!(
        load_map(&f.0, "testland").unwrap().victory_points[1].province,
        50
    );
}

#[test]
fn strict_csv_and_override_errors_have_exact_locations() {
    let f = Fixture::new();
    f.replace(
        "maps/testland/provinces.csv",
        "10,200,40,40,land,plains,true",
        "10,200,40,40,land,missing,true",
    );
    let err = load_map(&f.0, "testland").unwrap_err();
    assert_eq!((err.line, err.column), (2, 19));
    assert!(err.message.contains("unknown terrain"));
    for (text, message) in [
        ("a,b,kind\n10,999,strait\n", "unknown province"),
        ("a,b,kind\n10,10,strait\n", "self edge"),
        (
            "a,b,kind\n10,20,strait\n20,10,impassable\n",
            "duplicate override",
        ),
        ("a,b,kind\n10,40,river_small\n", "bitmap adjacency"),
        ("a,b,kind\n10,20,unknown\n", "override kind"),
    ] {
        let f = Fixture::new();
        f.write("maps/testland/adjacency_overrides.csv", text);
        f.error("adjacency_overrides.csv", message);
    }
}

#[test]
fn scale_is_exact_fixed_positive_and_checked() {
    for scale in ["0", "-1", "nan", "inf", "2147483648", "\"2\""] {
        let f = Fixture::new();
        f.write(
            "maps/testland/regions.toml",
            &format!("km_per_pixel = {scale}\n"),
        );
        f.error("regions.toml", "km_per_pixel");
    }
    let f = Fixture::new();
    f.write("maps/testland/regions.toml", "km_per_pixel = 0.1\n");
    assert_eq!(
        load_map(&f.0, "testland").unwrap().km_per_pixel.to_bits(),
        "0.1".parse::<oh_data::Fixed>().unwrap().to_bits()
    );
    let f = Fixture::new();
    f.write("maps/testland/regions.toml", "km_per_pixel = 2147483647\n");
    f.error("regions.toml", "distance overflow");
}

#[test]
fn req_map_01_unknown_rgb_single_pixel_missing_and_disconnected_islands() {
    let base = load_map(pack(), "testland").unwrap();
    let f = Fixture::new();
    let mut rgb: Vec<_> = base
        .index
        .iter()
        .flat_map(|&i| base.provinces[i as usize].rgb)
        .collect();
    rgb[0..3].copy_from_slice(&[1, 2, 3]);
    f.png(png::ColorType::Rgb, &rgb);
    f.error("provinces.png", "unknown RGB [1, 2, 3] at pixel (0,0)");
    let f = Fixture::new();
    let mut index = base.index.clone();
    for i in &mut index[1..] {
        if *i == 0 {
            *i = 1;
        }
    }
    f.pixels(&index);
    f.error("provinces.png", "1-pixel province 10");
    let f = Fixture::new();
    let index: Vec<_> = base
        .index
        .iter()
        .map(|&i| if i == 0 { 1 } else { i })
        .collect();
    f.pixels(&index);
    f.error("provinces.csv", "province 10 has no bitmap pixels");
    let f = Fixture::new();
    let mut index = base.index.clone();
    index[47] = 0;
    f.pixels(&index);
    let map = load_map(&f.0, "testland").unwrap();
    assert_eq!(map.warnings.len(), 1);
    assert_eq!(
        (
            map.warnings[0].province,
            map.warnings[0].line,
            map.warnings[0].column
        ),
        (10, 2, 1)
    );
    assert!(
        map.warnings[0]
            .message
            .contains("2 disconnected components")
    );
    let csv = fs::read_to_string(f.0.join("maps/testland/provinces.csv")).unwrap();
    let csv = csv
        .lines()
        .enumerate()
        .map(|(i, row)| {
            format!(
                "{row},{}\n",
                if i == 0 {
                    "island"
                } else if i == 1 {
                    "true"
                } else {
                    "false"
                }
            )
        })
        .collect::<String>();
    f.write("maps/testland/provinces.csv", &csv);
    assert!(load_map(&f.0, "testland").unwrap().warnings.is_empty());
    // Island exception must not suppress single-pixel errors.
    let mut index = base.index.clone();
    for i in &mut index[1..] {
        if *i == 0 {
            *i = 1;
        }
    }
    f.pixels(&index);
    f.error("provinces.png", "1-pixel province 10");
}

#[test]
fn req_map_01_png_format_and_csv_integrity() {
    for (color, bytes_per_pixel) in [(png::ColorType::Grayscale, 1), (png::ColorType::Rgba, 4)] {
        let f = Fixture::new();
        f.png(color, &vec![0; 48 * bytes_per_pixel]);
        f.error("provinces.png", "24-bit RGB");
    }
    let f = Fixture::new();
    fs::write(f.0.join("maps/testland/provinces.png"), b"not a png").unwrap();
    f.error("provinces.png", "invalid PNG");
    let f = Fixture::new();
    let mut bytes = fs::read(f.0.join("maps/testland/provinces.png")).unwrap();
    bytes.truncate(bytes.len() - 6);
    fs::write(f.0.join("maps/testland/provinces.png"), bytes).unwrap();
    f.error("provinces.png", "invalid PNG");
    for (from, to, message) in [
        ("20,40,200,40", "10,40,200,40", "duplicate province id"),
        ("20,40,200,40", "20,200,40,40", "duplicate province RGB"),
        ("10,200,40,40", "65536,200,40,40", "province id"),
        ("10,200,40,40", "10,256,40,40", "red"),
        ("land,plains,true", "unknown,plains,true", "province kind"),
        ("land,plains,true", "land,plains,1", "coastal"),
        ("land,plains,true", "land,\"plains\",true", "unquoted"),
    ] {
        let f = Fixture::new();
        f.replace("maps/testland/provinces.csv", from, to);
        f.error("provinces.csv", message);
    }
}

#[test]
fn req_map_02_row_order_crlf_and_irrational_distance_bits() {
    let f = Fixture::new();
    let original = load_map(&f.0, "testland").unwrap();
    let text = fs::read_to_string(f.0.join("maps/testland/provinces.csv")).unwrap();
    let mut rows = text.lines().skip(1).collect::<Vec<_>>();
    rows.reverse();
    f.write(
        "maps/testland/provinces.csv",
        &format!("id,r,g,b,kind,terrain,coastal\r\n{}\r\n", rows.join("\r\n")),
    );
    assert_eq!(original, load_map(&f.0, "testland").unwrap());
    // Independent floor(sqrt(2^2+3^2)*2^32), times scale 2.
    assert_eq!(
        original.edge(10, 40).unwrap().distance_km.to_bits(),
        30_971_449_624
    );
    assert_eq!(
        original.centers[0].map(|f| f.to_bits()),
        [1_i64 << 32, 1_i64 << 32]
    );
    assert_eq!(
        original.centers[2].map(|f| f.to_bits()),
        [1_i64 << 32, 4_i64 << 32]
    );
}

#[test]
fn req_map_03_toml_schema_and_registry_integrity() {
    for (from, to, message) in [
        ("id = 2", "id = 1", "duplicate state id"),
        ("state_testland_north", "", "name_key"),
        (
            "provinces = [10, 20]",
            "provinces = [10, 10, 20]",
            "repeatedly",
        ),
        ("provinces = [10, 20]", "provinces = []", "land"),
        ("population = 1000", "population = 1.5", "expected i64"),
        ("infrastructure = 1", "unknown = 1", "unknown field"),
    ] {
        let f = Fixture::new();
        f.replace("maps/testland/states.toml", from, to);
        f.error("states.toml", message);
    }
    let f = Fixture::new();
    f.write(
        "common/terrain.toml",
        "[[terrain]]\nid = \"plains\"\n[[terrain]]\nid = \"plains\"\n",
    );
    f.error("terrain.toml", "duplicate registry id");
    let f = Fixture::new();
    f.write("common/terrain.toml", "[[terrain]]\nid = 5\n");
    f.error("terrain.toml", "string");
    let f = Fixture::new();
    f.write("common/terrain.toml", "unknown = []\n");
    f.error("terrain.toml", "unknown registry");
    let f = Fixture::new();
    f.write("maps/testland/states.toml", "state = [\n");
    let err = load_map(&f.0, "testland").unwrap_err();
    assert_eq!(err.kind, oh_data::ErrorKind::Syntax);
    let err = load_map(&f.0, "../testland").unwrap_err();
    assert!(err.message.contains("map id"));
    fs::remove_file(f.0.join("common/terrain.toml")).unwrap();
    let err = load_map(&f.0, "testland").unwrap_err();
    assert_eq!(err.kind, oh_data::ErrorKind::Io);
}

#[test]
fn structural_map_schemas_match_checked_in_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schema");
    for (name, schema) in [
        ("province", oh_data::map::province_schema()),
        ("states", oh_data::map::states_schema()),
        ("regions", oh_data::map::regions_schema()),
    ] {
        let stored: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(format!("{name}.schema.json"))).unwrap())
                .unwrap();
        assert_eq!(stored, serde_json::to_value(schema).unwrap());
    }
}
