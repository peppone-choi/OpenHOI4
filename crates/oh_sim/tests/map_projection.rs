//! GWT1/GWT7. Scope extension approved by openhoi-lead in t_a8a76791.
use oh_core::{Fx, state_hash};
use oh_data::{map::MapProjection, national::LoadedNational};
use oh_sim::world::World;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

fn pack() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland")
}
fn hash(data: &LoadedNational) -> u64 {
    World::from_loaded(data).unwrap().definitions_hash()
}

// The pre-ADR-0702 tuple, independent of the production projection branch.
fn legacy_hash(data: &LoadedNational) -> u64 {
    assert!(data.military.is_none());
    let provinces: Vec<_> = data
        .map
        .provinces
        .iter()
        .map(|p| {
            (
                p.id,
                p.rgb,
                match p.kind {
                    oh_data::map::ProvinceKind::Land => 0u8,
                    oh_data::map::ProvinceKind::Sea => 1,
                    oh_data::map::ProvinceKind::Lake => 2,
                },
                &p.terrain,
                p.coastal,
                p.island,
            )
        })
        .collect();
    let states: Vec<_> = data
        .map
        .states
        .iter()
        .map(|s| (s.id, &s.name_key, &s.provinces))
        .collect();
    let edges: Vec<_> = data
        .map
        .edges
        .iter()
        .map(|e| {
            (
                e.a,
                e.b,
                match e.kind {
                    oh_data::map::EdgeKind::Normal => 0u8,
                    oh_data::map::EdgeKind::RiverSmall => 1,
                    oh_data::map::EdgeKind::RiverLarge => 2,
                    oh_data::map::EdgeKind::Strait => 3,
                    oh_data::map::EdgeKind::Impassable => 4,
                },
                e.distance_km,
            )
        })
        .collect();
    let vp: Vec<_> = data
        .map
        .victory_points
        .iter()
        .map(|v| (v.province, v.points, &v.name_key))
        .collect();
    state_hash(&(
        &data.nations,
        provinces,
        states,
        data.map.width,
        data.map.height,
        &data.map.index,
        &data.visuals,
        &data.scenario.map,
        &data.map.centers,
        edges,
        vp,
        data.map.km_per_pixel,
    ))
    .unwrap()
}

#[test]
fn uniform_preserves_pre_projection_definitions_tuple() {
    let data = oh_data::national::load_scenario(&pack(), "m1").unwrap();
    assert_eq!(data.map.projection, MapProjection::Uniform);
    assert_eq!(hash(&data), legacy_hash(&data));
}

#[test]
fn projection_identity_covers_every_extent_raw_bit_and_wrap() {
    for (pack_id, scenario) in [("testland", "m1"), ("testland_m2_military", "m2_military")] {
        let root = pack().parent().unwrap().join(pack_id);
        let mut data = oh_data::national::load_scenario(&root, scenario).unwrap();
        let uniform = hash(&data);
        let extent = [
            Fx::from_num(-180),
            Fx::from_num(-90),
            Fx::from_num(180),
            Fx::from_num(90),
        ];
        let projection =
            |[west, south, east, north]: [Fx; 4], wrap_x: bool| MapProjection::Equirectangular {
                west,
                south,
                east,
                north,
                wrap_x,
            };
        // Leave every other field byte-identical to isolate the projection identity.
        data.map.projection = projection(extent, true);
        let projected = hash(&data);
        assert_ne!(uniform, projected);
        assert_eq!(
            projected,
            state_hash(&(
                uniform,
                (
                    "equirectangular",
                    extent[0].to_bits(),
                    extent[1].to_bits(),
                    extent[2].to_bits(),
                    extent[3].to_bits(),
                    true
                )
            ))
            .unwrap()
        );
        for axis in 0..4 {
            let mut changed = extent;
            changed[axis] = Fx::from_bits(changed[axis].to_bits() + if axis < 2 { 1 } else { -1 });
            data.map.projection = projection(changed, true);
            assert_ne!(hash(&data), projected, "extent axis {axis}");
        }
        data.map.projection = projection(extent, false);
        assert_ne!(hash(&data), projected);
    }
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        fn copy(source: &Path, target: &Path) {
            fs::create_dir_all(target).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    copy(&entry.path(), &target.join(entry.file_name()));
                } else {
                    fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
                }
            }
        }
        let path = std::env::temp_dir().join(format!(
            "oh-world-projection-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy(&pack(), &path);
        fs::write(
            path.join("maps/testland/regions.toml"),
            "km_per_pixel = 2\nprojection = 'equirectangular'\nextent = [-180,-90,180,90]\n",
        )
        .unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn repeated_projected_pack_loads_have_identical_world_hash() {
    let f = Fixture::new();
    let a = oh_data::national::load_scenario(&f.0, "m1").unwrap();
    let b = oh_data::national::load_scenario(&f.0, "m1").unwrap();
    assert_eq!(a.map, b.map);
    assert_eq!(hash(&a), hash(&b));
    assert_ne!(
        hash(&a),
        hash(&oh_data::national::load_scenario(&pack(), "m1").unwrap())
    );
}
