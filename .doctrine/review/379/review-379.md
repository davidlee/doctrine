# Review RV-379 — reconciliation of SL-261

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-261 (`doctrine design adopt`, retire `adopt_authored`),
all six phases landed on `edge` (not dispatched; no candidate branch). Surface
reviewed: `edge` at the PHASE-06 tip.

Lines of attack:

1. **Path conformance** (`doctrine slice conformance 261`): attribute every
   undeclared path to its commit; separate SL-261's own touches from SL-263
   commits that ride the contiguous phase ranges.
2. **Criteria**: every `VT-` keyword present in its test file (script over
   plan.toml); `VA-` criteria re-run (PHASE-05 VA-1 grep; PHASE-06 VA-1/VA-2).
3. **Design vs code**: sec-2 refusal wording, sec-3 pure core, sec-4 retirement,
   sec-6 code impact, sec-7 verification, each against the landed tree.
4. **Prior ledgers**: RV-374/375/376/378 terminal; deferred items (ISS-477,
   IDE-056, ISS-478) captured.
5. **Gate**: `doctrine check gate` exit 0 at the PHASE-06 tip (8002 passed).

Invariants held: one read of design.md per adopt; one pipeline; the wire
passes `Crossing::Ordinary` only; `similar` confined to the shell; no test
weakened (each phase's removed-test set checked on hand-back).

## Synthesis

**SL-261 conforms.** The slice delivers what the locked design says: one
crossing of an authored divergence, `doctrine design adopt [--expect]
[--dry-run] [--diff]`, and the `adopt_authored` payload key deleted and
retired through the roster, so a stale payload is refused with a remedy that
names the verb. The load-bearing invariants hold in the tree: `run_adopt`
reads design.md once and carries that read through the one pipeline; the
aligned no-op precedes admission and writes nothing; the locked refusal
precedes any parse and is backstopped in the pure core by the same predicate;
the wire passes `Crossing::Ordinary` only; `similar` lives in the shell alone.
Every VT keyword across the six phases is present in its test file, the
PHASE-05 VA-1 grep leaves only the roster row, its tests and the regenerated
contract row, and `doctrine check gate` exits 0 (8002 passed).

The findings are bookkeeping, not behaviour. Two (F-1, F-2) are the selector
registry and sec-6 table falling short of the paths the slice actually
touched — each touch is in scope. Two (F-5, F-6) are the design lagging two
decisions made during execution: the pure fn's rename, and the aligned-probe
idiom that PHASE-05 had to supply because the design's "probes move to
--dry-run" did not survive the aligned short-circuit as written.

**Accepted tradeoffs.** F-3: a concurrent slice on shared `edge` puts foreign
commits inside SL-261's PHASE-06 range; the one-contiguous-range model cannot
exclude them, and they are attributable by commit. Earlier: RV-376 F-1 (the
wire's one document read precedes admission) stays tolerated.

**Standing risks.** Client projects' installed guidance and memories teach
`adopt_authored` until they reinstall; the retired-key refusal names the verb,
which is the designed mitigation (design sec-7). ISS-477 (ordinary mutations
on a locked run remain accepted), IDE-056 and ISS-478 are deferred and
captured.

## Reconciliation Brief

### Per-slice (direct edit)

- F-1: design.md sec-6 table — add rows for `src/design_run/document.rs`
  (`dropped_head`), `src/design_run/snapshot.rs` (`unchanged_since`,
  `reordered_since`, `changed_since`), `src/commands/guard.rs` (`write_class`
  Adopt arm), `tests/common/mod.rs` (`unchanged_ids`), `Cargo.lock`. Mirror
  only — the load-bearing change is the selector verb below.
- F-2: design.md sec-6 memory row — name all five memories (add
  `mem_019ff439…`, `mem_019faca1…`); add rows for SPEC-029 / REQ-434 via
  REV-057. Mirror only.
- F-5: design.md sec-3 snippet and sec-6 `run.rs` row — `adopt_authored(next,
  expect, derived)` → `adopt(next, expect, derived)`.
- F-6: design.md sec-7 (Verification) — the parser-readout probe on an aligned
  document prepends one blank line, then `adopt --dry-run`; oracle: no
  `section_fingerprint_changed` row, every id in `unchanged`, the `head:` line
  (the `parser_readout` helpers).

### Selector registry (load-bearing for F-1, F-2)

- `doctrine slice selector add 261` (design-target): `src/design_run/document.rs`,
  `src/design_run/snapshot.rs`, `src/commands/guard.rs`, `tests/common/mod.rs`,
  `Cargo.lock`, `.doctrine/spec/tech/029/spec-029.toml`,
  `.doctrine/requirement/434/requirement-434.toml`, and the five memory dirs
  under `.doctrine/memory/items/`.

### Governance/spec (REV)

- None outstanding. REV-057 (SPEC-029 command family + REQ-434 criterion) is
  approved, applied and done.
