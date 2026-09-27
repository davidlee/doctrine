# Apply truth residue

## Context

`SL-259` (*Truthful apply*) set the design run's exit-signal contract: an error
means nothing landed, a success reports rows that tell the truth, and input the
engine will not honour is refused by name. `RFC-031`'s 2026-09-27 triage found
four residual defects of that class, grouped there as clump 1:

| item | defect |
|---|---|
| `ISS-482` | `blocking: null` on a **finding** declaration reads as absent (non-blocking) — success for input the engine did not honour as written. The inquiry-node home already refuses `null` (`Refusal::BlockingJudgementWithdrawn`); the wire field is already `Sparse<bool>`. |
| `IMP-499` | `Refusal::SubmissionExpired` names no remedy. Its sibling `SubmissionReplayed` gained one under `ISS-361`. |
| `ISS-454` | A coverage-dead act that is later displaced is reported `act_invalidated` **twice**: `admit_and_record` reports displacement from the act store, which retains acts `live_acts` has already filtered out. |
| `ISS-488` | Re-wording an inquiry node's question emits **no change row**, though `REQ-478` (*Report every recorded mutation on the change log*) requires one, and under `DEC-301` a re-word re-faces the user's acts — so the log shows the act voided without the edit that voided it. |

## Scope & Objectives

Close all four, each with a behavioural test:

1. **Refusals** — refuse `null` `blocking` at the finding home, reusing the
   inquiry home's refusal shape; give `SubmissionExpired` a remedy that routes
   the caller to check whether the original landed before resubmitting (a blind
   resubmit could double-apply).
2. **Change rows** — suppress the displacement `act_invalidated` row when the
   displaced act was not live before the apply; add a question-changed member
   to the emittable `ChangeEvent` vocabulary, emitted on a re-word and driven
   by the fixture ladder (`REQ-478`'s roster criterion).

Affected surface: `src/design_run/{run.rs, refusal.rs, change_log.rs,
submission.rs, snapshot.rs}`, `src/design_run/render/change_row.rs`, the
design-run fixture ladder, `tests/e2e_design_state.rs`.

## Non-Goals

- `IDE-057` — whether a traversal-only apply owes a change row. Same
  requirement, deliberately excluded to keep the slice bounded.
- Batch fold order (`ISS-356`, `ISS-360`) — `RFC-031` clump 2, its own slice.
- Stage-guarding acts (the residual question left by `ISS-362`).

## Risks & Assumptions

- **Stored-state compatibility.** A new change-log event written by this binary
  and read by an older one degrades to a disclosed unreadable row (`DEC-249`;
  `STD-003`), not a refusal — the accepted path for vocabulary growth. Live
  state types are untouched.
- **`ISS-488` reverses a code comment, not a decision.** `run.rs` records a
  question change as "state, not delta" per `SL-233`'s projection-bounds
  sketch; `REQ-478` (accepted, later) supersedes that reasoning. The design
  confirms this reading rather than assuming it.
- **`ISS-454` code-vs-prose.** `RV-367` `F-1` recommends fixing the code; the
  design settles it.

## Verification

- Each item has a test that fails before its fix: e2e for observable output
  (refusal text, emitted rows), unit where the seam is internal.
- The emittable-roster test drives the new event.
- `doctrine check gate` green.

## Summary

## Follow-Ups
