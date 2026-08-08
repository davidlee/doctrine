# Notes SL-250: Retire the Claude plugin delivery channel

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-07 · design run `dr-019fd692` @ stage `locked` rev 51 · ebe741aa6

### Produced

- `slice-250.md` — scope, reconciled twice: once against the eight inquiry
  decisions (`inquire.scope`), once against what the review accepted
  (`review.scope`, `167ce6a03`). The settlements carry their evidence; do not
  re-derive.
- `research/` — three-thread round + four probes (runtime, gitignored, **not a
  durable sink**). Load-bearing findings are mirrored into `slice-250.md`;
  anything cited from `research.md` alone dies with the folder.
- `design.md` — seven sections, materialised. The run holds the section set and
  the decision set; do not restate either here. Read it with
  `doctrine design resume 250`.
- **Design run `dr-019fd692` is LOCKED at rev 51.** Policy switched to
  `adversarial-only` at rev 45 on DEC-074's human-proxy grant, which made the
  seven RV-348 lane attestations satisfy the required lane; reviewing runbook
  discharged 3/3 (46–48); RV-348 disposed **conducted** (49); design accepted
  (50); stage moved (51). The lock prints its own caveat — *an auditable agent
  claim of user acceptance, not authenticated proof of a human act*.
- **RV-348** — external design review at rev 39, two rounds, 17 findings, all
  terminal, none contested. **Concluded** 2026-08-07 (the marker's first real
  use). Round detail below; the ledger holds findings and responses.
- **IMP-392's concluded-pass marker, carved out and landed** (`f3ed222ae`).
  `[review].concluded` + `doctrine review conclude` + `review_conclude` (MCP);
  `read_pass_facts` reads it in place of a hard-coded `false`, which is what
  makes a `Conducted` disposition reachable at all. IMP-392 stays **open** on
  the section reference, the RV resolution, the `Finding` retirement and the
  severity summary.
- Backlog minted: IMP-406 (`.agents/skills` from the embed, `after` SL-250) ·
  IMP-407 (the doctor leg, carrying `R2`/`R7`/`OQ-5`) · **CHR-057** (retract a
  `needs` edge — see Open).
- Memories: `mem.fact.claude.settings-hooks-merge-and-matcher`
  (`mem_019fd67cad37…`, the probe results) · **`mem.fact.doctrine.needs-axis-append-only`**
  (`mem_019fd9a3c36a…`).
- Observations (RFC-011): `019fd685-…` SPEC-010/code divergence found only via
  git history · `019fd6a7-…` research-baseline restamp treadmill ·
  `019fd72c-…` `/design` omits the positional `<SLICE>` · `019fd989-…` the
  `Conducted` arm was unreachable, so a conducted pass could only be recorded as
  waived · `019fd99f-…` the `needs` axis has no retraction verb.
- **Gate:** `doctrine check gate` exit 0 after the last code change. All
  `.doctrine` changes committed alongside — nothing pending.

### Learned

- `mem_019fd9a3c36a…` — the `needs` axis is append-only and `unlink` does not
  reach it. Read it before hunting for a retraction verb.
- **Corrected `mem_019ec3392f247d53a1a4c910be8306aa`** in place — it claimed the
  merge core is generalized over event+matcher; `owned_positions` proves
  ownership by **command alone**.
- **SPEC-010 specifies code that does not exist** — deleted at `347197e8` with no
  amendment. Pre-existing divergence; `OQ-2b` closes it by conformance.
- **`design apply` silently swallows unknown payload keys** (`ApplyRequest` has
  `#[serde(flatten)]`, which disables `deny_unknown_fields`) — already
  `mem_019fd03e13397240b4eb05af218f5cf5`. Read `src/design_run/submission.rs`,
  don't probe.
- **The `Verb`/`TurnAct` split** — `Verb` is the finding-transition vocabulary
  (`can` / `required_for` / `gate` are keyed on a `FindingStatus`), so a
  pass-level act rides a sibling type rather than a sixth variant. Written up on
  IMP-392; the reasoning generalises to any future non-finding review act.
- **A committed default scope makes the baked exec path a POL-002 breach —
  SL-195 had already ruled on it.** Not raised by RV-348; found while
  remediating. `baked ⟺ gitignored` survives via `CommandForm { Baked, Portable }`.
  Detail in `design.md` § The command form and in `slice-250.md` Scope item 1;
  the POSIX-shell boundary that rides with it is in § Ruled, not designed below.
- **Design-review findings can be right about the claim and wrong about the
  address.** Three of nine helper citations in `sec-5` were off, one by 1837
  lines, while the table's argument held. Re-derive line refs at materialise
  time; a design's citations rot faster than its reasoning.
- **The raiser's pattern across both RV-348 rounds:** the mechanism is right and
  the sentence attached to it overclaims by one degree. An implementer reads a
  discharged-analysis sentence as discharged, so the round-2 remediation fixed
  sentences as well as mechanisms.

### Open

