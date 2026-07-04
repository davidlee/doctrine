# Notes SL-198: Mode B foundation — gated `worker_commit` + worker tool-surface lint

Durable per-slice scratchpad — tracked in git.

## State (2026-07-04)

- **Lifecycle:** `design`. Design authored + internal (F1–F5) + external codex (X1–X5)
  adversarial passes done and folded. X1 blocker **resolved by owner ruling** (agent-id
  registry lookup). **No plan, no phases yet.** Next step: `slice status 198 plan` → `/plan`.
- **Not started:** all implementation. This is a scoped+designed slice only.
- **Gate (`doctrine check gate`):** N/A — no code touched yet.

## Commits (all `.doctrine` state, landed promptly, no code)

- `7bd21f49` — RSK-225 gate discharged (probe: MCP-write bypasses SL-182 wall both
  arms); wall memory promoted inferred→witnessed; RFC-005 Mode B → validated default.
- `b6edbe61` / `349bf796` — SL-198 scoped, then narrowed to foundation; SL-199 capstone
  created (`needs` SL-198, serial-dependent).
- `9d0eb405` — SL-198 `design.md` (worker_commit + lint) + design-target selectors.
- `977244ac` — internal adversarial pass F1–F5.

## The design in one breath

Jailed worker's raw `git commit` is walled (ro `.git`). `worker_commit` MCP tool runs
in the **unconfined** doctrine MCP server → commits the worker's delta on the worker's
**own** worktree HEAD (Shape 1: one non-merge commit, `C^==B`). Main-thread orchestrator
then imports the **commit** (not the working-tree diff). Belts server-enforced:
`check commit` (commit tier, fmt+lint+test+build) → `classify_import` scope vs
design-target selectors → `HEAD==B` + one-non-merge. base **B** from arming slot.
Conformance **lint** = MCP allowlist (only `worker_commit`) on agent-defs marked
`doctrine-role: worker`. All over existing seams (`src/worktree/import.rs`,
`subagent.rs`, `verify.rs`). Subprocess arm + main-thread fallback untouched.

## External codex pass (2026-07-04) — REFRAMED the slice (design.md §10 X1–X5)

All 5 findings source-verified. **The reuse is real but the real work moved:**
- **X5 (de-risk):** OQ-2/OQ-2b **CLOSED YES**. `run_import` fork path takes a detached
  worktree-HEAD oid (`import.rs:301/354`); `run_verify_worker` already does
  `--is-ancestor B HEAD` not `HEAD==B` (`subagent.rs:360`). Import-switch = clean reuse.
- **X2 (adopt, net-new):** base B must be **snapshotted per-worktree at `create-fork`**
  (beside jail policy, `JAIL_SUBPATH`, ro to worker) — NOT read from the mutable arming
  slot at commit time (racy, overwritten on re-arm). worker_commit reads that immutable
  base. Touches `src/worktree/create.rs`. **Supersedes D4.**
- **X1 (RESOLVED by owner ruling 2026-07-04):** MCP server gets **no caller agent_id**
  (`tools.rs:395`), worker `Read` passes the wall → true caller-auth **unachievable in
  this harness**. **Ruling:** worker passes an **`agent` id (its worktree name), not a
  `dir`**; server **looks up** the worktree from the registry (`JAIL_SUBPATH/<agent>.toml`
  present ⟺ spawned — the target-fence; `dir`/`base`/`branch` derive from that one key).
  Worker cannot freely specify a path. **Residual accepted (small blast radius):** spoof a
  *sibling's* registered name → in-scope commit on its branch (attribution confusion,
  review-caught, **own work not promoted**, no escalation). Seam verified: registry keyed
  by name at `create.rs:245`, path `<coord>/.worktrees/<agent>` (`WORKTREES_SUBDIR`),
  branch `dispatch/<agent>`. Composes with X2 (base snapshot beside the same key).
  Follow-on caller-binding = RSK-226.
- **X3 (adopt):** lint targets `.doctrine/agents/**` + `install/agents/**` (authored),
  NOT `.claude/agents` (installed copy); marker **MANDATORY** (deny-by-default), fixes
  the fail-open R2. design-target selectors updated.
- **X4 (adopt):** reorder belts — cheap admissibility (target-fence, immutable base,
  HEAD==B, non-empty pre-fmt delta, pre-fmt scope) FIRST, then the mutating `check
  commit`. Subsumes F2.

**Real slice = (1) create-fork base snapshot + trusted read (X2), (2) worker_commit with
cheap-first belts over reused classify_import/fork-import (X1/X4/X5), (3) mandatory
correctly-targeted lint (X3).** The import switch is the small part.

## Open items → resolve at /plan (do NOT lose these)

- **OQ-2b (sharpest).** Does `run_import`'s fork path accept a **detached worktree-HEAD
  oid**, or need a named ref? If named-ref, cut `phase/<slice>-NN` at the worker HEAD
  (ADR-012 D3) or extend the fork path. **Gates the "reuse, no new code" claim.**
- **OQ-1.** Lint host (a `doctrine check` sub-check vs dedicated verb) + marker
  mechanism (frontmatter key vs `doctrine.toml` list). Lint-host **file not yet in the
  design-target selector set** — add it when chosen at plan.
- **OQ-2.** `run_verify_worker` base==B relaxation for the post-commit (HEAD==C) claude
  case, without disturbing the subprocess arm.
- **F2.** `check commit` fmt (mutating) runs *before* scope belt → a pre-existing
  mis-formatted untouched file could false-flag `undeclared-scope`. Scope the belt to
  the worker's intended diff, or assert B fmt-clean at fork.
- **F4 (governance ruling needed).** SL-198 un-jails nothing (rides witnessed
  passthrough) → **no ADR REV**. But a sanctioned worker MCP write is a deliberate hole
  in SL-182/ADR-008, made safe by the lint → likely a **note** on ADR-008/SL-182,
  ratified at reconcile. Confirm with reviewer / `/consult` if contested. (ADR-012 REV +
  ADR-011 D6 amendment belong to **SL-199**, not here.)
- **F3.** Concurrent `worker_commit` safe by linked-worktree isolation; server
  serialization is an SL-199 concern.

## Owner steers (locked this session)

- Two serial-dependent slices: SL-198 foundation now, SL-199 capstone shaped in parallel.
- Gate = `check commit` (heavy: fmt+test+build), "see how it plays" — keep, tier is
  `[verification]`-configurable.
- Shape 1 (worker commits own HEAD). base-B source: owner deferred → arming slot +
  `HEAD==B` (my call, D4). message worker-supplied, orchestrator may amend.
- Default reviewer: codex (GPT-5.5).

## Related durable knowledge (already memories — don't re-derive)

- [[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]] — the witnessed bypass.
- [[mem.fact.claude.worktree-remove-auto-teardown]] — orchestrator owns/reaps worktree.
- [[mem.fact.dispatch.single-slot-arming-rendezvous]] — one arming = one base; Agent
  call blocks.
- [[mem.fact.dispatch.coord-root-not-git-common-dir]] — layout-strip to coord root.
- [[mem.pattern.dispatch.prewarm-fork-target-reflink]] — warm target or `check commit`
  build times out.
