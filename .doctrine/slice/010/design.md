# Design SL-010: Symlink skills from a canonical .doctrine/skills tree (Claude-first)

## 1. Design Problem

`doctrine skills install` copies each embedded skill into `.claude/skills/<id>`
and skips when the dir exists (`skills.rs:188` `claude_steps`). Copies drift —
editing a `SKILL.md` and re-installing is a silent no-op, refreshable only by a
manual `rm -rf`. Replace the copy-and-skip with a single canonical tree plus
symlinks so a re-install always refreshes, with no flag and no risk to a user's
hand-edited skills.

## 2. Current State

Two independent installers, each with its own pure `Plan`/`Step` model:

- **`install.rs`** — `doctrine install` bootstraps `.doctrine/` from the embedded
  `install/manifest.toml`: creates `dirs.create` (already includes
  `.doctrine/skills`), copies embedded files, and appends `gitignore.entries` to
  the project `.gitignore` (additive, idempotent). It does **not** enumerate or
  install skills.
- **`skills.rs`** — `doctrine skills install` discovers embedded skills
  (`PluginAssets`, `plugins/<domain>/skills/<id>/`), then per agent:
  - **Claude (direct):** `claude_steps` → `Step::{Install,Skip}` over
    `.claude/skills/<id>`; `Install` runs `copy_skill` (writes embedded files);
    `Skip` when `dest.exists()`.
  - **Other agents (delegate):** `delegate_argv` → `npx skills add doctrine/doctrine
    --agent <a> [--skill …] --yes`; npx clones from GitHub and symlinks by default.

`claude_dir(root, global)` resolves `.claude/skills` under the project root, or
under `$HOME` when `--global`.

## 3. Forces & Constraints

- **Storage rule.** `.doctrine/skills` is derived (regenerable from the embed) →
  gitignored, `rm -rf`-able. Source of truth is the embed (`plugins/` here, the
  binary downstream). (SL-010 scope decision.)
- **Never clobber foreign data.** A skill the user hand-edited (a real dir, not a
  doctrine symlink) must survive every install.
- **Complexity budget.** Claude-first: own the local canonical tree + Claude links
  only; leave the `npx` delegation untouched. No agent→dir registry, no Node
  ownership.
- **Pure/imperative split.** Planning classifies (reads the filesystem, no
  writes); the link/relink/replace mutations live behind the existing seam.
- **Platform.** unix/nixos — `std::os::unix::fs::symlink`. Windows is a non-goal.

## 4. Guiding Principles

- One canonical copy; agents are pointers to it.
- The plan keys on *file type*, not a flag: a doctrine symlink is owned and
  refreshed; a real dir is foreign and sacrosanct.
- Reuse the seams that exist (`copy_skill`, `claude_dir`, the `Plan`/`execute`
  shape); add the minimum new structure.

## 5. Proposed Design

### 5.1 System Model

`skills install` (Claude target) becomes two ordered phases:

1. **Materialise canonical** — for each selected skill, clean-replace
   `.doctrine/skills/<id>/` from the embed (always overwrite; it owns no authored
   data). Reuses `copy_skill` after removing any prior dir (clean, not merged — no
   stale files).
2. **Link agent dir** — for each selected skill, reconcile `.claude/skills/<id>`
   against the canonical tree by type:
   - **missing** → create a relative symlink → canonical (`Link`);
   - **symlink** → replace it, pointing at canonical (`Relink`) — fixes dangling
     or moved targets, idempotent when already correct;
   - **real dir/file** → leave it, warn (`KeepForeign`) — the override.

The `npx` delegate path is unchanged: non-Claude agents still clone+symlink from
GitHub (accepted split-source; Claude-first).

### 5.2 Interfaces & Contracts

- CLI: **no new flag.** `--force` is not added (its premise dissolves).
- `skills.rs` `Step` (Claude path) replaces `{Install,Skip}` with:
  ```
  enum Link { Create { id, dest, target }, Relink { id, dest, target }, KeepForeign { id, dest } }
  ```
  and the Claude plan carries a canonical set + the link steps:
  ```
  AgentPlan::Claude { canonical: Vec<Canonical{ id, dest }>, links: Vec<Link> }
  ```
  `target` is the **relative** symlink value (`../../.doctrine/skills/<id>` from
  `.claude/skills/`); `Canonical.dest` is `<base>/.doctrine/skills/<id>`.
- New pure helpers: `canonical_dir(root, global)` (mirrors `claude_dir`), and
  `relative_target(agent_skills_dir, canonical_dir, id)`.
- New imperative helpers behind the seam: `materialise_canonical(entry, dest)`
  (rm + `copy_skill`) and `link_force(dest, target)` (atomic-ish: write to a temp
  name, `rename` over `dest`; remove a prior symlink first).
- `install/manifest.toml`: add `.doctrine/skills/*` to `[gitignore].entries`.

### 5.3 Data, State & Ownership

| Path | Tier | Owner | Tracked |
|---|---|---|---|
| `plugins/<domain>/skills/<id>/` | authored source (this repo) | doctrine | yes (embed source) |
| embed (`PluginAssets`) | derived (compiled in) | build | — |
| `.doctrine/skills/<id>/` | **derived** | `skills install` | gitignored |
| `.claude/skills/<id>` (symlink) | derived pointer | `skills install` | gitignored (`.claude`) |
| `.claude/skills/<id>` (real dir) | **authored override** | the user | `git add -f` |

