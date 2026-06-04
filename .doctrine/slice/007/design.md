# Design SL-007: Memory retrieval: find/retrieve + scope ranking + staleness

## 1. Design Problem

Land the **query half** of native memory v1 ([slice-007.md](slice-007.md)): make
an agent able to ask the SL-005 store "what do you know relevant to *this* path /
command / task" and get a deterministic, scope-ranked, staleness-annotated answer
— plus a security-safe agent-context block. The retrieval *algorithm* is already
locked and proven (lifted from spec-driver; [memory-spec](../../../doc/memory-spec.md)
§ Retrieval), so the design problem is not "invent ranking." It is three things
the spec assumed present and SL-005 did not build:

1. **The producer does not exist.** `record` (SL-005) captures *only tags* and
   workspace; `scope.paths`/`globs`/`commands`, `repo`, and the entire `[git]`
   anchor render empty / `none`. Scope-first matching (`paths`=specificity 3) and
   the attested staleness mode are therefore **consumers with no producer** — `find
   --path` would match nothing, forever. v1 retrieval is meaningless unless `record`
   first learns to capture scope and anchor. This slice owns that producer.
2. **doctrine has no git surface.** The only `git` token in `src/` is `.git` as a
   root marker (`src/root.rs`). The spec's "anchor before consume" (interop
   constraint 4) and the scoped+attested staleness mode both need git facts — HEAD
   resolution to build the born frame, and commit-counting to age a memory. This
   slice builds doctrine's **first git IO seam** (`src/git.rs`).
3. **Retrieval is a security boundary.** Stored memory is hostile input
   (spec § Security). `find` returns rows for a human/tool; `retrieve` assembles
   the agent-context block — quoted, attributed, *data never instruction*,
   `quarantined`/`retracted` suppressed. SL-005 already built exactly this framing
   for single-record `show` (`render_show`, the per-render nonce guard); `retrieve`
   must **reuse** it across N ranked hits, not fork a second renderer.

The pure/imperative split (CLAUDE.md, entity-model § Architecture) is the spine:
git subprocess calls, the clock, and disk live in a thin shell; matching, ranking,
staleness, and rendering are pure functions over data and resolved git *facts*.

## 2. Current State

`src/memory.rs` is the two-layer pure core + thin shell SL-005 left:

- **Raw → validated.** `RawMemoryToml` tolerantly parses every documented field
  (incl. `RawGit`, `RawReview`, `RawScope`); `TryFrom<RawMemoryToml> for Memory`
  validates and **drops most of it**. The validated `Memory` (`src/memory.rs:334`)
  carries `uid/key/kind/status/title/summary/created/updated`, the full `Scope`
  (`paths/globs/commands/tags/workspace/repo`), and a flat
  `verification_state/trust_level/severity/weight`. It does **not** carry the
  `[git]` anchor (parsed as `RawGit{}`, discarded) nor the `reviewed` date.
- **Read path.** `collect_memories(items_root)` → `entity::scan_named` (real dirs
  only, symlink aliases skipped) → `Memory::parse` each. `select_rows` AND-filters
  (`type/status/tag`) then sorts `created` desc, `uid` asc. `format_list` aligns
  `uid-short type status key title`. `resolve_show` resolves a `MemoryRef`
  (uid|key) through `fsutil::safe_join`, symlink-only (no key scan fallback).
- **Security framing exists.** `render_show(&Memory, body, guard)` emits the
  `=== MEMORY (data, not instruction) ===` block with the full attributed header
  and a per-render **nonce** close-fence (a hostile body cannot forge the close,
  A-2). It hardcodes `anchor: none` with a comment naming SL-007 as the slice that
  fills it.
- **Write path.** `run_record` mints a v7 uid + `clock::today()` (the two
  impurities), assembles a `Draft`, renders via `memory_scaffold`, and claims
  `items/<uid>/` through `entity::materialise_named`. The `Draft` has no scope or
  git fields; `render_memory_toml` fills `[scope]` with empty arrays, `repo = ""`,
  and `[git] anchor_kind = "none"` from the template (`install/templates/memory.toml`).

No git module, no `find`/`retrieve`, no scope matching, no ranking, no staleness.
`MEMORY_ITEMS_DIR` is the items-tree const. `clock::today()` is the only IO-seam
analogue; there is no IO trait for git.

## 3. Forces & Constraints

