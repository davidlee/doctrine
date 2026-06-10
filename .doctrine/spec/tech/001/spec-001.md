# SPEC-001: Graph-Derived Priority Engine

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The mechanism realising **PRD-011** (graph-derived backlog priority): a derived,
explainable "what should I look at next, and why?" view over doctrine's existing
entity graph, recomputed from authored state and never written back as truth.

The system is three separable layers (the central principle — *core owns how
values flow; policy owns what they mean; the adapter owns where they come from*):

```text
  doctrine CLI  (survey / next / explain / blockers / inspect)
        │
  ┌─────┴───────────────┐  doctrine container
  │  policy             │  channel semantics: eligibility, actionability,
  │  (src/priority/…)   │  relation interpretation, rank↔consequence, render
  │  adapter            │  registry scan → opaque nodes + typed edges,
  │                     │  id mapping, diagnostics mapped back to doctrine ids
  └─────┬───────────────┘
        │ opaque node ids + typed directed edges  (no doctrine vocabulary)
  ┌─────┴───────────────┐  cordage — NEW workspace member crate
  │  graph core         │  reverse index · reachability · per-overlay cycle
  │  (crates/cordage)   │  policy (reject / evict) · deterministic order · provenance
  └─────────────────────┘
```

The crate is **`cordage`** (`crates/cordage/`) — rigging over a tree of typed
overlays; doctrine stays the workspace root crate and depends on it by path (D1).

The **graph core** is a generic multi-channel evaluation engine over a tree plus
typed directed (DAG) overlays — *not* a backlog-priority library. Prioritisation
is one application of it; blocking diagnostics, reachability, and rollups are
others. Backlog-priority meaning lives entirely in the doctrine **policy**; the
mapping from authored entity TOML to the core's node/edge model lives entirely in
the **adapter**. C4 frame: the new crate is a fresh *container*; policy + adapter
are *components* inside the doctrine container.

## Responsibilities

Mirrors the structured `responsibilities` list. In short: own the generic graph
core as a standalone publishable crate; own the doctrine policy that interprets
kind / lifecycle / resolution / relation into derived channels; own the registry
adapter that feeds the core while keeping all interpretation out of it; fix the
crate and module boundary (D1); define the derived-surface CLI contracts; and
hold the forbidden-core line (Appendix B — no time, scheduling, commitment,
urgency, or product vocabulary in the crate).

## Concerns

- **Boundary purity (load-bearing).** The crate must contain no doctrine or
  product nouns. The acceptance proof is structural: the core test suite carries
  no doctrine entity vocabulary and no time/scheduling/commitment semantics,
  while policy tests carry those interpretations explicitly (PRD-011 §7
  verification; REQ-079).
- **Determinism.** Same registry graph + policy version + query options → same
  ordering and explanations (REQ-077). No clock, no RNG, no `HashMap` iteration
  order in any ordering path; the tie-break is an explicit total `order_key`
  tuple, not incidental ordering. (Pure/imperative split — clock/disk/git stay
  in the thin shell.)
- **Performance posture.** v1 recomputes from a full registry scan per query;
  the corpus is small (tens–hundreds of entities). Incremental recomputation and
  dirty-region evaluation are an adapter-level optimisation the boundary
  preserves but v1 does not build. Any cache is disposable; correctness is
  recomputation from authored state (REQ-078).
- **Failure mode — cycles.** Handled per overlay (D5): a `dep` cycle is invalid —
  diagnose (naming node ids + edge kinds), degrade the affected query, never emit
  a false topological order (REQ-076); a `seq` cycle cannot persist — the view
  iteratively evicts the minimal `(rank, age)` edge to a fixpoint at build time and
  surfaces each evicted edge in provenance (REQ-092). A cycle spanning *both*
  overlays is invisible to either per-overlay policy; the union-cycle rule (D9) —
  `dep` is authoritative, `seq` yields — closes that seam. No path mutates authored
  TOML.
- **The path-free-core tension (OQ-009, dependency-bearing).** Architectural triggers
  condition actionability on a *code-surface* (path-glob) match, but the core must
  not depend on file paths (PRD-011 §4 constraint). D6 fixes the *shape* — a
  **policy-layer actionability mask**, never a graph edge — but stays open on the
  plan/audit file-set source (still unbuilt); the authored `triggers` field is now
  blessed (PRD-009 FR-011 / `REQ-098`, OQ-007 resolved), so OQ-009 is
  resolved-pending-prerequisites, not closed.

## Hypotheses

- **H1 — full recompute is enough for v1.** The governed corpus is small enough
  that scan-and-recompute per query is acceptable; no incremental engine is
  needed yet. *Challenge:* if the corpus grows past comfort, the adapter's
  invalidation seam (not the core) absorbs it.
