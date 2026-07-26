# REQ-385: Every funnel verb is legality-gated on funnel position: it refuses out-of-order execution and names the expected next verb (report-and-halt); conclude refuses after skipped or failed verification.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

The statement carries two claims of different strength. The distinction is
load-bearing and was settled by REV-039 (SL-228 reconcile, RV-312 F-5):

- **Positional guidance — normative, and proven.** A verb reads position, refuses
  an out-of-order attempt, and names the expected next verb. Evidence: a
  memory-blind orchestrator with no corpus access and no rescue drove a full
  funnel and a crash recovery unaided (SL-228's OQ-5 benchmark; accepted at VH-1,
  2026-07-27).
- **Prescription completeness — a goal, not a claimed property.** That a
  refusal's text is *by itself* a sufficient recovery procedure does not hold
  today, and the requirement does not assert it. Verification vehicle: **IMP-321**
  (`after: SL-228`).

The phrase `with prescription` was removed from the statement at REV-039 because
it read as the second claim while only the first was earned.

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->

The force is zero-rescue. An orchestrator that cannot consult a human or a memory
corpus has only the verb's own output to act on, so a refusal that halts without
orienting the caller converts a recoverable state into a stuck one. Positional
naming is the minimum that makes the funnel drivable blind.

Prescription completeness is held as a goal rather than dropped because it is
what the zero-rescue posture is actually reaching for — but four counter-examples
(ISS-250's circular-and-destructive reap remedy, an empty-`detail` `stale-record`,
a bare-branch-name `unprovable-fork`, and ISS-254's two same-text-different-cause
completeness refusals) show the property is not yet held. The mechanical root is
known: 24 refusal-construction sites with a structurally empty `detail`, 14 in
`src/mcp_server/dispatch.rs`. The recurring failure is that a refusal names the
verb its *author* had in mind rather than the one the *operator* needs.
