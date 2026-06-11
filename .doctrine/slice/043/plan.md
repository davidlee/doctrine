# Implementation Plan SL-043: cordage scale & robustness hardening

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-043 is the green/fix half of the cordage scale cluster: SL-038 built the
generators, the four `#[ignore]`'d characterization reds, and the demo; this
slice makes the core hold at the ~tens-of-thousands target and flips those reds.
The design is locked through three adversarial passes (design lock → codex R2 →
Opus inquisition R3). Three fix clusters, three phases, partitioned by
file/concern (design D6) — the same boundaries the design's §5.4 P1/P2/P3 draw.

The load-bearing constraint across all three phases is the **behaviour-preservation
gate**: the pre-existing suite is the equivalence proof for every iterative rewrite
and must stay green *unchanged* — the single sanctioned exception is the four
`explain` tests in PHASE-03, which deliberately re-assert the new cone shape.

## Sequencing & Rationale

**Why three phases, in this order.** The phases are dependency-ordered, not merely
grouped. PHASE-01 delivers the iterative Tarjan that is the *single* SCC primitive
shared by both RSK-003 sites and reused conceptually downstream — so it lands first.
PHASE-02 and PHASE-03 are independent of each other (evaluate vs explain), but both
sit behind PHASE-01 and run as a pipeline P1 → P2 → P3 so each phase flips its own
red against a green baseline. No phase widens scope into the IMP-020 traversal
consolidation (design OQ-2, default out).

**PHASE-01 — resolve.rs.** Two independent overflow sites (`strongconnect`,
`level_of` — the latter blows on a clean acyclic chain) become explicit-stack
iterative rewrites, and the eviction-to-fixpoint passes become per-SCC localized.
The correctness argument is not byte-identical emission order (A-4 R2 reframed it
as *consumer order-insensitivity*); the localization argument rests on SCC
disjointness plus the **layer-k invariant** (G2 / A-3): every U-cycle present at
layer k contains a `layer_k` edge, because `compose_order` drives each prior layer
to fixpoint before inserting layer k. That invariant is a verifier guard, not an
assumption — VT-1 pins it. The `dense_evict` red cannot go linear-green in scope
(EXC-2 is deferred, not inherent — OQ-3); it stays `#[ignore]` and a new
many-small-cycles gate proves the real win plus evicted-set identity.

**PHASE-02 — query.rs evaluate.** The dominant risk of the whole slice. The
condensation fold reuses the build-time SCC *partition* but must rebuild the
condensation **edges and reverse-topo direction-resolved** — the linchpin the codex
R2 pass sharpened (A-2) and the inquisition R3 pass extended to the partition itself
(C1). Folding a forward-built DAG for an `Against` channel, or grouping a stored SCC
under `Direction::None`, silently corrupts every value — and the existing suite
cannot see it because no existing VT fixture has an SCC. Hence the mandatory G1
fixture matrix `{Along, Against, None} × {Max, CountDistinct}` over one degraded
`Reject` SCC: the `None`×cyclic and `Against`×cyclic cells are the two
silent-corruption surfaces. CountDistinct keeps its strict per-member exclusion
(C2 / F34) rather than sharing the SCC result — VT-2 guards the off-by-one.

**PHASE-03 — query.rs cone + lib.rs explain.** The exponential chain enumeration
collapses to a single cone-builder with a *global* visited set (the linearisation,
A-5). This is the only public-surface change (`paths` → `predecessors`), free
because no non-test consumer reads `explain`. The reshape is to the pure predecessor
sub-DAG, no bundled witness chain (D2 — D11/F13 role-agnostic structure). G3 is the
guard against the one sanctioned suite break masking a regression: the rewritten cone
tests must assert the *same* reachable-predecessor membership the old chains covered,
re-expressed as adjacency — roots, SCC-entry endpoints, the `{n:{}}` self case — not
a weaker shape.

## Notes

- **OQ-1 (REQ pin) resolved here**, recorded in `plan.toml [requirements]` and
  mirrored into `slice-043.toml relationships.requirements`. Rationale: the slice
  hardens the generic-core perf/robustness posture — `REQ-078` (recompute-from-
  authored at scale; the H1 O(V+E)/non-recursive guarantee) and `REQ-092`
  (per-overlay cycle policy) are the obligations *realised*; `REQ-076` (degrade,
  never a false order) and `REQ-077` (determinism) are *depended-on* contracts the
  behaviour-preservation gate proves are preserved.
- **OQ-2 (IMP-020 fold-in): out** — kept the slice at risk-closure, not a traversal
  refactor. **OQ-3 (incremental dense-SCC eviction): deferred** to a future IMP if a
  dense-SCC workload ever appears; `dense_evict` is its `#[ignore]` marker.
- Each phase ends on the project gate: `cargo clippy` zero warnings (bins/lib only,
  not `--all-targets`); `just check` before the commit. Cliff-test bounds budget
  debug ≈10× release; seed opaque ids from the builder, never `NodeId(0)`.
