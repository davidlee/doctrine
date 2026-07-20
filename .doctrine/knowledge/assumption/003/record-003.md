# ASM-003: Customization-status ships provisional {customizable, fixed}

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

PRD-017 OQ-5. Every published asset must carry a **customization-status** field
(REQ-359), but its real vocabulary is owned by the C2 supported-customization model
(RFC-021), which C1 does not deliver.

**Carried assumption for SL-223 and until C2 settles it:** the field ships as a
closed two-value enum `{customizable, fixed}` (STD-001 named constants), with the
provisional meaning *"whether an asset is intended to be customized."* A closed
enum honours the "every entry carries it" invariant, stays checkable, and marks
itself provisional so C2 can **widen** it as a controlled additive change rather
than inheriting an unconstrained free-text field.

If C2 lands a richer vocabulary, this assumption is superseded — the enum is the
migration point. See [[QUE-172]] for the sibling deferred question (OQ-1).
