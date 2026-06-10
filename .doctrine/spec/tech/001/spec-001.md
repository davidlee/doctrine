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
  evicts the minimal `(rank, age)` edge at build time (REQ-092). Neither path
  mutates authored TOML.
- **The path-free-core tension (OQ-009).** Architectural triggers condition
  actionability on a *code-surface* (path-glob) match, but the core must not
  depend on file paths (PRD-011 §4 constraint). Resolved by D6: the trigger is a
  **policy-layer actionability mask**, never a graph edge.

## Hypotheses

- **H1 — full recompute is enough for v1.** The governed corpus is small enough
  that scan-and-recompute per query is acceptable; no incremental engine is
  needed yet. *Challenge:* if the corpus grows past comfort, the adapter's
  invalidation seam (not the core) absorbs it.
- **H2 — tree + typed-DAG captures doctrine's relations.** Doctrine relations are
  authored outbound-only (ADR-004); reverse edges, blockers, and reachability are
  all derivable in-core from that alone, with no new durable inbound field.
- **H3 — `retrieve.rs` scope predicates are the trigger matcher.** The SL-008
  scope-admittance engine (path / glob matching) already decides whether a memory
  is in scope for a touched-path set; the same predicate matches an architectural
  trigger against a phase's planned file set. No new matcher (D6).
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
- **D5 — per-overlay cycle policy: `dep` rejects, `seq` evicts.** Closes PRD-011
  OQ-005. Acyclicity is enforced per overlay, by one of two core-configurable
  policies:
  - **reject-on-cycle** (`dep`): a cycle is **invalid**. The graph build raises a
    diagnostic naming the node ids + edge kinds; the affected query excludes the
    cyclic component or degrades to local authored priority + stable fallback. A
    `dep` cycle is an authoring error to fix, surfaced by `validate` — never a
    silent false topological order.
  - **evict-on-cycle** (`seq`): a would-be cycle is broken by **evicting the
    participating edge minimal under `(rank asc, age asc)`** — lowest rank, ties
    broken oldest-first. The overlay is therefore always acyclic; a new low-rank
    edge that only closes a cycle against stronger edges evicts itself (no-op).

  **Eviction is a build-time *derived* resolution, never an authored mutation**
  (storage rule): authored TOML may hold a `seq` cycle; every recompute evicts
  deterministically by `(rank, age)` to yield an acyclic view, leaving the
  authored edges untouched. `age` is a **clock-free creation-sequence stamp** the
  adapter supplies (not a wall-clock `created` date — day granularity would tie),
  keeping the core deterministic and pure. A future `link` verb could additionally
  enforce at write time, but the read-side resolution is the contract.
- **D6 — architectural triggers as a policy-layer actionability mask.** Closes
  PRD-011 OQ-009 (in-scope as a channel). A path-glob trigger holds an item
  non-actionable until a phase's planned/touched file set matches its trigger
  globs. It enters as a **policy mask**, resolved by reusing the `retrieve.rs`
  scope predicates (SL-008) — *not* as a graph edge, keeping the core path-free
  (PRD-011 §4 constraint). The authored trigger field is PRD-009's capture seam
  (`{ globs = [...], note = "…" }`); the planning gate (`/plan` / `/phase-plan`)
  is the v1 consumer that runs the scope match over the phase's declared paths.
  IMP-013 (two-path) and IMP-014 (single-path) are the acceptance fixtures; if
  the mask cannot express their triggers and surface them at the gate, D6 is not
  done. Gate enforcement strength (soft skill-check vs hard workflow/preflight
  step) is **OQ-6**.
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

## Open Questions

Local to this spec. PRD-011 OQ-003 / OQ-005 / OQ-008 / OQ-009 are closed above
(D1 / D5 / D7 / D6); PRD-011 OQ-002 (OQ-1) and crate name/layout (OQ-5) are now
closed by D4 and D1. PRD-011 OQ-001 (authored-seam field shape) is PRD-009's, not
this spec's.

Resolved:

- ~~**OQ-1** (PRD-011 OQ-002) — admitted relation set.~~ Closed by D4: two edge
  species `dep`/`seq`; doctrine authors both now; reference/lineage edges are
  consequence inputs.
- ~~**OQ-5** — crate name + workspace layout.~~ Closed by D1: `cordage` at
  `crates/cordage/`, doctrine the root crate depending by path.

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
  (skill remembers to check) vs hard (workflow/preflight step). Friction vs
  enforceability (IMP-012 tension).
