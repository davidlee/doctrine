# Extract per-kind validation contract to break integrity coupling

## Context

RSK-227's coupling map identified `integrity` (out=13, in=13) as the symmetric
nexus of the command-tier tangle — the single highest-leverage knot in the
23-node core SCC. Tier-2 mitigation IMP-266. Sequenced *after* SL-203 (the
warm-up), which proves the extraction pattern on the trivial `{commands,
mcp_server}` cycle first.

`integrity` is a validator that knows every entity kind. Its up-reaches are thin
by nature — it imports each kind's marker constant to drive a validation
dispatch table (`src/integrity.rs:19-33`):

```
use crate::adr::ADR_KIND;  use crate::backlog::{…_KIND};
use crate::concept_map::CONCEPT_MAP_KIND;  use crate::knowledge::{…};
use crate::policy::POLICY_KIND;  use crate::rec::REC_KIND;
use crate::requirement::REQUIREMENT_KIND;  use crate::review::REVIEW_KIND;
use crate::revision::REV_KIND;  use crate::rfc::RFC_KIND;
use crate::slice::SLICE_KIND;  use crate::spec::{PRODUCT_SPEC_KIND, TECH_SPEC_KIND};
use crate::standard::STANDARD_KIND;
```

13 same-tier up-reaches, of which `backlog, rec, review, revision` also depend
back on `integrity` — 4 true 2-cycles feeding the core SCC.

## Scope & Objectives

- Invert the dependency (SL-016 pattern, larger scale): define a per-kind
  **validation contract** — the kind marker(s) + whatever validation surface
  `integrity` consumes — in a **leaf** module (a kind registry / contract).
- Each kind module implements / registers against the contract; `integrity`
  depends only on the leaf contract, not on each kind module.
- Result: the 13 `integrity → kind` up-reaches leave the command-tier subgraph;
  the 4 back-cycles break; much of the 23-node core SCC collapses.
- Ratchet the `command` tangle baseline in `layering.toml` down to the new count.

## Non-Goals

- Promoting the engine tier to its own workspace crate — that is the Tier-3
  endgame this slice *unblocks* (ADR-001's "when engine churn settles"), not
  this slice.
- Re-tiering `catalog` / `relation_graph` (the other core hubs) — out of scope
  unless the contract extraction naturally frees them; if so, note as follow-up.
- Any change to validation *behaviour* — pure structural inversion; the existing
  validation suites are the behaviour-preservation proof.
- Fan-out/degree gating (RSK-227 deferred).

## Affected surface

- `src/integrity.rs` (~1039 lines) — the validator; drops its per-kind imports.
- new leaf module — the kind-validation contract / registry (name in `/design`).
- the 13 kind modules — `adr, backlog, concept_map, knowledge, policy, rec,
  requirement, review, revision, rfc, slice, spec, standard` — export/implement
  against the contract rather than being imported by `integrity`.
- `.doctrine/adr/001/layering.toml` — `[tiers]` (integrity/kinds may re-tier)
  and `[tangle_baseline] command` ratchet.
- `tests/architecture_layering.rs` — verifies the SCC collapse.

## Risks / assumptions / open questions

- **OQ-1** — contract shape: is it just relocating the `*_KIND` constants to a
  leaf registry (thin), or does `integrity` consume richer per-kind validation
  logic that must move behind a trait (thick)? Sizing depends on this — `/design`
  must read `integrity.rs`'s actual use of each import first.
- **OQ-2** — registration direction: kinds push into a registry (kinds depend on
  the leaf) vs `integrity` pulls via a trait object. Both break the cycle;
  pick per coupling cost.
- **ASM-1** — the up-reaches are marker-constant-shaped (thin), so a leaf kind
  registry suffices for most. Verify against `integrity.rs` line by line.
- **R1** — 13-module migration; risk of sprawl. Keep the contract minimal;
  phase by kind if needed. Behaviour-preservation gate is the guardrail.
- **R2** — some kind modules may gain a new leaf dependency that itself creates
  edges; check the gate delta, not just integrity's local count.

## Verification / closure intent

- `tests/architecture_layering.rs` green with `integrity`'s 13 up-reaches gone,
  the 4 back-cycles broken, and the `command` tangle baseline substantially
  lowered (VT).
- Full validation + entity suites green, unchanged — behaviour-preservation (VT).
- `doctrine slice conformance` clean against seeded selectors (VA).
- Design confirms whether `integrity`/`catalog`/`relation_graph` are now
  re-tierable toward engine (informs the Tier-3 follow-up) (VA).

## Summary

## Follow-Ups
