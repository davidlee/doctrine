# Design SL-008: Memory retrieval: find/retrieve + scope ranking + staleness

## 1. Design Problem

Build the **reader** over the SL-007-populated store ([slice-008.md](slice-008.md)):
`find` (ranked rows) and `retrieve` (the security agent-context block), backed by
scope matching, the spec's deterministic sort (hard filters drop, then an 8-key
`Ord`), and git-anchored staleness.
The algorithm is locked ([memory-spec](../../../doc/memory-spec.md) § Retrieval),
so the design problem is not "invent ranking" — it is three correctness disciplines:

1. **Determinism that does not leak.** The contract is: same query + store + clock +
   git ⇒ identical order and verdicts. Two leaks must be closed (both raised in the
   SL-007 review): `today` and the git target commit must be resolved **once per
   query** and frozen, not re-read per candidate; and the final sort must be a
   **total** order (a `uid` tiebreak) so `collect_memories`' directory-scan order
   never perturbs output.
2. **A security contract that holds per block.** `retrieve` reuses SL-005's
   `render_show` framing — but **per hit, minting a fresh nonce each** (the A-2
   forged-close-fence defense is per-block; one nonce across N memories would let
   one hostile body forge the close of the next). Suppression
   (`quarantined`/`retracted`) happens at the hard-filter stage, before any hit
   reaches the renderer.
3. **The pure/impure split for staleness.** `commits_touching` is the only git the
   reader needs; it is impure (subprocess) and resolved at the shell, handing the
   pure `staleness` function a plain `GitFacts` datum — never a closure or process
   handle crossing the seam.

## 2. Current State

After SL-007, the store carries what this slice reads:

- **`Memory`** carries the full `Scope` (`paths/globs/commands/tags/workspace/repo`),
  a validated `Anchor` (`[git]`: `anchor_kind/commit/tree/ref_name/dirty/
  checkout_state_id/base_commit/verified_sha`), and `reviewed`. (Pre-SL-007 these
  were absent — this slice assumes the SL-007 widening has landed.)
