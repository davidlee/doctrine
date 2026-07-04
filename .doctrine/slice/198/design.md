# Design SL-198: Mode B foundation — gated `worker_commit` + worker tool-surface lint

<!-- Reference forms: entity ids padded (SL-198, ADR-012, RSK-225); doc-local refs
     bare — OQ-1 (§6), D1 (§7), R1 (§8). -->

## 1. Design Problem

A dispatch worker runs in a linked worktree with the shared `.git` **read-only**
(SL-182 jail; witnessed). Its raw Bash `git commit` is therefore walled, so today the
orchestrator must import the worker's **working-tree diff** and commit on its behalf
(CLAUDE.md `# orchestration`). RFC-005 Mode B removes that dance by letting the jailed
worker **self-commit through the unconfined doctrine MCP server** — the one process
that can write the shared `.git`. This slice builds that keystone (`worker_commit`,
IMP-253) and the guard that keeps it safe (RSK-225: a worker must hold no other
writable MCP tool). It is driven by the **existing main-thread orchestrator**; the
confined-*subagent* orchestrator is the serial-dependent capstone SL-199.

## 2. Current State

- **Jail.** `src/worktree/pretooluse.rs` `decide()` mediates only `Bash|Edit|Write`;
  every other tool — including `mcp__*` — passes through. An MCP write by a jailed
  subagent runs in the harness-child MCP server (unconfined, cwd=primary) and lands in
  the primary tree, escaping the jail (RSK-225, witnessed; commit `7bd21f49`).
- **Import belts (the reuse surface).** `src/worktree/import.rs`: `classify_import`
  (pure scope belt), `run_prove_on` / `import_prove_verdict` (the `prove` gate),
  `run_import(path, base, fork, from_worktree, selectors)` — already **two-source**:
  `(Some(fork), None)` imports a committed source; `(None, Some(dir))` applies a
  working-tree diff. `src/worktree/subagent.rs` `run_verify_worker` asserts base==B.
- **Claude-arm funnel.** Orchestrator (main thread) reads the Agent footer
  `worktreePath`, `verify-worker --dir` (HEAD==B), `worktree import --from-worktree`
  (diff), commits, `git worktree remove --force`
  ([[mem.fact.claude.worktree-remove-auto-teardown]]).
- **Check tiers** (`src/verify.rs` + justfile): `commit` → `just check` =
  `fmt lint lint-js validate test build` (**fmt mutates**); `prove` → `just prove` =
  `fmt-check lint` (non-mutating). Tiers are project-overridable via `[verification]`.
- **MCP server** (`src/mcp_server/tools.rs`) exports review/memory/onboard tools only —
  no dispatch surface. Writable `memory_record`/`memory_edit` already exist (the
  RSK-225 footgun).

## 3. Forces & Constraints

- **Threat model (RFC-005, locked).** A prompt-injected worker is adversarial; do not
  rest integrity on trusting the worker. Belts are enforced **server-side**, never
  worker-run. Belts bound blast radius (path/base/gate), **not** intent — intent stays
  with review/audit/human.
- **ADR-012 D1/D4.** Projection is a two-stage sync verb **at conclude**; the
  coordination worktree is the sole write target; per-worker projection onto shared
  branches "re-creates the contention this topology removes" → forbidden. ⇒ D1 below.
- **ADR-006 sole-writer.** A worker writing shared refs breaches it. `worker_commit`
  writes only the worker's *own* worktree HEAD.
- **DRY / no parallel implementation.** Reuse `classify_import`, the check-tier runner,
  `run_import`'s fork path, `run_verify_worker` — do not re-author belts.
- **ADR-001 layering.** MCP handler (command tier) → engine belts (leaf/engine); no
  back-edge.
- **Behaviour preservation.** The existing import/funnel suites stay green unchanged;
  the subprocess arm is untouched.

## 4. Guiding Principles