- **H2 — tree + typed-DAG captures doctrine's relations.** Doctrine relations are
  authored outbound-only (ADR-004); reverse edges, blockers, and reachability are
  all derivable in-core from that alone, with no new durable inbound field.
- **H3 — `retrieve.rs` glob predicates are the trigger matcher.** SL-008's leaf
  predicates `glob_admits(pattern, query)` / `path_admits` (`src/retrieve.rs`) are
  generic `&str` matchers reusable verbatim — trigger globs are the *pattern*, a
  phase's file set the *subject*. (The higher `match_scope` is `Memory`-typed and is
  *not* reused; a thin policy caller wraps the leaf predicates.) No new glob engine
  (D6). *Caveat:* the matcher needs two inputs that do not yet exist — see D6's
  prerequisites.
- **H4 — the boundary is extractable later.** Carving the core as a neutral
  workspace crate *now* (path dependency) makes "ultimately external" a
  `cargo publish` + flip-path-to-version, not a refactor (D1).

## Decisions

- **D1 — three-layer split; the core is a neutral, external-ready workspace
  crate.** Closes PRD-011 OQ-003 and OQ-005-layout (OQ-5). The generic graph core
  becomes its own workspace member crate **`cordage`** at `crates/cordage/`,
  product-neutral (neither `doctrine-` nor `bough-` prefixed — both, and other
  products, are intended consumers); doctrine stays the workspace root crate and
  depends on `cordage` by path. Published-grade boundary from day one (no doctrine vocabulary, no product
  semantics); the actual crates.io publish is deferred. Ownership split per
  PRD-011 §8: **core** owns opaque node ids, typed edges, overlay selection,
  reverse index, reachability, topological order, cycle diagnostics,
  deterministic traversal, generic propagation, structured provenance; **adapter**
  owns registry-scan reading, doctrine-id↔node-id mapping, edge emission from
  authored relations, node attributes, diagnostic re-mapping; **policy** owns
  active-state rules, terminal-for-priority resolutions, relation-kind
  classification, rank↔consequence combination, inclusion/exclusion of
  terminal/promoted nodes, and explanation rendering. The crate boundary respects
  module layering (ADR-001): the core is a leaf the doctrine engine depends on,
  never the reverse. *Alt rejected:* doctrine-internal module (`src/graph/`) —
  cheap, but makes "external" a future refactor rather than a preserved seam;
  external-published-now — real release overhead before any consumer beyond
  doctrine exists.
