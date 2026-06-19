# Review RV-103 — reconciliation of SL-118

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Candidate surface: `cand-118-review-001` (merge of `review/118` onto `main`,
tip `d1fa743b`).

### Lines of attack

1. **One complete validation rule per facet.** PHASE-01's finiteness fix — does
   `estimate::validate` now reject inf/nan? Is `value::validate` wired into
   `normalise` (one rule site, not parallel)? Do pre-existing suites stay green
   unchanged?

2. **Edit-preserving leaf purity.** PHASE-02's `facet_write.rs` — does it import
   only toml_edit/anyhow/std (ADR-001)? Does it mutate only managed keys,
   preserve siblings, fail-loud on malformed shapes, no-op on identical values?
   Is it clock-free (no `updated` bump per D1)?

3. **CLI surface correctness.** PHASE-03 — does `exact: Option<f64>` prevent
   illegal mode combinations at clap + handler? Does validation run before the
   leaf write? Is the Write class registered? Does clear-absent exit 0? Is the
   path-from-ref helper properly factored?

4. **Dogfood.** Is an estimate authored on SL-118 via the verb itself?

5. **Gate.** `cargo clippy` zero warnings, `cargo test` green on the candidate
   surface.

## Synthesis

SL-118 delivers what it set out to: CLI verbs (`estimate set/clear`,
`value set/clear`) that author the estimate and value facets through the
**existing** parse/validate rules and an edit-preserving `toml_edit` leaf.
The implementation is tight, well-tested (36 new tests across three test
modules, 1994 pre-existing tests green), and architecturally clean.

### What went right

**PHASE-01 made `validate` the single complete rule.** The adversarial
finding from the design pass (finiteness was missing from `estimate::validate`
and `value` had no standalone `validate`) was addressed before the CLI was
built. Finiteness now lives in `validate` for both facets, and
`value::validate` is wired into `value::normalise` — one rule site, no
parallel implementation. The behaviour-preservation gate held: zero
pre-existing test edits, all suites green unchanged.

**PHASE-02 delivered a lean ADR-001 leaf.** `src/facet_write.rs` (512 lines,
17 tests) imports only `toml_edit`, `anyhow`, and `std`. The pure cores
(`set_facet`, `clear_facet`) are generic over table-name + scalar fields,
serving both facets with zero code duplication. The triple-safety design —
mutate-in-place (not replace), malformed-present fail-loud, no-op guard —
is comprehensively verified (VT-1 through VT-8). The D1 reversal (no
`updated` bump) keeps the leaf clock-free and avoids the per-kind
seed-presence logic the original design would have required.

**PHASE-03 wired clean CLI verbs.** The subcommand groups
(`Estimate`/`Value` → `Set`/`Clear`) are ergonomic and well-validated.
The `exact: Option<f64>` with `conflicts_with_all` and handler-enforced
exactly-one-mode correctly prevents illegal combinations (F2 resolved).
The `allow_hyphen_values = true` on value magnitude allows negative values
without `--` gymnastics. The path-from-ref helper (`resolve_entity_path_and_canonical`)
was factored cleanly — a simpler sibling to the existing `resolve_link_path`.
Write class registration (`Write("estimate")` / `Write("value")`) is complete.
Dogfood: an estimate on SL-118 itself (lower=3.0, upper=3.0 via `-x 3`)
is present on the candidate surface.

### Standing risks

- **Candidate worktree has a pre-existing e2e failure**
  (`relation_rows_of_one_label_are_contiguous` in
  `tests/e2e_relation_migration_storage.rs`) caused by ISS-030 corpus-data
  dirt in the candidate's copy of `slice-101.toml`. This reproduces without
  SL-118 code and does not gate the slice.

- **Output format is a sketch, not a contract.** The design §3 output
  examples differ slightly from the implementation (colon placement, no unit
  display). No VT pins output format strings; the implementation's output is
  clear and consistent. Unit display would couple the handler to the config
  read path — sensibly deferred.

### Tradeoffs consciously accepted

- **Confidence authoring is blocked on SL-104.** The `--*-confidence` flags
  the original open question explored are deferred — the writer preserves
  unknown sub-keys (VT-7), leaving the door open.

- **History (IDE-013) is deferred.** The writer is forward-compat-ready
  (unknown sibling keys survive every set, VT-7), but no history array is
  written. This keeps the slice focused on authoring, not time-series.

- **No `updated` bump.** D1 reversed after adversarial review — the facet
  is a side-table; provenance lives in git + conventional commits.

### Closure story

Three phases, dependency-ordered: complete the validation rule (PHASE-01),
build the write leaf in isolation (PHASE-02), wire the CLI on top
(PHASE-03). Each phase was dispatched to an isolated worker; the candidate
surface merges cleanly onto main. 36 new tests pin every design VT.
`cargo clippy` is zero-warning; `cargo fmt` is clean. The slice is
ready for reconciliation and close.

## Reconciliation Brief

No per-slice or governance/spec edits are required — every finding was
aligned (observation correct, no change needed). The implementation is
coherent with the design, ADRs, and governing specs.

### Per-slice (direct edit)

(none)

### Governance/spec (REV)

(none)

## Reconciliation Outcome

All 4 findings were `aligned` — observation correct, no change needed.
No writes required. Reconcile pass complete — handoff to /close.
