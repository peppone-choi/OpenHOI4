//! Data pack loading boundary (REQ-GEN-02 / AC-M0-03).
//!
//! The M0 entry point loads manifest metadata and numeric defines. The separate
//! `map` entry point validates M1 geometry, state and VP definitions. Full content
//! registries and pack merging belong to later WPs. `pack_validation` resolves
//! pack order and validates the currently registered content and Fluent catalogs.
//! Numeric defines have no defaults. I/O stays outside the simulation layer.
pub mod host_policy;
pub mod localisation;
pub mod m0;
pub mod map;
pub mod national;
pub mod pack_validation;
mod raw;
pub mod scenario_defines;
pub mod trigger;

use raw::Source;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use toml::de::DeValue as Raw;

/// DR-01 coefficient representation. Parsed directly from the source lexeme.
pub type Fixed = fixed::types::I32F32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Number {
    Integer(i64),
    Fixed(Fixed),
}
impl JsonSchema for Number {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Number".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        schemars::json_schema!({"type": "number", "description": "TOML integers are i64; decimals are checked I32F32 parsed from their lexemes. Nonfinite or overflowing values are rejected."})
    }
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum DefineValue {
    Number(Number),
    Array(Vec<Number>),
}

/// System -> item -> numeric scalar or array, in deterministic lexical order.
/// Unknown coefficient names remain available for future system registries.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
#[serde(transparent)]
pub struct Defines(pub BTreeMap<String, BTreeMap<String, DefineValue>>);
impl Defines {
    /// Look up a system.item key. Missing keys never acquire gameplay defaults.
    pub fn get(&self, key: &str) -> Option<&DefineValue> {
        let (system, item) = key.split_once('.')?;
        self.0.get(system)?.get(item)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    #[schemars(regex(pattern = "^[a-z0-9_]+$"))]
    pub id: String,
    #[schemars(length(min = 1))]
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[schemars(regex(pattern = "^[a-z0-9_]+$"))]
    pub id: String,
    #[schemars(length(min = 1))]
    pub name_key: String,
    #[schemars(with = "semver::Version")]
    pub version: String,
    #[schemars(length(min = 1))]
    pub engine: String,
    #[serde(default)]
    pub depends: Vec<Dependency>,
    #[serde(default)]
    #[schemars(inner(regex(pattern = "^[a-z0-9_]+$")))]
    pub conflicts: Vec<String>,
    #[serde(default)]
    #[schemars(inner(regex(pattern = "^[a-z0-9_]+$")))]
    pub load_after: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPack {
    pub manifest: Manifest,
    pub defines: Defines,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Io,
    Syntax,
    Schema,
}

/// One-based line and Unicode scalar column. Missing files use 1:1; missing
/// fields point at their enclosing table (1:1 for the document root).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataError {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{} — {}",
            self.path.display(),
            self.line,
            self.column,
            self.message
        )
    }
}
impl std::error::Error for DataError {}

/// Read the two required files, rejecting a pack atomically on the first error.
/// Dependency ranges are checked for syntax only; resolution belongs to WP-24.
pub fn load_pack(root: impl AsRef<Path>) -> Result<DataPack, DataError> {
    let root = root.as_ref();
    let manifest = read(&root.join("manifest.toml"))?;
    let manifest = parse_manifest(&manifest)?;
    let defines = read(&root.join("defines.toml"))?;
    let defines = parse_defines(&defines)?;
    Ok(DataPack { manifest, defines })
}
// Definition/codec readers do not render manifest names, but any present FTL
// still has to have valid syntax, parity and static references. Active callers
// use the full validation entry point to require their actual display keys.
pub(crate) fn check_present_catalogs(root: &Path) -> Result<(), DataError> {
    let diagnostics = localisation::validate_directory(&root.join("localisation"), &[])?;
    if let Some(d) = diagnostics
        .into_iter()
        .find(|d| d.severity == pack_validation::Severity::Error)
    {
        return Err(d.error);
    }
    Ok(())
}

/// Generate the JSON structural schema from the loader's public types.
pub fn manifest_schema() -> Schema {
    schemars::schema_for!(Manifest)
}
pub fn defines_schema() -> Schema {
    schemars::schema_for!(Defines)
}

