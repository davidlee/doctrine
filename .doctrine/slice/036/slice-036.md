# cordage graph core crate

## Context

First slice descending from **SPEC-001** (Graph-Derived Priority Engine, now
`active`). SPEC-001 **D1** carves the generic graph core into its own workspace
member crate **`cordage`** at `crates/cordage/` — product-neutral, published-grade
from day one (neither `doctrine-` nor `bough-` prefixed; both, and other products,
are intended consumers). Doctrine stays the workspace root crate and depends on
`cordage` **by path**; the crate is a **leaf** the doctrine engine depends on,
never the reverse (ADR-001). The crate exists today only as a single-member
workspace (`members = ["."]`); this slice adds the second member.

This is the foundation the doctrine **policy** and **adapter** layers (later
slices) build on. Carving the neutral seam *now* makes "ultimately external" a
`cargo publish` + flip-path-to-version, not a refactor (H4).

Governing canon (prose links, not edges): SPEC-001 §Decisions (D1–D11),
§Concerns, Appendix B (forbidden-core list); ADR-001 (module layering, leaf);
ADR-004 (relations outbound-only — reverse edges are derived, never stored).

## Scope & Objectives

Deliver `crates/cordage` as the **generic multi-channel evaluation engine over a
tree plus typed directed (DAG) overlays** — *not* a backlog-priority library.
Per D1's ownership split, the **core** owns:

- **Node/edge model** — opaque, stable node ids; typed directed edges; per-edge
  opaque attributes (`rank`, `age`) the core orders by but never interprets.
- **Overlay species selection** — an edge maps to one overlay; the core sees only
  an overlay's configured cycle policy, never `dep`/`seq` *meaning* (D4).
- **Reverse-edge index + reachability** — blocking closure, dependents, rollups,
  derived purely from authored outbound edges (H2, ADR-004).
- **Per-overlay cycle policy (D5/REQ-092):**
  - *reject-on-cycle* — raise a diagnostic naming node ids + edge kinds; never
    emit a false topological order (REQ-076).
  - *evict-on-cycle* — evict the globally-minimal participating edge under
    `(rank asc, age asc)`, repeat to a **fixpoint**; every evicted edge surfaced
    in provenance (REQ-092). Build-time *derived* resolution, never an authored
    mutation.
- **Union composition (D9)** — `order_key` built dep-topology first, then `seq`
  rank within a dep-eligible set, then deterministic fallback; a `seq` edge that
  would close a cycle against the resolved dep order evicts by the same rule.
- **Deterministic ordering** — `order_key` an explicit **total** tuple; no clock,
  RNG, or map-iteration order in any ordering path (REQ-077).
- **Generic channel propagation** — propagate typed channels over typed edges with
  chosen combinators (e.g. backward `max` over a DAG), knowing nothing of channel
  *meaning*; storage shaped so the v1 channel set is **not** assumed exhaustive
  (REQ-080 seam).
- **Structured provenance/explanation** — cycle diagnostics, evicted edges,
  explanation paths — structured data, not prose.

Closure intent: REQ-076, REQ-077, REQ-079, REQ-080, REQ-092 (the generic-core
subset). The crate honours the repo's pure/imperative split (no clock/RNG/git/disk
in the core) and the repo clippy posture (BTree not Hash, no indexing-slicing, no
`as`, expect+reason not bare allow).

## Non-Goals

Out of scope — later slices, named so the boundary is explicit:

- **Doctrine policy layer** (`src/priority/…` in doctrine): channel *semantics*,
  active-state rules, terminal-for-priority resolution, relation-kind → overlay
  classification, rank↔consequence combination, terminal/promoted inclusion,
  explanation **rendering**. (Core provides the mechanism; policy the meaning — D2.)
- **Registry adapter**: registry scan, doctrine-id↔node-id mapping, edge emission
  from authored relations, node attributes, the `age` ordinal derivation,
  diagnostic re-mapping back to doctrine ids.
- **CLI surfaces** `survey` / `next` / `explain` / `blockers` / `inspect` and any
  disposable cache/projection (D7/D8) — the core is pure and touches no disk, so
  REQ-074 (inbound-reference *display*) and REQ-078 (disposable cache) are
  delivered as primitives here but verified in the doctrine-side slices.
- **Authored capture schema** `needs`/`after`/`triggers` + `rank` — PRD-009's
  product-capture surface (FR-010/FR-011, `REQ-096`/`REQ-097`/`REQ-098`); consumed
  downstream, never minted here.
- **D6 trigger actionability mask** — policy-layer, and blocked on two unbuilt
  file-set sources (OQ-009 resolved-pending).
- **crates.io publish** — deferred; path dependency only (D1).

## Affected surface

- `crates/cordage/Cargo.toml`, `crates/cordage/src/lib.rs` (+ modules) — new.
- root `Cargo.toml` — add `crates/cordage` to `[workspace] members`; any new
  shared deps go through `[workspace.dependencies]`.
- doctrine root crate gains a **path** dependency on `cordage` only when the first
  consumer (adapter slice) lands — this slice may leave doctrine not yet depending
  on it (the crate stands alone with its own suite).

## Risks / Assumptions / Open Questions

- **Boundary purity is load-bearing and the hardest part (REQ-079).** The §9
  boundary test (D2) governs every core/policy placement; Appendix B is the
  standing prohibition list (no task/project/habit/deadline/urgency/scheduling/
  product vocabulary). The acceptance proof is **structural**: the core suite
  carries zero doctrine entity vocabulary and zero time/scheduling/commitment
  semantics.
- **Clock-free determinism.** Ordering rests on the explicit total `order_key`
  tuple, never incidental map order. `age` is an **adapter-supplied stable
  ordinal**, not wall-clock — the core depends only on it being *total and stable
  across recomputes*; it does not derive it (D5).
- **OQ (→ /design):** how broad the channel-propagation/combinator API should be
  in v1 (only what the policy needs vs a fuller algebra) without leaking meaning
  or foreclosing REQ-080; whether to take a graph dependency (e.g. `petgraph`) or
  hand-roll a small reverse-index + traversal (H2/the small-corpus posture lean
  hand-rolled); internal module decomposition.

## Verification / Closure intent

- `cordage` builds as a workspace member; `just check` green.
- **Boundary proof (REQ-079):** core test suite green with **zero** doctrine /
  product vocabulary; the crate compiles with no doctrine dependency.
- **Cycle fixtures:** dep-cycle → diagnostic naming ids/kinds + safe degrade, no
  false topo (REQ-076); seq-cycle → evict-to-fixpoint, evicted edge in provenance
  (REQ-092); union-cycle `A —dep→ B`, `B —seq→ A` → stable order, `seq` evicted,
  no panic, no false topo (D9).
- **Determinism (REQ-077):** same graph + options → identical order + explanation
  across runs; no clock/RNG/Hash-iteration in any ordering path.
- **Seam (REQ-080):** channel storage admits channels beyond the v1 set; node ids
  opaque/stable per run; relation kinds typed, never prose-encoded.
- clippy zero-warnings under the repo posture (plain `cargo clippy`).

## Follow-Ups

- Adapter slice — registry scan → opaque nodes/typed edges, id mapping, `age`
  derivation, diagnostic re-mapping (verifies REQ-074).
- Policy slice — channel semantics, classification, rank↔consequence, render.
- CLI/cache slice — `survey`/`next`/`explain`/`blockers`/`inspect` + disposable
  stamped projection (verifies REQ-078, D7).
- D6 trigger mask — once PRD-009 `triggers` + the plan/audit file-set sources land.
