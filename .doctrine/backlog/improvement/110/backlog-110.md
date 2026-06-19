# IMP-110: TECH spec: dispatch / worktree process — workflow, harness gotchas, decision logic, and tooling from the orchestrator perspective

A durable technical specification of the dispatch/worktree orchestration
process: the workflow, harness-specific behaviour and gotchas, decision logic,
and the CLI tooling surface — not the code internals, but the perspective of an
orchestrator (human or agent) driving a slice through dispatch.

---

## 1. Architecture overview

**Thesis (SL-056):** mechanism belongs in CLI verbs; judgment and harness
concessions belong in prose. The binary is the pure mechanism core; the harness
spawn is the thin impure shell.

**Keystone:** orchestrator-owned fork + disk marker as harness-agnostic worker
identity. Disk is the one identity medium every harness has; an env channel is
not (claude's `Agent` tool has none, and `claude -p` is API-billed +
harness-specific).

**Per-harness capability altitude** (ADR-011), not uniform:

| Layer | Mechanism | Fails | claude | codex/pi |
|---|---|---|---|---|
| CLI — identity | disk marker (orchestrator-stamped) → guard refuses writes | closed | ✓ primary | ✓ |
| CLI — worker-on-main catch | `DOCTRINE_WORKER` env | closed | ✗ no env seam | ✓ |
| OS — confinement | nested bwrap, rw only worktree+target | closed | ✗ no subprocess | ✓ |

### Key ADRs

- **ADR-006:** Worktree posture — policy-agnostic framework, orchestrator-sole-writer
- **ADR-008:** Project-local jail build isolation and worker confinement
- **ADR-011:** Harness-agnostic orchestrator spawn interface and per-harness capability altitude
- **ADR-012:** Dispatch integration topology — isolated coordination worktree, class-routed projection, preserved code branches

### Key slices (dispatch lineage)

| Slice | Title |
|---|---|
| SL-029 | Dispatch worktree creation: detection and creation paths with guards |
| SL-031 | Dispatch orchestrator funnel: worker-mode workers and import-verify-commit-record |
| SL-056 | **Mega-design.** Orchestrator spawn seam: worktree mechanism into CLI verbs — fork, import, gc, marker, land, per-harness spawn templates, privilege classes |
| SL-064 | Coordination-branch isolation: dedicated worktree + integration-sync seam (`dispatch setup`, `sync --prepare-review`, `sync --integrate`, `record-boundary`) |
| SL-068 | Dispatch candidates for safe audit interaction (candidate create/admit/status) |
| SL-084 | Decompose dispatch harness routing: per-harness spawn templates, model selection, agent-def parity for pi/codex/claude |
| SL-085 | Push dispatch drive loop into CLI — shrink skill to thin CLI-calling wrapper (`plan-next`, `status`) |
| SL-108 | pi dispatch worker integration via RPC mode |
| SL-117 | dispatch: preferred worker harness for arm selection (subagent vs subprocess) |

---

## 2. The dispatch funnel (orchestrator workflow)

### Outer loop

1. `dispatch setup --slice <N> --dir <path>` — create/resume coordination worktree. Emits env contract on stdout: `coordination_dir=`, `base=`, `slice=`, `dispatch_ref=`.
2. **Claude arm:** `cd` into the coordination dir and park Bash cwd there for the full drive loop. The Agent tool's `isolation: worktree` forks Bash cwd HEAD — this achieves `base == B` by placement (`mem_019ec65ecbc7`). Step out to session root only for authored writes.
3. `dispatch plan-next --slice <N>` — find next actionable phase(s); parallelize file-disjoint.
4. Route to correct arm per harness detection.
5. Funnel batch → repeat from new HEAD until all phases done.
6. Conclude: `dispatch sync --prepare-review` → remove coord worktree dir (KEEP refs) → audit.

### Per-batch funnel (shared across arms)

