# Design SL-010: Symlink skills from a canonical .doctrine/skills tree (Claude-first)

## 1. Design Problem

`doctrine skills install` copies each embedded skill into `.claude/skills/<id>`
and skips when the dir exists (`skills.rs:188` `claude_steps`). Copies drift —
editing a `SKILL.md` and re-installing is a silent no-op, refreshable only by a
manual `rm -rf`. Replace the copy-and-skip with a single canonical tree plus
symlinks so a re-install always refreshes, with no flag, **without ever mutating
anything doctrine did not create**.

## 2. Current State

Two independent installers, each with its own pure `Plan`/`Step` model:

- **`install.rs`** — `doctrine install` bootstraps `.doctrine/` from the embedded
  `install/manifest.toml`: creates `dirs.create` (already includes
  `.doctrine/skills`), copies embedded files, and appends `gitignore.entries` to
  the project `.gitignore` (additive, idempotent — `install.rs:182-191`). It does
  not enumerate or install skills.
- **`skills.rs`** — `doctrine skills install` discovers embedded skills
  (`PluginAssets`, `plugins/<domain>/skills/<id>/`), then per agent:
  - **Claude (direct):** `claude_steps` → `Step::{Install,Skip}` over
    `.claude/skills/<id>`; `Install` runs `copy_skill` (file-by-file, non-
    transactional — `skills.rs:312-327`); `Skip` when `dest.exists()`. **Today the
    installer never mutates an existing path** (`skills.rs:188-205,346-355`) — a
    property this slice must preserve.
  - **Other agents (delegate):** `delegate_argv` → `npx skills add doctrine/doctrine
    --agent <a> [--skill …] --yes`; npx clones from GitHub and symlinks by default.
- `claude_dir(root, global)` → `.claude/skills` under the project root, or `$HOME`
  when `--global`. **`resolve_agents` auto-detects via `root/.claude`
  (`skills.rs:274-284`) and `run_list` inspects `root/.claude/skills` with
  `.exists()` (`skills.rs:418`) — both project-local, no global mode. Pre-existing.**

## 3. Forces & Constraints

- **Storage rule.** `.doctrine/skills` is derived (regenerable from the embed) →
  gitignored, `rm -rf`-able. Source of truth is the embed.
- **Never clobber foreign data — hard.** Anything doctrine did not create (a real
  dir, *or a symlink pointing somewhere other than our canonical target*) must
  survive every install untouched. This is the binding constraint; the rest bends
  to it.
- **Crash-safety.** Managed `.claude/skills/<id>` symlinks are *live references*
  into the canonical tree. Refreshing canonical must not transiently break them.
- **Complexity budget.** Claude-first: own the local canonical tree + Claude links
  only; leave the `npx` delegation untouched. No agent→dir registry.
- **Pure/imperative split.** Planning classifies (reads the filesystem, no
  writes); mutations live behind the seam. Platform: unix/nixos; Windows non-goal.

## 4. Guiding Principles

- One canonical copy; agents are pointers to it.
- **Ownership is proven, not assumed.** A link is doctrine's iff its value equals
  the canonical target we would write. Type (`is_symlink`) is necessary but not
  sufficient.
- Every mutation is atomic (stage + `rename`) and idempotent.
- Reuse existing seams (`copy_skill`, `claude_dir`, the `Plan`/`execute` shape).

## 5. Proposed Design

### 5.1 System Model

`skills install` (Claude target) becomes two ordered phases:

1. **Materialise canonical — staged, minimal-window.** For each selected skill:
   clear any crashed leftover `.doctrine/skills/.tmp-<id>/`, stage the embedded
   files into it (`copy_skill`, same filesystem so `rename` is valid), then
   `remove_dir_all(.doctrine/skills/<id>)` and `rename(.tmp-<id> → <id>)`. **Note
   the unix reality: `rename` does NOT replace a non-empty directory** (would
   `ENOTEMPTY`), and `std` has no `renameat2(RENAME_EXCHANGE)`, so a full directory
   swap cannot be a single atomic op. The window is one syscall wide (between the
   `remove` and the `rename`); a crash there leaves the link dangling and the next
   `skills install` heals it (idempotent, derived tree). Always overwrite.
