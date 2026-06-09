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
  *does* depend on IMP-002. The two units have different dependency profiles; the
  lifecycle ships standalone value (solo `/execute` isolation works today) and
  carries the testable Rust. **The split seam is the `/worktree` skill's mode
  contract (§5), defined now so the funnel slice reuses it without re-deciding.**
- **OQ-3 (CLI vs skill boundary) → invariant + provision copy in Rust.** The CLI
  owns the exclusion-invariant assertion *and* the allowlist-driven copy — **the
  sole copy path**, so the copy physically cannot leak the coordination tier.
  Detection, ladder rung-selection, branch-point check, commit-before-spawn,
  baseline-verify stay skill-prose.
- **OQ-2 (framework-neutral fallback) → resolved by OQ-3.** `doctrine worktree
  provision` is **the only provisioner** — same behaviour under any harness. The
  creation backend (native tool vs `git worktree add`) varies; provisioning does
  not. Regenerate (project command) + baseline are skill-run.
- **Skill home → standalone `/worktree` skill.** Both `/execute` (optional) and the
  future `/dispatch` funnel (mandatory) invoke it. DRY; mirrors the
  `superpowers:using-git-worktrees` prior-art shape; `/route` is unchanged.
- **`provision` scope → copy axis only.** Regenerate + baseline-verify are the
  project command the skill runs; not duplicated in Rust.

## §1 Current vs target behaviour

**Current.** `/dispatch` is a placeholder. `/execute` runs serial, in-tree, no
isolation. No worktree machinery exists; there is no `.worktreeinclude`.

**Skill source-of-truth (corrected, B1).** Skills are authored under
**`plugins/doctrine/skills/<name>/SKILL.md`** and *installed* into
`.doctrine/skills/` — which is **gitignored** (`.gitignore:34`, the derived tier).
This slice authors skills in `plugins/`, never in `.doctrine/skills/`.

**Target** (solo, no funnel, no IMP-002 dependency):

- New **`plugins/doctrine/skills/worktree/SKILL.md`** owns the lifecycle: detect →
  create → **provision (always, via CLI)** → baseline → guards.
- **`plugins/doctrine/skills/execute/SKILL.md`** gains a thin *optional* isolation
  branch delegating to `/worktree`; the default in-tree path is untouched.
- CLI: **`doctrine worktree provision <fork>`** (allowlist copy, exclusion enforced
  at the copy seam) and **`doctrine worktree check-allowlist`** (static assertion).
- `.worktreeinclude` is **project-owned, not installed** (§3); absent ⇒ empty.
- `/dispatch` stays a placeholder (the funnel slice fills it).

This slice does **not** depend on IMP-002. The only execution mode here is solo
(`/execute` isolation, worker-mode OFF, D6a); the work-in-place fallback rung is
the blessed D1/D6a trunk path. **Solo `/execute` isolation never mints ids** (the
slice already exists; it writes status/notes/phase/memory, not new entities) — the
one act that would pull in trunk-ref minting (D3 / IMP-002). The no-dependency
claim holds at the one place it could break.

## §2 Detection + creation + the always-provision rule (skill-prose)

