//! Shared root -> scenario numeric overlay and registered runtime config contracts.
use crate::{DataError, DefineValue, Defines, ErrorKind, Number, raw::Source, read};
use std::path::Path;

pub fn load(root: &Path, id: &str, base: &Defines, required: bool) -> Result<Defines, DataError> {
    if !crate::valid_id(id) {
        return Err(DataError {
            path: root.join("scenarios"),
            line: 1,
            column: 1,
            kind: ErrorKind::Schema,
            message: "scenario ID must match [a-z0-9_]+".into(),
        });
    }
    let path = root.join("scenarios").join(id).join("defines.toml");
    let mut effective = base.clone();
    let overlay = if required || path.exists() {
        Some(read(&path)?)
    } else {
        None
    };
    if let Some(source) = &overlay {
        let defines = crate::parse_defines(source)?;
        for (system, items) in defines.0 {
            effective.0.entry(system).or_default().extend(items);
        }
    }
    let base = read(&root.join("defines.toml"))?;
    validate(&effective, &base, overlay.as_ref())?;
    Ok(effective)
}
/// Server's existing five mandatory transport settings, with source context.
pub fn require_network(root: &Path, id: &str, defines: &Defines) -> Result<(), DataError> {
    let base = read(&root.join("defines.toml"))?;
    let path = root.join("scenarios").join(id).join("defines.toml");
    let overlay = if path.exists() {
        Some(read(&path)?)
    } else {
        None
    };
    for item in [
        "delta_ms",
        "command_capacity",
        "max_message_bytes",
        "handshake_timeout_ms",
        "default_seed",
    ] {
        if !matches!(defines.get(&format!("network.{item}")),Some(DefineValue::Number(Number::Integer(n))) if *n>0)
        {
            return Err(error(
                &base,
                overlay.as_ref(),
                "network",
                item,
                "server requires a positive i64 scalar",
            ));
        }
    }
    Ok(())
}
fn source_for<'a>(
    base: &'a Source,
    overlay: Option<&'a Source>,
    system: &str,
    item: &str,
) -> &'a Source {
    overlay
        .filter(|s| {
            s.document().ok().is_some_and(|doc| {
                doc.get(system).is_some_and(
                    |v| matches!(v.get_ref(),toml::de::DeValue::Table(t) if t.contains_key(item)),
                )
            })
        })
        .unwrap_or(base)
}
fn error(
    base: &Source,
    overlay: Option<&Source>,
    system: &str,
    item: &str,
    message: &str,
) -> DataError {
    let source = source_for(base, overlay, system, item);
    let offset = source
        .document()
        .ok()
        .and_then(|d| {
            d.get(system).map(|v| {
                if let toml::de::DeValue::Table(t) = v.get_ref() {
                    t.get(item).map_or(v.span().start, |v| v.span().start)
                } else {
                    v.span().start
                }
            })
        })
        .unwrap_or(0);
    source.error(
        offset,
        ErrorKind::Schema,
        format!("{system}.{item}: {message}"),
    )
}
fn validate(defines: &Defines, base: &Source, overlay: Option<&Source>) -> Result<(), DataError> {
    for (system, allowed) in [
        ("time", ["speed_ms_per_tick", "initial_speed"].as_slice()),
        (
            "network",
            [
                "delta_ms",
                "command_capacity",
                "max_message_bytes",
                "handshake_timeout_ms",
                "default_seed",
            ]
            .as_slice(),
        ),
    ] {
        if let Some(items) = defines.0.get(system) {
            for item in items.keys() {
                if !allowed.contains(&item.as_str()) {
                    return Err(error(base, overlay, system, item, "unknown registered key"));
                }
            }
        }
    }
    match defines.get("time.speed_ms_per_tick") {
        Some(DefineValue::Array(values))
            if values.len() == 5
                && values
                    .iter()
                    .all(|v| matches!(v,Number::Integer(n) if *n>=0)) => {}
        _ => {
            return Err(error(
                base,
                overlay,
                "time",
                "speed_ms_per_tick",
                "required array of exactly five nonnegative i64 integers",
            ));
        }
    }
    if !matches!(defines.get("time.initial_speed"),Some(DefineValue::Number(Number::Integer(n))) if (1..=5).contains(n))
    {
        return Err(error(
            base,
            overlay,
            "time",
            "initial_speed",
            "required i64 scalar in 1..=5",
        ));
    }
    if let Some(items) = defines.0.get("network") {
        for (item, value) in items {
            let min = if item == "delta_ms" { 100 } else { 1 };
            if !matches!(value,DefineValue::Number(Number::Integer(n)) if *n>=min) {
                return Err(error(
                    base,
                    overlay,
                    "network",
                    item,
                    if item == "delta_ms" {
                        "must be an i64 scalar >=100"
                    } else {
                        "must be a positive i64 scalar"
                    },
                ));
            }
        }
    }
    Ok(())
}
