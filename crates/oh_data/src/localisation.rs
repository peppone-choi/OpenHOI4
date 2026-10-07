//! Fluent AST validation. No fallback can conceal a missing shipped translation.
use crate::{
    DataError, ErrorKind,
    pack_validation::{Diagnostic, Severity},
    raw::Source,
    read,
};
use fluent_syntax::{ast, parser};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct KeyUse {
    pub key: String,
    pub context: DataError,
}
pub fn keys_in_toml(path: &Path) -> Result<Vec<KeyUse>, DataError> {
    fn walk(source: &Source, value: &toml::Spanned<toml::de::DeValue<'_>>, out: &mut Vec<KeyUse>) {
        match value.get_ref() {
            toml::de::DeValue::Table(t) => {
                for (k, v) in t {
                    if ["name_key", "government_key"].contains(&k.get_ref().as_ref())
                        && let toml::de::DeValue::String(s) = v.get_ref()
                    {
                        out.push(KeyUse {
                            key: s.to_string(),
                            context: source.error(
                                v.span().start,
                                ErrorKind::Schema,
                                "localisation reference",
                            ),
                        });
                    }
                    if k.get_ref() == "ideology_support"
                        && let toml::de::DeValue::Table(t) = v.get_ref()
                    {
                        for key in t.keys() {
                            out.push(KeyUse {
                                key: key.get_ref().to_string(),
                                context: source.error(
                                    key.span().start,
                                    ErrorKind::Schema,
                                    "ideology localisation reference",
                                ),
                            });
                        }
                    }
                    walk(source, v, out);
                }
            }
            toml::de::DeValue::Array(a) => {
                for v in a {
                    walk(source, v, out);
                }
            }
            _ => {}
        }
    }
    let source = read(path)?;
    let d = source.document()?;
    let mut out = Vec::new();
    for (k, v) in &d {
        if ["name_key", "government_key"].contains(&k.get_ref().as_ref())
            && let toml::de::DeValue::String(s) = v.get_ref()
        {
            out.push(KeyUse {
                key: s.to_string(),
                context: source.error(v.span().start, ErrorKind::Schema, "localisation reference"),
            });
        }
        if k.get_ref() == "ideology_support"
            && let toml::de::DeValue::Table(t) = v.get_ref()
        {
            for key in t.keys() {
                out.push(KeyUse {
                    key: key.get_ref().to_string(),
                    context: source.error(
                        key.span().start,
                        ErrorKind::Schema,
                        "ideology localisation reference",
                    ),
                });
            }
        }
        walk(&source, v, &mut out);
    }
    Ok(out)
}
#[derive(Debug)]
struct Node {
    location: DataError,
    refs: Vec<(String, DataError)>,
}
#[derive(Default)]
struct Catalog {
    nodes: BTreeMap<String, Node>,
    messages: BTreeSet<String>,
    entries: BTreeSet<String>,
}
fn token(source: &Source, s: &str, message: impl Into<String>) -> DataError {
    source.error(
        s.as_ptr() as usize - source.text.as_ptr() as usize,
        ErrorKind::Schema,
        message,
    )
}
fn reference(
    source: &Source,
    id: &ast::Identifier<&str>,
    attribute: &Option<ast::Identifier<&str>>,
    term: bool,
    out: &mut Vec<(String, DataError)>,
) {
    let target = format!(
        "{}{}{}",
        if term { "-" } else { "" },
        id.name,
        attribute
            .as_ref()
            .map(|a| format!(".{}", a.name))
            .unwrap_or_default()
    );
    out.push((target, token(source, id.name, "Fluent reference")));
}
fn arguments(source: &Source, a: &ast::CallArguments<&str>, out: &mut Vec<(String, DataError)>) {
    for e in &a.positional {
        inline(source, e, out);
    }
    for a in &a.named {
        inline(source, &a.value, out);
    }
}
fn inline(source: &Source, e: &ast::InlineExpression<&str>, out: &mut Vec<(String, DataError)>) {
    match e {
        ast::InlineExpression::MessageReference { id, attribute } => {
            reference(source, id, attribute, false, out)
        }
        ast::InlineExpression::TermReference {
            id,
            attribute,
            arguments: a,
        } => {
            reference(source, id, attribute, true, out);
            if let Some(a) = a {
                arguments(source, a, out);
            }
        }
        ast::InlineExpression::FunctionReference { id, arguments: a } => {
            if !["NUMBER", "DATETIME"].contains(&id.name) {
                out.push((
                    format!("function:{}", id.name),
                    token(source, id.name, "unsupported Fluent function"),
                ));
            }
            arguments(source, a, out);
        }
        ast::InlineExpression::Placeable { expression: e } => expression(source, e, out),
        _ => {}
    }
}
fn expression(source: &Source, e: &ast::Expression<&str>, out: &mut Vec<(String, DataError)>) {
    match e {
        ast::Expression::Inline(e) => inline(source, e, out),
        ast::Expression::Select { selector, variants } => {
            inline(source, selector, out);
            for v in variants {
                pattern(source, &v.value, out);
            }
        }
    }
}
fn pattern(source: &Source, p: &ast::Pattern<&str>, out: &mut Vec<(String, DataError)>) {
    for e in &p.elements {
        if let ast::PatternElement::Placeable { expression: e } = e {
            expression(source, e, out);
        }
    }
}
fn add_node(
    c: &mut Catalog,
    source: &Source,
    key: String,
    name: &str,
    p: &ast::Pattern<&str>,
) -> Result<(), DataError> {
    if c.nodes.contains_key(&key) {
        return Err(token(source, name, format!("duplicate Fluent key {key}")));
    }
    let mut refs = Vec::new();
    pattern(source, p, &mut refs);
    c.nodes.insert(
        key,
        Node {
            location: token(source, name, "Fluent entry"),
            refs,
        },
    );
    Ok(())
}
fn catalog(paths: &[PathBuf]) -> Result<Catalog, DataError> {
    let mut c = Catalog::default();
    for path in paths {
        let source = read(path)?;
        let r = parser::parse(source.text.as_str()).map_err(|(_, errors)| {
            let e = &errors[0];
            source.error(
                e.pos.start,
                ErrorKind::Syntax,
                format!("invalid Fluent: {:?}", e.kind),
            )
        })?;
        for entry in r.body {
            match entry {
                ast::Entry::Message(m) => {
                    if !c.entries.insert(m.id.name.into()) {
                        return Err(token(
                            &source,
                            m.id.name,
                            format!("duplicate Fluent message {}", m.id.name),
                        ));
                    }
                    if let Some(p) = m.value {
                        c.messages.insert(m.id.name.into());
                        add_node(&mut c, &source, m.id.name.into(), m.id.name, &p)?;
                    }
                    for a in m.attributes {
                        add_node(
                            &mut c,
                            &source,
                            format!("{}.{}", m.id.name, a.id.name),
                            a.id.name,
                            &a.value,
                        )?;
                    }
                }
                ast::Entry::Term(t) => {
                    let key = format!("-{}", t.id.name);
                    if !c.entries.insert(key.clone()) {
                        return Err(token(
                            &source,
                            t.id.name,
                            format!("duplicate Fluent term {key}"),
                        ));
                    }
                    add_node(&mut c, &source, key.clone(), t.id.name, &t.value)?;
                    for a in t.attributes {
                        add_node(
                            &mut c,
                            &source,
                            format!("{key}.{}", a.id.name),
                            a.id.name,
                            &a.value,
                        )?;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(c)
}
fn diagnostic(mut error: DataError, severity: Severity, message: impl Into<String>) -> Diagnostic {
    error.message = message.into();
    Diagnostic { severity, error }
}
fn check_graph(c: &Catalog, out: &mut Vec<Diagnostic>) {
    for n in c.nodes.values() {
        for (key, loc) in &n.refs {
            if !c.nodes.contains_key(key) {
                out.push(diagnostic(
                    loc.clone(),
                    Severity::Error,
                    format!("missing Fluent reference {key}"),
                ));
            }
        }
    }
    // Iterative pruning avoids recursive depth and detects every residual static cycle.
    let mut graph: BTreeMap<_, BTreeSet<_>> = c
        .nodes
        .iter()
        .map(|(k, n)| {
            (
                k.clone(),
                n.refs
                    .iter()
                    .filter(|(k, _)| c.nodes.contains_key(k))
                    .map(|(k, _)| k.clone())
                    .collect(),
            )
        })
        .collect();
    loop {
        let leaves: Vec<_> = graph
            .iter()
            .filter(|(_, v)| v.is_empty())
            .map(|(k, _)| k.clone())
            .collect();
        if leaves.is_empty() {
            break;
        }
        for k in &leaves {
            graph.remove(k);
        }
        for deps in graph.values_mut() {
            for k in &leaves {
                deps.remove(k);
            }
        }
    }
    for k in graph.keys() {
        out.push(diagnostic(
            c.nodes[k].location.clone(),
            Severity::Error,
            format!("Fluent reference cycle involving {k}"),
        ));
    }
}
/// Explicit locale file lists also support the client catalog layout.
pub fn validate_catalogs(
    ko: &[PathBuf],
    en: &[PathBuf],
    uses: &[KeyUse],
    warn_unused: bool,
) -> Result<Vec<Diagnostic>, DataError> {
    let ko = catalog(ko)?;
    let en = catalog(en)?;
    let mut out = Vec::new();
    for (locale, c, other) in [("ko", &ko, &en), ("en", &en, &ko)] {
        check_graph(c, &mut out);
        for (key, node) in &c.nodes {
            if !key.starts_with('-') && !other.nodes.contains_key(key) {
                out.push(diagnostic(
                    node.location.clone(),
                    Severity::Error,
                    format!(
                        "missing {key} in {}",
                        if locale == "ko" { "en" } else { "ko" }
                    ),
                ));
            }
        }
        let mut used = BTreeSet::new();
        let mut todo = Vec::new();
        for usage in uses {
            if !c.messages.contains(&usage.key) {
                out.push(diagnostic(
                    usage.context.clone(),
                    Severity::Error,
                    format!("missing {locale} message value {}", usage.key),
                ));
            } else {
                todo.push(usage.key.clone());
            }
        }
        while let Some(k) = todo.pop() {
            if used.insert(k.clone())
                && let Some(node) = c.nodes.get(&k)
            {
                todo.extend(node.refs.iter().map(|(k, _)| k.clone()));
                for attr in c.nodes.keys().filter(|a| a.starts_with(&format!("{k}."))) {
                    todo.push(attr.clone());
                }
            }
        }
        if warn_unused {
            for k in &c.messages {
                if !used.contains(k) {
                    out.push(diagnostic(
                        c.nodes[k].location.clone(),
                        Severity::Warning,
                        format!("unused {locale} localisation key {k}"),
                    ));
                }
            }
        }
    }
    Ok(out)
}
pub fn validate_directory(root: &Path, uses: &[KeyUse]) -> Result<Vec<Diagnostic>, DataError> {
    let files = |locale: &str| -> Result<Vec<PathBuf>, DataError> {
        let dir = root.join(locale);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut paths = fs::read_dir(&dir)
            .map_err(|e| DataError {
                path: dir.clone(),
                line: 1,
                column: 1,
                kind: ErrorKind::Io,
                message: e.to_string(),
            })?
            .map(|e| {
                e.map(|e| e.path()).map_err(|e| DataError {
                    path: dir.clone(),
                    line: 1,
                    column: 1,
                    kind: ErrorKind::Io,
                    message: e.to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        paths.retain(|p| p.extension().is_some_and(|s| s == "ftl"));
        paths.sort();
        Ok(paths)
    };
    validate_catalogs(&files("ko")?, &files("en")?, uses, true)
}
