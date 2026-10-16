# Notes SL-281: Install artefact ownership

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-16 · PHASE-06 completed · 9dff138fa

### Produced
- design.md (locked, run dr-01a1422f…, rev 50); RV-418 (13/13 verified; pass waived for missing routes)
- DEC-372, DEC-373, DEC-374, DEC-375, DEC-376, DEC-377
- backlog: IMP-513, IMP-514, IMP-515, IMP-516, IMP-517, ISS-520; IMP-406 annotated + after SL-281
- design-target selectors (9)
- plan.toml/plan.md (PHASE-01..06); REV-071 drafted + approved (SPEC-009, SPEC-010, REQ-167/170/172/176/180)
- RV-419 (plan-time amendment pass, codex; F-1 verified)
- PHASE-01: manifest [framework] + [harness.*] registry; detect/boot/delegate read it (017329e71)
- fixture git helpers GIT_GUARD=off (e523e95c0) — gate green
- PHASE-02: fsutil path algebra lift (38ab4932b); src/install_ownership.rs pure plan + STAGING_PREFIX (9d312ebc8); VT-1 PASS
- PHASE-03: exclusive staging (0fde6d4c6)
- PHASE-04: convergence shell unwired (05e20f76d), cutover (87503e595), dry-run + e2e (b76a31511)
- PHASE-05: managed gitignore block (4242b63a8)
- PHASE-06: dogfood managed install (9dff138fa)
- CHR-178: cargo 1.100 adoption (4a7c4f7ff, 5512d9a61, aec6fa420)
- RV-420 (audit pass, 16 findings raised; F-2 = this harvest, F-5 = pi link untracked, e09b9fef6)
- phase detail: § Phase record below

### Learned
- ISS-216's refresh half already fixed; real fault is stale-layout links kept as foreign
- `.doctrine/agents/AGENTS.md` is a repo-local relic, not install output
- pi skills duplication = `npx skills --agent pi --agent universal` from detect_agents
- boot.rs detector misses `.pi` (RV-418 F-11)
- plan-time: ensure_gitignored has no prod caller beyond Step::Gitignore → deleted (design amended); REV widened past design table (same class as RV-418 F-7)
- jail git-guard refuses checkout -f / reset --hard in test temp repos → 26 unrelated gate failures (observation cdcd408a6)
- friction observations: 7 recorded through PHASE-02 (apply flag, runbook text, ISS-356 recurrence, missing routes → IMP-517, verify-vt unattributable before phase completion); further ones recorded by PHASE-03..06 workers
- PHASE-02: links map to members by site (Member.links), not by value; foreign symlinks reported only at member sites (OQ-1, user-agreed)
- PHASE-02: Snapshot scan contract — canon entries at member depth + containers; plan also keeps anything beneath a member (PHASE-04 scanner honours it)
- new src module must be classified in .doctrine/adr/001/layering.toml or the layering gate fails Unclassified
- four duplicated test `git` helpers (dispatch, slice, ledger, e2e_dispatch_candidate) — candidate shared fixture
- cargo 1.100 moves test exes to `<profile>/build/<pkg>/<hash>/`; `test_support::doctrine_bin` needed a walk-up fix (CHR-178)
- a sheet's stated retire/edit counts can drift from the plan text; check the diff, not the sheet (PHASE-05 EX-4)

### Open
- REV-071 applies at /reconcile
- RV-420 dispositions pending beyond F-2 and F-5
- PHASE-06 OQ-5 — block has no `/.claude/agents/capsule-*.md` link lines (see Phase record); disposition not recorded on the sheet
- PHASE-06 OQ-6 — research baseline drift (`doctrine slice research 281 --restamp`), orchestrator's call

## Phase record

Lifted from the gitignored phase sheets (`.doctrine/slice/281/phases/`) and the
phase commits. Decision labels (D1..) are per-phase, as in each sheet.

### Assent (2026-10-16)

The user was shown an explicit list of the dispositions below and replied,
verbatim: "ok i'll take your dispositions". That reply accepts exactly:

- PHASE-04 D1, D3, D4, D6
- PHASE-05 D2
- PHASE-06 OQ-1

PHASE-04 D2 and D5 and PHASE-05 D1 and D3 were orchestrator decisions, not
user-assented. PHASE-03 recorded no decisions.

### Toolchain episode (PHASE-03..05)

