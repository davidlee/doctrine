# IMP-362: Blank acceptance basis is refused at the lock gate but admitted at a checkpoint

DEC-088's acceptance basis is "concise and **required**". Two paths bind an
`AcceptanceAttestation` and they disagree on what required means:

- **the lock gate** (SL-233 PHASE-12, `design_run::run::apply`) refuses a blank or
  whitespace-only basis — `Refusal::AcceptanceBasisMissing`;
- **the checkpoint path** (PHASE-05, `plan_checkpoints` in
  `src/commands/design.rs`) binds whatever arrives, including `""`.

One type, one concept, two admission rules — and the weaker one is on the path
that can move a created knowledge record off its kind's default status, which is
the more consequential of the two.

**The obvious fix, and why PHASE-12 did not take it.** Strengthening
`AcceptanceAttestation::bind` to return `Result<Self, Refusal>` puts the check in
the one constructor both paths go through (one caller each today, so the ripple is
two lines). PHASE-12 declined because `bind` is PHASE-05's owned surface and the
change would land inside a completed phase without that phase's criteria covering
it. Do it here, deliberately, with the checkpoint path's tests as the evidence.
