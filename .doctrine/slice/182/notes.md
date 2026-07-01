# Notes SL-182: Claude-arm subagent write-confinement hooks

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

---

## ▶ STATE (2026-07-01) — SECOND INQUISITION (RV-201), design LOCK BROKEN → reconcile before `/plan`

Lifecycle: **design** (LOCK CONTESTED). Second adversarial round on the *reconciled*
design (codex GPT-5.5 + inquisitor) → **RV-201: 5 findings, 1 blocker, 3 majors, 1
minor** — all verified against the SOURCE SEAMS, not the prose. `/plan` is **NOT**
clear until reconciled. Ledger: `doctrine review show RV-201`.

- **F-1 (BLOCKER, option-bearing) — preferred registration ships FAIL-OPEN.** D-reg's
  "resolved absolute doctrine (fail-closed)" is false as-built: the plugin `hooks.json`
  is a verbatim byte-copy (src/skills.rs:1046-1049) of bare-`doctrine`
  (plugins/doctrine/hooks/hooks.json:7,18); `resolve_exec` is never invoked there.
  Fail-closed holds ONLY on the settings.local fallback. **User decision needed:**
  template the embedded JSON through resolve_exec at materialization, OR embed+
  materialize an exit-2 shim (and name what invokes it). Mirrors RV-200 option-bearing.
- **F-2 (major) — capture-before-remove leads with the wrong hook.** §5.4 leads with
  WorktreeRemove (no decision control, fire-and-forget, hooks.md:680/814/2442);
  SubagentStop is the blocking-capable/awaited capture point (hooks.md:658,1930-1957).
  Commit to SubagentStop; demote WorktreeRemove to cleanup; re-pin OQ-2 abort to it.
- **F-3 (major) — scope split-brain.** slice-182.md objective 3 (47-50) STILL says
  agent_id keying / per-worker / extra_ro / strict-loose — RV-200 F-4's "scope doc
  corrected" is a false attestation. Finish the scope rewrite.
- **F-4 (major) — shared-profile safety rests on unspecified machinery.** Declaration
  file unnamed, no atomicity contract with `base`, create-fork provision step is
  net-new (classify_create writes nothing under jail/ today, create.rs:166-187). MODEL
  is sound; ground "must not interleave" in the blocking-Agent structure, not discipline.
- **F-5 (minor) — vestigial resolve_exec** in §5.1/D1 runtime layer; install-time fix.
- **Acquitted:** V-plugin deferral defensible; OQ-2 defined-abort holds; per-arming
  keying MODEL is right. Next: `/reconcile` (or a `/design` sitting) on F-1's option.

### Prior round — RV-200 (reconciled, terminal; design was LOCKED)

