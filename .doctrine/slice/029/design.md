# SL-029 — Worktree lifecycle: detection, creation ladder, provisioning, guards

Canonical technical design. Governs IMP-003's **lifecycle half**; the orchestrator
funnel half is split to a follow-up slice (OQ-1). Governing policy: **ADR-006**
(D1/D5/D6a/D9 mechanised here; D2/D6/D7 land in the funnel slice). Prereq IMP-002
is assumed, not built (and is **not** depended on by this slice — see §1).

## Resolved design questions

- **OQ-1 (altitude / sprawl) → slice-split.** SL-029 owns the worktree
  *lifecycle* + the `/execute` optional-isolation path: **solo, no funnel, no
  IMP-002 dependency.** The orchestrator funnel (import→verify→commit→record,
  worker-vs-solo D6a-ON, `/dispatch` ships) is a separate follow-up slice that
  *does* depend on IMP-002. Rationale: the two units have different dependency
  profiles; the lifecycle ships standalone value (solo `/execute` isolation works
  today) and carries the testable Rust, so splitting unblocks SL-029 from the
  IMP-002 sequencing gate and keeps each slice small.
- **OQ-3 (CLI vs skill boundary) → invariant + provision copy in Rust.** The CLI
  owns the exclusion-invariant assertion *and* the allowlist-driven copy (one
  seam, so the copy physically cannot leak the coordination tier). Detection,
  ladder rung-selection, branch-point check, baseline-verify stay skill-prose.
- **OQ-2 (framework-neutral fallback) → resolved by OQ-3.** `doctrine worktree
  provision` is the framework-neutral copy rung: it reads the *same*
  `.worktreeinclude` that Claude Code's native `WorktreeCreate` hook reads, so a
  non-Claude harness gets byte-identical provisioning. Creation fallback (`git
  worktree add`) and regenerate (`cargo build`) are explicit skill-prose / project
  commands.
- **Skill home → standalone `/worktree` skill.** Both `/execute` (optional) and
  the future `/dispatch` funnel (mandatory) invoke it. DRY / single
  responsibility; mirrors the `superpowers:using-git-worktrees` prior-art shape;
  `/route` is unchanged (isolation is a sub-mechanism of execution, not a
  top-level intent).
- **`provision` scope → copy axis only.** Regenerate (`cargo build`) +
  baseline-verify are the same project command the skill runs; no reason to
  duplicate them in Rust.

## §1 Current vs target behaviour

**Current.** `/dispatch` is a placeholder. `/execute` runs serial, in-tree, with
no isolation. No worktree machinery exists anywhere; there is no `.worktreeinclude`.

**Target** (solo, no funnel, no IMP-002 dependency):

- New **`/worktree`** skill owns the lifecycle: detect → create-ladder →
  provision → baseline → guards.
- **`/execute`** gains a thin *optional* isolation branch delegating to
  `/worktree`; the default in-tree path is untouched.
- CLI: **`doctrine worktree provision <fork>`** (allowlist copy, exclusion
  enforced at the copy seam) and **`doctrine worktree check-allowlist`** (static
  exclusion assertion).
- A default **`.worktreeinclude`** *template* is documented by the skill; the
  installer ships no root-level file (§3, F2).
- `/dispatch` stays a placeholder (the funnel slice fills it).

This slice does **not** depend on IMP-002: the only execution mode here is solo
(`/execute` isolation, worker-mode OFF, D6a), and the work-in-place fallback rung
is the already-blessed D1/D6a trunk path. Worker-mode enforcement (D2a) only
matters once the funnel dispatches *workers* — the follow-up slice. **Solo
`/execute` isolation never mints ids** (the slice already exists; it writes
status/notes/phase/memory, not new entities) — the one act that would pull in
trunk-ref minting (D3 / IMP-002). The no-dependency claim holds at the one place
it could break.

## §2 Detection + creation ladder (skill-prose, framework-neutral)