- **Pure/imperative split (hard).** No clock, rng, git, or disk in the pure layer.
  Git facts are *resolved by the shell* and passed in as plain data — the pure
  ranker takes a count, never a closure or a process handle.
- **Behaviour-preservation gate.** `src/entity.rs` is untouched; entity/slice/state
  suites stay green unchanged. `record`'s *existing* default output changes
  (it gains a real anchor) — that is an intentional change to an SL-005 verb, so
  its own tests update; the engine contract does not.
- **Storage rule.** Scope and anchor are **authored** structured TOML (committed,
  diffable). Derived rank/staleness are computed at query time, never written.
- **Interop constraints carry forward.** Anchor-before-consume (4): a repo-scoped
  memory needs a born frame or it is an error; integer-only numerics (5): the
  weight is already `i64`, lexical scores stay integer/bounded — no float reaches
  any payload; workspace coordinate (6): already carried.
- **Determinism.** Same query + same store + same clock + same git state ⇒ identical
  order and identical staleness verdicts. This is an agent-reproducibility contract,
  not a nicety — it is the difference between a tool an agent can trust and noise.
- **Hostile input.** Every retrieved byte is data. Suppression
  (`quarantined`/`retracted`) is non-negotiable and lives at the hard-filter stage,
  before rendering.
- **Corpus scale.** Tens of memories. `collect_memories` already loads all into
  memory; per-query full scan is acceptable. A persistent index is premature.
- **No git lib dependency.** doctrine shells `git` (subprocess) rather than
  vendoring a git library — smallest surface, matches "doctrine builds the frame."

## 4. Guiding Principles

1. **Make it work end-to-end, or don't ship it.** Producer + reader together; a
   half that only works against hand-edited TOML is not a feature.
2. **Resolve impurity at the edge, rank in the pure core.** The shell gathers
   `today` + git facts per candidate; everything downstream is a pure total order.
3. **Reuse the proven seam.** `find` rides `collect_memories` → pure-filter/sort →
   format (the `list` shape). `retrieve` rides `render_show`. No parallel renderer,
   no second read path.
4. **Explicit over silent.** Every undecidable git/staleness case resolves to a
   named state (`unknown`/`unanchored`), never a silent hide or a silent over-trust.
5. **Narrowest honest scope.** Build the *born* anchor (record-time), not the
   re-stamp verbs. Build the *scan* lexer, not a persistent index. Each deferral is
   a seam the follow-up rides, not a rewrite.

## 5. Proposed Design

### 5.1 System Model

Four units, three pure and one impure:

```
            ┌─────────────────────────── shell (impure) ───────────────────────────┐
 record ──▶ │ git::head_frame()  ─▶ born GitFrame ─▶ render [git] + [scope] into toml│
 find    ─▶ │ collect_memories ─▶ for each candidate: git::commits_touching(...)      │
 retrieve ─▶│                      + clock::today()  ─▶ GitFacts                       │
            └───────────────┬───────────────────────────────────────────────────────┘
                            │ (data only: Memory, QueryContext, GitFacts, today)
            ┌───────────────▼───────────────── pure core ──────────────────────────┐
            │ match_scope(&Memory,&QueryContext) -> Option<ScopeMatch>               │
            │ staleness(&Memory,&GitFacts,today) -> Staleness                        │
            │ rank(Vec<Candidate>) -> Vec<Ranked>     (the 9-key total Ord)          │
            │ format_find(&[Ranked]) / render_retrieve(&[Ranked], nonce) -> String   │
            └───────────────────────────────────────────────────────────────────────┘
```

`src/git.rs` is the new impure module. `src/memory.rs` gains the pure cores and the
two verb shells. `main.rs` gains the `find`/`retrieve` CLI arms and the new
`record` scope/anchor flags.

### 5.2 Interfaces & Contracts

**Git seam (`src/git.rs`, impure).**

```rust
pub(crate) struct GitFrame {            // the born frame, built at record time
  pub anchor_kind: AnchorKind,          // Commit | CheckoutState | None
  pub commit: String,                   // HEAD sha   (Commit)
  pub base_commit: String,              // HEAD sha it sits on
  pub ref_name: String,                 // refs/heads/...
  pub checkout_state_id: String,        // dirty-tree id (CheckoutState)
}
pub(crate) fn head_frame(repo_root: &Path) -> Result<GitFrame>;
// reachability: commits touching `paths` in `since..HEAD`. None = undecidable.
pub(crate) fn commits_touching(repo_root: &Path, paths: &[String], since: &str)
  -> Result<Option<u32>>;
```