`/inquisition` (codex GPT-5.5
+ inquisitor) → **RV-200: 10 findings, 3 blockers** — ALL reconciled into design.md
+ scope this session; RV-200 findings verified terminal. Two User decisions taken:
**F-1 = serial-scope, parallel workers SHARE ONE PROFILE** (User steer: prefer
"share one profile" over "baseline-only" — rationale durable in design §5.3);
**F-3 = snapshot-before-remove** (capture worker diff in a `WorktreeRemove`/
`SubagentStop` hook before the harness auto-removes the tree; abort to Path C/IDE-024
if the capture can't observe the tree intact). **Next step: `/plan`.** Verdict +
synthesis: `doctrine review show RV-200`.

### RV-200 verdict (the heresy)

- **F-1 (blocker)** per-worker custom policy is UNBUILDABLE through the single-slot
  arming rendezvous (`arm-spawn` = one shared `base`; `dispatch-agent` allows N
  parallel spawns/arming). Cut to strict default floor (rec) or serial-scope it.
  → couples F-4 (D2 §7 + authored scope still say `agent_id` keying §5.3 repudiated).
- **F-2 (blocker)** installer fails OPEN: bare-PATH plugin exec + only `exit 2`
  blocks (hooks.md:629-643) ⇒ stale/missing binary runs UNCONFINED (RSK-014
  reopened). §5.1/D1 (resolve_exec) contradicts §5.4 (bare PATH). Fail closed:
  absolute resolved exec or a shim that `exit 2`s on not-found.
- **F-3 (blocker)** funnel convergence rests on doc-DISFAVORED teardown:
  `WorktreeRemove` auto-`git worktree remove`s the subagent worktree on finish,
  NO decision control (hooks.md:2442/680/814) ⇒ uncommitted diff destroyed;
  "identical on both arms" is FALSE (pi orchestrator owns lifecycle, claude harness
  doesn't). Name a contingency: snapshot `git diff` in WorktreeRemove/SubagentStop
  before removal (rec), or Path C/IDE-024, or defer ro-`.git`.
- **majors** F-5 V-plugin fallback forbidden→make D-reg conditional, fallback
  same-phase · F-6 Edit/Write wall matches UNDOCUMENTED `NotebookEdit`/`notebook_path`
  (drop or pin schema first).
- **minor/nit** F-7 `network=true` default vs §4 "strictest floor" wording · F-8
  policy file's false "ancestor" rationale (ro-ness is `--ro-bind / /`) · F-10 §10
  understates doc coverage (agent_id hooks.md:595, updatedInput :818 ARE doc'd).
- **F-9 ACQUITTED** R7 orchestrator pass-through residual is defensible — agent_id
  harness-stamped present-iff-subagent (probe), worker can't forge absence; OQ-5
  deferral sound. Soft-target-4 answered: accepted, not must-land.

## HANDOVER — for the inquisition agent

Read in order: `doctrine slice show SL-182` (scope), then **`design.md`** (the
target), `doctrine backlog show RSK-014` (the proven probe this graduates),
`doctrine adr show ADR-008` (the confinement gap this closes) + ADR-006 (D2b /
sole-writer). Evidence/apparatus: `.doctrine/backlog/risk/014/probe-h1/`
(`results.md`, `pretooluse-wrap.sh`, `pretooluse-pathcheck.sh`). Recipe memory:
`mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement` (trust high,
verified). Proven flag set: `scripts/pi-spawn-confined.sh`.

**Verify hook claims against `docs/claude`** (local official-docs cache,
authoritative over web/haiku — per CLAUDE.md). Already cross-checked: plugin
`PreToolUse` supported (`plugins-reference.md:111-119`), matcher regex
(`:98`), no hot-reload → `/reload-plugins` (`:394`).

### The design in one breath

Graduate the proven two-wall confinement (`PreToolUse(Bash)` → nested bwrap rw-only
the worktree + ro-`/`; `PreToolUse(Edit|Write)` pathcheck `realpath ⊆ cwd`) from
probe bash scripts into a **Rust subcommand** `doctrine worktree pretooluse` (pure
`jail.rs` + thin `pretooluse.rs`), registered via the **plugin `hooks.json`**.
Per-worker jail policy (`extra_rw`+`network`) keyed by worktree name, provisioned by
`create-fork`. `.git` hard-ro → worker can't self-commit → claude `/dispatch` funnel
converges onto the pi arm's working-tree-diff import. Linux/bwrap only; fail-closed
when bwrap absent (macOS → IMP-045). **Path L** (linked worktree); the standalone-
clone alternative is **IDE-024**; selector-sourced write-allowlist is **IDE-025**.

### Where to push hardest (the soft targets)

1. **A7 keying model (freshly rewritten — highest risk).** The first draft keyed
   the policy by `agent_id` written by the orchestrator pre-spawn; that is
   IMPOSSIBLE (`agent_id` is harness-assigned at spawn). Now: orchestrator
   (`dispatch arm-spawn`) declares policy to a pre-spawn handshake location;
   `create-fork` (knows the new worktree `name`) provisions it to
   `<main>/.doctrine/state/dispatch/jail/<name>.toml`; PreToolUse resolves by
   `cwd → basename`. **Is this handshake actually race-free and is the pre-spawn
   declaration location real?** Does `create-fork`'s payload truly carry the name
   for the *claude Agent* spawn (vs the benign passthrough)? Probe finding 5 says
   benign spawns are detached-HEAD passthroughs — confirm the *armed* worker path.
2. **V-plugin bet (R2/D-reg).** The probe proved PreToolUse via `settings.local.json`,
   NOT the plugin path. Design *chooses* plugin (user steer: prior tests uniform).
   Docs confirm the event is supported, but **does it fire for a worktree
   subagent + honour `updatedInput` via the plugin path?** Unproven; gated as the
   first execute step. Is gating-not-proving acceptable at design-lock?
3. **Funnel convergence (objective 5 / R8).** ro-`.git` removes claude self-commit,
   breaking the `B..S` single-commit delta-check (`dispatch/SKILL.md:46`). Design
   converges to working-tree-diff import. **OQ-2: does the claude harness surface
   the worktree diff to the orchestrator when the worker's `git commit` is blocked
   RO?** Unverified harness behaviour (could the harness drop uncommitted changes on
   worktree collapse?). End-to-end gate, but is it a design-blocker?
4. **R7 orchestrator pass-through god-mode.** Accepted residual (OQ-5 deferred).
   Is "accepted" defensible, or must OQ-5 land with this slice?
5. **INV-5 shell-quoting** in `opaque_wrap` (paths with spaces/quotes) — injection
   surface; confirm the test pins it.
6. **D3 `.git` hard-ro** — is the `validate_policy` rejection of `.git`-touching
   `extra_rw` complete (symlink/`..` evasion of the reject)?

### Don't-lose / durable findings

- **Funnel discovery:** the claude `/dispatch` arm currently *expects a worker
  commit* (`dispatch/SKILL.md:46` delta-check), unlike pi. Confinement forces
  convergence — both arms onto working-tree-diff import. This is the real
  cross-cutting consequence of the slice.
- **Efficiency tradeoff (R8):** convergence imposes the pi arm's "can't trust
  worker green → orchestrator re-runs suite" cost on the claude arm. Deliberate;
  IDE-024 (clone + cherry-pick) is the efficiency recovery, prioritise on observed
  cost.
- **Existing hook machinery is all Rust subcommands** (`boot --emit`,
  `worktree create-fork`, `worktree marker --stamp-subagent`) installed via
  embedded `plugins/doctrine/hooks/hooks.json` (auto-discovered) — the seam this
  rides. `src/skills.rs:1024` install; `src/worktree/create.rs:295` create-fork
  handler; `src/boot.rs:1098+` settings hook merge (the fallback path).
- **Decisions locked:** D1 Rust subcommand · D2 per-worker policy (worktree-name
  key) · D3 `.git` hard-ro · D4 Path L · D5 single-sourced bwrap core + parity
  test · D6 schema (`extra_rw`+`network`, footgun-deny) · D-reg plugin hooks.json
  (gated V-plugin).
- **Touch-set (design-target selectors):** `src/worktree/{jail,pretooluse,mod,
  shared,create}.rs`, `src/dispatch.rs`, `.claude/skills/dispatch-agent/SKILL.md`,
  `plugins/doctrine/hooks/hooks.json`. **+ a `WorktreeRemove` capture hook** (F-3
  snapshot-before-remove; new hook handler alongside `create-fork`) and the
  **fail-closed exec resolution** at install (F-2; `src/skills.rs` / `boot.rs`
  materialization emits an absolute `doctrine` path or exit-2 shim).

### Durable harness gotchas confirmed by RV-200 (→ `/record-memory` candidates)

Verified against `docs/claude` (authoritative cache), high confidence:

- **PreToolUse hooks fail OPEN.** Only `exit 2` blocks a tool call; ANY other
  non-zero exit (incl. command-not-found 127 from a missing/stale binary) is a
  NON-blocking error and the tool PROCEEDS (`docs/claude/hooks.md:629-643` + the
  Warning). A hook meant to enforce confinement MUST resolve to a guaranteed-present
  absolute binary or use a shim that `exit 2`s on exec failure — bare-PATH exec is
  not fail-closed. (Exception: `WorktreeCreate`, where any non-zero aborts.)
- **`WorktreeRemove` auto-destroys an `isolation:worktree` subagent's tree on
  finish.** Fires when the subagent completes; Claude runs `git worktree remove`
  automatically; the hook has NO decision control and failures are debug-log-only
  (`hooks.md:2442`, `:680`, `:814`). Uncommitted worktree changes are LOST unless
  snapshotted before removal. Consequence: a harness-owned worktree (claude Agent
  arm) is NOT lifecycle-equivalent to an orchestrator-owned worktree (pi/subprocess
  arm) — any "import the worker's diff" cadence must capture before teardown.
- **Single-slot arming rendezvous can't key per-worker state.** `dispatch arm-spawn`
  writes ONE shared `base` file per arming dir; `dispatch-agent` issues N parallel
  spawns off one arming (all read the same B). The harness-assigned worktree `name`
  exists only at create-fork, not pre-spawn — so any per-worker pre-declared state
  through the arming dir is batch-shared, not per-worker. (Dispatch design fact.)

## PHASE-01 LIVE BATTERY VERDICT (2026-07-01, post-restart) — SL-182

claude-code 2.1.x / NixOS bwrap 0.11.2. Two `isolation:worktree` general-purpose
subagents driven via the `Agent` tool. All three probe items GREEN; teardown clean.

- **F-T1 (EX-1, item 1 — plugin PreToolUse fires + honours `updatedInput`).**
  Plugin-registered (`.claude/skills/doctrine/hooks/hooks.json`) PreToolUse FIRES
  for a worktree subagent and honours `updatedInput` — `wrapper.log` shows `WRAP`
  for every subagent Bash (PASS-THROUGH for the orchestrator). Containment held
  under live escape battery: parent-repo Bash write → `Read-only file system`
  (bwrap `--ro-bind /`); `/tmp` Bash write → jail `--tmpfs /tmp` (host `/tmp`
  untouched, verified absent); Write-tool parent escape → DENIED (native
  shared-checkout guard); Write-tool `/tmp` escape → DENIED (`pathcheck`
  `escapes-worktree`); in-worktree writes (Bash + Write) PASS and persist to the
  real worktree (bind mount). Canaries ALL intact (`verify.sh` OK). ⇒ **VERDICT:
  YES — PHASE-03 builds the plugin registration path. NOT S2.**
  - Note: native harness ALSO guards Edit/Write against shared-checkout paths
    (its own message: "This agent is isolated in the worktree … Edit the worktree
    copy"). Our `pathcheck` is the strictly-wider wall (catches `/tmp`, `$HOME`,
    anywhere the native guard does not). Both walls confirmed independently.

- **F-T2 (EX-2, item 2 — SubagentStop blocking + tree-intact + correlation).**
  ALL THREE sub-claims green:
  - **Blocking/AWAITED:** one-shot `BLOCK_ONCE` exit-2 HELD the stop — two `STOP`
    events ~3s apart for the same agent (capture → exit 2 → subagent re-activated,
    acknowledged on stderr → stopped → capture again, sentinel consumed). The
    harness runs SubagentStop to completion and honours exit 2.
  - **Tree-intact:** `CAPTURE-OK tree-intact` — `git -C <wt> diff` succeeded at
    hook time; patch (393 B) + untracked tar captured OUTSIDE the worktree under
    `.harness/probe/captured/`. Capture lands BEFORE `git worktree remove` ⇒ the
    funnel-import seam (PHASE-05) is viable.
  - **Correlation:** payload carries `agent_id` + `agent_transcript_path` + `cwd`
    (= worktree) and **NO `worktree_path`** (RV-202 confirmed live). All three
    correlators resolve: (a) `<root>/.worktrees/agent-<agent_id>`, (b) hook
    `cwd`/pwd, (c) SubagentStart `agent_id→cwd` map. Winner by probe order = **(a)**.
  ⇒ **VERDICT: ALL YES — Path L holds. NOT S1.**

- **F-T3 (EX-3, item 3 — hook `command` is shell-run).** `SHELLRUN_MARKER` present
  (`printf shellrun > MARKER` redirection executed by a shell, not literal argv).
  ⇒ **shell-run confirmed — the F-1 `|| exit 2` vanish-guard is viable in PHASE-03.**

### Decisions resolved
- **D-reg-path = plugin** (`.claude/skills/doctrine/hooks/hooks.json`) — F-T1.
- **D-funnel-path = Path L** (SubagentStop awaited capture-before-remove) — F-T2.
- **D-shellrun = shell-run ⇒ `|| exit 2` guard viable** — F-T3.
- **D-correlator = (a) `agent-<id>` path**, with (c) SubagentStart map as the
  robust fallback if the naming convention ever drifts (the map is harness-token-
  independent). Both proven to resolve live.

VA-1: items 1/2/3 each pinned-green; registration path = **plugin**, funnel path =
**Path L**. Teardown clean (EX-4). Awaiting **VH-1** (User accept) before PHASE-02.

---

## PHASE-02 — pure jail core (`src/worktree/jail.rs`) COMPLETE (2026-07-01)

TDD T1–T8 landed against the T0-locked interface; T9 gate green. 31 in-module
tests cover VT-1..VT-8. `cargo clippy --bin doctrine` zero-warn; `cargo test
--test architecture_layering` 17 pass (MixedUmbrella green — leaf tier holds).
Commit `b67b6299`.

### Adjudicated T0 decisions — as-implemented
- **Typed `PolicyError` enum** (`IsRoot`/`AncestorOfMainRoot`/`TouchesGit`/
  `Malformed(String)`) for `validate_policy` + `from_toml_str`. **Diverges from
  design §5.2's literal `Result<_,String>`** — sanctioned phase-delegated seam
  decision (T0 decision 3). **RECONCILE DEBT: coherence-flag §5.2 at audit** so
  the design text matches the impl. Deny *reasons* that ride to the user
  (`Backend::Deny{reason}`, `Target::Reject`) stay `String` (they ARE the JSON).
- **`base64` crate added** (`base64 = "0.22"`, workspace + root dep) — leaf-legal
  external, cf. `worktree::allowlist`→`glob`. Standard alphabet == probe's
  `base64 -w0`/`-d`.
- **`Path::starts_with` component-wise** (never string prefix) — VT-2 pins the
  sibling-prefix guard (`/wt` ⊄ `/wt-evil`).
- **`#[serde(deny_unknown_fields)]`** — a typo'd key is `Err(Malformed)`, not a
  silent fall-through to the permissive Default floor (VT-3 unknown-field case).

### Decisions resolved this phase
- **D-parity-source = extract-at-test-time (token-filter).** `pi_spawn_core_tokens`
  reads `scripts/pi-spawn-confined.sh` at test time, strips comment lines, splices
  `\`-continuations, takes tokens between `bwrap` and `pi`, and filters pi-specific
  groups (`--bind <…/.pi> <…/.pi>`, `--setenv NAME VAL`). A core-flag edit to the
  script breaks VT-7 loudly (R2 mitigated). Line-slice rejected (handover/plan
  disagreed on the range; the pi bind interleaves the core flags).
- **D-opaque-exec-test = hermetic `sh -c`.** VT-5 assembles the wrap, runs it via
  `sh -c`, and asserts stdout. `env P=<space+quote path>` threads the tricky value
  through argv single-quote-escaping; the decoded `orig_cmd` echoes `$P` back ⇒
  round-trip AND execution proven in one shot. Host has coreutils (no skip needed).

### SL-183 seam-gap — RESERVED (load-bearing)
`Backend::Seatbelt(ResolvedMac)` (was a unit variant) is the **additive data
channel**: `select_jailer` threads `ResolvedMac` into the `Seatbelt` jailer; the
macOS `wrap_argv` body is deferred (`unimplemented!("SL-183")`, unreachable on
Linux). `ResolvedMac` is an empty `Default` struct today — SL-183 populates the
fields (getconf DUTMP, TMPDIR, materialized profile path) and fills `wrap_argv`
with **no SL-182 signature refactor** (OQ-mac3 satisfied). Field-level
`#[cfg_attr(test, expect(dead_code, ...))]` self-clears when SL-183 reads it.

### Boundary obligations carried forward (do NOT lose)
- **R4-canon → PHASE-03/04.** The leaf trusts every path arrives shell-
  canonicalized (symlink-resolved, absolute) and each `extra_rw` **materialized**
  (bind-source existence, TOCTOU-safe). Security-load-bearing; no leaf test can
  catch a bypass. PHASE-03 (`decide_write` `real`) + PHASE-04 (`extra_rw`
  provision) MUST carry boundary tests asserting canonicalization + materialization
  before the leaf is called.
- **MF-3 reconcile debt.** plan.toml PHASE-02 **EX-1 lists `load_policy`** in the
  pure surface, but `load_policy` reads disk (shell-owned) and is correctly ABSENT
  (leaf owns only `from_toml_str`). EX-1 unsatisfiable as written by a pure leaf.
  plan.toml is authored/locked ⇒ correct EX-1 text (`load_policy`→`from_toml_str`)
  via `/reconcile` at audit; do NOT silently edit.

### Lint note (dead_code expect topology)
The module carries `#![cfg_attr(not(test), expect(dead_code, ...))]` — the pure
surface has no `not(test)` consumer until PHASE-03. Under `test` the VT suite
makes the surface live, so the module expect is `not(test)`-scoped; the lone
still-dead item under `test` (the reserved `Seatbelt.resolved` field) carries its
own `#[cfg_attr(test, expect(dead_code))]`. Both expectations verified fulfilled
against ground truth (`cargo test` + `cargo clippy --bin doctrine`), not the LSP.

---

## PHASE-03 — pretooluse shell, registration & fail-closed install (2026-07-01)

Implementation COMPLETE; **VA-1 (live) pending** (needs install + session restart
— cannot run in the authoring session). Phase stays `in_progress` until VA-1.
Commits: `d457859a` (T1-T3/T5 shell) · `7b48995c` (T4/T6/T7/T8 install+register).
Green: 11 `worktree::pretooluse` + 49 `skills::tests` + 17 architecture_layering;
`cargo clippy --bin doctrine` zero-warn; fmt clean.

### Decisions resolved this phase
- **D-anchor = `CLAUDE_PROJECT_DIR` + git-common-dir equality (A1).**
  `cwd_is_project_worktree` = `is_linked_worktree(cwd)` AND
  `common_git_dir(cwd) == common_git_dir(CLAUDE_PROJECT_DIR)`. **Robust either
  way:** the equality holds whether the anchor points at the main tree or any
  same-repo worktree (both resolve to the one shared `.git`); a sibling repo's
  worktree (e.g. a ro-mounted `/workspace` repo) differs ⇒ Reject. Replaces the
  probe's hard-coded `$ROOT/.worktrees/agent-*` prefix. Fail-closed: absent anchor
  or any git error ⇒ `false` ⇒ Reject (deny). Added `shared::common_git_dir`
  (DRY — `is_linked_worktree` now rides it).
  - **EMPIRICAL GATE (VA-1):** that `CLAUDE_PROJECT_DIR` is SET (and points at
    main, or any same-repo tree) in a *worktree-subagent* hook process is
    UNVERIFIED — the PHASE-01 probe hard-coded ROOT and never dumped hook env. If
    absent for real workers, ALL worker writes/Bash deny (over-confinement — safe
    but breaks dispatch). VA-1 must confirm presence; if absent, fall back
    (is_linked_worktree-only, accepting the sibling-repo edge as deferred
    hardening) — a `/consult`, not a silent change.
- **D-canon-impl = shell out to `realpath -m`.** `canonicalize_missing` runs the
  host `realpath -m` (missing-safe), joining `cwd` for relative paths — byte-parity
  with the proven `pretooluse-pathcheck.sh`. Chosen over a lexical normalizer for
  security fidelity on the write-wall (correctness-first). **Efficiency note:** a
  subprocess per Edit/Write (plus 2-3 git rev-parse per call for topology) on the
  hook hot path; the orchestrator fast-path (no `agent_id`) short-circuits BEFORE
  any of it. A lexical realpath-m (no subprocess) is a viable later optimization
  if measured — not needed now.
- **D-quote-reuse.** Install templating reuses the leaf's INV-5
  `jail::shell_single_quote` (now `pub(crate)`, re-exported from `worktree`) to
  single-quote the baked absolute exec — one canonical POSIX escaper, no parallel
  impl (design §5.4 tied the quoting to "same discipline as INV-5"). Candidate for
  a shared leaf util if a third consumer appears.
- **D-policy-floor.** P03 resolves policy to `JailPolicy::default()` (strictest
  floor). The per-worktree `jail/<name>.toml` lookup by `cwd→basename` +
  `validate_policy` is **PHASE-04** — wired at the single `let policy = …` site in
  `run_pretooluse`.
- **Guard class = Read.** `WorktreeCommand::Pretooluse` writes no authored state
  and runs INSIDE the confined subagent (worker context) on every tool call ⇒
  must be open under worker-mode (`commands/guard.rs`).
- **Unregistered tool ⇒ PassThrough** (not deny): the matcher only routes
  Bash/Edit/Write; guarding an unread tool would be a latent jail hole (§5.2).

### VT/EX map (as-built)
- EX-1/VT-1 → `decide`+`render`, all §5.2 shapes (bash-wrap, write-deny,
  orchestrator pass-through, isolation:none deny, INV-2 repo-root deny, in-worktree
  pass). EX-2/VT-2 → `template_hooks_commands` + `install_materializes_…` (real
  install path bakes absolute exec, both walls, `|| exit 2`). EX-3/VT-3 →
  `jailed_bash_with_no_bwrap_backend_denies…` (runtime Deny) + templating guard
  (exec-vanish). R4-canon → T4 tempdir boundary (`..` + symlink escape denied).
  EX-4/VA-1 → **pending live** + runbook shipped (dispatch-agent SKILL.md).

### Carry-forward / reconcile debt (unchanged, still open)
- **MF-3** — plan P02 EX-1 lists `load_policy`; leaf exposes only `from_toml_str`.
  Fix EX-1 text via `/reconcile` at audit.
- **§5.2 PolicyError divergence** — typed enum vs design's literal
  `Result<_,String>`; coherence-flag at audit.
- **VA-1 — RESOLVED GREEN (2026-07-01).** Install verified (both walls, absolute
  exec, `|| exit 2`). Tier-A installed-exec battery (8 cases) + Tier-B live
  registered-hook battery both green — full log at
  `probe-evidence/logs/pretooluse-installed-battery.log`. Live worktree subagent:
  in-worktree write ALLOWED (⇒ `CLAUDE_PROJECT_DIR` present in the HOOK env,
  D-anchor CONFIRMED), escape write DENIED (`worktree-jail: escapes-worktree`),
  bash ran confined. Both walls fire; updatedInput honoured (Tier-A emission +
  P01 harness-apply proof). Registration via `/reload-plugins` ALONE (no full
  restart). D-anchor EMPIRICAL GATE closed green — no /consult, no fallback taken.
  Funnel: Path L + plugin registration.
- **jail.rs dead_code expect** — still fulfilled in P03 (validate_policy /
  from_toml_str stay `not(test)`-dead; the shell consumes the rest). P04 makes
  them live ⇒ narrow/remove the module expect then.

### CHR-014 cross-slice catch (2026-07-01, commit 66821afe)
SL-183 agent surfaced (sl182.txt) that PHASE-03's VT-7 helper
`pi_spawn_core_tokens()` baked the repo root via `env!("CARGO_MANIFEST_DIR")`
(jail.rs:597, from b67b6299) — a CHR-014/SL-162 violation tripping
`e2e_no_baked_paths`. Unit-only test runs had masked it; the full
`check commit` recipe was RED on baseline, blocking SL-183's own gate.
Fixed as owner: swap to runtime `test_support::repo_root().join(...)`
(the guard's mandated form). Test-only, behaviour-identical. Guard now green.
Cross-slice ownership stayed clean because SL-182's agent made the fix.
Audit: fold into VT-7 conformance evidence.

---

## PHASE-04 — Per-arming policy provision (objective 3) COMPLETE (2026-07-01)

All three write/read seams wired end to end; `doctrine check gate` exit 0
(workspace clippy zero-warn + fmt + tests), `architecture_layering` 17 pass.
Commits: T1 (Serialize on JailPolicy) · T3 `b42df19c` (create-fork provision) ·
T4 (arm-spawn declares) · T2/T5 (pretooluse resolve). EX-1..EX-4 met.

### The surface, as built
- **arm-spawn declares** (`dispatch.rs run_arm_spawn` + `write_arming_jail_policy`):
  `--extra-rw <path>` (repeatable) + `--no-network` build a `JailPolicy`; written
  as `spawn/jail.toml` beside `base` in ONE arming step (F-4). A Default policy
  (no flags) writes nothing and CLEARS any stale `jail.toml` (pairing hygiene —
  a leftover declaration can never ride a fresh base). D-armcli User-decided.
- **create-fork provisions** (`create.rs provision_jail_policy`, NET-NEW in
  `act_on_create` Fork arm): copies `spawn/jail.toml` → `jail/<name>.toml` under
  the coord root, keyed by the harness worktree name, before the worker's first
  tool call. Absent declaration ⇒ no-op ⇒ Default. Copy failure with a present
  declaration ⇒ propagate (create-fork fail-closed, aborts spawn).
- **pretooluse resolves** (`pretooluse.rs resolve_provisioned_policy`+`load_policy`,
  replaces the `JailPolicy::default()` seam at :241): keyed `cwd →
  basename(worktree) → jail/<name>.toml`. Binary outcome — a fully-valid
  materialised policy, or Default. NEVER denies (objective 3).

### Decisions resolved this phase (as-built deltas — HARVEST AT AUDIT)
- **D-provision-site = `create.rs:act_on_create`, NOT `fork.rs:fork_core`.** §5.3
  says "beside run_provision + write_marker" — those calls live in `fork_core`,
  but that copier is SHARED by non-dispatch fork paths and writes INSIDE the fork;
  the jail provision is dispatch-create-fork-specific and writes OUTSIDE the fork
  (coord root, name-keyed). So it lands in `act_on_create` after `fork_core`
  succeeds. Matches VT-1 `test_file=create.rs`. `fork_core` unchanged.
  → coherence-flag §5.3 wording at audit.
- **D-mainroot = strip `.worktrees/<name>`, NOT git-common-dir.** git-common-dir
  points at the PRIMARY `.git`, not the coord root where `.worktrees/` + the jail
  dir co-reside. Reader recovers the coord root by stripping the doctrine-OWNED
  `.worktrees/<name>` layout (create.rs owns both write + read shape; promoted
  `WORKTREES_SUBDIR` `pub(crate)`). Locating the file is layout-owned, distinct
  from worktree RECOGNITION/trust (A1, handled by `cwd_is_project_worktree`).
- **D-materialize-before-validate (security order).** `load_policy` canonicalises
  each `extra_rw` (R4-canon: existence + symlink-resolve) BEFORE `validate_policy`,
  so validate judges the paths the leaf actually binds — a symlink cannot resolve
  past the wall after validation. Any non-canonicalising OR unsafe entry ⇒ floor
  the WHOLE policy to Default (binary outcome). Closes the PHASE-02 R4-canon
  boundary obligation with tests (`non_existent_extra_rw_floors_whole_policy`,
  `unsafe_extra_rw_ancestor_floors`).
- **D-loadfloor.** Every resolution failure (absent / malformed / non-materialising
  / validate-reject) folds to Default, never deny — a worker with no valid widening
  is still fully jailed to its worktree (the safe outcome; EX-3).
- **D-serialize.** `Serialize` added to `JailPolicy` — one schema source for
  write (`toml::to_string` in arm-spawn) and read (`from_toml_str` in pretooluse).

### T5 — module dead_code expect KEPT (plan prediction wrong)
The plan/handover predicted P04 would make the jail.rs module
`#![cfg_attr(not(test), expect(dead_code))]` unfulfilled (validate_policy +
from_toml_str were the last not(test)-dead items). Ground truth (clippy): the
SL-183 `Seatbelt` seam is STILL legitimately dead under not(test), so the module
expect stays fulfilled. Removing it surfaces those as `-D warnings`. No change.

### Carry-forward / reconcile debt to /audit
- **MF-3** (unchanged) — plan P02 EX-1 lists `load_policy` in the pure surface;
  it is shell (reads disk). Fix EX-1 text via `/reconcile`.
- **§5.2 PolicyError** typed-enum vs design's `Result<_,String>` — coherence-flag.
- **§5.3 provision-site wording** ("beside run_provision + write_marker" ⇒ implies
  fork.rs) vs as-built create.rs site — coherence-flag (D-provision-site above).
