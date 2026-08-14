# Notes SL-254: Collapse dispatch onto one subprocess arm

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-13 · PHASE-10 complete (8/10) · PHASE-08 ESCALATED pre-authoring · see git log

### Produced
- PHASE-10 done — Mode B's shipped implementation surface is gone (F-7). Four
  files deleted whole, one edited. Deleted: `install/hymns/role/orchestrator.md`
  (the live session-start role band; Mode B end to end — THE WALL, `arm-spawn`,
  the six-step `worker_commit`→`dispatch_import`→`conclude`→`reap` cadence),
  `install/workflows/drive-slice.js` (353 lines), and the two claude agent defs
  `dispatch-orchestrator.md` (jailed under the `pretooluse.rs` wall PHASE-04
  deleted) + `dispatch-probe.md` (orphaned — `drive-slice.js` was its only
  caller). Edited: `install/hymns/role/worker.md`, one clause — the worker now
  simply never commits. Nine lockstep sites updated (four
  `publication/manifest.toml` entries, `install/manifest.toml`'s `expose`,
  `src/install.rs`'s const + forward-step-3b leg + two tests + three stale
  comments, `tests/e2e_prompt_resolve_golden.rs`'s `EXPOSED` table made
  count-free). `just gate` exit 0, `publication validate` clean (90 entries),
  `boot --check` clean, `prompt check` OK, `doctrine install` re-run and
  resurrected nothing

### Learned (PHASE-10)
- **A tenth undercount, and the one that mattered most.** `EX-8` named the
  `.claude/workflows/drive-slice.js` SYMLINK but not its target. That target,
  `.doctrine/workflows/drive-slice.js`, is **git-tracked** — the materialised
  install copy, and the actual payload Claude Code loads as the live
  `/drive-slice` skill in this repo. Deleting only the shipped source plus the
  symlink would have left a tracked, invocable program driving the deleted
  Mode-B landing cadence — the exact failure `VH-1` exists to catch, surviving
  the phase meant to remove it. Class: a *materialised installed copy* is a
  distinct site from both the shipped source and the harness symlink. Also
  swept: `.doctrine/agents/dispatch-probe.md` + its `.claude/agents/` symlink
  (untracked).
- **`EX-3`'s ruling is right; its stated reason was not the whole story.**
  `drive-slice.js` is not pure Mode B — it carries a real (A)/pi subprocess arm
  (`fork_tip: null`, worktree-diff import). What actually kills it for BOTH arms
  is that its only landing verb is `dispatch_import`, and
  `src/mcp_server/dispatch.rs:289-307` resolves the phase from the durable fork
  binding via `require_binding`; the collapsed arm's forks are UNBOUND (`OQ-1`).
  Its (B) path also spawns via in-session nested `agent(isolation:'worktree')`,
  and its (A) path does not spawn at all. Verified before deleting, not assumed.
- **`EX-8`'s golden-test wording was imprecise (conclusion unaffected).**
  `tests/e2e_prompt_resolve_golden.rs` does not hardcode `orchestrator.md`'s
  first line as a positive golden — `EXPOSED`'s 5th field is a framework-body
  substring asserted ABSENT, and the orchestrator row's value matched this
  repo's LOCAL twin, not the shipped body, so it was already vacuous and would
  have stayed green either way. The row went for truth (it claimed a framework
  slot that no longer ships, and is coupled to `expose`), not for redness.
- **Coverage genuinely retired, not silently dropped.**
  `install_agent_def`'s derived-dest-filename guard needed a MARKER-FREE shipped
  def to prove it, and `dispatch-probe.md` was the only one. Both survivors
  carry `WORKER_RESOLVE_MARKER`, and their basenames are exactly the name the
  original bug hardcoded, so derived and hardcoded are now indistinguishable
  from any real asset; `install_agent_def` reads the embed directly, so no
  synthetic can be injected. Recorded in situ at the deletion site.
- **`VH-1`'s trap is real and was avoided.** Checked in a fresh `mktemp -d` +
  `git init` + `doctrine install`, NOT here: this repo's tracked local twins
  `.doctrine/hymns/role/{orchestrator,worker}.md` override the shipped default
  and would have false-greened any resolve run locally. In the clean project no
  `role/orchestrator` is projected at all, and `prompt resolve` for BOTH roles
  is free of every Mode-B token (`arm-spawn`, `worker_commit`, `dispatch_import`,
  `dispatch_conclude_phase`, `dispatch_reap`, THE WALL, self-commit).
  `prompt check` OK there too. The two local twins were separately read and are
  arm-agnostic prose — they correctly survive as project overrides.

### Open (PHASE-10)
- `--role orchestrator` in a clean project now resolves to the sealed
  `preamble/core.md`, whose first line reads *"You are a doctrine dispatch
  worker"*. Pre-existing (the preamble is unconditional and always said this),
  but with the orchestrator role band gone that worker framing is now
  unopposed. Not fixed here — it is precisely what `IDE-053` (a stub `agent`
  role + clone-workflow rewrites of the role prompts) was deferred to decide,
  and this is a concrete datapoint for it.
- `src/hymns.rs`'s `Role::Orchestrator` and `src/doctor_checks.rs`'s
  `ROLE_ORCHESTRATOR` / `ROLE_PROBE` arms were deliberately LEFT IN PLACE. The
  role remains a resolvable slot and a legal `--role` value, and those arms
  validate the `doctrine-role:` marker on a project's OWN agent defs — deleting
  them would invalidate a locally-authored orchestrator def. Consequence
  recorded, not absorbed: both arms are now dead-in-production (no shipped def
  carries `orchestrator` or `probe`) and are exercised only by synthetic
  fixtures.

### Produced (earlier phases)
- PHASE-07 done — one spawn skill. `/dispatch-agent` + `/dispatch-subprocess` →
  `/dispatch-spawn` (both source dirs deleted, so PHASE-08's TWO dangling
  `spec-021.toml` anchors resolve as `EX-8` predicts — **and PHASE-08 should ADD
  one for `plugins/doctrine/skills/dispatch-spawn/SKILL.md`**). `/dispatch`'s
  front-matter no longer claims the funnel is "identical on both arms"; the
  funnel is now described as bound-fork machinery that the shipped path
  deliberately does not mint (`OQ-1`), so `dispatch next` sits at `spawn` and the
  main-thread orchestrator drives the landing. `dispatch-mechanics.md` re-cut,
  Mode B section retired. Six undercounts fixed and recorded, not absorbed —
  incl. a Rust unit test that READ the deleted skill's text (second instance of
  the class the symbol census cannot see) and four stale clap help strings.
  `just gate` exit 0, `doctor` exit 0, `boot --check` clean (c4a596774)
- PHASE-06 done — the last four claude-arm-only surfaces are gone:
  `mcp_server/worker_commit.rs` (1477) deleted and unregistered, `arm-spawn` and
  the `Spawn`-row recorder out of `dispatch.rs` (−861 lines), `create.rs`'s Fork
  arm + `ARMING_*` + `JAIL_SUBPATH` + `provision_jail_policy` gone (−969) with
  Passthrough and `run_provision` kept (D1), and
  `claude-force-subprocess-dispatch` removed from every live surface (VA-1).
  Doctor check #9 re-cut: a `--strict-mcp-config` worker holds NO `mcp__*` token
  (DEC-216). `just gate` green, `doctrine doctor` exit 0
- PHASE-05 done — the disk marker is gone; worker identity is `DOCTRINE_WORKER`
  alone, topology-independently (DEC-207). `marker.rs` 439 → 128 lines,
  `subagent.rs` (401) deleted, `describe_mode` is a two-row truth table over one
  input, `worker_guard` lost its root resolution entirely (the verdict is a
  property of the PROCESS). `worktree marker`, `verify-worker` and
  `status --assert` no longer parse; `WriteClass::MarkerClear` AND `Hookmint` both
  retired. ISS-028 closes. 39 files, 9 test suites retargeted, 2 deleted.
  `just gate` green at zero clippy warnings