`head_frame` shells `git rev-parse HEAD`, `git symbolic-ref`, `git status
--porcelain` (dirty). `commits_touching` shells `git rev-list --count
<since>..HEAD -- <paths>`; a non-zero git exit (non-ancestor sha, shallow clone,
detached/unborn) maps to `Ok(None)` — *not* an error. Non-git root, missing `git`
binary → `head_frame` yields `anchor_kind = None`; `commits_touching` → `Ok(None)`.

**Query context (pure input, built from flags).**

```rust
pub(crate) struct QueryContext {
  pub paths: Vec<String>, pub globs: Vec<String>,
  pub commands: Vec<String>, pub tags: Vec<String>,
  pub query: Option<String>,            // free-text lexical
}
```

**Pure cores.**

```rust
fn match_scope(m: &Memory, q: &QueryContext) -> Option<ScopeMatch>;
// ScopeMatch { specificity: u8 }  — max over matched dimensions: path 3 > glob 2 > command 1 > tag 0.
// None when the query is scope-bearing and the memory matches no dimension.
fn staleness(m: &Memory, facts: &GitFacts, today: &str) -> Staleness;
// Staleness ∈ Fresh | Stale | Unknown | Unanchored
fn rank(cands: Vec<Candidate>) -> Vec<Ranked>;   // stable sort by the 9-key Ord
```

`GitFacts` is the *resolved* per-memory git answer the shell computed:
`{ commits_since: Option<u32> }` (None ⇒ undecidable). The pure `staleness` never
calls git; it reads this datum.

**Verb shells.**

```
doctrine memory find  [--path P]… [--glob G]… [--command C]… [--tag T]… [--query Q]
                      [--type T] [--status S] [--include-draft] [-p ROOT]
doctrine memory retrieve  <same query/filter flags> [--limit N] [-p ROOT]
doctrine memory record … [--path P]… [--glob G]… [--command C]… [--repo R]   (NEW flags)
```

`find` prints aligned rows: `uid-short  type  status  staleness  spec  title`
(`spec` = matched-dimension marker). `retrieve` prints, per hit, the
`render_show` block extended with `anchor:` (real, not `none`) and a `staleness:`
header line, suppressing `quarantined`/`retracted` unconditionally.

### 5.3 Data, State & Ownership

- **`Memory` widening (additive).** Add a validated `Anchor` (from `[git]`) and the
  `reviewed` date (from `[review]`). Existing fields and readers untouched; the new
  fields default empty for legacy memories (anchor_kind `none`, reviewed `""`),
  which resolve to honest `Unanchored`/`Unknown` staleness.
- **`record` ownership.** `record` now owns born-frame construction (calls
  `head_frame`) and writes the real `[git]` block + the `[scope]` arrays from flags.
  `--repo` defaults to the resolved remote/root id when a frame is born. This is the
  single deliberate behaviour change to an SL-005 verb.
- **Derived, never stored.** `ScopeMatch.specificity`, lexical score, `Staleness`,
  rank order — all computed per query, never persisted. No `index/` write (open Q1).
- **Storage rule honoured.** Scope + anchor are authored structured TOML; the body
  (`memory.md`) is untouched prose.

### 5.4 Lifecycle, Operations & Dynamics

- **record (capture):** resolve root → `head_frame` → assemble `Draft` (now with
  scope + anchor) → render → `materialise_named`. A repo-scoped memory in an unborn/
  non-git context is a hard error (interop constraint 4); an unscoped memory there
  is permitted (`anchor_kind = none`).
- **find/retrieve (query):** resolve root → `collect_memories` → for each: build
  `GitFacts` (shell `commits_touching` when scope+`verified_sha` present, else skip)
  + read `today` → pure `match_scope` (drop `None` under a scope-bearing query) →
  pure `staleness` → pure `rank` → `format`/`render`. The git calls are the only
  per-candidate impurity; bounded by corpus scale.
- **Hard-filter stage (pure, pre-rank):** workspace/repo match, lifecycle status
  (default active-only; `--include-draft` adds draft; `quarantined`/`retracted`
  always excluded from `retrieve`), and **thread expiry** — a `thread` survives only
  with a scope match *and* `reviewed` within 14 days of `today`, else dropped.

