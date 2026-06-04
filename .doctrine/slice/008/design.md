# Design SL-008: Memory retrieval: find/retrieve + scope ranking + staleness

## 1. Design Problem

Build the **reader** over the SL-007-populated store ([slice-008.md](slice-008.md)):
`find` (ranked rows) and `retrieve` (the security agent-context block), backed by
scope matching, the spec's deterministic 9-key sort, and git-anchored staleness.
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
- **`src/git.rs`** exists with `head_frame` / `head_commit` + repo-identity. This
  slice **extends** it with `commits_touching` — the reader's only git addition.
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
- **Locked 9-key sort (hard):** the tuple order is fixed by the spec; lexical and
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
    snapshot = { today: clock::today(), target: git::head_commit(root) }
    mems = collect_memories(items_root)
    for m in mems (scope-matched & attested):
        facts[m] = GitFacts { commits_since: git::commits_touching(root, m.scope.paths,
                                                                    m.anchor.verified_sha,
                                                                    snapshot.target) }
  ─▶ pure core (over mems, QueryContext, facts, snapshot.today):
        hard_filter  →  match_scope  →  staleness  →  rank (9-key total Ord)
  ─▶ find: format_find(&[Ranked])           (rows)
     retrieve: for hit in [Ranked]: render_show(hit, body, fresh_nonce())  (blocks)
```

### 5.2 Interfaces & Contracts

```rust
struct QueryContext { paths: Vec<String>, globs: Vec<String>,
                      commands: Vec<String>, tags: Vec<String>, query: Option<String> }
struct Snapshot { today: String, target: Option<String> }   // frozen once per query
struct GitFacts { commits_since: Option<u32> }               // None = undecidable
enum Staleness { Fresh, Stale, Unknown, Unanchored }

fn match_scope(m: &Memory, q: &QueryContext) -> Option<ScopeMatch>;   // ScopeMatch{ specificity:u8 }
fn lexical_score(m: &Memory, q: &Option<String>) -> u32;              // bounded token-overlap, integer
fn staleness(m: &Memory, facts: &GitFacts, today: &str) -> Staleness;
fn rank(cands: Vec<Candidate>) -> Vec<Ranked>;                        // stable, total 9-key Ord

// git seam extension (impure):
fn commits_touching(root: &Path, paths: &[String], since: &str, target: &str) -> Option<u32>;
```

**Git seam.** `commits_touching` shells `git rev-list --count <since>..<target> --
<paths>`; non-zero exit / parse failure ⇒ `None` (non-ancestor, shallow, detached,
non-git). `target` is `snapshot.target` (frozen), **never** a literal `HEAD` —
closing the determinism leak. Called only for candidates that are scope-matched and
carry a `verified_sha` (attested); otherwise skipped (no git cost).

**`find` / `retrieve` CLI.**
```
doctrine memory find     [--path P]… [--glob G]… [--command C]… [--tag T]… [--query Q]
                         [--type T] [--status S] [--include-draft] [-p ROOT]
doctrine memory retrieve <same query/filter flags> [--limit N] [--min-trust L] [-p ROOT]
```
`find` rows: `uid-short  type  status  staleness  spec  title` (`spec` = matched
dimension). `retrieve`: per hit, `render_show` + a `staleness:` header line,
suppressing quarantined/retracted, applying the trust floor.

### 5.3 Data, State & Ownership

- **Derived, never stored:** `ScopeMatch.specificity`, `lexical_score`, `Staleness`,
  rank order — all per-query. No `index/` write (open Q1).
- **Read-only:** the slice never touches `memory.toml`; it owns no persistent state.
- **`Candidate` / `Ranked`** are in-memory pure structs (Memory ref + match +
  facts + staleness + scores), discarded after the query.

### 5.4 Lifecycle, Operations & Dynamics

- **Query:** freeze `Snapshot` → `collect_memories` → **hard-filter** (workspace/repo
  match; lifecycle: active-only default, `--include-draft` adds draft, quarantined/
  retracted always excluded; **thread expiry**) → `match_scope` (drop `None` under a
  scope-bearing query) → resolve `GitFacts` (attested candidates only) → `staleness`
  → `rank` → format/render.
- **Thread expiry (review #7):** a `thread` passes only if scope-matched **and**
  `verification_state == verified` **and** `reviewed` within 14 days of
  `snapshot.today`.
- **`retrieve` suppression** is pre-render: a suppressed memory never reaches
  `render_show`, so its body cannot leak inside a framed block.

### 5.5 Invariants, Assumptions & Edge Cases

- **Total order:** the 9-key `Ord` ends on `uid`; shuffled `collect_memories` order
  ⇒ identical output (property test).
- **Frozen snapshot:** all staleness/ordering computed against one `today` + one
  `target` — a query spanning midnight or a concurrent commit is still internally
  consistent (review #5).
- **No-scope memory:** excluded from a scope-bearing query; included by a bare
  `find` (degenerates to `list` + staleness).
- **Undecidable git** (`commits_since=None` with `verified_sha` present) ⇒ `Unknown`,
  never `Fresh`. No `verified_sha` ⇒ time-based (days since `reviewed`); no anchor ⇒
  `Unanchored`; dirty (`checkout_state`) anchor ⇒ time-based (no clean sha to count).
- **Float ban:** `lexical_score`/specificity/weight all integer.
- **Per-block nonce:** N hits ⇒ N nonces; no shared guard (review #6).

## 6. Open Questions & Unknowns

1. **Lexical scan contract (spec open Q1) — RESOLVED for v1:** in-process
   token-overlap (case-fold, split on non-alphanumeric) over `title+summary+tags`;
   score = match count, integer, bounded. No stemming, no persistent index. *Confirm
   token set includes `memory_key` segments* (lean: yes, they are strong signals).
2. **`retrieve` trust floor (review-seeded Q2).** Default `--min-trust` excludes
   `trust_level=low` when `severity≥high`? Spec § Security says "low-trust high-risk
   held back." *Confirm the exact default predicate.*
3. **Staleness as a filter (open Q4).** v1 treats staleness as display + a feed into
   the verification-recency sort key, never a hide. *Confirm no `--fresh-only`*
   (lean: defer).
4. **`find` lexical without scope.** A bare `--query` with no scope flags — rank by
   lexical alone over all active memories? *Lean:* yes, lexical is a valid
   scope-free entry; no-scope exclusion applies only to scope-*bearing* queries.

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
  precedence; the 9-key `Ord` (each key decisive when higher keys tie; total-order
  property under shuffle); `staleness` truth table (3 modes × {fresh, stale,
  undecidable, unanchored, dirty}); `lexical_score` token cases.
- **Git seam:** temp-repo fixture — commit, touch a scoped path, second commit ⇒
  `commits_touching` counts; non-ancestor sha / shallow / non-git ⇒ `None`; target
  is the frozen sha, not live HEAD.
- **Verb integration:** `record`ed (SL-007) memory with scope + anchor → `find
  --path` returns it ranked with a staleness column; `retrieve` frames it as data
  with anchor + staleness; quarantined/retracted absent from `retrieve`; stale
  unverified `thread` dropped, fresh verified one surfaces; per-block nonces distinct.
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
