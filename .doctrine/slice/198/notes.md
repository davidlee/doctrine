# Notes SL-198: Mode B foundation — gated `worker_commit` + worker tool-surface lint

Durable per-slice scratchpad — tracked in git.

## State (2026-07-04)

- **Lifecycle:** `ready` (flipped 2026-07-04 after codex pass-3 clean). Design locked
  (internal F1–F5 + **three** external codex passes; pass-2 LOCK-READY, pass-3
  LOCK-READY-WITH-PINS). Plan authored, critically revised, **4 phases materialised**
  (PHASE-01..04).
- **Codex pass-3 (2026-07-04) — the config-surface delta.** Half STILL-OPEN was a category
  read of net-new design-target work as "missing source" (discounted). Real signal folded as
  PIN-1..4 (design §10 pass-3; plan EN-3/EX-5/EX-6, PHASE-03 EX-6):
  1. **PIN-1** — `allowlist.rs:96` rejects `!` negation → **`ignore` crate forced** (EN-3
     settled, not deferred; new dep by elimination).
  2. **PIN-2** — `.doctrine/**` floor is a **separate code check** with precedence, NOT a
     line in the merged GitignoreBuilder (last-match-wins would fail OPEN on a user `!` line).
  3. **PIN-3** — INV-5 = stage-by-path **after** fmt (post-fmt content, not a pre-fmt blob).
  4. **PIN-4** — PHASE-03 preserves the import-time `UndeclaredScope` refusal
     (import.rs:130/171); the soft tier is bounded only by import staying strict.
  - **flake.nix ruling:** no legacy installs + POL-002 = "this repo" → stays a
    block-by-default, negatable template entry (not a floor). Fence altitude unchanged.
- **Post-pass-2 owner steers folded (design §5.2/§5.3/§10, plan PHASE-02/03):**
  1. **Pre-fmt trunk before arming** (PHASE-03 EX-5) → B fmt-clean at fork, F2 moot
     operationally; stage-classified-paths (INV-5) is the fallback.
  2. **Two-tier scope belt.** HARD = `[dispatch].worker-forbidden-writes` **config surface**
     (gitignore syntax, library matcher — no hand-roll; defaults ship in install template
     `install/doctrine.toml.example`: `.doctrine/**`, `.claude/**`, `.agents/**`,
     `install/agents/**`, `flake.nix`; project-negatable) + **code floor `.doctrine/**`
     fail-closed** (precedence over config). SOFT = `undeclared:[paths]` (src outside
     selectors commits + is returned; orchestrator blesses or rejects at import). POL-002
     resolved at root (`justfile`/CI not defaulted — host-project).
  3. New design-targets: `src/dispatch_config.rs`, `install/doctrine.toml.example`,
     `src/worktree/gc.rs`.
- **Not started:** all implementation. This is a scoped+designed slice only.
- **Gate (`doctrine check gate`):** N/A — no code touched yet.

## Commits (all `.doctrine` state, landed promptly, no code)

- `7bd21f49` — RSK-225 gate discharged (probe: MCP-write bypasses SL-182 wall both
  arms); wall memory promoted inferred→witnessed; RFC-005 Mode B → validated default.
- `b6edbe61` / `349bf796` — SL-198 scoped, then narrowed to foundation; SL-199 capstone
  created (`needs` SL-198, serial-dependent).
- `9d0eb405` — SL-198 `design.md` (worker_commit + lint) + design-target selectors.
- `977244ac` — internal adversarial pass F1–F5.
- codex pass-2 + folds → `1dfd58e6` (auto-committer swept it). Plan: `3a6f6928` (4-phase),
  `3c428105` (critical pass), `8791676e` (two-tier belt + pre-fmt), `179f8fea` (POL-002),
  `34a8c343` (worker-forbidden-writes config surface). All `.doctrine` state, no code.
- **Gate (`doctrine check gate`):** N/A — no code touched. `slice verify-vt 198` parses;
  PHASE-02 net-new `worker_commit.rs` reads FAIL (expected pre-impl red).

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
- **X1 (RESOLVED — owner ruling + codex pass-2, 2026-07-04):** MCP server gets **no caller
  agent_id** (`tools.rs:395`) and root is fixed to **primary** at startup (`mod.rs:26`) —
  it does NOT know the coord root. **Final mechanism (design §10 pass-2, LOCK-READY):**
  worker passes an **opaque `agent` id, no path**; server sanitises it (one validator,
  `create.rs:108`) → `git worktree list --porcelain` (`git.rs:1380`) enumerates live
  `dispatch/<NNN>` coord trees → probes each for the per-worktree record `jail/<agent>.toml`
  → **exactly one live hit** (0=`unknown-agent`, >1=`ambiguous-agent`) → validates
  `{dir,branch,base}` consistent. **No worker path, no new coord registry** (git worktree
  list IS the primary-readable coord pointer — codex X-1). Record `{name,dir,branch,base,
  coord}` written by the **trusted create-fork hook**, **deleted at reap/gc** (net-new
  `gc.rs` step — fixes the stale-oracle; supersedes pass-1 "base beside jail policy" +
  D4). Residual: spoof a *sibling's* live agent → commit on its branch (own branch stays
  at B ⇒ own work unpromoted, orchestrator imports the branch it armed via
  `verify-worker --branch`, `subagent.rs:316`). RSK-226 = caller-binding follow-on.
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
