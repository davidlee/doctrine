# CHR-023: ADR-005 full compliance: reference-doc IA, user hooks, restate-line audit

## Context

ADR-005 (shipped knowledge tiering) was accepted on 2026-06-08 and its
open questions were resolved by inquisition. Several deliverables were left
as follow-up: the PULL-tier CLI/editing reference doc, the restate-line
enforcement on skills, and the reference-forms block in boot. Initial scoping
(ADR-005 R-C1, R-C3) set an evidence-bound bar.

Since then, the codebase has grown new entity kinds (REC, REV, POL, STD,
knowledge) and new CLI surfaces — the reference docs and templates have not
kept pace. The user-facing `.md` hooks (governance.md, boot-footer.md) exist
but their IA and discoverability are unexamined.

This slice picks up the full ADR-005 compliance work that was carved out of
CHR-021 (SL-143).

## Scope

### In scope

- **Information architecture audit** of `install/*.md` — routing-process.md,
  using-doctrine.md, glossary.md, governance.md, boot-footer.md,
  review-ledger.md. Are they coherently organised? Do they overlap, conflict,
  or leave gaps? Does the IA support an agent finding what it needs?
- **User-serviceable `.md` hooks** — review and document the customisation
  surface: `governance.md` (boot-injected), `boot-footer.md` (boot-injected),
  and a new hook for project-custom reconciliation rules (consumed by the
  reconcile/close loop). Ensure each hook is discoverable, documented, and
  has a clear contract.
- **Restate-line audit** (ADR-005 R-OQ-4) — scan every skill for violations
  of the rule: skills MUST NOT reproduce flag syntax, option/enum tables,
  or storage-tier mechanics as prose. They MAY name a verb and cite a rule
  by name.
- **Reference-doc currency** — update `glossary.md` and `using-doctrine.md`
  to cover all current entity kinds (REC, REV, POL, STD, knowledge records)
  and CLI verbs. Ensure they are pointed-at by the boot digest and relevant
  skills.
- **Reachability check** — verify every shipped reference doc is reachable
  from at least one skill or the boot digest. Fix orphans.
- **PUSH-tier completeness** — verify the reference-forms block in
  routing-process.md is present and correct per ADR-005 R-OQ-5.

### Out of scope

- **Shipped memory bodies.** The 30 shipped memories (signposts, concepts,
  patterns, facts) are handled by SL-143.
- **Architecture changes to the memory engine.**
- **New entity kinds or CLI verbs.** This slice documents what exists; it does
  not create new kinds or commands.

## Relationships

- `governed_by` ADR-005
- `related` SL-143 (carved out from)

## Risks & Assumptions

- The restate-line audit may produce many findings if skills drifted before
  ADR-005 was ratified. Scope as evidence-bound (ADR-005 R-C1): fix named
  file:line offenders, no blanket sweep.
- New reconcile hook must not break existing reconcile/close flow.
  Design for backward compatibility.

## Verification / Closure Intent

- Every `install/*.md` doc audited for IA coherence; gaps documented and
  resolved.
- User hooks documented with clear contracts (what each hook controls, how
  it's injected, caveats).
- All skills comply with the restate line (or exceptions are documented
  in the ADR-005 review ledger).
- Glossary and using-doctrine.md cover all current entity kinds and verbs.
- Every reference doc is reachable from at least one skill or boot digest.
