# Memory retrieval: find/retrieve + scope ranking + staleness

## Context

SL-005 landed memory v1's **write + read-by-id** half: the `Memory` entity (UUID
identity, no-reservation engine variant), the pure schema/parse core, and the
`record` / `show` / `list` verbs. What it did not land is the reason memory
exists — **scope-aware query**. A store you can only write to and read by uid is
write-only in practice; an agent cannot ask "what do you know about *this* file /
command / task" without it. The memory umbrella ([memory-spec](../../../doc/memory-spec.md))
names the missing surface explicitly: `find` / `retrieve`, scope-first lexical
retrieval, the deterministic ranking tuple, and git-anchored staleness
(§ Retrieval & ranking). This slice is **roadmap step 2's remainder** — the query
half of native v1.

The spec already locks the algorithm (it is lifted, proven, from spec-driver), so
this slice is mostly *honest implementation of a settled design* plus one real
schema widening. Three things shape it:

- **Scope is the retrieval key.** Matching is OR across `paths`(3) / `globs`(2) /
  `commands`(1) / `tags`(0) with specificity weights; a memory with no scope is
  excluded from scope-filtered queries. The validated `Memory` already carries the
  full `Scope` (`src/memory.rs`), so matching is a pure function over data that is
  already in hand.
- **Staleness needs a git frame the current `Memory` throws away.** Raw parse keeps
  `[git]` as the empty `RawGit{}`; `Memory` never carries the anchor. The
  scoped+attested staleness mode needs `verified_sha` + the scope paths to count
  intervening commits. So this slice **widens `Memory` to carry the git anchor**
  and adds the IO-seam git-reachability query — the pure/imperative split holds
  (doctrine builds the frame, the pure ranker takes it as input; interop
  constraint 4).
- **Retrieval is a security boundary.** Stored memory is hostile input. `find`
  returns ranked rows for a human/tool; `retrieve` assembles the agent-context
  block — quoted, attributed, delimited, never instruction — and is where the
  suppression rules (`quarantined` / `retracted` never reach context) and the
  hard filters live (§ Security).

## Scope & Objectives

- **Scope matching (pure).** A `match_scope(&Memory, &QueryContext) -> Option<ScopeMatch>`
  over the validated `Scope`: OR across the four dimensions, specificity weight
  `paths=3 / globs=2 / commands=1 / tags=0`, `**`-aware glob, path exact-or-prefix,
  command token-prefix, tag set-intersection. No-scope memories yield `None` under
  a scope-filtered query. Pure — `QueryContext` (the caller's paths / commands /
  tags) is an input.

- **Deterministic sort (pure).** The spec's 9-key sort tuple (§ Retrieval):
  hard filters → lexical relevance + exact `memory_key` → scope specificity →
  verification state → trust → severity → weight → review recency → uid/key
  tiebreak. Same query ⇒ same order (agent reproducibility). Implemented as a total
  `Ord` over a derived `Ranked` row so the ordering is testable in isolation. The
  lexical score is a **bounded signal into the tuple**, never the final word.

- **Git-anchored staleness.** Widen `Memory` to carry the `[git]` anchor
  (`anchor_kind`, `verified_sha`, `base_commit`, `ref_name`) the raw layer already
  reads but currently discards. A `staleness(&Memory, &GitFrame, today) -> Staleness`
  pure function realising the three modes — scoped+attested (commits touching
  scoped paths since `verified_sha`), scoped-unattested (days since `reviewed`),
  unscoped (days since `reviewed`) — resolving to an **explicit**
  `fresh | stale | unknown | unanchored`, never a silent hide or over-trust.
  Undecidable reachability (shallow/partial clone, detached HEAD, non-ancestor
  anchor, non-git) → `unknown` / `unanchored`. The commit-count query sits behind
  the existing IO seam (the `doctrine install` / `slice` git surface); the pure
  layer takes a resolved `GitFrame`.

- **`doctrine memory find` (ranked search).** The human/tool query verb: take a
  `QueryContext` from flags (`--path`, `--glob`, `--command`, `--tag`, free-text
  `--query`), apply hard filters (workspace, repo, lifecycle status default-active,
  quarantine/trust, git visibility), scope-match, rank, format `id status type
  scope staleness title`-style rows. Rides the `collect_memories` →
  pure-filter/sort → `format` split that `list` already uses (`src/memory.rs`).
  `--include-draft` and status overrides mirror `list`.

- **`doctrine memory retrieve` (agent-context block).** The security-rendered
  surface: same query + rank, but emits each hit as a **quoted, delimited,
  attributed data block** carrying `memory_uid` / `memory_key`, `trust_level`,
  `verification_state`, scope, anchor, and staleness — explicitly *data, never
  instruction* (§ Security, locked decision 8). `quarantined` and `retracted` are
  suppressed unconditionally; low-trust-high-severity held back. This is the verb
  an agent boot / pre-task hook will eventually call; v1 ships it on-demand only.

- **Thread expiry.** A `thread` surfaces only with a scope match **and**
  verification within 14 days (`today` is an input); otherwise excluded. Folded
  into the hard-filter stage, not a special case at the call site.