**Detection.** `git rev-parse --git-dir` ≠ `git rev-parse --git-common-dir` ⇒ the
CWD is already inside a linked worktree (isolation already present — adapt, don't
re-create; D1). **Submodule guard:** a submodule *also* trips that inequality;
disambiguate by the `.git` gitdir pointing under `worktrees/` (a worktree) vs
`modules/` (a submodule), or treat a non-empty `git rev-parse
--show-superproject-working-tree` as "submodule, not worktree". (superpowers
prior art.)

**The creation ladder — four rungs, degrade in order:**

1. **Detect existing isolation** (D1) — already forked → skip creation, go straight
   to provision + baseline.
2. **Native harness tool** — Claude Code `WorktreeCreate` hook, fed by
   `.worktreeinclude`; the hook performs the copy itself. **Opportunistic, not
   assumed (F1):** this hook is sourced to a GitHub *discussion* (mattbrailsford
   #54), not a confirmed-shipped feature. Treat rung 2 as a fast-path taken only
   when the hook is actually present; **never** design around its existence.
3. **`git worktree add <path> <branch>`** (framework-neutral fallback) → then
   `doctrine worktree provision <fork>` performs the allowlist copy. **This is the
   blessed default and the tested path** — the one rung guaranteed present in any
   git harness. Rung 2 is merely an optimisation over it.
4. **Work-in-place** (solo, no funnel) on sandbox denial — the already-blessed
   D1/D6a trunk path. No fork, nothing to provision.

Rung 2 provisions natively; rung 3 provisions via the CLI (the neutral, tested
path, OQ-2); rung 4 has nothing to provision. **The same `.worktreeinclude` feeds
rungs 2 and 3** — byte-identical provisioning across harnesses, so falling from
rung 2 to rung 3 changes *who* copies, never *what* is copied.

## §3 The CLI heart — `provision` + `check-allowlist`

New module **`src/worktree.rs`**, ADR-001 leaf layering, pure/imperative split
(no disk/git in the pure core — passed in as inputs).

### Pure core

```rust
// The load-bearing constant: the coordination/runtime tier that must never enter
// a fork (partition memory mem.concept.dispatch.gitignored-tier-partition).
// Hardcoded; a test cross-checks it against the runtime-tier globs in .gitignore /
// memory-spec so drift is caught (not derived at runtime — avoids a parse surface
// and a runtime failure mode).
const COORDINATION_GLOBS: &[&str] = &[
    ".doctrine/state/",      // phase sheets, boot.md
    "phases",                // the per-slice symlink into the state tree
    "handover.md",           // disposable agent context
    // memory caches (index / embeddings / state) — exact globs pinned vs memory-spec
];

struct Allowlist { patterns: Vec<Pattern> }            // gitignore syntax
fn parse_allowlist(text: &str) -> Allowlist;
fn exclusion_violations(a: &Allowlist) -> Vec<Violation>;   // patterns *naming* the tier → static reject
fn select_copies(a: &Allowlist, candidates: &[RelPath]) -> Selection; // { copy, skipped_excluded }
```

### Impure shell

**Candidate set = gitignored files only (F5).** Committed files are already in the
fork (it forks HEAD); provision's domain is the *irreducible gitignored* tier.
Enumerate via `git ls-files --others --ignored --exclude-standard` (mirroring the
`src/git.rs` subprocess seam); `select_copies` filters them; copy each into the
fork preserving its relative path (extend `src/fsutil.rs` with a recursive,
`safe_join`-guarded copy that **does not follow out-of-tree symlinks**).

**Invocation context (F4).** `provision` runs from the **source (main-tree)
root** — auto-detected via `root::find` from CWD — and writes into the `<fork>`
destination argument. It must *not* be run from inside the fresh fork (root
detection would resolve to the fork, not the source). The skill's rung-3 prose
runs it from the main tree immediately after `git worktree add`.

### Two-layer exclusion (the OQ-3-B payoff — defense in depth)

- **`check-allowlist` (static smell test):** refuses any *pattern* that names the
  coordination tier. Catches the obvious mistake early; CI / `validate`-usable.
  This is the ADR-006 Verification "allowlist asserted to exclude the
  coordination/runtime globs" bullet, made testable.
- **`select_copies` (copy-time, the real guard):** drops any *file* under the tier
  **even when matched by a broad `*` / `**` pattern that `check-allowlist` cannot
  statically reject.** The copy therefore *physically cannot* leak
  `.doctrine/state/` — the invariant is enforced at the copy, not merely asserted
  alongside it. **Behaviour on a broad-pattern tier hit: skip + warn** (name the
  withheld file, exit 0) — a wildcard allowlist stays usable and the operator is
  told what was withheld.

### Verbs

- **`doctrine worktree provision <fork>`:** read `.worktreeinclude` from the source
  root → run the exclusion check (**fail closed** on a statically-bad allowlist) →
  resolve matching files → copy, dropping (skip+warn) any tier file → report
  copied / withheld.
- **`doctrine worktree check-allowlist`:** parse + assert; nonzero exit on a
  pattern that names the tier. **Green here is *not* a completeness guarantee
  (F7):** it only proves no pattern *names* the tier — a broad `**` can still
  sweep tier files, which `select_copies` withholds at copy time. `check-allowlist`
  is the early smell test; `select_copies` is the guarantee.

### The `.worktreeinclude` (no installed root file — F2)

Doctrine itself has no secrets / irreducible local files, so the default is
**nothing to copy**. Rather than ship a root-level file via the installer (a new,
clobber-prone manifest pattern — it could overwrite a consuming project's
existing `.worktreeinclude`), **`provision` tolerates an absent `.worktreeinclude`
as an empty allowlist** (copy nothing; safe out of the box, D1 policy-agnostic).
The `/worktree` skill *documents* a commented template the project can adopt;
the file is project-owned, never installer-managed.

## §4 Guards (skill-prose, `/worktree`)

- **Commit-before-spawn (D5).** `/worktree` refuses to fork with doctrine-relevant
  uncommitted changes; the fork sees only committed HEAD. Prose: require a clean
  (or at least HEAD-current) tree before rungs 2/3. **In scope** — fork-cleanliness
  applies even to solo isolation.
- **Branch-point check (D5) — deferred to the funnel slice (F3).** The check (HEAD
  pre-spawn == worktree HEAD post-spawn) guards a *spawn where a concurrent actor
  could move HEAD*. Solo `/execute` isolation forks and works in-place with no
  concurrent mover, so the check is near-vacuous here. It becomes load-bearing only
  under the funnel's concurrent dispatch — it lands there, with the worker contract
  it actually protects.
- **Baseline-verify (D9).** After provision, run the project's regenerate + test
  command (`cargo build && cargo test`) in the fork; a green gate before handoff.
  Project command (VA/VH), not Rust. An unbuildable fork is fixed in provisioning,
  never handed off.

## §5 `/execute` isolation thread + scope reconciliation

- **`/execute` thread.** Add an optional branch: *isolation requested? → invoke
  `/worktree` (fork + provision + baseline) → execute inside the fork → the fork
  branch is the deliverable.* **Solo: worker-mode OFF (D6a)** — `/execute` is a
  full agent and its own orchestrator; it writes its own doctrine state directly
  (slice status, notes, phase, memory). The default no-isolation path is unchanged.
- **Isolation trigger is explicit (F8).** Isolation is *opt-in*, never automatic:
  the agent requests it (user instruction, or a plan/phase annotation). The
  default `/execute` stays in-tree, so the common path is untouched and the new
  branch is dead code until deliberately invoked.
- **Squash-orphan caveat (ADR-006 Open / Negative).** Memory recorded inside a
  worktree branch is orphaned by a squash-merge (content survives; SL-008
  staleness fires). `/worktree` / `/execute` prose nudges record-on-trunk for
  durable memory.
- **Scope reconciliation.** `slice-029.md` is narrowed to the lifecycle + the
  `/execute` path; the funnel / `/dispatch` deliverables move out; affected
  surface becomes `.doctrine/skills/worktree/` + `src/worktree.rs` + CLI + the
  `/execute` thread (no installed `.worktreeinclude`, F2); a follow-up records the
  new funnel slice (the OQ-1 split half, IMP-002-dependent).

## §6 Verification alignment

**ADR-006 Verification bullets SL-029 discharges:**

- *allowlist excludes the coordination/runtime globs* → `check-allowlist` +
  `select_copies` tests (VT).
- *baseline build+test passes in the fork before dispatch* → `/worktree` baseline
  step (VA).
- *tier merge-safety (D4): the `phases` symlink is relative* → existing invariant;
  this slice introduces no shared-mutable authored file (VT, no regression).

**Test cases (SL-029):**

- *pure:* `parse_allowlist` round-trip; `exclusion_violations` canary (a pattern
  naming `.doctrine/state/` is rejected); **`select_copies` drops a
  `.doctrine/state/...` file under a `**` allowlist** (the load-bearing case);
  `COORDINATION_GLOBS` cross-check vs `.gitignore` / memory-spec.
- *impure / e2e:* `provision` copies an allowlisted file into a fork; withholds a
  tier file with a warning (exit 0); `check-allowlist` exits nonzero on a
  statically-bad allowlist.

**Deferred to the funnel slice (not verified here):** worker-mode guard
(D2a / IMP-002); funnel order (D7); `memory record` worktree-warn; the
branch-point check as an orchestrator assertion under dispatch.

## §7 Affected surface

- `.doctrine/skills/worktree/SKILL.md` — **new**: the lifecycle skill (detection,
  ladder, guards, baseline prose; invokes the CLI verbs).
- `src/worktree.rs` — **new**: pure core (`Allowlist`, `COORDINATION_GLOBS`,
  `exclusion_violations`, `select_copies`) + impure shell (`provision`).
- `src/main.rs` — new `Worktree { Provision, CheckAllowlist }` subcommand.
- `src/fsutil.rs` — recursive `safe_join`-guarded copy helper.
- `.doctrine/skills/execute/SKILL.md` — thin optional-isolation thread.
- `.worktreeinclude` — **not installed** (F2): project-owned; `provision` tolerates
  absence; the `/worktree` skill documents the template.
- `install/manifest.toml` — **untouched** for this file (skills already globbed; no
  root-file row needed).
- `.doctrine/skills/dispatch/` — **untouched** (stays placeholder; funnel slice).

## Open items / carried risks

- **A-1 (carried, downgraded).** IMP-002 is a prereq for the *funnel slice*, not
  for SL-029. SL-029 has no IMP-002 dependency.
- **R-1.** Worker self-verify degradation (D6) is a funnel-slice concern; N/A here.
- **Memory-cache glob precision.** `COORDINATION_GLOBS`' memory-cache entries must
  be pinned against the actual memory runtime layout (memory-spec / `.gitignore`)
  — the cross-check test is the guard.
- **Native rung is unconfirmed (F1).** The Claude Code `WorktreeCreate` hook is a
  GitHub-discussion proposal, not a verified feature. Rung 3 is the tested default;
  if the hook proves absent everywhere, rung 2 is simply never taken — no design
  change, only a dead optimisation. Confirm at `/plan` or first execution.

## Adversarial review log

Internal hostile pass integrated (F1–F8):
- **F1** native rung may be unshipped → rung 3 reframed as the blessed/tested
  default; rung 2 demoted to opportunistic (§2, Open items).
- **F2** root-level `.worktreeinclude` install is clobber-prone → not installed;
  `provision` tolerates absence; skill documents the template (§3, §5, §7).
- **F3** branch-point check is vacuous for solo → deferred to the funnel slice;
  commit-before-spawn retained (§4).
- **F4** `provision` CWD ambiguity → run from source root, `<fork>` is destination
  (§3 Impure shell).
- **F5** candidate set unstated → gitignored files only; no out-of-tree symlink
  following (§3 Impure shell).
- **F6** id-minting could pull in D3 → made explicit that solo `/execute`
  isolation never mints (§1).
- **F7** `check-allowlist` green ≠ completeness → stated; `select_copies` is the
  guarantee (§3 Verbs).
- **F8** isolation trigger unspecified → explicit opt-in, never automatic (§5).
