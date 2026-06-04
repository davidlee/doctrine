---
name: handover
description: Author or replace the disposable handover.md that bootstraps a clean start on the next slice phase.
---

# Handover

Write `.doctrine/slice/<nnn>/handover.md` — the disposable, gitignored "start
here" for the agent picking up the next phase. It is scaffolding for the next
session, not a durable record: durable facts live in `design.md` / `plan.toml` /
`notes.md` / `doc/memories/`; the handover only points at them and frames the
immediate work.

## When to use

- Closing out a phase, before the next agent starts.
- The current `handover.md` (if it exists) targets a phase that is now done.

## TODO

- [ ] Read the just-completed `state/.../phases/phase-NN.md` (findings,
      hand-forward) and the slice `notes.md` for durable decisions.
- [ ] Confirm the next phase's scope from `plan.toml` (EX/VT are authoritative).
- [ ] record any information worth durably persisting in `notes.md`, or as appropriate.
- [ ] Emit the sections below into `handover.md`
- [ ] Print the path to `handover.md` then STOP — do not start the next phase's code.

## Shape (sections to emit)

- **Where this is** — phase status ladder + commit refs; what is DONE, what is now.
- **The gate** — `no code without an approved plan`; first action is the phase sheet.
- **Read before you plan** — pointers to design/plan/notes + key source `file:line`s.
- **What the last phase built** — the surface to ride, not refork.
- **Next-phase scope** — EX/VT restated, plus watch-outs and any seam decisions.
- **Immediate next actions** — the literal `doctrine slice phase … in_progress`
  command, fill the sheet, TDD, commit, flip completed.
- **Environment notes** — build/gate/lint specifics; pre-existing dirty files to leave.
- **Artifacts / pointers** — scope / design / plan / notes / specs.

## Rules

- Gitignored + `rm -rf`-able — never put the only copy of a durable fact here.
- Link to durable artifacts, don't repeat their contents.
- Supersede stale notes explicitly; do not leave contradictory guidance.
- Terse. Fragments fine. The reader is an agent, not a human reviewer.