Capture `B = git rev-parse HEAD` pre-spawn. After workers return:

1. Precond — worktree/index clean, HEAD == B
2. Delta check — net diff `B..S`, single non-merge commit, `S^ == B`
3. R-5 belt — reject any `.doctrine/` or `.claude/` touch
4. Import — apply surviving net-diffs onto `B`, non-committing
5. Verify — run project verify; isolate RED offender per delta
6. Branch-point guard — coordination HEAD still `B`?
7. Commit — ONE commit on coordination branch
8. Record — knowledge trails the confirmed commit

Report-and-halt on conflict, moved HEAD, or authored-tree touch — never
auto-resolve.

### Import (the core funnel verb)

`doctrine worktree import` applies a worker's single delta `S` onto the
coordination branch:
- `git diff B..S` → pipe to `git apply --3way --index` (non-committing)
- Refuses if `S^ != B` (wrong base — worker forked off wrong commit)
- Refuses if delta touches `.doctrine/` or `.claude/` (R-5 belt)
- **Known bug (ISS-026):** piped diff to `git apply` drops trailing newline →
  `corrupt patch at <stdin>:N`. Workaround: diff to file, apply from file.

### Projection (`dispatch sync`)

- `--prepare-review` (stage-1): creates `review/<slice>` and `phase/<slice>-NN`
  refs from the run ledger, parented on the pinned fork-base (not live trunk
  tip — `mem_019ec66a43ee`). Stage-2 `integrate` is `/close`'s job post-audit.
- `record-boundary`: between funnel steps 7 (code commit) and 8 (knowledge
  commit), writes a boundary row to `boundaries.toml`. Claude-arm-only (no fork
  branch); skipped on codex/pi where the fork branch IS the native phase unit.

---

## 3. Harness-specific arms

### Claude arm (`/dispatch-agent`)

**Spawn:** `Agent` tool with `subagent_type: dispatch-worker`, `isolation:
worktree`. Claude default-creates the worktree; doctrine does not intermediate
creation (`mem_019ebfd16f8e`).

