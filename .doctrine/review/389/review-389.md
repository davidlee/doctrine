# Review RV-389 — code-review of SL-264

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Pre-audit code review of SL-264's implementation delta (PHASE-01..PHASE-05).

**Delta** — the SL-264 commits only, not the `d128f79..HEAD` range (other slices
interleave): `37d26a5ff` (PHASE-01), `a67060fe5` (PHASE-02), `24b38cd61`
(PHASE-03), `6db9eb66b` + `b91be7168` (PHASE-04), `e37cd62ab` (PHASE-05),
`1f8c21d10` (selectors). ~2.9k+/0.7k-, 26 files; ~1.3k non-test `src/design_run/*`.

**Authority** — the locked design (`sec-1`..`sec-7`), `DEC-300`/`DEC-301`/`DEC-302`,
the `RV-386` amendments (`F-1`..`F-18`), ADR-001, STD-001, STD-003. Deviations
already disclosed in `notes.md` (PHASE-02 EX file list, PHASE-01 VT-2 weak red,
PHASE-03→04 coverage hole, `sec-5` under-declaration, PHASE-01 missing boundary
row) are checked, not re-raised.

**Passes** — one per dimension; each finding title is prefixed with its dimension:

1. `[correctness]` — staleness per `Coverage` (`InquiryMap` vs `ReviewedGraph`);
   per-node legacy fallback; the derived blocking condition still fires (R1:
   loosening invalidation is a truthfulness change); stored snapshots parse and keep
   their verdicts (R2); `Sparse<bool>` + two-home key null/omit/create; change log
   agrees with the gate.
2. `[conformance]` — code vs design sections and DECs; VT/EX criteria actually
   evidenced; layering and standards; `install/*.md` mirrors vs code.
3. `[dry]` — parallel implementations, cohesion, naming, function length, legacy
   read-only surface area.
4. `[tests]` — brittleness, theatre, fixture duplication, golden pins, red-first
   honesty of each VT.

**Invariants held** — no condition that should invalidate silently stops doing so;
no stored snapshot becomes unreadable or changes verdict except where the design
says so; `needs: null` ≡ `needs: []`.

## Synthesis

**Overall**: acceptable — revision-required at raise, now reconciled; 18 findings,
all terminal (17 fixed, `F-5` follow-up to `IMP-474`).

**Synopsis.** SL-264 stops a change of inquiry-map shape from voiding the user's
attested judgements, derives the blocking set from a per-node `blocking`
attribute, and makes `needs: null` clear. Round 1 ran four dimension passes
(correctness and tests on codex; conformance and DRY on Opus). The production
logic of the narrowing held under every scenario tried. What was weak was the
*evidence* for it and one design promise:

- The two risks the design named were under-evidenced. R2 (stored snapshots stay
  readable and keep verdicts) had no test that parsed a stored snapshot (`F-8`,
  `F-13`); R1's change-log/gate agreement checked one of two acts (`F-10`,
  `F-14`). Both were proven blind by mutation, then closed with a frozen
  pre-change snapshot fixture whose verdicts were checked against a pre-change
  binary, and an agreement test over both acts and six cases.
- `NodeCreated` did not carry the creation's `blocking` judgement (`F-9`), so
  the change log could not say a new node was born blocking — the event the
  slice exists to surface. Fixed in code, not waived.
- DRY: `Coverage`'s carried shape was spelled in four places, the legacy class
  and the blocking-marks builder each had two definitions (`F-1`..`F-7`).

Round 2 (Opus, adversarial correctness) confirmed all repairs by re-running the
mutations, and found `F-16`: a delegated proposal's `null` is dropped by TOML
storage and replays as a persist, bypassing both this slice's `blocking: null`
refusal and its `needs: null` fix. Pre-existing mechanism (`SL-233`), but it
falsified two of this slice's published contracts. Closed by rehearsing a
proposal at `Propose` through the direct path's rules and refusing any `null`;
`F-18` then removed the resulting false refusal at a finding. `F-17` recorded a
`POL-002` breach the `F-16` repair itself introduced — `IMP-483` cited in a
shipped prompt and a user-facing refusal — surfaced by a concurrent audit.

**Standing risks / accepted tradeoffs.**
- A proposal cannot clear a field until stored proposals preserve `null`
  (`IMP-483`); the refusal is loud, so nothing is lost silently.
- Only inquiry subjects are rehearsed at `Propose`; checkpoint and section
  declarations need shell-minted inputs and are still first fully applied at
  `Accept`.
- The PHASE-05 fitness re-measure used a synthetic run (`F-11`), disclosed in
  `notes.md` for `/audit` to rule on.
- For `/reconcile`: `SL-233`'s `sketches/projection-bounds.md` §(d) still lists
  `node_created` with two terms.
- Process: the round-1 correctness pass returned zero findings with no scenario
  list — thin evidence, which is why round 2 re-ran correctness. Repair briefs
  must restate `POL-002` for shipped surfaces; this one did not.

**Haiku**

    the map may grow now —
    but a null, stored overnight,
    woke up as *keep it*
