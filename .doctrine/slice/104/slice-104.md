# SPEC-020: Estimate hardening — NFR verification, confidence legitimization, polish

## Context

SL-101–103 implemented and integrated the estimate/value facets onto `main`: the
models, parse/validate, project-wide unit resolution, catalog scan-side read, and
the policy-free graph projection (`units` block + per-node `estimate`/`value`). The
stale-memory claim that the facets are dead/unwired is **no longer true** — only the
human-`show` display path remains unwired (deferred to IMP-112).

This slice is the hardening pass: it pins the one NFR guarantee that nothing yet
enforces, legitimizes the confidence residue stranded by SL-101, dogfoods the facets
on real entities, and adds defensive edge-case coverage.

**Depends on SL-101–103** — runs after the feature is built and integrated.

## Scope & Objectives

- **Confidence spec legitimization (governance).** `src/estimate.rs` carries
  `lower_confidence`/`upper_confidence`/`resolve_confidence` + `DEFAULT_*_CONFIDENCE`
  with no governing REQ — SPEC-020/PRD-014 never mention confidence. Author a `REQ`
  homing it: the **percentile model** — `estimate.lower`/`upper` are interpreted as
  the project's P-low/P-high band; defaults `0.1`/`0.9` resolved from
  `doctrine.toml [estimation]`, configurable; finite, in `[0,1]`, low < high. Intent
  is **display framing of the bounds, nothing more** — explicitly no gating and no
  aggregation effect in v1. Amend SPEC-020 to home the REQ. The amendment routes
  through a Revision folded into SL-104's reconcile, mirroring REV-002 (governance
  dependency, ADR-013). The confidence code stays `expect(dead_code)` until IMP-112
  consumes it; this slice only corrects the **stale** `expect` reason strings (they
  wrongly cite "consumed by SL-102/103," which never touched confidence).

- **NF-001 (REQ-275) — structural non-blocking, enforced.** Baseline already holds
  (zero estimate/value refs in the gating modules). Add a source-scanning `#[test]`
  that **pins** it: the gating predicates (`dispatch`, `lifecycle`, `reconcile`,
  `governance`, close-gate, audit) carry zero `estimate`/`value`/`confidence` reads.
  Self-fails if a future change wires a facet into a predicate — this realises the
  spec's "structural, not run-proven" demand.

- **NF-002 (REQ-276) — kind-agnostic, confirmed on real data.** Already proven by
  the generic scan read and `graph.rs` VT-3 (ADR alongside slice). Reinforce with
  the dogfood (≥2 kinds) on real data; no new mechanism.

- **NF-003 (REQ-277) — forward-compat, confirmed.** Already covered for both facets
  (`e19`, `custom_deserialize_unknown_keys`, value `v7`, `*_raw_absorbs_unknown_keys`).
  No new mechanism; cross-reference only.

- **Dogfood.** Author real `[estimate]`/`[value]` tables on a few live entities
  spanning **≥2 kinds**; verify they parse, list, validate, and round-trip cleanly.

- **Edge cases.** Large bounds; int-vs-float normalization round-trip (`2` ≡ `2.0`).
  Zero-width (`lower == upper`) already covered by `e4`.

## Non-Goals

- **Confidence display / graph exposure** — deferred to IMP-112 (wire estimate
  display, incl. percentile framing, into the `show` path).
- New features otherwise — this slice hardens, it doesn't build.
- Write/edit CLI for estimates.

## Summary

A hardening and legitimization pass: enforce NF-001 structurally, spec the
confidence residue (percentile framing, display-only intent) via a reconcile-folded
Revision, dogfood the facets on real entities, and add edge-case coverage. Leaves
the estimate/value facets production-ready and free of unspec'd dead code.
