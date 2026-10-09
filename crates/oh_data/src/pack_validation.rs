//! Host-only pack selection, deterministic ordering and registered content validation.
use crate::{DataError, DataPack, ErrorKind, load_pack, raw::Source, read};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}
/// Explicit host purpose, never selected by --force or inferred from directory names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationPurpose {
    Strict,
    Active,
    RestoreV1,
    ServerStartup {
        scenario_id: String,
        kind: crate::host_policy::ScenarioKind,
    },
    ServerRestore {
        scenario_id: String,
        kind: crate::host_policy::ScenarioKind,
        header: crate::host_policy::HeaderBinding,
    },
    CliV1Resume {
        scenario_id: String,
        kind: crate::host_policy::ScenarioKind,
        header: crate::host_policy::HeaderBinding,
    },
}
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub error: DataError,
    pub code: DiagnosticCode,
    pub applied_policy: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticCode {
    Data,
    MissingMessageValue {
        locale: String,
        key: String,
        source_field: Option<String>,
    },
}
#[derive(Debug, Clone)]
pub struct ResolvedPack {
    pub root: PathBuf,
    pub data: DataPack,
    pub content_hash: u64,
    pub source_files: Vec<crate::host_policy::FileFingerprint>,
}
#[derive(Debug, Default)]
pub struct ValidationReport {
    pub packs: Vec<ResolvedPack>,
    pub diagnostics: Vec<Diagnostic>,
    pub maps: usize,
    pub scenarios: usize,
}
impl ValidationReport {
    pub fn failed(&self, deny_warnings: bool) -> bool {
        self.diagnostics
            .iter()
            .any(|d| deny_warnings || d.severity == Severity::Error)
    }
    fn error(&mut self, e: DataError) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            error: e,
            code: DiagnosticCode::Data,
            applied_policy: None,
        });
    }
    pub(crate) fn warning(&mut self, e: DataError) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            error: e,
            code: DiagnosticCode::Data,
            applied_policy: None,
        });
    }
}
fn io(path: &Path, e: impl std::fmt::Display) -> DataError {
    DataError {
        path: path.into(),
        line: 1,
        column: 1,
        kind: ErrorKind::Io,
        message: e.to_string(),
    }
}
fn regular(path: &Path) -> Result<fs::Metadata, DataError> {
    let m = fs::symlink_metadata(path).map_err(|e| io(path, e))?;
    #[cfg(windows)]
    let reparse = {
        use std::os::windows::fs::MetadataExt;
        m.file_attributes() & 0x400 != 0
    };
    #[cfg(not(windows))]
    let reparse = false;
    if m.file_type().is_symlink() || reparse || (!m.is_file() && !m.is_dir()) {
        return Err(io(
            path,
            "pack paths must be regular files/directories; symlink/reparse unsupported",
        ));
    }
    Ok(m)
}
/// Same sorted relative-path/byte postcard hash as the existing save pack identity.
fn collect_files(root: &Path) -> Result<Vec<(String, Vec<u8>)>, DataError> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<(), DataError> {
        for entry in fs::read_dir(dir).map_err(|e| io(dir, e))? {
            let p = entry.map_err(|e| io(dir, e))?.path();
            let m = regular(&p)?;
            if m.is_dir() {
                walk(root, &p, out)?;
            } else {
                let rel = p
                    .strip_prefix(root)
                    .map_err(|e| io(&p, e))?
                    .components()
                    .map(|c| {
                        c.as_os_str()
                            .to_str()
                            .ok_or_else(|| io(&p, "non-UTF8 pack path"))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join("/");
                out.push((rel, fs::read(&p).map_err(|e| io(&p, e))?));
            }
        }
        Ok(())
    }
    if !regular(root)?.is_dir() {
        return Err(io(root, "pack root must be a directory"));
    }
    let mut files = Vec::new();
    walk(root, root, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}
pub fn content_hash(root: &Path) -> Result<u64, DataError> {
    oh_core::state_hash(&collect_files(root)?).map_err(|e| io(root, e))
}
pub fn fingerprint(root: &Path) -> Result<crate::host_policy::PackFingerprint, DataError> {
    use sha2::{Digest, Sha256};
    let files = collect_files(root)?;
    let content_hash = oh_core::state_hash(&files).map_err(|e| io(root, e))?;
    let files = files
        .into_iter()
        .map(|(path, data)| crate::host_policy::FileFingerprint {
            path,
            bytes: data.len() as u64,
            sha256: Sha256::digest(data)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
        })
        .collect();
    Ok(crate::host_policy::PackFingerprint {
        content_hash,
        files,
    })
}
pub fn verify_source(
    root: &Path,
    hash: u64,
    files: &[crate::host_policy::FileFingerprint],
) -> Result<(), DataError> {
    let actual = fingerprint(root)?;
    if actual.content_hash != hash || actual.files != files {
        return Err(io(
            root,
            "pack changed after validation (full file/SHA256 identity)",
        ));
    }
    Ok(())
}
fn at(
    source: &Source,
    field: &str,
    index: Option<usize>,
    sub: Option<&str>,
    message: impl Into<String>,
) -> DataError {
    let offset = source
        .document()
        .ok()
        .and_then(|d| {
            d.get(field).map(|v| {
                if let Some(i) = index
                    && let toml::de::DeValue::Array(a) = v.get_ref()
                    && let Some(e) = a.get(i)
                {
                    if let Some(sub) = sub
                        && let toml::de::DeValue::Table(t) = e.get_ref()
                    {
                        return t.get(sub).map_or(e.span().start, |v| v.span().start);
                    }
                    return e.span().start;
                }
                v.span().start
            })
        })
        .unwrap_or(0);
    source.error(offset, ErrorKind::Schema, message)
}
/// Returns no partial load result on error. Edges are deduplicated across depends/load_after.
pub fn resolve_packs(roots: &[PathBuf]) -> Result<Vec<ResolvedPack>, DataError> {
    if roots.is_empty() {
        return Err(io(
            Path::new("manifest.toml"),
            "at least one selected pack is required",
        ));
    }
    let mut entries = BTreeMap::new();
    let engine = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|e| io(Path::new("manifest.toml"), e))?;
    for root in roots {
        let stamp = fingerprint(root)?;
        let data = load_pack(root)?;
        let source = read(&root.join("manifest.toml"))?;
        if !semver::VersionReq::parse(&data.manifest.engine)
            .map_err(|e| at(&source, "engine", None, None, e.to_string()))?
            .matches(&engine)
        {
            return Err(at(
                &source,
                "engine",
                None,
                None,
                format!(
                    "engine {} is incompatible with {engine}",
                    data.manifest.engine
                ),
            ));
        }
        let id = data.manifest.id.clone();
        if entries.contains_key(&id) {
            return Err(at(
                &source,
                "id",
                None,
                None,
                format!("duplicate pack ID {id}"),
            ));
        }
        entries.insert(
            id,
            (
                ResolvedPack {
                    root: root.clone(),
                    data,
                    content_hash: stamp.content_hash,
                    source_files: stamp.files,
                },
                source,
            ),
        );
    }
    let mut predecessors: BTreeMap<String, BTreeSet<String>> = entries
        .keys()
        .map(|k| (k.clone(), BTreeSet::new()))
        .collect();
    for (id, (pack, source)) in &entries {
        let m = &pack.data.manifest;
        let mut seen = BTreeSet::new();
        for (i, dep) in m.depends.iter().enumerate() {
            if &dep.id == id || !seen.insert(&dep.id) {
                return Err(at(
                    source,
                    "depends",
                    Some(i),
                    Some("id"),
                    "duplicate or self dependency",
                ));
            }
            let Some((target, _)) = entries.get(&dep.id) else {
                return Err(at(
                    source,
                    "depends",
                    Some(i),
                    Some("id"),
                    format!("missing dependency {}", dep.id),
                ));
            };
            let version = semver::Version::parse(&target.data.manifest.version)
                .map_err(|e| at(source, "depends", Some(i), Some("version"), e.to_string()))?;
            if !semver::VersionReq::parse(&dep.version)
                .map_err(|e| at(source, "depends", Some(i), Some("version"), e.to_string()))?
                .matches(&version)
            {
                return Err(at(
                    source,
                    "depends",
                    Some(i),
                    Some("version"),
                    format!(
                        "dependency {} version {version} does not match {}",
                        dep.id, dep.version
                    ),
                ));
            }
            predecessors
                .get_mut(id)
                .expect("selected ID")
                .insert(dep.id.clone());
        }
        for (field, values) in [("conflicts", &m.conflicts), ("load_after", &m.load_after)] {
            let mut seen = BTreeSet::new();
            for (i, target) in values.iter().enumerate() {
                if target == id || !seen.insert(target) {
                    return Err(at(
                        source,
                        field,
                        Some(i),
                        None,
                        format!("duplicate or self {field}"),
                    ));
                }
                if field == "conflicts" {
                    if entries.contains_key(target) {
                        return Err(at(
                            source,
                            field,
                            Some(i),
                            None,
                            format!("pack {id} conflicts with {target}"),
                        ));
                    }
                } else {
                    if !entries.contains_key(target) {
                        return Err(at(
                            source,
                            field,
                            Some(i),
                            None,
                            format!("missing load_after pack {target}"),
                        ));
                    }
                    predecessors
                        .get_mut(id)
                        .expect("selected ID")
                        .insert(target.clone());
                }
            }
        }
    }
    let mut order = Vec::new();
    while !predecessors.is_empty() {
        let next = predecessors
            .iter()
            .find(|(_, deps)| deps.is_empty())
            .map(|(id, _)| id.clone());
        let Some(next) = next else {
            let (id, deps) = predecessors.first_key_value().expect("nonempty graph");
            let (pack, source) = &entries[id];
            if let Some(i) = pack
                .data
                .manifest
                .depends
                .iter()
                .position(|d| deps.contains(&d.id))
            {
                return Err(at(
                    source,
                    "depends",
                    Some(i),
                    Some("id"),
                    "pack order cycle (dependency/load_after)",
                ));
            }
            let i = pack
                .data
                .manifest
                .load_after
                .iter()
                .position(|d| deps.contains(d));
            return Err(at(
                source,
                "load_after",
                i,
                None,
                "pack order cycle (dependency/load_after)",
            ));
        };
        predecessors.remove(&next);
        for deps in predecessors.values_mut() {
            deps.remove(&next);
        }
        order.push(next);
    }
    Ok(order
        .into_iter()
        .map(|id| entries.remove(&id).expect("selected ID").0)
        .collect())
}
fn children(root: &Path) -> Result<Vec<PathBuf>, DataError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut result = fs::read_dir(root)
        .map_err(|e| io(root, e))?
        .map(|e| e.map(|e| e.path()).map_err(|e| io(root, e)))
        .collect::<Result<Vec<_>, _>>()?;
    result.sort();
    Ok(result)
}
pub fn validate_packs(roots: &[PathBuf]) -> ValidationReport {
    validate_for_purpose(roots, ValidationPurpose::Strict)
}
pub fn validate_for_purpose(roots: &[PathBuf], purpose: ValidationPurpose) -> ValidationReport {
    let mut report = ValidationReport::default();
    let packs = match resolve_packs(roots) {
        Ok(p) => p,
        Err(e) => {
            report.error(e);
            return report;
        }
    };
    validate_resolved(packs, purpose)
}
fn validate_resolved(packs: Vec<ResolvedPack>, purpose: ValidationPurpose) -> ValidationReport {
    let mut report = ValidationReport::default();
    for pack in &packs {
        if let Err(e) = validate_content(pack, &mut report, &purpose) {
            report.error(e);
        }
        match fingerprint(&pack.root) {
            Ok(s) if s.content_hash == pack.content_hash && s.files == pack.source_files => {}
            Ok(_) => report.error(io(&pack.root, "pack changed during validation")),
            Err(e) => report.error(e),
        }
    }
    if !report.failed(false) {
        report.packs = packs;
    }
    report
}
fn validate_content(
    pack: &ResolvedPack,
    report: &mut ValidationReport,
    purpose: &ValidationPurpose,
) -> Result<(), DataError> {
    let root = &pack.root;
    let mut keys = crate::localisation::keys_in_toml(&root.join("manifest.toml"))?;
    for (file, kind) in [
        ("terrain.toml", "terrain"),
        ("resources.toml", "resource"),
        ("buildings.toml", "building"),
    ] {
        let p = root.join("common").join(file);
        if p.exists() {
            crate::map::registry(&p, kind)?;
        }
    }
    for p in children(&root.join("maps"))? {
        let id = p
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| io(&p, "non-UTF8 map ID"))?;
        let map = crate::map::load_map(root, id)?;
        report.maps += 1;
        keys.extend(crate::localisation::keys_in_toml(&p.join("states.toml"))?);
        let vp = p.join("visuals.toml");
        if vp.exists() {
            crate::national::validate_visuals(&vp, &map)?;
        }
        for w in map.warnings {
            report.warning(DataError {
                path: w.path,
                line: w.line,
                column: w.column,
                kind: ErrorKind::Schema,
                message: format!("province {}: {}", w.province, w.message),
            });
        }
    }
    let mut economic_files = BTreeSet::new();
    let mut production_files = BTreeSet::new();
    let mut military_files = BTreeSet::new();
    for p in children(&root.join("scenarios"))? {
        let id = p
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| io(&p, "non-UTF8 scenario ID"))?;
        let source = read(&p.join("scenario.toml"))?;
        let doc = source.document()?;
        if doc.contains_key("map") {
            let national = crate::national::read_scenario(root, id)?;
            validate_date(&source, &national.scenario.start_date)?;
            if let Some(id) = &national.scenario.military {
                military_files.insert(root.join("common/military").join(format!("{id}.toml")));
            }
            if let Some(id) = &national.scenario.production {
                let path = root.join("common/production").join(format!("{id}.toml"));
                keys.extend(crate::localisation::keys_in_toml(&path)?);
                production_files.insert(path);
            }
            if let Some(id) = &national.scenario.economy {
                let path = root.join("common/economy").join(format!("{id}.toml"));
                keys.extend(crate::localisation::keys_in_toml(&path)?);
                economic_files.insert(path);
            }
            for file in children(&p.join("nations"))? {
                if !file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| national.scenario.nations.iter().any(|tag| tag == s))
                {
                    reject_unselected_nation(&file)?;
                }
                keys.extend(crate::localisation::keys_in_toml(&file)?);
            }
        } else {
            #[derive(serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Legacy {
                start_date: String,
            }
            let legacy: Legacy = toml::from_str(&source.text).map_err(|e| {
                source.error(
                    e.span().map_or(0, |s| s.start),
                    ErrorKind::Schema,
                    e.message(),
                )
            })?;
            validate_date(&source, &legacy.start_date)?;
            crate::scenario_defines::load(root, id, &pack.data.defines, false)?;
            for file in children(&p.join("nations"))? {
                if file.extension().is_some_and(|e| e == "toml") {
                    reject_unselected_nation(&file)?;
                }
            }
        }
        report.scenarios += 1;
    }
    for file in children(&root.join("common/economy"))? {
        if !economic_files.contains(&file) {
            return Err(io(
                &file,
                "unregistered economy file: explicit scenario reference required",
            ));
        }
    }
    for file in children(&root.join("common/production"))? {
        if !production_files.contains(&file) {
            return Err(io(
                &file,
                "unregistered production file: explicit scenario reference required",
            ));
        }
    }
    for file in children(&root.join("common/military"))? {
        if !military_files.contains(&file) {
            return Err(io(
                &file,
                "unregistered military file: explicit scenario reference required",
            ));
        }
    }
    unsupported_files(root, root, report)?;
    let mut diagnostics =
        crate::localisation::validate_directory(&root.join("localisation"), &keys)?;
    crate::host_policy::apply(pack, purpose, &mut diagnostics)?;
    report.diagnostics.extend(diagnostics);
    Ok(())
}
fn reject_unselected_nation(path: &Path) -> Result<(), DataError> {
    let tag = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| io(path, "invalid nation filename"))?;
    let nation = crate::national::read_nation(path, tag)?;
    crate::national::validate_nation_names(path, &nation)?;
    crate::national::validate_nation_ideology(path, &nation)?;
    let source = read(path)?;
    Err(at(
        &source,
        "capital",
        None,
        None,
        "unregistered nation file: not selected by this scenario; explicit map/nations context required for references",
    ))
}
fn unsupported_files(
    root: &Path,
    dir: &Path,
    report: &mut ValidationReport,
) -> Result<(), DataError> {
    for p in children(dir)? {
        if regular(&p)?.is_dir() {
            unsupported_files(root, &p, report)?;
            continue;
        }
        if !p
            .extension()
            .is_some_and(|s| ["toml", "ftl", "csv", "png"].iter().any(|e| s == *e))
        {
            continue;
        }
        let relative = p
            .strip_prefix(root)
            .map_err(|e| io(&p, e))?
            .components()
            .map(|c| {
                c.as_os_str()
                    .to_str()
                    .ok_or_else(|| io(&p, "non-UTF8 path"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let registered = match relative.as_slice() {
            ["manifest.toml" | "defines.toml"] => true,
            [
                "common",
                "terrain.toml" | "resources.toml" | "buildings.toml",
            ] => true,
            [
                "maps",
                _,
                "regions.toml"
                | "states.toml"
                | "provinces.csv"
                | "provinces.png"
                | "adjacency_overrides.csv"
                | "visuals.toml",
            ] => true,
            ["common", "economy" | "production" | "military", file] => file.ends_with(".toml"),
            ["scenarios", _, "scenario.toml" | "defines.toml"] => true,
            ["scenarios", _, "nations", file] => file.ends_with(".toml"),
            ["localisation", "ko" | "en", file] => file.ends_with(".ftl"),
            _ => false,
        };
        if !registered {
            report.warning(io(
                &p,
                "unsupported content adapter (WP-13/14/15/35); schema/ref not validated",
            ));
        }
    }
    Ok(())
}
fn validate_date(source: &Source, value: &str) -> Result<(), DataError> {
    if value
        .split('-')
        .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(at(
            source,
            "start_date",
            None,
            None,
            "invalid Gregorian start_date",
        ));
    }
    let parts = value
        .split('-')
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<_>, _>>();
    let valid = parts.is_ok_and(|p| {
        if p.len() != 3 || p[0] == 0 || p[0] > 9999 || p[1] == 0 || p[1] > 12 {
            return false;
        }
        let leap = p[0] % 4 == 0 && (p[0] % 100 != 0 || p[0] % 400 == 0);
        let days = match p[1] {
            2 => {
                if leap {
                    29
                } else {
                    28
                }
            }
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        p[2] > 0 && p[2] <= days
    });
    if valid {
        Ok(())
    } else {
        Err(at(
            source,
            "start_date",
            None,
            None,
            "invalid Gregorian start_date",
        ))
    }
}

#[cfg(test)]
mod source_change_tests {
    use super::*;
    #[test]
    fn source_changed_between_resolution_and_content_validation_returns_no_packs() {
        fn copy(source: &Path, target: &Path) {
            fs::create_dir_all(target).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let source = entry.unwrap().path();
                let target = target.join(source.file_name().unwrap());
                if source.is_dir() {
                    copy(&source, &target);
                } else {
                    fs::copy(source, target).unwrap();
                }
            }
        }
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root = base.join(format!(
            "target/evidence/WP-24-P06/source-change-{}",
            std::process::id()
        ));
        copy(&base.join("data/packs/testland"), &root);
        let roots = std::slice::from_ref(&root);
        assert!(!validate_packs(roots).failed(false));
        let packs = resolve_packs(roots).unwrap();
        let path = root.join("defines.toml");
        let mut bytes = fs::read(&path).unwrap();
        bytes
            .extend_from_slice(b"\n# Changed after resolution; identical effective definitions.\n");
        fs::write(path, bytes).unwrap();
        let report = validate_resolved(packs, ValidationPurpose::Strict);
        assert!(report.failed(false));
        assert!(report.packs.is_empty());
        assert!(report.diagnostics.iter().any(|d| {
            d.severity == Severity::Error && d.error.message == "pack changed during validation"
        }));
        // The changed source is valid alone; its difference from the snapshot rejects it.
        assert!(!validate_packs(roots).failed(false));
    }
}
