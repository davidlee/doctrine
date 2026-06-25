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
- **Jail**: worker worktrees must live inside the project root
  (bubblewrap-confined `/workspace/<repo>`).
- **DRY / no parallel implementation**: the hook *shells the existing*
  `fork --worker`; it does not reimplement creation, provisioning, or marking.

## 4. Guiding Principles

- **Collapse the arms literally.** Success = both arms call one byte-identical
  `fork --worker` core. They differ only in how its arguments arrive: subprocess
  passes argv; the claude hook reads them from a marker + derives the rest from
  the payload (the harness owns the spawn, so doctrine cannot pass argv).
- **Explicit intent over placement inference.** The orchestrator *arms* a spawn
  (drops a marker); the hook does not infer "this is a dispatch worker" from
  placement alone. This is the move away from the fragile placement-implicit base.
- **Base is explicit, never inferred.** The marker carries B; the hook does not
  read it from cwd HEAD. cwd only *addresses* the marker.
- **Fail-closed on the dispatch path; robust pass-through off it.**

## 5. Proposed Design

### 5.1 System Model

```
ORCHESTRATOR (claude session, sole writer, parked Bash cwd = coord tree)
  │  dispatch arm-spawn --base B        ── writes the cwd-local base marker (§5.3)
  │  <Agent spawn, isolation:worktree>  ── harness fires WorktreeCreate synchronously
  ▼
WorktreeCreate HOOK  →  doctrine worktree create-fork           (the new verb, §5.2)
  │  read payload {cwd, name}
  │  resolve coord-tree root = git -C <cwd> --show-toplevel
  │  marker present at <root>/.doctrine/state/dispatch/pending-spawn ?
  │   ├─ YES (dispatch): base=marker.B; branch=dispatch/<name>; dir=<root>/.worktrees/<name>
  │   │        → fork --worker --base B --branch ... --dir ...   (IDENTICAL to subprocess arm)
  │   │        → print dir on stdout ; any failure → non-zero exit (fail-closed)
  │   └─ NO  (benign):   git worktree add <root>/.worktrees/<name> HEAD → print dir
  ▼
WORKER subagent runs in the created tree (base-guard, work, return footer)
  ▼
ORCHESTRATOR post-spawn (unchanged belt):
  read worktreePath:/worktreeBranch: from footer
  doctrine worktree verify-worker --base B --dir <path> --branch <branch>
```

The SL-123 `SubagentStart` stamp hook is **no longer wired on the claude arm**
(D2): `fork --worker` already provisioned + marked atomically inside
WorktreeCreate.

### 5.2 Interfaces & Contracts

Two thin verbs, both shells over existing pure logic:

**`doctrine dispatch arm-spawn --base <B> [--slice <N>]`** (orchestrator side)
- Writes / overwrites the cwd-local base marker (§5.3) in the current
  coordination tree. Persistent; rewritten whenever B advances (per serial
  phase / per parallel batch). **Not** a one-shot.
- Sole-writer; serial within the orchestrator's own funnel. Idempotent.

**`doctrine worktree create-fork`** (hook side; reads stdin payload)
- Gather → pure-classify → act, mirroring `run_stamp_subagent`:
  1. read stdin → `{cwd, name}` (malformed ⇒ empty ⇒ benign/refuse, never panic);
  2. resolve coord-tree root from `cwd` (`--show-toplevel`); locate the marker;
  3. **pure classifier** `classify_create(...)` → `Fork{base}` | `Passthrough`
     (+ named refusals);
  4. act: `Fork` → `run_fork(base, branch=dispatch/<name>,
     dir=<root>/.worktrees/<name>, worker=true)`; `Passthrough` →
     `git worktree add <root>/.worktrees/<name> HEAD`;
  5. **print the created absolute path on stdout** (the harness path protocol);
  6. **any failure ⇒ non-zero exit** (fail-closed creation).
- Branch / dir derived from the payload `name` (harness-unique per spawn) — the
  per-spawn uniqueness git requires (§5.5), without the orchestrator
  pre-choosing them.

`fork --worker` is **unchanged** — `create-fork` is a new caller of the existing
function, exactly as the subprocess arm calls it.

### 5.3 Data, State & Ownership

