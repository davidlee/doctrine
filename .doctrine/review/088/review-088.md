# Review RV-088 — reconciliation of SL-107

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Self-audit (conduct self/auto), facet `reconciliation`, mode **conformance**.
Subject: SL-107 — the hand-port of `candidate/101/review-001`'s Estimate & Value
integration delta onto main (base `5e8b2a64` → HEAD `1e382acd`, 6 files / +436).
Surface reviewed: the committed source on `main` (D3 hand-port — no candidate
interaction branch; this slice was not `/dispatch`-driven). Curated areas,
invariants, and risks: `domain_map.md` (this dir).

**Lines of attack:**

1. **Behaviour-preservation gate** (AGENTS.md) — do the existing suites stay green
   *unchanged*? Only additive deltas permitted. Independent full `just gate` is the
   proof, not the phase-sheet's recorded run.
2. **D1 dead-code tripwire** (design §4) — plain `cargo clippy` (no `--all-targets`)
   warning-clean AND no `expect(dead_code)` fires. An unfulfilled expect = a
   type/fn went live unexpectedly = scope breach.
3. **D2 narrow boundary** (design §7) — facets parsed, **not** rendered. No display
   path in `slice.rs` (VA-1); graph untouched. SL-107 ships data only.
4. **Design ↔ implementation conformance** — the §2 current/target table and §3
   per-file reconciliation match what landed.
5. **Plan EX-4 deviation** (5 → 6 expect surfaces, phase-sheet F-1) — is the extra
   module-level expect on `pub(crate) mod display;` correct, scoped, and durably
   recorded? Was it consulted?
6. **Contract non-revision** — PRD-014 / SPEC-020 (REV-002) unchanged; spec
   `validate` corpus clean.

## Synthesis

**Closure story.** SL-107 set out to land the integration delta SL-101 designed
but never delivered to `main` — the Value module, both facet configs in
`DoctrineToml`, both optional `SliceDoc` facet fields (parsed, not rendered), the
dead-code hygiene swap on `estimate.rs`, and the config example. All six target
files landed as the design's §2/§3 tables specify (+436 / 6 files). The audit
re-ran the evidence independently rather than trusting the phase-sheet record:

- **Behaviour-preservation** (the load-bearing gate, AGENTS.md): full workspace
  `just gate` → **2194 pass / 0 fail**, deltas additive only (V1–V7, dtoml
  default-config tests, SliceDoc round-trip/malformed). No existing
  entity-engine / slice / e2e suite changed or reddened. (F-1, aligned.)
- **D1 dead-code tripwire**: plain `cargo clippy` warning-clean, **zero expect
  fires** — the facet *types* went live via wiring while the display/graph
  *helpers* stay legitimately dead behind self-clearing `expect`s. (F-2, aligned.)
- **D2 narrow boundary / VA-1**: no facet display path in `slice.rs`; fields are
  serde parse-only. Display stays SL-102, graph SL-103. (F-3, aligned.)
- **Contract non-revision**: `spec validate` corpus clean; PRD-014 / SPEC-020
  untouched.

**Tradeoff consciously accepted (F-4, tolerated).** Plan EX-4 specified *exactly
5* item-level expects; the implementation ships **6**. The extra one is a
module-level `expect(dead_code)` on `pub(crate) mod display;`, covering three
unconsumed renderers SL-102 landed on `main` *after* EX-4 was authored — a surface
the plan could not have anticipated. It was consulted and User-approved (Option
1), touches zero of SL-102's code (D2 boundary intact), was empirically verified
to propagate-and-fulfil, and is durably recorded (phase-sheet F-1 + memory
`mem.pattern.lint.module-decl-expect-propagates`). The expect is self-clearing —
it fires unfulfilled the moment SL-102 wires display in — so the deviation cannot
rot into hidden dead code.

**Standing risks.** None blocking. The lone observable behaviour change is that a
*malformed* authored `[estimate]`/`[value]` on a slice toml now errors at parse
(validation live via the serde path); no corpus toml carries these tables today
(grep clean), so nothing existing trips it. The `expect` integration-debt is
tracked and self-documenting; SL-102/103 inherit the obligation to remove the
expects as they consume the helpers.

**Process note (F-5, aligned).** The initial baseline was red on two unrelated
date-dependent supersede tests (real-clock leak); the User reset `main` to a clean
green `5e8b2a64` before the port proceeded. The behaviour-preservation datum the
audit rests on is therefore sound.

## Reconciliation Brief

All five findings are terminal — four `aligned`, one `tolerated`. **No
spec/governance change and no per-slice canon edit is required.**

### Per-slice (direct edit)
- *(none)* — design.md / plan.toml are accurate to intent. The EX-4 5→6 deviation
  (F-4) is consciously tolerated, not a design defect: design §4 is a historical
  locked artifact and the debt is carried by the self-clearing `expect` + memory,
  not by a canon edit. `/close` records the 5→6 note in the slice `## Summary`
  rollup (already pending per the template's "filled at close").

### Governance/spec (REV)
- *(none)* — PRD-014 / SPEC-020 (REV-002) are authoritative and unchanged;
  `spec validate` corpus clean. SL-107 implements the reconciled contract, it does
  not revise it.

Hand-off to `/reconcile`: a clean brief — confirm the rollup, no write surfaces to
touch beyond the close-time summary.
