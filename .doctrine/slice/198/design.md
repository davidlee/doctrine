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
   │  calls mcp__doctrine__worker_commit { dir, message }
   ▼
 doctrine MCP server (UNCONFINED, cwd=primary, .git RW)
   │  belt: check commit (in dir) → scope classify → one non-merge commit parent==B
   ▼
 worker worktree HEAD advances B → C   (C = one commit, C^ == B)
   │  (worker returns)
   ▼
 orchestrator (main thread, unchanged actor)
      verify-worker --dir (C descends B) → run_import FORK path (commit, not diff)
      → commit onto dispatch/<slice> → reap
```

### 5.2 Interfaces & Contracts

**MCP tool** (`src/mcp_server/tools.rs` registration + `src/mcp_server/*.rs` handler):

```
name: "worker_commit"
input:  { dir: string (required),      # the worker's own worktree path (its cwd)
          message: string (required) } # worker-authored; orchestrator may amend
returns: { "Committed": { oid: string, base: string } }   # or a typed refusal
```

Handler (imperative shell; unconfined; operates on `dir`), in order:
1. **Resolve + validate.** Assert `dir` is a linked project worktree (reuse the
   `run_verify_worker` fact-gather). Recover the coord root from `dir` by layout-strip
   ([[mem.fact.dispatch.coord-root-not-git-common-dir]]); read base **B** from the
   arming slot `<coord>/.doctrine/state/dispatch/spawn/base`
   ([[mem.fact.dispatch.single-slot-arming-rendezvous]]).
2. **Gate belt — `check commit`.** Run the resolved `commit` tier
   (`resolve_check(Commit)`) in `dir`. fmt mutates the tree *first*, then lint/validate/
   test/build. Red ⇒ refuse `commit-gate-red` (report captured output). (Replaces a
   separate `prove` on this path; `prove` stays for import/integration.)
3. **Scope belt.** `classify_import` of the staged change-set vs the slice's
   `design-target` selectors; reject `.doctrine/`/`.claude/` + undeclared paths ⇒
   `undeclared-scope`.
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

- `worker_commit` **writes only** the worker's own worktree HEAD (its own branch/detached
  HEAD). It never touches `dispatch/<slice>`, `.doctrine/` authored state, or any shared
  ref — sole-writer + ADR-012 preserved.
- Base **B** is *read* from the arming slot (owned by the orchestrator at arm time); the
  tool does not write it.
- The lint is pure static analysis of authored agent-def files; no runtime state.

### 5.4 Lifecycle, Operations & Dynamics

- **Order matters:** gate (fmt mutates) → stage → commit. Committing before fmt would
  bake unformatted trees.
- **Warm target.** `check commit` runs `test`+`build` in `dir`; the fork must be
  reflink-prewarmed ([[mem.pattern.dispatch.prewarm-fork-target-reflink]]) or the build
  times out. (Pre-warm is orchestrator setup, already in the funnel; note the
  dependency.)
- **Orchestrator import switch (claude arm only).** Swap `--from-worktree` (diff) for
  `run_import`'s **fork/commit** path — the worker HEAD is now a commit. Reuse the
  existing branch; no new projection code. `run_verify_worker`'s expectation relaxes
  from `HEAD==B` to `HEAD is one non-merge commit, HEAD^==B` (exact delta → plan).
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

(pending adversarial pass)
