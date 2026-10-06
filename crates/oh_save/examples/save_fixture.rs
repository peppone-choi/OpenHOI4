//! Native test executable. This is fixture construction, not gameplay/UI input.
#[path = "../tests/support/mod.rs"]
mod support;
use oh_core::{NationId, canonical_bytes};
use oh_save::{Limits, SaveContext, decode, encode, read_file, write_atomic};
use oh_sim::Command;
use std::path::Path;
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("capture") if args.len() == 2 => {
            let out = Path::new(&args[1]);
            std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
            let root = out.join("pack");
            support::mutable_pack(&root);
            let c = SaveContext::national(&root, "m1")?;
            let mut continuous = support::scheduled(&c);
            support::advance(&mut continuous, 96);
            support::assert_expiry(&continuous, 12884901888);
            let mut split = support::scheduled(&c);
            support::advance(&mut split, 48);
            support::assert_expiry(&split, 12884901890);
            let bytes = encode(&split, &c, 0, vec![])?;
            write_atomic(&out.join("saved.ohsave"), &bytes, &c)?;
            let mut resumed = decode(&bytes, &c, false)?.simulation;
            support::advance(&mut resumed, 48);
            assert_eq!(
                continuous.state_hash().unwrap(),
                resumed.state_hash().unwrap()
            );
            let snapshot = split.export_save()?;
            let report = serde_json::json!({"fixture":"actual-m1-expiry-v1","pid":std::process::id(),"split":snapshot,"canonical_hex":hex(&canonical_bytes(&split).unwrap()),"split_hash":format!("{:016x}",split.state_hash().unwrap()),"continuous_hash":format!("{:016x}",continuous.state_hash().unwrap()),"continuous":continuous.export_save()?,"pack":c.pack()});
            std::fs::write(
                out.join("capture.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .map_err(|e| e.to_string())?;
            let mut paused = split.clone();
            paused
                .enqueue(48, NationId(65535), 20, Command::Pause(true))
                .unwrap();
            assert!(!paused.step().unwrap().advanced);
            write_atomic(
                &out.join("paused.ohsave"),
                &encode(&paused, &c, 0, vec![])?,
                &c,
            )?;
            println!("{}", serde_json::to_string(&report).unwrap());
        }
        Some("resume") if args.len() == 3 => {
            let c = SaveContext::national(Path::new(&args[1]), "m1")?;
            let mut loaded = decode(
                &read_file(Path::new(&args[2]), &Limits::default())?,
                &c,
                false,
            )?
            .simulation;
            let initial = loaded.export_save()?;
            let initial_hash = loaded.state_hash().map_err(|e| e.to_string())?;
            support::advance(&mut loaded, 48);
            support::assert_expiry(&loaded, 12884901888);
            println!(
                "{}",
                serde_json::json!({"pid":std::process::id(),"initial":initial,"initial_hash":format!("{initial_hash:016x}"),"resumed_hash":format!("{:016x}",loaded.state_hash().unwrap()),"resumed":loaded.export_save()?})
            );
        }
        _ => return Err("save_fixture capture <out> | resume <pack> <file>".into()),
    }
    Ok(())
}
