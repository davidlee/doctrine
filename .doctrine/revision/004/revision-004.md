# REV REV-004 — reconcile SL-112

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-112 shipped a `syn`-based dependency-fitness gate (`cargo test --test
architecture_layering`) that enforces the ADR-001 layering rules against the
production `crate::` dependency graph. The gate is green (17/17 tests pass),
with honest baselines: 10 frozen accepted violations and command-tier tangle
(120 cyclic edges) openly unmet-and-tracked. RV-104 F-1 (per-slice fix) applied.

This REV amends ADR-001 to stop contradicting the shipped gate — ADR-001's
Verification section currently *rejects* this test, creating a live governance
contradiction until amended (design D5, EX-3).

**Changes:**
- Verification: overturn the "Explicitly rejected — homegrown module-graph
  unit test" stance. Record the `syn`-based gate as now-enforcement. Reasons:
  the cycles arrived (SL-111 broke engine→command); `syn` removes the
  brittleness that grounded the original rejection.
- Decision: replace the stale per-module tier table (written when ~16 modules
  existed; now 67) with tier *definitions* + a pointer to
  `.doctrine/adr/001/layering.toml` as the authoritative assignment.
- Rule 1: note hard-gated over literal `crate::` path edges.
- Rule 2: amend from "No cycles" to a non-increasing cyclic-edge ratchet
  (per-tier, same-tier edges inside non-trivial SCCs). The command tangle is
  openly recorded as unmet-and-tracked (120 cyclic edges); the engine tangle
  baseline is 0 (CHR-015 done).
- Rule 3: note as deferred (impure-leaf refinement is follow-up work).
- Consequences negative: update "Enforcement is review-only" to note the
  `syn` gate now exists.
- `input` reclassified as engine (design F-6).

**Covered finding:** RV-104 reconciliation brief → governance/spec REV.

### Before/after excerpts

#### Verification — before (current ADR-001)

> - **Explicitly rejected — a homegrown module-graph unit test.** Parsing source
>   to assert an adjacency allow-list is brittle toil and conflicts with the
>   project's "test behaviour, not trivial implementation" standard. Wait for
>   the crate boundary instead.

#### Verification — after (amended)

> - **Now — the `syn`-based fitness gate.** SL-112 shipped
>   `tests/architecture_layering.rs` — a `cargo test` under `just gate` that
>   extracts the production `crate::` dependency graph via `syn` (canonical AST;
>   not brittle regex) and enforces: (1) cross-tier direction hard-gated over
>   literal `crate::` path edges, modulo a frozen `ACCEPTED_VIOLATIONS` baseline;
>   (2) intra-tier cycles ratcheted by a per-tier cyclic-edge count that may not
>   grow; (3) mixed umbrellas forced to sub-classify. The authoritative tier
>   assignment is `.doctrine/adr/001/layering.toml`. A new upward edge fails
>   `just gate` naming the offending edge.

#### Decision tier table — before (current)

> | Tier | What it is | Members (current) | May depend on |
> |---|---|---|---|
> | **leaf / seam** | primitives & IO seams; no domain knowledge | `clock`, `root`, `fsutil`, `input`, `git` (impure seam) | nothing in-crate (or other leaves) |
> | ... | ... | ... | ... |

#### Decision tier table — after (amended)

> | Tier | What it is | May depend on |
> |---|---|---|
> | **leaf / seam** | primitives & IO seams; no domain knowledge | nothing in-crate (or other leaves) |
> | **engine** | core domain model & state machinery, pure-first | leaf tier |
> | **command** | CLI shells — one per verb, thin | engine + leaf |
>
> The authoritative per-module tier assignment lives at
> `.doctrine/adr/001/layering.toml` — a companion TOML surface under this ADR's
> directory. The map classifies 67 units (23 leaf / 18 engine / 26 command) with
> per-unit rationale, mixed-umbrella sub-classifications, and frozen baselines.