- PHASE-04 done — `worktree pretooluse` and the whole pure decision layer it was
  the sole caller of are gone. `pretooluse.rs` deleted (877 lines); `jail.rs`
  2,267 → 1,652, keeping EXACTLY the transitive closure of `jail_prefix.rs`'s
  imports plus the `JailPolicy` re-export (EX-3 verified closed both ways —
  nothing missing, nothing extra). Installed hook set 9 entries/3 events → 5/3;
  the two surviving `PreToolUse` entries are `memory surface` and were not
  touched. `just gate` green at zero clippy warnings, which IS VA-1's claim
- PHASE-03 done — `nominate`/`denominate`, the `PreToolUse(Agent|Workflow)` spawn
  gate and doctor check #10 `SpawnSeamSymmetry` are gone (DEC-205), taking
  `PRIVILEGED_AGENT_TYPES`, `is_privileged_agent_type`, `decide_agent`,
  `decide_workflow`, `resolve_target`'s nomination parameter, seven orphaned
  constants and `Category::SpawnSeamSymmetry` with them. ~1,265 lines net removed
  across 18 files. `just gate` green at zero clippy warnings
- PHASE-02 done — `scripts/pi-spawn-confined.sh` → `scripts/spawn-confined.sh`,
  harness as its first argument, `claude` profile added alongside `pi`
  (DEC-209/210/215/216), Linux `bwrap` probe naming `bwrap-unavailable` ahead of
  the fork (D7/F-6), and `sandbox_exec_argv` now carries `DOCTRINE_WORKER=1` in
  its trailing `env` run (F-2). New `tests/e2e_worker_confinement.rs` proves the
  shipped PREFIX tokens confine live, and skips named where `bwrap` is absent
- PHASE-01 done — the four jail primitives re-homed to `jail.rs`, `jail_prefix.rs`
  re-pointed, `BWRAP_BIN` collapsed onto `jail::BWRAP` (ef59ad0f5). Both relocated
  unit tests byte-identical; `just gate` and `doctrine check gate` exit 0
- capsule execution posture, entity-id collision rule, and the `ADR-001` follow-up
  recorded (f6096d932) — see `## Execution environment and standing directions`
- design locked and nine phases materialised (caf7f2a21)
- minted across scope + design: DEC-202..DEC-218, EVD-023, QUE-214, QUE-215,
  ISS-347, ISS-349, IMP-428, IMP-429, CHR-062, and SL-255 (the clone half)
- ~~**`research/` is GONE**~~ — **RESTORED by the user 2026-08-13**, same session.
  It had been absent when this capsule started (runtime tier, gitignored, so the
  checkout carried no copy). `research.md` reads intact and its baseline pins
  exactly to `5a68bfd9e`. **`raw/` did NOT come across** — the five per-thread
  agent outputs the header cites (`raw/governance.md` …) are still missing, and
  they exist nowhere durable
- the design run `dr-019ff653` snapshot is still gone (runtime loss, not a
  missing run). Authored `design.md` + `plan.toml` stand; `design show` refuses
- 11 friction observations recorded, all committed
- `.doctrine` changes committed promptly throughout, and PHASE-01's code went out
  in its own commit separate from the sheet/notes commit that preceded it

