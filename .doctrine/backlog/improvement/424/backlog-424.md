# IMP-424: Fold sl-248's stranded LOOP.md orchestration lessons into the loop template

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happened

`LOOP.md` diverged in two directions from the `edge`/`sl-248` merge base and was
resolved to `edge`'s copy at landing (merge `3953b74c3`), stranding the other
side.

**This item was opened naming three stranded commits. The real number was 21.**
`edge`'s copy was not a rewrite that dropped content — it was a cherry-pick of
`72ff933ce`, the one commit both lineages share, plus `SL-249`'s four
generalisation commits (~80 lines: slice-parametric subject, primary-worktree
posture, a `plan.toml` field correction, commit-per-task). `sl-248` made 21
further commits on that same base, +365 lines, none of which reached `edge`.

That inverted the plan recorded here: grafting 365 lines onto `edge` was the
wrong direction. The correction is in the resolution below.

## Resolution

`LOOP.md` was **rewritten as a merge of both lineages**, not replayed as
commits, and kept at the repo root as an explicit **template**: copy it to
`.doctrine/slice/<N>/LOOP.md`, bake the slice number in, customise § *Where this
runs*, and fire the loop at the copy. That also delivers `IMP-423`.

Recovered into the file — everything a cold firing must *act on*:

- the four-leg liveness guard with each leg's blind spot, replacing `edge`'s
  two-leg version, and the mechanics that make each leg actually work;
- **the notification is the only proof of completion; silence is never proof**;
- *any sign of life beats a cold sheet*, and the waiting-is-cheap-reaping-is-not
  asymmetry;
- **budget is NOT a stop condition** — `edge` still listed it as one, which is
  the regression that turns an autonomous loop back into a babysat one;
- hold orchestrator commits while a worker is live; `verify-vt` is an
  orchestrator instrument, not a worker self-check; the worker owns `notes.md`
  § *Owed* with the orchestrator verifying at beat 3; Traps live in `notes.md`.

Moved to the memory corpus instead — reusable past one slice, so not worth
re-reading every firing. § *Method* in the file points at each:

- `mem.pattern.dispatch.loop-death-triage-resume-vs-respawn`
- `mem.pattern.dispatch.brief-a-diagnosis-as-a-hypothesis`
- `mem.pattern.dispatch.budget-is-a-handover-condition`
- `mem.pattern.testing.convict-a-race-by-causality-not-repetition`
- `mem.pattern.testing.floor-a-destructive-instrument-before-aiming-it`

The race-evidence memory **corrects** an existing one
(`mem.pattern.testing.timing-tests-need-a-load-tally`): its "press it with
spinners" step cannot find a hazard on a channel the spinners do not touch.

## Outcome against the ~200-line budget

415 lines drafted → **298 landed**, against `edge`'s 294 and `sl-248`'s 557 —
so roughly `edge`'s length while carrying `sl-248`'s extra 365 lines of
material, most of it now in the corpus. The stated aim was ~220. Closing the
last ~78 means moving § *Sub-agent discipline* — the worker's contract — out of
the file, which conflicts with it being a template a slice copies and
customises, so it was left in. Open for a later call.
