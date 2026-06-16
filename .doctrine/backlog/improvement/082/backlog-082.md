# IMP-082: Slice cannot author `related` edges

`doctrine link SL-NNN related SL-MMM` is refused — `related` is not in SL's
legal label set (`specs`, `requirements`, `supersedes`, `governed_by`).

Cross-slice relationships (e.g. SL-080 noting SL-082 as parallel cleanup) have
no home in the semantic graph and are relegated to prose.

The `related` label is available on ADR and SPEC — the carve-out for SL is a gap
in the relation-rules vocabulary (ADR-010). Either `related` should be added to
SL's legal set, or a new label (e.g. `see_also`, `parallel`) should be defined.

Discovered during SL-080 design (IMP-008 reconciliation).
