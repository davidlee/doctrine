# Implementation Plan SL-013: memory skills install ergonomics + off-script skill-port record

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

One implementation phase. The slice is a small, single code surface — a
convenience selector that rides the existing `select`/`validate_filters`/
`build_plan` pipeline unchanged. The design's deliverable 1 (the flag) is the
only buildable work; deliverable 2 (the off-script port record) is already
discharged by the scope document and is verified, not authored, at close.

## Sequencing & Rationale

**Why a single phase.** The pure core (`subset_ids`, `resolve_install_ids`) and
the CLI wiring (`--only-memory` clap arg, the `run_install` thread) are one
coherent unit. Splitting them would land tested-but-uncalled functions in a first
phase — dead code the repo's clippy denials reject. Keeping them together lets a
single red/green/refactor cycle end green with no suppression and no artificial
wiring. The inquisition's gain (Charge I) is preserved regardless: the pure layer
is independently testable *within* the phase — that is a structural property of
`resolve_install_ids`, not a phase boundary.

**TDD order within the phase.** Build inside-out so each step has a failing test
first:
1. `subset_ids` — pure path→id extraction (VT-1, synthetic paths).
2. `resolve_install_ids` — derivation + the empty-set bail (VT-1 bail arm); this
   is where the D3 guard lives, reachable without the embed.
3. live derivation pins the embed-follows-symlinks assumption (VT-2).
4. derived ids → `validate_filters` + `build_plan`/`select` = exactly two
   (VT-3) — exercises the cross-domain identity invariant.
5. clap arg + `conflicts_with_all`; `run_install` gains `only_memory` as a thin
   shell (VT-4 parse-time conflict).
End green; existing suites unchanged (behaviour preservation); clippy clean.

**Why VT-5 and VT-6 are not phase work.** VT-5 (marketplace install-smoke) is a
one-shot manual action whose evidence lands in `audit.md` — an `/audit` concern,
not a build step. VT-6 (the deliverable-2 record) is a closure attestation that
the scope document already carries the off-script port history. Both are listed
in `plan.toml` so the VT ledger is complete and nothing orphans (the lesson of
inquisition Charge II), but they are dispositioned at `/audit` and `/close`, not
implemented in PHASE-01.

## Notes

- No new persistent state; the derived id set is computed per-invocation from the
  compile/runtime embed (design §5.3).
- Behaviour-preservation gate: the existing `skills install` suites are the proof
  the additive selector changes nothing downstream — they must stay green
  unchanged. `run_install`'s new `only_memory` parameter is threaded by the
  `main.rs` call site, not by the pure-layer tests, so those suites are
  untouched.
- Pure/imperative split: both `subset_ids` and `resolve_install_ids` take their
  input as a path iterator — no embed or disk in the pure layer (design §3).