**The marker.** Path `<coord-tree>/.doctrine/state/dispatch/pending-spawn`.
- **Per-coord-tree**, in the coord tree's own runtime state — already gitignored
  (runtime tier) and already in the ADR-006 D9 **withheld** list, so it is never
  copied into a worker fork. Cross-slice and parallel-across-slices partition
  for free (different coord trees ⇒ different files).
- **Carries only the shared `--base B`** (one minimal line / `base = "<sha>"`).
  Because file-disjoint parallel phases share B, one persistent value serves a
  whole parallel batch — **no consume, no rename, no serialization, no per-spawn
  correlation key.** A parallel batch is N concurrent *reads* of one stable value.
- Owner: the orchestrator (sole writer). The hook only **reads** it.

**Why base shared but branch/dir not** (the crux):

| argv | parallel batch | source |
|---|---|---|
| `--base B` | **shared** (all fork the coord tip) | the marker |
| `--branch` | **distinct** (git: one branch per worktree) | hook-derived `dispatch/<name>` |
| `--dir` | **distinct** (one path per worktree) | hook-derived `<root>/.worktrees/<name>` |

The thing we could not get pre-spawn (`name`) is exactly the thing the hook
derives uniqueness from, locally. The orchestrator learns each worker's
branch/dir from the **return footer** (it already reads `worktreePath:`/
`worktreeBranch:` today) — free **iff the footer survives hook-creation (P2)**.

### 5.4 Lifecycle, Operations & Dynamics

- **Serial drive (default).** arm-spawn at B → spawn → hook forks at B → worker
  runs → funnel commit advances coord HEAD → arm-spawn at B′ → next spawn. The
  marker is rewritten each phase; base is explicit so coord-HEAD drift between
  arm and create is irrelevant.
- **Parallel batch (file-disjoint).** arm-spawn at B once → issue N spawns; each
  hook reads the same B, derives its own branch/dir from its own `name`. No
  serialization of *runs*; creation is naturally independent (distinct
  branch/dir per `name`).
- **Benign isolated subagent (any repo).** No marker in the resolved root ⇒
  pass-through ⇒ a working detached worktree, **not** stamped/forked as a worker.
- **Cleanup.** Native `git worktree remove` handles teardown (real git
  worktree); the leftover branch is the funnel's import S (benign). No
  WorktreeRemove hook needed (D10).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1.** On the dispatch path the created worker forks **exactly B** (marker
  value), or the spawn aborts. No silent fallback.
- **INV-2.** A benign subagent is never stamped/forked as a worker.
- **INV-3.** `fork --worker` semantics are identical on both arms (shared fn).
- **ASM-1.** Payload `cwd` is the orchestrator's parked Bash cwd (the coord
  tree) — **gating probe P3**; the marker is located via it.
- **ASM-2.** The return footer survives hook-creation — **gating probe P2**;
  selects the marker schema (§7 D8).
- **Edge — stale marker.** A marker left from a prior phase could in principle
  greet an unrelated `isolation:worktree` subagent spawned from the same coord
  tree. Mitigations: (a) the orchestrator does not spawn benign subagents from a
  parked coord tree during a drive; (b) optional cwd cross-check; (c)
  `verify-worker` is the post-spawn backstop. Residual is low given sole-writer +
  serial arming.
- **Edge — malformed/empty payload.** Fold to a named refusal; fail-closed;
  never panic on hook input (mirrors `run_stamp_subagent`).
- **Edge — pass-through fail.** A non-zero exit aborts a *benign* subagent's
  creation (RSK-1). The pass-through must be robust; `.worktreeinclude` fidelity
  is the known gap (OQ-2).

## 6. Open Questions & Unknowns

Probes (need a live 2.1.181 dispatch; likely a you-run-it step). The design is
sequenced so these resolve **before** the dependent schema/emission locks.

- **P1 (was OQ-5) — plugin-hook parity.** Does a `WorktreeCreate` hook in plugin
  `hooks/hooks.json` fire identically to the settings-block form? *Gates the
  secondary plugin step.* Expected yes; verify before relying.
- **P2 — footer survival.** Does the harness still emit `worktreePath:`/
  `worktreeBranch:` when the hook created the worktree? *Gates the marker schema
  fork (D8).*
- **P3 — payload cwd identity.** Is payload `cwd` the parked Bash cwd (coord
  tree) or the session root? *Gating* — the marker is located via it.
