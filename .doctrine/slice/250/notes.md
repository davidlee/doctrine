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

## Design surface triage (design run `dr-019fd692`, 2026-08-06)

Recorded for runbook step `explore.triage`. The inquiry map carries the
questions; this is the surface they sit on.

### Constraining governance (read, not merely cited)

- **SPEC-011 Responsibility 6 + `REQ-186`** — verified verbatim: `boot install`
  merges the `SessionStart` hook into `.claude/settings.local.json`. This is the
  real hook-write authority and the REV's primary target. Research `X2` confirmed.
- **SPEC-010 responsibilities** — verified: still specify "Claude materialises a
  derived canonical `.doctrine/skills/<id>` tree and reconciles a relative agent
  symlink into it". That code is deleted. Divergence confirmed at the source.
- **POL-002 facet (3)** — the doctor's per-user probe is permitted as a
  *feature-scoped capability*: opt-in, no default path acquires it, absence
  yields a message naming what was missing. Confirmed against the policy text.
- **ADR-019 position 1** — hook entries written into `.claude/settings.json` are
  a *projection*, so they need an explicit owner, a mutation policy, and a stated
  project-level reason. The owner-locked merge core supplies the first two.
- **ADR-019 position 2** — a published-but-not-projected `.claude-plugin/
  marketplace.json` is exactly the sanctioned posture. No Non-Goal conflict.
- **ADR-011 `D3`** — a Claude-only doctor leg is an honest per-harness altitude
  row, not a violation.
- **STD-001** — the project-scope path, any new matcher tokens, and the scope
  flag values all need named constants beside `SETTINGS_REL`.

### Two findings that resize the work

**T1 — the hook inventory is mostly unbuilt, not merely relocated.** Only ONE
`HookSpec` is live on the Claude arm today (`HookSpec::sync`, from
`src/corpus.rs:491`). `boot` and `create_fork` exist but are `expect(dead_code)`,
test-only. `boot_emit` is Codex-only (`SESSION_MATCHER_CODEX`). The four commands
`worktree nominate`, `worktree denominate`, `worktree pretooluse` and
`memory surface` have **no spec and no ownership predicate at all** — they exist
only as plugin JSON. So the hook leg builds ~6 new specs with predicates, not
"add a constructor".

**T2 — the plugin's `SessionStart` hook is stale relative to the live code.**
`plugins/doctrine/hooks/hooks.json` ships `${DOCTRINE_BIN:-doctrine} boot --emit`
on matcher `*`. The canonical current args are `RESOLVE_EMIT_ARGS`
(`prompt resolve --role orchestrator`, `src/boot.rs:862`) and the canonical
SessionStart matcher is `SESSION_MATCHER` = `startup|clear` (`:554`). The slice's
"the command strings port verbatim" holds for `${VAR:-default}` expansion but
NOT for which string is canonical — porting the plugin's copy would enshrine the
legacy form on a wider matcher. `is_doctrine_emit_command` already accepts both,
so this is a choice, not a compatibility break.

**T3 — the canonical-tree + proven-ownership-link pattern is live in three
places, not deleted.** `src/install.rs` runs it for agent defs
(`agent_canonical_dir` → `.claude/agents` / `.pi/agents` — already multi-target)
and for workflows (`workflow_canonical_dir` → `.claude/workflows`). The shared
helpers (`classify_link` :1918, `write_link` :1964, `relative_target` :1877,
`install_base` :1816) are already factored out; only the materialise step differs
(single file vs directory tree). `OQ-2b` restores the skills leg as a fourth
consumer — the no-parallel-implementation question is whether the *link phase*
gets one generic multi-target driver rather than a third hand-rolled loop.

### Assumptions carried

- `A1` (slice) — the 2.1.198 empirical findings are not re-derived. The probe ran
  on 2.1.220; no contradiction observed.
- The behaviour-preservation gate is satisfiable by construction if the merge
  core generalises rather than narrows — see the map's `inq-2`.
