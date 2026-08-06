# Notes SL-250: Retire the Claude plugin delivery channel

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-06 · design run `dr-019fd692` @ stage `drafting` rev 38 · 6e454608d

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
- Design run `dr-019fd692` — opened, exploring runbook discharged, 10-node
  inquiry map declared, both exploring-edge gates cleared, now at `inquiring`.
  **The run holds the live question set; do not restate it here.**
- IMP-407 — the doctor leg, carved out of this slice (user, 2026-08-06).
  Carries `R2`, `R7`, `OQ-5`, and a second leg the carve-out surfaced.
- Observation `019fd6a7-…` — RFC-011: research-baseline restamp treadmill; a
  design run's own slice-card edits regress its `explore.research` step.
- **The `inquiring` stage, complete.** All eight blocking inquiries dispositioned,
  each carrying a settled record; two nodes stay deferred to IMP-407. Both
  runbook steps discharged, sufficiency accepted, stage advanced to `drafting`.
  - `DEC-161` — `HookSpec` carries an ordered matcher set; ownership stays
    command-only (settles `R5`).
  - `DEC-162` — nine entries, six specs, one scope dial; closes `T2`. Also
    absorbed `inq-3` by adoption rather than a duplicate record.
  - `DEC-163` — scope is a sticky `doctrine.toml` key, no `--scope` flag; the
    installer announces its target.
  - `DEC-164` — writing one scope evicts the abandoned one and reports it;
    `R10` engineered, not documented.
  - `DEC-166` — extract the link-reconcile helper; skills loops targets locally.
  - `DEC-167` — cutover order: install, then disable the plugin.
  - `DEC-171` — REV targets SPEC-011 / `REQ-186` alone; SPEC-010 closes by
    conformance.
  - `QUE-209` — REV requirement granularity, deferred to reconciliation.
- **`slice-250.md` reconciled against all eight decisions** (runbook step
  `inquire.scope`). Six passages contradicted them and were corrected: the
  Scope items 1–3, the merge-core Non-Goal, `R5`, `R10`, `OQ-1`, and the closure
  criteria. Research baseline restamped twice.
- **The `drafting` draft — seven sections, materialised to `design.md`** (956
  lines). `sec-1` activation architecture · `sec-2` the ordered matcher set and
  the seven specs · `sec-3` scope key + abandoned-scope sweep · `sec-4` the write
  seam and the retirement act · `sec-5` the skills channel and the extracted link
  trichotomy · `sec-6` cutover and docs · `sec-7` verification. The run holds the
  section set; do not restate it here.
- **Design-target selectors recorded** (runbook step `draft.selectors`,
  discharged): `src/boot.rs`, `src/install.rs`, `src/install_config.rs`,
  `src/commands/cli.rs`, `install/**`, `.doctrine/spec/tech/011/**`. The
  scope-relevant-only entries (`src/corpus.rs`, `src/doctor_checks.rs`,
  `src/commands/doctor.rs`, `plugins/doctrine/hooks/**`, `.claude-plugin/**`,
  `.doctrine/spec/tech/010/**`) are deliberately NOT design targets — each is
  read or verified, none is edited.

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
- **`T1`/`T2`/`T3`** — the three design-surface findings that resize the hook and
  skills legs. Written up in § Design surface triage below; `T2`'s drift is now
  IMP-407's Leg 2 and `T3` reframes `OQ-9` as a no-parallel-implementation
  question rather than a speculative generalisation.
- **The link-reconcile duplication already existed, twice.** The trichotomy match
  is byte-identical in `src/install.rs:2179-2191` (agents) and `:2279-2291`
  (workflows). `T3` framed skills as a *risk* of a fourth copy; it is really the
  third, which is what made `DEC-166` a DRY fix on live code rather than a
  speculative generalisation.
- **`T2` resolves mechanically, not by choice.** The plugin's `boot --emit` is
  `boot_emit`'s hook, not `HookSpec::boot`'s — `is_doctrine_boot_command`
  (`src/boot.rs:891`) requires the trailing arg to be literally `boot`. And
  `is_doctrine_emit_command` (`:932`) already recognises the legacy args to
  self-heal. So writing the canonical form retires the stale `*` entry with no
  migration step.