- **OQ-2 (residual) — pass-through `.worktreeinclude` fidelity.** What does a
  non-dispatch subagent that relied on `.worktreeinclude` lose? Decide:
  replicate vs accept-the-gap-and-document.

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

- **D3 — Handshake: cwd-local marker carrying ONLY the shared base; branch/dir
  hook-derived from `name`.** Rationale: base is the *only* shared argv across a
  parallel batch; deriving the distinct argv from the harness-unique `name`
  collapses the marker to one persistent value ⇒ no consume / serialization /
  correlation key. The unobtainable per-spawn key is exactly what the hook
  derives uniqueness from.
  - *Alt — global `.doctrine/state/` marker:* rejected — cross-slice + parallel
    contention on one shared path.
  - *Alt — serialized single-entry triple (orchestrator pre-chooses branch/dir,
    creation serializes):* the **P2-fails fallback** (D8), not primary.
  - *Alt — pure cwd-discrimination, no marker (infer dispatch from "cwd is an
    active coord tree"):* rejected — placement-implicit (regresses the slice
    thesis) and risks a benign false-positive.
  - *Alt — base derived by the hook from cwd HEAD:* rejected — fragile vs drift
    and regresses toward implicit base; base stays explicit in the marker.

- **D4 — Discrimination = marker present (explicit intent) vs absent (benign
  pass-through).** Keeps benign subagents safe and makes arming an explicit act.

- **D5 — Base explicit in the marker; cwd only addresses the marker.** Strictly
  more robust than today's placement-implicit base (immune to coord-HEAD drift
  between arm and create).

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

- **D8 — P2 fork in the marker schema.** *P2 holds:* marker = bare base;
  branch/dir from `name`; orchestrator reads them from the footer (primary,
  parallel-clean, no serialization). *P2 fails:* orchestrator pre-chooses
  branch/dir, marker carries the full triple, creation serializes (arm → spawn →
  hook consumes synchronously → arm next; runs still parallel). Same
  `create-fork` verb and `fork --worker` core; differ only in marker payload +
  whether creation serializes.

- **D9 — Pass-through = `git worktree add <root>/.worktrees/<name> HEAD`
  (detached).** `.worktreeinclude` gap accepted pending OQ-2; the hook must be
  robust because it is repo-global (RSK-1).

- **D10 — No WorktreeRemove hook.** Native `git worktree remove` cleans the
  real git worktree; the leftover branch is the funnel import S. Full
  WorktreeRemove ownership is a follow-up.

## 8. Risks & Mitigations

- **RSK-1 (repo-global blast radius, ADR-011 D7 σ).** Every
  `isolation:"worktree"` subagent routes through the hook; a fragile
  pass-through or a hook bug breaks unrelated subagent use, fail-closed.
  *Mitigations:* robust pass-through; pure classifier + golden refusal tokens;
  never-panic on payload; behaviour-preservation suite for the benign path.
- **RSK-2 (plugin-idiom scope creep).** The secondary can swallow the slice.
  *Mitigation:* primary is achievable with zero plugin work; the plugin step is
  additive, gated on P1, and droppable.
- **RSK-3 (stale-marker hijack).** §5.5 edge — low residual under sole-writer +
  serial arming; optional cwd cross-check; `verify-worker` backstop.
- **RSK-4 (P2 fails).** Footer absent ⇒ orchestrator cannot learn hook-derived
  branch/dir. *Mitigation:* D8 fallback (pre-chosen triple + serialized
  creation) — designed, not improvised.

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
  `Passthrough` / named refusals for the matrix (marker present/absent, cwd
  resolves/not, name present/empty); goldens assert the distinct tokens, not a
  proxy.
- **VT — behaviour preservation.** Existing dispatch suites + the subprocess arm
  stay green unchanged; `doctrine install` emits the hook.
- **VT (secondary) — plugin parity.** The hook in `hooks/hooks.json` fires
  identically to the settings-block form (P1 resolved); the settings block is
  removed in the same step (no double-wiring).
- **VA/VH — probes P1/P2/P3** run on live 2.1.181 before the dependent locks.

## 10. Review Notes

(Adversarial pass + external/inquisition findings recorded here.)

- The design leans on three live-harness probes (P1/P2/P3). P3 (payload cwd
  identity) is the load-bearing one — if cwd is the session root rather than the
  parked coord tree, the marker-location seam needs an alternate address
  (revisit D3). P2 selects the schema (D8); P1 only gates the secondary.
