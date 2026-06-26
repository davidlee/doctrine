//! The kind-identity vocabulary: canonical prefix per kind + the relation
//! source/target groupings. Leaf tier (ADR-001) — depends on nothing in-crate,
//! so the engine borrows identity without reaching up into command modules.
//! The prefix is the canonical kind identity (compared by `==` everywhere).
#![allow(
    dead_code,
    reason = "consumed by relation engine + command modules in PHASE-02/03"
)]

pub(crate) const SL: &str = "SL";
pub(crate) const PRD: &str = "PRD";
pub(crate) const SPEC: &str = "SPEC";
pub(crate) const CM: &str = "CM";
pub(crate) const REQ: &str = "REQ";
pub(crate) const ADR: &str = "ADR";
pub(crate) const POL: &str = "POL";
pub(crate) const STD: &str = "STD";
pub(crate) const RV: &str = "RV";
pub(crate) const REC: &str = "REC";
pub(crate) const REV: &str = "REV";
pub(crate) const RFC: &str = "RFC";
pub(crate) const ISS: &str = "ISS";
pub(crate) const IMP: &str = "IMP";
pub(crate) const CHR: &str = "CHR";
pub(crate) const RSK: &str = "RSK";
pub(crate) const IDE: &str = "IDE";
pub(crate) const ASM: &str = "ASM";
pub(crate) const DEC: &str = "DEC";
pub(crate) const QUE: &str = "QUE";
pub(crate) const CON: &str = "CON";

/// Every governance kind — `supersedes`/`related` source-set + `governed_by` targets.
pub(crate) const GOV: &[&str] = &[ADR, POL, STD];
/// Every backlog item kind — they share one `relation_edges` accessor.
pub(crate) const BACKLOG: &[&str] = &[ISS, IMP, CHR, RSK, IDE];
/// Every knowledge-record kind.
pub(crate) const RECORD: &[&str] = &[ASM, DEC, QUE, CON];

/// Membership predicate over [`RECORD`] — the single source for "is this a
/// knowledge-record kind?" so adding/renaming a record kind edits RECORD,
/// not every call site.
pub(crate) fn is_record(prefix: &str) -> bool {
    RECORD.contains(&prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groupings_match_documented_membership() {
        assert_eq!(GOV, &[ADR, POL, STD]);
        assert_eq!(BACKLOG, &[ISS, IMP, CHR, RSK, IDE]);
        assert_eq!(RECORD, &[ASM, DEC, QUE, CON]);
    }
}
