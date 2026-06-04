# Memory retrieval: find/retrieve + scope ranking + staleness

## Context

The **reader** half of memory v1's retrieval story. SL-007 (the producer) makes a
recorded memory carry its scope (`paths`/`globs`/`commands`/`repo`) and a git born
anchor, and adds `verify` to stamp the verification axis (`verified_sha`/`reviewed`/
`verification_state`). This slice consumes that populated data to answer the
question memory exists for: *"what do you know relevant to this path / command /
task?"* — deterministically ranked and staleness-annotated, plus a security-safe
agent-context block ([memory-spec](../../../doc/memory-spec.md) § Retrieval & ranking).

The retrieval *algorithm* is locked and proven (lifted from spec-driver), so this
slice is mostly honest implementation of a settled design. Three things shape it:

- **Scope is the retrieval key.** Matching is OR across `paths`(3) / `globs`(2) /
  `commands`(1) / `tags`(0) with specificity weights; a memory with no scope is
  excluded from a scope-bearing query. SL-007 ensures the scope is actually
  populated; this slice matches against it (pure, over data already in `Memory`).
- **Determinism is a contract, and git/clock are where it leaks.** Same query +
  store + clock + git state ⇒ identical order and identical staleness verdicts. The
  reader resolves the clock and git facts **once per query** (a frozen snapshot) and
  ranks in a pure total order — never re-resolving `HEAD` or `today` per candidate
  (the determinism hole the SL-007 review flagged).
- **Retrieval is a security boundary.** `find` returns rows for a human/tool;
  `retrieve` assembles the agent-context block — quoted, attributed, *data never
  instruction*, `quarantined`/`retracted` suppressed. It reuses SL-005's
  `render_show` framing **per hit with a fresh nonce each**, never one nonce
  fanned across N memories (the A-2 forged-fence defense must hold per block).

## Scope & Objectives

- **Scope matching (pure).** `match_scope(&Memory, &QueryContext) -> Option<ScopeMatch>`:
  OR across the four dimensions, specificity `paths=3 / globs=2 / commands=1 /
  tags=0`, `**`-aware glob, path exact-or-prefix, command token-prefix, tag
  set-intersection. A scope-bearing query that matches no dimension yields `None`
  (excluded). `QueryContext` (caller paths/globs/commands/tags + free-text) is an
  input.

- **Deterministic 9-key sort (pure).** The spec's tuple (§ Retrieval): hard filters
  → lexical relevance + exact `memory_key` → scope specificity → verification state
  → trust → severity → weight → review recency → `uid`/`key` tiebreak. A **total**
  `Ord` over a derived `Ranked` row so the order is reproducible regardless of
  directory-scan order; lexical score is a **bounded integer signal** into the
  tuple, never the final word (no float, interop constraint 5).

- **`commits_touching` (git seam extension).** Add to SL-007's `src/git.rs`:
  `commits_touching(repo_root, paths, since_sha, target) -> Option<u32>` — the
  staleness reachability query: a `git merge-base --is-ancestor` precheck (since
  `<since>..<target>` is a set-difference, not an ancestry test), then
  `git rev-list --count <since>..<target> -- <paths>`, resolved against the **frozen
  target commit** (not a live `HEAD`). `None` when undecidable (non-ancestor sha,
  shallow clone, non-git); a **detached** HEAD against a frozen target is *not* `None`
  — still anchored, still countable. The reader's only git need.

- **Git-anchored staleness (pure).** `staleness(&Memory, &GitFacts, today) -> Staleness`
  over SL-007's populated anchor + `reviewed`, realising the three spec modes —
  scoped+attested (has `verified_sha`: commits touching scoped paths since it,
  resolved into `GitFacts`), scoped-unattested (no `verified_sha`: days since
  `reviewed`), unscoped (days since `reviewed`) — to an **explicit**
  `fresh | stale | unknown | unanchored`, never a silent hide/over-trust. Mode is
  keyed on `verified_sha` presence, **not** `anchor_kind`: a memory recorded dirty
  then `verify`-attested clean uses its `verified_sha`. Undecidable reachability →
  `unknown`; git-anchored but never attested → `unknown`; no anchor → `unanchored`.

