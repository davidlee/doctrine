# Implementation Plan SL-050: Priority surface efficiency and conceptual-precision cleanup

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Cleanup of the seven SL-047 review findings (F1–F7), already triaged in
`design.md` into two sections: §1 the shared-scan + existence-gate restructure
(F2 + F6 — the one foundational signature change) and §2 the six smaller
findings (F1, F3, F4, F5, F7) that ride on it. The plan keeps that split and goes
further: it isolates the *only* operator-observable change (the `explain` label
drop + the `priority.v1 → v2` envelope bump) into a single last phase, so the
three preceding phases are provably byte-identical and the review burden lands in
one place.

The governing constraint is the behaviour-preservation gate. None of the findings
is a correctness blocker; the surfaces are read-only and advisory. So the proof
that the efficiency work is safe IS the existing goldens staying byte-identical —
which only works if golden-changing work is quarantined from byte-identical work.
That quarantine is the spine of the phasing.

## Sequencing & Rationale

**PHASE-01 first because it is the signature change everything rides.** §1
introduces the `_from` seam (pre-scanned entry points) and moves the single corpus
scan up to `run_inspect`. Every later phase touches functions that this phase
reshapes or calls. Doing it first means PHASE-02..04 build on the settled seam
rather than racing it. It also carries the F6 existence gate — the one place a
*behaviour* legitimately changes outside the final phase (missing id: empty view →
error, VT-5 flips). Bundling F6 with F2 is deliberate: both live in the same seam
(the gate needs the projection the restructured surfaces already hold), and the
F-2 reorder — gate on the cheap relation graph *before* building the heavy
priority block — is itself a property of how the scan is rewired. Splitting them
would mean touching `run_inspect` twice.

**PHASE-02 (F1) and PHASE-03 (F3) are independent byte-identical refactors**,
ordered after the seam but mutually unordered — F1 lives in `scan_entities`, F3 in
`surface::survey`; they do not touch each other. They are kept as separate phases
(not merged) because they verify against different golden sets and carry distinct
finding provenance, and because a one-finding-per-phase tracking sheet keeps the
behaviour-preservation claim auditable per finding. Both must end with their
goldens unchanged — that green is the entire point.

**PHASE-04 last because it is the only phase that moves goldens.** F4, F5, F7, and
D4 are entangled and belong together: F5 drops `OrderContrib`, which (a)
mechanically deletes the second transitive walk and so resolves F4 with no extra
plumbing, and (b) takes the always-`None` `seq_rank` — an F7 dead-vocabulary item
— down with it. The remaining F7 drops (the `Fallback` variant, the `Dangling`
struct, the `ref_overlays` Vec) retire the same cluster of `dead_code`
suppressions, so doing them in one pass is what lets the gate go clean. D4 (the
`priority.v1 → v2` bump) is *forced* by F5 removing a field from the versioned
`--json` envelope, so it cannot be separated from F5. Running this phase last, on
top of an established byte-identical baseline, means its golden deltas are
isolable: the human surfaces stay byte-identical, the `--json` envelopes move one
`policy_version` line (plus the dropped field for `explain`), and nothing else.

The F7 graph-test rewrite is the one piece `design.md` deferred to plan/execute.
The plan's stance: drop the dead-artifact assertions but re-express their
*behavioural* core ("an unresolved target produces no edge") against the real edge
set, so test coverage of that property survives the removal of the `dangling` Vec
that happened to carry it. That is captured as an explicit PHASE-04 exit criterion
and verification, not left implicit.

## Notes

- The single-scan property (D3) is a *structural* guarantee — the two redundant
  `scan_entities` calls are gone from `run_inspect` — not a counter assertion. The
  behavioural proof is real-id goldens staying byte-identical.
- F1's "one parse per entity" holds for the common (non-RV/REC) path only; the
  scope sanctions RV's residual double read. Don't over-reach into a `review`-module
  refactor to chase it — that is out of scope.
- D2: the gate message diverges deliberately from `show`'s per-kind wording. These
  surfaces are kind-agnostic; a path would force a kind choice. Cross-kind,
  no-path `<CANONICAL>: no such entity` is the settled form.
- Watch the sweeps at execute: `blockers_json` for a `policy_version` assertion (not
  confirmed pinned), and any unit test constructing `Explanation { order_contrib, .. }`.
- Build/gate environment: jail target redirect (`cargo build` writes
  `~/.cargo/doctrine-target-jail/debug`; `./target/debug/doctrine` is stale);
  `just check` runs plain `cargo clippy`, NO `--all-targets`.