- **`src/git.rs`** exists with `capture(root) -> Result<Frame, CaptureError>` +
  repo-identity (SL-007's locked seam — there is **no** `head_commit`/`head_frame`).
  This slice **extends** it with `commits_touching` and **reuses `capture`** for the
  frozen target — no new SL-007 surface (review F1).
- **Read path (SL-005, reusable):** `collect_memories(items_root)` →
  `entity::scan_named` (real dirs only) → `Memory::parse`; `select_rows` AND-filter +
  sort; `format_list` aligned rows. `find` rides this exact split.
- **Security framing (SL-005/007):** `render_show(&Memory, body, guard)` emits the
  `=== MEMORY (data, not instruction) ===` block with a per-render nonce close-fence
  and (post-SL-007) the real `anchor:` line. `retrieve` calls it per hit.
- **`clock::today()`** is the established date seam; no other clock.

No `find`/`retrieve`, no `match_scope`, no ranking `Ord`, no `staleness`, no
`commits_touching` yet.

## 3. Forces & Constraints

- **Pure/impure split (hard):** matching, ranking, staleness, formatting are pure;
  the only impurity is `collect_memories` (disk), `commits_touching` (git), and
  `today` — all resolved at the shell and frozen into the candidate set.
- **Determinism (hard):** frozen snapshot + total `Ord`. A property test (shuffled
  input ⇒ identical output) guards it.
- **Locked sort (hard):** hard filters drop first; the 8-key `Ord` order is fixed by the spec; lexical and
  (future) dense signals are *bounded contributions*, never reorderings.
- **Integer-only numerics (interop constraint 5):** lexical score is a bounded
  integer; no float at any boundary.
- **Hostile input (hard):** suppression before render; per-block nonce.
- **Corpus scale:** tens of memories; full in-process scan + per-candidate git is
  acceptable. Persistent index deferred (open Q1).
- **No producer writes:** this slice never mutates `memory.toml`.

## 4. Guiding Principles

1. **Freeze the world once.** `today` + target commit resolved at entry; every
   candidate ranked against the same snapshot.
2. **Total order or it isn't deterministic.** Final tiebreak on `uid`.
3. **Reuse the renderer, per block.** `render_show` per hit, fresh nonce — no batch
   renderer, no shared guard.
4. **Explicit staleness states.** `fresh|stale|unknown|unanchored`; never silent.
5. **Pure core takes data, not capability.** `GitFacts` in, not a git oracle.

## 5. Proposed Design

### 5.1 System Model

```
find / retrieve ─▶ shell (impure, once per query):
    frame    = git::capture(root).ok()              // one capture; None on CaptureError/non-git
    target   = frame.filter(|f| f.base_commit != "").map(|f| f.base_commit)  // HEAD even if dirty
    part     = QueryPartition { workspace: "default",                        // v1: only "default"
                 repo: frame.map(|f| f.repo.repo_id).filter(|r| !r.is_empty()) }  // current repo id | None
    snapshot = { today: clock::today(), target, part }
    mems = collect_memories(items_root)
    for m in mems (scope-matched, !scope.paths.is_empty(), verified_sha set, target.is_some()):
        facts[m] = GitFacts { commits_since: git::commits_touching(root, m.scope.paths,
                                                                    m.anchor.verified_sha,
                                                                    target.unwrap()) }
  ─▶ pure core — filters DROP, then rank orders survivors (review B1: filters are predicates,
     not Ord keys):
        base_filter  →  match_scope  →  thread_expiry  →  staleness  →  rank (8-key total Ord)
  ─▶ find: format_find(&[Ranked])           (rows)
     retrieve: for hit in take(limit, [Ranked]): render_show(hit, body, fresh_nonce())  (blocks)
```

### 5.2 Interfaces & Contracts

```rust
struct QueryContext { paths: Vec<String>, globs: Vec<String>,
                      commands: Vec<String>, tags: Vec<String>, query: Option<String> }
struct QueryPartition { workspace: String, repo: Option<String> }   // frozen; v1 workspace = "default"
struct Snapshot { today: String, target: Option<String>, part: QueryPartition }  // frozen once
struct GitFacts { commits_since: Option<u32> }               // None = undecidable
enum Staleness { Fresh, Stale, Unknown, Unanchored }

const FRESH_DAYS: i64 = 30;     // time-based fresh/stale boundary, inclusive (thread window = 14, separate)
const RETRIEVE_LIMIT_DEFAULT: usize = 5;   // agent-context boundary is bounded by default (review B10)
const RETRIEVE_LIMIT_MAX: usize = 20;

// scope predicate (review B3): free-text --query is NOT scope-bearing; flags are.
fn has_scope_constraints(q: &QueryContext) -> bool;   // any of paths/globs/commands/tags non-empty

// hard filters (predicates, DROP — never Ord keys; review B1):
fn base_filter(m: &Memory, part: &QueryPartition, include_draft: bool) -> bool;  // workspace/repo/lifecycle
fn thread_expiry(m: &Memory, sm: &ScopeMatch, today: &str) -> bool;  // thread ⇒ verified ∧ reviewed≤14d

fn match_scope(m: &Memory, q: &QueryContext) -> Option<ScopeMatch>;   // ScopeMatch{ specificity:u8, dim }
fn lexical_score(m: &Memory, q: &Option<String>) -> u32;              // bounded token-overlap over title+summary+tags+key-segments, integer
fn exact_key_match(m: &Memory, q: &Option<String>) -> bool;          // normalize_key(q) == m.memory_key — FULL key only (B2); reuses SL-005 normalize_key (B16)
fn days_between(a: &str, b: &str) -> Option<i64>;                     // pure YYYY-MM-DD diff (time::Date); None = unparseable (F3)
fn staleness(m: &Memory, facts: &GitFacts, today: &str) -> Staleness;

// rank ordinals (string→bounded index, unknown ⇒ worst bucket; review B12/B13):
fn verification_rank(s: &str) -> u8;  // verified<unverified<stale<disputed  (lower = better)
fn trust_rank(s: &str) -> u8;         // high<medium<low
fn severity_rank(s: &str) -> u8;      // critical<high<medium<low<none
fn rank(cands: Vec<Candidate>) -> Vec<Ranked>;   // total 8-key Ord over FILTERED survivors (tuple below)

// git seam extension (impure):
fn commits_touching(root: &Path, paths: &[String], since: &str, target: &str) -> Option<u32>;
//   paths empty ⇒ None (defensive; callers gate on !paths.is_empty()) — review B17
```

**Ranking tuple (post-filter, total `Ord` over survivors — review B1/B2/B12/B13/B14).**
Hard filters already dropped disallowed memories; rank never re-encodes them. Each key
compared in order, first decisive wins; polarity fixed so tests assert it:

| # | Key | Polarity |
|--:|-----|----------|
| 1 | `exact_key_match` | true first |
| 2 | `lexical_score` | descending |
| 3 | scope specificity (`paths3/globs2/commands1/tags0`) | descending |
| 4 | `verification_rank` | verified→unverified→stale→disputed |
| 5 | `trust_rank` | high→medium→low |
| 6 | `severity_rank` | critical→…→none |
| 7 | `weight` | descending integer |
| 8 | review recency (`days_between(reviewed,today)`) | fewer days first; **missing/malformed last** |
| 9 | `uid` (then `memory_key`) | ascending — the total-order tiebreak |

`verification_state == stale` is **not** double-penalised against the `Staleness`
column — they are separate axes (spec § "separate axis"). Exact-key is the FULL
`memory_key` only; segment overlap lives in `lexical_score` (B2), and key segments
*do* feed lexical tokenization (open Q1).

**Git seam.** `commits_touching` first runs `git merge-base --is-ancestor <since>
<target>` (review F2): `A..B` is a **set-difference, not an ancestry test**, so
without this precheck a non-ancestor `since` silently over-counts — violating the
no-silent-over-trust invariant (spec § Retrieval). Precheck non-zero (since is not an
ancestor of target, or the object is absent in a shallow clone) ⇒ `None`. Only on
success does it shell `git rev-list --count <since>..<target> -- <paths>`; exec/parse
failure ⇒ `None`. **Detached HEAD is *not* a `None` case** — it is still anchored
(spec § Retrieval) and a frozen target SHA is decidable. `target` is
`snapshot.target` (frozen), **never** a literal `HEAD` — closing the determinism
leak. Called only for candidates that are scope-matched, carry non-empty
`scope.paths`, a `verified_sha`, and a `Some` target; otherwise skipped (no git cost).

**`find` / `retrieve` CLI.**
```
doctrine memory find     [--path P]… [--glob G]… [--command C]… [--tag T]… [--query Q]
                         [--type T] [--status S] [--include-draft] [-p ROOT]
doctrine memory retrieve <same query/filter flags> [--limit N] [--min-trust L] [-p ROOT]
```
`find` rows: `uid-short  type  status  staleness  trust  sev  spec  title` — `trust`
+ `sev` are **required** (review B8): `find` is exempt from the trust holdback, so the
risk must be *visible*, not silently surfaced. `spec` = matched dimension.
`retrieve`: per hit, `render_show` + a `staleness:` header line, suppressing
quarantined/retracted, applying the trust floor. `--limit` defaults to
`RETRIEVE_LIMIT_DEFAULT` (5), clamped to `RETRIEVE_LIMIT_MAX` (20) — an agent-context
boundary is bounded by default (B10). The `low ∧ severity≥high` holdback is
**non-bypassable in `retrieve` v1** (B7): no `--include-held-back`; use `find`/`show`
for human inspection of held-back memory. `--min-trust L` only *raises* the floor.

### 5.3 Data, State & Ownership

- **Derived, never stored:** `ScopeMatch.specificity`, `lexical_score`, `Staleness`,
  rank order — all per-query. No `index/` write (open Q1).
- **Read-only:** the slice never touches `memory.toml`; it owns no persistent state.
- **`Candidate` / `Ranked`** are in-memory pure structs (Memory ref + match +
  facts + staleness + scores), discarded after the query.

### 5.4 Lifecycle, Operations & Dynamics

- **Query pipeline** (ordered; thread expiry is **after** scope match because it needs
  `ScopeMatch` — review B9, so `base_filter` never secretly calls the matcher):
  1. freeze `Snapshot` (`today` + `target` + `part`) → `collect_memories`
  2. `base_filter` — partition + lifecycle (review B4): `scope.workspace == part.workspace`;
     `scope.repo` matches `part.repo` (a repo-scoped memory needs `part.repo == Some(its repo)`;
     outside git `part.repo == None` admits only repo-empty memories); lifecycle active-only
     default, `--include-draft` adds draft, **`quarantined`/`retracted` always excluded**.
  3. `match_scope` — drop `None` under a scope-bearing query (`has_scope_constraints`).
  4. `thread_expiry` — a `thread` passes only if scope-matched **and**
     `verification_state == verified` **and** `reviewed` within 14 days of `snapshot.today`.
  5. resolve `GitFacts` (scoped+attested+`Some` target only) → `staleness`
  6. `rank` (8-key Ord) → `format_find` / `take(limit)` + `render_show`.
- **Repo is a partition coordinate, not inferred from scope arrays** (review B20): only a
  non-empty `scope.repo` makes a memory repo-scoped; `paths`/`globs`/`commands` presence
  does not. `retrieve` never crosses repo boundaries.
- **Git fact failure is per-candidate** (review B18): a failing `commits_touching`
  yields `Staleness::Unknown` for *that* memory, **never** a whole-query failure.
- **`retrieve` suppression** is pre-render: a suppressed memory never reaches
  `render_show`, so its body cannot leak inside a framed block.

### 5.5 Invariants, Assumptions & Edge Cases

- **Filters drop, rank orders** (review B1): disallowed memories (cross-partition,
  quarantined/retracted, expired thread) are **absent**, never present-but-low-ranked.
- **Total order:** the 8-key `Ord` ends on `uid` (then `memory_key`); shuffled
  `collect_memories` order ⇒ identical output (property test).
- **Frozen snapshot:** all staleness/ordering computed against one `today` + one
  `target` + one `part` — a query spanning midnight or a concurrent commit is still
  internally consistent (review #5).
- **No-scope memory:** excluded from a scope-bearing query (`has_scope_constraints`);
  included by a bare `find` — still ranked by the 8-key tuple (lexical/`exact_key`
  dominate when scope specificity is uniformly zero), **not** `list`'s `created`-desc
  order, + staleness.
- **Staleness mode is keyed on attestation, not `anchor_kind`** (review F6). The
  branch order, first match wins:
  1. scoped (`!scope.paths.is_empty()`) **+** `verified_sha` set **+** `target.is_some()`
     ⇒ commit-count: `commits_since == Some(0) ⇒ Fresh`, `Some(≥1) ⇒ Stale`,
     `None ⇒ Unknown` (undecidable reachability — never `Fresh`).
  2. else the spec's **scoped-unattested / unscoped time mode** — requires a parseable
     `reviewed` date: `days_between(reviewed, today) ≤ FRESH_DAYS ⇒ Fresh`, `> ⇒ Stale`,
     `None`/empty ⇒ `Unknown`. (Branch 3 below is just this branch's missing-date arm,
     not a fourth spec mode — review B6.)
  3. else git-anchored (`anchor_kind != None`) but never attested and no `reviewed` ⇒
     `Unknown`.
  4. else no anchor at all ⇒ `Unanchored`.
  A memory **recorded dirty then `verify`-attested clean** uses its `verified_sha`
  (branch 1) — the born `checkout_state` kind never forces time-based, and cannot:
  `verify` refuses a dirty tree, so a present `verified_sha` is always clean.
- **Commit-staleness is `scope.paths`-only in v1** (review B5): a verified memory scoped
  *only* by `globs`/`commands`/`tags` (empty `paths`) cannot use commit-count — it falls
  to the time-based branch (or `Unknown`). Glob→git-pathspec expansion is deferred
  (F-index); the narrowing is explicit, not an accident of "scoped + attested".
- **Frozen target on a dirty tree** (review B19): when `capture` returns
  `anchor_kind = CheckoutState`, `snapshot.target` is still `base_commit` (HEAD), never
  the `checkout_state_id`.
- **`commits_touching` with empty paths ⇒ `None`** (defensive; callers gate, review B17).
- **Float ban:** `lexical_score`/specificity/weight all integer.
- **Per-block nonce:** N hits ⇒ N nonces; no shared guard (review #6).

## 6. Open Questions & Unknowns

1. **Lexical scan contract (spec open Q1) — RESOLVED for v1:** in-process
   token-overlap (case-fold, split on non-alphanumeric) over `title+summary+tags` **+
   `memory_key` segments** (confirmed yes — strong signals, review B2). Score = match
   count, integer, bounded. No stemming, no persistent index. **The `memory.md` body is
   *not* scanned in v1** (review B15) — speed/simplicity; `retrieve --query` cannot match
   body-only content. `retrieve` still *renders* the body framed-as-data. Body text is an
   F-index follow-up.
2. **`retrieve` trust floor (review-seeded Q2) — RESOLVED (lock, D8).** Default
   holdback predicate: `trust_level == low && severity >= high` is suppressed from
   `retrieve` (the agent-context boundary; spec § Security "low-trust high-risk held
   back" is normative, not opt-in). `--min-trust L` raises the floor to `L`. **`find`
   does *not* apply the holdback** — it is a human/tool query surface that annotates
   trust instead; `quarantined`/`retracted` stay excluded from *both*.
3. **Staleness as a filter (open Q4) — RESOLVED (lock, D19).** v1 treats staleness as
   display + a feed into the verification-recency sort key, **never a hide**. No
   `--fresh-only` in v1. Condition: staleness stays *visible* on both surfaces — the
   `find` `staleness` column and the `retrieve` `staleness:` header line are
   load-bearing, not optional.
4. **`find` lexical without scope (open Q5) — RESOLVED (lock, D20).** A bare `--query`
   with no scope flags ranks by lexical alone over all active memories. The no-scope
   exclusion applies only to scope-*bearing* queries; lexical is a valid scope-free
   entry.

## 7. Decisions, Rationale & Alternatives

- **D1 — freeze `today` + target commit once per query (review #5).** *Rationale:*
  per-candidate re-resolution of `HEAD`/`today` breaks the determinism contract.
  *Alternative rejected:* resolve lazily per candidate — non-reproducible across a
  midnight/commit boundary.
- **D2 — `retrieve` calls `render_show` per hit with a fresh nonce (review #6).**
  *Rationale:* the A-2 forged-fence defense is per-block; one nonce across N bodies
  lets body *i* forge the close of body *i+1*. *Alternative rejected:* a batch
  `render_retrieve(&[..], one_nonce)` — weakens the SL-005 security contract.
- **D3 — pure `staleness` takes resolved `GitFacts`, git stays in the shell.**
  *Alternative rejected:* a `&dyn GitOracle` in the ranker — leaks impurity into the
  pure core and defeats plain-data testing.
- **D4 — in-process token-scan lexer, no persistent index (open Q1).** Corpus scale;
  `collect_memories` already loads all. *Deferred:* `index/` (F-index).
- **D5 — total `Ord` with `uid` final tiebreak.** Determinism over scan order.
- **D6 — thread expiry requires verified + recent (review #7).** *Rationale:* spec's
  "verification within 14 days" means the verification axis, not mere `reviewed`
  recency. *Alternative rejected:* reviewed-recency alone — surfaces unverified
  stale threads.
- **D7 — staleness is display + recency sort input, not a hard filter (open Q4).**
- **D8 — `retrieve` trust floor locked: suppress `low ∧ severity≥high` (review F5).**
  *Rationale:* spec § Security holdback is normative. *`find` exempt* — human surface
  annotates, does not suppress. *Alternative rejected:* leave the default open — ships
  a security posture as a coin-flip.
- **D9 — frozen target derived from SL-007's `capture().base_commit`, not a new
  `head_commit` (review F1).** *Rationale:* `head_commit`/`head_frame` don't exist in
  the locked seam; `base_commit` is HEAD even on a dirty tree; reuse over new surface
  (DRY). `CaptureError`/non-git ⇒ `target=None` (staleness degrades, query never
  hard-fails). *Alternative rejected:* amend SL-007 to add `head_commit` — re-opens a
  locked design for a value `capture` already returns.
- **D10 — `commits_touching` runs a `merge-base --is-ancestor` precheck (review F2).**
  *Rationale:* `<since>..<target>` is a set-difference; a non-ancestor `since`
  over-counts silently. Detached HEAD stays decidable (not `None`). *Alternative
  rejected:* trust `rev-list` exit codes — they don't signal non-ancestry.
- **D11 — staleness mode keyed on `verified_sha` presence, not `anchor_kind`; thresholds
  locked `FRESH_DAYS=30` (commit mode `0⇒Fresh`) (review F4/F6).** *Rationale:* spec
  selects the git mode by "scope + `verified_sha`"; born `checkout_state` is subsumed
  (`verify` refuses dirty). 30d is the v1 time-based boundary — a single tunable const,
  distinct from the 14d thread window. *Alternative rejected:* the dirty-anchor branch —
  a contradictory third axis that discards a later clean attestation.
- **D12 — `Candidate` carries an explicit `exact_key_match: bool` (review F9).**
  *Rationale:* tuple key 2 is "lexical + exact `memory_key`"; folding it into
  `lexical_score` would let overlap mask an exact-key hit. Surfaced as its own signal,
  dominant within key 2.

External-review pass (2026-06-04, contract-tightening — no architecture change):

- **D13 — hard filters are predicates, not `Ord` keys (review B1).** The pipeline
  *drops* disallowed memories (`base_filter`/`thread_expiry`) before an **8-key** rank;
  the "9-key" prose conflated the two. *Alternative rejected:* encode lifecycle/partition
  as rank fields — would leave disallowed memories present-but-low-ranked.
- **D14 — exact-key is FULL `memory_key` equality only, via SL-005 `normalize_key`
  (review B2/B16).** Segment overlap is `lexical_score`, not exact-key; key segments still
  feed lexical tokens. *Alternative rejected:* segment match as "exact" — makes "exact"
  meaningless; a separate ad-hoc matcher — parallel impl of `normalize_key`.
- **D15 — `has_scope_constraints` is normative: `--query` alone is *not* scope-bearing
  (review B3).** Only `paths/globs/commands/tags` make a query scope-bearing → no-scope
  memories included by bare/`--query`-only `find`, excluded once a scope flag is set.
- **D16 — frozen `QueryPartition { workspace, repo }`; repo is a captured coordinate,
  not inferred from scope arrays (review B4/B20).** v1 `workspace="default"`; in-git
  `repo = Some(capture.repo_id)`, outside git `None` (admits only repo-empty memories);
  `retrieve` never crosses repos. *Alternative rejected:* infer repo-scoped from
  path/glob presence — cross-repo leak or hidden anchored memory.
- **D17 — `find` shows `trust`+`sev` columns; `retrieve` holdback is non-bypassable in
  v1; `--limit` default 5 / max 20 (review B7/B8/B10).** *Rationale:* `find`'s holdback
  exemption is only honest if risk is *visible*; an agent-context boundary must be
  bounded; an audit escape hatch (`--include-held-back`) is deferred, not shipped quietly.
- **D18 — rank ordinal polarity locked (review B12/B13/B14):** verification
  `verified<unverified<stale<disputed`; trust `high<med<low`; severity
  `critical<…<none`; weight desc; review-recency fewer-days-first, missing/malformed
  **last**. `verification_state=stale` is not double-penalised against the `Staleness`
  column (separate axes, spec § "separate axis").
- **D19 — staleness never hides; no `--fresh-only` in v1 (open Q4).** *Rationale:* spec
  mandates holdback for *trust*, not staleness; both axes (git `commits_since`, time
  `reviewed`-age) only badge + feed sort-key 8. *Condition:* the `find` `staleness`
  column and `retrieve` `staleness:` header stay load-bearing — defer is honest only
  while staleness is visible. *Alternative rejected:* a `--fresh-only` filter — surface
  + tests for a hide the spec does not require.
- **D20 — bare `--query` (no scope flags) ranks by lexical alone over all active
  memories (open Q5).** *Rationale:* the no-scope exclusion guards scope-*bearing*
  queries only; lexical is a valid scope-free entry. *Alternative rejected:* require a
  scope flag — blocks the obvious "search titles" use.

## 8. Risks & Mitigations

- **R1 — determinism regression.** *Mitigation:* property test (shuffled input ⇒
  identical order); golden-output test for a fixed fixture store + query.
- **R2 — security regression in `retrieve`** (leaked suppressed memory, forged
  fence). *Mitigation:* suppression pre-render (asserted absent uids); per-block
  fresh nonce (asserted distinct per block); body-as-data framing reused, not forked.
- **R3 — per-candidate `commits_touching` cost (N subprocesses).** *Mitigation:*
  bounded by corpus scale; invoked only for scope-matched + attested candidates;
  documented as the F-index trigger.
- **R4 — git/clock leaking into ordering.** *Mitigation:* frozen `Snapshot`; the
  pure `rank` is a total order over already-resolved data.
- **R5 — depends on SL-007 landing first.** *Mitigation:* sequencing gate — SL-008
  plan starts only once SL-007 is `done`; the `Memory` anchor/`reviewed` fields and
  `src/git.rs` must exist. Stated as an explicit prerequisite.
- **R6 — lexical over-ranking stale/poisoned memory.** *Mitigation:* lexical is a
  *bounded* signal *below* verification/trust/scope in the tuple (spec § Known
  risks); never the top key.

## 9. Quality Engineering & Validation

- **Pure unit tests (the bulk):** `match_scope` per dimension + specificity
  precedence; the 8-key `Ord` (each key decisive when higher keys tie; total-order
  property under shuffle); `exact_key_match` dominates `lexical_score` within key 2;
  `staleness` truth table over the 4 branches (commit `0/≥1/None`, time-based
  `≤30/>30/unparseable`, anchored-unattested, unanchored) — incl. recorded-dirty-then-
  attested ⇒ commit mode; `days_between` (valid diff, inclusive boundary, malformed ⇒
  `None`); `lexical_score` token cases.
- **Git seam:** temp-repo fixture — commit, touch a scoped path, second commit ⇒
  `commits_touching` counts; **non-ancestor `since` ⇒ `None`, not an over-count**
  (the `merge-base` precheck); shallow / non-git ⇒ `None`; **detached HEAD against a
  frozen target ⇒ a real count, not `None`**; target is the frozen sha, not live HEAD.
- **Verb integration:** `record`ed (SL-007) memory with scope + anchor → `find
  --path` returns it ranked with staleness + `trust`/`sev` columns; `retrieve` frames it
  as data with anchor + staleness; quarantined/retracted absent from `retrieve`; stale
  unverified `thread` dropped, fresh verified one surfaces; per-block nonces distinct.
- **Filter-not-rank contract (review B1/B4/B7):** quarantined/retracted **and**
  cross-repo memories are *absent* from output, not merely last; thread expiry runs
  *after* scope match (verified-recent passes, unverified-recent fails); low-trust ∧
  high-severity is *absent* from `retrieve` (non-bypassable) yet *present with a visible
  risk marker* in `find`.
- **Ranking precision (review B2/B12-14):** exact-key beats higher lexical overlap;
  every ordinal polarity asserted (verification/trust/severity/weight/recency, null last);
  two otherwise-identical rows break on `uid`/`key` under shuffled input.
- **Query semantics (review B3/B5/B10):** `--query` alone admits no-scope memories,
  `--path` excludes them; glob-only verified memory resolves time-based/`Unknown` (not
  commit-count); `retrieve` default limit 5 and max 20 enforced.
- **Partition (review B4/B20):** current-repo memory included, different-repo excluded,
  non-git behaviour explicit (only repo-empty memories).
- **Behaviour-preservation:** all SL-005/007 + entity/slice/state suites green
  unchanged.
- **Gate:** `cargo clippy` zero warnings; `cargo fmt`; `just lint && just test` per
  commit.

## 10. Review Notes

> Carries the reader-side findings from the original combined-SL-007 review (codex,
> 2026-06-04): #5 (snapshot determinism, D1), #6 (per-block nonce, D2), #7 (thread
> expiry verified+recent, D6). Re-review before `slice plan`, seeding: the 9-key
> `Ord` totality, D2's per-block nonce as the security crux, open Q2 (`retrieve`
> trust-floor default), and the SL-007 prerequisite gate (R5).
>
> **Design pass 2026-06-04 (claude + codex, grounded against real src/ + SL-007
> lock).** 9 findings, all resolved into the design above:
> F1 git symbol drift (`head_commit` absent ⇒ reuse `capture().base_commit`, D9);
> F2 `rev-list A..B` over-counts non-ancestors ⇒ `merge-base --is-ancestor` precheck,
> detached is *not* `None` (D10); F3 no day-arithmetic helper ⇒ pure `days_between`;
> F4 staleness thresholds undefined ⇒ `FRESH_DAYS=30`, commit `0⇒Fresh` (D11);
> F5 trust floor locked (D8); F6 dirty-anchor axis dropped — keyed on `verified_sha`
> (D11); F8 bare-`find` reuse claim narrowed (does not inherit `list` order); F9
> explicit `exact_key_match` signal (D12). Re-review before `slice plan` if the
> 30-day boundary or the `find` holdback-exemption want a second opinion.
>
> **External review pass 2026-06-04 (contract-tightening, no architecture change).**
> Verdict: plan-ready after tightening. 21 findings, all accepted, scoped to v1 minimal
> where options were offered → D13–D18 + tuple table (§5.2) + reordered pipeline (§5.4):
> B1 filters-drop-not-rank (8-key, D13); B2/B16 exact-key = full key via `normalize_key`
> (D14); B3 `has_scope_constraints` normative (D15); B4/B20 `QueryPartition`, repo not
> inferred (D16); B5 commit-staleness `paths`-only (glob expansion deferred); B6
> branch-3 = time mode's missing-date arm; B7/B8/B10 non-bypassable holdback + `find`
> risk columns + limit 5/20 (D17); B9 thread-expiry after scope-match; B11 verb-default
> divergence; B12-14 ordinal polarity locked (D18); B15 body not lexically scanned (Q1);
> B17 empty-path ⇒ None; B18 per-candidate git Unknown; B19 dirty target = `base_commit`.
> No open architectural questions remain; SL-007 landing (R5) is still the only gate.
