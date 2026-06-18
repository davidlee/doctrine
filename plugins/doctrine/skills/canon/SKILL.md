---
name: canon
description: Use before making architectural or workflow choices, when correctness depends on project governance, or when entering an unfamiliar subsystem — to load the project's authoritative rules before you act on assumptions.
---

# Canon

Understand project governance before you act, so you do not spread heresy.

## Articles of truth

Identify the authorities that bear on your task, then read the ones that carry
meaning for it — not all of them, every time:

- `CLAUDE.md`, `AGENTS.md` — standing instructions and conventions.
- `.doctrine/adr/` — project-global ADRs (authored; status lives in
  `adr-nnn.toml`). List them: `doctrine adr list`. Read the relevant bodies.
- `.doctrine/spec/tech/` — authoritative technical specs (the *how*).
- `.doctrine/adr/` — authoritative decisions.
- the governing slice's `design.md` — canon for *this* change's design intent.

For subsystem-level truth (gotchas, patterns, invariants tied to files or
commands), `/retrieve-memory` rather than rediscovering it.

**Read entities tier-aware — via `show`, never a single raw file.** The storage
rule + read-via-`show` discipline are resident in the boot digest and detailed in
`using-doctrine.md`. Reading one tier and concluding "empty" is false witness.

The boot snapshot you are reading (`@.doctrine/state/boot.md`) was inlined
at session start. If you just edited governance (`governance.md`, an ADR, a
memory) and need *this* context to reflect it, run `doctrine boot` to regenerate
the snapshot, **then** `/clear` or restart — regenerate THEN clear. `doctrine
boot` alone cannot refresh an already-inlined prefix, and `/clear` alone serves
the pre-edit snapshot.

## The mortal sins

Beware — and refuse to commit:

- **assumption** — acting on what you guessed instead of what you confirmed.
- **guesswork** — inventing ids, command shapes, or file locations.
- **duplication** — a parallel implementation where a seam already exists.
- **architectural infidelity** — violating the design or an ADR.
- **heretical coupling** — new imports or dependencies that breach boundaries.

If governance is missing, contradictory, or ambiguous on a choice that matters,
stop and `/consult` rather than normalizing around a guess.