Smallest coherent foundation: **one** MCP tool, **one** lint, **one** import-path
switch — each over an existing seam. `worker_commit` does purely *local* work on the
worker's own tree; everything cross-boundary stays with the (unchanged) orchestrator.

## 5. Proposed Design

### 5.1 System Model

```
 worker (jailed, cwd=worktree, .git RO)
   │  calls mcp__doctrine__worker_commit { agent, message }   # agent = OPAQUE id (worktree name), NO path
   ▼
 doctrine MCP server (UNCONFINED, root=PRIMARY, .git RW)
   │  sanitise(agent) → git worktree list --porcelain → filter live dispatch/<NNN> coord trees
   │  → probe each <coord>/…/jail/<agent>.toml → EXACTLY ONE live hit ⇒ record {dir,branch,base,coord}
   │  belt: check commit (in dir) → scope classify → one non-merge commit parent==B
   ▼
 worker worktree HEAD advances B → C   (C = one commit, C^ == B)
   │  (worker returns)
   ▼
 orchestrator (main thread, unchanged actor)
      learns name post-return via basename(worktreePath) (create.rs:70)
      verify-worker --dir --branch (C descends B ∧ branch==dispatch/<name> it armed)
      → run_import FORK path (commit, not diff) → commit onto dispatch/<slice>
      → reap (DELETES the per-worktree record)
```

### 5.2 Interfaces & Contracts

**MCP tool** (`src/mcp_server/tools.rs` registration + `src/mcp_server/*.rs` handler):

```
name: "worker_commit"
input:  { agent: string (required),    # worker's own agent-id / worktree name (self-reported)
          message: string (required) } # worker-authored; orchestrator may amend
returns: { "Committed": { oid: string, base: string,
                          undeclared: [string] } }        # or a typed refusal
                          # undeclared = in-scope-committed src paths outside the
                          # design-target selectors (soft warn; orchestrator amends)
```

