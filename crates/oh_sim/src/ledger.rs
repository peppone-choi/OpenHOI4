//! Authoritative fixed-point breakdowns (02 §6.3, REQ-UI-04).
//!
//! A ledger owns its calculation tick, final value and ordered entries. Hosts
//! retain it with the derived stat and convert it in oh_proto for display.
//! No gameplay coefficients or source-specific rules live in this module.
use crate::formula;
use oh_core::Fx;
use serde::{Deserialize, Serialize};

/// Operation order is Add before Mul; each phase sorts sources lexically.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ModifierOp {
    Add,
    Mul,
}

/// Data-supplied contribution. `expires` is the first excluded absolute tick;
/// `None` never expires. `source` and `target_stat` are identifiers, not UI text.
/// Two active modifiers with the same (target, op, source) are rejected.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    pub source: String,
    pub target_stat: String,
    pub op: ModifierOp,
    pub value: Fx,
    pub expires: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum LedgerOp {
    Base,
    Add,
    Mul,
}

/// One row, including the base. Base has no modifier source. Both numeric
/// fields retain Fx bits; clients must display the server-supplied accumulation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LedgerEntry {
    pub source: Option<String>,
    pub op: LedgerOp,
    pub value: Fx,
    pub accumulated: Fx,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LedgerError {
    EmptyTargetStat,
    EmptySource,
    DuplicateModifier {
        source: String,
        op: ModifierOp,
    },
    Overflow {
        source: String,
        op: ModifierOp,
    },
    AppliedValueMismatch {
        expected_bits: i64,
        actual_bits: i64,
    },
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTargetStat => f.write_str("empty target stat identifier"),
            Self::EmptySource => f.write_str("empty active modifier source identifier"),
            Self::DuplicateModifier { source, op } => {
                write!(f, "duplicate active modifier {source} ({op:?})")
            }
            Self::Overflow { source, op } => {
                write!(f, "fixed-point overflow at modifier {source} ({op:?})")
            }
            Self::AppliedValueMismatch {
                expected_bits,
                actual_bits,
            } => {
                write!(
                    f,
                    "ledger bits {expected_bits} differ from applied bits {actual_bits}"
                )
            }
        }
    }
}
impl std::error::Error for LedgerError {}

/// Immutable calculation result: no partial ledger is returned on failure.
/// Accessors expose no mutable references and Deserialize is intentionally absent.
/// ```compile_fail
/// fn change_result(ledger: &mut oh_sim::ledger::StatLedger) {
///     ledger.value = oh_core::Fx::ZERO;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StatLedger {
    target_stat: String,
    tick: u64,
    value: Fx,
    entries: Vec<LedgerEntry>,
}

impl StatLedger {
    /// Re-evaluate after a base/modifier change or at an expiry boundary. This
    /// pure query does not mutate the modifier collection or simulation state.
    pub fn evaluate(
        target_stat: &str,
        base: Fx,
        modifiers: &[Modifier],
        at_tick: u64,
    ) -> Result<Self, LedgerError> {
        let mut entries = vec![LedgerEntry {
            source: None,
            op: LedgerOp::Base,
            value: base,
            accumulated: base,
        }];
        let value =
            formula::visit_stat(base, target_stat, modifiers, at_tick, |m, accumulated| {
                entries.push(LedgerEntry {
                    source: Some(m.source.clone()),
                    op: match m.op {
                        ModifierOp::Add => LedgerOp::Add,
                        ModifierOp::Mul => LedgerOp::Mul,
                    },
                    value: m.value,
                    accumulated,
                });
            })?;
        Ok(Self {
            target_stat: target_stat.into(),
            tick: at_tick,
            value,
            entries,
        })
    }

    pub fn target_stat(&self) -> &str {
        &self.target_stat
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    /// Systems apply this exact value and retain this result for query handling.
    pub fn value(&self) -> Fx {
        self.value
    }
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }
    /// Assert that a separately stored/applied stat has not drifted from this
    /// ledger. Comparison uses raw bits, never rounded display values.
    pub fn verify_applied_value(&self, applied: Fx) -> Result<(), LedgerError> {
        if self.value.to_bits() != applied.to_bits() {
            return Err(LedgerError::AppliedValueMismatch {
                expected_bits: self.value.to_bits(),
                actual_bits: applied.to_bits(),
            });
        }
        Ok(())
    }
}