### 5.5 Invariants, Assumptions & Edge Cases

- **Determinism:** the 9-key `Ord` is **total** (final tiebreak on `uid`), so the
  sort is stable and reproducible regardless of `collect_memories` dir order.
- **No-scope memory** is excluded from a scope-bearing query (it cannot be *found*
  by context) but still listed by bare `find` with no scope flags (degenerates to
  `list` + staleness).
- **Undecidable git** (`commits_since = None` with an anchor present) ⇒ `Unknown`,
  never `Fresh`. Missing anchor ⇒ `Unanchored`. Dirty-tree anchor
  (`checkout_state`) ⇒ time-based mode (no sha to count from).
- **Float ban:** no score is a float at any boundary; lexical score is an integer
  rank contribution (bounded token-overlap count), consistent with interop
  constraint 5 even though nothing is exported here.
- **`retrieve` suppression is unconditional** and happens before rendering — a
  suppressed memory never reaches the renderer, so the nonce-guarded block cannot
  leak it.
- **Symlink aliases** never double-count (`scan_named` returns real dirs only,
  inherited from SL-005).

## 6. Open Questions & Unknowns

1. **Lexical backend (spec open Q1) — RESOLVED for v1:** grep-class **token-overlap
   scan** over `title + summary + tags`, computed in-process during the query (no
   persistent `index/`). At tens of memories this is sub-millisecond; the ranking
   tuple is shaped so a BM25/embedded index swaps in as a bounded signal later (F3).
   *Confirm:* token normalization (case-fold + split on non-alphanumeric) is enough,
   or do we need stemming? (Lean: no stemming in v1.)
2. **`retrieve` ordering vs `find`:** identical rank, or does `retrieve` apply a
   stricter trust floor (suppress low-trust-high-severity)? Spec § Security says
   "held back" — propose a `--min-trust` defaulting to exclude `low`+`severity≥high`.
   *Confirm the default.*
3. **`--repo` identity:** remote URL vs a normalized `host/owner/name`. Spec uses
   `github.com/owner/repo`. *Confirm derivation when multiple remotes exist*
   (lean: `origin`, else first, else `--repo` required).
4. **Staleness as a filter?** v1 treats staleness as a displayed annotation + a sort
   input (via verification recency), never a hide. *Confirm no `--fresh-only` in v1*
   (lean: defer; explicit annotation is enough).

## 7. Decisions, Rationale & Alternatives

- **D1 — SL-007 builds the producer (scope + anchor capture), not just the reader.**
  *Rationale:* scope-first retrieval is inert without scope; attested staleness is
  dead without an anchor (§1). *Alternative rejected:* read-only retrieval over
  hand-edited TOML scope — violates "ask, don't infer" and the spec (record builds
  the frame), and ships a feature that only works if a human edits TOML by hand.
- **D2 — first git surface is a subprocess seam (`src/git.rs`), not a git library.**
  *Rationale:* smallest dependency surface; matches "doctrine builds the frame, the
  backend never shells" (constraint 4); the three `git` invocations are stable
  plumbing commands. *Alternative rejected:* `git2`/`gix` crate — a large dep for
  three reads, and a porcelain we don't need.
