# Design SL-152: Claude-arm WorktreeCreate worker creation

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The claude `/dispatch` arm spawns workers via the `Agent` tool with
`isolation:"worktree"`. Today the **harness** creates that worktree, off the
Bash-tool cwd HEAD (with `worktree.baseRef="head"`). Under shared-clone git-lock
contention the spawn silently **falls back to the main worktree**, where
`baseRef:"head"` tracks a moving `main` — hazard **H1** (RFC-005, ISS-034): the
worker runs on a wrong / dirty / moving base instead of the coordination tip B.
SL-123 added a loud **post-run** `verify-worker` belt (pre-import halt), but the
race itself is unchanged: under churn the wasted-worker-run residual remains and
throughput is zero.

A `WorktreeCreate` hook **fully replaces** native worktree creation. When
doctrine is the creator there is **no native creation left to fall back to** —
H1 cannot occur — and a hook failure aborts the spawn **fail-closed**. This
converts H1 from a permanent harness tax into a fixable mechanism defect, and
lets the claude arm create workers through the **same `doctrine worktree fork
--worker`** path the subprocess arm already uses (converging hazard **H3**).

This design builds the hook to **test** the collapse-the-arms hypothesis: with
doctrine as the worktree creator, both arms run one byte-identical
`fork --worker` core; they differ only in *how its arguments arrive*.

## 2. Current State

The claude arm is a six-step dance; most of SL-152's machinery already exists.

| Seam | Location | Role today |
|---|---|---|
| `worktree fork --worker` | `src/worktree/fork.rs:133` | add + provision + mark at explicit `--base/--branch/--dir`. **The subprocess arm's path.** |
| `run_provision` | `src/worktree/provision.rs` | sole copier; ADR-006 D9 gitignored allowlist |
| `write_marker` | `src/worktree/marker.rs` | stamps `.doctrine/state/dispatch/worker` (presence-only, withheld tier) |
| SubagentStart stamp | `src/worktree/subagent.rs:162` (`run_stamp_subagent`) | provisions + marks the **harness-created** worktree; `already-marked` re-entrant refusal (`classify_stamp`, `:99`). **Retired on the claude arm by this slice (D2).** |
| `verify-worker` | `src/worktree/subagent.rs:343` | post-spawn base==B belt (`no-worker-head`/`not-isolated`/`unstamped`/`wrong-base`/`branch-mismatch`). **Kept.** |
| install / hook emission | `src/boot.rs` (`install_claude_hook` `:1552`, `HookSpec` `:938`, `install_refresh` `:1197`) | merges hooks into `.claude/settings.local.json`; sets `worktree.baseRef="head"` |
| `dispatch setup` | `src/dispatch.rs:407` | emits `base=<tip>`, `coordination_dir=` |
| dispatch-agent skill | `.agents/skills/dispatch-agent/SKILL.md` | cd into coord tree (base==B by placement); reads `worktreePath:`/`worktreeBranch:` footer; runs `verify-worker` |
| plugin stub | `plugins/doctrine/.claude-plugin/plugin.json` | bare name/version/description; **no hooks yet** |
| dropped verb | `src/boot.rs:1073` comment | `create-fork` was designed then dropped ("payload carries no agent_type/path"). **Revived here.** |

Dispatch topology: **one coordination worktree per slice** (`dispatch setup
--slice N --dir`); parallel file-disjoint phases run from that **same** coord
tree (shared cwd).

### Empirical knowns (claude-code 2.1.181, pinned)

From the WorktreeCreate probe ([[mem.pattern.dispatch.worktreecreate-replace-base-control]],
high trust) and `docs/claude/hooks.md` + `plugins-reference.md`:

- **Fires for every `isolation:"worktree"` spawn**, including a *named* subagent
  (`dispatch-worker`). (hooks.md:52)
- **No matcher support** — repo-global; a `matcher` field is silently ignored.
  The probe confirmed the hook fired for `general-purpose` too. Cannot scope by
  agent_type. (hooks.md:237)
- **Replaces native creation entirely.** (hooks.md:2390)
- **`.worktreeinclude` is NOT processed** under the hook — the hook must copy any
  local files itself. (hooks.md:2392)
- **Must print the absolute worktree path on stdout**; that path becomes the
  isolated session's cwd. (hooks.md:2394, 2435)
- **Any non-zero exit aborts creation, fail-closed** — the *only* hook event
  where any non-zero exit blocks. (hooks.md:644, 679)
- **Payload** = `{session_id, transcript_path, cwd, hook_event_name, name}`.
  `name` is a unique slug (`agent-<hex>` for tool spawns, or `bold-oak-a3f2`).
  **No `agent_type`, no base, no target path.** (hooks.md:2419)
- **WorktreeRemove**: for *git* worktrees Claude auto-cleans via
  `git worktree remove`; it leaves the branch behind. A custom remove hook is
  only needed for non-git VCS. (hooks.md:2442)
- **Plugin hooks** live in `hooks/hooks.json` (or inline in plugin.json), use
  `${CLAUDE_PLUGIN_ROOT}` for script paths, and need `/reload-plugins` (or
  restart) to hot-load. (plugins-reference.md:87, 394)
- Probe result: a named `dispatch-worker` spawned with the hook active landed at
  doctrine's chosen base (`68250bcd`) in doctrine's path, **overriding
  `worktree.baseRef="head"`**.