End state: an agent can ask memory "what's relevant to this path / command / task"
and get a deterministic, scope-ranked, staleness-annotated answer — and a
security-safe agent-context block. Native v1's read surface is then complete
(`record` / `show` / `list` / `find` / `retrieve`); only the reserved seam
(ledger, interchange, event-store backend, dense/graph retrieval) remains deferred.

## Non-Goals

- **Lexical backend sophistication (open question #1).** Whether the lexical score
  is a grep-class token scan or an embedded BM25 index is a **design-doc decision**,
  not a scope expansion. v1 picks the simplest backend that satisfies the bounded-
  signal contract at current corpus scale (tens of memories); the ranking tuple is
  designed so a stronger lexical backend swaps in without reordering. The derived
  `index/` subtree is already gitignored (SL-005 manifest split) and stays
  rebuild-on-read until the corpus justifies persistence.

- **Dense / graph retrieval.** Embedding sidecars and graph expansion are deferred
  sidecars (spec § Retrieval, open questions #2/#6). When added they contribute
  *bounded signals into the same tuple*; they never break the deterministic final
  ordering. Out of scope — no `embeddings/` write path here.

- **Proactive / pre-hook surfacing (open question #4).** `visibility = pre`
  (memory injected into context before a task without an explicit call) is
  deferred. v1 ships `retrieve` as an **on-demand** verb only; the boot/pre-task
  hook that calls it is a follow-up once a boot-context generator exists (the same
  gap ADR's governance-boot listing waits on).

- **Links / backlinks (roadmap step 3).** Folding `[[...]]` wikilinks and
  `[[relation]]` rows into the relation-index registry is the *next* step, not this
  one. `find` / `retrieve` rank on scope + the existing facets; relation edges do
  not yet contribute a ranking signal. Kept separate so this slice stays the
  retrieval primitive and step 3 is purely additive.

- **Lifecycle / review *mutation* verbs.** `supersede` / `retract` / `review` /
  `reanchor` / `promote` (which advance `status` and `verification_state`, and
  write `verified_sha`) are the mutation half; they belong with the reserved
  ledger seam (every mutation is also an event, interop constraint 1). v1 retrieval
  **reads** verification/anchor state; it does not advance it. Without a `review`
  verb, `verified_sha` is whatever `record` wrote — staleness still computes
  honestly (an unattested memory simply takes the days-since-`reviewed` mode).

- **Reserved seam.** `events.toml` ledger, NDJSON import/export, the event-store
  backend adapter — all deferred (spec § reserved seam). Retrieval is a pure-read
  projection over the current-state files; it does not touch the ledger.

- **Engine change.** Retrieval adds query functions to `src/memory.rs` and CLI
  arms to `main.rs`; it does not touch `src/entity.rs`. The existing entity /
  slice / state / memory suites are the behaviour-preservation proof and stay green
  unchanged. The one `Memory` widening (git anchor) is additive — every existing
  `Memory` field and its readers are untouched.

## Summary

The query half of native memory v1: scope-first, lexical-first, deterministic
retrieval over the SL-005 store. Two pure cores — `match_scope` (OR across
paths/globs/commands/tags with specificity weights) and the 9-key deterministic
sort — plus a `staleness` function that needs the one real schema change here:
widening validated `Memory` to carry the `[git]` anchor the raw layer already
parses but discards, so the scoped+attested mode can count commits touching scoped
paths since `verified_sha`. Two verbs ride the existing `collect_memories` →
pure-filter/sort → format split: `find` (ranked human/tool rows) and `retrieve`
(the security-contract agent-context block — quoted, attributed, data-never-
instruction, quarantined/retracted suppressed). Thread 14-day expiry folds into
the hard-filter stage. Git reachability sits behind the established IO seam; the
pure layer takes a resolved `GitFrame`.

The lexical-backend choice (open question #1), the exact `QueryContext` flag set,
the `Ranked` row and its `Ord`, the `GitFrame` shape + the reachability query, and
the `retrieve` block format live in the design doc ([design.md](design.md)) —
authored with this slice, pending adversarial review per the slice-002/003/004
rhythm.

## Follow-Ups

- **F1 — lifecycle / review mutation verbs.** `review` (write `verified_sha` +
  `reviewed` + horizon), `supersede`, `retract`, `reanchor`, `promote` — the
  edit-preserving `toml_edit` mutation half. Each is also a ledger event (interop
  constraint 1), so this is the natural caller that turns on the reserved
  `events.toml` seam.
- **F2 — links/backlinks via relation-index (roadmap step 3).** Resolve `[[...]]`
  + `[[relation]]` into the shared registry; let relation edges contribute a
  bounded ranking signal and feed `show` derived backlinks.
- **F3 — persistent lexical index.** Materialise the gitignored `index/` (rebuild-
  on-write or on-demand) when corpus scale outgrows the v1 scan; swap the lexical
  backend behind the unchanged ranking tuple.
- **F4 — proactive surfacing (`visibility = pre`).** When a boot-context generator
  exists, call `retrieve` from a pre-task hook to inject scope-matched memory
  ahead of work (open question #4).
- **CLAUDE.md.** Add `doctrine memory find|retrieve` to the CLI surface and update
  the memory-verbs known-gap note when this lands.
