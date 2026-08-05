# IMP-404: Execute SL-112's deferred engine/leaf crate extraction

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Promote ADR-001's tiers from a test-enforced convention to compiler-enforced crate
boundaries: extract `leaf` and `engine` as workspace crates, leave the command tier
in the root binary.

## Why this is latent rather than new

`SL-112` (done, 2026-06-19) faced this fork and split it deliberately: ship a `syn`
dependency-fitness test **now**, defer the crate split to a follow-on slice, because
*"the layer map this slice authors de-risks that later cut."* ADR-001 itself names
crate promotion as the escalation past a fitness function once cycles recur.

That follow-on was never captured. It has been sitting in a done slice's scope
document since June. This item is that capture, not a new proposal.

The de-risking artefact is already banked: `tests/architecture_layering.rs` carries
the authoritative module→tier `LAYER_MAP` with per-module rationale, a hard gate on
upward cross-tier edges against a frozen `ACCEPTED_VIOLATIONS` set that may shrink
but never grow, and a monotonic-down ratchet on intra-tier cycle counts — all under
`just gate`. SL-112 also found ADR-001's own tier table *wrong* rather than merely
stale, so the LAYER_MAP supersedes it as the classification of record.

## Why it matters now

`DEC-153` answers `QUE-207` with two binaries — `doctrine` (agent-facing, runs in the
capsule) and `doctrine-control` (SPEC-030 transaction verbs, control host only) —
and takes the cheap path: a lib target on the root package, with `doctrine-control`
as a workspace member depending on it. No crate extraction in v0.

The accepted price is that `doctrine-control` links the root crate and inherits every
embed root (`install/`, `plugins/`, `memory/`, `publication/`, `web/map/dist/`,
`templates/mcp.ts`). This extraction is where that stops: the embed roots hang off
command-tier and engine modules the control plane does not need, so a control binary
depending on `leaf` + a narrow `engine` sheds them.

Note the authority boundary does **not** depend on this work. Cargo refuses
package-level cycles, so the one-way rule (`agent-facing tools -X► control-plane
implementation`) is a compiler error from day one under the cheap path. This is a
cost-and-hygiene improvement, not a correctness prerequisite.

## Triggers

Not urgent; extraction is behaviour-preserving, so the decision does not get harder
by waiting. Concrete triggers rather than an open-ended "later":

- the control binary's size or the `flake.nix` `srcWithDist` graft burden bites —
  the failure mode is a silently hollow binary with no compile error, so this wants
  a canary rather than a judgement call;
- a **third** consumer appears (a minimal capsule-side binary, or `QUE-207` option C
  remote execution), forcing a real interface rather than re-exports;
- workspace compile times under two binaries linking one fat lib.

## Notes

- Precedent is `crates/cordage` — a product-neutral leaf, depended on by path, never
  the reverse (ADR-001 leaf).
- `just check` already carves out the cordage member. Extraction adds cases; the
  carve-out logic needs to stay comprehensible rather than accreting.
- Do **not** build `QUE-207`'s six-crate sketch (`doctrine-core`, `-protocol`,
  `-graph`, `-documents`, `-prompts`, `-capsule`) speculatively. Extract along the
  tiers the LAYER_MAP already classifies, when a consumer forces the boundary.