`doctrine install` owns only the *existence* of `.doctrine/skills` (manifest
`dirs.create`) and its gitignore entry; `skills install` owns its *contents* and
the agent links. Clean division: bootstrap prepares the location + ignore rule,
`skills install` populates and links.

### 5.4 Lifecycle, Operations & Dynamics

- `doctrine skills install` (Claude): materialise canonical (overwrite) → link
  each agent dir (create/relink/keep). `--dry-run` prints the classified plan;
  `--global` puts both trees under `$HOME` (symmetric, so the relative target is
  unchanged).
- Re-install: canonical rewritten from the current embed; symlinks reconciled;
  foreign dirs kept. This is the refresh that the old skip silently dropped.
- Override adopt: replace a symlink with a real copy (`cp -rL`) → next install
  keeps it. Revert: `rm -rf` it → next install relinks.
- Reporting verbs: `refreshed <id>` (canonical), `linked`/`relinked`/`kept <id>
  (real dir — not a symlink)`.

### 5.5 Invariants, Assumptions & Edge Cases

- **Detection** uses `symlink_metadata().is_symlink()`, never `exists()` (which
  follows links). A dangling symlink classifies as `Relink`.
- A symlink pointing somewhere *other* than canonical is still doctrine's →
  relinked. Only a **real** dir/file is foreign. (Documented; the override is a
  real copy, not a hand-made symlink.)
- Clean-replace of canonical is safe: it is derived and nothing but doctrine's own
  symlinks reference it.
- **Orphaned canonical** (a skill deleted from the embed) is not pruned here —
  follow-up. Low harm (gitignored, unreferenced).
- `--global` canonical lives at `$HOME/.doctrine/skills`; relative target holds
  because both trees share the `$HOME` base.

## 6. Open Questions & Unknowns

- **Q1 — does `skills install` need `.doctrine/skills` pre-created by `doctrine
  install`, or create it itself?** Lean: create-on-demand in `materialise_canonical`
  (don't couple to a prior bootstrap); the manifest dir-create becomes belt-and-
  suspenders. Confirm in PHASE planning.
- **Q2 — prune orphaned canonical / orphaned links on install?** Deferred to a
  follow-up unless trivial.
- **Q3 — atomicity of relink.** `rename` over an existing symlink is atomic on
  unix; removing-then-creating has a small window. Use temp-name + `rename`.

## 7. Decisions, Rationale & Alternatives

- **D1 — symlink to a canonical tree, not copy.** Kills the drift/staleness class
  outright; `--force` becomes unnecessary. Alt (rejected): keep copies + add
  `--force` — leaves drift, needs a flag, still clobbers blindly.
- **D2 — `.doctrine/skills` derived/gitignored, regenerated by install.** Matches
  the storage rule; no committed copy to drift. Alt (rejected): commit it (drift,
  duplicates `plugins/`); config knob (complexity budget).
- **D3 — type-keyed reconcile (symlink=ours / real=foreign).** The override hatch
  falls out for free; no flag, no registry of "pinned" skills. Alt (rejected): a
  `--force`/`--keep` flag matrix.
- **D4 — Claude-first; `npx` delegation untouched.** Smallest diff, no agent→dir
  registry. Accepts split-source for non-Claude agents. Alt (deferred):
  doctrine owns the `.agents` matrix (own-the-matrix) — larger, drops Node.
- **D5 — canonical owned by `skills install`, location by `doctrine install`.**
  Keeps the bootstrap/skills division already in the codebase.

## 8. Risks & Mitigations

- **R1 — relink clobbers a user's *own* symlink** (not a real dir). Mitigation:
  documented; the supported override is a real copy. Low likelihood.
- **R2 — partial failure mid-plan** (canonical written, link fails). Mitigation:
  canonical is idempotent; re-run reconciles. No destructive op on foreign data.
- **R3 — manifest gitignore drift downstream** (the derived tree committed).
  Mitigation: this slice adds the entry; covered by an `install.rs` manifest test.
- **R4 — behaviour change to `claude_steps`** breaks its existing tests.
  Expected — the copy path is being replaced; rewrite those tests for the
  trichotomy. The entity-engine preservation gate is untouched (skills don't use
  it); the `delegate_argv` tests stay green.

## 9. Quality Engineering & Validation

- **Pure-ish planning tests** (tempdir states, the codebase idiom): classify a
  missing dest → `Create`; an existing symlink → `Relink`; a dangling symlink →
  `Relink`; a real dir → `KeepForeign`. `relative_target` returns
  `../../.doctrine/skills/<id>`. `canonical_dir` honours `--global`.
- **Materialise tests**: canonical overwrite is clean (a stale file from a prior
  version is gone after re-materialise).
- **Execute tests** (fake `Runner` unaffected; fs via tempdir): create → symlink
  resolves to canonical content; relink over a dangling link heals it; keep leaves
  a real dir byte-identical and emits the `kept` warning.
- **Manifest test** (`install.rs`): `.doctrine/skills/*` is gitignored and the dir
  is created.
- `delegate_argv` tests unchanged (behaviour-preservation for the npx path).
- `just check` green; clippy zero warnings.

## 10. Review Notes

Adversarial review pending (the slice-002/003/004 rhythm). Candidate probes:
the foreign-symlink clobber (R1) — is "symlink = ours" too aggressive?; whether
`--global` canonical under `$HOME/.doctrine` is desirable or surprising; whether
canonical materialisation belongs in `skills install` or a shared lower layer if
own-the-matrix (D4) is ever revisited.
