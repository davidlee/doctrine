# Implementation Plan SL-168: Unified corpus health doctor verb

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases turn the locked design (RV-183, 11 findings terminal) into a shipping
`doctrine doctor`. The spine: build the pure contract (PHASE-01), make the three
legacy Error sources callable as **adapters** (PHASE-02), add the lighter new
native checks (PHASE-03), then the heavy precision check ProseCite alone
(PHASE-04), wire the command + black-box goldens + the superset invariant
(PHASE-05), and only LAST perform the D12-gated native re-point of #1/#4 behind a
freshly-authored byte-exact golden (PHASE-06).

## Sequencing & Rationale

**Contract before consumers (PHASE-01 first).** The `finding` leaf is what every
check returns and what the command renders; nothing can be wired until it exists.
It is a pure leaf (ADR-001) so `integrity`/`registry`/`memory` import *down* into
it without a cycle, and `Category::severity()` is the single severity source (no
per-finding field to drift). Building it first lets every later phase be
test-first against a stable shape.

**Adapter-first, native last — the D12 ordering (PHASE-02 vs PHASE-06).** The
sharpest lesson of the design passes (RV-183 F-3/F-8): the byte-exact "safety net"
the pass-1 design leaned on does not yet exist — `validate` is only substring-
asserted and `memory validate` has no output golden at all. So a native re-point
of #1/#4 *cannot* be trusted until a real golden guards it. PHASE-02 therefore
takes only the no-risk step (make the sources callable, render them through
`from_lines`, leave the shipping commands untouched), and the native upgrade is
deferred to PHASE-06 where its first act is to author the missing goldens *red*.
The v1-native declaration is already satisfied by the four new checks (#5–#8,
native by construction), so #1/#4 going native is an upgrade, not a v1 gate —
adapter is an honourable fallback if drift proves non-trivial (R1). The memory
pure-fn extraction in PHASE-02 is forced regardless (no reusable findings fn
exists today), but its *render* stays legacy until PHASE-06.

**ProseCite gets its own phase (PHASE-04).** It is the heaviest and most
precision-critical check, and the two design passes spent most of their blood
there. It needs a *new* scanner — not a reuse of `line_cites`, whose alphanumeric
token boundaries would match `DEC-005` inside `DEC-005-C` and silently defeat the
3-part exclusion (F-1). The scanner must recognise the maximal hyphenated token,
carry fenced-block state across lines, and apply an exclusion set whose
completeness was verified empirically against the live corpus (F-1/F-2/F-4) — plus
a scan scope that diverges from `reseat`'s to skip the process-exhaust tier
including `.doctrine/review/**` (D11/F-5/F-9). Bundling this with the lighter
checks would starve it of attention.

**The lighter new checks cluster (PHASE-03).** Lifecycle, RawLabel, and TomlParse
are independent, native-by-construction, and each carries one design correction
worth its own tests: the done-but-open `≥1 linked slice` + open-item guards (F2/
F-6), the RawLabel seam pointing at `CatalogEdgeLabel::Raw` not the nonexistent
`RelationLabel::Raw` (F-7), and TomlParse scoped to facets+`plan.toml` so it does
not double-report entity-toml already owned by #1 (F-10).

**Command + proof (PHASE-05).** Once all eight checks exist, wiring `run_doctor`
is mechanical; the value is in the goldens (grouped table + `--json` envelope over
the built binary) and the superset invariant that pins doctor ⊇ validate by
comparing *rendered message strings* (F4), making a future `validate` retirement
(IMP-193) a clean step.

## Notes

- Exit semantics (D4): any Error finding → non-zero; warnings-only → 0. Advisory
  checks never break CI — the whole point of the severity split.
- No flags in v1 (D10): `--check`/`--strict`/`--verbose` deferred to backlog.
- `validate` stays (D9); doctor is repositioned as primary, removal tracked by
  IMP-193 once doctor's speed is proven (R6 — no shared corpus snapshot in v1).
- Open plan-time calls carried from the design: native-vs-adapter for #2/#3
  (OQ-A, lower-risk-first), grouped-table layout specifics (OQ-B, pinned at
  golden-authoring time), and whether D11's blunt scope cut needs a sharper
  example-detection heuristic (R8 residual noise).
