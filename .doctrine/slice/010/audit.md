# Audit SL-010: Symlink skills from a canonical .doctrine/skills tree

Hand-authored verification / code-review / drift findings (no scaffold). The
design is locked ([design.md](design.md) §10, two codex passes); this records the
**implementation** verification: an adversarial review of the SL-010 diff against
the design's load-bearing invariants, plus dispositions.

## Method

- TDD red/green/refactor across PHASE-01..05; each phase ended green
  (`just check`: fmt + clippy `-D warnings` + test + build).
- End-to-end over the built binary: `tests/e2e_skills_symlink.rs` —
  install → link resolves → re-install refreshes a stale canonical → a real-dir
  override is kept.
- Adversarial code-review of `git diff 097370c~1..33e786c -- src/skills.rs
  src/install.rs install/manifest.toml` against design §3/§5/§7 invariants
  (independent reviewer agent).

## Findings & disposition

- **A1 (bug, accepted+fixed) — `materialise_canonical` stale-temp cleanup used
  `exists()`.** `exists()` follows symlinks: an odd dangling-symlink leftover at
  `.tmp-<id>` would be missed, and `remove_dir_all` errors on a symlink anyway, so
  `copy_skill`'s `create_dir_all` would then fail on the stale link. Fixed →
  lexists (`symlink_metadata`) + type-aware removal (dir → `remove_dir_all`, else
  `remove_file`). Upholds the design's never-`exists()` rule for the temp too.
  Test: `materialise_clears_a_dangling_temp_leftover`.
- **A2 (risk, accepted+fixed, load-bearing) — TOCTOU between plan classification
  and link mutation.** `build_plan` classifies ownership; `execute` mutates later
  (the confirm-prompt window, or a concurrent install). A foreign **symlink/file**
  appearing at `dest` in that gap would be silently clobbered by `write_link`'s
  `rename`, violating the binding never-clobber constraint (§3/D3) and §5.5's
  wording ("mutates only when [at mutation] missing or ours"). Fixed → `execute`
  re-classifies each Create/Relink dest immediately before `write_link`; if it
  turned foreign it is kept+warned, never clobbered. (A real **dir** was already
  safe — `rename` cannot replace a directory.) Test:
  `execute_re_keeps_a_dest_that_turned_foreign_after_planning`.

## Invariants verified (green)

- **Ownership by target equality (D3).** `classify_link` mutates only
  missing-or-exactly-ours; foreign symlink + real dir both kept byte/target-
  identical + warned. Now enforced at BOTH plan and mutation time (A2).
  Tests: `classify_link_covers_the_ownership_trichotomy`,
  `execute_keeps_a_foreign_real_dir`, `execute_keeps_a_foreign_symlink`.
- **Detection never `exists()`.** `classify_link` + `lexists` + materialise temp
  (post-A1) all use `symlink_metadata`/`read_link`. A dangling-but-ours link is
  healed. Tests: trichotomy, `execute_relink_heals_a_dangling_owned_link`,
  `lexists_reports_a_dangling_managed_link_as_installed`.
- **Asymmetric atomicity (D5).** `write_link` atomic (temp symlink + rename);
  canonical swap staged minimal-window (remove + rename), partial stage only under
  `.tmp-<id>`. Tests: `materialise_overwrites_stale_canonical`,
  `materialise_heals_an_interrupted_stage`.
- **Self-enforced ignore (F4).** `skills install` writes `.doctrine/skills/*` with
  no prior `doctrine install`. Tests: `run_install_self_enforces_the_skills_gitignore`,
  manifest test, `ensure_gitignored_*`.
- **list lexists (F5).** Dangling managed link reports installed.
- **Behaviour preservation.** `delegate_argv` + `AgentPlan::Delegate` + entity/
  slice/state suites green unchanged.

## Second independent pass (post-close-out)

A fresh `/code-review` over the same diff (`097370c~1..6f2f3e2`), tree green at
HEAD (277 unit + 2 e2e, clippy `-D warnings` clean). Goal: break what the above
claims green. A1/A2 confirmed held; the project-local never-clobber path could not
be broken. New findings (none blocking; no code changed — `no code without an
approved plan`):

