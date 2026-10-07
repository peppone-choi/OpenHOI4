//! Static client key inputs are collected outside the Rust Fluent parser.
use oh_data::{
    DataError, ErrorKind,
    localisation::{KeyUse, validate_catalogs},
    pack_validation::Severity,
};
#[derive(serde::Deserialize)]
struct Usage {
    key: String,
    path: std::path::PathBuf,
    line: usize,
    column: usize,
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("catalog_check <ko.ftl> <en.ftl> <uses.json>".into());
    }
    let uses: Vec<Usage> =
        serde_json::from_str(&std::fs::read_to_string(&args[2]).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let uses: Vec<_> = uses
        .into_iter()
        .map(|u| KeyUse {
            key: u.key,
            context: DataError {
                path: u.path,
                line: u.line,
                column: u.column,
                kind: ErrorKind::Schema,
                message: "client key use".into(),
            },
        })
        .collect();
    let diagnostics = validate_catalogs(
        &[args[0].clone().into()],
        &[args[1].clone().into()],
        &uses,
        true,
    )
    .map_err(|e| e.to_string())?;
    for d in &diagnostics {
        eprintln!(
            "{}: {}",
            if d.severity == Severity::Error {
                "error"
            } else {
                "warning"
            },
            d.error
        );
    }
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return Err("localisation check failed".into());
    }
    Ok(())
}