An uncommitted `flake.lock` bump (someone else's) put the toolchain on
cargo/clippy 1.100. `doctrine check gate` went red on 41 pre-existing clippy
errors outside the slice, and real-binary e2e failed NotFound because
`test_support::doctrine_bin` looked in `deps/`, not `build/<pkg>/<hash>/`.

- PHASE-03..05 were verified by a **substitute gate**: plain `cargo clippy`
  (no `--tests`/`--all-targets`) with zero diagnostics in touched files, plus
  `cargo test --no-fail-fast -p doctrine -p cordage` green. e2e ran through
  **symlink shims** (`target/debug/build/doctrine/*/doctrine` -> the built
  binary). `e2e_surface_neutral_roundtrip` failed only because of the shim.
- Retroactive cover: CHR-178 adopted the beta properly (4a7c4f7ff file,
  5512d9a61 fix: `doctrine_bin` walk-up, root package inherits `rust-version`
  1.85, four post-1.85 API uses fixed, redundant clones and two unfulfilled
  `#[expect]` removed, flake.lock to 1.100.0-beta.5). A green
  `doctrine check gate` then ran at `aec6fa420` and again at `9dff138fa`,
  which covers the code PHASE-03..05 verified under the substitute.

### PHASE-03 — exclusive staging (0fde6d4c6)

- Plan followed. `staging_path` -> `.tmp-<name>.<pid>.<n>` (prefix from
  `STAGING_PREFIX`); `stage_from`/`stage_exclusive` retry on EEXIST (bounded,
  64), create via `symlink`/`create_dir`; `write_link` and
  `materialise_canonical` lose their stale-clear blocks and on failure remove
  only what they created. `materialise_canonical` now needs
  `create_dir_all(dest.parent())` first.
- No decisions or findings were recorded on the sheet.
- Swap window (remove-then-rename in `materialise_canonical`) deliberately
  untouched; still the open probe noted under Design review passes.
- Leftover `.tmp-*.<pid>.<n>` entries now accumulate after a crash by design;
  PHASE-02's plan reports them as stale staging and never prunes them.
- Gate: gate red on the toolchain baseline only (see Toolchain episode).

### PHASE-04 — convergence cutover

Split into three ordered, individually green commits: **a** shell parts
unwired (05e20f76d: `desired_set`, `scan`, `execute`, `action_line`, behind
`cfg_attr(not(test), expect(dead_code))`), **b** atomic cutover (87503e595),
**c** dry-run and nine-case e2e (b76a31511). Legs deleted: per-harness agent,
skills and workflows installers, both `_ => pi` arms, the dispatch-worker
asset constants, `classify_link`, `reconcile_link`, `relative_target`.
`fsutil::symlinked_component` added for the ancestor lstat.

Decisions (sheet proposed; final status here):

- D1 no `relinked` line for an already-correct link; idempotent re-runs are
  silent for links. **User-assented.**
- D2 `Shadows` wording for real files and real dirs alike (old `kept … (real
  dir)` line goes). **Orchestrator decision.**
- D3 convergence runs once after the per-agent prompts, only when at least one
  harness was confirmed. **User-assented.** (The gitignore block, PHASE-05, is
  written regardless.)
- D4 executor aborts on the first I/O or bake error. **User-assented.**
- D5 shell code stays in `src/install.rs`; generic `symlinked_component` in
  `src/fsutil.rs`. **Orchestrator decision.**
- D6 `framework_roots()` is test-only (`#[cfg(test)]`). **User-assented.**

Plan adaptations and gotchas:

- Sheet error corrected: the VT-3 mapping on the sheet was wrong; VT-3 is
  `plan_excludes_marketplace_only_skills_from_members` in `src/install.rs`
  (marketplace-only skills never desired members), landed in sub-task a. The
  plan.toml wording is authoritative.
- Member identity: build `members` once, select by clone; a skill member with
  fewer `links` is a different member and would be written twice.
- Scan reads only; it never creates a directory (dry-run asserts no
  `.doctrine`/`.pi` appears).
- Bake loads the hymn corpus once per run, lazily. Global installs still bake
  against the project corpus (ISS-520, untouched).
- `with_wanted` shares desired-set wiring between dry-run and live `converge`.
- Sheet rule: never run `doctrine install` against this repo before PHASE-06.
- Substitute gate and shims apply (Toolchain episode). No findings recorded.

### PHASE-05 — managed gitignore block (4242b63a8)

Pure core in `install_ownership.rs`: `doctrine_links`, `render_block`,
`splice_block` (`SpliceRefusal::Malformed{starts,ends}`), `links_after`
(predicts the post-run link set for `--dry-run`). Shell writes the block last
from a re-scan of registered link dirs (no second walker). `Step::Gitignore`,
`ensure_gitignored`, `read_gitignore_lines` deleted; `[gitignore].entries`
drops `.doctrine/skills/*` (now from the roots glob).

Decisions:

- D1 dry-run predicts links as disk links plus the plan's `Link`/`Relink`
  dests minus `Unlink` paths (`links_after`) rather than disk-only.
  **Orchestrator decision.**
- D2 malformed markers: file left untouched, marker counts reported, install
  exits 0. **User-assented.**
- D3 block is written even with zero confirmed harnesses (and after a
  framework-step failure), because it reflects what is on disk and
  `e2e_memory_sync` expects the shipped-corpus entry. **Orchestrator decision.**

Plan adaptations and gotchas:

- **EX-4 retired 5 tests, not 2.** plan.toml EX-4 says "the two
  `ensure_gitignored_*` tests retire"; the sheet's T5 and the commit retire
  five: both `ensure_gitignored_*`, `plan_includes_gitignore_entries`,
  `gitignore_skips_duplicate_entries`, `execute_appends_gitignore`. All tested
  only the deleted append path; T1/T3 cover the behaviour. The plan text is
  stale on the count.
- RV-419 F-1 migrations done: `observation_publication_temps_are_ignored` and
  `embedded_manifest_ignores_the_skills_derived_tree` now assert via
  `render_block`.
- Disclosure line always printed: `npx skills` copies (`.agents/skills/<id>`)
  are not listed in the block.
- Gitignore has no inline comments; only the two marker lines carry `#`.
- User `!` negations survive because the block is replaced in place, not
  rewritten from the top.
- Substitute gate and shims apply. No findings recorded.

### PHASE-06 — dogfood (9dff138fa)

Installed this repo with the fresh dev binary. T7 changes (from the diff
against the pre-install copies):

- `.claude/agents/capsule-{orchestrator,phase-planner,worker}.md` hand copies
  were deleted (untracked) and replaced by install-owned symlinks into
  `.doctrine/agents/claude/`; `dispatch-worker.md` relinked from the flat
  layout to `claude/`. Phase-planner and worker were byte-identical to the
  embedded source.
- `capsule-orchestrator.md` gained the shipped "Never pass `name` either: a
  named child becomes a teammate" line (the hand copy was older). PHASE-06
  OQ-1, **user-assented**.
- `.pi/agents/dispatch-worker.md` link moved from the codex canon to the pi
  canon (`.doctrine/agents/pi/`), a tracked change (`.pi/agents/dispatch-worker.md`);
  it was later untracked by RV-420 F-5 (e09b9fef6).
- `.gitignore`: the managed block landed; loose lines it owns were dropped
  (state, phases, handover, memory index/embeddings/state/shipped, dispatch,
  observation temps, `.doctrine/agents/*` and its `!AGENTS.md` negation,
  `.doctrine/skills/*`, inquisition). Kept outside the block: `/.claude/*` and
  its negation, `/.agents`, `/.pi/*` and negations, `.doctrine/slice/*/research/`.
- The observation-records comment block that sat in `.gitignore` was lost with
  the loose lines. Its rationale (records are authored; only `.tmp.` publication
  temps ignored; do not broaden) already lives in `install/manifest.toml`
  beside the entry, so nothing is orphaned.
- `.doctrine/agents/AGENTS.md` removed from tracking (OQ-2 resolved: staged
  removal, committed by the orchestrator). `git ls-files .doctrine/agents` is
  empty.
- Install pruned `.doctrine/agents/{codex,universal}`, the flat
  `dispatch-worker.md`, and `dispatch-agent`/`dispatch-subprocess` skills and
  links (OQ-4: accepted, all untracked).
- VT-1 `repo_gitignore_contains_static_block_entries` added in `src/install.rs`;
  derives the static lines from `render_block`, no hand-copied list.
- Seven friction observation records committed with the phase.

Open at harvest:

- OQ-5 — the block has no link lines for `/.claude/agents/capsule-*.md`, so in
  a client repo those symlinks would show untracked (this repo hides them under
  `/.claude/*`). Possible gap in PHASE-04/05; not fixed here; needs a backlog
  item or a reopen.
- OQ-6 — research baseline drift; restamp is the orchestrator's call.
- OQ-7 (T3/T4 spawn window): no outcome recorded on the sheet.

## Design review passes

- **RV-418** (adversarial, codex `gpt-6-sol`, 4 rounds, 2026-10-16): 13
  findings, all verified. The hard part was F-1, prune and staging under
  symlinked ancestors. It took three rounds to settle on: removals gated by an
  ancestor `lstat`; writes and relinks limited to member paths; staging always
  exclusive-create and never pruned.

**Further pass: none needed before lock.** The design surfaces that remain
unprobed by RV-418 are implementation-level, and `/audit` covers them better
than another design pass:

- that `discover()` and the registry seams compose as the Code impact table
  claims;
- the exact report-line wording;
- the REV's before/after excerpts, which are drafted at `/plan` and approved
  separately.

If anything, a later pass should probe the staging rule's interaction with
`materialise_canonical`'s remove-then-rename window once code exists.
