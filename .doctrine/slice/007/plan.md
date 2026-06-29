# Implementation Plan SL-007: Memory anchoring & capture: record scope+git frame, verify, git seam

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Six phases build the producer half of memory v1: doctrine's first git seam, the
parser/Memory widening that carries the anchor, the `record` capture path, the
`verify` verb, and the `show` consumer + close-out. The spine is the design's
pure/imperative split and the one non-negotiable from re-review: the born frame is
**doctrine's own frozen `GitContextFrameV1`, implemented byte-for-byte**, not
invented ad-hoc — because the same fileset must always
derive identical `repo_id`/`checkout_state_id` to dedup at the frame seam
([design.md](design.md) D2/D7; [audit.md](audit.md) B4).

The phases are sized so each ends green and each is a clean review unit. The git
seam splits pure (PHASE-01) from impure (PHASE-02) so the byte-identity logic — the
risky part — is unit-testable without a subprocess, and the conformance
golden-vector that pins the frozen frame lands the moment capture exists.

## Sequencing & Rationale

- **PHASE-01 → 02 (git seam, pure then impure).** The normalizer and
  `checkout_state_id` composition are pure functions; isolating them first means the
  repo_id reference table (the byte-identity proof) is asserted with no git involved.
  PHASE-02 then wraps them in `capture` (subprocess, normative flags, the
  born/unborn/non-repo + unstable-frame guards) and adds the conformance vector. This
  ordering also front-loads the highest-risk work (R3 frame drift) behind its
  strongest test.
- **PHASE-03 (parser/Memory) depends on 02** only for the finalised persisted field
  set (the `Anchor` mirrors the frame's persisted subset). It is otherwise read-path
  and legacy-compat work — the `serde(default)` widening + the explicit empty→`none`
  normalization (M1) — kept separate from any write change so the legacy fixture is
  the gate.
- **PHASE-04 (record) depends on 02 + 03.** The write path needs `capture` (the
  frame) and the widened `Memory`/template (somewhere to put it). It seeds the
  verify-mutable keys empty — the structural precondition for PHASE-05's F-1 guard
  (B3) — and is the only phase that intentionally changes SL-005 output (R1).
- **PHASE-05 (verify) depends on 04.** It can only reuse the adr F-1 missing-key
  guard once records reliably seed those keys; refuse-on-dirty (Q-B) and the atomic
  temp+rename write (M6) are the two behaviours that keep attestation honest.
- **PHASE-06 (show + e2e + docs)** is last: `show` is the first consumer of the
  populated anchor, and the end-to-end test exercises the whole producer against a
  real git repo. Close-out harvests durable findings and updates the CLI surface.

The dependency graph is a chain except that PHASE-03 and PHASE-04 both fan in from
PHASE-02; PHASE-03 can proceed in parallel with PHASE-04's template work if needed,
but the plan sequences them linearly for a single reviewer.

## Notes

- **Behaviour-preservation gate.** `src/entity.rs` is untouched all slice; its suite
  (plus slice/state) must stay green unchanged. The only intended behaviour change is
  `record`'s rendered output (PHASE-04) — an SL-005 verb whose own tests update.
- **Self-contained implementation (D7).** doctrine implements the frozen frame
  in-tree and pins equivalence with the golden-vector. If conformance ever needs
  to be shared, extracting a `git_context` crate is a future slice/ADR, not this one.
- **adr's plain write (M6).** `adr::set_adr_status` uses a non-atomic `fs::write`;
  `verify` will not copy that — it writes temp+rename. Unifying both behind one
  atomic-editor helper is out of scope here (don't touch adr unannounced).
- **just drift.** `just check` does not exist (CLAUDE.md references it); the gate is
  `cargo clippy` + `cargo fmt` + `just lint && just test`. Not fixed in this slice.
- **Plan, then phases.** Materialise per-phase tracking with
  `doctrine slice phases 7`, then detail each phase in its
  `state/.../phase-NN.md` just prior to execution (the core process).