- **CHR-057 — ISS-314 reads blocked on delivered work.** Its `needs: IMP-392`
  was for the marker alone; the marker landed and the edge cannot be retracted.
  The correction lives in prose on IMP-392 until a verb exists. `after` has the
  same gap.
- **Next: `/execute`.** The plan is authored and the six phase sheets are
  materialised; the slice is `ready`. `/phase-plan PHASE-01` expands the runtime
  sheet just before execution.
- **At reconcile:** `QUE-209` — does the REV widen `REQ-186` or add new
  requirements for the newly-governed hook set and the scope key? The REV target
  is SPEC-011 (`REQ-186`) **alone**; SPEC-010 is a conformance-verification
  target under `DEC-171`. RFC-018 takes the harness field notes.
- **At reconcile — one design patch, deliberately deferred (user, 2026-08-08).**
  `design.md` `sec-2` places `ClaudeSettingsScope::command_form()` in
  `src/install_config.rs`. ADR-001 forbids it: `.doctrine/adr/001/layering.toml`
  classifies `install_config = "leaf"` at **out=0** (`:34`) and `boot =
  "command"` (`:90`), so the method would be a tier inversion *and* a cycle —
  `boot` already reaches `install_config` through `dtoml`.
  `tests/architecture_layering.rs` enforces this, so the design as written does
  not compile past the gate.

  The correction is forced and is the design's own rule: `sec-3` already sites
  `settings_rel(scope)` in `boot.rs` on the ground that "putting the path in
  `install_config` would give a pure leaf domain knowledge it does not otherwise
  have (ADR-001, and the module's own doc)". The identical argument covers the
  form mapping. **Nothing else in `sec-2` moves** — `CommandForm` still lives in
  `boot.rs`, is still deliberately *not* `ClaudeSettingsScope`, and the Codex arm
  still answers `Baked` on its own file's gitignored status without ever seeing a
  Claude-settings type. `ClaudeSettingsScope` keeps `sibling()`, which is pure
  vocabulary.

  Not patched at design time because the run is locked and the gates are
  unforgiving for a one-sentence correction. **Where it must land during
  implementation:** `PHASE-02` `EX-2` (the placement) and `PHASE-02` `VA-2` (the
  layering gate as a criterion, not an accident) — both authored in `plan.toml`,
  argued in `plan.md` § *The one departure from design `sec-2`*. **What
  reconciliation owes:** amend `sec-2`'s sentence to match what shipped. This is
  a per-slice artefact edit, so it is a direct edit at reconciliation, not a REV
  — it changes no governance and does not join the SPEC-011 amendment.
- **At close:** IMP-400 does *not* close with this slice — its `OQ-4` (migrating
  existing `enabledPlugins` / marketplace registrations) is out of scope and
  keeps the item open. Mirrored in `slice-250.md` § Follow-Ups.
- **What a further review pass should probe** — five items, in § What a further
  pass should probe below, written after the last pass.

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

## RV-348 round 2 — responder pass

Round 2 raised six findings against `design.md` at run revision 41 — 1 blocker,
3 major, 2 minor — after verifying `F-1`…`F-11` terminal. **All six upheld**,
none contested; each re-derived against `src/boot.rs` before disposing.
Remediated at revisions 42 (adopt) / 43 (materialise), round-tripped
byte-identically.

The round scored four for four against the *self-authored* probe list above,
which is the useful signal: the list was written by the same context that wrote
the defects, so it located the weak surface correctly and could not see through
it. Probe 4 in particular ("that holds only while nothing else ever writes
`worktree.baseRef`") is `F-14` at one remove — the author reached the edge of
the defect and stopped at the wrong side of it.

### The blocker: an ordering that did not buy what it claimed

`F-12`. `sec-3` ordered write-before-evict and said that guaranteed activation
lands before removal. It does not, because `RefreshOutcome::PrintedFallback` is
an **`Ok` value** (`src/boot.rs:1159-1178`, out through `:1611`): a malformed
*target* yields no write, the `?` does not short-circuit, and the sweep then
succeeds against a perfectly readable sibling. Zero activation, produced by the
ordering that exists to prevent it.

Distinct from `F-4` and not covered by its fix: there the sibling could not be
read; here the sweep **succeeding** is the defect. The general shape worth
keeping: *a fail-soft return type defeats sequencing arguments written as
though it were fail-hard.* `?` sequences errors, not failures.

### The pattern both rounds found, now named in the design

The raiser's synthesis: the mechanism is right nearly every time, and the
sentence attached to the mechanism overclaims by one degree — "nine entries",
"no change to `corpus.rs`", "a compiler-checked rename", "cannot disagree with
the live one", "this works because hooks are shell form". Accepted without
qualification. It is expensive because an implementer reads a discharged-analysis
sentence as discharged. This round the remediation fixed sentences as well as
mechanisms (`F-14`, `F-16` are pure-sentence findings and are treated as such).

### Where the remediation went past the ledger

- **`F-13` handed back a better shape than the finding asked for.** Threading
  `ClaudeSettingsScope` into the shared merge core to reach `.codex/hooks.json`
  would have made the type stop denoting what its name says — the same ADR-001
  objection `sec-3` raises one section earlier, pointed the other way. The axis
  is now `CommandForm { Baked, Portable }`; Codex answers `Baked` on its own
  file's gitignore status, not on a borrowed scope.
- **`F-14` resolved harder than raised.** The finding left the merge direction
  open; `docs/claude/settings.md:56-57` settles it — Local overrides Project for
  scalars, so a stranded `worktree.baseRef` override **still governs** and
  doctrine's fresh value is the inert one, the exact inverse of the deleted
  claim. Key still not swept (no-clobber), but now read and reported.
- **`F-15` was a type-shape finding.** `Removed` and `Unreadable` are not
  mutually exclusive *about a file*, so the fold needed a different type from the
  per-spec sum. `EvictOutcome` stays per-spec (gaining `NotAttempted` for
  `F-12`); `SweepReport { removed, unreadable, skipped }` is the per-file fold,
  with no absorbing state. The rider prints every true line, not the worst one.
- **`F-13` and `F-17` converged on `src/boot.rs:3953`** from opposite directions
  — the untested byte-identical-Codex claim, and the omission from the
  class-(ii) precision table. Better evidence for the `CommandForm` axis than
  either finding made alone.

### Ruled, not designed

**Windows is out of scope** (user, 2026-08-06). `${VAR:-default}` is POSIX
parameter expansion; PowerShell does not provide it and `PreToolUse` hooks fail
open, so it would degrade silently there. `F-16` is remediated as a *stated
scope boundary* plus the asymmetry note that matters regardless of platform:
`PORTABLE_EXEC`'s two consumers do not carry the same guarantee — `.mcp.json` is
expanded by the client at load, the hook by whichever shell the platform picks.

### What a further pass should probe

1. **`SweepReport`'s three fields against the seven-spec loop.** The fold is new
   and the rider is now multi-line; probe for a state where two lines print and
   contradict each other, and for `dry_run` interaction (a skipped sweep under
   `--dry-run` is not the same fact as a skipped sweep under a failed write).
2. **The `CommandForm` wire, end to end.** Six signatures widen. Probe for a
   consumer still reading a rendered command, and for whether
   `RefreshOutcome::Refreshed(command)`'s payload now varies by form in a way a
   test asserts on.
3. **`install_baseref` reading the sibling** is new I/O on a path that had none.
   Probe its failure mode when the sibling is malformed — the hook leg has
   `PrintedFallback` for that, this leg does not obviously.
4. **`F-12`'s guard at `dry_run`.** Under `--dry-run` nothing is written, so
   every write "did not land" by the file-system test but did by the plan's.
   The design gates on the *outcome*, not on the write; confirm that is right.
5. **The class-(ii) test enumeration, as a closure claim.** Added at the scope
   reconciliation (below): the slice's closure criteria now assert that the
   design's enumeration of knowingly-rewritten tests still matches what actually
   changed. That is a claim only an implementer can falsify — a further pass can
   only check the enumeration is internally exhaustive against the code-impact
   table, which `F-17` already did once.

### Scope reconciled against the accepted decisions (2026-08-06)

`review.scope` discharged. `slice-250.md` was asserting four things the design
run and the RV-348 remediation had moved past, all in the same direction — the
scope described the pre-remediation design:

- **The command form.** The scope said the strings "port into a settings file
  verbatim". True of the *plugin's* file, silent about doctrine's writer, which
  bakes `current_exe()`. Now carries the `CommandForm { Baked, Portable }` axis,
  SL-195's `baked ⟺ gitignored` invariant, and the Codex arm's answer.
- **POL-002.** The scope had it retired ("no longer a live constraint here") on
  the strength of the doctor leg leaving. It is live again on the *other* facet —
  no absolute host path in a tracked file — and that facet is what dictates code.
- **The merge-core Non-Goal's "four extensions".** Now seven, including the
  args-not-command shape, scope resolved *inside* `install_claude_hook`,
  `EvictOutcome`/`SweepReport`, `RefreshReport.hook` as a collection, and the
  shared announcement writer.
- **`corpus.rs` on the behaviour-preservation gate.** It is a changed path
  (`F-2`). The gate's two classes are now stated separately, and the closure
  criterion for the suites went from "stay green unchanged" — false — to the
  class (i) / class (ii) split.

Also corrected: `A3` (the gitignore change is already in place, and that is what
*costs*, not what saves), the config key's home (`[install]` in
`install_config.rs`, not beside `[dispatch]` in `dtoml.rs`), the SL-247 pointer
("behind a flag" → behind the sticky key), and `install_baseref` + the Windows
boundary, neither of which the scope mentioned at all. New closure criteria: no
absolute host path in a tracked file; the stranded-`baseRef` report; the
announcement on both install paths.

`review.selectors` discharged with it: `src/corpus.rs` promoted
`scope-relevant` → `design-target`, `src/dtoml.rs` added as a design target for
the module-doc mirror.