**Detection.** `git rev-parse --git-dir` ≠ `git rev-parse --git-common-dir` ⇒ the
CWD is already inside a linked worktree (adapt, don't re-create; D1). **Submodule
guard:** a submodule *also* trips that; disambiguate by the `.git` gitdir under
`worktrees/` (worktree) vs `modules/` (submodule), or a non-empty `git rev-parse
--show-superproject-working-tree` ⇒ submodule. (superpowers prior art.)

**The lifecycle is creation + a mandatory provision step — not a copy ladder.**
The earlier "native hook copies via `.worktreeinclude`" framing is dropped: a
native copy bypasses `select_copies`, so a broad `**` allowlist could leak
`.doctrine/state/` before doctrine votes (B2). **Resolution: `provision` is the
sole copier.** Creation has a backend choice; provisioning is invariant.

**Creation backend, degrade in order:**

1. **Detect existing isolation** (D1) — already forked → skip creation.
2. **Harness native worktree-*creation*** if present and invocable **creation-only**
   (no auto-copy). Opportunistic; the Claude Code `WorktreeCreate` hook is a GitHub
   *discussion* proposal (mattbrailsford #54), unconfirmed-shipped (F1).
3. **`git worktree add <path> <branch>`** — the guaranteed-present, tested default.
4. **Work-in-place** (solo, no funnel) on sandbox denial — the blessed D1/D6a trunk
   path. No fork.

**Then, for any forked path (1–3), always:** `doctrine worktree provision <fork>`
→ run guards → baseline-verify. **Invariant-strength caveat (honest framing):** the
copy-seam guarantee holds because provision is the only copier. *If* a harness
force-copies on creation and cannot be run creation-only, doctrine cannot prevent
that copy — the guarantee there degrades to the static `check-allowlist` only, and
the project must keep `.worktreeinclude` precise. Prefer rung 3 (fully controlled)
as the default; never depend on rung 2.

## §3 The CLI heart — `provision` + `check-allowlist`

New module **`src/worktree.rs`**, ADR-001 leaf layering, pure/imperative split
(no disk/git in the pure core — passed in as inputs).

### Withhold authority — one structured list (F4)

The coordination/runtime tier that must never enter a fork is a **single
structured const** in `src/worktree.rs`, categorised, derived from the actual
runtime tier in `.gitignore` (verified lines 24/31–38):

```rust
enum Tier { State, PhaseLink, Handover, MemoryCache }
struct Withhold { tier: Tier, glob: &'static str }
const WITHHELD: &[Withhold] = &[
    W(State,       ".doctrine/state/**"),            // phase sheets, boot.md
    W(PhaseLink,   ".doctrine/slice/*/phases"),       // per-slice symlink into state
    W(Handover,    "**/handover.md"),                 // disposable agent context
    W(MemoryCache, ".doctrine/memory/index/**"),
    W(MemoryCache, ".doctrine/memory/embeddings/**"),
    W(MemoryCache, ".doctrine/memory/state/**"),
    W(MemoryCache, ".doctrine/memory/shipped/**"),    // synced global corpus
];
```

`.doctrine/skills/*` (also gitignored) is **derived**, not coordination — it is
regenerated by `doctrine install` in the fork, not copied and not a hazard; it is
documented but out of `WITHHELD`. **Parity test (real, not prose cross-check):** a
test asserts every `.doctrine/**` runtime glob in `.gitignore` is either covered by
`WITHHELD` or explicitly classified derived-regenerable — so adding a runtime glob
to `.gitignore` without classifying it fails CI.

### Allowlist syntax — a documented subset (M6)

The repo has only the `glob` crate (no gitignore-semantics crate). v1
`.worktreeinclude` is therefore a **documented subset**: blank lines, `#` comments,
literal repo-relative paths, and simple `glob` patterns (`*`, `**`, `?`). **No `!`
negation, no anchoring rules** in v1 (rejected by the parser with a clear error, so
a project can't silently rely on unsupported semantics). Fixtures cover each
supported class. This is independent of any harness hook (provision is the sole
copier), so native-parity is a non-goal.

### Pure core

```rust
struct Allowlist { patterns: Vec<glob::Pattern> }       // documented subset
fn parse_allowlist(text: &str) -> Result<Allowlist, ParseError>; // rejects '!' etc.
fn is_withheld(rel: &RelPath) -> Option<Tier>;          // matches WITHHELD
fn select_copies(a: &Allowlist, candidates: &[RelPath]) -> Selection; // {copy, withheld}
fn allowlist_violations(a: &Allowlist) -> Vec<Violation>; // patterns that *name* a withheld glob
```

### Impure shell

**Candidate set = gitignored files only**, enumerated with **`-z` (NUL-delimited,
matching the `src/git.rs` seam, m9)**: `git ls-files -z --others --ignored
--exclude-standard`. Committed files are already in the fork. `select_copies`
filters; copy each into the fork preserving its relative path.

**Invocation context (F4/F7).** `provision` runs from the **source (main-tree)
root** (`root::find` from CWD) and writes into `<fork>`. It must not run from inside
the fork.

**Copy safety (F5) — `safe_join` is insufficient.** The copy helper must:
- **canonicalize** source root, `<fork>`, and every destination component
  (`symlink_metadata` on parents) so no symlink component escapes the fork;
- **verify `<fork>` is a real sibling worktree**: it shares the source's
  `git-common-dir` and is not the source itself;
- **symlink policy:** for an allowlisted source symlink, resolve its target; copy
  **only if** the target stays inside the source tree *and* is not withheld;
  otherwise **skip + warn** (never follow out-of-tree or into `.doctrine/state/`).

### Two-layer exclusion (the OQ-3-B payoff)

- **`select_copies` (copy-time, the guarantee):** drops any *file* matching
  `WITHHELD` **even under a broad `*`/`**`** — skip + warn (name the file, exit 0).
  A wildcard allowlist stays usable; the copy physically cannot leak the tier.
- **`check-allowlist` (static smell test):** `allowlist_violations` rejects a
  *pattern that names* a withheld glob; nonzero exit. **Green is NOT completeness
  (F7):** it only proves no pattern names the tier — `select_copies` remains the
  guarantee. CI / `validate`-usable.

### Verbs

- **`doctrine worktree provision <fork>`:** read `.worktreeinclude` from source root
  (**absent ⇒ empty ⇒ copy nothing**, F2) → `allowlist_violations` (**fail closed**)
  → enumerate candidates (`-z`) → `select_copies` → safe copy, skip+warn withheld →
  report copied / withheld.
- **`doctrine worktree check-allowlist`:** parse + `allowlist_violations`; nonzero on
  a tier-naming pattern or an unsupported-syntax (`!`) pattern.

### The `.worktreeinclude` (not installed — F2)

Doctrine has no secrets/irreducible local files → default is **nothing to copy**.
Not shipped by the installer (a root-file install is clobber-prone — could overwrite
a consuming project's file). `provision` tolerates absence; `/worktree` documents a
commented template the project may adopt. Project-owned, never installer-managed.

## §4 Guards (skill-prose, `/worktree`)

- **Commit-before-spawn (D5) — exact gate (F7).** Before creation, run `git status
  --porcelain -z`; **abort** if any tracked file is dirty **or** any untracked,
  non-ignored file exists (it would be silently absent from the fork). Only a clean
  tree (modulo ignored files, which provision handles) may fork. The fork sees only
  committed HEAD.
- **Branch-point check (D5) — IN SCOPE (reversed from defer, B3).** Capture HEAD on
  the source pre-create (`git rev-parse HEAD`); after creating the worktree, assert
  the worktree HEAD equals the captured SHA; mismatch ⇒ abort/recreate. Cheap,
  ADR-D5-mandated, needs no IMP-002 — so it lands here even though solo rarely
  exercises it; the funnel slice only *extends* it to the concurrent case.
- **Baseline-verify (D9) — project-configured command (m10).** After provision, run
  the project's regenerate + verify command in the fork; green gate before handoff.
  **For this repo that is `just check`** (fmt+lint+test+build), not a hardcoded
  `cargo …`. The command is project/skill-provided (doctrine is a framework), not
  baked into Rust. An unbuildable fork is fixed in provisioning, never handed off.

## §5 `/worktree` mode contract + `/execute` thread + scope reconciliation

**`/worktree` skill contract (defined now — F8, the OQ-1 split seam).** The skill
is parameterised so the funnel slice reuses it without inheriting solo semantics:

- **Inputs:** `mode = solo | worker`; `allow_work_in_place: bool` (true only for
  `solo`); requested branch/path.
- **Behaviour:** `solo` may degrade to the work-in-place rung on sandbox denial;
  **`worker` must NOT** — a worker with no real fork is a hard failure (the funnel's
  isolation is mandatory). SL-029 implements `solo`; `worker` is declared in the
  contract, implemented by the funnel slice.
- **Outputs:** `{ fork_path, branch, head_sha, provision_report{copied, withheld},
  baseline_result }`.

**`/execute` thread (solo).** Optional branch: *isolation requested? → invoke
`/worktree` mode=solo → execute inside the fork → the fork branch is the
deliverable.* **Worker-mode OFF (D6a)** — `/execute` is its own orchestrator and
writes doctrine state directly. Default no-isolation path unchanged. **Isolation is
explicit opt-in** (user/plan annotation), never automatic (F8).

**Squash-orphan caveat (ADR-006 Open / Negative).** Memory recorded inside a
worktree branch is orphaned by a squash-merge (content survives; SL-008 staleness
fires). Skill prose nudges record-on-trunk for durable memory.

**Scope reconciliation.** `slice-029.md` narrowed to the lifecycle + `/execute`
path; funnel/`/dispatch` moved out; affected surface = `plugins/doctrine/skills/
{worktree,execute}/` + `src/worktree.rs` + CLI + `src/fsutil.rs`; follow-up records
the funnel slice.

## §6 Verification alignment

**ADR-006 Verification bullets SL-029 discharges:**
- *allowlist excludes the coordination/runtime globs* → `select_copies` +
  `check-allowlist` tests (VT).
- *baseline build+test passes in the fork before handoff* → `/worktree` baseline
  step running `just check` (VA).
- *branch-point check (D5)* → skill-prose HEAD pre/post compare (VA; cheap solo).
- *tier merge-safety (D4): `phases` symlink relative* → existing invariant (VT, no
  regression).

**Test cases (SL-029):**
- *pure:* `parse_allowlist` per supported class + **rejects `!`/unsupported** (M6);
  `allowlist_violations` canary (pattern naming `.doctrine/state/` rejected);
  **`select_copies` withholds a `.doctrine/state/...` file under a `**` allowlist**
  (load-bearing); **`WITHHELD`↔`.gitignore` parity** (F4: an unclassified runtime
  glob fails).
- *impure / e2e:* `provision` copies an allowlisted gitignored file into a fork;
  withholds a tier file with warning (exit 0); **refuses an out-of-tree / into-state
  symlink** (F5); **refuses a `<fork>` that isn't a sibling worktree** (F5);
  candidate enumeration is `-z`-safe for newline/quoted paths (m9); `check-allowlist`
  nonzero on a statically-bad allowlist.

**Deferred to the funnel slice:** worker-mode guard (D2a/IMP-002); funnel order
(D7); `memory record` worktree-warn; branch-point under *concurrent* dispatch;
`/worktree` `mode=worker` implementation.

## §7 Affected surface

- `plugins/doctrine/skills/worktree/SKILL.md` — **new**: the lifecycle skill (the
  §5 mode contract; detection, creation, guards, baseline prose; invokes the CLI).
- `plugins/doctrine/skills/execute/SKILL.md` — thin optional solo-isolation thread.
- `src/worktree.rs` — **new**: pure core (`Allowlist`, `WITHHELD`, `is_withheld`,
  `select_copies`, `allowlist_violations`) + impure `provision` + safe copy.
- `src/main.rs` — new `Worktree { Provision, CheckAllowlist }` subcommand.
- `src/fsutil.rs` — recursive, canonicalize-guarded copy helper (beyond `safe_join`).
- `.worktreeinclude` — **not installed** (F2); project-owned; `provision` tolerates
  absence; the `/worktree` skill documents the template.
- `install/manifest.toml` — **untouched** (no root-file row; skills shipped via the
  existing `plugins/` mechanism).
- `.doctrine/skills/*` — **derived/gitignored** (B1); regenerated by install, never
  authored or copied.
- `plugins/doctrine/skills/dispatch/SKILL.md` — **untouched** (placeholder; funnel).

## Open items / carried risks

- **R-2 (native rung unconfirmed, F1).** The Claude Code `WorktreeCreate` hook is a
  discussion proposal; rung 3 is the tested default. If the hook is absent, rung 2
  is never taken — no design change. If a harness force-copies, the invariant
  degrades to `check-allowlist` only (§2 caveat) — documented, not papered over.
- **Memory-cache glob precision.** `WITHHELD` memory-cache entries are pinned to
  `.gitignore` lines 35–38; the parity test is the guard against drift.
- **A-1 (downgraded).** IMP-002 is a prereq for the funnel slice, not SL-029.
- **R-1.** Worker self-verify degradation (D6) is a funnel-slice concern; N/A here.

## Adversarial review log

**Pass 1 — internal (F1–F8):** integrated (native rung opportunistic; no installed
`.worktreeinclude`; candidate set = gitignored; provision from source root;
check-allowlist ≠ completeness; explicit isolation trigger; solo never mints ids).
Note: pass-1 F3 (defer branch-point) was **reversed** by pass 2 / B3.

**Pass 2 — external, gpt-5.5 via codex MCP.** Findings integrated:
- **B1** skill source is `plugins/doctrine/skills/`, not gitignored
  `.doctrine/skills/` → affected surface corrected (§1, §7).
- **B2** native copy bypasses `select_copies` → `provision` is the *sole* copier;
  native rung is creation-only/opportunistic; honest invariant-degradation caveat
  (§2).
- **B3** branch-point deferral violates D5 and isn't IMP-002-bound → **re-included**
  in SL-029 (§4).
- **B5** `safe_join` insufficient → canonicalize + sibling-worktree check + explicit
  symlink policy (§3 Copy safety).
- **M4** glob authority weak → single structured `WITHHELD` + real parity test;
  `.doctrine/skills/*` classified derived (§3).
- **M6** allowlist syntax underspecified + only `glob` crate → documented subset,
  `!`/anchoring rejected; native-parity dropped (§3).
- **M7** candidate/commit-gate holes → exact `git status --porcelain -z` gate;
  untracked-non-ignored aborts (§4); `-z` enumeration (§3).
- **M8** `/worktree` had no mode contract → `mode=solo|worker` +
  `allow_work_in_place` defined now (§5).
- **m9** `git ls-files` needs `-z` → required (§3).
- **m10** baseline hardcoded `cargo …` → project-configured; `just check` here (§4).