2. **Reconcile the agent link by proven ownership.** For each selected skill,
   classify `.claude/skills/<id>` by `symlink_metadata` + `read_link`:
   - **missing** → create the relative symlink → canonical (`Link`);
   - **symlink whose value == our canonical target** → ours: ensure it (no-op, or
     heal if its canonical target is being re-materialised) — `Relink`;
   - **symlink pointing elsewhere, OR a real dir/file** → **foreign** → leave it,
     warn (`KeepForeign`). This is both the override hatch *and* the
     never-clobber guarantee.

The `npx` delegate path is unchanged (Claude-first; accepted split-source).

### 5.2 Interfaces & Contracts

- CLI: **no new flag.** `--force` is not added.
- `skills.rs` Claude-path step model replaces `{Install,Skip}`:
  ```
  enum Link { Create { id, dest, target }, Relink { id, dest, target }, KeepForeign { id, dest, reason } }
  AgentPlan::Claude { canonical: Vec<Canonical{ id, dest }>, links: Vec<Link> }
  ```
  `target` is the **relative** symlink value; `reason` distinguishes
  `real-dir` vs `foreign-symlink → <whereitpoints>` for an honest warning.
- New pure helpers:
  - `canonical_dir(root, global)` — mirrors `claude_dir` (same base: root, or
    `$HOME` for `--global`), so both trees share a base and the relative target is
    stable.
  - `relative_target(agent_skills_dir, canonical_dir, id)` — computes the link
    value (`../../.doctrine/skills/<id>` in the common project-local case);
    **derived from the two dirs, not hard-coded**, so global / unusual layouts
    stay correct.
  - `classify_link(dest, target) -> Link` — the ownership decision (read_link +
    compare).
- New imperative helpers behind the seam:
  - `materialise_canonical(entry, canonical_dir)` — clear stale temp, stage, then
    `remove_dir_all` + `rename` (minimal-window; see §5.1).
  - `write_link(dest, target)` — create the symlink via temp-name + `rename`. This
    one **is** genuinely atomic: `rename` *does* replace an existing symlink/file
    (only non-empty dirs are the exception), so an owned-link relink never leaves a
    half-state.
  - `ensure_gitignored(root, "​.doctrine/skills/*")` — shared with `install.rs`
    (extract from its gitignore step): `skills install` enforces its own derived-
    tree ignore invariant rather than depending on a prior `doctrine install`
    (F4).
- `install/manifest.toml`: add `.doctrine/skills/*` to `[gitignore].entries` (the
  bootstrap path also writes it).
- `run_list`: installed-presence test changes from `.exists()` to a `lexists`
  (`symlink_metadata().is_ok()`), so a managed (even momentarily dangling) link
  reports installed (F5).

### 5.3 Data, State & Ownership

| Path | Tier | Owner | Tracked |
|---|---|---|---|
| `plugins/<domain>/skills/<id>/` | authored source (this repo) | doctrine | yes (embed source) |
| `.doctrine/skills/<id>/` | **derived** | `skills install` | gitignored |
| `.claude/skills/<id>` → canonical (symlink) | derived pointer | `skills install` | gitignored |
| `.claude/skills/<id>` (real dir, or foreign symlink) | **authored override** | the user | `git add -f` |

`doctrine install` ensures the dir + ignore entry (manifest); `skills install`
owns canonical *contents*, the agent links, **and** its own ignore invariant
(F4 — self-enforcing, not order-dependent).

### 5.4 Lifecycle, Operations & Dynamics

- `skills install` (Claude): ensure ignore → materialise canonical (atomic
  overwrite) → reconcile each agent link (create / relink-if-ours / keep-foreign).
  `--dry-run` prints the classified plan.
- Re-install: canonical re-materialised from the current embed; only *our* links
  touched; foreign paths kept. The refresh the old skip silently dropped.
- Override adopt: replace a managed symlink with a real copy (`cp -rL`) → install
  classifies it foreign → kept. Revert: `rm -rf` → install re-links. Reporting:
  `refreshed <id>` (canonical) · `linked`/`relinked`/`kept <id> (<reason>)`.
- `--global`: canonical + agent dirs both under `$HOME`; the relative target is
  computed, so it stays correct. **Out of scope: global auto-detection and a
  global mode for `skills list`** — both pre-exist this slice (§2) and are left as
  a follow-up; `--global install --agent claude` is the supported global path.

