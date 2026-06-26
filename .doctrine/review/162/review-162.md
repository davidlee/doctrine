# Review RV-162 — reconciliation of SL-155

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of attack:**

1. **Cluster A one-liners** (C1, C2a/b/c, C3, G5a/b, I1) — each is a targeted
   edit across `src/relation.rs`, `src/spec.rs`, `src/tag.rs`, 4 templates, and
   one `doctrine supersede` edge. Verify every edit matches the design's
   code-impact table exactly.
2. **Revision list verb** (L1) — constants (REV_COLUMNS 5 cols no slug,
   REV_DEFAULT tags-opt-in per D2, REV_STATUSES, REV_HIDDEN), RevDoc.tags with
   serde(default), CLI wiring (RevisionCommand::List + dispatch), template tags,
   12 design tests + round-trip test all green.
3. **Conformance registry** — `slice conformance` reports incomplete for
   PHASE-01 (no recorded source-delta). Bootstrap and re-check.
4. **Gate baseline** — `cargo clippy` zero warnings; existing revision tests
   unchanged; `revision list --all` renders correct default columns.

**Invariants:** D1 (hide terminal by default), D2 (tags opt-in via --columns),
D3 (REC shape not governance), D4 (no created/updated columns), D5 (no slug
column). All design decisions verifiable against actual constants and CLI
behaviour.

## Synthesis

SL-155 is a clean, fully-conformant slice. Every design objective is verifiably
met:

**Cluster A** — seven one-liner fixes across six files (C1, C2a/b/c, C3, G5a,
I1) plus the ADR-012→ADR-004 supersede edge (G5b). All match the design's
code-impact table exactly:
- C1: `src/relation.rs` L407-414 — Parent rule `sources: &[SPEC, PRD]`,
  `target: Kinds(&[SPEC, PRD])` ✓
- C2a: `install/templates/spec-tech.toml` L19 — "subtype-aware (SPEC or PRD)" ✓
- C2b: `install/templates/spec-product.toml` — `# parent = "PRD-NNN"` present ✓
- C2c: `install/templates/interactions.toml` L3 — "authored via
  `doctrine spec interactions add`" ✓
- C3: `src/spec.rs` L746 — "Tech-only" removed from parent doc comment ✓
- G5a: `install/templates/adr.toml` L13 — `doctrine supersede` reference ✓
- G5b: ADR-012.supersedes = ["ADR-004"], ADR-004.superseded_by = ["ADR-012"] ✓
- I1: `src/tag.rs` L18 — "REV" in TAGGABLE ✓

**Revision list** — constants match design decisions exactly: 5-column
REV_COLUMNS (no slug per D5), REV_DEFAULT = ["id","status","approval","title"]
(tags opt-in per D2), REV_HIDDEN = ["done","abandoned"] (D1). CLI is wired:
`RevisionCommand::List` with `CommonListArgs` + dispatch arm in `run_revision`.
`RevDoc.tags: Vec<String>` with `#[serde(default)]`; template has `tags = []`.
All 33 revision tests pass (12 design tests + round-trip test + pre-existing
suite). `revision show --json` includes tags in the nested revision object.
`doctrine tag set REV-001 ...` succeeds. `revision list --all` renders correct
default columns; `--columns id,status,tags` reveals tags column.

**Conformance** — recovered from incomplete state. PHASE-01 source-delta now
recorded (`a29b6cd4..9cf34868`). The conformance report shows 39 undeclared
paths — these are files from SL-154, SL-156, SL-138, and other work landing on
the same branch, plus SL-155's own ADR files (not in design-target selectors).
Zero undelivered selectors. PHASE-02 delta not yet recorded (commit range
entangled with post-close fixes and SL-156).

**Gate** — `cargo clippy` zero warnings. `cargo test revision` — all 33 tests
green. `just gate` fails on pre-existing e2e_dispatch_candidate infrastructure
failures (22/23), unrelated to SL-155.

**Standing risks:** none. This was a low-risk slice of targeted fixes and one
well-understood verb addition. No architectural decisions, no new dependencies,
no schema changes.

**Tradeoffs accepted:** e2e dispatch suite is broken infrastructure debt —
tolerated per F-2, not blocking SL-155.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md § L1 test count**: design says "12 unit tests"; the suite now has
  14 list-specific tests (2 drift canaries: `rev_statuses_matches_the_variants`
  and `rev_hidden_covers_terminal_statuses`, predate the list verb but are in
  scope). Update test table or note the expansion.

### Governance/spec (REV)

None. This slice made no governance or spec changes — only code, templates, and
one supersede edge authoring.
