# SL-015 Implementation Notes

Durable, committed record of decisions/findings that outlive a phase sheet.
Design rationale lives in `design.md`; phase criteria in `plan.toml`; this is for
cross-phase implementation decisions the design didn't pin.

## Cross-phase decisions

- **D-1 — requirement `kind` is template-seeded, then overwritten post-reserve.**
  `ReqKind` (functional|quality) is not carried by `entity::ScaffoldCtx`, and the
  engine must stay unchanged (R6 gate). So `install/templates/requirement.toml`
  seeds a default `kind = "functional"`, and `spec req add --kind` (PHASE-03) sets
  the real value after reservation via an edit-preserving `toml_edit` write —
  exactly the `adr::set_adr_status` pattern (status seeded `proposed`, later
  flipped). Spans PHASE-01 (seed) → PHASE-03 (overwrite). No engine edit.

- **D-2 — staged-landing lint bridge.** A module landing one phase ahead of its
  first production caller (PHASE-01 `requirement.rs` before PHASE-03 `spec req
  add`) is `dead_code` in the bins/lib build. Bridge each pending item with
  `#[cfg_attr(not(test), expect(dead_code, reason = "first caller in PHASE-NN"))]`.
  Bare `#[allow]` is a hard error in this repo — see the recorded pattern memory
  `mem.pattern.lint.expect-not-allow` (`doctrine memory retrieve --query expect
  allow`). PHASE-02 is expected to unwind it as `spec.rs` references the
  requirement types; `expect` makes any stale bridge a build error, so it cannot
  rot.

- **D-3 — `Source` shape (PHASE-02, user-confirmed).** `design.md` §5.3 declares
  `sources: Vec<Source>` (tech-only) but never defines `Source`. Resolved to the
  legacy-canon shape (`doc/spec-entity-spec.md:202`): `Source { language: String,
  identifier: String, #[serde(default)] module: Option<String> }` — a code anchor.
  Tech-only, seeded-empty in P2; payload first rendered by `spec show` (PHASE-04).
  The design should be back-filled with this when §5.3 is next touched (P6 canon
  sweep is a natural home).

- **D-4 — `spec list` renders per-subtype sections, not one table.** `PRD-001` and
  `SPEC-001` co-exist (independent reservation namespaces), so a single `id` column
  would be ambiguous. `run_list` prints a `product` block then a `tech` block, each
  a labelled `id status slug #members` table over the shared `meta::render_table`.
  Empty subtype → suppressed. (PHASE-02.)

## Findings

- **F-1 — lint suppression form** captured durably as memory
  `mem.pattern.lint.expect-not-allow` (not repeated here; the storage rule).

- **F-2 — `entity::Kind.dir` is project-root-relative and MUST include
  `.doctrine/`** (`.doctrine/spec/product`, not `spec/product`). PHASE-02 first set
  it without the prefix; tempdir unit tests passed because they built their
  expected path from the same constant — only a **CLI smoke test against the built
  binary** exposed the misplacement. Durable lesson: smoke the real binary for any
  new `Kind`; a self-referential path assertion proves consistency, not
  correctness.

- **F-3 — the `#members` column needs zero `meta.rs` change.** `meta::read_metas`
  (stem-parametric) + `meta::render_table` (generic grid) already suffice; the
  derived cell is built in `spec.rs`, mirroring `slice.rs`'s `phases` cell. The
  shared 4-column `format_list` path is untouched — the strongest form of the
  behaviour-preservation gate (PHASE-02 VT-3).