### 5.5 Invariants, Assumptions & Edge Cases

- **Ownership invariant:** the installer mutates `.claude/skills/<id>` **only** when
  it is missing or is a symlink whose value already equals our canonical target.
  Everything else is foreign and untouched. Preserves the current "never mutate an
  existing path" property *except* for our own links.
- **Detection** uses `symlink_metadata`/`read_link`, never `exists()` (which
  follows links). A dangling link with *our* target → ours → healed by the
  canonical re-materialise.
- **Atomicity (asymmetric, by unix reality):** the **link** write is atomic
  (`rename` replaces a symlink in one op). The **canonical dir** swap is *not* a
  single atomic op (`rename` can't replace a non-empty dir; no `renameat2` in std)
  — it is staged with a one-syscall window, crash-healed by an idempotent re-run.
  A crash never corrupts a *partial* tree (the half-written copy lives in
  `.tmp-<id>`, never under `<id>`); the only failure mode is a dangling link until
  re-install.
- **Ownership spelling (accepted limitation):** ownership is raw
  `read_link(dest) == <relative target>`. An equivalent link in a *different
  spelling* (absolute, or a normalised relative path — e.g. one a future version
  emitted) classifies as **foreign** → kept + warned, **not healed**. This is
  safe-degrading (never clobbers, never data-loss) but means a spelling change
  across versions needs an explicit migration. v1 emits exactly one spelling, so
  the case does not arise in-version. Normalising both sides before comparison is a
  possible future hardening (rejected for v1 — budget; dangling-but-ours links
  don't resolve, so canonicalisation isn't reliable anyway).
- **Partial multi-agent failure** (F6): the Claude phase is atomic and
  non-destructive; if a later `npx` agent fails, the Claude side is correctly
  applied and the command errors — benign, idempotent on re-run. Cross-`npx`
  transactionality is not attempted (and not possible).
- **Orphaned canonical / orphaned links** (a skill removed from the embed) are not
  pruned — follow-up. Low harm (gitignored, unreferenced).

## 6. Open Questions & Unknowns

- **Q1 — canonical creation vs bootstrap.** Resolved: `skills install` creates
  `.doctrine/skills` on demand *and* ensures the ignore entry itself (F4); the
  manifest entry is belt-and-suspenders for the `doctrine install` path.
- **Q2 — prune orphaned canonical/links?** Deferred (follow-up).
- **Q3 — `write_link` atomicity.** Temp-name symlink + `rename` over `dest` (only
  when `dest` is ours or missing).
- **Q4 — global coherence.** `--global` install works for the explicit-agent path;
  global auto-detect + `skills list --global` are pre-existing gaps, follow-up.

## 7. Decisions, Rationale & Alternatives

- **D1 — symlink to a canonical tree, not copy.** Kills drift; `--force`
  unnecessary. Alt (rejected): copies + `--force`.
- **D2 — `.doctrine/skills` derived/gitignored, regenerated by install.** Storage
  rule. `skills install` self-enforces the ignore entry (F4).
- **D3 — ownership by *target equality*, not file type.** A link is ours iff its
  value equals our canonical target; foreign symlinks are preserved exactly like
  real dirs. This is the fix for the review's load-bearing finding (F2): `is_symlink`
  is type, not ownership. Alt (rejected): "relink any symlink" — a destructive
  regression against the never-clobber constraint.
- **D4 — Claude-first; `npx` delegation untouched.** Smallest diff. Global
  detection/list deferred (Q4).
- **D5 — staged materialise (stage to temp → remove → rename), not copy-in-place.**
  Live links forbid a copy that half-writes under `<id>` (F1). The dir swap is
  minimal-window, not atomic — `rename` can't replace a non-empty dir and std lacks
  `renameat2` (review pass 2, finding 4). The leaf-link write *is* atomic. Alt
  (rejected): file-by-file copy directly into `<id>` — exposes a half-written tree
  to readers. Alt (rejected for v1): symlink-indirection canonical (`<id>` → a
  content dir, swap the inner symlink atomically) — real atomicity but double
  indirection, over budget.
- **D6 — canonical owned by `skills install`, dir location by `doctrine install`;
  ignore invariant enforced by both.** (F4.)

## 8. Risks & Mitigations

- **R1 — clobbering foreign data.** *Retired* by D3: only missing-or-exactly-ours
  links are mutated; foreign symlinks and real dirs are kept + warned.
- **R2 — partial/interrupted refresh.** *Retired* by D5: atomic stage+rename for
  both canonical and links; re-run reconciles.
- **R3 — manifest gitignore drift downstream.** Mitigated: this slice adds the
  entry and `skills install` self-enforces it; `install.rs` manifest test guards.
- **R4 — behaviour change to `claude_steps`.** Expected — the copy path is
  replaced; its tests are rewritten for the trichotomy. The "never mutate an
  existing path" property is *preserved* for foreign paths (D3). `delegate_argv`
  tests stay green; the entity-engine preservation gate is untouched.
- **R5 — `list`/`installed` mismatch.** Mitigated: `lexists` (F5).

## 9. Quality Engineering & Validation

- **Pure planning tests** (tempdir states): missing → `Create`; symlink with our
  target → `Relink`; **symlink pointing elsewhere → `KeepForeign(foreign-symlink)`**;
  real dir → `KeepForeign(real-dir)`; dangling-but-ours → `Relink`.
  `relative_target` over project-local and `--global` bases; `canonical_dir`
  honours `--global`.
- **Materialise tests:** atomic overwrite is clean (a stale file from a prior
  version is gone); an interrupted stage (temp left, `rename` not run) leaves the
  prior canonical intact.
- **Execute tests** (fs via tempdir): create → link resolves to canonical content;
  relink-ours over a dangling link heals it; **a foreign symlink and a real dir are
  both left byte/target-identical and emit the `kept` warning.**
- **`run_list` test:** a managed (dangling) link still reports installed (F5).
- **Manifest test** (`install.rs`): `.doctrine/skills/*` gitignored + dir created.
- `delegate_argv` tests unchanged. `just check` green; clippy zero warnings.

## 10. Review Notes

Adversarial review by codex mcp (read-only) on the draft — **verdict: red**, six
findings. Disposition:

- **F1 (canonical rm+copy unsafe while links are live)** — accepted → D5 (atomic
  stage+rename), §5.1/§5.5.
- **F2 ("any symlink = ours" repoints foreign links)** — accepted, load-bearing →
  D3 (ownership by target equality); R1 retired.
- **F3 (`--global` detection/list not wired)** — accepted, but the gaps pre-exist
  SL-010 → demoted: §2 documents the pre-existing state, Q4 + §5.4 scope `--global`
  to the explicit-agent path, detection/list global-mode deferred.
- **F4 (gitignore unenforced if `skills install` precedes `doctrine install`)** —
  accepted → `skills install` self-enforces via shared `ensure_gitignored` (§5.2,
  D6).
- **F5 (`list` `.exists()` hides dangling managed link)** — accepted → `lexists`
  (§5.2, R5).
- **F6 (multi-agent non-transactional)** — accepted-in-part: dissolves once F1/F2
  make the Claude phase atomic + non-destructive; residue documented (§5.5).

**Second codex pass** (revised design) — **verdict: red**, but four of six findings
were category errors: the reviewer read the *committed code* and flagged that the
design is not yet implemented (old `Step::{Install,Skip}`, no `ensure_gitignored`,
`list` still `exists()`, old tests). Those are the implementation work, gated behind
the plan — not design defects; they are correct observations of "not built yet."
Two were genuine design bugs, both accepted:

- **P2-F4 (load-bearing) — `rename` cannot replace a non-empty directory on unix;
  no `renameat2` in std.** The draft's "stage + rename = atomic" was false for the
  canonical *directory*. Fixed → §5.1/§5.2/§5.5/D5: staged + minimal-window swap
  (remove + rename), one-syscall window, crash-healed by idempotent re-run; the
  *leaf-link* write remains genuinely atomic.
- **P2-F5 — raw `read_link` string equality misclassifies a differently-spelled own
  link as foreign.** Accepted as a documented, safe-degrading limitation (kept +
  warned, never clobbered); v1 emits one spelling. §5.5 + D3.

Design is considered **locked** after these two fixes: further design-vs-code review
will only re-surface "not implemented" until the plan executes. The atomicity and
ownership-spelling trade-offs are explicit; remaining correctness (e.g. the exact
swap sequence, stale-temp cleanup, `lexists`) is now a matter of faithful
implementation + the §9 tests, which the plan owns.
