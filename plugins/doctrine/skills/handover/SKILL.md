---
name: handover
description: Use when handing work to a fresh context — first bring the durable harvest current, then emit a continuation at one of two weights: a printed continuation prompt (light) or a handover.md context packet (full). Use whenever a fresh agent would more efficiently continue.
---

# Handover

First capture, then continue: ensure the durable harvest is current, then emit
the transient continuation the next agent starts from.

Guiding principle: existing durable artifacts should already provide suitable
context for handover. This skill is to **improve token efficiency** with
transient guidance those durable artifacts elide — and to confirm they are in
good condition.

## Harvest first — always, both branches

Before emitting anything, confirm the capture pass is current:

- **Governing slice:** its `## Harvest` must be fresh — the `fresh-as-of` stamp
  matches the current lifecycle position (phase / stage + head commit). If
  stale, run `/harvest` first — do not re-survey what the harvest already swept.
- **Another governing artifact (RFC, spec, review …):** the capture pass still
  runs — route produced / learned / open to their sinks per `harvest.md` §5;
  orientation falls back to entity queries.

Phase status accurate and work committed (or its uncommitted state noted)
remain adjacent truth checks.

## The dial

One skill, two weights:

- **light** — print a continuation prompt; nothing written.
- **full** — write the `handover.md` context packet and print instructions
  pointing at it.

Default by context: phase boundary, dispatch conclude, or complex in-flight
state → full; a simple "continue this in a fresh context" → light. The user's
explicit ask overrides.

## Light: the continuation prompt

Print:

```
/route
(path to the governing artifact and the next obvious task)
```

If the next task is implementation-bound, include the specific artefacts to read
(`design.md`, `plan.toml`, the active runtime phase sheet) and name any
unresolved assumptions or design questions the next agent must assess before
declaring readiness. Have the continuation prompt **cite the open ids** from the
governing slice's `## Harvest` — the open DEC / QUE / ASM (decisions / questions
/ assumptions) — so the next agent inherits the live open items, not a stale
story.

## Full: the context packet

Write `.doctrine/*/<nnn>/handover.md` — the disposable, gitignored "start
here" for the agent picking up the next Phase or otherwise continuing the work.

It is scaffolding for the next session, not a durable record: durable facts
live in persisted artifacts, `notes.md` or the memory store (`doctrine
memory`); the handover only points at them and frames the immediate work.

For a phase:

- [ ] Read the just-completed `state/.../phases/phase-NN.md` (findings,
      hand-forward) and the slice `notes.md` `## Harvest` section for the
      produced / learned / open ids.
- [ ] Confirm the next phase's scope from `plan.toml` (EX/VT are authoritative).
- [ ] **At a dispatch conclude (SL-170 S6):** embed the `slice verify-vt <id>` VT
      summary block **and** a one-line S1 regression status (from the verify
      beat's `check regression diff`) in the packet — surfacing any
      `UNCHECKABLE` / `WAIVED` / `Fail` gap at handover, not at audit.
- [ ] Emit the sections below into `handover.md`
- [ ] Print instructions (with path to `handover.md`) addressed to the next agent

For another artifact:

- [ ] adapt as appropriate — the harvest-first step above still applies
- [ ] author or replace `handover.md` in the most relevant artifact's folder
- [ ] Print instructions (with path to `handover.md`) addressed to the next agent

Then: STOP

### Shape (sections to emit) for a phase packet

- **Where this is** — phase status ladder + commit refs; what is DONE, what is
  now.
- **Reading List** — pointers to design/plan/notes, relevant other artifacts /
  code, key source `file:line`s. What the next agent should ingest to build
  context efficiently.
- **Artifacts / pointers** — scope / design / plan / notes / specs. Optional /
  reference material.
- **Terrain** — the surface to ride, not refork; e.g. what the last phase
  built. Point to it described elsewhere, or summarise if it is not.
- **Next actions** — the literal `doctrine slice phase <ID> PHASE-NN --status in_progress`
  (status is the `--status` flag, not a trailing positional) / other obvious and
  immediate next step(s).
- **Procedure & Caveats** — risks, unknowns and open decisions. Process
  guardrails (command, fill the sheet, TDD, commit, flip status, etc).
- **Environment notes** — build/gate/lint specifics; pre-existing dirty files
  to leave.

## Rules

- handover is gitignored + will be deleted — never put the only copy of a
  durable fact here.
- Link to durable artifacts, don't repeat their contents.
- Supersede stale notes explicitly; do not leave contradictory guidance.
- Terse. Fragments fine. Reader is an agent, not human reviewer.