**Base control — cd into coord tree (ISS-029):** The Agent tool's `isolation:
worktree` forks off the **Bash tool's cwd HEAD**, not the session root
(`mem_019ec65ecbc7`, `mem_019ec6142d3b`). With `worktree.baseRef='head'` in
`.claude/settings.local.json`, the orchestrator must `cd` into the coordination
worktree before every spawn — Bash cwd HEAD == B, so the worker forks exactly
the intended base. Park Bash cwd in the coord tree for the full drive loop;
serial dependent phases self-base automatically.

**Identity — SubagentStart stamp (best-effort, fail-open):** A matcher-scoped,
sync-blocking `SubagentStart` hook runs `doctrine worktree marker
--stamp-subagent` (reads payload `{cwd, agent_type, …}` on stdin). The hook:
- Parses stdin JSON (trust boundary — harness-facing untrusted input)
- Provisions the worktree (ADR-006 D9 allowlist)
- Writes the worker marker
- **Not fail-closable (ADR-011 D6):** SubagentStart is read-only — non-zero exit
  only surfaces stderr; the subagent runs regardless. On success the marker +
  provisioning are present before the worker's first action (blocking ⇒ no
  write race). On failure the worker proceeds unstamped — fenced by the layered
  funnel (`import` belt + `verify-worker` post-spawn check), not the hook.

**Worker marker stamp — known issues:**
- The `SubagentStart` hook's merge keys identity on command only — a stale
  matcher never heals on reinstall (ISS-011)
- The hook silently no-fires for nested-session dispatch workers
  (`mem_019ec84b9740`)
- WorktreeCreate hook payload carries no type, no target path, no base — can't
  use it for fail-closed creation (`mem_019ec093bd7b`)

**Verify-worker (post-spawn belt):** After the worker returns, before import:
`doctrine worktree verify-worker --base <B> --dir <worktree>`. Checks:
- HEAD resolves?
- Marker present?
- `merge-base --is-ancestor <B> HEAD` (wrong-base verdict)

**Boundary recording:** Claude-arm-only — between code commit and knowledge
commit, `dispatch record-boundary` appends a phase boundary row. Stage-1
`prepare-review` tree-reads this to cut `phase/<slice>-NN` refs.

### Codex/pi arm (`/dispatch-subprocess`)

**Spawn:** `doctrine worktree fork --worker --base <B> --branch <name> --dir
<path>`, then spawn subprocess with cwd bound to fork:
- **codex:** `env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "<prompt>"`
- **pi (RPC mode):** fifo-based RPC spawn with `timeout 300`, `--approve`,
  `--offline`, `--no-extensions --no-skills --no-themes`
  (`mem_019ed44caf55`)

**`fork --worker` steps (compensating rollback):**
1. `git worktree add -b <branch> <dir> <B>`
2. `doctrine worktree provision <dir>` (sole copier; withheld tier excluded)
3. Write marker before any spawn window
4. Emit per-worktree env contract on stdout

Failure after step 1 triggers best-effort rollback (`worktree remove --force`,
`branch -D`, dir reap); a rollback that fails names the leftover.

**Subprocess advantages (codex/pi only):**
- `DOCTRINE_WORKER=1` env-arm catches worker-on-main
- Per-worktree env provisioning (`CARGO_TARGET_DIR`)
- Nested bwrap confinement (ADR-008 D-B3)

### Harness routing

| Detected harness | Arm | Spawn mechanism |
|---|---|---|
| codex / pi (subprocess-capable) | `/dispatch-subprocess` | `worktree fork --worker` + `env -C "$D"` / bwrap `--chdir` |
| claude (`Agent` tool, `isolation: worktree`) | `/dispatch-agent` | `Agent` `subagent_type: dispatch-worker` + SubagentStart stamp |

Route only on self-belief ↔ env-marker agreement. Mismatch/unknown → refuse
naming the cause. Config override: `doctrine.toml` → `[dispatch]` →
`claude-force-subprocess-dispatch` (SL-117).

---

## 4. Worker identity & privilege classes

### Disk marker (primary, harness-agnostic)

```
marker path: <root>/.doctrine/state/dispatch/worker
worker_mode(root) := (is_linked_worktree(root) && marker_present(root))  // PRIMARY
                     OR env DOCTRINE_WORKER set                          // codex/pi catch