- **`design apply` silently swallows unknown payload keys.** `ApplyRequest`
  carries `#[serde(flatten)]`, which disables `deny_unknown_fields`, so a
  misspelled or invented top-level key is a no-op that still bumps the revision —
  it looks like it applied. Already recorded as
  `mem_019fd03e13397240b4eb05af218f5cf5`; hit again this session while probing
  for the disposition schema. Read `src/design_run/submission.rs`, don't probe.
- **The plugin-step fork, settled by the user (2026-08-06): delete them.** No DEC
  covered what happens to the ~150 lines that *perform* plugin activation
  (`select_marketplace_source`, `marketplace_action`, `claude_plugin_*`,
  `enable_key`, `parse_registered_source`, `refresh_failure_is_fatal`, plus their
  tests). Option (b) — keep them behind an opt-in — was rejected on `DEC-163`'s
  own argument: a re-enterable plugin path re-creates `R8`'s double-fire, and
  `R8` is the one risk doctrine cannot detect or reconcile. Written up in `sec-4`.
  Consequence: `--dev` (`src/commands/cli.rs:126`) goes with them — its sole
  consumer is `select_marketplace_source`. `[install] repo` survives; the npx
  delegate still reads it.
- **Correctness catch: the scope must resolve INSIDE `install_claude_hook`**, not
  be passed to it. Its second caller is `run_sync_install` (`src/corpus.rs:506`)
  — exactly the routine flagless install `DEC-163` argues about. As a parameter,
  `memory sync install` could omit it and re-create the entry in the abandoned
  file on every run, reintroducing `R10` as the treadmill `DEC-163` set out to
  prevent. Resolving inside makes that unspellable and `corpus.rs` needs no
  change at all. In `sec-3`.
- **`A3` is already satisfied** — `.gitignore:4` carries `!/.claude/settings.json`
  beside `/.claude/*`. No edit needed; the project settings file is already
  tracked here.
- **`.doctrine/skills/*` needs no `ensure_gitignored` call** — `install/
  manifest.toml:46` already lists it. But `ensure_gitignored`'s doc-comment still
  claims "`skills install` reuses this", which is stale and will send the next
  reader looking for a call site that should not exist. Flagged in `sec-5`.
- **Two DRY fixes on live code, beyond the DECs.** (1) Four of five ownership
  predicates in `boot.rs` are the same suffix-strip shape; four more were coming
  — they collapse onto one `is_doctrine_command(cmd, args)` helper.
  `is_doctrine_boot_command` is deliberately left alone (equivalent, but guards a
  spec nothing ships). (2) `SETTINGS_REL` → `SETTINGS_LOCAL_REL`: once doctrine
  writes either of two settings files, "the settings file" is an ambiguity
  someone will misread.
- **The vestigial "Hooks plugin leg" comment** at `src/install.rs:2295-2300`
  documents code that no longer exists. Not captured as a backlog item — it dies
  with `sec-4`'s deletion pass.
- **`R6` withdrawn on a false premise** (recorded on `slice-250.md`). It assumed
  retirement blinds `SpawnSeamSymmetry`; but the Non-Goals keep `plugins/` and
  `R9` needs the plugin working as the managed-policy escape hatch, so
  `hooks.json` survives and the check is untouched. Its closure criterion
  inverted from "input migrates" to "needs no change".

### Open

- **The inquiry set is CLOSED.** All eight blocking nodes dispositioned; two
  remain deferred to IMP-407. The run is at `drafting` — its live state is the
  section set, not the question set. Read it with `doctrine design resume 250`.
- **The draft is complete and materialised; the drafting runbook is discharged.**
  Next is the user's read of `design.md` and the advance to `reviewing`. Nothing
  in the draft is provisional on an unsettled question.
- **At reconcile:** `QUE-209` — does the REV widen `REQ-186` or add new
  requirements for the newly-governed hook set and the scope key? Deferred here
  by the user at the sufficiency gate; the REV is authored at reconcile, which is
  where requirement granularity is the natural call.
- **At close:** IMP-400 does *not* close with this slice — its `OQ-4`
  (migrating existing `enabledPlugins` / marketplace registrations) is out of
  scope and keeps the item open. Mirrored in `slice-250.md` § Follow-Ups.
