# ISS-476: Design-run review pass RV is invisible, so agents mint a second

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

When a run enters `reviewing`, it mints its own RV as the pass slot (SL-244,
through DEC-086's journalled intent). Nothing the agent reads names that RV:
the envelope, `reviewing.md`, `/inquisition` and `/code-review` are all
silent. So agents treat it as some unknown other party's activity, `review
new` a fresh RV for the adversarial pass, and reach `review-disposed` with the
findings on the wrong ledger. `review-disposed` accepts only the run's own
pass (a `ForeignPass` refusal otherwise), and the slot can't be re-pointed.
The only ways through both misrepresent the review:

- *conducted* against the empty run RV, with a basis pointing at the real
  ledgers (a "conducted" claim over a pass with no findings); or
- *waived*, with a reason pointing at them (reads as "no review happened").

## Recurrence

It recurs on nearly every design run with an adversarial pass.

- 2026-08-16, SL-256: run minted RV-359; the pass ran on RV-360 (16/16
  verified). Two refusals spent finding out: `ForeignPass`, then the missing
  concluded-pass marker. Observation `01a00a7c-1918-79b0-b8f5-52a4b55b975c`.
- 2026-09-23 (user report): run RV-008 empty; the real passes are on RV-009 and
  RV-010. The agent offered "conducted against RV-008 plus an explanatory
  basis" as its recommendation.

## Fix directions (for the next agent to shape)

1. **Name the pass (cheap, likely sufficient on its own).** At `reviewing`, the
   envelope names the pass RV ("raise this pass's findings on RV-NNN; do not
   mint another"). `reviewing.md` says the same, and `/inquisition` and
   `/code-review` check for an existing design-pass RV before `review new`.
2. **Tighten IMP-467's lock example.** `reviewing.md`'s "a concluded pass names
   its RV instead" should read "the run's own pass RV". As written, it invites
   naming a foreign one.
3. **An engine decision.** Should a run accept, or adopt, several RVs as its
   pass? Examples: `conducted` naming any concluded RV that references the
   slice, or a verb that re-points or adds to the pass slot. This sits against
   DEC-125 / DEC-138 and IMP-392's unification; settle it there rather than
   loosening `ForeignPass` ad hoc.

Recommend 1 + 2 first; they may make 3 unnecessary.