- **D2 — the §9 boundary test governs every core/policy placement.** A rule
  belongs in the core iff it can be stated without product nouns ("propagate a
  channel backward over a DAG with `max`"); a rule that needs semantic
  interpretation ("a promoted backlog item is not active work") belongs in
  policy. Appendix B (forbidden core: task/project/habit terms, deadline /
  scheduled-for / best-before, lateness cost, remaining-work, commitment
  pressure, urgency scoring, calendar/capacity, sequential/parallel policy,
  resurfacing, product defaults) is the standing prohibition list.
- **D3 — derived channels, v1 set, open-ended.** `eligible`, `actionable`,
  `blocked_by`, `blocking`, `consequence`, `order_key`, `explanation` (PRD-011
  §7). No channel is authored backlog truth. Storage must not assume the v1 set
  is exhaustive — a richer engine may emit more (REQ-080 seam): node ids stay
  opaque and stable per run, relation kinds stay typed (never prose-encoded), and
  the policy version is explicit in derived output.
- **D4 — two edge species; policy classifies, the core mechanises.** Closes
  OQ-1. Every typed edge maps to one of two overlay species, and *which* doctrine
  relation kind is which is policy:
  - **`dep`** — hard dependency / blocked. Strictly acyclic (see D5 reject).
  - **`seq`** — soft sequencing / priority preference, carrying an int **`rank`**.
    Acyclic by construction (see D5 evict). Higher rank = stronger preference.

  Per OQ-1 (option: *introduce blocks now*), doctrine **authors** both kinds — a
  hard `dep` edge and a soft `seq` edge with `rank` — rather than deriving
  actionability from reference edges alone (the corpus has no dependency edge
  today; only `specs`/`slices`/`requirements`/`drift` references, `supersedes`/
  `descends_from` lineage, and the `origin` promote bridge). Reference and lineage
  edges remain **consequence** inputs (inbound count raises derived importance);
  `origin` drives promoted-exclusion. The core stays generic — it propagates typed
  channels over typed edges with chosen combinators and knows nothing of `dep` vs
  `seq` meaning; it sees only an overlay's configured cycle policy and opaque edge
  attributes (`rank`, `age`).

  `dep`/`seq` edges are **optional enrichment**, never required capture — an item
  with no authored edge still surveys by derived consequence + fallback, so PRD-011
  §4 ("capture must never require dependency modelling") holds. The *authored* edge
  schema itself (new relation kinds + `rank`) is **product-capture surface owned by
  PRD-009** (its relation seam, §2), not minted in this tech spec; SPEC-001
  *consumes* it. Pushed to PRD-009 OQ-007 — FR-005/REQ-096 is buildable only once
  that schema is blessed.
- **D5 — per-overlay cycle policy: `dep` rejects, `seq` evicts.** Closes PRD-011
  OQ-005. Acyclicity is enforced per overlay, by one of two core-configurable
  policies:
  - **reject-on-cycle** (`dep`): a cycle is **invalid**. The graph build raises a
    diagnostic naming the node ids + edge kinds; the affected query excludes the
    cyclic component or degrades to local authored priority + stable fallback. A
    `dep` cycle is an authoring error to fix, surfaced by `validate` — never a
    silent false topological order.
  - **evict-on-cycle** (`seq`): while the overlay contains a cycle, **evict the
    globally-minimal participating edge under `(rank asc, age asc)`** — lowest rank,
    ties broken oldest-first — and **repeat to a fixpoint** (each eviction strictly
    reduces edge count, so it terminates; disjoint cycles each lose their own minimal
    edge). The overlay is therefore always acyclic; a new low-rank edge that only
    closes a cycle against stronger edges evicts itself (no-op). Every evicted edge
    is **surfaced in provenance** for the affected nodes — an authored preference is
    never dropped silently (REQ-077; PRD-011 "explanations are structured, not
    magic").

  **Eviction is a build-time *derived* resolution, never an authored mutation**
  (storage rule): authored TOML may hold a `seq` cycle; every recompute evicts
  deterministically by `(rank, age)` to yield an acyclic view, leaving the
  authored edges untouched. `age` is a **clock-free, stable authoring ordinal** the
  adapter supplies (not a wall-clock `created` date — day granularity would tie). The
  *contract* the core depends on is only that `age` be **total and stable across
  recomputes**; the derivation is an adapter-slice concern (a candidate: source
  entity id, then append position within the outbound array — clock-free, stable
  while arrays stay append-only). Because that ordinal is an authoring/structural
  artifact rather than true wall-clock age, the equal-rank tie-break is deterministic
  but not a semantic "oldest wins." A future `link` verb could additionally enforce
  at write time, but the read-side resolution is the contract.
- **D6 — architectural triggers as a policy-layer actionability mask
  (dependency-bearing; OQ-009 resolved-pending).** A path-glob trigger holds an item
  non-actionable until a file set matches the globs of **any** of its triggers. It
  enters as a **policy mask** — `mask(item, files) = ∃ t ∈ item.triggers ·
  glob_admits(t.globs, files)` over the leaf `retrieve.rs` predicates (SL-008; trigger
  globs = pattern, file paths = subject) — *never* a graph edge, keeping the core
  path-free (PRD-011 §4). This fixes the *shape*; it does not yet close OQ-009, because
  of the three inputs the matcher needs, the authored field is now **blessed** but two
  file-set sources remain unbuilt:
  1. **the authored `triggers` field** — a list of `{ globs = [...], note = "…" }`
     riders; an item is masked until a file set matches **any** entry, and the matching
     entry's `note` surfaces. **Minted by PRD-009 FR-011 (`REQ-098`); OQ-007 resolved**
     — schema blessed, awaiting backlog implementation. Promotes IMP-013/014's coarse
     `trigger` *tag* to typed structure.
  2. **(a) the planned file set** — a declared-paths field on the plan/phase, read at
     the planning gate (`/plan` / `/phase-plan`). The mask fires here **prospectively**,
     surfacing the prefactor opportunity *before* the code exists — an architectural
     rider that only fired post-build would be useless.
  3. **(b) the touched file set** — the audited reality at `/audit`, derived from the
     worktree diff (no new authored field). The mask re-checks here against what
     *actually* happened.

  **Both (a) and (b) are necessary, for different reasons:** (a) is the plan, (b) is
  the audited truth. When they **diverge** enough to flip a trigger the plan missed,
  that is a **caught oops** — a drift signal surfaced at audit, not a silent miss.
  IMP-013 (two-path) / IMP-014 (single-path) become real acceptance fixtures *once
  backlog implements the `triggers` field* (schema now blessed; the two file-set
  sources still unbuilt); until then the "if the mask can't surface them, D6 is
  not done" criterion is not yet evaluable. Gate enforcement strength (soft
  skill-check vs hard workflow/preflight step) is **OQ-6**; the plan↔audit divergence
  surface is folded into it.
- **D7 — deterministic ordering and stamped cache.** `order_key` is a total
  deterministic tuple; ordering never depends on clock, RNG, or map-iteration
  order. Cache/projection output stamps the policy version + an input signature
  so stale derived results are recognisable (closes PRD-011 OQ-008). Deleting any
  cache and recomputing yields equivalent output (REQ-078).
- **D8 — derived outputs are disposable.** Ephemeral, or stored only as
  cache/projection; never authored into entity TOML as canonical truth. Derived
  actionability never mutates `status`, `resolution`, `item_kind`, priority
  metadata, or relations (REQ-078 invariant). Reverse references are always
  computed from authored outbound edges (ADR-004), never a separate inbound field
  (REQ-074).
- **D9 — overlay composition and the union-cycle rule: `dep` is authoritative,
  `seq` yields.** D5 enforces acyclicity *per overlay*, which is provably blind to a
  cycle spanning both — e.g. `A —dep→ B` (acyclic in `dep`) with `B —seq→ A`
  (acyclic in `seq`), whose **union** is cyclic. The composition is fixed: `order_key`
  is built **dep-topology first, then `seq` rank within a dep-eligible set, then the
  deterministic fallback** — `seq` is a preference *within* the dependency order, it
  never reorders across a `dep` edge. A `seq` edge whose inclusion would close a cycle
  **against the resolved `dep` order** is therefore evicted by the same
  `(rank asc, age asc)` rule (and surfaced in provenance, per D5), exactly as for an
  intra-`seq` cycle. Result: the union is always acyclic, `dep` blocking is never
  overridden by a soft preference, and no false topological order is ever emitted
  (REQ-076/REQ-092). *Verification:* seeding `A —dep→ B`, `B —seq→ A` yields a stable
  order with the `seq` edge reported evicted — no panic, no false topo.

## Open Questions

Local to this spec. PRD-011 OQ-003 / OQ-005 / OQ-008 are closed above (D1 / D5 /
D7); PRD-011 OQ-002 (OQ-1) and crate name/layout (OQ-5) are closed by D4 and D1.
PRD-011 OQ-009 is **resolved-pending-prerequisites** (D6 fixes the shape; the
authored `triggers` field is now blessed, the plan/audit file-set sources remain
unbuilt). PRD-011 OQ-001 (item-level authored-priority seam) is PRD-009's, not this
spec's. The `dep`/`seq` and trigger **authored schemas** D4/D6 rely on are PRD-009's
capture surface — now minted there (PRD-009 OQ-007 resolved: FR-010 `needs`/`after`,
FR-011 `triggers`); this spec consumes them, it does not mint them.

Resolved:

- ~~**OQ-1** (PRD-011 OQ-002) — admitted relation set.~~ Closed by D4: two edge
  species `dep`/`seq`; doctrine authors both now; reference/lineage edges are
  consequence inputs.
- ~~**OQ-5** — crate name + workspace layout.~~ Closed by D1: `cordage` at
  `crates/cordage/`, doctrine the root crate depending by path.
- ~~**OQ-7** (→ PRD-009) — the authored capture schema D4/D6 depend on.~~ Resolved by
  PRD-009 OQ-007: the `dep`/`seq` edges land as the agent-facing `needs`/`after` edges
  (FR-010 / `REQ-096` consumes `REQ-097`); the architectural trigger lands as the
  optional `triggers` list `{ globs, note }` (FR-011 / `REQ-093` consumes `REQ-098`).
  Authored names are decoupled from the `dep`/`seq` overlay vocabulary (policy/adapter
  classifies); `after`'s `rank` is a pairwise-edge attribute, distinct from PRD-011
  OQ-001's still-open item-level scalar. Schema blessed; FR-005/D6 buildable once
  backlog implements it.

Remaining:

- **OQ-2** (PRD-011 OQ-006) — whether v1 derived `consequence` accounts for
  PRD-010 knowledge-record state, or defers governance pressure until PRD-010
  ships.
- **OQ-3** (PRD-011 OQ-007) — whether an authored `seq` rank overrides
  actionability in `survey` while `next` still prefers actionable work (how the
  two surfaces diverge on a ranked-but-blocked item).
- **OQ-4** (PRD-011 OQ-004) — transitive blocking shown by default, or only on an
  explicit `--transitive` / explain surface.
- **OQ-6** — planning-gate enforcement strength for D6's trigger channel: soft
  (skill remembers to check) vs hard (workflow/preflight step), and how the
  plan↔audit divergence (D6 (a) vs (b)) is surfaced. Friction vs enforceability
  (IMP-012 tension).