- **At reconcile:** the REV target is SPEC-011 (`REQ-186`) **alone** — SPEC-010
  dropped out under `DEC-171`, since `OQ-2b` makes its responsibilities 3–6 true
  again rather than needing amendment; it is a conformance-verification target
  instead. RFC-018 takes the harness field notes.

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

## RV-348 round 1 — responder pass (design review of SL-250)

Eleven findings raised against `design.md` at run revision 39: 2 blocker, 4
major, 3 minor, 2 nit. **All eleven upheld**, each re-derived against the source
before disposing. Remediated at revisions 40 (adopt) / 41 (materialise); the
materialiser round-tripped the edits byte-identically.

### What the two blockers actually cost

`F-1` was not an arithmetic slip. Four counts live at four altitudes — ten plugin
entries, seven specs, eleven settings entries, seven printed hook lines — and
three of the four were stated wrong because specs and entries were conflated
throughout. The consequence landed in `sec-7`: the `VH` gate read "all seven
entries", so the slice's only load-bearing acceptance criterion was satisfiable
by a settings file four entries short. `sec-1` now carries a count ledger and
every other mention cites it.

`F-2` was an inverted impact claim hiding an unreported destructive write.
`src/corpus.rs:506` matches `install_claude_hook` against `RefreshOutcome`
directly, so the `HookWrite` return does not compile — but the compile break was
the cheap half. Both the `DEC-163` announcement and the `DEC-164` eviction rider
had been sited on `boot::wire`, which `run_sync_install` never enters, so
`memory sync install` would have swept the sibling silently. The seam is now a
shared writer both callers reach.

### The finding the review did not raise

Remediation surfaced a **POL-002 breach the ledger missed**, and the human author
ruled on it rather than the responder improvising: `HookSpec` bakes
`current_exe()`, and `DEC-163` flips the default scope to `.claude/settings.json`
— a committed file. SL-195 had already closed exactly this defect on `.mcp.json`
and left the invariant **baked ⟺ gitignored** behind, with Claude hooks left
baked *because* they were gitignored and an acceptance criterion reading *no
absolute host path in any tracked file*. The scope flip would have reintroduced
the breach in the same installer, one key over.

Settled by riding SL-195's own seam: the committed scope writes
`${DOCTRINE_BIN:-doctrine}` (proven live — the shipped `hooks.json` has carried
that form for all ten of its entries), `Local` keeps the baked path, and
`is_doctrine_program` gains one arm so five predicates own both forms. That arm
is what makes a scope switch *heal* — an abspath entry is rewritten in the file
being written and evicted from the file being abandoned — which is precisely
`is_doctrine_mcp_entry`'s posture.

Second ruling, same conversation: **`install_baseref` follows the scope dial**.
Left "unchanged" it would have written `worktree.baseRef` to
`settings.local.json` while eleven hook entries went to `settings.json`, so
doctrine would author two Claude settings files per install with only one swept.

### What a further pass should probe

The remediation is broader than the ledger, so the next pass has new surface
rather than a re-read:

1. **The command-form change is the biggest untested claim in the design.**
   `sec-2`'s `command_for` makes the hook command scope-dependent, which turns
   `HookSpec` from carrying a rendered `String` into carrying `exec` + `args`.
   Probe whether any consumer of `spec.command` was missed — `fallback_for`,
   `entry_is_canonical`, the `RefreshOutcome::Refreshed(command)` payload — and
   whether `EvictOutcome::merge`'s absorbing-`Unreadable` fold is right when
   different specs disagree.
2. **Ownership widening.** `is_doctrine_program` accepting the portable literal
   is a strict widening on paper. Attack it for a foreign command that becomes
   ours, and for the `Local`→`Project`→`Local` round trip.
3. **The class-(ii) test table in `sec-7`** was enumerated by reading, not by
   running. A pass that actually compiles the rename would find any site the
   table misses — `:5302` "and neighbours" is the loosest entry in it.
4. **Whether `install_baseref` following the scope needs its own eviction.** The
   design argues the stale copy is inert because the value is invariant. That
   holds only while nothing else ever writes `worktree.baseRef`.
5. `QUE-209` (REV granularity) remains deferred to reconciliation, untouched.
