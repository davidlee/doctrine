# Notes SL-008: Memory retrieval: find/retrieve + scope ranking + staleness

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — durable decisions (commit 5a826c2)

- **Location-probe match model (NEEDS DESIGN FOLD-IN at close-out).** `match_scope`
  treats the query as a working *location* (`paths ∪ globs`, as path subjects) +
  facets (`commands`, `tags`); a memory matches if its scope ADMITS the location
  via any dimension, highest-specificity dim wins. Per-dim admit rules: paths
  component-prefix (3), globs `**`-aware via the `glob` crate (2), commands
  token-prefix (1), tags set-intersection (0). A query PATH probes both scope.paths
  AND scope.globs. This resolves **codex review F1** + the design open-Q on match
  *direction* — design.md currently pins only the specificity *table*, not the
  direction. **Action:** add a D-decision/addendum to design.md when the slice
  closes (the codex review log at design.md bottom should gain an F1-resolved line).
- **`glob` crate added** (`glob::Pattern::matches`, `**`-aware) as a workspace dep —
  user decision over hand-rolling. A malformed stored pattern = non-match, never a
  reader hard-fail (the store is tool-authored; reader degrades).
- **`days_between` / `parse_ymd` live in `src/retrieve.rs`** (not deferred to
  PHASE-02): one pure YYYY-MM-DD primitive shared by `thread_expiry` and PHASE-02
  staleness/recency — no parallel impl. Already `pub(crate)`; PHASE-02 reuses it.
- **Pure layer parked under module-level `#![expect(dead_code, reason=…)]`** — it
  has no shell caller until PHASE-04, and the expectation self-clears (errors) once
  PHASE-04 wires it. Do not delete it early; do not switch to `#[allow]` (denied).

## PHASE-02 — durable decisions (pure ordering core)

- **`staleness` signature is `(m, GitFacts /*by value*/, today)` — NO `target`
  param.** Design §5.5 branch-1 lists a `target.is_some()` gate; that gating is
  pushed to the PHASE-04 shell (it resolves real `GitFacts` only when target is
  `Some`, else passes `commits_since: None`). Consequence the shell must honour: a
  scoped+attested memory queried with `target == None` lands in branch 1 and yields
  `Unknown` (undecidable), NOT the time branch. This is the deliberate
  divergence from design's "target absent ⇒ time mode" — flagged for the close-out
  audit / design fold-in. `GitFacts` is `Copy`; passed by value (clippy
  `trivially_copy_pass_by_ref`).