### Learned
- **A skill-path grep of `src/` is a required leg of any skill deletion.** The
  symbol census (Rust symbols outward, literal strings inward) reaches neither a
  Rust test that *reads a markdown file by path* nor a clap doc-comment's prose.
  PHASE-02 hit the first form on the spawn script; PHASE-07 hit it again on a
  SKILL.md (`mod.rs`'s `subagent_type` pin) plus four stale `--help` strings.
  Generalisable: before deleting or renaming any non-Rust shipped asset, grep
  `src/` and `tests/` for its **path**, not just its symbols
- mem.pattern.refactor.move-closure-exceeds-consumer-imports — a re-home set
  derived from the consumer's imports under-counts the definitions' own needs
- mem_019ff650d94a7960a638913a40416165 — collide "what calls this" research
  findings against "what should exist" decisions at synthesis
- mem.fact.dispatch.worker-confinement-is-actor-based — strengthened with the
  process-vs-tree generalisation and re-attested

### Open
- **⚠ ESCALATED at PHASE-08, UNRESOLVED — `ADR-020` reserves `ADR-006`/`008`/`011`/
  `012` to `REV-046`, and nobody asked whether it constrains this REV.** PHASE-08
  stopped **before authoring anything**; the full re-derivation it completed first
  (nine entities, ~191 regions — a 4× undercount, the eleventh) and the escalation's
  three routes are in `### PHASE-08 re-derivation` and `### ESCALATION, PHASE-08`
  at the end of this file. Coupled question in the same ruling: `ADR-011` is 56 of
  ~70 regions falsified with a dead title-half and a Verification section it now
  fails — is `EX-2`'s "in-place amendment" still the right instrument, or a
  supersession? Anchor sweep (`EX-8`) is **done and confirms `EX-8` exactly**: two
  dangling anchors, both `spec-021.toml` (`:30`, `:34`)
- **⚠ ESCALATED at PHASE-07, UNRESOLVED — an entire shipped asset class still
  instructs the retired arm, and no phase in the plan reaches it.** Four files,
  ~500 lines, absent from design §5.6's table and from every phase's criteria:
  `install/hymns/role/orchestrator.md` (59 lines) **is Mode B written as a role
  prompt** — "your cwd is jailed to the coordination tree … the one raw-Bash
  exception is `arm-spawn`", then a cadence over `arm-spawn` → nested `Agent
  isolation:worktree` spawn → `worker_commit` → `dispatch_import` →
  `dispatch_conclude_phase`. It is resolved into a live agent's context by
  `doctrine prompt resolve --role orchestrator`, and the hymn bands are
  role/harness/model/stage with **no arm band**, so nothing selects it away.
  With it: `hymns/role/worker.md` (55, names both arms and the gated
  `worker_commit`), `install/workflows/drive-slice.js` (353, the shipped
  `/drive-slice` workflow — a program, not prose, with ~6 `worker_commit` /
  arm-routing sites), and `install/agents/claude/dispatch-orchestrator.md` (31,
  the agent def for that role). **Not absorbed into PHASE-07** because the
  question is not prose: `DEC-217` retires Mode B, and these files *are* Mode B's
  shipped implementation surface, so someone must rule whether the
  confined-orchestrator ROLE is deleted, rewritten onto the main-thread posture,
  or left standing — a delete-a-shipped-capability call with `DEC` weight
  dragging a 353-line JS workflow behind it. PHASE-08 is the governance corpus
  only; PHASE-09 is the live run. **Recommendation: a new phase between 07 and
  08, or an explicit owner ruling carried into the reconciliation brief.**
- **`doctrine install` cannot prune a retired SKILL's projection** — the same
  mechanism already recorded here for a retired *hook* entry. After the merge,
  `.agents/skills/`, `.doctrine/skills/` and `.claude/skills/` all still carried
  `dispatch-agent` and `dispatch-subprocess`; install visits the skills in the
  registry and never sees a directory whose source is gone. Cleared by hand
  (disposable projection state). Note also that the `.agents/` tier is written by
  the delegated `npx skills add` leg, which fetches from the **github** remote —
  so a locally-added skill never lands there until published (CHR-049's known
  `skills-lock.json` staleness). The claude-tier projections did pick up
  `/dispatch-spawn`. Candidate backlog card at reconcile — no id minted
  (collision rule)
- **⚠ ESCALATED, UNRESOLVED — `DEC-204`/`DEC-213`'s premise is false and a
  documented security control is now inert.** Both say `worker_commit` may retire
  because "`classify_import` survives as the scope belt's enforcing reader", and
  `PHASE-06/EX-6` keeps `worker-forbidden-writes` on that basis. Verified against
  `import.rs:120`: `classify_import` enforces HEAD/tree/single-commit, the
  `.doctrine/` and `.claude/` prefixes, and the selector-scope leg — it has NEVER
  read `worker-forbidden-writes`. Its enforcing caller was `worker_commit`, now
  deleted. `.doctrine/**` and `.claude/**` stay fenced (classify_import's own
  constants); what loses enforcement is the configurable remainder —
  `.agents/**`, `install/agents/**`, `flake.nix` ("highest security leverage in
  the repo"). The template ships that block commented, so no default silently
  degrades, but a project that enabled it loses the fence with no signal. Key and
  matcher KEPT under an `expect(dead_code)` naming the gap, so the loss is visible.
  Wiring `classify_import` to read the key would contradict PHASE-06's own `VT-1`
  and §5.2.5's `INV-2` — **a decision is required, not a comment**
- **`observation_record` is now unreachable for a confined dispatch worker.**
  PHASE-06/EX-7's re-cut forced stripping both `mcp__*` tokens from the authored
  `install/agents/claude/dispatch-worker.md` (otherwise `doctor` check #9 is
  permanently red on doctrine's own corpus, reddening `just validate`). This
  follows from `DEC-216`, but it contradicts `CLAUDE.md`'s instrumentation table
  row prescribing that exact tool for a confined worker. Recorded nowhere else
- **The spawn-side claim lock lost its last production caller.** `create.rs`'s
  Fork arm was the only prod caller of `claim_lock::acquire`; `fork --worker`
  never took it. The spawn↔gc race `SL-228 §3` bought it for is now guarded only
  by `fork_core`'s atomic branch-ref claim. `acquire` gated to `#[cfg(test)]`,
  `fork.rs`'s contrary doc corrected. A real behaviour change the plan omits
- **`PHASE-06/VA-1` is unsatisfiable as written** — ~60
  `claude-force-subprocess-dispatch` hits remain in `.doctrine/` historical
  entities (settled ledgers, closed slices, `SPEC-021`/`REQ-288`, this slice's own
  design). Scrubbing them would falsify the record, and the governance ones are
  what PHASE-08's REV retires. Clean across every LIVE surface; the criterion
  should be scoped to those
- **`design.md` §5.2.3's `land` substitute is WRONG AS WRITTEN — carry to PHASE-08.**
  It says to substitute `classify_worktree_role` "returning `fork`" for `land`'s
  `bears_marker` read. That classifier returns `fork` for EVERY linked non-coord
  worktree, and `land` only ever runs against a linked worktree — so taken
  literally it refuses every `land`, including the solo TDD branches the verb
  exists for. PHASE-05 implemented `shared.rs::is_dispatch_fork_branch` (the
  `dispatch/` prefix AND a non-numeric suffix) instead; the design's intent and its
  "strictly stronger than a marker read" claim are unchanged. The design TEXT still
  carries the wrong prescription and should be corrected in the REV
- **VA-1 has one stated exception**: `.doctrine/state/dispatch/worker` survives as a
  literal in `scripts/spike-capsule/control/audit-nohooks.sh`'s `NOHOOK_TOKENS`,
  where it is an ABSENCE assertion over a separate spike rig rather than a consumer.
  No reader or writer remains anywhere else. Left alone deliberately — rewriting a
  spike's recorded evidence to satisfy a grep destroys more than it proves
- **Accepted behaviour change at the guard (DEC-207 consequence, not a new call).**
  The `observation` refusal's MCP-broker advice was previously withheld from the
  env-on-non-linked-tree case; topology can no longer be consulted, so it is now
  unconditional. Recorded rather than escalated because `DEC-207` states the
  position directly; the remedy rides `WORKER_ENV_CAUSE`
- **`base64` is confirmed unused** (verified at PHASE-05 against the current tree:
  zero references in `src/` or `tests/`). Gate-neutral — cargo does not warn on an
  unused dep — so still left for reconcile rather than touching `Cargo.toml`
  mid-slice
- ~~The per-arming jail policy file has no production reader~~ — **resolves inside
  PHASE-06**, not a standing item: `create.rs`'s `provision_jail_policy` and
  `dispatch.rs`'s `ARMING_JAIL_FILE` write are both deleted by PHASE-06/EX-2+EX-3,
  so the live-write/dead-read asymmetry ends with the writer. Watch for
  `JailPolicy::from_toml_str` orphaning at zero warnings when it lands
- **PHASE-04's `EX-4` and `EX-8` are mis-phased and were DEFERRED to PHASE-05
  as its new `EX-9`** (annotated in `plan.toml` at all three sites). They ask
  `guard.rs` to drop the `Marker { stamp_subagent: true }` arm and the
  `WriteClass::MarkerClear` class, on the stated ground that these "name
  variants deleted by EX-1/EX-4" — but `EX-1` deletes `Pretooluse`, never
  `Marker`, whose `--clear` / `--stamp-subagent` forms die in PHASE-05. Since
  `write_class` is wildcard-free, removing the class early does not compile;
  removing only the `stamp_subagent` arm DOES compile and is the trap — the
  verb would fall through to `MarkerClear`, which `worker_guard` passes
  through, silently losing the worker-mode refusal while the verb still exists
- **The per-arming jail policy file has no production reader after PHASE-04.**
  `RealEnv::read_policy` and `JailPolicy::from_toml_str` are test-only now
  (`jail-prefix` takes policy inline via `--extra-rw`/`--network`), yet
  `create-fork` still WRITES `.doctrine/state/dispatch/jail/<name>.toml`. Live
  write, dead read. Candidate PHASE-05/06 or reconcile item
- **`base64` is now an unused dependency** — it existed for `opaque_wrap`, which
  PHASE-04 deleted. `Cargo.toml` still declares it citing SL-182 `INV-5`. Left
  alone rather than touch `Cargo.toml` mid-slice
- **`doctrine install` CANNOT prune a retired hook entry, so `PHASE-03/EX-3`'s
  and `PHASE-04/EX-5`'s prescribed mechanism does not reach their stated
  outcome.** Hook reconciliation is per-`HookSpec` and ownership-keyed: install
  visits each spec IN the registry and adds/refreshes its entry. An entry whose
  spec has been *deleted* is never visited — and the predicate that could
  recognise it (`is_doctrine_nominate_command`, keyed on `NOMINATE_ARGS`) was
  deleted along with it. So the retirement of a hook is exactly what makes its
  installed entry unownable and therefore unprunable. Verified live: after the
  registry edit, `doctrine install` left both `SubagentStart`/`SubagentStop`
  entries in `.claude/settings.json` untouched *and* re-added the four
  `worktree pretooluse` entries. `PHASE-04` inherits this for its own four.
  Candidate backlog card at reconcile — no id minted here (collision rule)
- **Running `doctrine install` in this capsule RE-ARMS the wall mid-session.**
  It restored the four `PreToolUse` `worktree pretooluse` entries the capsule
  had locally stripped, and the wall immediately began refusing this session's
  `Bash` *and* `Edit`/`Write` calls (`worktree-jail: cwd-not-a-worktree`) —
  the primary checkout is not a linked worktree, so an agent-attributed call
  resolves to `Target::Reject`. Recovery was via a tool outside the hook's
  matcher set. The standing note that "confinement hooks are inert in this
  capsule" holds only while nothing re-runs `install`
- **`bwrap_argv` has the same hole `F-2` fixed on macOS** (PHASE-02 finding).
  The BINARY's Linux prefix builder emits no `--setenv DOCTRINE_WORKER 1`; only
  the spawn script's inline array does. Dormant today because nothing on Linux
  shells `worktree jail-prefix` — but `IMP-429` (the prefix's permanent home) is
  exactly the move that would wake it. Not fixed in PHASE-02: outside every
  criterion, and it would move tokens `EX-2` freezes
- **A Rust unit test consumes the spawn script's TEXT** — `jail.rs`'s core-flag
  parity test opens the script by path and parses its inline array. Neither half
  of the slice's two-direction census (Rust symbols outward, literal strings
  inward) reaches a Rust consumer of a non-Rust file's *text*. Cost one red
  cycle in PHASE-02; re-anchored on `PREFIX=(` and re-keyed to `$CFG_DIR`.
  Observation `019ffbeb-d553` records the generalisable form
- `ADR-001` posture of `jail.rs` — two unseamed impure functions now sit in a
  registered `leaf`; candidate seventh `REV` target for PHASE-08's re-derivation
- macOS/Darwin path unverified — `jail_prefix.rs:40`/`:46` are cfg-stripped in
  this capsule; carried to the AARCH64 sweep, `OQ-5`, PHASE-09 `VA-2`
- entity-id collision risk — ids minted in this capsule can clash with the host's;
  flag every one in the phase sheet and commit message
- `SL-254` `OQ-1` — `pi-spawn-confined.sh:56` passes neither `fork --worker` flag,
  so the collapsed arm inherits unbound forks; a precondition of PHASE-09's `VA-1`
- `SL-254` `OQ-2`..`OQ-5` (`design.md` §6) — none blocking
- deletion boundary widened at drafting (`design.md` §7.2 `D1`-`D3`, §8 `R6`) —
  four surfaces objective 3 does not name; carry into the reconciliation brief
- `SL-254` `OQ-2` backlog dissolution set — IMP-269, IMP-342, IMP-334, IMP-337,
  IMP-407, IMP-401, IDE-024; confirm at reconcile
- memory-corpus sweep at reconcile — 25+ stale claude-arm memories, one in the
  boot snapshot
## Design surface triage
<!-- `explore.triage` runbook step, design run dr-019ff653, 2026-08-13 -->

Ten inquiry nodes declared and resolved: `DEC-203`..`DEC-212`.

### Shaping decisions

| node | record | settled |
|---|---|---|
| inq-1 | `DEC-203` | Bounded Pole B — worker side goes capsule-shaped, `ADR-012`'s coordination topology untouched |
| inq-2 | `DEC-209` | Generalise the incumbent prefix; take no `doctrine-control` dependency, port no hardening |
| inq-3 | `DEC-204` | `worker_commit`'s **transport** retires; two of its six belts re-home to the fetch/admit step |
| inq-4 | `DEC-210` | Subscription credential, `$HOME/.claude` bound wholesale like pi's `$HOME/.pi`; narrowing deferred |
| inq-5 | `DEC-205` | nominate/denominate die with `pretooluse` — closed loop, not a separate choice |
| inq-6 | `DEC-206` | Re-home four jail primitives to `jail.rs` as the FIRST step, before any deletion |
| inq-7 | `DEC-207` | Identity collapses to the env leg; delete the marker and `describe_mode`'s `is_linked` conjunct |
| inq-8 | `DEC-208` | One arm. No degraded in-session rung; fail closed at spawn as the pi arm already does |
| inq-9 | `DEC-211` | One REV over `ADR-011`, `ADR-006`, `SPEC-021`, `SPEC-012` + a hand anchor sweep |
| inq-10 | `DEC-212` | Behaviour-preservation restated as the funnel's observable contract, not "suites unchanged" |

### Governance constraining the surface

Read directly this stage rather than through the research round's quotations —
`DEC-211`'s body carries the site-by-site detail.

- **`ADR-011`** — **eight** amendment regions (superseding this section's earlier
  "five", which mis-cited 263): Context (21-25), `D1` (39-56), `D2` (58-74),
  `D3`'s table (82-92), `D4` (94-111), `D6` (146-207), Consequences (248-259),
  Verification (265-275). `D5` (113-144) and `D7` (209-232) considered and
  deferred. Site-by-site detail in `DEC-211`'s `choice`.
- **`ADR-006` §D2b** — thread 1's claim verified; the `SL-181` note pattern is real
  at line 143. Second site at 308. Its "degenerate case" list already contemplates
  a standalone clone — `SL-254` makes that the *normal* case, so the note re-cuts
  rather than appends.
- **`ADR-012`** — **not touched**, per `DEC-203`. Load-bearing boundary: it is what
  keeps the REV inside the surveyed set.
- **`POL-002`** — satisfied, not strained (`DEC-206` keeps the argv builder in the
  binary and harness specifics at the script tier).
- **`STD-001`** — governs the thoroughness of the `claude-force-subprocess-dispatch`
  deletion and of the belt consts' single-sourcing.

### Corrections this stage made to prior artefacts

- **`DEC-202`'s `choice`** — corrected in place: it read as a claim about worktrees.
- **The scope's behaviour-preservation gate** — "codex/pi suites stay green
  unchanged" is unachievable once the pi arm moves to clones too (`DEC-212`).
- **The scope's governance survey** — missed `SPEC-012`'s responsibility *prose*
  (`spec-012.toml:18`, `fork`'s "stamp", `import` as "the belted dispatch funnel")
  and `ADR-011`'s fifth site.
- **Research thread 3's auth recommendation** — `ANTHROPIC_API_KEY` forfeits the
  subscription billing `EVD-023` establishes (`DEC-210`).
- **Research thread 2's `worker_commit` reading** — "every check is meaningless in a
  clone" is wrong on two of six belts (`DEC-204`).

### Newly surfaced — NOT in the scope's reconcile survey

- **The memory corpus.** At least 25 memories describe claude-arm mechanisms this
  slice deletes — `SubagentStart` stamping, `PreToolUse` jail behaviour,
  `WorktreeCreate` provisioning, `worker_commit` resolution, marker identity —
  several at `high` trust / `high` severity. After the collapse they are
  **stale-but-plausible**, the most dangerous class: an agent retrieving
  `mem_019ebfb61ba870219aafc14f8dc7da3b` ("worker identity via `SubagentStart`
  hook") would act on a deleted mechanism. `mem.signpost.doctrine.dispatch-claude-arm-wrong-base`
  is indexed in the **boot snapshot**, so the staleness reaches every session's
  context. Needs a deliberate `/reviewing-memory` sweep at reconcile — carried in
  the scope's Follow-Ups.

### Open / deferred

- `IMP-428` — harden the worker confinement prefix (deferred out by `DEC-209`).
- Clone disk/time cost for this repo: still unmeasured. `POL-002` forbids baking a
  local measurement into the platform regardless.
- `../microvm-spike`'s narrowed `~/.claude` mount set: deferred by `DEC-210`; its
  identity-section partial is not definitively proven.

## Review passes — RV-355 and what a further pass would probe

*Written 2026-08-13, after the first pass, per the reviewing runbook's
`review.passes` obligation.*

**Pass 1 (done).** External adversarial, codex/GPT-5.5, against the design at run
`dr-019ff653` rev 41. Seven findings, all verified against the code before
disposition, all dispositioned `design-wrong` — the defects are in the design
artefact, not the implementation. Three blockers: `F-1` (Mode B loses its funnel
entry point), `F-4` (`SPEC-012` carries four falsified requirements, not one
responsibility), `F-5` (`ADR-012`'s harness-synthesis rule names the deleted arm
normatively). The pass also reported eight claims it checked that held — the
`require_binding` census, the nomination closed loop, `verify-worker`/`arm-spawn`
having no surviving consumer, `classify_import`'s single-sourcing, the
pi-specificity of the fifo/reap machinery, and no reachable unconfined fallback.
That negative half is worth as much as the findings: it is what makes silence in
those areas informative.

**What a second pass should probe, once the `F-1`–`F-7` fixes land.**

1. **The widened governance landing.** `F-4` and `F-5` grow the `REV` from four
   target entities to five and `SPEC-012` from one responsibility to four
   requirements plus that responsibility. The survey has now been wrong twice in
   the same direction — under-counting — so a second pass should re-derive the
   target set from the entities rather than from `DEC-211`, and specifically ask
   what else in `SPEC-012` and `ADR-012` the first two sweeps missed.
2. **Mode B's retirement, once it is written down.** `F-1`'s remedy records the
   confined-orchestrator funnel arm as retiring with the in-session arm. That is
   a consequence nobody chose directly — it falls out of the unbound-fork
   settlement — so it deserves adversarial attention in its own right: does
   anything else ride Mode B, and is `REQ-335`'s "stays pending" still honest
   when the substrate it was pending on is deleted?
3. **The macOS arm as a whole.** `F-2` found the seatbelt argv missing
   `DOCTRINE_WORKER` — the one place `OQ-5`'s "parity holds by construction" was
   false. One instance of a class is not a census. A pass over the Darwin path
   specifically, holding it to every claim §5.1 and §5.2.1 make about the two
   profiles differing in exactly two tokens, is the obvious next probe.
4. **What a design pass structurally cannot reach.** Nothing here exercises a
   real `claude -p` under bwrap. `A2`'s residue is provisioning, and provisioning
   fails at runtime or not at all — that is the `VA` leg's job, not a reviewer's.
   A second design pass should not spend effort simulating it.

**What a second pass should not re-litigate.** `DEC-213`'s split into `SL-255`,
and the unbound-fork settlement it forced. Both are owner decisions taken on
stated evidence. A finding that this slice should have taken the clone half, or
should bind its forks, is a finding about those decisions — admissible as one,
but not as a defect in the draft.

---

**Pass 1 closed 2026-08-13, rev 42 → 49.** All seven findings verified as raiser
and `RV-355` concluded. Every section body moved, so the pass is `STALE` against
current content by construction — a second pass reviews a different artefact, not
a patched one.

*Probe 1 above was discharged in the course of the fixes, and it found more than
it expected.* Re-deriving the target set from the entities rather than from
`DEC-211` grew the `REV` from four entities to **six**, not the five `F-4`/`F-5`
implied: `ADR-008` was absent from every prior survey and carries seven regions
(`D-B3`'s "codex/pi-only … not a subprocess to wrap" clause, `D-B6`'s whole
nominate/gate/`SubagentStop` mechanism, `N1`'s `worker_commit` exception);
`ADR-006` is nine regions rather than two, reaching `D2a`'s decision body and both
`D9` amendments; `ADR-011` is eleven rather than eight. Recorded as `DEC-218`,
which supersedes `DEC-211`'s enumeration and carries the derivation *method* so
the next reader re-derives rather than re-cites. `R8` and the second `VH` claim
(§9.4) are the standing guards.

*Probe 2 is now written down as `DEC-217`* — Mode B retires with the in-session
arm, as a consequence of the unbound-fork settlement rather than a choice. Its
two sub-questions are answered in the record: what else rides Mode B is
`SPEC-021` `REQ-384`/`REQ-387`, both narrowed in the `REV`; and `REQ-335`'s
"stays pending" is honest **as a contract** while its one partial implementation
retires, which the scope's Non-Goals now say explicitly. Still worth adversarial
attention, but against a stated position rather than a silence.

*Probe 3 (the macOS arm as a census, not one instance) is untouched and remains
the sharpest open probe.* `F-2`'s fix is one token in `sandbox_exec_argv`; nobody
has swept the Darwin path against every claim §5.1 and §5.2.1 make.

**New, found while fixing, not by the pass.** Two `doctor` checks are live
consumers of deleted symbols and were in no earlier list: #10 `SpawnSeamSymmetry`
reads `PRIVILEGED_AGENT_TYPES` and the `SubagentStart`/`PreToolUse` registries, so
**the deletion does not compile** without removing it; #9 `AgentConformance`
allowlists `mcp__doctrine__worker_commit` as the worker's one MCP token, which
`DEC-216` leaves empty. `src/finding.rs` loses a `Category` variant with #10. A
second pass should ask what *else* consumes a deleted symbol — the design's §5.6
is a hand-built list, and this was the second sweep to extend it.

---

## The symbol census — sixth sweep of §5.6, and the method change

*Run 2026-08-13 at rev 49, before section attestation, so §5.6's edits do not
stale an attestation twice.*

The `doctor_checks.rs` find above was a compile-breaker sitting behind a symbol
the design had already marked for deletion — which said the *method* was wrong,
not that one entry was missing. Sweeps one to five read the code and asked "what
does this change touch", which is answered from a mental model. The sixth
inverted it: enumerate every `pub` item in the three dying modules and in
`jail.rs`, grep each across `src/` and `tests/`, subtract consumers that are
themselves dying. Mechanical, and it reproduced `doctor_checks.rs` as a positive
control before finding anything new.

**What it found.**

1. **`jail.rs` loses its entire pure decision layer** — ~300 of 981 production
   lines and about half its ~60 unit tests. `pretooluse.rs` was the sole caller of
   `resolve_target`, `decide_agent`, `decide_workflow`, `decide_bash`,
   `decide_write`, `pathcheck`, `opaque_wrap`, `shell_single_quote`,
   `acquire_policy`, `resolve_inputs`, `seatbelt_backend`, `is_privileged_agent_type`,
   `enum Decision`, `enum Target` and five `REASON_*` constants. `pub(crate)` with
   no caller is `dead_code`, and `just gate` is clippy-at-zero-warnings, so this is
   a gate-breaker in the same class as `doctor_checks.rs`. §5.6's old jail.rs row
   read as a four-item edit; it is a third of the module.
2. **`src/commands/guard.rs` is where the write-class registry lives**, not
   `main.rs` as §5.6 said. Its `Marker { stamp_subagent: true }`, `Nominate` and
   `Denominate` arms don't compile once those `Command` variants go; the bespoke
   `WriteClass::MarkerClear` class retires with `run_marker_clear`; and
   `worker_guard`'s two-branch dual-cause message collapses when the marker leg does.
3. **`justfile:37`** — `validate`'s worker-context skip has a marker-file leg
   alongside the two env legs. Non-Rust, so no symbol grep would have found it; it
   surfaced only by grepping the literal path string.
4. **Six unlisted test files**, one of them (`e2e_dispatch_arm_spawn.rs`) an
   outright delete, plus `tests/common/mod.rs`, whose fixture helper *stamps* the
   marker.

**Two positives worth as much as the findings.** `F-2`'s fix is on live code —
`sandbox_exec_argv` is reached from `jail_prefix.rs:168` via `Seatbelt::wrap_argv`,
not from the dying wall, so the census does not undercut it. And `DEC-206`'s
re-homing set is provably complete: `jail_prefix.rs` imports exactly those four
primitives from `pretooluse.rs` and nothing else. ~~No fifth primitive is
hiding.~~ — **qualified 2026-08-13 at `/phase-plan` PHASE-01.** True of
`jail_prefix.rs`'s *import list*, which is what it was derived from. The
**move's** closure is wider: `have_bwrap` reads two private `pretooluse` consts,
`BWRAP_BIN` (`:71`) and `ENV_PATH` (`:77`), and `BWRAP_BIN` is an `STD-001`
duplicate of a constant `jail.rs` already owns (`:73`, `const BWRAP`). Not a
design defect — a scope-of-claim correction, and the ninth time a survey here
has read low.

**Method limits, recorded so the plan phase inherits them.** A negative grep is
worthless without a positive control — `e2e_priority_golden.rs`'s three
`denominate` hits are the arithmetic denominator, and `boot.rs`/`relation_graph.rs`
both define unrelated `resolve_target` functions. And the census sees Rust symbols
only; every non-Rust consumer (`justfile`, `.claude/settings.json`, the scripts,
the `spec-*.toml` source anchors) needs a separate literal-string sweep.

**Probe 3 (the macOS/Darwin census) is still untouched** and remains the sharpest
open probe for a second adversarial pass. This sweep touched the Darwin path only
where it intersected the orphan question.

## Execution environment and standing directions (implementation sessions)

Established by the user at `/phase-plan` PHASE-01, 2026-08-13. These are facts
about *where this slice is being built*, not about what it builds — but several
of them invert assumptions the design and the skills would otherwise apply.

**This is a microVM capsule holding a separate checkout.** Consequences, each of
which switches off machinery the repo's standing guidance would otherwise
demand:

- **No dispatch, no worktree, no fork.** No main-worktree contention to isolate
  from, so `/worktree` and `/dispatch` are not in play — work happens directly
  in this checkout. The `edge`/`main` promotion ritual and the
  "never checkout the primary tree" rule are about the host repo, not here.
- **No confinement hooks.** The `PreToolUse` wall, the worker marker and the
  `DOCTRINE_WORKER` env leg are all *subjects* of this slice here, never active
  constraints on the session doing the work. A confinement behaviour observed —
  or not observed — in this capsule is evidence about nothing.
- **Transient state is not visible from the host.** Anything the host session
  would have seen in `.doctrine/state/` is absent or divergent here. Do not
  reason from its absence.
- **Claude is available two ways** — `claude -p` and in-session subagents. `pi`
  is installed but has **no API keys and no egress route configured**, so
  `./scripts/pi-scout` / `pi-research` will not work until the user sets that
  up. Research that needs them has to wait or route through claude.

**Entity-id collision risk — flag every minted id.** Ids minted in this capsule
(`DEC-`, `RV-`, `IMP-`, `ISS-`, `EVD-`, …) can collide with ids minted
concurrently on the host, because the reservation surfaces are not shared.
**Memories and observations are exempt** — those are uid-keyed, not counter-keyed.
Standing rule for the rest: **any id minted during implementation gets called out
explicitly in the phase sheet and in the commit message**, so reconciliation can
find and re-key them. Prefer not minting at all where a phase-sheet note or a
`notes.md` entry carries the same weight.

**Adapt the plan; keep receipts.** User direction, verbatim in substance: given
the choice between following the plan slavishly and doing the thing properly,
choose properly — and record the divergence. Two immediate applications:

1. **`PHASE-01`/`VA-1`'s byte-identity leg is relaxed.** The authored criterion
   asks that the two relocated `write_seatbelt_profile` unit tests be diffed
   line-by-line and be "identical but for the module they sit in". The user is
   not uptight about existing tests holding at the byte level. **What survives
   is the substance**: a relocated test whose *assertions* changed means the move
   was not behaviour-preserving and the phase has failed. Import-form and
   symbol-reference churn (`fs::` qualification, `BWRAP_BIN` → `jail::BWRAP`) is
   not a failure. `EX-4`'s `git diff --stat tests/` leg is **untouched** and
   still binds — that is about the e2e suites, which are the real
   behaviour-preservation proof.
2. **`PHASE-01`/`D1` is settled on the collapse** — see below.

**macOS is acknowledged and deferred.** `jail_prefix.rs:45`/`:47` are
`cfg(target_os = "macos")` and are not compiled in this capsule, so the
re-pointed imports there pass a green gate unverified. The user will sweep the
Darwin path in a later phase on AARCH64. This is `OQ-5`'s residual and design
§9.5's "deliberately not verified here", arriving one phase earlier than the
design anticipated; `PHASE-09`/`VA-2` (probe 3, the Darwin census) is where it
lands.

### Follow-up: `jail.rs`'s ADR-001 posture

**The finding.** `.doctrine/adr/001/layering.toml:138` classifies
`"worktree::jail" = "leaf"` with the comment *"pure jail core — no
disk/git/clock/rng"*, and `jail.rs`'s own module doc opens *"PURE leaf: no clock
/ git / disk / rng"*. Two of `DEC-206`'s four re-homed primitives are impure:
`have_bwrap` stats the filesystem (`dir.join(BWRAP).is_file()`) and reads the
environment; `write_seatbelt_profile` calls `fs::write`. The design does not
mention purity, ADR-001, or the leaf classification anywhere — the re-homing set
was derived from `jail_prefix.rs`'s imports, and the question of whether the
destination could hold them was never asked.

**Why PHASE-01 proceeds anyway.** The stated contract is already not the real
one. `jail.rs` houses `impl ResolveEnv for RealEnv` (`:865-916`), which shells
`getconf` and makes three `std::fs` calls. The module's genuine invariant is
*"impurity behind the injected `ResolveEnv` seam, so the pure surface stays
`FakeEnv`-testable"* — not "no I/O". The two arrivals are **unseamed free
functions**, which is new for the module, but routing them through `ResolveEnv`
would change `jail_prefix.rs`'s call sites, and PHASE-01 is defined by not doing
that. So: move verbatim per `DEC-206`, record here.

**Nothing mechanical catches this.** `tests/architecture_layering.rs` parses
crate-module `use` edges and checks tier direction; it has no notion of
`std::fs`. The gate is green either way — which is the reason to write this down
rather than the reason to let it go.

**What to do about it, in order of preference.**

1. **`PHASE-08` candidate target.** That phase re-derives the governance `REV`
   target set from `.doctrine/adr` against the corpus as it then stands (`EX-1`,
   `VH-2`), and design §5.6's six-entity table does not name `ADR-001`. This is
   exactly the "any entity or region beyond the six is **recorded, not
   absorbed**" case. At minimum `layering.toml:138`'s comment wants correcting to
   describe the seam rather than assert an absence that has not held since
   SL-183.
2. **If `PHASE-08` declines it**, a backlog card: seam `have_bwrap` and
   `write_seatbelt_profile` behind `ResolveEnv`, or split them out to a
   `worktree::jail_io` command-tier sibling. Either restores the stated contract;
   neither belongs inside a behaviour-preserving re-home.

**Standing caution for the rest of the slice.** `PHASE-04` deletes ~300 lines of
`jail.rs`'s pure decision layer and about half its unit tests. What is left is
disproportionately the impure remainder plus the argv builders. The leaf
classification deserves a deliberate look at that point rather than an inherited
one — the module that emerges is not the module ADR-001 classified.

### Escalation, PHASE-06: `worker-forbidden-writes` lost its only reader

**The finding.** The Batch B orchestrator (PHASE-05/06) verified directly that
`classify_import` (`import.rs:120`) — the belt `DEC-204`/`DEC-213` name as
`worker_commit`'s surviving replacement — never read
`DispatchConfig::worker_forbidden_writes`. It only ever enforced the two
hard-coded floors, `.doctrine/**` and `.claude/**`. `worker_commit` was that
config's *only* production reader, and PHASE-06 orphaned
`ForbiddenWrites`/`is_forbidden` the moment it was deleted — confirmed by the
compiler. So the config's configurable tail (`.agents/**`,
`install/agents/**`, `flake.nix` — "the highest security leverage in the
repo" per `install/doctrine.toml.example:121`) now enforces nothing. The two
code floors are unaffected. No config in this repo sets the key today, so
there is no live exposure — but the feature no longer does what it documents
for a project that does set it. Left honestly marked
(`expect(dead_code)` naming the gap) rather than silently dropped.

**Why this escalated rather than being settled phase-locally.** Fixing it
inside `SL-254` would reverse PHASE-06/VT-1 ("the import belt is UNTOUCHED")
and `INV-2` with a same-slice `DEC` — a worse trade than carrying it forward,
per the orchestrator's own read and the super-orchestrator's confirmation.

**Owner decision, 2026-08-13.** Accept the gap for `SL-254`'s scope; resolve in
`SL-255` instead, which is already building the clone-provisioning arm's new
commit/import machinery and so can add a reader without contradicting a
criterion this slice already closed. Refined by the owner: the fix should NOT
be a Rust-side belt that runs after a worker has already written — that is
strictly weaker than this slice's own confinement standard (`PHASE-02/VT-3`:
"refused by the KERNEL not a hook"). Instead extend the confinement prefix
builder to accept extra `--ro-bind` paths at spawn time, so
`worker-forbidden-writes` gets the same kernel-level enforcement the worktree
boundary already has. Captured as `IDE-051` ("Oubliette: enforce
worker-forbidden-writes via bwrap read-only binds, not a post-import belt"),
linked `originates_from SL-254` / `concerns SL-255`. `SL-255` should read
`IDE-051` before designing its commit/import replacement.

**PHASE-07 status at the time of this escalation.** Unblocked and independent
— every verb/tool its prose references was already in final state, and
PHASE-06 had already removed the retired config key's routing from
`plugins/doctrine/skills/dispatch/SKILL.md` (a skill telling an orchestrator to
read a deleted key is a live defect, not PHASE-07 prose). Proceeded without
waiting on this decision.

### PHASE-01 `D1` — settled: collapse `BWRAP_BIN` onto `jail::BWRAP`

Once `have_bwrap` moves out, `BWRAP_BIN` (`pretooluse.rs:71`) has exactly one
remaining reference — a unit test at `:516` — so it is dead under
`cfg(not(test))`, and `Cargo.toml:206`'s `warnings = "deny"` makes that a hard
compile error rather than a lint.

**Chosen:** delete `BWRAP_BIN`, promote `jail.rs:73`'s existing
`const BWRAP: &str = "bwrap"` to `pub(crate)`, point the moved `have_bwrap` at
it, and retarget the one test reference. **Rejected:** keep `BWRAP_BIN` alive
under a broadened `#[cfg_attr(not(test), expect(dead_code, …))]`.

The only argument for the rejected route was preserving a staying test's text
byte-for-byte, and the user has released that constraint. What remains is
one-sided: the collapse **removes** a live `STD-001` duplicate of the literal
`"bwrap"` across two modules of one subsystem, where the annotation route spends
an `expect(dead_code)` to **keep** it — and keeps it only until `PHASE-04`
deletes `pretooluse.rs` whole, at which point the duplicate would have gone
anyway. Paying a lint suppression for three phases of duplication is the worse
trade in every dimension. The test's assertion is untouched; only the symbol it
names changes.

### PHASE-08 re-derivation (`EX-1`, `VH-2`) — the target set is NINE entities, ~191 regions

*Run 2026-08-13 at the head of PHASE-08, against the corpus as it then stood
(PHASE-07 + PHASE-10 landed). Recorded here because the phase **escalated before
authoring** — see the escalation below — so this derivation must not be lost with
the session that produced it.*

**Method (the `R8`/`DEC-218` obligation — re-derive, don't re-cite).** Two
independent grep sweeps over `.doctrine/adr`, `.doctrine/spec`, `.doctrine/policy`,
`.doctrine/standard`, then every hit read in context:

1. *Mechanism sweep* — `pretooluse`, `nominat*`, `SubagentStart|Stop`,
   `worker_commit`, `arm-spawn`, `claude-force-subprocess`, `dispatch-agent`,
   `dispatch-subprocess`, `pi-spawn-confined`, `dispatch-orchestrator`,
   `dispatch-probe`, `drive-slice`, `WorktreeCreate`, `marker`, `verify-worker`,
   `in-session|claude arm|both arms`, `privileged.?agent`, `mode b`.
2. *Consequence sweep* — the vocabulary a falsified region uses when it does
   **not** name the mechanism: `stamp`, `subagent`, `altitude`, `self-commit`,
   `commit gate`, `isolation.?worktree`, `claim.?lock`, `strict-mcp`,
   `DOCTRINE_WORKER`, `jail.?prefix`, `arming`, `write_class`, `hook.?mint`,
   `degraded`, `in-session`, `nested agent`, `harness capabilit`.

Sweep 2 is what `DEC-218`'s method note predicted would be needed
(`ADR-012` `D3` says "the Claude `Agent` arm", never "subagent") and it is what
found `PRD-015` and `SPEC-022`, both of which `DEC-218`'s negative control had
cleared. Positive control: `SPEC-011`/`SPEC-023`/`SPEC-024`/`ADR-018`/`PRD-006`/
`PRD-014` all hit on `marker`/`nominat`/`altitude` in unrelated senses and were
correctly discriminated as false positives, as `DEC-218` also found.

**The count, against `DEC-218`'s floor.** Every entity read end to end.

| entity | `DEC-218` floor | re-derived | delta |
|---|---|---|---|
| `ADR-011` | 11 | **56** | +45 |
| `SPEC-012` | 10 | **44** | +34 |
| `ADR-008` | 7 | **35** | +28 |
| `ADR-006` | 9 | **23** | +14 |
| `SPEC-021` | 6 | **~16** | +10 |
| `ADR-012` | 3 | **~7** | +4 |
| `ADR-001` | *not in set* | **~4** | new |
| `PRD-015` | *not in set* | **~4** | new |
| `SPEC-022` | *anchor-only* | **~2** | new |
| **total** | **~46 / six entities** | **~191 / nine entities** | **4×** |

**The eleventh undercount, and the first that changes the instrument.** The
pattern held — always low, never high — but the size is new in kind, not only in
degree. `ADR-011` is ~56 falsified regions out of ~70: its **title's** second
half (*"per-harness capability altitude"*) is dead, and its **Verification**
section is now unsatisfiable — `VA-2` requires that *"no harness-specific command
(`claude -p`) appears as a required element"*, while `scripts/spawn-confined.sh:220`
runs exactly `claude -p --output-format stream-json`. An `accepted` ADR that fails
its own acceptance basis is a supersession question, not a 56-blockquote
amendment. `ADR-008` has the same shape: `D-B6` entire (the nominate / spawn-gate
/ `SubagentStop` mechanism) and `N1` entire (the `worker_commit` exception to the
`PreToolUse` wall) are void — both the exception and the wall it excepts from.

**The three entities beyond the six.**

- **`ADR-001`** — `layering.toml` registers two modules that no longer exist:
  `"worktree::pretooluse" = "command"` (`:139`) and `"worktree::subagent" = "engine"`
  (`:148`), plus `:149`'s comment describing `marker.rs` as doing *"file I/O for
  marker state"* when it is now a ~120-line env-only module. This is the candidate
  seventh target `## Follow-up: jail.rs's ADR-001 posture` already flagged, now
  confirmed with three more rows than that note anticipated.
- **`PRD-015`** — two must-fix regions. Its **Alternate flow — degraded harness**
  (`:129-131`) describes a harness that isolates and funnels at *"reduced
  enforcement altitude"*; `DEC-208` abolished that rung outright (fail closed at
  spawn, no unconfined fallback). And **`OQ-1`** (`:172-176`) rests the
  coordinator's write-permission on marker-*absence* with `IMP-065` as *"the real
  close"* — the marker is deleted and `IMP-065` closed **obsolete** 2026-07-02 via
  `REV-018`. Considered and **kept**: the *"Harness parity of guarantee, honesty of
  altitude"* success measure (`:110-112`) and the scope line (`:33`) — the promise
  is *state your shortfalls honestly*, which survives and is now trivially met.
- **`SPEC-022`** — `DEC-218` treated it as anchor-sweep-only, but `:205-210` is live
  **prose**, not a comment: *"the funnel (claude arm) does not always commit
  `boundaries.toml`"* (`ISS-039`). The `[[source]]` sibling comment at `:43-45` is
  the separate, already-known item.

**Anchor sweep (`EX-8`, `VH-1`) — complete, and `EX-8`'s count is exactly right.**
Every uncommented `identifier` in every `spec-*.toml` in the corpus checked against
disk. **Two** dangling anchors, both `spec-021.toml`: `:30`
(`plugins/doctrine/skills/dispatch-agent/SKILL.md`) and `:34`
(`…/dispatch-subprocess/SKILL.md`). Nothing else in the corpus dangles — the
`doctrine/cli` hits are inside commented template blocks, not live rows. Still to
do when the phase resumes: **add** the third anchor for
`plugins/doctrine/skills/dispatch-spawn/SKILL.md` (PHASE-07's own prediction), and
correct the two sibling comment lists (`spec-012.toml:28-30`, `spec-022.toml:43-45`)
which name the deleted `pretooluse.rs`/`subagent.rs` **and** omit the live
`claim_lock.rs`.

**Considered and ruled OUT of the target set, with reasons.**

- **`funnel-machine.md`** — a GENERATED artefact pinned byte-for-byte to
  `src/funnel_machine.rs` by a golden test. The machine retains `WorkerCommitted` /
  `RecordWorkerCommit`, and import's heal-forward still lands the
  `[Spawn, RecordWorkerCommit, Import]` prefix, so the text is TRUE. Independently
  reached, and it agrees with `DEC-218`. **Recorded, not absorbed:** the
  `worker-committed` position is now reachable *only* via that synthetic
  heal-forward prefix — no worker commits any more — so the name is a vestige.
  Renaming it is a code change outside this slice; backlog candidate at reconcile.
- **`ADR-020` / `SPEC-030`** — not targets. `SPEC-030:195-196` states *positively*
  that harness-specific in-session subagent identity is outside the capsule
  contract, which SL-254 confirms rather than falsifies. **But `ADR-020` is a
  constraint** — see the escalation below.
- **Pre-existing drift found while sweeping, NOT caused by this slice** — held out
  of the REV deliberately, since folding it in would make SL-254 the reconciler of
  other slices' debt: `SPEC-012`'s five-vs-eight `Tier` variants (`allowlist.rs`
  now has eight); `SPEC-012`'s `descends_from = "PRD-015"` contradicting its own
  body's *"carries no descent"*; the `CARGO_TARGET_DIR` redirect retired by SL-156
  but still asserted in `ADR-008`'s Context, `ADR-006` `D9`, and `ADR-011`'s
  References; `REQ-252`'s per-worktree env contract, dead since SL-156 (`fork.rs:216`
  emits nothing) — of which only the *"subprocess-only (codex/pi)"* parenthetical is
  SL-254's to narrow. Backlog candidates at reconcile; no ids minted (collision rule).

### ESCALATION, PHASE-08: `ADR-020` reserves these four ADRs to `REV-046`

**The finding.** `ADR-020` (**accepted**, "Adopt execution capsules as the dispatch
authority boundary") carries two clauses that name exactly the ADRs this phase's
REV revises:

> This ADR establishes target architecture, not current implementation. ADR-011 and
> the incumbent worktree dispatch remain authoritative until REV-046's cutover gates
> are met. (`:86-88`)

> ADR-006, ADR-008, ADR-011, ADR-012 — incumbent worktree, confinement, spawn,
> and integration decisions **revised only at cutover through REV-046**. (`:145-146`)

`REV-046` is `proposed · approval=none`, and its own rationale says it *"does not
authorize deleting present-tense governance before an implementation exists"* and
that *"ADR-011 and the existing dispatch specs remain authoritative for shipped
dispatch until the capsule cutover."*

**Why this was missed.** `DEC-211` and `DEC-218` both checked `ADR-020` and
`REV-046` — but only as REV **targets** ("does ADR-020 need changing?"), and both
cleared them as not-applicable. Neither asked whether `ADR-020` **constrains** what
this REV may touch. That is the `/canon` question, and it was never put.

**Why it is genuinely ambiguous rather than a simple blocker.** The `:145-146`
clause sits in a References section and reads two ways, both defensible:

- *(constraining)* those four ADRs may be revised **only** through `REV-046`, in
  which case PHASE-08's REV is out of order as planned; or
- *(self-scoping)* **`ADR-020` itself** revises them only at cutover — a statement
  about ADR-020's own reach, not a prohibition binding other slices.

The second is probably the author's intent, and SL-254 is not a capsule adoption
(`DEC-203` bounded it). But the first reading is available on the words, and there
is a sharper second-order question underneath it: **`REV-046`'s rationale
enumerates precisely the mechanisms SL-254 has now deleted** — *"worktree marker
identity, `DOCTRINE_WORKER`, SubagentStart stamping, base-by-placement, the gated
`worker_commit` exception, per-harness arm routing and altitude"*. So either
SL-254's arm collapse **is** part of `REV-046`'s cutover (and these corrections
belong in `REV-046`, whose gates and status then need attention), or it is not
(and `REV-046`'s rationale is itself now partly stale, describing as
unimplemented-target a set of deletions that have shipped — which would make
`REV-046` a **tenth** target of this REV).

**Why it escalated rather than being settled phase-locally.** Every route out
requires a call with governance weight beyond the `DEC-2xx` set this slice minted:
route (A) proceeds with a new REV and appends a scoping note to `ADR-020` — this
slice editing the accepted ADR that arguably forbids it; route (B) lands the
corrections through `REV-046`, coupling SL-254 to capsule evidence it does not
have; route (C) proceeds with a new REV on the self-scoping reading and records the
rationale without touching `ADR-020`. **Recommendation: (C), falling back to (A)**
— the corpus should not knowingly carry ~191 false regions until an unscheduled
cutover, and the falsity is a consequence of shipped deletions, not of adopting the
capsule architecture.

**Second, coupled question for the same ruling.** Given `ADR-011` at 56/~70 regions
with a dead title-half and a self-failing Verification section, is `EX-2`'s *"in-place
amendment"* still the right instrument, or is `ADR-011` (and `ADR-008`'s `D-B6`/`N1`)
now a **supersession**? `EX-2` was written when the count was 11.

**State at escalation: nothing authored.** No governance file edited, no REV minted,
no id minted, tree clean but for a pre-existing `skills-lock.json` modification that
is not this session's. `just gate` untouched and green as PHASE-10 left it.

**Owner ruling, 2026-08-14.** Route **(C)**: proceed with PHASE-08's REV now, over
the re-derived nine-entity target set, on the self-scoping reading — `ADR-020`
is NOT edited in this phase. `ADR-011` is amended in place (not superseded),
matching how the other eight entities are handled and keeping the decision
history in one document despite the scale.

**Standing follow-up, explicitly carried to `/reconcile`, not this phase:**
review `ADR-020` for consistency once SL-254's REV has landed — the owner's
own framing ("make sure it's left in a consistent state") is broader than the
blocking question this escalation raised, so `/reconcile` should re-read
`ADR-020` fresh against the shipped result, not just re-check this escalation's
narrow two readings. Do not let this fall out of the reconciliation brief.
