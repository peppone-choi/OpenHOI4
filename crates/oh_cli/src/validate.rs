//! CLI diagnostics are host output, not game UI strings.
use oh_data::pack_validation::{Severity, ValidationReport};
use std::path::PathBuf;

pub fn execute(args: &[String]) -> Result<ValidationReport, String> {
    let mut roots = Vec::new();
    let mut deny = false;
    for arg in &args[1..] {
        if arg == "--deny-warnings" {
            if deny {
                return Err("duplicate --deny-warnings".into());
            }
            deny = true;
        } else if arg.starts_with('-') {
            return Err(format!("unknown validate option {arg}"));
        } else {
            roots.push(PathBuf::from(arg));
        }
    }
    if roots.is_empty() {
        return Err("usage: oh_cli validate [--deny-warnings] <pack-root>...".into());
    }
    let report = oh_data::pack_validation::validate_packs(&roots);
    for d in &report.diagnostics {
        eprintln!(
            "{}: {}",
            match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            d.error
        );
    }
    for p in &report.packs {
        println!(
            "pack={} version={} hash={:016x}",
            p.data.manifest.id, p.data.manifest.version, p.content_hash
        );
    }
    println!("maps={} scenarios={}", report.maps, report.scenarios);
    if report.failed(deny) {
        return Err("validation failed".into());
    }
    Ok(report)
}