```

- Marker present ⇒ writes refused. Presence-only, no contents.
- `DOCTRINE_WORKER=1` is a codex/pi optimisation, not the identity — its one
  job is catching worker-on-main.
- Solo `/execute` sets neither signal → writes freely.
- `doctrine worktree status` prints the resolved mode and its cause
  (`worker fork: yes/no; signal: env|marker|both`).
- `marker --clear` removes a stray marker; refused if env-set, cwd-not-tree-root,
  or (in linked worktree) without `--operator`.

### Privilege classes (SL-056 §5)

| Class | Members | Refused under `worker_mode`? |
|---|---|---|
| Orchestrator | `fork`, `import`, `land`, `gc` | Yes |
| Hook-mint | `marker --stamp-subagent` | Yes — but legit first-stamp exempt by floor (no marker yet ⇒ worker_mode false) |
| Write | authoring writes + `claude install` | Yes |
| Read | `provision`, `check-allowlist`, `branch-point-check`, `status` | No |
| `marker --clear` | bespoke | No — locking the only remover behind the marker is self-brick |

### Worker contract

- Mutate SOURCE only — no `.doctrine/` authored trees, runtime state, or memory
- Commit exactly ONE non-merge commit on top of B
- Self-arm: `export DOCTRINE_WORKER=1` (codex/pi; fail-open prompt contract)
- Return structured report

---

## 5. Coordination worktree & sync

### Setup (`dispatch setup`)

Creates or resumes `dispatch/<slice>` in its own worktree off the resolved
trunk. Markerless (the orchestrator IS the coordination tree). Create-or-resume:
a live worktree on the same branch is refused (`coordination-live`); a
branch with no live worktree resumes (reattach — resume-stable for handover).

Regenerates runtime phase sheets from committed `plan.toml`.

### Sync (`dispatch sync`)

Two-stage lifecycle:
- **Stage-1 (`--prepare-review`):** materialises `review/<slice>` and
  `phase/<slice>-NN` refs from the run ledger, parented on the pinned
  fork-base (`git merge-base(refs/heads/dispatch/<slice>, trunk)`), NOT
  the live trunk tip (`mem_019ec66a43ee`). Runs under CAS journal; never
  writes trunk.
- **Stage-2 (`--integrate`):** pushes code onto trunk via ff-only merge.
  This is `/close`'s job, post-audit — never land code pre-audit.

### Projection parents on pinned fork-base (not live trunk tip)

A foreign commit landing on trunk between `worktree coordinate` and `dispatch
sync` silently reparents the projection onto the moved tip: per-phase diffs
stop being exact, and `integrate --trunk`'s non-ff safety net is bypassed.
Fix: project off the merge-base; keep `trunk_commit()` (live tip) only at
integrate's actual trunk push under CAS (`mem_019ec66a43ee`).

### Stage-1 integration known issues

- `dispatch sync --integrate` left staging area in stale reverse-diff state
  after advancing trunk (ISS-022)
- `dispatch sync --integrate` is silent about its trunk/worktree outcome
  (IMP-078)
- `--integrate --help` needs to clarify `--trunk` dry-run semantics
  (IMP-103)
- **ISS-030:** `dispatch sync --integrate` leaves stale worktree when run
  from the trunk branch — ref advances, index + worktree stay at
  pre-integration state, creating a phantom reverse-diff. Close step 3a
  verify reads ref not tree, so the stale tree isn't caught. Recovery:
  `git restore --source=HEAD --staged --worktree -- src/` (not `reset
  --hard` — preserves in-flight work). Same family as ISS-029 (git ref vs
  working-tree placement).

---

## 6. Known gotchas & sharp edges

### Claude arm base control

- **ISS-029 (FIXED):** The `isolation: worktree` forks Bash cwd HEAD, not
  session root. The skill never instructed `cd` into the coord tree before
  spawn → workers forked off `main` instead of B. Fixed by adding
  pre-spawn cd instruction to `dispatch-agent/SKILL.md`.
- `baseRef: "head"` in `.claude/settings.local.json` is honoured against
  Bash cwd HEAD — base is controllable by placement, not ref-redirect
  (`mem_019ec6142d3b`).

### Worker marker

- **ISS-028:** Worker-marker confinement refuses `.doctrine/` CLI writes inside
  a stamped fork — tests that shell the CLI fail. Not regressions; trust
  post-import coordination verify, not fork test results.
- **ISS-011:** SubagentStart hook merge keys identity on command only — stale
  matcher never heals on reinstall (fail-open unstamped worker).
- SubagentStart stamp hook silently no-fires for nested-session dispatch
  workers (`mem_019ec84b9740`).
- WorktreeCreate hook can't fix anything — payload carries no `agent_type`,
  no target path, no base (`mem_019ec093bd7b`).
- **IMP-052:** Post-spawn marker check aborts an unstamped worker fork (enforced
  where the harness *can* abort).
- **IMP-065:** Positive coordination-tree marker (close OQ-D) — replace
  marker-absence dependence in D2a.

### Import & git

- **ISS-026:** `worktree import` pipes diff to `git apply` → drops trailing
  newline → `corrupt patch at <stdin>:N`. Workaround: diff to file, apply from
  file.
- **IMP-043:** Import re-anchor (`--allow-reanchor`) — 3-way onto moved
  coordination HEAD with computable path-disjointness. Deferred.
- Under rtk (git proxy), `git diff` is stat-proxied — use `git checkout` to
  import (`mem_019ebf75e27a`).
- `git cat-file -e` exit code can false-positive under rtk hook; use `ls-tree`
  for existence (`mem_019ec4402638`).
- rtk masks git plumbing — funnel re-anchor proofs must bypass via rtk proxy
  git; checkout-import unsafe under real overlap (`mem_019ec6431b4d`).

### Testing

- Removing a dispatch worktree leaves `env!(CARGO_MANIFEST_DIR)`-baked test
  binaries pointing at the deleted fork path — false RED until recompiled
  (`mem_019ebc8e46`).
- Worker verify gate: run with `DOCTRINE_WORKER` unset when tests mint
  entities (`mem_019eba28977b`).
- Dispatch verify shared-target false-green: touch + re-run to confirm a
  fresh compile (`mem_019ebb7a2599`).
- `candidate create`: stray `.doctrine/slice/` dirs in worktree break
  corpus-scanner tests (ISS-024).

### Review & audit

- RV review verbs refuse on a worktree fork — drive the audit from the parent
  tree or merge-first (`mem_019eb7415390`).
- Data-only phase mutating corpus contents must re-run corpus-walk oracles /
  full gate (`mem_019ec47bbd4e`).

### Cleanup

- GC squash-merge is indistinguishable from a never-landed fork
  (`mem_019ec166d8bf`).
- Fork landed-oracle: `--merged`, delta-emptiness AND the import receipt all
  unsound; use `git patch-id` check (`git cherry`) over all B..fork commits
  (`mem_019ebed87aca`).

---

## 7. Key design decisions

### Option C — positive-signal worker-mode floor (SL-056 PHASE-05)

`worker_mode = (is_linked_worktree && marker_present) OR env DOCTRINE_WORKER`.
**Marker present ⇒ refuse; marker absent ⇒ allow.** The fail-closed
(marker-absent ⇒ refuse) floor was reversed: P(SubagentStart hook failure) ≈ 0
(it blocks; a miss needs a crash) × jail-bounded harm ⇒ the security delta is
negligible. The legit first-stamp is allowed automatically (no marker yet ⇒
worker_mode false) — no verb-identity carve-out needed.

### Claude arm: SubagentStart stamp (not WorktreeCreate)

The WorktreeCreate `create-fork` path was the original design but is **not
buildable** — payload carries no `agent_type`, no target path, no base
(`mem_019ec093bd7b`). Deferred until payload grows type+path or IDE-004 env
channel lands. Live mechanism is SubagentStart-stamp (best-effort, fail-open).
WorktreeCreate hook for pre-worker fail-closability → IMP-072 (deferred).

### Per-harness capability altitude (not uniform)

The spawn backend is a harness concession, not the keystone. Three enhancements
ride the subprocess seam (codex/pi-only): env-arm `DOCTRINE_WORKER`,
per-worktree env provisioning, nested bwrap. The CLI identity rung (disk
marker) is the agnostic floor every harness reaches.

---

## 8. Reference index

### Memories

| UID | Key | Summary |
|---|---|---|
| `mem_019eb7263a90` | fork-rung3-base-not-session-head | Worker must fork from explicit B, never session HEAD (HIGH trust, HIGH severity) |
| `mem_019ec65ecbc7` | agent-worktree-forks-bash-cwd-head | claude `isolation: worktree` forks Bash cwd HEAD; cd into coord tree ⇒ base == B |
| `mem_019ec6142d3b` | claude-isolation-worktree-forks-orchestrator-session-head | `baseRef=head` honoured; base controllable by placement |
| `mem_019ebfd16f8e` | claude-agent-worktree-not-fork-provisioned | Claude worktree is harness-born, not fork-provisioned |
| `mem_019ec093bd7b` | claude-worktreecreate-payload-minimal | WorktreeCreate payload has no type/path/base |
| `mem_019ebfb61ba8` | claude-subagentstart-worker-identity | SubagentStart is the usable worker-identity seam |
| `mem_019ebeeda9c2` | spawn-backend-harness-agnostic | Worker identity cannot rely on free env seam (HIGH trust, HIGH severity) |
| `mem_019ec66a43ee` | project-off-pinned-fork-base-not-live-trunk-tip | Sync projection must use merge-base, not live trunk tip |
| `mem_019eba28977b` | worker-verify-unset-doctrine-worker | Run verify with DOCTRINE_WORKER unset for tests that mint entities |
| `mem_019ebb7a25ad` | three-way-import-onto-moved-shared-main | Import: 3-way net-diff, stage-only-delta, commit without -a |
| `mem_019ebb430f96` | reanchor-base-on-disjoint-head-move | Funnel re-anchors B to moved coordination HEAD on disjointness proof |
| `mem_019ed44caf55` | pi-subagent-cwd-binding | pi subagent cwd binding works for dispatch worker forks |
| `mem_019ec4a71f0f` | claude-dispatch-agent-worker-commit-integrates | Claude worker commit integrates onto parent branch |
| `mem_019ec84b9740` | subagentstart-stamp-hook-silently-no-fires | Hook no-fires for nested-session workers |
| `mem_019ebc8e46` | removing-worktree-leaves-stale-binaries | Removing worktree leaves stale env-baked test binaries |
| `mem_019eb7415390` | rv-review-verbs-refuse-on-worktree-fork | RV verbs refuse on worktree fork |
| `mem_019ed624cc9c` | worktree-coordinate-fails-find-plan | coordinate fails to find committed plan.toml on provisioned worktree |
| `mem_019ec166d8bf` | gc-squash-merge-indistinguishable | GC squash-merge is indistinguishable from never-landed fork |
| `mem_019ebed87aca` | fork-landed-oracle | Use git cherry (patch-id) over all B..fork commits |

### Backlog items

| ID | Kind | Summary |
|---|---|---|
| ISS-011 | issue | SubagentStart hook merge keys identity on command only — stale matcher never heals |
| ISS-022 | issue | dispatch sync --integrate left staging area in stale reverse-diff state |
| ISS-024 | issue | candidate create: stray .doctrine/slice/ dirs break corpus-scanner tests |
| ISS-026 | issue | worktree import: piped diff drops trailing newline → corrupt patch |
| ISS-028 | issue | worker-marker confinement refuses CLI writes in fork, breaking tests |
| ISS-029 | issue | (FIXED) missing cd-into-coord-tree instruction — workers forked off main (after ISS-030) |
| ISS-030 | issue | dispatch sync --integrate leaves stale worktree; close verify reads ref not tree |
| IMP-004 | improvement | Jail dispatch isolation spike: per-worktree target and bwrap confinement |
| IMP-043 | improvement | import verb: moved-HEAD re-anchor (--allow-reanchor) |
| IMP-052 | improvement | orchestrator post-spawn marker check: abort unstamped worker fork |
| IMP-065 | improvement | positive coordination-tree marker (close OQ-D) |
| IMP-072 | improvement | WorktreeCreate hook for pre-worker fail-closability (deferred) |
| IMP-078 | improvement | dispatch sync --integrate is silent about trunk/worktree outcome |
| IMP-101 | improvement | dispatch: deliver_to config field in doctrine.toml [dispatch] section |
| IMP-103 | improvement | sync --integrate --help: clarify --trunk dry-run semantics |
| RISK-008 | risk | Closure should gate on live Failed coverage cell (forget evidence-erasure) |

### Slices (full dispatch lineage)

SL-029, SL-031, SL-056, SL-064, SL-068, SL-084, SL-085, SL-108, SL-117.

### ADRs

ADR-006, ADR-008, ADR-011, ADR-012.