- **B1 (bug, reachable, FIXED) — F4 ignore was wrong under `--global`.**
  `run_install` (`skills.rs:722`) unconditionally `ensure_gitignored(&root,
  ".doctrine/skills/*")`, but `--global` materialises the canonical tree at
  `$HOME/.doctrine/skills` (`canonical_dir`, `:206`), not under `root`. So on the
  reachable global path (`SkillsCommand::Install { global }`, main.rs:330) the
  invariant inverts: the tree written ($HOME) is left un-ignored, and a spurious
  `.doctrine/skills/*` is appended to the *project* `.gitignore`. F4's promise
  ("ignore the derived tree I just wrote") fails for global. No test covers
  `run_install` with `global=true`. Harm low ($HOME rarely a repo) but it is a
  load-bearing invariant breaking on a shipped path. Fix: derive the ignore root
  from the canonical base, or document `--global` ignore as out of scope.
  **Fixed:** extracted `install_base(root, global)` — the single base both
  `.claude/skills` and `.doctrine/skills` hang off — and anchored the F4 ignore
  there (`run_install:722`), so global writes the ignore beside its `$HOME` tree.
  Also kills the HOME/root duplication that `claude_dir`/`canonical_dir` carried.
  Test: `install_base_anchors_both_trees_and_the_ignore` locks "ignore base ==
  tree base" (global e2e not run — it would write the developer's real `$HOME`).
- **B2 (robustness, OPEN, low likelihood) — canonical `dest` removal not
  type-aware (asymmetric vs A1).** `materialise_canonical:399` still does
  `if dest.exists() { remove_dir_all(dest) }` two lines below the A1-hardened
  type-aware temp cleanup. A stray regular file / live symlink-to-dir at
  `.doctrine/skills/<id>` errors (`ENOTDIR`) instead of self-healing — the exact
  mode A1 fixed for the temp. Derived/gitignored tree so unlikely; the asymmetry is
  the trap. (A *dangling* symlink at `<id>` is safe — `exists()` false, `rename`
  heals.)
- **B3 (hardening, accepted-residual) — `write_link` Create not atomically
  clobber-safe.** `write_link:319` uses temp+`rename` for both Create and Relink;
  `rename` always wins, so A2's residual TOCTOU (foreign symlink appearing between
  mutation-time `classify_link` and the `rename`) is real for Create. A direct
  `symlink(target, dest)` on the Create arm fails closed (`EEXIST`) — true
  never-clobber, and simpler (temp+rename is only needed to *replace* on Relink).
  Design §5.5 accepts the residual window, so opportunity not blocker; recorded so
  the guarantee is read as "narrow window", not "closed", for Create.
- **B4..B6 (cosmetic/cohesion, optional).** `let _ = dest;` pointless bind in the
  `execute` KeepForeign arm (`skills.rs:448`); `Step::Gitignore.dest` is now
  display-only while `execute_plan` recomputes the path independently
  (`install.rs:270` vs `:313` / `ensure_gitignored`) — two sources that happen to
  agree; "refreshed" logged on first install (no prior canonical); unreadable
  symlink renders `(foreign symlink → )` with an empty path.

Disposition: B1 is the only one worth a fix decision (small, reachable). B2/B3 are
defensible-as-is given the derived-tree + §5.5-accepts-residual framing. B4–B6 are
cleanup. None block the slice; left for the owner to triage.

## Known limitations (documented, accepted)

- Ownership-spelling: a differently-spelled own link classifies foreign → kept,
  not healed (design §5.5/P2-F5; v1 emits one spelling).
- Out of scope: `--global` auto-detect + `list --global` (Q4); orphan prune (Q2);
  `DELEGATE_SOURCE` typo (slice-010.md follow-up).

## Environmental note (not an SL-010 defect)

- During close-out the shared working tree carried parallel **SL-008** WIP
  (`src/main.rs` `mod retrieve;` with no `src/retrieve.rs`, + `Cargo.toml`),
  which breaks a tree-wide `cargo build`. SL-010's committed src (skills.rs,
  install.rs, manifest) is independent and was validated in an isolated worktree
  at the SL-010 HEAD (no SL-008 WIP). Not harvested into SL-010.