- **D3 — `record` anchors at capture; re-stamp verbs deferred.** `verified_sha` =
  HEAD at record time gives attested staleness a working baseline immediately
  (commits accrue → ages honestly). *The over-claim worry, resolved:* `verified_sha`
  and `verification_state` are **orthogonal axes** the spec deliberately separates
  (memory-spec § lifecycle-vs-review). `verified_sha` is the *anchor* axis — "the
  SHA the claims were last checked against reality"; recording **is** writing the
  memory against the current tree, so HEAD is the honest value. `verification_state`
  is the *review* axis — independent confirmation — and correctly stays `unverified`
  until a `verify` verb runs. Attested staleness keys on the anchor SHA, never on
  the review state, so there is no contradiction. *Alternative rejected:* leave
  `verified_sha` empty at record (spec's "unattested → time-based" path) — but
  `reviewed` is *also* empty at record, so a fresh memory would resolve to `Unknown`
  and attested mode stays dead in v1 (the option the user explicitly rejected).
- **D4 — pure `staleness` takes resolved `GitFacts`, not a `GitFrame` + callback.**
  *Rationale:* keeps git fully out of the pure layer (no closures, no process
  handles crossing the seam) — testable with plain data. *Alternative rejected:*
  pass a `&dyn GitOracle` into the ranker — leaks impurity into the pure core.
- **D5 — in-process token-scan lexer, no persistent index (open Q1).** *Rationale:*
  corpus scale; `collect_memories` already loads everything. *Alternative deferred:*
  persistent `index/` (F3) when scale demands.
- **D6 — `retrieve` reuses `render_show`, generalized to N hits + real anchor +
  staleness line.** *Rationale:* one security renderer, one nonce contract; no
  parallel implementation (CLAUDE.md). *Alternative rejected:* a second
  `render_retrieve` from scratch — drift risk on the security-critical framing.
- **D7 — staleness is display + recency sort input, not a hard filter (open Q4).**
  Keeps the surface small; explicit annotation satisfies "no silent over-trust."

## 8. Risks & Mitigations

- **R1 — `record` behaviour change breaks SL-005 record tests.** The born anchor
  changes the rendered toml. *Mitigation:* intentional; update those tests in the
  same phase, keep the *engine* suites untouched (behaviour-preservation gate is
  about `entity.rs`, not the deliberately-changed verb). New scope/anchor flags are
  optional — a bare `record` in a clean repo still succeeds (now with a real
  anchor).
- **R2 — git subprocess portability / absence.** Missing binary, non-git dir,
  shallow clone, detached HEAD, unborn branch. *Mitigation:* every git failure maps
  to `None`/`anchor_kind=none`, never a panic or an error that aborts a query;
  explicit `Unknown`/`Unanchored` states; unit tests drive each edge via a temp-repo
  fixture (and a non-git temp dir).
- **R3 — per-candidate `commits_touching` cost (N subprocesses).** *Mitigation:*
  bounded by corpus scale (tens); only invoked for memories that are both
  scope-matched and attested; documented as the F3 trigger when scale grows.
- **R4 — security regression in `retrieve` (leak of suppressed memory, forged
  fence).** *Mitigation:* suppression at the pre-render hard-filter stage (suppressed
  memory never reaches the renderer); reuse the proven per-render nonce; conformance
  tests assert suppressed uids are absent and the close-fence carries the nonce.
- **R5 — non-determinism from git/clock leaking into ordering.** *Mitigation:* git
  facts + `today` are resolved once at the seam and frozen into the candidate set;
  the pure `rank` is a total order with a `uid` final tiebreak — property test: a
  shuffled input yields an identical output order.

## 9. Quality Engineering & Validation

- **Pure unit tests (the bulk):** `match_scope` per dimension + specificity
  precedence; the 9-key `Ord` (each key decisive when higher keys tie; total-order
  property under input shuffle); `staleness` truth table across the three modes ×
  {fresh, stale, undecidable, unanchored, dirty}. All plain-data in, verdict out.
- **Git seam tests:** temp-repo fixture (init, commit, touch scoped path, second
  commit) asserting `commits_touching` counts; edge fixtures (non-ancestor sha,
  non-git dir, dirty tree) asserting `None`/`anchor_kind=none`.
- **Verb integration:** `record --path … --command …` then `find --path …` returns
  it ranked; `retrieve` frames it as data with a real anchor + staleness; a
  `quarantined`/`retracted` memory is absent from `retrieve`; a stale `thread` is
  dropped, a fresh one surfaces.
- **Behaviour-preservation:** full entity/slice/state suites green unchanged;
  SL-005 memory show/list tests green except the intentionally-updated record-anchor
  assertions.
- **Gate:** `cargo clippy` zero warnings; `cargo fmt`; `just lint && just test`
  green before each commit (note: justfile has no `check` recipe — CLAUDE.md drift,
  tracked separately).

## 10. Review Notes

> Pending. Author the design, then run the adversarial review (a second agent or
> codex mcp, the slice-002/003/004 rhythm) before `slice plan`. Seed the reviewer
> with: D1 (is folding the producer into the "retrieval" slice scope creep, or the
> honest minimum?), D3 (is record-time `verified_sha` the right baseline, or does it
> over-claim verification the memory never received?), D6 (does generalizing
> `render_show` to N hits weaken the single-record security contract?), and the
> open questions (esp. Q2 trust floor on `retrieve`).