## 3. Forces & Constraints

- **Governance.** ADR-006 (orchestrator-sole-writer worktree posture; D9
  gitignored-allowlist provisioning), ADR-011 (harness-agnostic spawn interface;
  D7 σ repo-global blast-radius is the named cost), ADR-012 (dispatch
  integration topology). Supersedes the placement-only decision in SL-064 §8
  (option Y). Origin: IMP-072 (re-scoped — its "base control solved by
  placement" premise was falsified by contention + the probe).
- **Pure/imperative split** (CLAUDE.md, slices-spec § Architecture): no git /
  disk / env / clock in the pure layer. The base B is an **input**, never
  derived in a pure layer.
- **Behaviour-preservation gate**: the subprocess arm and existing dispatch
  suites must stay green unchanged — `fork --worker` is shared machinery.
- **Repo-global blast radius** (RSK-1): every `isolation:"worktree"` subagent in
  any repo that installs doctrine now routes through the hook. The benign
  pass-through must be robust or it breaks unrelated subagent use.
- **The hook is a write (ADR-006 fidelity, F6).** `create-fork` creates +
  provisions + marks. It is a **trusted synchronous extension of the sole
  writer** — orchestrator-initiated, same standing as the subprocess `fork`
  invocation — not a second writer. It only acts when the orchestrator has armed
  a spawn (D4).
- **Jail**: worker worktrees must live inside the project root
  (bubblewrap-confined `/workspace/<repo>`).
- **DRY / no parallel implementation**: the hook *shells the existing*
  `fork --worker`; it does not reimplement creation, provisioning, or marking.

## 4. Guiding Principles

- **Collapse the arms literally.** Success = both arms call one byte-identical
  `fork --worker` core. They differ only in how its arguments arrive: subprocess
  passes argv; the claude hook reads base from the arming dir's `base` file +
  derives the rest from the payload (the harness owns the spawn, so doctrine
  cannot pass argv).
- **Explicit intent over placement inference.** The orchestrator *arms* a spawn
  (cd's into the dedicated arming dir); the hook does not infer "this is a
  dispatch worker" from coord-tree placement alone — it requires the orchestrator
  to have stepped into the arming dir. Move away from the fragile
  placement-implicit base.
- **Base is explicit, never inferred.** The `base` file carries B; the hook does
  not read it from cwd HEAD. cwd is the *discriminator*, not the base source.
- **Fail-closed on the dispatch path; robust pass-through off it.**

## 5. Proposed Design

### 5.1 System Model

```
ORCHESTRATOR (claude session, sole writer; default cwd = coord tree root)
  │  dispatch arm-spawn --base B    ── mkdir <coord>/.doctrine/state/dispatch/spawn/,
  │                                    write base file there, then cd INTO it (§5.3)
  │  <Agent spawn, isolation:worktree>  ── harness fires WorktreeCreate synchronously
  ▼                                       (payload cwd = the spawn dir — P3)
WorktreeCreate HOOK  →  doctrine worktree create-fork           (the new verb, §5.2)
  │  read payload {cwd, name}
  │  resolve coord-tree root = git -C <cwd> --show-toplevel
  │  is <cwd> the arming convention  <root>/.doctrine/state/dispatch/spawn ?
  │   ├─ YES (dispatch): base=<spawn>/base; branch=dispatch/<name>; dir=<root>/.worktrees/<name>
  │   │        → fork --worker --base B --branch ... --dir ...   (IDENTICAL to subprocess arm)
  │   │        → print dir on stdout ; any failure → non-zero exit (fail-closed)
  │   └─ NO  (benign, cwd = coord root or anywhere else):
  │            git worktree add <root>/.worktrees/<name> HEAD + provision (I2) → print dir
  │  (disarm = orchestrator cd's back to coord root; self-clearing, §5.4)
  ▼
WORKER subagent runs in the created tree (base-guard, work, return footer)
  ▼
ORCHESTRATOR post-spawn:
  read worktreePath: from footer (normative; I3)
  name = basename(worktreePath); branch = dispatch/<name>   (derived, not footer-read)
  doctrine worktree verify-worker --base B --dir <worktreePath> --branch <branch>
```

The SL-123 `SubagentStart` stamp hook is **no longer wired on the claude arm**
(D2): `fork --worker` already provisioned + marked atomically inside
WorktreeCreate.

### 5.2 Interfaces & Contracts

Two thin verbs, both shells over existing pure logic:

**`doctrine dispatch arm-spawn --base <B> [--slice <N>]`** (orchestrator side)
- Creates the arming dir `<coord>/.doctrine/state/dispatch/spawn/` and writes the
  base file `<spawn>/base` (`<sha>`, one line). Prints the dir so the orchestrator
  `cd`s into it before the Agent spawn(s) — **the cwd, not the file's existence,
  is the discriminator** (§5.3). Idempotent; rewrites `base` when B advances.
- Sole-writer; serial within the orchestrator's own funnel.
- **disarm = `cd` back to the coord root** (self-clearing; §5.4). An optional
  `dispatch disarm` may `rm` the dir, but it is **not** load-bearing: a lingering
  dir cannot misfire because the trigger is cwd-position, not file-presence.

**`doctrine worktree create-fork`** (hook side; reads stdin payload)
- Gather → pure-classify → act, mirroring `run_stamp_subagent`:
  1. read stdin → `{cwd, name}` (malformed ⇒ empty ⇒ benign/refuse, never panic);
  2. resolve coord-tree root from `cwd` (`--show-toplevel`, canonicalised); test
     whether `cwd` **is** the arming dir `<root>/.doctrine/state/dispatch/spawn`
     (both sides realpath'd — symlink-safe); if so read `<cwd>/base`;
  3. **pure classifier** `classify_create(...)` → `Fork{base}` | `Passthrough`
     (+ named refusals) — `Fork` iff cwd is the arming dir **and** `base` parses;
  4. act: `Fork` → `run_fork(base, branch=dispatch/<name>,
     dir=<root>/.worktrees/<name>, worker=true)`; `Passthrough` →
     `git worktree add <root>/.worktrees/<name> HEAD` **then provision via the
     same copier `run_fork` uses** (replicate `.worktreeinclude` — I2; hooks
     bypass the harness's native `.worktreeinclude`, so the hook must restore
     fidelity itself);
  5. **print the created absolute path — and ONLY that — on stdout** (the harness
     path protocol; all diagnostics to stderr). NB: `run_fork` emits the
     per-worktree env contract (`CARGO_TARGET_DIR=…`) on *its* stdout; that would
     corrupt the path protocol, so `create-fork` calls the add+provision+mark
     CORE without the env-contract emission and prints the path itself (D11);
  6. **any failure ⇒ non-zero exit** (fail-closed creation); the benign
     pass-through compensates (removes the tree it added) before exiting so a
     fail-closed abort does not leak a half-created worktree.
- **Root / provision-source resolution (F2/I5 — locked contract).**
  `create-fork` **always** resolves `root = git -C <payload.cwd> --show-toplevel`
  and passes it **explicitly** into `run_fork` / `run_provision` — it never
  relies on process cwd (P3 proves *payload* cwd, not *process* cwd). This makes
  the provision source identical to the subprocess arm's; a divergent root would
  silently break the "byte-identical core" claim (provisioning copies from the
  wrong tree — the ISS-011 trap). Discharged by an e2e proving provisioned files
  come from the coord tree, not the fresh fork (F3, §10 pre-plan). **This resolution
  deliberately DIFFERS from `run_stamp_subagent`**, which uses
  `git::primary_worktree(<cwd>)` because the *stamp* fires INSIDE the fork (process
  cwd == fork). `create-fork` fires in the PARENT (the arming dir, before the fork
  exists), so `--show-toplevel` of the payload cwd is the coord tree — the correct
  source. Mirror the stamp's gather→classify→act *shape*, not its root resolution.
- Branch / dir derived from a **sanitised** payload `name` (I4): a canonical
  validator maps `name` → a ref- and path-safe slug and **rejects** anything
  outside the envelope (empty, whitespace, `/`, `..`, ref-invalid chars, or a
  name colliding with a live `dispatch/<name>` / `.worktrees/<name>` in the coord
  tree). Inside the envelope, `name` gives the per-spawn uniqueness git requires
  (§5.5) without the orchestrator pre-choosing branch/dir.

`fork --worker`'s **creation semantics are unchanged** — `create-fork` is a new
caller of the same add+provision+mark core, exactly as the subprocess arm calls
it. The only refactor permitted is separating that core from the CLI's stdout
env-contract emission so `create-fork` can print the path alone (D11); the
creation behaviour and the subprocess arm stay byte-identical (behaviour-
preservation gate).

### 5.3 Data, State & Ownership

**The arming signal is the orchestrator's cwd, not a file.** Discrimination is
**positional**: a spawn is a dispatch worker iff the payload `cwd` *is* the
arming dir `<coord-tree>/.doctrine/state/dispatch/spawn/`. The orchestrator's
default cwd is the coord root; it `cd`s into the spawn dir only to issue worker
spawns. This is why I1 (the false-positive window) closes — see §5.5.

**The arming dir + base file.** `<coord-tree>/.doctrine/state/dispatch/spawn/`,
containing `base` (`<sha>`, one line — the **only** thing it must carry; no
encoded branch/dir/correlation, no base64).
- **Per-coord-tree**, in the coord tree's own runtime state — gitignored (runtime
  tier) and in the ADR-006 D9 **withheld** list, so never copied into a worker
  fork. Cross-slice / parallel-across-slices partition for free (different coord
  trees ⇒ different dirs).
- The `base` file carries only the **shared `--base B`**. File-disjoint parallel
  phases share B, so one value serves a whole batch — **no consume, no rename, no
  serialization, no per-spawn correlation key.** A parallel batch is N concurrent
  *reads* of one stable value from one cwd.
- The file's **presence is not the trigger** (cwd-position is), so a lingering
  `base` from a prior phase is inert — it can only ever be read by a spawn the
  orchestrator deliberately issued from inside the dir.
- Owner: the orchestrator (sole writer). The hook only **reads** it.

**Why base shared but branch/dir not** (the crux):

| argv | parallel batch | source |
|---|---|---|
| `--base B` | **shared** (all fork the coord tip) | the `base` file in the arming dir |
| `--branch` | **distinct** (git: one branch per worktree) | hook-derived `dispatch/<name>` |
| `--dir` | **distinct** (one path per worktree) | hook-derived `<root>/.worktrees/<name>` |

The thing we could not get pre-spawn (`name`) is exactly the thing the hook
derives uniqueness from, locally. The orchestrator learns each worker's location
from the **return footer**, which survives hook-creation (P2 PASS). **The
normative datum is `worktreePath`** (proven present); the orchestrator derives
`name = basename(worktreePath)` and `branch = dispatch/<name>` from it (I3).
`worktreeBranch` is **not** relied on (the probe saw it `undefined` for a
detached tree); the dispatch-agent post-spawn contract is updated to bind
`verify-worker`/funnel to the derived branch, not the footer field.

### 5.4 Lifecycle, Operations & Dynamics

- **Serial drive (default).** arm-spawn at B (write `base`, cd into spawn dir) →
  spawn → hook forks at B → orchestrator cd's back to coord root (disarm) → funnel
  commit advances coord HEAD → arm-spawn at B′ → next spawn. `base` is rewritten
  each phase; base is explicit so coord-HEAD drift between arm and create is
  irrelevant.
- **Parallel batch (file-disjoint).** arm-spawn at B once, cd into spawn dir →
  issue N spawns (all carry the spawn-dir cwd) → each hook reads the same B,
  derives its own branch/dir from its own `name` → cd back. No serialization of
  *runs*; creation is naturally independent (distinct branch/dir per `name`).
- **Benign isolated subagent (any repo).** Spawned from the orchestrator's normal
  cwd (coord root, main, anywhere ≠ the arming dir) ⇒ cwd-position test fails ⇒
  pass-through ⇒ a working worktree provisioned via the same copier (I2), **not**
  stamped/forked as a worker.
- **Cleanup.** Native `git worktree remove` handles teardown (real git
  worktree); the leftover branch is the funnel's import S (benign). No
  WorktreeRemove hook needed (D10).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1.** On the dispatch path the created worker forks **exactly B** (the
  `base` file value), or the spawn aborts. No silent fallback.
- **INV-2.** A benign subagent is never stamped/forked as a worker — held
  *positionally*: a benign spawn's cwd is not the arming dir (§5.3).
- **INV-3.** `fork --worker` semantics are identical on both arms (shared fn).
- **ASM-1.** Payload `cwd` is the orchestrator's cwd at spawn time — **CONFIRMED,
  probe P3 (§10)**; discrimination is positional via it.
- **ASM-2.** The return footer survives hook-creation — **CONFIRMED, probe P2
  (§10)**; `worktreePath` is the normative datum (I3).
- **Edge — discrimination false-positive (I1, resolved).** The trigger is the
  orchestrator's cwd being the arming dir, not a file existing. A benign
  `isolation:worktree` spawn issued from the orchestrator's normal cwd (coord
  root) → cwd-position test fails → passthrough. The only residual is a benign
  spawn issued *while cwd is still the arming dir* — a narrow, self-clearing state
  (cd-out is the natural next step), versus the old persistent-file window (F4)
  which lingered for the coord-tree lifetime. `verify-worker` remains the
  post-spawn backstop. The payload carries no class tag, so this residual is the
  mechanical floor; positional arming sits at that floor.
- **Edge — malformed/empty payload.** Fold to a named refusal; fail-closed;
  never panic on hook input (mirrors `run_stamp_subagent`).
- **Edge — pass-through fail.** A non-zero exit aborts a *benign* subagent's
  creation (RSK-1). The pass-through must be robust and provision via the same
  copier (I2); `.worktreeinclude` parity is required, not deferred.
- **Edge — F4 (persistent-marker window) — DISSOLVED.** Superseded by positional
  arming: there is no persistent on-disk discriminator, so no lingering
  false-positive window. A leftover `base` file is inert (§5.3).
- **Edge — re-dispatch leak (F5).** A retried worker gets a fresh `name` ⇒ new
  `dispatch/<name>` branch + `.worktrees/<name>` dir; native remove drops the
  tree but leaves the branch (D10), so branches accumulate across retries. Prune
  in the WorktreeRemove follow-up or a `dispatch gc`.
- **Edge — `name` shape (I4).** Observed `agent-<hex>` (harness auto-generated
  for Agent-tool spawns). Not *assumed* safe: the harness owns the field and docs
  admit human-style names, so `create-fork` runs a canonical sanitiser/validator
  (§5.2) and fail-closed rejects anything outside the ref+path-safe envelope
  rather than feeding it raw into `dispatch/<name>` / `.worktrees/<name>`.

## 6. Open Questions & Unknowns

Probes (need a live 2.1.181 dispatch; likely a you-run-it step). The design is
sequenced so these resolve **before** the dependent schema/emission locks.
**Run order: P3 → P2 → P1.**

- **P3 — payload cwd identity (run FIRST; foundational, F1). RESOLVED ✓ (PASS,
  §10).** Payload `cwd` is the orchestrator's parked Bash cwd (the coord tree),
  not the session root — `cd` shifts it and the harness persists it across tool
  calls. Positional arming (§5.3, D3) keys off it; D3 stands.
- **P2 — footer survival. RESOLVED ✓ (PRESENT, §10).** The harness emits
  `worktreePath:` in the Agent return footer even when the hook created the
  worktree. D8 **primary** selected (orchestrator reads the worker location from
  the footer, derives branch from it (I3); no serialization).
- **P1 (was OQ-5) — plugin-hook parity.** Does a `WorktreeCreate` hook in plugin
  `hooks/hooks.json` fire identically to the settings-block form? *Gates only
  the secondary plugin step.* Expected yes; verify before relying.
- **OQ-2 — CLOSED (I2).** Pass-through replicates `.worktreeinclude` through the
  same provision copier (D9). Accept-the-gap rejected — the allowlist is
  non-empty in doctrine's own repo.

## 7. Decisions, Rationale & Alternatives

- **D1 — WorktreeCreate hook replaces native creation; revive `worktree
  create-fork` as the hook target.** Rationale: only a replacing creator removes
  the H1 fallback; the dropped verb's only blocker (thin payload) was a wrong
  turn — the hook *sets* base/branch/dir, it does not need them in the payload.

- **D2 — Retire the SubagentStart stamp hook on the claude arm (R1).** Once
  `fork --worker` does add+provision+mark atomically inside WorktreeCreate, the
  stamp is redundant (it would hit `already-marked` every dispatch). Rationale:
  it is the only option that *literally* collapses the arms (H3) onto one seam.
  The "keep it in modified form to carry a correlation id" idea is **dead on
  ordering** — `SubagentStart` fires *after* `WorktreeCreate`, so it can never
  feed data forward to base selection. Backstop on a missed mark stays
  `verify-worker` (`unstamped`/`wrong-base`, pre-import).
  - *Alt R2 (keep as no-op belt):* rejected — makes `already-marked` the normal
    path, muddies diagnostics, no collapse.
  - *Alt R3 (split: hook controls base only, stamp keeps provision+mark):*
    rejected — breaks the collapse hypothesis (arms no longer share `fork
    --worker`).

- **D3 — Handshake: positional arming via a dedicated cwd; the arming dir carries
  ONLY the shared base; branch/dir hook-derived from `name`.** The orchestrator
  `cd`s into `<coord>/.doctrine/state/dispatch/spawn/` (holding `base`) before a
  worker spawn; the hook keys off payload `cwd` *being* that dir. Rationale: P3
  proved payload `cwd` is the orchestrator's cwd, so cwd is a per-spawn,
  transient, orchestrator-controlled channel. Base is the only shared argv; the
  distinct argv come from the harness-unique `name`. One stable `base` value
  serves a whole batch ⇒ no consume / serialization / correlation key.
  - *Alt — presence-marker (a `pending-spawn` file at the coord root triggers
    dispatch):* rejected (was the prior D3) — a persistent file is a lingering
    false-positive window (F4) and discriminates only by existence, the I1
    blocker. Positional arming moves the trigger to transient cwd-position.
  - *Alt — pure cwd-discrimination on "cwd is an active coord tree":* rejected —
    placement-implicit (regresses the slice thesis) and benign-false-positive
    prone. Here cwd is a *dedicated* dir the orchestrator only enters to dispatch,
    not the coord tree at large.
  - *Alt — encode base (and more) into the dir name / a base64 cwd blob:*
    rejected — the dir carries only what it must; base lives in a `base` file,
    nothing else encoded into the path.
  - *Alt — base derived by the hook from cwd HEAD:* rejected — fragile vs drift;
    base stays explicit in the `base` file.

- **D4 — Discrimination = payload `cwd` IS the arming dir (positional) vs anything
  else (benign pass-through).** The orchestrator's default cwd is the coord root;
  it steps into the arming dir only to spawn workers, so arming is an explicit,
  self-clearing act (cd-out disarms). Closes I1's false-positive window to the
  mechanical floor (§5.5).

- **D5 — Base explicit in the `base` file; cwd is the discriminator, not the base
  source.** Strictly more robust than placement-implicit base (immune to
  coord-HEAD drift between arm and create); the file carries base only.

- **D6 — Two thin verbs (`dispatch arm-spawn`, `worktree create-fork`), pure
  classifier `classify_create` mirroring `classify_stamp`.** Honors the
  pure/imperative split and the gather→classify→act idiom already in the module.

- **D7 — Install emission: settings-block primary, plugin migration secondary +
  droppable, mutually exclusive.** The primary hypothesis test needs no plugin;
  emit the hook via the proven `HookSpec`/`install_claude_hook` machinery into
  `settings.local.json`. The secondary *migrates* the hook into
  `plugins/doctrine/hooks/hooks.json` (after P1) — it must **remove** the
  settings block when it adds the plugin form (double-wiring ⇒ double creation).
  Droppable if it threatens the primary (RSK-2).

- **D8 — Footer-read worker location (P2 holds → primary selected).** The arming
  dir carries only `base`; branch/dir derive from `name`; the orchestrator learns
  the worker location from the return footer's **`worktreePath`** (P2 PASS),
  deriving `name = basename` and `branch = dispatch/<name>` (I3) — parallel-clean,
  no serialization. *Retired fallback (P2 had failed):* orchestrator pre-chooses
  branch/dir, the arming dir carries the full triple, creation serializes. Moot —
  P2 holds.

- **D9 — Pass-through = `git worktree add <root>/.worktrees/<name> HEAD`
  (detached) + provision via the same copier (I2).** Because the hook *replaces*
  native creation, the harness skips `.worktreeinclude`; the benign path must
  replicate it through the same provision copier `run_fork` uses, or it silently
  regresses provisioned files (`.doctrine/doctrine.just`, `web/map/dist/**`) for
  every benign `isolation:worktree` subagent in any installing repo. OQ-2 closed:
  **replicate**, do not accept-the-gap. The hook must be robust because it is
  repo-global (RSK-1).

- **D10 — No WorktreeRemove hook.** Native `git worktree remove` cleans the
  real git worktree; the leftover branch is the funnel import S. Full
  WorktreeRemove ownership is a follow-up.

- **D11 — The byte-identical core is *add+provision+mark*; the env-contract stdout
  emission is arm-specific (surfaced in /plan review, 2026-06-25).** `run_fork`
  today writes the per-worktree env contract (`CARGO_TARGET_DIR=…`) to **stdout**
  (`fork.rs:209-211`) — consumed by the subprocess arm via `env $(fork …)`. The
  WorktreeCreate hook protocol requires **only the worktree path** on stdout, so
  `create-fork` must NOT let that emission reach its stdout. Rationale: the claude
  arm never consumed the env contract anyway — today's stamp flow
  (`run_stamp_subagent`) runs `run_provision`+`write_marker` and emits **no** env
  contract, so the claude worker inherits the orchestrator's `CARGO_TARGET_DIR`.
  Suppressing it in `create-fork` is therefore behaviour-preserving for the claude
  arm (per-worktree target isolation on the claude arm stays a non-goal, as today —
  a possible follow-up). Implementation: split `run_fork`'s add+provision+mark
  core (returns the created dir) from the CLI-layer env-contract emission, so both
  arms share the core and only the subprocess CLI prints the env contract; OR
  `create-fork` invokes `fork --worker` as a subprocess and discards its stdout.
  Either keeps the *creation* core byte-identical (D1 preserved); the "unchanged
  `fork --worker`" wording in §5.2 means the creation semantics, not the CLI's
  stdout side-effect. Decision deferred to PHASE-02 (prefer the core/emission
  split — cleaner cohesion, no extra process).

## 8. Risks & Mitigations

- **RSK-1 (repo-global blast radius, ADR-011 D7 σ).** Every
  `isolation:"worktree"` subagent routes through the hook; a fragile
  pass-through or a hook bug breaks unrelated subagent use, fail-closed.
  *Mitigations:* robust pass-through; pure classifier + golden refusal tokens;
  never-panic on payload; behaviour-preservation suite for the benign path.
- **RSK-2 (plugin-idiom scope creep).** The secondary can swallow the slice.
  *Mitigation:* primary is achievable with zero plugin work; the plugin step is
  additive, gated on P1, and droppable.
- **RSK-3 (false-positive hijack) — largely retired by positional arming (I1).**
  No persistent on-disk trigger; residual = a benign spawn issued while cwd is the
  arming dir (mechanical floor). *Mitigations:* self-clearing cd-out; canonicalised
  cwd-position test; `verify-worker` backstop.
- **RSK-4 (P2 fails) — retired.** P2 holds (§10); `worktreePath` is present and
  normative (I3). The serialized fallback is moot.

## 9. Quality Engineering & Validation

Verification / closure intent (TDD red/green/refactor; pure core + golden tokens):

- **VT — base==B under churn.** A dispatch worker spawned through the funnel
  lands at base B deterministically under simulated moving-`main` (the H1
  scenario) — no fallback-to-main.
- **VT — benign pass-through.** A non-dispatch `isolation:"worktree"` subagent
  gets a working worktree and is **not** stamped/forked as a worker.
- **VT — fail-closed.** Hook failure aborts the spawn (non-zero exit); no silent
  fallback.
- **VT — pure classifier.** `classify_create` returns `Fork{base}` /
  `Passthrough` / named refusals for the matrix (cwd is/isn't the arming dir,
  `base` file present/parses/absent, cwd resolves/not, name valid/empty/unsafe);
  goldens assert the distinct tokens, not a proxy.
- **VT — behaviour preservation.** Existing dispatch suites + the subprocess arm
  stay green unchanged; `doctrine install` emits the hook.
- **VT — stamp retirement (F7).** `doctrine install` (claude arm) **no longer
  emits** the SubagentStart stamp hook, **and** a worker spawned through
  `create-fork` is still marked (provision + `write_marker` ran inside the fork).
  The worker-marker invariant holds via the new seam, not the retired one.
- **VT (secondary) — plugin parity.** The hook in `hooks/hooks.json` fires
  identically to the settings-block form (P1 resolved); the settings block is
  removed in the same step (no double-wiring).
- **VT — benign provisioning parity (I2).** A benign `isolation:worktree`
  subagent's tree contains the `.worktreeinclude` files
  (`.doctrine/doctrine.just`, `web/map/dist/**`) — pass-through provisions
  through the same copier, no regression vs native creation.
- **VT — footer datum is `worktreePath` (I3).** Orchestrator derives
  `name`/`branch` from `worktreePath`; `verify-worker` passes even when the
  footer's `worktreeBranch` is absent/undefined.
- **VT — name sanitiser (I4).** `classify_create` rejects empty / whitespace /
  `/` / `..` / ref-invalid / colliding `name` with a named refusal (fail-closed),
  and accepts the canonical `agent-<hex>` shape.
- **VA/VH — probes P1/P2/P3** run on live 2.1.181 before the dependent locks
  (P2/P3 done — §10; P1 gates the secondary plugin step).

## 10. Review Notes

### Internal adversarial pass (2026-06-25)

Findings raised and dispositioned:

- **F1 — P3 foundational, not just gating.** Integrated: §6 reordered (P3 first);
  a negative result re-opens D3, with the consequence stated.
- **F2 — `create-fork` root / provision-source must match the subprocess arm.**
  Integrated: §5.2 pins root = coord tree.
- **F3 — `fork --worker` may be extracted-but-not-live** (`fork.rs:1`
  `expect(unused … PHASE-03 prunes)`). **Pre-plan check (open):** confirm
  `worktree fork --worker` is CLI-wired and green before planning leans on it.
- **F4 — persistent-marker false-positive window.** Integrated: §5.5 edge +
  optional `dispatch disarm`; accepted as bounded by coord-tree lifetime.
- **F5 — re-dispatch branch/dir leak.** Integrated: §5.5 edge; prune in the
  WorktreeRemove follow-up / `dispatch gc`.
- **F6 — hook is a write (ADR-006).** Integrated: §3 trusted-extension framing.
- **F7 — stamp-retirement verification gap.** Integrated: §9 VT added.
- **D8 divergence note:** the P2-fails fallback changes marker lifecycle
  (persistent-shared → one-shot-consumed/serialized), a larger fork than
  "payload + serialization" alone — flagged for the plan if P2 fails. **Moot —
  P2 holds (below); primary path selected.**

### Probe results (live claude-code 2.1.181, 2026-06-25)

Run in this session via a scratch `WorktreeCreate` hook (logged payload, created
the tree at `$cwd/.worktrees/$name` detached, echoed the path); two trivial
`isolation:worktree` general-purpose spawns; artifacts cleaned up after.

- **P3 — PASS (the spine).** Payload `cwd` follows the orchestrator's working
  directory, not the session launch root. Spawn #1 (cwd `/workspace/doctrine`) →
  payload `cwd=/workspace/doctrine`. After a Bash `cd .dispatch/SL-123` (the
  harness persists Bash cwd across tool calls), spawn #2 → payload
  `cwd=/workspace/doctrine/.dispatch/SL-123`. So the orchestrator parks in the
  coord tree and the hook reads that path from the payload. Each coord tree is
  its own git worktree, so `git -C "$cwd" rev-parse --show-toplevel` resolves to
  the coord-tree root even from a subdir — marker addressing is sound. **D3 holds
  (does not re-open); ASM-1 confirmed.** Full payload still thin:
  `{session_id, transcript_path, cwd, hook_event_name, name:"agent-<hex>"}` —
  no `agent_type`/base/path (matches `mem.pattern.dispatch.worktreecreate-replace-base-control`).
- **P2 — PRESENT.** Both Agent return footers carried
  `worktreePath: <created path>` (plus `agentId:`). **ASM-2 confirmed; D8 primary
  selected.** Caveat: `worktreeBranch:` came back `undefined` — but only because
  the scratch hook created a **detached** worktree (`git worktree add … HEAD`,
  no branch). Residual to settle in the plan, not a blocker: confirm
  `worktreeBranch` populates when the hook creates the named `dispatch/<name>`
  branch (`fork --worker` does). Independent of that, the orchestrator can derive
  branch from the footer's worktree path (`basename` = `name` ⇒ `dispatch/<name>`),
  so D8 primary stands on `worktreePath` alone.
- **P1 — not yet run** (gates only the secondary plugin step; expected yes).

### External inquisition (codex / GPT-5.5, 2026-06-25)

Verdict: **needs rework** — discriminator and benign-path blast radius are
doctrinal defects, not polish. Both factual premises verified in-repo
(`.worktreeinclude` non-empty; dispatch-agent contract hands `worktreeBranch` to
`verify-worker`/funnel). Dispositions:

- **I1 (BLOCKER) — discrimination false-positive (D4/§5.5/INV-2/RSK-3).** The
  *persistent base-only marker* made any `isolation:worktree` spawn from the
  parked coord tree indistinguishable from a worker. **RESOLVED — positional
  arming (D3/D4 rewritten).** The discriminator is now payload `cwd` *being* the
  dedicated arming dir, not a file existing: a benign spawn from the
  orchestrator's normal cwd (coord root) passes through; arming is a transient,
  self-clearing cd-in/cd-out act (no load-bearing `disarm`). F4 dissolves; INV-2
  holds positionally. Residual = a benign spawn issued *while* cwd is the arming
  dir — the mechanical floor (payload has no class tag), far narrower than the old
  lingering-file window; `verify-worker` backstops. *(Origin: the cwd-as-channel
  hack — a direct application of the P3 result.)*
- **I2 (BLOCKER) — benign pass-through provisioning regression (D9/RSK-1).**
  `.worktreeinclude` carries `.doctrine/doctrine.just`, `web/map/dist/**`; the
  benign path `git worktree add … HEAD` skips it (hooks bypass `.worktreeinclude`
  by design). Repo-global ⇒ doctrine's own benign `isolation:worktree` subagents
  silently lose provisioned files, fail-closed blast radius. **ACCEPTED — fix:**
  the benign path provisions through the **same copier** as the dispatch path
  (replicate `.worktreeinclude`); D9 upgraded; OQ-2 closed (replicate, not
  accept-the-gap); VT added (§9).
- **I3 (MAJOR) — D8 primary leaned on the unproven `worktreeBranch` field.**
  Probe proved `worktreePath`, not `worktreeBranch` (came back `undefined` for the
  detached scratch tree); the live dispatch-agent contract still hands
  `worktreeBranch` to `verify-worker` and the funnel. **ACCEPTED — fix:**
  `worktreePath` is the **normative** datum; orchestrator derives
  `name = basename(worktreePath)`, `branch = dispatch/<name>`. `worktreeBranch`
  is no longer load-bearing. dispatch-agent SKILL post-spawn contract updates in
  the plan; VT added (worktreeBranch absent). (Cheap confirming probe — does
  `worktreeBranch` populate for a *named*-branch hook fork — is now nice-to-have,
  not gating.)
- **I4 (MAJOR) — payload `name` treated as ref/path-safe without a contract
  (D3/§5.2/§5.5).** Harness owns `name`; docs admit human-style names; it flows
  unsanitised into `dispatch/<name>` and `.worktrees/<name>`. **ACCEPTED — fix:**
  canonical sanitiser/validator payload `name` → ref+path-safe slug; reject
  outside the envelope (spaces, `/`, `..`, dupes, empty); VTs added (§9).
- **I5 (MAJOR) — "byte-identical core" root-forcing not a locked contract
  (D1/F2/§5.2).** P3 proves *payload* cwd, not *process* cwd; "runs with cwd
  there, or pass `-p`" is not a guarantee. **ACCEPTED — fix:** `create-fork`
  **always** resolves `root = git -C <payload.cwd> --show-toplevel` and passes it
  explicitly into `run_fork`/`run_provision` (never relies on process cwd); F3
  discharged by an e2e proving provisioned files come from the coord tree, not
  the fresh fork. §5.2 tightened.
- **Dismissed by the reviewer:** marker addressing via `--show-toplevel` (sound
  for subdir / sibling / nested / jail); ADR-006 "sole writer" (holds *iff* I1's
  window is closed).

### I1 decision (RESOLVED 2026-06-25) — positional arming via cwd

Decided with the User: **positional arming** (the cwd-as-channel hack), base-in-a
`base`-file (the dir carries only base — nothing encoded into the path). Rejected
alternatives considered: mandatory disarm bracket (still disciplinary), credit-
token directory (machinery for a residual it can't fully close), one-shot
serialize (loses parallel-clean). Positional arming sits at the mechanical floor
with the simplest schema and a self-clearing disarm. Folded into §5.1–5.5,
D3/D4/D5/D8; F4 dissolved.

### Pre-plan checks (carry into /plan)

1. **F3** — verify `run_fork` / `--worker` is live (not dead extraction);
   discharge per I5 with an e2e proving provision source = coord tree.
2. Confirm `dispatch setup` already persists / can surface base B for
   `arm-spawn` to write (vs only emitting it to stdout).
3. Confirm `.worktrees/` is (or will be) gitignored in the coord tree so the
   nested worker worktree does not pollute the coord working set.

### /plan critical review (2026-06-25) — four-agent code+docs grounding

Findings from grounding the plan against `src/` and `docs/claude/`. Design-affecting
ones folded above; plan-level ones live in `plan.toml` / `notes.md`.

- **G1 (design) → D11.** `run_fork` emits the env contract on stdout; the
  WorktreeCreate protocol wants the path alone. Resolved: split core from emission;
  claude arm never consumed the contract, so it is behaviour-preserving.
- **G2 → §5.2.** create-fork root resolution deliberately DIFFERS from the stamp's
  `primary_worktree(cwd)` (stamp fires inside the fork; create-fork in the parent).
  Mirror the *shape*, not the resolution.
- **G3 → §5.2 step 6.** The benign pass-through must compensate (remove the tree it
  added) on failure before the fail-closed exit, or it leaks a half-created tree.
- **Stamp emission is install-driven at TWO sites** (`skills.rs:1056-1077`,
  `install.rs:366-385`), gated `!global` + Claude target — both retired by D2.
- **Stale comments to reconcile when create-fork revives:** `subagent.rs:137-139`
  ("`create-fork` is DROPPED") and `fork.rs:51-52` (cleanup "shared by … PHASE-10's
  create-fork"). The drop rationale (thin payload) is obsoleted by positional arming.
- **Payload `agent_type`:** docs' common-fields rule (`hooks.md:591-596`) adds
  `agent_id`/`agent_type` for in-subagent hooks, but P3 logged a thin WorktreeCreate
  payload on 2.1.181 — WorktreeCreate fires in the *parent* (creating the tree
  before the child runs), so the in-subagent fields don't apply. Either way the
  design is agent_type-agnostic (positional arming); non-issue.
- **Footer `worktreePath`:** empirically confirmed (P2) though undocumented; the
  docs' `worktreePath` is the unrelated HTTP-hook output field — do not conflate.
- **`name` shape:** sanitiser must accept BOTH `agent-<hex>` (P3, tool spawns) and
  the moby `word-word-hex` form (`hooks.md:2419`, user/`--worktree` spawns).

### Lock status

P3 + P2 resolved (probes); I1 resolved (positional arming); I2–I5 absorbed.
The schema (positional arming, `base`-file, footer-read, D8 primary) is settled.
Remaining before `/plan` leans on them: the three pre-plan checks above (F3/I5
e2e, `arm-spawn` base-B source, `.worktrees/` gitignore). **P1** (plugin-hook
parity) gates only the secondary plugin step. No open design forks.
