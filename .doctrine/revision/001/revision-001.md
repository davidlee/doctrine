# REV REV-001 — reconcile SL-101

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-101 (Estimate & Value facets) shipped with two conscious divergences from
SPEC-020, both upheld by the User during the RV-082 design inquisition: the
design is the forward-looking authority and the spec must follow.

### Amendment 1 — Default estimation unit: `high_caffeine_hours` → `espresso_shots`

| Where | Before | After |
|---|---|---|
| spec-020.toml responsibilities[2] | `defaulting to \`high_caffeine_hours\`` | `defaulting to \`espresso_shots\`` |
| spec-020.md § Project-wide unit resolution | `defaulting to \`high_caffeine_hours\`` | `defaulting to \`espresso_shots\`` |
| spec-020.md § Display rendering example | `Estimate: 2-8 high_caffeine_hours` | `Estimate: 2-8 espresso_shots` |
| spec-020.md § Display rendering example | `Attention width: 6 high_caffeine_hours` | `Attention width: 6 espresso_shots` |
| FR-003 (REQ-271) acceptance criterion 2 | `defaults to high_caffeine_hours` | `defaults to espresso_shots` |

Driven by: RV-082 F-1 (design inquisition). The design chose `espresso_shots`; the
spec's `high_caffeine_hours` was the stale term.

### Amendment 2 — Value facet coverage

SL-101 shipped a `ValueFacet` (`src/value.rs` — a single f64 magnitude, unit
`magic_beans`) alongside `EstimateFacet`. SPEC-020 covered only the Estimate
facet; the Value facet was a vanguard without spec parentage. This amendment adds
Value facet coverage to SPEC-020:

- **New responsibility** in spec-020.toml: define the reusable `ValueFacet` model
  and its `[value]` parse seam.
- **New prose section** in spec-020.md: ValueFacet model, parse, unit resolution
  (`magic_beans`).
- **New requirements:** FR-007 (ValueFacet model), FR-008 (value validation),
  FR-009 (value unit resolution).
- **Overview updated** to reflect the dual-facet scope.

Driven by: RV-082 F-2 (design inquisition). The design shipped the Value facet;
the spec must retroactively cover it.
