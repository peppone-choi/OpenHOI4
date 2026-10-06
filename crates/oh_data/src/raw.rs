//! TOML syntax tree spans for diagnostics and exact numeric lexemes.
use crate::{DataError, ErrorKind};
use std::path::PathBuf;
use toml::de::DeTable;

pub(crate) struct Source {
    pub path: PathBuf,
    pub text: String,
}
impl Source {
    pub fn error(&self, offset: usize, kind: ErrorKind, message: impl Into<String>) -> DataError {
        // All offsets originate from the TOML parser and lie at UTF-8 boundaries.
        let prefix = &self.text[..offset.min(self.text.len())];
        let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
        let column = prefix
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .chars()
            .count()
            + 1;
        DataError {
            path: self.path.clone(),
            line,
            column,
            kind,
            message: message.into(),
        }
    }
    pub fn document(&self) -> Result<DeTable<'_>, DataError> {
        DeTable::parse(&self.text)
            .map(toml::Spanned::into_inner)
            .map_err(|err| {
                self.error(
                    err.span().map_or(0, |span| span.start),
                    ErrorKind::Syntax,
                    err.message(),
                )
            })
    }
}
