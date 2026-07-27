# IMP-331: dispatch-agent arm-spawn template omits the mandatory --phase half

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

The `/dispatch-agent` skill's pre-spawn template reads:

```
doctrine dispatch arm-spawn --base <B> [--slice <N>]
```

`--phase` is absent entirely, and `--slice` is rendered optional. Both halves
are in fact required to bind a fork: the hook snapshots `(slice, phase)` into
the `DispatchRecord` one-shot at fork time, and **a half-arm binds nothing**.
The CLI's own `--help` says so; the skill contradicts it.

Source: `plugins/doctrine/skills/dispatch-agent/SKILL.md` (the shipped copy
under `.doctrine/skills/` is derived — edit the plugin, per
`mem.pattern.doctrine.skill-source-of-truth`).

## Why it costs a full worker turn

The failure is **end-loaded**. An unbound fork spawns, guards, reads, edits and
tests exactly like a bound one. It fails only at hand-back, when
`worker_commit` refuses `unprovable-fork` — after the entire worker turn has
been spent. Observed on SL-231 PHASE-04: ~265k tokens and ~28 minutes burned
before the refusal surfaced.

`mem_019f9effcf4a7922b31c1a1b37841d06` already documents the trap and its
recovery options. This item is about the *skill text that produces it* — the
memory is the safety net, the template is the hazard.

## Proposed fix

1. Correct the literal to `arm-spawn --base <B> --slice <N> --phase PHASE-NN`,
   both halves non-optional, with a one-line note that a half-arm binds nothing
   and fails end-loaded.
2. Consider making `arm-spawn` itself fail closed on a partial arm — refusing
   at arm time converts an end-loaded worker-turn loss into an immediate,
   free error. That is the durable fix; the skill text is the cheap one.

Related: RV-317-era SL-231 dispatch notes; ISS-218 / SL-225 for the adjacent
fork-gate class.
