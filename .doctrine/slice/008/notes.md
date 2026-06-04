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
