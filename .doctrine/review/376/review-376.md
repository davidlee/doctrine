# Review RV-376 — code-review of SL-261

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Subject: `149b16c33` — SL-261 PHASE-02 (`refactor(SL-261): split the apply
pipeline and add the crossing mode`). A behaviour-preserving refactor of
`commands::design::apply` into `parse_payload` + `apply_pipeline`, with the pure
`design_run::run::apply` switching adoption on a `Crossing` rather than the wire
`adopt_authored` key.

Lines of attack (the user's steer — where a subtle ordering mistake would hide):

- **EX-3, the one-read claim.** On `Adopt`, does `apply_pipeline` really derive
  the fingerprint and `authored_sections` from *one* `read_design_doc`? RV-374
  F-1 (blocker) was the two-read window: an edit landing between the fingerprint
  read and the section read seats B's bodies under A's watermark when the file
  returns to A before the pre-write re-check. Probe `AuthoredRead`/
  `read_authored` and every `design.md` read on the path.
- **EX-4, the step order.** Is the pipeline's order still `admit` → divergence
  (Ordinary) → pass 1 → DEC-250 mint hoisting → mints → pass 2 → rebaseline
  (Adopt) → journal → pre-write re-check → snapshot? Read code, not the digest.
  The implementer's `notes.md` discloses one deviation: the entry read now
  precedes `admit`.
- Also probed: the transitional wire mapping (`adopt_authored` →
  `Crossing::Adopt { expect: Some(declared) }`, EX-6); the VT-2 falsification
  (the crossing, not the request, decides adoption); scope (nothing under
  `tests/`, run.rs hunks only mechanical `&Crossing` additions); and the gate.

Evidence gathered before raising: all 11 `e2e_design_*` suites green
(2,024 tests passed, 11 ignored), the PHASE-02 unit test green,
`architecture_layering` green, `cargo clippy --bin doctrine` clean. The tree is
the primary worktree on `edge` at `149b16c33`.

## Synthesis

**Overall**: acceptable

**Synopsis**: PHASE-02 splits `commands::design::apply` into `parse_payload` and
`apply_pipeline`, and lifts the adoption switch out of the wire request into
`design_run::run::Crossing`. The two criteria the review was pointed at both
hold.

*EX-3 (one read).* `read_authored` performs the sole `read_design_doc` on the
path, and its `AuthoredRead { text, fingerprint }` carries both the bytes and
the fingerprint of those bytes. `apply_pipeline` takes that value as an argument
and derives both `observed` and `authored_sections` from it, so RV-374 F-1's
two-read window is structurally closed: there is no second read that an edit
could land between, and the pre-write re-check remains the one intentional
second observation (`read_authored_fingerprint`, basis `AdmittedAt`).

*EX-4 (step order).* `admit` → divergence (Ordinary) → pass 1 → DEC-250
`refuse_unresumable_mints` → mints → pass 2 → rebaseline (Adopt) → journal →
`pre_write` → re-check → snapshot → `complete_journal` is intact, checked
argument-for-argument against the pre-refactor body (`149b16c33^`), so the design
review's second probe — DEC-250 mint hoisting in code — also holds.

👍 The VT-2 test falsifies correctly: identical inputs adopt under
`Crossing::Adopt { expect }` and are left alone under `Crossing::Ordinary`, so
deleting the switch reds the test. The transitional wire mapping (EX-6) is
faithful — `adopt_authored` still crosses with its declared fingerprint, and
`WRITER_ACTS` still counts the key until PHASE-05 retires it, which is right
while it can still ride beside a delegation.

*Standing risks*: none blocking. F-1 (nit, **tolerated**) records the disclosed
ordering change — the entry read now precedes admission, altering error
precedence only on a replay whose `design.md` is unreadable. F-2 (nit,
**design-wrong**) records plan/implementation type drift on
`AdoptionStale.expected`; the code is right and the plan criterion over-specifies.

*Behaviour-preservation evidence*: all 11 `e2e_design_*` suites green (2,024
passed), the new pure test green, `architecture_layering` green, clippy clean;
no file under `tests/` touched, and the `run.rs` test hunks are mechanical
`&Crossing::Ordinary` additions only (VA-1 confirmed by diff inspection).

*One unguarded property, named but not raised*: the one-read rule has no
regression test, and cannot easily have one — the guarantee is structural (one
constructor, one read) rather than observable at a seam, and the plan verifies
EX-3 by code read. That is a reasonable verification choice, not a finding.

**Haiku**:

the crossing decides
one read, one fingerprint
not what it re-reads
