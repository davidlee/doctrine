# SPEC-020: Estimate hardening — NF-001 tripwire + confidence legitimization

## Context

SL-101–103 implemented and integrated the estimate/value facets onto `main`: models,
parse/validate, project-wide unit resolution, catalog scan-side read, and the
policy-free graph projection (`units` block + per-node `estimate`/`value`). The
stale-memory claim that the facets are dead/unwired is **no longer true** — only the
human-`show` display path remains unwired (deferred to IMP-112).

This slice is a **deliberately narrow** hardening pass. It was challenged as ceremony
beyond its two real deliverables and cut to them: the NF-001 guarantee that nothing
yet enforces, and the confidence residue stranded unspec'd by SL-101. Plus one cheap
test for a genuinely untested contract.

**Depends on SL-101–103** — runs after the feature is built and integrated.

## Scope & Objectives

- **Confidence spec legitimization (governance).** `src/estimate.rs` carries
  `lower_confidence`/`upper_confidence`/`resolve_confidence` + `DEFAULT_*_CONFIDENCE`
  with no governing REQ. Author a `REQ` homing the **percentile model** — `lower`/
  `upper` interpreted as the project's P-low/P-high band; defaults `0.1`/`0.9` from
  `doctrine.toml [estimation]`, configurable; finite, in `[0,1]`, low < high. Intent
  is **display framing only** — explicitly no gating/aggregation effect in v1. Amend
  SPEC-020 to home the REQ, via a Revision folded into reconcile (ADR-013, REV-002
  precedent). Confidence is estimate-only. Code stays `expect(dead_code)` until
  IMP-112; this slice only corrects the **stale** `expect` reason strings (they wrongly
  cite SL-102/103, which never touched confidence).

- **NF-001 (REQ-275) — structural non-blocking, enforced.** Baseline holds (zero facet
  refs in gating modules) but nothing pins it. Add a two-tier dependency-free proof:
  Tier 1 — a source-scanning `tests/` allowlist test confining facet symbols to the
  known exposure files; Tier 2 — a compile-time exhaustive-destructure guard on `Gate`
  (the closure-gate input type carries no facet field). Realises the spec's
  "structural, not run-proven" demand. The honest residual gap (a hand-written close
  fn in `slice.rs` reading the facet) is documented in `audit.md`, not test-covered.

- **Value asymmetry test (FR-008).** `[value] value=-5` is valid (value has no range
  constraint, unlike estimate's `lower>=0`) — currently untested. One unit test pins it.

## Non-Goals

- **Confidence display / graph exposure** — deferred to IMP-112 (wire estimate
  display, incl. percentile framing, into the `show` path).
- **Dogfood on real entities** — cut: re-proves NF-002 (already green via graph VT-3)
  and sits inert until a consumer (IMP-112 / Cordage) exists.
- **NF-002 / NF-003 re-verification** — already green (graph VT-3; `e19`/`v7`/
  `custom_deserialize_unknown_keys`); cited at audit, not re-built.
- **Redundant edge tests** (large bounds, int≡float) — covered by `e2`/`e3`/`e4`.
- New features otherwise; write/edit CLI for estimates.

## Summary

Two real deliverables plus one cheap test: enforce NF-001 structurally (allowlist scan
+ `Gate` compile-guard), legitimize the confidence residue (percentile framing,
display-only) via a reconcile-folded Revision, and pin the untested value-asymmetry
contract. Leaves the facets production-ready and free of unspec'd dead code, with no
inert ceremony.
