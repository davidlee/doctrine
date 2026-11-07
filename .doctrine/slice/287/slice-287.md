# Commitment ledger for plan authoring and review

## Context

Plans for complex designs lose things in translation. Two defect classes
dominate, measured on `SL-251`'s plan review (recorded in `RFC-027` § G4):

- **unclaimed** — a design commitment no `EX`/`VT` criterion claims (2 of 9);
- **cited-but-wrong** — the criterion names the design pin, then restates it
  incorrectly (5 of 9). A pointer-only cross-check scores these as covered.

The user has used an informal remedy in other doctrine repos, effectively:

1. extract a ledger of the commitments the design sets out, explicit or
   implicit;
2. author the plan, recording in the ledger where and how each commitment is
   satisfied;
3. cross-check the verification plan and phase entry criteria against the same
   ledger, for coverage and proportionality.

and a two-reviewer plan review: one reviewer runs the ledger cross-check
explicitly; the other checks sequencing and phase gates. Reported as very
effective at finding and correcting omissions.

Adjacent deliberations bound the shape:

- `RFC-027` `H9` rejected a plan-level obligation *dependency graph*. This is a
  coverage cross-check, not a dependency graph; it adds no stored structure.
- `RFC-029` owns the `VT`→`EX` proof seam (`IMP-409`, `SPEC-031` D6). Step 3
  rides that seam informally until it is built.
- `SL-258` (commitment extraction/admission, in design) owns the mechanised
  design→`EX` seam. This slice is the cheap, unmechanised precursor; its outcome
  is evidence for `SL-258`'s design.

## Scope & Objectives

Codify the pattern as shipped workflow guidance — prose, no engine change:

1. **Plan authoring.** `/plan` gains a procedure, triggered for complex designs
   (trigger to be defined in design), that builds the commitment ledger, records
   satisfaction sites while authoring, and cross-checks verification and entry
   criteria against it.
2. **Ledger entries quote or anchor the design text**, and the cross-check
   compares content, not citations — the remedy for the cited-but-wrong class.
3. **Proportionality, not counts.** The cross-check judges whether each
   commitment's verification is proportionate; it does not count tests.
4. **Plan review shape.** Codify the two-reviewer plan review (ledger
   cross-check; sequencing and phase gates) on the existing RV ledger
   machinery.

## Non-Goals

- No new entity kind, schema field, CLI verb or check. (`RFC-029`'s warning: a
  new record here should expect to fail the ownership test.)
- Not building `IMP-409` (`VT`→`EX` link) or any part of `SL-258`.
- No test-count tracking.
- Not "obligation": that word is the runbook primitive (`DEC-101`); use
  "commitment", matching `SL-258` (`STD-002`).

## Affected surface

- `plugins/doctrine/skills/plan/SKILL.md` — the `/plan` skill master.
- Probably a new shipped reference doc for the procedure body (`ADR-005`:
  skills route, reference docs explain) — home decided in design.
- Wherever plan review is currently described (design to locate:
  `install/review-ledger.md`, `/code-review`, or `/plan`).

Shipped text must not cite repo-private ids (`POL-002`, `ADR-024`).

## Risks, assumptions, open questions

- **Assumption:** the informal results transfer to agents following written
  guidance, including cheaper models. The follow-up trial tests this.
- **Risk:** extraction becomes the new paraphrase site. Quoting/anchoring is the
  mitigation; the follow-up trial should show whether cited-but-wrong shrinks.
- **Risk:** ceremony on simple plans. Mitigation: an explicit trigger; skip
  otherwise.
- **OQ:** where the ledger artefact lives (slice-folder markdown, plan notes,
  or the RV ledger itself), and whether it is authored or runtime.
- **OQ:** trigger for "complex design".
- **Adjacent, not in scope:** `/plan` step 4 says `specs`/`requirements` stay
  empty "no registry yet" — stale per `RFC-029` § 5 (`ISS-321`).

## Verification / closure intent

- Shipped guidance reads coherently, validates (`doctrine check gate`), and
  carries no repo-private ids.
- The trial is **not** a closure condition (see Follow-Ups).

## Summary

## Follow-Ups

- **Trial** (deferred out of scope by the user, 2026-11-08; tracked in
  `RFC-029`): run the procedure on the next complex slice, sort its plan-review
  findings by the `SL-251` classes (unclaimed / cited-but-wrong / needs a tree
  check), and record the result as evidence for `SL-258`'s design.
