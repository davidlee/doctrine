# IMP-056: Coverage CLI status rendering: stable kebab-case formatter, not Debug

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised at SL-057 audit (RV-017 F-3, minor; deferred out of the slice).

`coverage_verify::status_label` and `coverage_store::withdrawal_line` render
`CoverageStatus` to user-facing CLI output via `Debug` (`format!("{status:?}")`).
That leaks the internal Rust spelling (`InProgress`) where the *input* vocabulary
is kebab-case (`in-progress`, via `coverage_store::parse_status`), so the surface
is asymmetric in/out. The PHASE-05 goldens pin only `Planned`/`Verified`/`Failed`
renderings; `Blocked` and `InProgress` output is unpinned.

**Do:** add one stable status formatter (the serde rename token / the same
kebab-case `parse_status` accepts), use it in both the verify report and the
forget withdrawal line, and extend the goldens to cover every rendered status.

**Why deferred from SL-057:** minor (no soundness impact — the tokens are stable
under the existing goldens for the rendered cases); and `withdrawal_line` is
PHASE-03 code whose output a fix would re-pin, widening churn beyond the PHASE-05
surface. Out of the audit's remediation scope; the two majors (F-1 requirement
canonicalization, F-2 `verify --all` error propagation) were fixed in-slice.
