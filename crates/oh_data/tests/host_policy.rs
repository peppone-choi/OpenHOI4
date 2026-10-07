use oh_data::{
    host_policy::{BoundPack, HeaderBinding, ScenarioKind},
    pack_validation::{
        ValidationPurpose as Purpose, fingerprint, validate_for_purpose, verify_source,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
fn original() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/examples/m0/testland")
}
fn purpose() -> Purpose {
    Purpose::ServerStartup {
        scenario_id: "testland".into(),
        kind: ScenarioKind::Empty,
    }
}
fn binding() -> HeaderBinding {
    HeaderBinding {
        format_version: 1,
        scenario_id: "testland".into(),
        has_world: false,
        packs: vec![BoundPack {
            id: "m0_testland".into(),
            version: "0.1.0".into(),
            content_hash: 8_747_082_838_669_556_868,
        }],
    }
}
fn clone_pack() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/evidence/WP-24-P06/policy-fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
    copy(&original(), &p);
    p
}
fn copy(a: &Path, b: &Path) {
    fs::create_dir_all(b).unwrap();
    for e in fs::read_dir(a).unwrap() {
        let p = e.unwrap().path();
        let t = b.join(p.file_name().unwrap());
        if p.is_dir() {
            copy(&p, &t)
        } else {
            fs::copy(p, t).unwrap();
        }
    }
}
#[test]
fn approved_callers_keep_exact_context_and_typed_warning() {
    let root = original();
    for p in [
        purpose(),
        Purpose::ServerRestore {
            scenario_id: "testland".into(),
            kind: ScenarioKind::Empty,
            header: binding(),
        },
        Purpose::CliV1Resume {
            scenario_id: "testland".into(),
            kind: ScenarioKind::Empty,
            header: binding(),
        },
    ] {
        let r = validate_for_purpose(std::slice::from_ref(&root), p);
        assert!(!r.failed(false), "{:?}", r.diagnostics);
        assert!(r.failed(true));
        assert_eq!(r.diagnostics.len(), 2);
        assert!(r.diagnostics.iter().all(
            |d| d.applied_policy.is_some() && d.error.message.contains("metadata unavailable")
        ));
        assert_eq!(r.packs[0].source_files.len(), 3);
    }
    for p in [
        Purpose::Strict,
        Purpose::Active,
        Purpose::RestoreV1,
        Purpose::ServerStartup {
            scenario_id: "other".into(),
            kind: ScenarioKind::Empty,
        },
        Purpose::ServerStartup {
            scenario_id: "testland".into(),
            kind: ScenarioKind::National,
        },
    ] {
        assert!(validate_for_purpose(std::slice::from_ref(&root), p).failed(false));
    }
    for i in 0..7 {
        let mut h = binding();
        match i {
            0 => h.format_version = 2,
            1 => h.scenario_id = "other".into(),
            2 => h.has_world = true,
            3 => h.packs[0].id = "other".into(),
            4 => h.packs[0].version = "0.2.0".into(),
            5 => h.packs[0].content_hash ^= 1,
            _ => h.packs.push(h.packs[0].clone()),
        };
        let p = Purpose::CliV1Resume {
            scenario_id: "testland".into(),
            kind: ScenarioKind::Empty,
            header: h,
        };
        assert!(
            validate_for_purpose(std::slice::from_ref(&root), p).failed(false),
            "condition {i}"
        );
    }
}
#[test]
fn full_source_set_sha_lengths_bytes_and_unrelated_errors_are_not_exempted() {
    for i in 0..8 {
        let root = clone_pack();
        let before = fingerprint(&root).unwrap();
        match i {
            0 => {
                let p = root.join("defines.toml");
                let mut b = fs::read(&p).unwrap();
                b[0] = b';';
                fs::write(p, b).unwrap();
            }
            1 => {
                fs::write(root.join("extra.txt"), "extra").unwrap();
            }
            2 => {
                fs::remove_file(root.join("defines.toml")).unwrap();
            }
            3 => {
                let p = root.join("manifest.toml");
                let t = fs::read_to_string(&p).unwrap();
                fs::write(p, t.replace("testland_name", "other_name")).unwrap();
            }
            4 => {
                let p = root.join("manifest.toml");
                let t = fs::read_to_string(&p).unwrap();
                fs::write(p, t.replace("m0_testland", "other_pack")).unwrap();
            }
            5 => {
                fs::create_dir_all(root.join("localisation/en")).unwrap();
                fs::write(root.join("localisation/en/broken.ftl"), "broken = {\n").unwrap();
            }
            6 => {
                let p = root.join("manifest.toml");
                let t = fs::read_to_string(&p).unwrap();
                fs::write(
                    p,
                    t.replace("depends = []", "depends = [{ id='missing', version='*' }]"),
                )
                .unwrap();
            }
            _ => {
                let p = root.join("scenarios/testland/scenario.toml");
                fs::write(p, "start_date='2000-01-01'\nunknown='missing'\n").unwrap();
            }
        }
        assert!(verify_source(&root, before.content_hash, &before.files).is_err());
        let r = validate_for_purpose(&[root], purpose());
        assert!(r.failed(false), "mutation {i}");
        assert!(r.diagnostics.iter().all(|d| d.applied_policy.is_none()));
    }
}
#[test]
fn policy_schema_snapshot_and_manifest_crypto_vectors() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("schema/legacy-validation.schema.json");
    let expected: serde_json::Value = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(oh_data::host_policy::schema()).unwrap(),
        expected
    );
    use sha2::{Digest, Sha256};
    for (input, expected) in [
        (
            b"".as_slice(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc".as_slice(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
    ] {
        let h: String = Sha256::digest(input)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(h, expected);
    }
    let p = clone_pack();
    #[cfg(windows)]
    {
        let base = p
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_owned();
        let p = PathBuf::from(base);
        let target = p.join("target");
        fs::create_dir(&target).unwrap();
        let junction = p.join("link");
        let status = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(&junction)
            .arg(&target)
            .status()
            .unwrap();
        assert!(status.success());
        assert!(fingerprint(&p).is_err());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(p.join("defines.toml"), p.join("link")).unwrap();
        assert!(fingerprint(&p).is_err());
    }
}
