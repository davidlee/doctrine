# ISS-206: Prompt cascade concatenates same-slot Framework+User twins instead of overriding

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Home:** SL-186 (prompt cascade resolver). Surfaced during IMP-197 preflight.

## Symptom

`doctrine prompt resolve --role worker` emits the `role/worker` hymn **twice**.
Visible in the baked claude subagent def (`.claude/agents/dispatch-worker.md`,
"Role guidance:" block) — the worker contract paragraph is inlined verbatim two
times. The install-time bake (`expand_worker_marker`, `src/install.rs`) faithfully
inlines whatever `resolve` emits, so the duplication originates in the resolver.

## Root cause

`install/hymns/role/worker.md` (Framework) and `.doctrine/hymns/role/worker.md`
(User overlay) are **byte-identical** and occupy the **same slot** (`role/worker`).
`prompt explain --role worker`:

```
role/worker  prov=Framework spec=(1,0) rank=1
role/worker  prov=User      spec=(1,0) rank=2 ★ WINNER
```

Both are active and **both concatenate**. Per hymns INV-2, every matching snippet
emits exactly once and **only an explicit `replaces` suppresses** — the User copy
carries no `replaces`, so it *appends* rather than overriding its Framework twin.
"★ WINNER" is only the equal-specificity provenance tiebreak (for `replaces`
resolution), not suppression. Result: a projected **exposed** overlay stacks on
its own Framework origin.

## Question for design

An exposed slot's User overlay is *projected from* the Framework snippet and is
meant to let the user **override** that default — not stack a second copy on it.
So a same-slot (same band+label) User snippet should **implicitly override** its
Framework twin (identity ⇒ replacement), not append. Options:

- implicit same-slot override: a User snippet suppresses the Framework snippet of
  the identical slot without needing `replaces` (band+label identity is the
  replacement signal);
- keep append semantics but dedup byte-identical bodies within a slot;
- require the projected overlay to carry `replaces = "<own slot>"` and fix the
  projector to write it.

First option looks right (matches the seal/expose intent) but touches INV-2/INV-3
semantics — design call.

## Impact

Cosmetic today (duplicated worker guidance wastes a few tokens in every baked
worker def and every `resolve` call), but it is a **correctness smell** in the
resolver's compose invariant that will mislead any future overlay author: editing
the User twin to diverge would then emit *both* the old Framework text and the new
User text. Fix before overlays are used in anger.

## Relates

- SL-186 (cascade resolver — the home); SL-187 (delivery/bake).
- Discovered by IMP-197 (worker prompt template) preflight.
