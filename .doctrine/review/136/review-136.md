# Review RV-136 — reconciliation of SL-139

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit surface: candidate interaction branch `candidate/139/review-001`
(cand-139-review-001), built from the review/139 impl bundle merged onto main.
The 4 phases span one shared helper module (paths.rs), concept-map --json
shorthand, and paths-verb wiring across all 13 in-scope entity kinds.

Lines of attack:

1. **paths.rs conformance** — Does the engine-tier module match design §5.3?
   No clap imports, no command-tier deps? Exclusion filter (§5.5) correct?
   Selector logic (§5.2) correct? Canonical order enforced?

2. **concept-map --json parity** — Is the boolean flag a true shorthand for
   `--format json`? Byte-identical output? Existing tests green? D8 scope
   boundary (CLI-grammar parity only) respected?

3. **numeric-stem paths wiring (PHASE-03)** — All 9 kinds accept paths?
   Governance kinds share `governance::run_paths`? Per-kind thin dispatch?
   Multi-ref order preserved? Invalid ref → non-zero exit + empty stdout?

4. **umbrella/named paths wiring (PHASE-04)** — Prefix-to-sub-kind resolution
   correct for backlog/spec/knowledge? Memory uid/key resolution correct?
   Selector flags work across all 4 kinds?

5. **behaviour-preservation gate** — All existing show/list/status suites
   stay green? No regression in guard.rs or architecture_layering?

6. **SPEC-013 drift** — Design §6 / D7 names this as reconciliation work.
   Confirmed the spec update is deferred, not forgotten.

Invariants held to the design:
- Output: root-relative, one per line, no table/JSON/headers
- Selectors: default all-files → selected classes only when any flag present
- --single: truncates per ref to first path
- Atomic stdout: all refs succeed before any output
- No partial output on error
- show --json: no paths field added (D1)

## Synthesis

The SL-139 implementation is a clean, narrow delivery across four phases. The
shared paths helper (`src/paths.rs`) sits correctly in the engine tier with no
clap or command-tier imports, providing pure types (`EntityPathSet`,
`PathSelection`) and an impure scanner that respects the exclusion filter and
canonical ordering. All 13 in-scope entity commands gained a `paths` verb with
identical selector semantics — `--toml`/`-t`, `--md`/`-m`, `--entity`/`-e`,
`--single`/`-s` — sharing a single implementation spine.

**Closure story.** Every invariant from the design is satisfied. Output is
root-relative, one path per line, with no table/JSON/headers. Selector logic
toggles cleanly between default-all and selected-classes modes. `--single`
truncates per ref. Multi-ref splats preserve input order, and invalid refs
produce a non-zero exit with no partial output. `concept-map show --json` is a
true byte-identical shorthand for `--format json`, achieving show-parity without
touching JSON shape normalization (D8). The behaviour-preservation gate holds:
2304 tests pass, clippy is zero-warning, and the `architecture_layering` test
accepts `paths` in the engine tier.

**Standing risks.** None identified specific to SL-139. The shared helper is
narrow and well-tested; per-kind adapters are thin wrappers over
`paths::scan_entity_dir` + `paths::select_paths`. The 3 pre-existing test
failures (worktree marker env, knowledge round-trip) are unrelated and
pre-date this slice.

**Tradeoffs consciously accepted.**
- **No trailing newline guarantee.** Output joins with `\n` but does not
  append a final newline — consistent with how `show` works across doctrine.
  Shell pipelines (`while read line; do ...`) handle this idiomatically.
- **No golden test files in `tests/`.** The verification goldens are
  unit-test assertions within the per-kind modules (governance.rs tests,
  slice.rs tests, etc.), not separate e2e golden files. This matches the
  existing pattern for entity-verb testing in this codebase.
- **SPEC-013 amendment deferred.** The design recorded this intentionally
  (§6, D7). The reconciliation brief below maps the exact amendment needed.

## Reconciliation Brief

### Governance/spec (REV)

- **SPEC-013** (§ uniform verb set): The current wording describes
  `new/list/show/status` as the uniform verb grammar. `paths` is now a
  proven, working verb across all 13 entity commands. Amend SPEC-013 to
  include `paths` in the uniform verb set. Exact location: the verb-grammar
  section that enumerates the uniform CLI verbs. → REV modify
  (Finding: F-4)

### Per-slice (direct edit)

None. All design-to-implementation mappings are aligned. No per-slice
artifacts need correction.

## Reconciliation Outcome

### REVs completed

- **REV-007** (`reconcile-sl-139`): done — SPEC-013 amended to include `paths`
  in the uniform-verb set (covers RV-136 F-4). Rationale in revision-007.md.

  Change: `modify SPEC-013` — added `paths` to the shared verb set enumeration
  in the Uniform command grammar section.

### Direct edits applied

None.

### Withdrawn / tolerated

None.
