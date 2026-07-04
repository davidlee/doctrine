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

## Execution (2026-07-04) — PHASE-01/02/03 landed via claude-arm dispatch

Driven on coord tree `dispatch/198` (`.dispatch/SL-198`). Base B walked:
`5a1a06e8` (P01 tip) → `617129ff` (orchestrator pre-added `ignore` dep, P02 base)
→ `c6fb7c8a` (P02 tip) → `02645cb4` (P03 tip) → `49032c3f` (RFC-011 case-note).

- **PHASE-01 — per-worktree dispatch record + resolver** (`5a1a06e8`, prior session).
  `src/worktree/dispatch_record.rs`: `resolve_agent` + `DispatchRecord{name,dir,branch,
  base,coord}` + `ResolveRefusal`. Record at `.doctrine/state/dispatch/record/<name>.toml`
  (built path is `record/`, NOT design §5.3's `jail/` → **F-record-path**, encapsulated
  by `resolve_agent`).
- **PHASE-02 — keystone `worker_commit` MCP tool** (`c6fb7c8a`, 9 files +792/−12).
  Confined claude-arm worker (`agent-ae991b01…`) forked at B=`617129ff`, built it TDD,
  handed back a working-tree delta; orchestrator funnelled it: verify-worker (HEAD==B) →
  `import --from-worktree` → **markerless-coord gate** → regression diff vs B clean →
  branch-point stationary → one path-limited commit → boundary → reaped.
  - `dispatch_config.rs` (`worker-forbidden-writes` + `ForbiddenWrites` matcher, `.doctrine/`
    floor fail-closed w/ precedence, PIN-2); `mcp_server/worker_commit.rs` (new handler,
    cheap-first belts, two-tier scope reusing import prefixes + `undeclared_paths`, PIN-4);
    tools.rs registration 18→19; agent-def pin; install template defaults.
  - **Worker false-negative caught (durable lesson).** Worker reported unit-green (3101)
    and claimed its gate `test`-step red was "all environmental" (worker-marker refusals).
    Distrusted → ran full markerless suite → ONE real failure hid in the noise:
    `vt2_tools_list` (e2e twin of the tool-count assert) still hardcoded 18. The
    worker-marker turns real e2e failures into authored-write refusals that look identical
    to env noise. Orchestrator fixed the 2-line e2e assert (T3 completion). → candidate
    memory (below).
  - **Seam promotions (worker-disclosed):** `import.rs` `DOCTRINE_PREFIX`/`CLAUDE_PREFIX`/
    `gather_worktree_delta_paths` → `pub(crate)`; `worktree/mod.rs` re-exports of P01.
    `worktree/mod.rs` is `src/worktree/**` **scope-relevant, not design-target** →
    **F-seam-promotion** (imported w/o `--slice`; the byte-for-byte belt still HARD-rejects
    `.doctrine/`/`.claude/`).
- **PHASE-03 — commit-import switch = PURE REUSE** (`02645cb4`, in-thread, owner-approved
  approach a). Design **X5 verified in source**: `run_import` `--fork` path already takes a
  detached worker-HEAD oid (import.rs:274); `run_verify_worker` already gates
  `merge-base --is-ancestor B HEAD` (subagent.rs:362), not `HEAD==B` → a worker one
  non-merge commit above B verifies green **unchanged**. Zero new production code.
  - `tests/e2e_worktree_verify_worker.rs` VT-1 **guard test** (worker one commit above B on
    a coherent branch verifies OK — locks the is-ancestor semantics; regression to `HEAD==B`
    fails loud). `classify_worker_verify` already fully unit-tested (mod.rs:1156-1219).
  - **Note home corrected (owner steer):** generic two-arm mechanics → shipped
    `install/dispatch-mechanics.md` (new section); `.doctrine/governance.md` narrowed to the
    pi-arm note + pointer. Plan/design said "CLAUDE.md # orchestration" but no such section
    exists and the note is generic doctrine knowledge → **F-note-home** (shipped, not
    project-local; not CLAUDE.md).
  - EX-6/PIN-4: import-time `UndeclaredScope` (import.rs:73/135) preserved, no change.

### Reconcile carries (harvest at `/reconcile`)
- **F-order** — design §5.2 gate/scope step-numbering vs plan EX-2 cheap-first; plan
  correct per INV-5 (pre-fmt snapshot precedes mutating fmt).
- **F-record-path** — §5.3 `jail/<name>.toml` vs built `record/<name>.toml`.
- **F-seam-promotion** — `worktree/mod.rs` re-exports touched (scope-relevant, not
  design-target); precise-selector or accept.
- **F-x5-holds** — P03 objective text implies a code relax X5 already obviated; switch is
  operational + documented, not coded.
- **F-note-home** — orchestration note is shipped-generic (`dispatch-mechanics.md`), not
  `CLAUDE.md`; update design §5.4 / plan EX-4/EX-5 wording.

### Boundaries / gate / deferred
- `boundaries.toml`: PHASE-01/02/03 recorded (contiguous, provenance `funnel`).
- **Gate:** P03 `doctrine check commit` **green** (exit 0) at last code change; regression
  vs B clean. All `.doctrine`/code committed promptly, path-limited.
- **Deferred to close:** (a) `doctrine boot` regen (governance.md edited — needs `/clear`
  to reload the inlined snapshot); (b) `cargo build` re-embed + `doctrine install`
  (`install/dispatch-mechanics.md` is a RustEmbed asset — installed copy inert until
  re-embedded); (c) VA-1 live end-to-end (jailed worker calls `worker_commit`) needs the
  new binary installed — natural at audit.

### Remaining
- **PHASE-04** — worker tool-surface conformance lint (RSK-225), `src/commands/doctor.rs`,
  wired into `just validate`. Real code → **worker-dispatch** (unlike P03). VH-1: owner
  confirms allowlist semantics before close.
- Then: `dispatch refresh-base` (trunk moved ahead of fork-point) → `slice verify-vt` →
  prepare-review → `/audit` → `/reconcile` (carry the 5 F-items) → `/close`.

### Candidate memory (run `/record-memory` before close)
- **Confined-worker gate-green ≠ coord-green.** A dispatch worker's in-jail gate masks real
  e2e failures: the worker-mode marker makes authored-write verbs refuse, so genuine e2e
  failures surface as marker-refusals indistinguishable from environmental noise. The
  orchestrator MUST re-run the full gate in the **markerless coord tree** post-import and
  enumerate the **complete** failure set (`--no-fail-fast`) — never trust the worker's
  blanket "environmental" verdict. (Sharpens `dispatch-mechanics.md` "never trust the
  worker's self-reported success".)

## Related durable knowledge (already memories — don't re-derive)

- [[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]] — the witnessed bypass.
- [[mem.fact.claude.worktree-remove-auto-teardown]] — orchestrator owns/reaps worktree.
- [[mem.fact.dispatch.single-slot-arming-rendezvous]] — one arming = one base; Agent
  call blocks.
- [[mem.fact.dispatch.coord-root-not-git-common-dir]] — layout-strip to coord root.
- [[mem.pattern.dispatch.prewarm-fork-target-reflink]] — warm target or `check commit`
  build times out.
