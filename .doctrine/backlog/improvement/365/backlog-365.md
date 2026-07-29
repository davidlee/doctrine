# IMP-365: Pre-run VA grep criteria at plan time

A `VA-` criterion of the form *RUN: `rg -n '<pattern>' <paths>` — zero hits* is
authored from intent and never executed against the tree it will be run on. Four
times on SL-233 the criterion was **unsatisfiable or vacuous as literally
worded**, discovered at execution rather than at plan time:

- PHASE-11 `VA-3`;
- PHASE-13 (recorded in that phase's findings);
- PHASE-12 `VA-4` — `rg 'reviewer|posture' … | rg -i 'arg|flag|option|config'`
  stayed clean *by luck of layering*: `Option<…>` matches `option`
  case-insensitively, so one line carrying both `reviewer` and `Option` in the
  greped files turns the criterion red with nothing wrong;
- PHASE-10 `VA-3` — `rg 'Command::new|spawn|fork|exec|reqwest|tokio::process'
  src/design_run/ src/commands/design.rs` is **already red at head**: bare `exec`
  matches `execution` in three doc comments and `execute_checkpoint` at its
  definition and its call site. Five hits, none of them a spawn transport, none
  introduced by the phase the criterion governs.

The cost is asymmetric. Authoring the criterion costs one line; discovering it is
unrunnable costs an execution-time adjudication, a disclosure in the phase sheet,
and a reconcile item — and because criteria ids are immutable and the plan can
only append, the wrong wording is permanent and each instance is carried to close
rather than fixed.

**The improvement.** At `/plan` (and `/phase-plan`, which is the last chance
before the criterion is load-bearing), *execute* every `VA-` criterion whose text
is a runnable command, against the tree at head, and record the observed result
beside it. A grep criterion that is meant to be zero-hit and is not zero-hit at
head is either mis-worded or is asserting something the phase must first clean
up — and which of those it is, is exactly what plan time should settle. Word
boundaries (`\bexec\b`), anchors, and a positive control belong in the criterion
text, not in the execution-time rescue.

Related: `mem_019fa18161f47651af7687d8dccbbc67` (a negative grep result is
untrustworthy without a positive control) — this is the same failure mode one
step earlier, at authoring rather than at running.
