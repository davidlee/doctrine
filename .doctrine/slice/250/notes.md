# Notes SL-250: Retire the Claude plugin delivery channel

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-06 · design (pre-design research complete, no design run) · 48880f64

### Produced

- `slice-250.md` — scope, with every open question either settled inline or
  explicitly deferred. The settlements carry their evidence; do not re-derive.
- `research/` — three-thread round + four probes (runtime, gitignored, **not a
  durable sink**). Its load-bearing findings are mirrored into `slice-250.md`;
  anything cited from `research.md` alone dies with the folder.
- IMP-406 — serve non-Claude harnesses from the embed via `.agents/skills`.
  Sequenced `after` SL-250; carries the `.agents/skills` generalisation this
  slice deliberately does not ship.
- `mem.fact.claude.settings-hooks-merge-and-matcher` (`mem_019fd67cad37…`) — the
  probe results, recorded durably.
- Observation `019fd685-…` — RFC-011 instrumentation: the SPEC-010 / code
  divergence was found only by reading git history during research.

### Learned

- **Corrected `mem_019ec3392f247d53a1a4c910be8306aa`** in place. It claimed the
  merge core is "generalized over event+matcher" — true for *writing* and for
  `entry_is_canonical`, but `owned_positions` proves ownership by **command
  alone**. It also still cited the retired `HookSpec::stamp_subagent`. A future
  agent reading it would have concluded the multi-matcher case was handled.
- **SPEC-010 specifies code that does not exist** — the canonical-tree-plus-
  symlink Claude channel, deleted at `347197e8` with no amendment. Pre-existing
  divergence, independent of this slice; `OQ-2b` closes it. In the observation,
  not yet in a governance sink.

### Open

- Nothing blocking. Design may start.
- `OQ-2b`'s scope note: parameterise the mechanism over the target directory.
  Cited to `git show 347197e8^:src/skills.rs` in `slice-250.md` § `OQ-9`.
- `R5` — the merge-core ownership widening is the central design problem and the
  one Non-Goal re-draw. Everything else in the risk register is documented-not-
  engineered per the less-code posture.
- **At close:** IMP-400 does *not* close with this slice — its `OQ-4`
  (migrating existing `enabledPlugins` / marketplace registrations) is out of
  scope and keeps the item open. Mirrored in `slice-250.md` § Follow-Ups.
- **At reconcile:** REV targets are SPEC-011 (`REQ-186` + Responsibility 6) and,
  narrowly, SPEC-010. RFC-018 takes the harness field notes.
