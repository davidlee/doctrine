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
  ┌─────┴───────────────┐  NEW workspace member crate (neutral, external-ready)
  │  graph core         │  reverse index · reachability · typed acyclicity +
  │                     │  cycle diagnostics · deterministic order · provenance
  └─────────────────────┘
```

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
- **Failure mode — cycles.** A relation kind declared acyclic that contains a
  cycle must degrade, never lie: emit a diagnostic naming the node ids and edge
  kinds, exclude the cyclic component or fall back to local authored priority +
  stable order, and never emit a false topological result (REQ-076).
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
  crate.** Closes PRD-011 OQ-003. The generic graph core becomes its own
  workspace member crate, neutral-named (neither `doctrine-` nor `bough-`
  prefixed — both, and other products, are intended consumers), depended on by
  path. Published-grade boundary from day one (no doctrine vocabulary, no product
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
- **D4 — relation-kind semantics live in policy, never the core.** The core
  propagates typed channels over typed edges with chosen combinators; *which*
  kind blocks, influences, or merely contextualises is policy. The v1 admitted
  set is provisional (§7: `blocks`/`blocked_by` acyclic dependency;
  `promotes_to`/`origin` lifecycle bridge, not a dependency; `relates_to`
  contextual; `shapes`/`constrains` governance influence; `supersedes` acyclic
  lineage; `members` non-priority; `drift_affects` remediation consequence) —
  the exact admitted set is **OQ-1**.
- **D5 — typed per-relation-kind acyclicity; degrade on cycle.** Closes PRD-011
  OQ-005. Acyclicity is a per-relation-kind contract, not a global graph
  invariant: a kind declared acyclic (e.g. `blocks`, `supersedes`) reporting a
  cycle yields a diagnostic naming node ids + edge kinds, and the affected query
  excludes the cyclic component or degrades to local authored priority + stable
  fallback; kinds not declared acyclic tolerate cycles. Never a silent false
  topological order.
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

Local to this spec (the durable architecture is settled; these are remaining
policy/layout calls). PRD-011 OQ-003 / OQ-005 / OQ-008 / OQ-009 are closed above
(D1 / D5 / D7 / D6). PRD-011 OQ-001 (authored-seam field shape) is PRD-009's, not
this spec's.

- **OQ-1** (PRD-011 OQ-002) — the exact relation-kind set admitted into v1
  actionability/blocking, vs merely contextual. The §7 / D4 list is provisional;
  this fixes the blocker set and the per-kind acyclicity contract.
- **OQ-2** (PRD-011 OQ-006) — whether v1 derived `consequence` accounts for
  PRD-010 knowledge-record state, or defers governance pressure until PRD-010
  ships.
- **OQ-3** (PRD-011 OQ-007) — whether an authored rank overrides actionability in
  `survey` while `next` still prefers actionable work (how the two surfaces
  diverge on a ranked-but-blocked item).
- **OQ-4** (PRD-011 OQ-004) — transitive blocking shown by default, or only on an
  explicit `--transitive` / explain surface.
- **OQ-5** — the crate's neutral name and exact workspace layout (directory,
  manifest, where it sits relative to `src/`).
- **OQ-6** — planning-gate enforcement strength for D6's trigger channel: soft
  (skill remembers to check) vs hard (workflow/preflight step). Friction vs
  enforceability (IMP-012 tension).
