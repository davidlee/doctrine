# Notes SL-029: Dispatch worktree creation: detection and creation paths with guards

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — CLI heart (`src/worktree.rs` + `provision`/`check-allowlist`)

Shipped in `feat(SL-029): worktree provision …`. All 9 VTs discharged; `just
check` green, plain clippy zero.

### Decisions taken during execution

- **D-1 Glob match options (load-bearing).** All matching goes through one
  `MATCH_OPTS` const with `require_literal_separator = true`: `**` is the *only*
  cross-`/` wildcard, so a single `*` matches exactly one path component. Without
  this a bare `*` in `.worktreeinclude` would silently span `.doctrine/state/...`.
  `**` still recurses under this flag — verified by VT-3/VT-5.
- **D-2 `is_withheld`/`select_copies`/`allowlist_violations` are non-fallible
  (no `Result`).** The static `WITHHELD` globs are constants; rather than thread a
  compile-error `Result` through the whole pure core, `is_withheld` swallows a
  (impossible) `Pattern::new` error via `.ok()`, and a unit test
  (`withheld_globs_all_compile`) is the guard that they all compile. Keeps the
  call sites clean; the only `Result` in the pure layer is `parse_allowlist`
  (user input genuinely fails).
- **D-3 `allowlist_violations` works by *representative path*.** For each WITHHELD
  glob, build a concrete sample (`**`/`*`/`?` → literal `x`, e.g.
  `.doctrine/state/**` → `.doctrine/state/x`) and flag any allowlist pattern that
  matches it. This is why a broad `**` IS flagged (fail-closed at the static gate)
  yet `select_copies` can still be the deeper guarantee — the two layers use the
  same matcher but at file vs representative granularity. Note the F7 gap is real:
  a pattern crafted to match deep tier files but not the one-level representative
  (e.g. `.doctrine/state/*/**`) slips the static gate — `select_copies` still
  withholds it at copy time. That asymmetry is the whole point of two layers.
- **D-4 git-seam reuse (R-a resolved, NO `/consult`).** Widened only
  `git_bytes`/`git_text` to `pub(crate)`. These are the generic normative-flag
  runners, not born-frame internals (`capture` et al. stay private), so exposing
  them did not pollute git.rs's focus — no third runner, no STOP triggered. The
  module-level `expect(dead_code)` stays fulfilled because `capture` and friends
  remain dead in the non-test build.
- **D-5 fsutil↔worktree cycle avoided (B5).** The per-file symlink copy lives in
  `fsutil::copy_selected`; its "is this symlink target withheld?" check is an
  injected `&dyn Fn(&Path) -> bool` predicate, so fsutil never imports worktree.
  `worktree` → `fsutil` (copy) is the only edge; no cycle.

### Sharp edges for the next agent / PHASE-02

- **`git ls-files --others --ignored` and whole-ignored directories.** A directory
  ignored *as a whole* (`/build/` in `.gitignore`) can collapse to a single dir
  entry rather than its files. The e2e fixtures deliberately ignore by extension
  (`*.txt`, `*.md`) so candidates enumerate individually. If a real project relies
  on dir-level ignores, confirm enumeration granularity before trusting coverage.
- **Sibling-worktree check is by canonical `git-common-dir`.** `provision` resolves
  `rev-parse --git-common-dir` for both source and fork and compares canonicalized
  paths; a separate repo (even a clone) is refused (VT-6). Relies on both roots
  being real git worktrees of the *same* common dir.
- **Provision is the SOLE copier — `select_copies` is the guarantee, not
  `check-allowlist`.** Do not let PHASE-02 skill-prose imply the static gate is
  completeness (design F7). The copy-seam guarantee only holds because no native
  hook copies; if a harness force-copies on creation, the guarantee degrades to
  the static check (design §2 caveat) — keep that honest in the `/worktree` prose.
- **PHASE-02 (`/worktree` + `/execute` skills) is unstarted.** Commit-before-spawn,
  branch-point check, baseline-verify (`just check`) are all skill-prose per design
  §4; the Rust does the copy axis only. `.worktreeinclude` is project-owned, NOT
  installed (design F2) — `/worktree` documents a commented template.

## PHASE-02 — `/worktree` lifecycle skill + `/execute` solo-isolation thread

Shipped: T1 `feat(SL-029): /worktree lifecycle skill …` (511e4f4); T2 + T3 this
pass. `just check` green; both VA read-throughs PASS (skill prose matches design
§2/§4/§5 in substance; `/execute` thread matches §5 clause-for-clause).

### Decisions / findings (durable subset; full set in the phase sheet)

- **`/worktree` is slice-agnostic.** It takes `mode`/`allow_work_in_place`/branch
  as inputs; the slice→branch mapping (`slice/SL-NNN-slug`, dir `.worktrees/SL-NNN`)
  lives in the `/execute` thread (PD-3), NOT in `/worktree` — so the funnel reuses
  it for non-slice worker branches.
- **`/execute` isolation is opt-in only, worker-mode OFF (D6a).** The default
  in-tree path is unchanged; `/dispatch` SKILL untouched. The opt-in branch sits
  before step 5 (the TDD loop runs inside the fork).
- **Skill refresh ≠ `doctrine install`.** `doctrine install` only ensures the
  `.doctrine/skills/` dir exists (content `skip`ped as exists). New SKILL.md /
  NOTICE.md content + `.claude/skills/<id>` relink land via `doctrine skills
  install -y`. And a lone `plugins/` edit does NOT re-embed on `cargo build` (no-op
  in <1s) — `touch src/skills.rs` first to force the embedding crate to recompile
  ([[mem.pattern.build.rust-embed-no-rerun]]). Working sequence: touch → build →
  `skills install`. (Phase-sheet F-5.)
- **NOTICE.md ships automatically** via the skills dir-copy (F-4) — no install
  change needed.
