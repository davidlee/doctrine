# Name the unified `doctrine show` router in the remaining agent-facing guidance

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Where this comes from

The remainder of `RV-384`'s `F-18`, found while reviewing `SL-265` (*kind-blind
`doctrine show <REF>`*). `SL-265` brings only the two surfaces an agent reads at
the moment it chooses a verb — `install/using-doctrine.md`'s verb table and the
`install/routing-process.md` guardrail (the boot digest's own source). Every
other agent-facing surface still prescribes the per-kind form, so an agent
following them reaches for `<kind> show` and never the router.

## The surfaces, verified 2026-09-25

- `install/authority-model.md:22` — "Read via `doctrine <kind> show`"
- `install/agents/claude/capsule-phase-planner.md:28`
- `.pi/skills/canon/SKILL.md:19,29` and `.pi/skills/walkthrough/SKILL.md:103`
- Shipped memories: `memory/mem_019ec92b10…`, `mem_019e9a11b3…`,
  `mem_019e9a12b0…` — plus four local items under `.doctrine/memory/items/`

## Why it was deferred rather than folded in

Two reasons, and the second is the load-bearing one:

1. **Scope.** `SL-265`'s design boundary excludes docs; folding the sweep in would
   have amended that boundary and its selectors for surfaces the change is not
   about.
2. **A different mechanism.** The skills and memories are not reference docs. A
   shipped-memory edit needs a rebuild (RustEmbed), then `doctrine memory sync`,
   then `doctrine install` to refresh an installed copy — a heavier and separately
   verifiable change than a prose edit.

## Note on what is NOT stale

The CLI's own `--help` is generated from the command tree, so the router appears
there for free — that surface needs nothing. Only the authored prose and the
memories carry the old form.

## Verification

`SL-265`'s own amendment should state the tradeoff; this item is the other half.
When it lands, re-grep the four surfaces above plus `install/` and `memory/` for
`<kind> show`, and confirm the boot snapshot regenerates with the router named.

## References

- `RV-384` `F-18` — the finding this is the residue of
- `SL-265` — the router that makes the old form redundant
- `install/routing-process.md:58` — the guardrail, in scope in `SL-265`
