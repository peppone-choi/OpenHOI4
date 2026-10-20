//! Opt-in hosted geometry harness. No product pack or source loader relaxation.
use oh_data::{
    Fixed,
    map::{EdgeKind, MapProjection, ProvinceKind, load_map},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    time::Instant,
};

#[derive(Deserialize)]
struct Expected {
    width: u32,
    height: u32,
    provinces: Vec<u16>,
    counts: BTreeMap<u16, u64>,
    kinds: BTreeMap<u16, String>,
    kind_pixels: BTreeMap<String, u64>,
    edges: BTreeSet<(u16, u16)>,
    seam_samples: Vec<(u16, u16)>,
    polar: Vec<u16>,
}

struct TemporaryPack(PathBuf);
impl Drop for TemporaryPack {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove only harness-owned temporary pack");
    }
}

#[test]
fn world_preview_geometry_load_map() {
    let Some(source) = std::env::var_os("OH_WORLD_GEOMETRY") else {
        eprintln!(
            "SKIP: set OH_WORLD_GEOMETRY and OH_WORLD_EXPECTED for hosted geometry harness, not product pack"
        );
        return;
    };
    eprintln!("geometry harness, not product pack");
    let source = PathBuf::from(source);
    let expected_path = PathBuf::from(
        std::env::var_os("OH_WORLD_EXPECTED").expect("independent Python oracle required"),
    );
    let expected: Expected = serde_json::from_slice(&fs::read(&expected_path).unwrap()).unwrap();
    let root = std::env::temp_dir().join(format!("oh-world-geometry-{}", std::process::id()));
    // Never reuse/delete another run's directory, including a stale one.
    fs::create_dir(&root).unwrap();
    let pack = TemporaryPack(root);
    let map_dir = pack.0.join("maps/geometry");
    fs::create_dir_all(&map_dir).unwrap();
    fs::create_dir(pack.0.join("common")).unwrap();
    let testland =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland/common");
    for name in ["terrain.toml", "resources.toml", "buildings.toml"] {
        fs::copy(testland.join(name), pack.0.join("common").join(name)).unwrap();
    }
    for name in [
        "provinces.png",
        "provinces.csv",
        "adjacency_overrides.csv",
        "regions.toml",
    ] {
        fs::copy(source.join(name), map_dir.join(name)).unwrap();
    }
    let land: Vec<_> = expected
        .provinces
        .iter()
        .copied()
        .filter(|id| expected.kinds[id] == "land")
        .collect();
    fs::write(map_dir.join("states.toml"), format!(
        "# geometry harness, not product pack\n[[state]]\nid = 1\nname_key = 'geometry_harness'\nprovinces = {land:?}\npopulation = 0\nresources = {{}}\nbuildings = {{}}\ninfrastructure = 0\n"
    )).unwrap();
    let start = Instant::now();
    let loaded = load_map(&pack.0, "geometry");
    let elapsed = start.elapsed().as_secs_f64();
    let metrics = serde_json::json!({
        "marker": "geometry harness, not product pack",
        "load_map_seconds": elapsed,
        "ok": loaded.is_ok(),
        "error": loaded.as_ref().err().map(ToString::to_string),
    });
    fs::write(
        expected_path.with_file_name("load-map-metrics.json"),
        serde_json::to_vec_pretty(&metrics).unwrap(),
    )
    .unwrap();
    let map = loaded.expect("actual load_map must succeed without relaxed validation");
    assert!(matches!(
        map.projection,
        MapProjection::Equirectangular { wrap_x: true, .. }
    ));
    assert_eq!((map.width, map.height), (expected.width, expected.height));
    assert_eq!(
        map.provinces.iter().map(|p| p.id).collect::<Vec<_>>(),
        expected.provinces
    );
    let mut counts = BTreeMap::<u16, u64>::new();
    let mut kind_pixels = BTreeMap::<String, u64>::new();
    for &dense in &map.index {
        let p = &map.provinces[dense as usize];
        *counts.entry(p.id).or_default() += 1;
        let kind = match p.kind {
            ProvinceKind::Land => "land",
            ProvinceKind::Sea => "sea",
            ProvinceKind::Lake => "lake",
        };
        assert_eq!(kind, expected.kinds[&p.id]);
        *kind_pixels.entry(kind.to_owned()).or_default() += 1;
    }
    assert_eq!(counts, expected.counts);
    assert_eq!(kind_pixels, expected.kind_pixels);
    assert_eq!(
        map.edges
            .iter()
            .map(|e| (e.a, e.b))
            .collect::<BTreeSet<_>>(),
        expected.edges
    );
    for edge in &map.edges {
        assert_eq!(edge.kind, EdgeKind::Normal);
        assert!(edge.distance_km > Fixed::ZERO);
    }
    assert_eq!(expected.seam_samples.len(), 3);
    for &(a, b) in &expected.seam_samples {
        assert!(map.edge(a, b).is_some(), "seam {a}/{b}");
    }
    assert_eq!(expected.polar, [6788, 18014]);
    for cap in expected.polar {
        let ci = map.province_index(cap).unwrap() as usize;
        let mut checked = 0;
        for edge in map.edges.iter().filter(|e| e.a == cap || e.b == cap) {
            let other = if edge.a == cap { edge.b } else { edge.a };
            let oi = map.province_index(other).unwrap() as usize;
            let dy = (map.centers[ci][1] - map.centers[oi][1]).abs();
            assert_eq!(
                edge.distance_km,
                dy.checked_mul(map.km_per_pixel).unwrap(),
                "polar dx=0 {cap}/{other}"
            );
            checked += 1;
        }
        assert!(checked > 0);
        eprintln!("polar {cap}: {checked} exact dx=0 distances");
    }
    assert_eq!(map.states.len(), 1);
    assert_eq!(map.states[0].provinces, land);
    eprintln!(
        "actual load_map PASS: {} provinces, {} edges, {elapsed:.3}s, {} warnings (not suppressed)",
        map.provinces.len(),
        map.edges.len(),
        map.warnings.len()
    );
}