- **`doctrine memory find` (ranked rows).** Build `QueryContext` from flags
  (`--path`/`--glob`/`--command`/`--tag` repeatable, `--query` free-text), apply
  hard filters (workspace/repo, lifecycle status default-active, `--include-draft`,
  quarantine/trust), scope-match, rank, format
  `uid-short type status staleness spec title` rows. Reuses `collect_memories` + the
  hard-filter predicate `list` uses, but **not** its order or formatter — `find`
  always applies the 9-key rank (not `created`-desc) and a `Ranked` formatter with
  the staleness/spec columns.

- **`doctrine memory retrieve` (agent-context block).** Same query+rank; emits each
  hit via SL-005's `render_show` (extended with the real `anchor:` + a `staleness:`
  line), **a fresh per-hit nonce**, suppressing `quarantined`/`retracted`
  unconditionally before rendering, plus an optional trust floor (low-trust +
  high-severity held back). On-demand only.

- **Thread expiry (hard-filter stage).** A `thread` surfaces only with a scope match
  **and** `verification_state = verified` **and** `reviewed` within 14 days of the
  frozen `today`; else excluded. (Tightened per review — verified + recent, not
  reviewed-recency alone.)

End state: native memory v1's read surface is complete (`record`/`show`/`list`
[SL-005], `verify` [SL-007], `find`/`retrieve` [here]). Only the reserved seam
(ledger, interchange, event-store backend, dense/graph retrieval) remains deferred.

## Non-Goals

- **Producer capture / anchoring.** Scope + born-frame capture and `verify` are
  SL-007. This slice reads what they wrote; it does not write scope or anchors.

- **Lexical backend sophistication (spec open Q1).** v1 is an in-process
  token-overlap scan over `title+summary+tags` — no persistent `index/`. The tuple
  is shaped so a BM25/embedded index swaps in as a bounded signal later (F-index).
  The derived `index/` subtree stays gitignored + unbuilt.

- **Dense / graph retrieval.** Embedding sidecars, graph expansion — deferred
  sidecars contributing bounded signals into the same tuple when they land.

- **Proactive / pre-hook surfacing (spec open Q4).** `visibility = pre` (injecting
  memory before a task) is deferred; `retrieve` is on-demand only until a
  boot-context generator exists.

- **Links / backlinks (roadmap step 3).** Folding `[[...]]` + `[[relation]]` into
  the relation-index registry is the next step; relation edges do not yet feed
  ranking.

- **Heavier lifecycle / re-stamp verbs.** `reanchor`/`supersede`/`retract`/`promote`
  and the ledger seam — SL-007 F1.

- **Engine change.** Adds pure cores + `find`/`retrieve` to `src/memory.rs`, extends
  `src/git.rs` with `commits_touching`, adds CLI arms; does not touch `src/entity.rs`
  or SL-007's producer code. All existing suites stay green unchanged.

## Summary

The reader half: scope-first, lexical-first, deterministic retrieval over the
SL-007-populated store. Three pure cores — `match_scope` (OR with specificity
weights), the 9-key total `Ord`, and `staleness` (three modes over a frozen
`GitFacts` snapshot) — feed two verbs riding `collect_memories` → filter/sort →
format: `find` (ranked rows) and `retrieve` (the security agent-context block,
per-hit `render_show` with a fresh nonce each, quarantined/retracted suppressed).
`src/git.rs` gains `commits_touching` (the only reader git need), resolved against
a target commit frozen once per query alongside `today` — closing the determinism
hole. Thread expiry (verified + reviewed-within-14-days) folds into the
hard-filter stage.

The lexical-scan contract (open Q1), the `Ranked`/`Ord` shape, the `GitFacts`
snapshot discipline, the `retrieve` trust floor (open Q2), and staleness-as-filter
(open Q4) live in the design doc ([design.md](design.md)) — authored with this
slice, pending adversarial review per the slice-002/003/004 rhythm.

## Follow-Ups

- **F-index — persistent lexical index.** Materialise the gitignored `index/`
  (rebuild on write/demand) when corpus scale outgrows the in-process scan; swap the
  lexical backend behind the unchanged ranking tuple.
- **F-links — relation-index fold (roadmap step 3).** Resolve `[[...]]` +
  `[[relation]]` into the shared registry; let edges contribute a bounded ranking
  signal and feed `show` backlinks.
- **F-pre — proactive surfacing (`visibility = pre`).** Call `retrieve` from a
  pre-task hook when a boot-context generator exists (spec open Q4).
- **CLAUDE.md.** Add `doctrine memory find|retrieve` to the CLI surface when this
  lands.
