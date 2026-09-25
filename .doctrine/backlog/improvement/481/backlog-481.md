# IMP-481: Assess and author governance coverage for the RV kind

`ADR-007` is the only artifact that owns the RV kind; no PRD or SPEC covers it
(`grep` over spec titles finds none). The ADR has drifted from the implementation
in at least two places (D-C8 empty ledger, D-C10 warm-cache) and its shipped
protocol doc drifted too.

The RFC-032 programme intends each slice to leave behind spec coverage. First step:
a spec-coverage assessment (see `IMP-295`'s skill) for the review surface, to decide
whether a PRD/SPEC owns the kind or ADR-007 stays sole authority and is kept
current. See RFC-032 `research.md` F13.
