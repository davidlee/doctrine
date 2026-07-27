# DEC-084: Existing designs enter through explicit conservative import

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

When an authored `design.md` exists but no managed runtime snapshot survives,
plain `doctrine design resume SL-NNN` reports that exact procedural resume is
unavailable. It does not infer a run from prose.

The explicit command `doctrine design start SL-NNN --from-design` creates a new
run UID and imports the document's headings and bodies as fingerprinted,
unreviewed section drafts. It records their authored source and does not invent
inquiry history, stage clearance, user acceptance, review attestations, or
other procedural evidence.

This preserves the strong meaning of exact resume while providing a bounded
migration and runtime-loss recovery path for existing designs. The sources from
which open inquiry nodes may additionally be bootstrapped are settled
separately by QUE-196.

This new-run recovery path is distinct from DEC-072's live-run re-adoption. If
the runtime snapshot survives, an explicit `apply` declaration can adopt a
human-edited materialised document back into that same run using its stable
section markers. It does not mint a new run UID or reconstruct inquiry history;
it preserves procedural state and invalidates only evidence bound to changed
section fingerprints.
