# Review RV-378 — code-review of SL-261

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Cadence: **per-phase** review of PHASE-05's uncommitted working tree on `edge`
(the executing-agent hand-back carried `git diff --stat` only, no test/verification
evidence). Depth: full pass — the diff is ~670 lines and the phase's own risk is
"a migrated test that quietly asserts less".

Governing artifacts loaded: SL-261 `plan.toml` PHASE-05 (EX-1..EX-4, VT-1..VT-3,
VA-1), design `sec-2` *Refusal wording*, `sec-3` *Pure core*, `sec-4` *Retiring
adopt_authored*, and the runtime phase sheet (`A1`–`A9`, `D1`, R1–R3).

Lines of attack:

1. **Assertion survival (R1/R3/A4/A5).** For every migrated test, read old and new
   side by side; the oracle must be preserved. Verified by `git show HEAD:<f>`
   diffing the `#[test]` fn-name set (renames only, no deletions) and reading each
   migrated body. The A2 parser-readout probe is the sharpest risk: does
   `unchanged == every declared id` really equal the retired digest-map oracle?
2. **Retirement wiring (`DEC-243`/`DEC-278`).** `RETIRED_KEYS` row → owner-identity
   lookup (`std::ptr::eq`, `PAYLOAD` now `static`) → `RetiredPayloadKey` before
   `UnknownPayloadKey`; the wire path must never accept-then-ignore the key, and
   the pass-through must be `Crossing::Ordinary` only.
3. **Behaviour preservation.** `adopt` loses the `markers` arg and
   `refuse_invalid_markers`; the locked backstop and `--expect` CAS must be
   untouched; the wire `apply` must never parse authored sections.
4. **Counts (A6).** `WRITER_ACTS` 9→8, the delegation table 9→8, `ACT_KEYS`
   10→9, contract closure 12→11 structs / 25→24 types — and nothing else. Any
   *other* count that moved with the deletion is a candidate.
5. **Vocabulary sweep (T9/VA-1).** `adopt_authored|AdoptAuthored` must survive
   only as the roster row, its tests, the retired-key e2e and the regenerated
   contract; and the doc comments the deletion orphaned must not now state
   falsehoods.
6. **Gate.** `doctrine check gate` exit 0; clippy zero warnings; the three e2e
   suites and `--bin doctrine` green.

## Synthesis

**Overall: solid.**

PHASE-05 retires the `adopt_authored` payload crossing by deletion, not by a
shadow path. `AdoptAuthored`, `ApplyRequest.adopt_authored`, `ADOPT_AUTHORED` and
its `PAYLOAD` row, `WRITER_ACT_ADOPT_AUTHORED`, `Refusal::AdoptionMarkersInvalid`
and `refuse_invalid_markers` are gone; `run::apply` no longer reads a request
field, and the wire shell hard-codes `Crossing::Ordinary` so only the `design
adopt` verb can adopt. The retirement rides PHASE-01's roster: `RETIRED_KEYS`
(payload_contract.rs:1525) now carries the real row, matched by node identity
(`std::ptr::eq`, with `PAYLOAD` promoted to `static` so the address is stable),
and it fires `RetiredPayloadKey` ahead of `UnknownPayloadKey`, naming the verb as
its remedy. The e2e proves the real row end to end (`adopt_authored_is_refused_
as_retired`, snapshot bytes unchanged). The two refusal texts now name
`doctrine design adopt SL-N` with the live slice id from
`crate::listing::canonical_id`, and no source string outside the roster row, its
tests and the regenerated contract still says `adopt_authored` (VA-1 clean).

Where it stands: the migrated suites preserve their oracles. No `#[test]` fn was
deleted — the two changes are renames plus A5(i)'s sanctioned leg (b) drop — and
the `parser_readout` idiom is genuinely as strong as the digest map it replaces:
held fingerprints *are* the declared bodies' digests, and a never-diverged probe
short-circuits to the no-op and emits no `unchanged` line, so the
`unchanged == every declared id` equality cannot pass vacuously. Counts moved by
exactly one where the phase sheet said they should (`WRITER_ACTS` 9→8, the
delegation table 9→8, `ACT_KEYS` 10→9, closure 12→11 structs / 25→24 types), the
contract was regenerated and is pinned, and `doctrine check gate` exits 0.

Four findings, none above `minor`; all four were `fix-now` acked and applied on
the hot tree, then verified (`RV-378`, `done`). F-1 renumbered — by naming, not
re-counting — the act-level doc ordinals that drifted when the tenth act field
left. F-2 corrected a stale seam justification. F-3 made both `parser_readout`
helpers self-check A2's third conjunct (`head:`) instead of two-thirds of it. F-4
single-sourced `unchanged_ids` into `tests/common/mod.rs`, the same move the
module already records for `sha256`.

Consciously accepted, no raise: the retired-key remedy keeps the literal
`<slice>` placeholder (the roster is wire-independent and has no slice to name);
the materialise read-back remedy names the verb and the slice without the
`--dry-run --diff` hint (design `sec-2`'s materialise row does not ask for it,
though EX-3's shared parenthetical could be read to); and the untracked
`.doctrine/observations/records/41/…toml` friction record the worker left is the
instrumentation contract working as intended, not phase scope.

Standing risks unchanged by this phase: `DEC-279`'s bare-adopt residual (an agent
that adopts unseen bytes) and the `DEC-100` materialise lost-update window. A8's
hymn / `drafting.md` / memory / SPEC-029 hits stay for PHASE-06.

Haiku —

*the wire key lets go;
the engine derives, the caller
confirms what it reads.*
