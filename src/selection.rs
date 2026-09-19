// SPDX-License-Identifier: GPL-3.0-only
//! `selection` — the one type `relation_graph` and `knowledge` share (SL-246
//! PHASE-02, DEC-274).
//!
//! `relation_graph::select_knowledge` builds `SelectedRecord`s from an
//! `InspectView`; `knowledge`'s render path consumes them. Siting the type in
//! either peer would force the other to import it, and `knowledge` +
//! `relation_graph` are both tier `command` in `.doctrine/adr/001/layering.toml`
//! (lines 105, 121) with a reverse path already live —
//! `knowledge → install → boot → memory → priority → relation_graph` — so a
//! `relation_graph → knowledge` edge would close a cycle. Sited in this leaf
//! instead, BOTH peers depend downward on it and NEITHER imports the other
//! (ADR-001): the tangle does not grow.
//!
//! **Leaf tier (ADR-001).** Pure data — imports nothing from the engine or any
//! command module.

/// One record chosen for a composed read, with the text it renders under. Built by
/// the engine-tier selection that turns an inspect view into an ordered,
/// deduplicated record list (SL-246 design). `caption` is TEXT, not a
/// `RelationLabel` (DEC-147): at one hop it is the derived inbound verb
/// (`crate::relation::inbound_name`); at depth N (IMP-398 S5) it becomes a path,
/// and the renderer must not have to change to learn that.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "PHASE-03 render consumes this; select_knowledge constructs it now"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedRecord {
    pub(crate) reference: String,
    pub(crate) caption: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------
    // PHASE-02 (SL-246): SelectedRecord — VT-2.
    // -------------------------------------------------------------------

    #[test]
    fn selected_record_carries_reference_and_caption() {
        let sr = SelectedRecord {
            reference: "DEC-145".to_string(),
            caption: "shaped_by".to_string(),
        };
        assert_eq!(sr.reference, "DEC-145");
        assert_eq!(sr.caption, "shaped_by");
    }
}
