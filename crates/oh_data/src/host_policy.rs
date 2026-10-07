//! Engine-owned REQUEST-0008 A. Input packs/flags cannot supply this policy.
use crate::{
    DataError, ErrorKind,
    pack_validation::{Diagnostic, DiagnosticCode, ResolvedPack, Severity, ValidationPurpose},
    raw::Source,
};
use schemars::{JsonSchema, Schema};
use serde::Deserialize;
use std::collections::BTreeSet;
pub const POLICY_TEXT: &str = include_str!("../policies/legacy-validation.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    Empty,
    National,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileFingerprint {
    pub path: String,
    pub bytes: u64,
    #[schemars(regex(pattern = "^[a-f0-9]{64}$"))]
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackFingerprint {
    pub content_hash: u64,
    pub files: Vec<FileFingerprint>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundPack {
    pub id: String,
    pub version: String,
    pub content_hash: u64,
}
/// Values copied from an actual bounded header by the host, before constructing authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderBinding {
    pub format_version: u16,
    pub scenario_id: String,
    pub has_world: bool,
    pub packs: Vec<BoundPack>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Caller {
    ServerStartup,
    ServerRestore,
    CliV1Resume,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PolicySource {
    id: String,
    #[schemars(regex(pattern = "^[a-f0-9]{40}$"))]
    source_commit: String,
    source_reference: String,
    files: Vec<FileFingerprint>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum AllowedCode {
    MissingMessageValue,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Disposition {
    WarningUnavailable,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Entry {
    policy_id: String,
    source_id: String,
    caller: Caller,
    pack_id: String,
    pack_version: String,
    #[schemars(regex(pattern = "^[a-f0-9]{16}$"))]
    pack_content_hash: String,
    scenario_id: String,
    scenario_kind: ScenarioKind,
    save_format: Option<u16>,
    source_field: String,
    name_key: String,
    diagnostic_code: AllowedCode,
    locales: Vec<String>,
    disposition: Disposition,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Policy {
    #[schemars(range(min = 1, max = 1))]
    policy_format: u16,
    sources: Vec<PolicySource>,
    entries: Vec<Entry>,
}
pub fn schema() -> Schema {
    schemars::schema_for!(Policy)
}
fn source(text: &str) -> Source {
    Source {
        path: std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("policies/legacy-validation.toml"),
        text: text.into(),
    }
}
fn error(s: &Source, msg: impl Into<String>) -> DataError {
    s.error(0, ErrorKind::Schema, msg)
}
fn hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn parse(text: &str) -> Result<Policy, DataError> {
    let s = source(text);
    s.document()?;
    let policy: Policy = toml::from_str(text).map_err(|e| {
        s.error(
            e.span().map_or(0, |p| p.start),
            ErrorKind::Schema,
            e.message(),
        )
    })?;
    if policy.policy_format != 1 {
        return Err(error(&s, "unsupported engine policy format"));
    }
    let mut sources = BTreeSet::new();
    for p in &policy.sources {
        if p.id.is_empty()
            || !sources.insert(&p.id)
            || !hex(&p.source_commit, 40)
            || p.source_reference.is_empty()
            || p.files.is_empty()
        {
            return Err(error(&s, "invalid source provenance/duplicate source ID"));
        }
        let mut previous = None;
        for f in &p.files {
            if f.path.is_empty()
                || f.path.contains(['\\', ':'])
                || f.path
                    .split('/')
                    .any(|p| p.is_empty() || p == "." || p == "..")
                || !hex(&f.sha256, 64)
                || previous.is_some_and(|p: &str| p >= f.path.as_str())
            {
                return Err(error(
                    &s,
                    "source files must have unique sorted relative POSIX paths and SHA256",
                ));
            }
            previous = Some(f.path.as_str());
        }
    }
    let mut ids = BTreeSet::new();
    let mut signatures = BTreeSet::new();
    for e in &policy.entries {
        if e.policy_id.is_empty()
            || !ids.insert(&e.policy_id)
            || !sources.contains(&e.source_id)
            || !crate::valid_id(&e.pack_id)
            || semver::Version::parse(&e.pack_version).is_err()
            || !hex(&e.pack_content_hash, 16)
            || !crate::valid_id(&e.scenario_id)
            || e.source_field != "manifest.name_key"
            || e.name_key != "testland_name"
            || e.locales.iter().collect::<BTreeSet<_>>()
                != BTreeSet::from([&"en".to_owned(), &"ko".to_owned()])
            || e.locales.len() != 2
        {
            return Err(error(&s, "invalid engine policy entry constraints"));
        }
        if (e.caller == Caller::ServerStartup && e.save_format.is_some())
            || (e.caller != Caller::ServerStartup && e.save_format != Some(1))
        {
            return Err(error(
                &s,
                "restore policy requires actual v1 header; startup has no save format",
            ));
        }
        if !signatures.insert(format!(
            "{:?}|{}|{}|{}|{}|{:?}|{:?}",
            e.caller,
            e.pack_id,
            e.pack_version,
            e.pack_content_hash,
            e.scenario_id,
            e.scenario_kind,
            e.save_format
        )) {
            return Err(error(&s, "duplicate engine policy constraints"));
        }
    }
    Ok(policy)
}

fn request(
    purpose: &ValidationPurpose,
) -> Option<(Caller, &str, ScenarioKind, Option<&HeaderBinding>)> {
    match purpose {
        ValidationPurpose::ServerStartup { scenario_id, kind } => {
            Some((Caller::ServerStartup, scenario_id, *kind, None))
        }
        ValidationPurpose::ServerRestore {
            scenario_id,
            kind,
            header,
        } => Some((Caller::ServerRestore, scenario_id, *kind, Some(header))),
        ValidationPurpose::CliV1Resume {
            scenario_id,
            kind,
            header,
        } => Some((Caller::CliV1Resume, scenario_id, *kind, Some(header))),
        _ => None,
    }
}
fn bound(e: &Entry, header: Option<&HeaderBinding>) -> bool {
    match (header, e.save_format) {
        (None, None) => true,
        (Some(h), Some(format)) => {
            h.format_version == format
                && h.scenario_id == e.scenario_id
                && h.has_world == (e.scenario_kind == ScenarioKind::National)
                && h.packs.len() == 1
                && h.packs[0].id == e.pack_id
                && h.packs[0].version == e.pack_version
                && format!("{:016x}", h.packs[0].content_hash) == e.pack_content_hash
        }
        _ => false,
    }
}
fn matching_diagnostic(e: &Entry, pack: &ResolvedPack, d: &Diagnostic) -> bool {
    matches!((&e.diagnostic_code,&d.code),(AllowedCode::MissingMessageValue,DiagnosticCode::MissingMessageValue{locale,key,source_field:Some(field)}) if e.locales.contains(locale)&&key==&e.name_key&&field==&e.source_field)
        && d.error.path == pack.root.join("manifest.toml")
        && d.severity == Severity::Error
}
fn apply_policy(
    policy: &Policy,
    pack: &ResolvedPack,
    purpose: &ValidationPurpose,
    diagnostics: &mut [Diagnostic],
) -> Result<(), DataError> {
    let Some((caller, scenario, kind, header)) = request(purpose) else {
        return Ok(());
    };
    for e in &policy.entries {
        if e.caller != caller
            || e.scenario_id != scenario
            || e.scenario_kind != kind
            || e.pack_id != pack.data.manifest.id
            || e.pack_version != pack.data.manifest.version
            || format!("{:016x}", pack.content_hash) != e.pack_content_hash
            || pack.data.manifest.name_key != e.name_key
            || !bound(e, header)
        {
            continue;
        }
        let p = policy
            .sources
            .iter()
            .find(|p| p.id == e.source_id)
            .expect("validated source reference");
        if p.files != pack.source_files {
            continue;
        }
        let matching: Vec<_> = diagnostics
            .iter()
            .enumerate()
            .filter(|(_, d)| matching_diagnostic(e, pack, d))
            .map(|(i, _)| i)
            .collect();
        if matching.len() != e.locales.len() {
            continue;
        }
        let locales: BTreeSet<_> = matching
            .iter()
            .filter_map(|i| match &diagnostics[*i].code {
                DiagnosticCode::MissingMessageValue { locale, .. } => Some(locale),
                _ => None,
            })
            .collect();
        if locales != e.locales.iter().collect() {
            continue;
        }
        for i in matching {
            let d = &mut diagnostics[i];
            match e.disposition {
                Disposition::WarningUnavailable => {
                    d.severity = Severity::Warning;
                    d.applied_policy = Some(e.policy_id.clone());
                    d.error.message = format!(
                        "metadata unavailable (engine policy {}): {}",
                        e.policy_id, d.error.message
                    );
                }
            }
        }
    }
    Ok(())
}
pub(crate) fn apply(
    pack: &ResolvedPack,
    purpose: &ValidationPurpose,
    diagnostics: &mut [Diagnostic],
) -> Result<(), DataError> {
    if request(purpose).is_none() {
        return Ok(());
    }
    apply_policy(&parse(POLICY_TEXT)?, pack, purpose, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack_validation::{resolve_packs, validate_packs};
    #[test]
    fn typed_causes_and_full_sha_proofs_cannot_be_replaced_by_message_text_or_fnv() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/packs/examples/m0/testland");
        let base = resolve_packs(std::slice::from_ref(&root))
            .unwrap()
            .remove(0);
        let diagnostics = validate_packs(&[root]).diagnostics;
        let purpose = ValidationPurpose::ServerStartup {
            scenario_id: "testland".into(),
            kind: ScenarioKind::Empty,
        };
        let policy = parse(POLICY_TEXT).unwrap();
        for i in 0..7 {
            let mut p = base.clone();
            let mut d = diagnostics.clone();
            match i {
                0 => p.source_files[0].sha256.replace_range(0..1, "0"),
                1 => p.source_files[0].bytes += 1,
                2 => p.source_files[0].path = "other.toml".into(),
                3 => d[0].code = DiagnosticCode::Data,
                4 => {
                    if let DiagnosticCode::MissingMessageValue { locale, .. } = &mut d[0].code {
                        *locale = "fr".into();
                    }
                }
                5 => {
                    if let DiagnosticCode::MissingMessageValue { source_field, .. } = &mut d[0].code
                    {
                        *source_field = Some("nation.name_key".into());
                    }
                }
                _ => {
                    if let DiagnosticCode::MissingMessageValue { key, .. } = &mut d[0].code {
                        *key = "another-key".into();
                    }
                }
            }
            assert_eq!(p.content_hash, base.content_hash);
            apply_policy(&policy, &p, &purpose, &mut d).unwrap();
            assert!(
                d.iter().all(|d| d.applied_policy.is_none()),
                "condition {i}"
            );
        }
        let mut d = diagnostics.clone();
        let mut other = d[0].clone();
        other.code = DiagnosticCode::Data;
        other.error.message = "unrelated schema error".into();
        d.push(other);
        apply_policy(&policy, &base, &purpose, &mut d).unwrap();
        assert_eq!(d[2].severity, Severity::Error);
        assert!(d[2].applied_policy.is_none());
    }
    #[test]
    fn engine_policy_rejects_malformed_unknown_format_path_duplicate_and_locale() {
        for text in [
            POLICY_TEXT.replace("policy_format = 1", "policy_format = 2"),
            POLICY_TEXT.replace("path = \"defines.toml\"", "path = \"../defines.toml\""),
            POLICY_TEXT.replace("locales = [\"ko\", \"en\"]", "locales = [\"ko\", \"ko\"]"),
            format!("unknown = 1\n{POLICY_TEXT}"),
            POLICY_TEXT.replace(
                "policy_id = \"original-m0-server_restore\"",
                "policy_id = \"original-m0-server_startup\"",
            ),
        ] {
            assert!(parse(&text).is_err());
        }
    }
}
