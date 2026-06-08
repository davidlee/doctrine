---
name: audit
description: Use after a slice's phases are implemented, when the task is now evidence, conformance, and reconciliation against the design — disposition every finding and harvest durable risks into audit.md before closure.
---

# Audit

You are running the reconciliation loop: does the work match its design and
governance, and is every gap consciously dispositioned before close?

> **Tooling gap.** Doctrine has no audit scaffold yet — `audit.md` is
> hand-authored, a sibling of `design.md` under the slice folder. There is also
> no specs/contracts registry or `sync`/`validate` surface; reconciliation here
> means reconciling against `design.md`, ADRs, and `doc/*`, not a spec engine.

Inputs:

- the slice's implemented phases and their verification evidence
- `design.md` (canonical), `slice-nnn.md`, `plan.toml`
- relevant ADRs and `doc/*` specs (see `/canon`)

## Process

1. Determine audit mode:
   - **conformance** — post-implementation audit tied to a slice (the usual case)
   - **discovery** — backfill or existing-code investigation
2. Author or update `audit.md` in the slice folder. Treat loose notes as
   insufficient for closure-grade work — the findings belong in `audit.md`.
3. Gather evidence:
   - run the tests/checks the design and plan require, plus `just check`
   - inspect observed behaviour against `design.md` and the phase `VT-` criteria
   - note where behaviour and design diverge
4. Record every finding in `audit.md` with: what was expected (cite design / ADR
   / criterion), what was observed, the evidence, and a disposition.
5. Disposition every finding explicitly — do not leave closure-grade findings
   undispositioned:
   - **aligned** — observed behaviour is already correct; no follow-up.
   - **fix now** — reconcile inside this slice before closing.
   - **design was wrong** — the design, not the code, is the defect; reconcile
     `design.md` (and the slice scope) so canon tells the truth.
   - **follow-up slice** — owned future work is the correct route; capture it.
   - **tolerated drift** — explicit unresolved drift, with rationale, only when
     the tradeoff is consciously accepted.
6. Resist easy escapes: do not pick "follow-up slice" just because the fix feels
   large, and do not normalise "tolerated drift" without a real rationale. If the
   correct route is ambiguous after reading `design.md` and governance, stop and
   `/consult`.
7. Harvest durable risks, decisions, and gotchas from the disposable phase sheets
   into `audit.md` / `notes.md`, and promote reusable facts via `/record-memory`.
   Capture durable follow-up **work** the audit surfaced — risks, issues, chores —
   as backlog items with `backlog new`, alongside that harvest (the
   work / knowledge / decision boundary: `using-doctrine.md`).
8. Hand off to `/close` only after `audit.md`, `design.md`, and any follow-up
   refs tell a coherent closure story — not merely when the tests pass.

## Outcomes

- Audit evidence is recorded in `audit.md`.
- Every finding ends with an explicit disposition.
- Design and governance are reconciled before closure handoff.
- `/close` receives work that is actually audit-ready.
