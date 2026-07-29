# IMP-361: Derive the six remaining gate conditions instead of accepting a claim

SL-233 PHASE-12 made the four `reviewing -> locked` conditions **derived** from
the run's own review state, and refuses a payload that claims one
(`Refusal::DerivedConditionClaimed`, single-sourced at `Condition::is_derived`).
The other six are still satisfied by a caller-supplied `EvidenceDeclaration`
that `apply` binds to a section fingerprint.

At least two of the six are mechanically derivable today:

- `materialisation-current` — from `run.authored.watermark` against the
  fingerprint Doctrine reads for `design.md`; the comparison already exists as
  `observe_watermark` in `src/commands/design.rs`.
- `required-sections-exist` — from the section set itself.

The remaining four (`governing-context-recorded`, `initial-concerns-recorded`,
`blocking-inquiries-dispositioned`, `user-accepts-sufficiency`) are judgements or
user acts, so a claim is the honest mechanism for them — but
`blocking-inquiries-dispositioned` is arguably derivable from the inquiry map the
same way `blocking-findings-disposed` now is from the finding set.

**Why it matters.** A claimed condition cannot be the subject of a test that
proves anything about the machinery it names: PHASE-12's four-refusal criterion
(VA-2) is what surfaced this, because four tests each omitting one component
would otherwise all have re-proved `gate::advance`'s filter rather than the
components. The same weakness applies to every claimed condition.

Recorded as a disclosed asymmetry at `Condition::is_derived`, and in SL-233
`notes.md` § Learned. PHASE-12 owned one boundary and deliberately stopped there.
