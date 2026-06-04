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