- **Staleness = three spec modes → four states, first-match cascade.** Branch order:
  (1) scoped∧attested ⇒ commit mode; (2) parseable `reviewed` ⇒ time mode (≤30
  Fresh); (3) anchored (`kind != None`) w/ no usable date ⇒ Unknown; (4) no anchor
  ⇒ Unanchored. The missing-date case is split by anchor presence (3 vs 4) — that is
  how `Unanchored` stays reachable (design §5.5's "branch 3 = branch 2's missing-date
  arm" reconciled). `FRESH_DAYS = 30` inclusive; distinct from thread's 14.
- **Lexical OPEN (phase-sheet) RESOLVED → SET semantics.** Score = count of
  *distinct* query tokens that hit the doc token bag (title+summary+tags+key
  segments); repeats count once. Body not scanned (B15). `Memory` carries no body
  field, so "body excluded" is structural, not a guard.
- **Rank is a 10-element tuple `cmp`** (the §5.2 9-key table; key 9 = uid then
  memory_key). `today: &str` passed into `rank` (recency computed inside, not a
  Candidate field — one date source). `Ranked` is just sorted `Vec<Candidate>`, no
  wrapper. Polarity asserted per-key (`rank_keyN_*` tests); staleness is display
  only — proven NOT an Ord key by `rank_verification_stale_not_double_penalised`.
- **Test `Fixture` widened** to carry uid/key/title/summary/`[git]`/trust/severity/
  weight so ranking + staleness branches drive the real `Memory::parse`, not
  hand-built structs (test behaviour via the real parser).

## PHASE-03 — durable decisions (impure git seam)

- **`commits_touching(root, paths, since, target) -> Option<u32>` in `src/git.rs`**
  is the ONE new git the reader needs; its return IS `GitFacts.commits_since`
  (`Some(0)` Fresh / `Some(≥1)` Stale / `None` Unknown). Rides the SL-007 runners
  (`run_git` for the precheck — needs raw `ExitStatus`; `git_opt` for the count),
  adds no new SL-007 surface, resolves no HEAD (codex F1 — `target` is always a
  frozen SHA the PHASE-04 shell hands in).
- **F2 ancestry gate is mandatory, not an optimisation.** `git merge-base
  --is-ancestor since target` runs *before* the count; non-success ⇒ `None`. Exit 1
  (non-ancestor) and exit ≥2 (object absent / shallow) both fold to `None` via one
  `status.success()` check. Skipping it lets `since..target` (a set difference)
  silently over-count a non-ancestor `since` — the slice's headline trap.
- **Everything folds to `None` (Option, never Result).** Per-candidate degradation
  (B18): exec/parse/non-ancestor/missing-object all ⇒ `None` ⇒ `Staleness::Unknown`,
  so one bad candidate never aborts the query. Cost — real git breakage reads as
  `Unknown` — accepted because staleness stays visible (D19). `CaptureError`
  swallowed with `.ok()`.
- **Guards short-circuit before any subprocess:** empty `paths` (B17) AND empty
  `since`/`target` (defence in depth past the PHASE-04 gate). Proven by a `None`
  against a non-repo temp dir (no spawn dependency).
- **`--` pathspec separator is load-bearing** (a path equal to a ref name is
  otherwise ambiguous); the pathspec genuinely narrows — a commit touching only
  other paths yields `Some(0)`. `let range = format!(…)` bound before the args vec
  borrows `&range`.

## PHASE-04 — durable decisions (find shell + shared pipeline)

- **The impure shell lives in `src/retrieve.rs`** alongside the pure core it drives
  (mirrors `memory.rs`'s pure-render / impure-`run_*` split). `freeze` (one
  `capture` + one `clock::today`), `git_facts` (the §5.1 gate), the shared `query`
  pipeline, `format_find`, `run_find`. The PHASE-01 module-level
  `#![expect(dead_code)]` is RETIRED — the shell makes the pure layer live.
- **No `Ranked` type — `Candidate` post-`rank` IS the ranked unit.** Design writes
  `Vec<Ranked>` conceptually; impl returns `Vec<Candidate<'a>>` borrowing an owned
  `Vec<Memory>` the caller holds. The shared `query(mems, q, snap, include_draft,
  root)` is surface-agnostic — **PHASE-05 retrieve reuses it verbatim** (EX-6/F3),
  reading bodies per `take(limit)` hit via the existing `resolve_show` seam.
- **GitFacts gate = staleness branch-1 cond + `Some` target:**
  `!scope.paths.is_empty() && !verified_sha.is_empty() && target.is_some()` ⇒ call
  `commits_touching`; else `GitFacts::default()` (no subprocess). Keeps git cost to
  candidates whose staleness actually needs it.
- **`capture` errors degrade, never fail** (B18/B19): `capture(root).ok()` ⇒
  `target None` + `repo None`. A multi-root/submodule tree then finds only
  repo-empty (global) memories, visibly `Unknown`/time-mode. Hides genuine git
  breakage — accepted v1 (D19 visibility); a future `--explain`/warn could surface.
- **DRY reuse, no parallel impl:** promoted `memory::{collect_memories, select_rows,
  scrub_line, MEMORY_ITEMS_DIR, WORKSPACE}` to `pub(crate)`. `--type/--status` ride
  `select_rows` (F2); `--tag` is a SCOPE dimension (`QueryContext.tags`), so
  `select_rows` is called with `tag_f = None`. `format_find` reuses `scrub_line`
  (F-A10) so a newline in a value cannot forge a row.
- **find row uid = FULL uid**, superseding design §5.2 `uid-short` (F-A11 actionable
  + F-A12 v7 prefixes collide). design.md §10 review log carries the fold-in
  F-finding. find is holdback-EXEMPT (D8/D17): trust+sev are visible COLUMNS, not
  filters; suppression is PHASE-05's job. `base_filter` already drops
  quarantined/retracted/superseded/archived (draft unless `--include-draft`).
- **CLI `--path-scope`** (not `--path` — `-p/--path` is the root flag, same
  collision `record` resolved identically). `run_find` carries
  `#[expect(clippy::too_many_arguments)]` (CLI surface fans flags 1:1).