Handler (imperative shell; unconfined; **opaque-id resolution, no worker-supplied path**),
in order (codex pass-2 shape, §10):
1. **Resolve target from an OPAQUE agent-id (owner ruling, X1 — final).** The worker
   passes only `agent` (its worktree name); it **cannot** name a path. Resolution is
   entirely server-side from the **primary** root:
   a. **Sanitise** `agent` through the *same* validator `create-fork` uses (`sanitise_name`,
      `create.rs:101/108`) — rejects traversal / illegal forms before any path join (fix
      finding X-4).
   b. **Enumerate** active coord trees: `git worktree list --porcelain` from primary
      (`git.rs:1380`), filter to **live** coordination worktrees (branch `dispatch/<NNN>`,
      `!prunable`, `path.exists()` — `inventory.rs:62/192`). No worker input, no new
      registry — the git worktree list *is* the primary-readable coord pointer (X-1).
   c. **Probe** each coord for the per-worktree record
      `<coord>/.doctrine/state/dispatch/jail/<agent>.toml`. Require **exactly one live
      hit**: zero ⇒ refuse `unknown-agent`; **more than one ⇒ refuse `ambiguous-agent`**
      (name uniqueness is not source-guaranteed — X-5/#6). The record carries
      `{name, dir, branch, base=B, coord}` (see §5.3 — written by the trusted create-fork
      hook, X-2/X-5).
   d. **Validate consistency** before trusting it: `dir.exists()`, `branch` resolves,
      worktree `HEAD == base` (pre-commit). Any mismatch ⇒ refuse `stale-record` (a
      record whose worktree was reaped/moved is not a live target — fixes the stale-oracle,
      X-2).
   Residual (accepted, X1): a worker that knows a *sibling's* live `agent` can target its
   record → commit on the sibling's branch. Attribution confusion, review-caught, **own
   work unpromoted**, no escalation. RSK-226 tracks a true caller-binding follow-on.
2. **Gate belt — `check commit`.** Run the resolved `commit` tier
   (`resolve_check(Commit)`) in `dir`. fmt mutates the tree *first*, then lint/validate/
   test/build. Red ⇒ refuse `commit-gate-red` (report captured output). (Replaces a
   separate `prove` on this path; `prove` stays for import/integration.)
3. **Scope belt — two tiers (owner steer 2026-07-04).** `classify_import` of the
   pre-fmt intended delta vs the slice's `design-target` selectors, split by zone:
   - **Hard reject (`forbidden-zone`).** Writes to escalation surfaces — `.doctrine/**`,
     `.claude/**`, `.agents/**`, `install/agents/**` (governance state, plans, selectors,
     agent-def tool-grants), and build/gate config (`justfile`, `flake.nix`, CI). A worker
     writing these is always wrong (it could rewrite its own scope, tool-grant, or gate) ⇒
     refuse the commit. This is the security fence.
   - **Soft warn (`undeclared`).** Src paths outside the current `design-target` selectors
     but in no forbidden zone. The planner under-declares (leaves off a file that
     legitimately needs the change), so this **does not block** — the commit lands and the
     paths ride back in the result as `undeclared: [paths]`. The orchestrator amends the
     selectors (bless) or rejects at import; either way it feeds the existing audit-time
     `slice conformance` delta rather than pre-empting it. Same audit-caught class as the
     sibling-spoof residual (X1) — within the locked threat model.
4. **Commit invariant.** Assert worktree `HEAD == B` (pre-commit); if `HEAD != B` the
   tree is resumed/stacked ⇒ refuse `not-at-base`. Stage the in-scope delta; create
   **exactly one non-merge commit** with `message`; the new tip `C` then satisfies
   `C^ == B`, `parents(C) == 1`.
5. Return `{ oid: C, base: B }`.

**Conformance lint** — a `doctrine` verb (wired into `just gate`/`just check`) scanning
`.claude/agents/*.md`. For every agent-def carrying the frontmatter marker
`doctrine-role: worker` (or `orchestrator`, for SL-199 reuse), assert its `tools:` list
contains **no `mcp__*` token except `mcp__doctrine__worker_commit`** and **no bare
`mcp__doctrine` server grant**. Any other MCP token ⇒ lint failure with the offending
token + file. Allowlist semantics — no writability introspection of third-party
servers required.

### 5.3 Data, State & Ownership

- **Per-worktree dispatch record (NET-NEW, load-bearing — X-2/X-5).** `create-fork`
  (trusted, pre-worker, harness-invoked hook) writes an atomic record at
  `<coord>/.doctrine/state/dispatch/jail/<name>.toml` carrying
  `{name, dir=<coord>/.worktrees/<name>, branch=dispatch/<name>, base=B, coord}`. It
  already knows all of these at the fork point (`create.rs:276`) and already has
  `write_atomic` (`create.rs:257`); it already reads `base` from the arming slot to fork.
  This **supersedes D4** (base from the live arming slot at commit time — racy) and the
  earlier "beside the jail *policy*" framing: the record is the single source of truth
  `worker_commit` consumes, and `create.rs:205`'s "GC'd with teardown" comment becomes
  *true* only once reap deletes it (below).
- **Record lifecycle (fixes the stale-oracle, X-2 / finding #2).** create-fork **writes**
  it; **reap/gc deletes** it when removing the worker worktree (net-new deletion step in
  `gc.rs` — today `gc.rs:36/203` reap worktree+branch only). Presence-plus-consistency,
  not presence alone, is the target-fence (§5.2 step 1d).
- `worker_commit` **writes only** the resolved agent's worktree HEAD (B→C on its
  `dispatch/<name>`). It never touches `dispatch/<slice>`, `.doctrine/` authored state, or
  any shared ref — sole-writer + ADR-012 preserved.
- The lint is pure static analysis of authored agent-def files; no runtime state.

### 5.4 Lifecycle, Operations & Dynamics

- **Order matters:** gate (fmt mutates) → stage → commit. Committing before fmt would
  bake unformatted trees.
- **Pre-dispatch fmt-clean trunk (owner steer 2026-07-04).** The orchestrator runs `fmt`
  on trunk/main **before arming**, so base **B** is fmt-clean at fork. Then `check commit`'s
  fmt only ever touches worker-changed files and the post-fmt diff equals the in-scope
  delta — F2 does not arise in normal operation. The "stage only the pre-fmt classified
  paths" invariant (§5.5) remains the belt-and-suspenders guarantee that holds even if this
  ritual is skipped. Add to the CLAUDE.md `# orchestration` pre-dispatch ritual.
- **Warm target.** `check commit` runs `test`+`build` in `dir`; the fork must be
  reflink-prewarmed ([[mem.pattern.dispatch.prewarm-fork-target-reflink]]) or the build
  times out. (Pre-warm is orchestrator setup, already in the funnel; note the
  dependency.)
- **Orchestrator import switch (claude arm only).** Swap `--from-worktree` (diff) for
  `run_import`'s **fork/commit** path — the worker HEAD is now a commit. Reuse the
  existing branch; no new projection code. `run_verify_worker`'s expectation relaxes
  from `HEAD==B` to `HEAD is one non-merge commit, HEAD^==B` (exact delta → plan).
  The orchestrator imports the branch **it armed** (`--branch dispatch/<name>`,
  `subagent.rs:316`) — this is what makes the X1 residual bounded: a poisoner who spoofed
  a sibling's `agent` committed to the *sibling's* branch and left **its own** branch at
  B, so the orchestrator (importing the branch it spawned) promotes nothing of the
  poisoner's (finding X-3).
- **Subprocess arm** keeps `--from-worktree` + its stdio-jail import; no change.
- **CLAUDE.md `# orchestration`** note updated: worker self-commits (claude arm); the
  import-the-diff dance is retired *there*, retained for the subprocess arm + as the
  MCP-down fallback.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1.** `worker_commit` produces exactly one non-merge commit with `C^ == B`, or
  refuses. No stacks, no merges, no fast-forward past B.
- **INV-2.** Every write path in `worker_commit` is belt-gated server-side; a worker
  cannot bypass a belt by not running it.
- **INV-3 (lint).** A marked worker/orchestrator def exposes no writable MCP surface
  beyond `worker_commit`.
- **INV-4 (resolution).** `worker_commit` accepts only an opaque, sanitised `agent` id;
  the target is resolved server-side and refuses on `unknown-agent` (0 hits),
  `ambiguous-agent` (>1 live hit — name uniqueness not assumed), or `stale-record`
  (record ↔ worktree inconsistent). No worker-supplied path enters a filesystem join.
- **INV-5 (F2 — commit stages the classified set, not the post-fmt diff).** The commit
  includes exactly the pre-fmt in-scope classified paths. A repo-wide `check commit` fmt
  that normalises a pre-existing/out-of-scope file cannot drag it into the commit. Holds
  independently of the pre-fmt-clean-trunk ritual (§5.4) — that ritual makes the case not
  arise; this invariant makes it safe if it does.
- **INV-6 (scope tiers).** A write to an escalation zone (`.doctrine/**`, `.claude/**`,
  `.agents/**`, `install/agents/**`, build/gate config) hard-refuses (`forbidden-zone`).
  A src write outside the `design-target` selectors but in no forbidden zone commits and
  is reported as `undeclared` — never silently dropped, never hard-blocked.
- **Edge — empty delta.** Worker made no in-scope change ⇒ refuse `empty-delta` (no
  empty commit).
- **Edge — gate mutates then still red.** fmt reformats but lint/test fails ⇒ refuse;
  the reformat is left in the worktree (harmless — orchestrator reaps, or worker retries).
- **Edge — `dir` not a worktree / wrong project.** Refuse via the `verify-worker`
  fact-gather (`no-worker-head` class).
- **Assumption.** Claude-arm MCP-write bypass holds across harness upgrades
  (version-sensitive; re-probe on upgrade — [[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]]).

## 6. Open Questions & Unknowns

- **OQ-1 — lint host + marker.** Exact verb/command wiring (`doctrine check` sub-check
  vs a dedicated conformance verb) and whether the marker is a new frontmatter key vs a
  `doctrine.toml` worker-list. Lean: frontmatter marker + gate-wired verb. → plan.
- **OQ-2 — `run_verify_worker` delta.** How much the base==B assertion relaxes for the
  post-commit (HEAD==C) claude-arm case without disturbing the subprocess arm's use. →
  plan/execute.
- **OQ-2b — `run_import` fork-source shape (from F1).** Does the fork path accept a
  detached worktree-HEAD oid, or need a named ref (⇒ cut `phase/<slice>-NN` at the
  worker HEAD per ADR-012 D3)? Gates the "reuse, no new code" claim. → plan.
- **OQ-3 — message stamping.** Whether to prefix a server-side `slice/phase` tag for
  funnel legibility, or leave fully worker-authored (orchestrator amends). Low stakes.

## 7. Decisions, Rationale & Alternatives

- **D1 — Shape 1: commit on the worker's own HEAD; orchestrator imports the commit.**
  *Chosen.* Keeps `worker_commit` purely local; projection stays the orchestrator/sync
  step (ADR-012 D1/D4, ADR-006). Reuses `run_import` fork path.
  *Rejected — Shape 2* (server projects onto `dispatch/<slice>`/`phase` per worker):
  violates ADR-012's no-per-worker-projection + sole-writer.
- **D2 — Gate = `check commit` (the `commit` tier), server-enforced in `dir`.** *Chosen*
  (owner). Forces fmt + full lint/test/build at the commit boundary; subsumes `prove`
  on this path. Heavy (test+build per commit) but project-configurable; "stick with it,
  see how it plays" (owner). *Rejected* — lighter `prove`-only gate: weaker guarantee.
- **D3 — Lint rule = MCP allowlist (only `worker_commit`), not writability
  classification.** *Chosen.* Enforceable without introspecting third-party servers;
  strictly safer (deny-all-MCP-except-one). *Rejected* — "no *writable* MCP tool":
  can't classify arbitrary servers.
- **D4 — base B from the arming slot + assert `HEAD==B`.** *Chosen.* Authoritative B
  independent of HEAD catches the resumed-stack case; HEAD==B holds pre-commit. Owner
  deferred ("idk"); rationale as recorded. *Alt* — fork-point `merge-base`: redundant
  under D4.

## 8. Risks & Mitigations

- **R1 — heavy commit gate latency / cold-build timeout.** `check commit` runs test+build
  per worker commit. *Mitigation:* reflink-prewarm the fork (existing funnel step); tier
  is `[verification]`-configurable. Accepted for now (D2).
- **R2 — lint marker not adopted by a project's custom worker def.** An unmarked worker
  escapes the invariant. *Mitigation:* doctrine ships the marked `dispatch-worker`;
  document the marker requirement; SL-199 orchestrator uses the same. Residual: a
  project authoring an unmarked worker — a documentation/《known-gap》, not a regression.
- **R3 — belt drift between `worker_commit` and the import path.** Two callers of the
  same belts could diverge. *Mitigation:* call the *same* functions (`classify_import`,
  the tier runner, the commit-invariant check); no forked copies (behaviour-preservation
  gate — existing suites stay green).
- **R4 — MCP soft-dependency.** If the server is down the worker can't self-commit.
  *Mitigation:* the main-thread import-the-diff path remains as fallback (retired only
  from the happy path, not deleted).

## 9. Quality Engineering & Validation

TDD, behaviour-preserving. Key cases:
- **worker_commit belts (unit/e2e):** refuse `commit-gate-red`, `undeclared-scope`
  (a `.doctrine/`/`.claude/` write; an out-of-selector path), `not-at-base` (HEAD≠B
  stack), `empty-delta`, merge/multi-commit; **accept** a clean in-scope delta →
  exactly one commit `C^==B` (assert at the commit grain, not tip-equality —
  [[mem.pattern.dispatch.prepare-review-rerun-not-idempotent-until-gate]] cousin).
- **jailed-worker end-to-end (claude arm):** a worker whose raw Bash `git commit` is
  walled self-commits via `worker_commit`; the delta lands as one commit; orchestrator
  imports the *commit*.
- **lint:** fails a def granting a second `mcp__*` tool / bare server grant; passes the
  pinned `dispatch-worker`.
- **behaviour preservation:** existing import + subprocess-arm funnel suites green
  unchanged.
- **belt reuse:** assert `worker_commit` and `import` share the scope/gate verdict on
  the same input (no divergence).

## 10. Review Notes

### Internal adversarial pass (2026-07-04)

- **F1 (sharpest — reuse may not hold).** §5.4 claims the orchestrator imports the
  worker commit via `run_import`'s **fork path** `(Some(fork), None)`. But that path
  likely expects a **branch ref**, whereas the claude-arm worker HEAD is a **detached**
  commit (baseRef=head fork). Must verify `run_import`'s fork source accepts a worktree
  detached-HEAD oid; if it needs a named ref, either cut an ephemeral `phase/<slice>-NN`
  ref at the worker HEAD (ADR-012 D3 already cuts `phase/<slice>-NN` from
  `dispatch/<slice>` at sync on the Agent arm — align with that) or extend the fork
  path to take an oid. **Promoted to OQ-2b; resolve at plan before committing to
  "reuse, no new code."**
- **F2 (edge).** `check commit` runs `fmt` (mutating) *before* the scope belt. If base
  B carried a pre-existing mis-formatted file the worker didn't touch, fmt rewrites it
  → `classify_import` flags a false `undeclared-scope`. Low risk (B is formatted by its
  own prior gate) but real. Mitigation: run the scope belt on the worker's *intended*
  change-set (diff vs B **excluding** fmt-only hunks in untouched files), or assert B
  is fmt-clean at fork. → plan.
- **F3 (concurrency, mostly SL-199).** Concurrent `worker_commit` calls act on
  **distinct** linked worktrees (separate index + HEAD files; git object db writes are
  concurrent-safe), so they don't corrupt each other. Whether the MCP server serializes
  tool calls is an SL-199 (parallel-orchestrator) concern; SL-198's main-thread arm is
  effectively serial. Recorded as an assumption, not a blocker.
- **F4 (governance — needs a ruling).** SL-198 **rides** the witnessed passthrough and
  **un-jails nothing**, so it needs no ADR-012/ADR-011 REV (those are SL-199 / Mode A).
  BUT: granting a worker a *sanctioned* writable MCP tool is a deliberate hole in the
  SL-182/ADR-008 confinement premise, made safe by the lint. Is that a **note** on
  ADR-008/SL-182 (recommended) or does it rise to an amendment? Flagged for the external
  reviewer + `/consult` if contested. Lean: a note, ratified at reconcile.
- **F5 (lint parsing).** The lint assumes `.claude/agents/*.md` YAML frontmatter with a
  parseable `tools:` list + the new `doctrine-role:` marker. Confirm the agent-def
  frontmatter schema (list vs comma-string) at plan; the marker is a new doctrine
  convention shipped on `dispatch-worker`.

### External adversarial pass (codex / GPT-5.5, 2026-07-04) — all findings source-verified

This pass **amends** the sections it names; the body §§5–8 are reconciled at `/plan`.
Net reframe: the design's "one import switch, mostly reuse" framing was **wrong** — the
reuse is real (F5), but the true net-new work is **trusted per-worktree base/target
plumbing** (X1/X2) + a **non-fail-open lint** (X3).

- **X5 (de-risk — supersedes OQ-2, OQ-2b) — the reuse claims HOLD (source-verified).**
  `run_import`'s `fork` is a **revspec** — resolves `{fork}^^{commit}`, diffs
  `{base}..{fork}`, `git apply`s into the coord index non-committing, and already
  requires `S^==B` single-non-merge (`import.rs:301`, `:323`, `:354`). A **detached
  worktree-HEAD oid works as the fork revspec** — the claude-arm switch from
  `--from-worktree` to the fork path is clean reuse, **no new code**.
  `run_verify_worker` already asserts `merge-base --is-ancestor B HEAD` (**not**
  `HEAD==B`; `subagent.rs:360`), so the post-commit `HEAD==C, C descends B` case passes
  **unchanged**. ⇒ OQ-2 and OQ-2b are **closed YES**; delete them from the plan's risk.

- **X2 (BLOCKER → adopt) — base B must be snapshotted per-worktree at spawn, not read
  from the live arming slot.** The arming slot (`ARMING_SUBPATH/base`, `create.rs:194`)
  is a **single per-coord slot overwritten on re-arm** (`dispatch.rs` arm-spawn); reading
  it at *commit* time means a later spawn silently redefines B for an in-flight worker
  (parallel fan-out corrupts; serial overlap is brittle). **Fix:** at `create-fork`,
  snapshot B into per-worktree trusted state beside the jail policy
  (`JAIL_SUBPATH/<name>.toml` pattern — outside the fork, ro to the worker,
  `create.rs:198`, `:216`). `worker_commit` reads **that immutable per-worktree base**,
  not the arming slot. Supersedes §5.2 step 1 and D4. **This is load-bearing net-new
  work** (touches `src/worktree/create.rs`).

- **X1 (BLOCKER → RESOLVED by owner ruling — registry-keyed target, residual accepted).**
  The MCP server entrypoint gets `name`+arbitrary JSON args (`tools.rs:395`, `:408`) —
  **no caller `agent_id`**, and a worker's `Read` passes the wall so **no on-disk secret
  is safe** either. So true caller-auth ("this caller owns this worktree") is **not
  achievable in this harness**. **Owner ruling (2026-07-04):** don't accept a
  worker-supplied `dir` at all — the worker passes an **`agent` id (its worktree name)**,
  and the server **looks up** the worktree from the per-worktree registry
  (`JAIL_SUBPATH/<agent>.toml`, present ⟺ legitimately spawned; `dir`, `base`, `branch`
  all derive from that one key — §5.2 step 1). The worker **cannot freely specify** a
  path; the identifier must hit the registry. **Residual (accepted, small blast radius):**
  a poisoned worker that knows/guesses a *sibling's* registered name can land an in-scope,
  gate-passing commit onto that sibling's `dispatch/<name>` branch — attribution
  confusion, review/audit-caught, **not** a jail escape or escalation; **and the
  poisoner's own work is not promoted** (the orchestrator imports the branch it spawned).
  This aligns with the locked threat model (belts bound blast radius, not intent). A true
  caller-binding (harness `agent_id`→server channel) is the follow-on — **RSK-226**.

- **X3 (BLOCKER → adopt) — the lint targets the wrong files and fails open.** Worker
  defs live at `.doctrine/agents/dispatch-worker.md` + `install/agents/{claude,pi}/
  dispatch-worker.md` (source-verified) — **not** `.claude/agents/*.md` (that is the
  installed copy). And R2 accepting an unmarked worker as a "doc gap" is **fail-open**.
  **Fix:** lint the **authored + install** trees; make the `doctrine-role` marker
  **mandatory** — an unmarked def used as a dispatch worker is a **lint failure**, not a
  doc gap (deny-by-default). Supersedes §5.2 (lint target), §7 D3, and R2. Update the
  `design-target` selectors to `.doctrine/agents/**` + `install/agents/**`.

- **X4 (MAJOR → adopt) — belt order is backwards.** `check commit` mutates (`fmt`)
  *before* scope + `HEAD==B` refusal, so stacks/empty/out-of-scope pay the full
  test+build, and `fmt` can widen the touched-path set before classification (the
  general form of F2). **Fix order:** target-fence → immutable-base resolve → `HEAD==B`
  → non-empty **pre-fmt** intended delta → **pre-fmt** scope belt → *then* the mutating
  `check commit` gate → commit. Supersedes §5.2 belt order; subsumes F2.

**Plan consequences:** the real slice is (1) `create-fork` base snapshot + trusted
per-worktree base read (X2), (2) `worker_commit` MCP tool with the reordered cheap-first
belts over that base + reused `classify_import`/fork-import (X1/X4/X5), (3) a mandatory,
correctly-targeted conformance lint (X3). The "import-path switch" is the *small* part.

### External adversarial pass 2 (codex / GPT-5.5, 2026-07-04) — X1 resolution hardened

Pass-1's X1 resolution (worker passes `agent`, server "recovers the coord root") **hand-waved
the coord-root seam** and left the record lifecycle/namespace loose. Pass-2 attacked exactly
that; all claims re-verified against source. Outcome — the X1 mechanism is now concrete and
**achieves the owner ruling literally** (opaque id, no worker path). This block **supersedes**
the loose parts of pass-1 X1/X2 (§5.2 step 1, §5.3); the finding numbers below are pass-2's.

- **X-1 (BLOCKER → resolved) — the server has no coord root; my §5.2 hand-waved it.** MCP
  `root` is fixed at startup to the **primary** tree (`mcp_server/mod.rs:26`), tools/call gets
  no cwd/agent_id (`tools.rs:395/408`); the coord tree ≠ primary. **But** an existing
  primary-readable pointer already enumerates coord trees: `git worktree list --porcelain`
  (`git.rs:1380`) → filter live `dispatch/<NNN>` (`inventory.rs:62/192`). **No new coord
  registry needed** (my proposed setup-written registry is redundant). Server discovers coord
  trees itself, then probes each for `jail/<agent>.toml`. (X-2's "orchestrator/setup registers
  coord" and pass-1's worker-supplied anchor are both **dropped** in favour of this.)
- **X-2 (BLOCKER → adopt, supersedes pass-1 X2/D4) — one trusted per-worktree record with a
  real lifecycle.** create-fork (trusted, pre-worker) writes `jail/<name>.toml` =
  `{name, dir, branch, base=B, coord}` atomically (`create.rs:257/276`); **reap/gc deletes
  it** (net-new `gc.rs` step — today `gc.rs:36/203` reap worktree+branch only). Presence
  **plus** consistency (`dir.exists()`, branch resolves, `HEAD==base`) is the fence — kills
  the stale-file oracle. Base moves *into* this record (was arming-slot-only, `dispatch.rs:445`).
- **X-3 (MAJOR → adopt) — bound the residual with `--branch`.** The "own work unpromoted"
  claim now cites its seam: the orchestrator imports the branch **it armed**
  (`verify-worker --branch dispatch/<name>`, `subagent.rs:316`), so a poisoner's own branch
  (left at B) promotes nothing. §5.4 updated.
- **X-4 (MAJOR → adopt) — one name validator, no traversal.** `agent` is sanitised through
  the *same* `sanitise_name` create-fork uses (`create.rs:101/108`) before any path join.
- **X-5 (MAJOR → adopt) — refuse on ambiguity, not assume uniqueness.** "Globally unique
  name" is **not** source-proven (`create.rs:213` says only "unique slug"); >1 live hit ⇒
  refuse `ambiguous-agent`. Cross-slice concurrent coords make this real (`dispatch.rs:3099`);
  same-slice duplication already refused (`coordinate.rs:67/163`). Subprocess arm unaffected —
  `worker_commit` is claude-arm-only; subprocess keeps `--from-worktree` (`mod.rs:243/249`).

**Verdict:** codex — **LOCK-READY-WITH-THIS.** Net new `design-target`: `src/worktree/gc.rs`
(record deletion). Plan consequences unchanged in shape; step (1) now = "create-fork writes the
`{name,dir,branch,base,coord}` record + gc deletes it", step (2)'s resolver = git-worktree-list
enumerate → probe → one-live-hit → validate.
