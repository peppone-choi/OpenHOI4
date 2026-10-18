//! ADR-0702 synthetic fixtures. Existing packs and golden values are untouched.
use oh_data::{
    Fixed,
    map::{EdgeKind, MapProjection, cos_fx, load_map},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(width: u32, height: u32, pixels: &[u16]) -> Self {
        assert_eq!(pixels.len(), (width * height) as usize);
        let root = std::env::temp_dir().join(format!(
            "oh-projection-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("maps/test")).unwrap();
        fs::create_dir_all(root.join("common")).unwrap();
        fs::write(
            root.join("common/terrain.toml"),
            "[[terrain]]\nid = 'sea'\n",
        )
        .unwrap();
        fs::write(root.join("common/resources.toml"), "").unwrap();
        fs::write(root.join("common/buildings.toml"), "").unwrap();
        let mut csv = String::from("id,r,g,b,kind,terrain,coastal\n");
        for id in pixels
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
        {
            csv.push_str(&format!("{id},{id},0,0,sea,sea,false\n"));
        }
        fs::write(root.join("maps/test/provinces.csv"), csv).unwrap();
        fs::write(root.join("maps/test/states.toml"), "state = []\n").unwrap();
        fs::write(root.join("maps/test/adjacency_overrides.csv"), "a,b,kind\n").unwrap();
        let bytes: Vec<u8> = pixels.iter().flat_map(|&id| [id as u8, 0, 0]).collect();
        let mut encoder = png::Encoder::new(
            fs::File::create(root.join("maps/test/provinces.png")).unwrap(),
            width,
            height,
        );
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&bytes)
            .unwrap();
        let f = Self(root);
        f.regions("[-180,-90,180,90]");
        f
    }
    fn write(&self, file: &str, text: &str) {
        fs::write(self.0.join("maps/test").join(file), text).unwrap();
    }
    fn regions(&self, extent: &str) {
        self.write(
            "regions.toml",
            &format!("km_per_pixel = 2\nprojection = 'equirectangular'\nextent = {extent}\n"),
        );
    }
    fn load(&self) -> oh_data::map::MapData {
        load_map(&self.0, "test").unwrap()
    }
    fn error(&self, message: &str) {
        let e = load_map(&self.0, "test").unwrap_err();
        assert!(e.path.ends_with("regions.toml"), "{e}");
        assert_eq!(e.kind, oh_data::ErrorKind::Schema);
        assert!(e.line > 0 && e.column > 0, "{e}");
        assert!(e.message.contains(message), "{e}");
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn stripes() -> Fixture {
    Fixture::new(8, 4, &[1, 1, 2, 2, 2, 2, 3, 3].repeat(4))
}

#[test]
fn cos_fixed_vectors_and_monotonicity() {
    // PI truncated to Q32.32; exact outputs also pin Horner rounding.
    let pi = Fixed::from_num(fixed::consts::PI).to_bits();
    assert_eq!(pi, 13_493_037_704);
    for (x, bits) in [
        (0, 4_294_967_296),
        (pi / 6, 3_719_550_787),
        (pi / 3, 2_147_483_648),
        // Dropping the x^18 term changes this vector by one raw bit.
        (5_395_933_243, 1_328_436_926),
        (pi / 2, 0),
    ] {
        assert_eq!(cos_fx(Fixed::from_bits(x)).to_bits(), bits);
    }
    let mut previous = Fixed::ONE;
    for step in 0..=64 {
        let c = cos_fx(Fixed::from_bits((pi / 2) * step / 64));
        assert!(c >= Fixed::ZERO && c <= previous);
        previous = c;
    }
}

#[test]
fn wrap_adjacency_then_override_and_no_latitude_wrap() {
    let f = stripes();
    let map = f.load();
    assert_eq!(map.edge(1, 3).unwrap().kind, EdgeKind::Normal);
    assert_eq!(map.edge(1, 3).unwrap().distance_km.to_bits(), 4_i64 << 32);
    f.write("adjacency_overrides.csv", "a,b,kind\n1,3,river_large\n");
    assert_eq!(f.load().edge(1, 3).unwrap().kind, EdgeKind::RiverLarge);
    f.write("adjacency_overrides.csv", "a,b,kind\n1,3,impassable\n");
    assert_eq!(f.load().edge(1, 3).unwrap().kind, EdgeKind::Impassable);
    f.write("adjacency_overrides.csv", "a,b,kind\n");
    f.regions("[-90,-90,90,90]");
    assert!(f.load().edge(1, 3).is_none());
    let f = Fixture::new(
        4,
        6,
        &[
            1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3,
        ],
    );
    assert!(f.load().edge(1, 3).is_none());
}

#[test]
fn dateline_centroid_even_odd_and_tie_a() {
    for (width, row) in [(8, vec![1, 2, 2, 2, 2, 2, 2, 1]), (5, vec![1, 2, 2, 2, 1])] {
        let f = Fixture::new(width, 2, &row.repeat(2));
        assert_eq!(f.load().centers[0][0], Fixed::ZERO);
        f.regions("[-90,-90,90,90]");
        assert_eq!(f.load().centers[0][0], Fixed::from_num(width) / 2);
    }
    // A and B have identical variance; A's centroid must be retained.
    let f = Fixture::new(4, 2, &[1, 2, 1, 2].repeat(2));
    assert_eq!(f.load().centers[0][0].to_bits(), 3_i64 << 31);
}

#[test]
fn high_latitude_distance_and_equator_exact_bits() {
    let mut pixels = Vec::new();
    for row in 0..12 {
        let base = if row < 2 {
            1
        } else if row < 10 {
            3
        } else {
            5
        };
        pixels.extend([
            base,
            base,
            base,
            base,
            base + 1,
            base + 1,
            base + 1,
            base + 1,
        ]);
    }
    let map = Fixture::new(8, 12, &pixels).load();
    let equator = map.edge(3, 4).unwrap().distance_km.to_bits();
    let north = map.edge(1, 2).unwrap().distance_km.to_bits();
    assert_eq!(equator, 24_i64 << 32); // dx=4, r=3, scale=2, cos(0)=1
    assert_eq!(north, 24 * 1_111_619_334); // Q32.32 cos(75 degrees)
    assert_eq!(map.edge(5, 6).unwrap().distance_km.to_bits(), north);
    assert!(north < equator);
}

#[test]
fn full_longitude_polar_cap_has_only_north_south_distance() {
    let f = Fixture::new(
        8,
        4,
        &[vec![1; 8], [2, 2, 2, 2, 3, 3, 3, 3].repeat(3)].concat(),
    );
    let map = f.load();
    assert_eq!(map.centers[0][0], Fixed::from_num(4));
    for id in [2, 3] {
        assert_eq!(map.edge(1, id).unwrap().distance_km.to_bits(), 4_i64 << 32);
    }
}

#[test]
fn invalid_projection_extent_and_unknown_fields() {
    let f = stripes();
    for text in [
        "projection = 'uniform'",
        "projection = 1",
        "projection = 'equirectangular'",
        "extent = [-180,-90,180,90]",
        "projection = nan",
    ] {
        f.write("regions.toml", &format!("km_per_pixel = 2\n{text}\n"));
        f.error(if text.starts_with("extent") {
            "extent"
        } else {
            "projection"
        });
    }
    for extent in [
        "1",
        "[]",
        "[-180,-90,180]",
        "[-180,-90,180,90,1]",
        "['-180',-90,180,90]",
        "[nan,-90,180,90]",
        "[-inf,-90,180,90]",
        "[-181,-90,180,90]",
        "[-180,-91,180,90]",
        "[-180,-90,181,90]",
        "[-180,-90,180,91]",
        "[0,-90,0,90]",
        "[-1,2,1,2]",
        "[1,-90,-1,90]",
        "[-1,3,1,2]",
    ] {
        f.regions(extent);
        f.error("extent");
    }
    f.write("regions.toml", "km_per_pixel = 2\nunknown = 1\n");
    f.error("unknown regions field");
}

#[test]
fn exact_source_numbers_and_checked_small_extent_overflow() {
    let f = stripes();
    f.regions("[-1.00000000023283064365386962890625,-1,1,1]");
    let MapProjection::Equirectangular { west, wrap_x, .. } = f.load().projection else {
        panic!()
    };
    assert_eq!(west.to_bits(), -(1_i64 << 32) - 1);
    assert!(!wrap_x);
    // One raw bit / height floors to a zero vertical divisor, never panic.
    f.regions("[-180,0,180,0.00000000023283064365386962890625]");
    f.error("extent distance overflow");
    // Nonzero divisor, but the horizontal/vertical ratio cannot fit I32F32.
    f.regions("[-180,0,180,0.00000001]");
    f.error("extent distance overflow");
    f.write(
        "regions.toml",
        "km_per_pixel = 2147483647\nprojection = 'equirectangular'\nextent = [-180,-90,180,90]\n",
    );
    f.error("distance overflow for");
}

#[test]
fn zero_distance_rejected_only_in_new_mode() {
    let f = Fixture::new(4, 4, &[1, 2, 2, 1, 2, 1, 1, 2, 2, 1, 1, 2, 1, 2, 2, 1]);
    f.regions("[-90,-90,90,90]");
    f.error("zero distance for 1/2");
    f.write("regions.toml", "km_per_pixel = 2\n");
    let map = f.load();
    assert_eq!(map.projection, MapProjection::Uniform);
    assert_eq!(map.edge(1, 2).unwrap().distance_km, Fixed::ZERO);
}