fn read(path: &Path) -> Result<Source, DataError> {
    let text = fs::read_to_string(path).map_err(|err| DataError {
        path: path.to_owned(),
        line: 1,
        column: 1,
        kind: ErrorKind::Io,
        message: err.to_string(),
    })?;
    Ok(Source {
        path: path.to_owned(),
        text,
    })
}

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
fn check_id(source: &Source, value: &str, offset: usize) -> Result<(), DataError> {
    if valid_id(value) {
        Ok(())
    } else {
        Err(source.error(offset, ErrorKind::Schema, "pack id must match [a-z0-9_]+"))
    }
}
fn check_range(source: &Source, value: &str, offset: usize) -> Result<(), DataError> {
    semver::VersionReq::parse(value).map(|_| ()).map_err(|err| {
        source.error(
            offset,
            ErrorKind::Schema,
            format!("invalid version range: {err}"),
        )
    })
}

fn parse_manifest(source: &Source) -> Result<Manifest, DataError> {
    // Parse grammar before deserialization so TOML syntax and schema failures
    // have distinct diagnostic categories, without relying on message text.
    let document = source.document()?;
    let manifest: Manifest = toml::from_str(&source.text).map_err(|err| {
        source.error(
            err.span().map_or(0, |span| span.start),
            ErrorKind::Schema,
            err.message(),
        )
    })?;
    check_id(source, &manifest.id, document["id"].span().start)?;
    if manifest.name_key.is_empty() {
        return Err(source.error(
            document["name_key"].span().start,
            ErrorKind::Schema,
            "name_key must not be empty",
        ));
    }
    semver::Version::parse(&manifest.version).map_err(|err| {
        source.error(
            document["version"].span().start,
            ErrorKind::Schema,
            format!("invalid SemVer version: {err}"),
        )
    })?;
    check_range(source, &manifest.engine, document["engine"].span().start)?;
    if let Some(depends) = document.get("depends") {
        // Shape has already been checked by the shared Manifest deserializer.
        if let Raw::Array(entries) = depends.get_ref() {
            for (dependency, entry) in manifest.depends.iter().zip(entries.iter()) {
                if let Raw::Table(fields) = entry.get_ref() {
                    check_id(source, &dependency.id, fields["id"].span().start)?;
                    check_range(source, &dependency.version, fields["version"].span().start)?;
                }
            }
        }
    }
    for key in ["conflicts", "load_after"] {
        if let Some(values) = document.get(key)
            && let Raw::Array(entries) = values.get_ref()
        {
            for entry in entries.iter() {
                if let Raw::String(id) = entry.get_ref() {
                    check_id(source, id, entry.span().start)?;
                }
            }
        }
    }
    Ok(manifest)
}

fn parse_defines(source: &Source) -> Result<Defines, DataError> {
    let document = source.document()?;
    let mut systems = BTreeMap::new();
    for (system, value) in document {
        let Raw::Table(fields) = value.get_ref() else {
            return Err(source.error(
                value.span().start,
                ErrorKind::Schema,
                "defines root values must be system tables",
            ));
        };
        let mut items = BTreeMap::new();
        for (item, value) in fields {
            let parsed = match value.get_ref() {
                Raw::Array(values) => DefineValue::Array(
                    values
                        .iter()
                        .map(|v| parse_number(source, v))
                        .collect::<Result<_, _>>()?,
                ),
                _ => DefineValue::Number(parse_number(source, value)?),
            };
            items.insert(item.get_ref().as_ref().to_owned(), parsed);
        }
        systems.insert(system.into_inner().into_owned(), items);
    }
    Ok(Defines(systems))
}

fn parse_number(source: &Source, value: &toml::Spanned<Raw<'_>>) -> Result<Number, DataError> {
    match value.get_ref() {
        Raw::Integer(number) => i64::from_str_radix(number.as_str(), number.radix())
            .map(Number::Integer)
            .map_err(|err| {
                source.error(
                    value.span().start,
                    ErrorKind::Schema,
                    format!("invalid i64 define: {err}"),
                )
            }),
        Raw::Float(_) => {
            // DeFloat retains the decimal lexeme without converting it to f64.
            // Parse the source directly with fixed's checked decimal parser.
            let lexeme = source.text[value.span()].replace('_', "");
            lexeme.parse::<Fixed>().map(Number::Fixed).map_err(|err| {
                source.error(
                    value.span().start,
                    ErrorKind::Schema,
                    format!("invalid fixed-point coefficient: {err}"),
                )
            })
        }
        _ => Err(source.error(
            value.span().start,
            ErrorKind::Schema,
            "define value must be a number or an array of numbers",
        )),
    }
}
