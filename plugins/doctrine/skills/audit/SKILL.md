---
name: audit
description: Use after a slice's phases are implemented, when the task is now evidence, conformance, and reconciliation against the design — disposition every finding on a reconciliation review ledger (the RV kind) before closure.
---

# Audit

You are running the reconciliation loop: does the work match its design and
governance, and is every gap consciously dispositioned before close?

The audit stage is now a **review ledger** — the RV kind (`RV-NNN`, ADR-007).
You open a `reconciliation`-facet review targeting the slice, raise each finding
as a structured ledger entry, and dispose/verify it through the turn graph. The
ledger replaces the old hand-made `audit.md`: findings are append-only and
field-owned, "no undispositioned findings before close" is enforced by the binary
(the close-gate teeth), and the audit prose becomes the review's `## Synthesis`.

> **Self-audit (the usual case).** When you are both reviewer and author, drive
> both roles with `--as <role>` — the raiser raises/verifies/withdraws, the
> responder disposes. The per-review lock and the per-finding `can()` gate keep a
> one- or two-party audit correct; `--as` is cooperative role assertion, not a
> security boundary (ADR-007).

> **`audit.md` is retired for new audits.** Existing `audit.md` files remain valid
> — there is no migration. Do not author a new one; open an RV instead.

> **Reconciliation scope.** Doctrine has no specs/contracts registry or
> `sync`/`validate` surface; reconciliation here means reconciling against
> `design.md`, ADRs, and `doc/*`, not a spec engine.

Inputs:

- the slice's implemented phases and their verification evidence
- `design.md` (canonical), `slice-nnn.md`, `plan.toml`
- relevant ADRs and `doc/*` specs (see `/canon`)

## Process

1. Determine audit mode:
   - **conformance** — post-implementation audit tied to a slice (the usual case)
   - **discovery** — backfill or existing-code investigation
2. Open the review ledger for the slice (replaces authoring `audit.md`):
   - `doctrine review new --facet reconciliation --target SL-NNN`
   - then warm the reviewer context: `doctrine review prime RV-NNN --seed`
     emits git-changed candidate paths (a starting point, not authority); curate
     them into a `domain_map` and `doctrine review prime RV-NNN` (stdin or
     `--from <file>`) to persist the curated areas/invariants/risks. `review
     status RV-NNN` then reports `cache: current`/`stale` as an optimization
     signal — never a gate.
   - Fill the ledger's `## Brief` (in `review-NNN.md`) with the lines of attack:
     what this audit is probing and the invariants it holds the slice to.
   - Treat loose notes as insufficient for closure-grade work — the findings
     belong in the RV ledger, not in conversation context.
3. Gather evidence:
   - run the tests/checks the design and plan require, plus `just check`
   - inspect observed behaviour against `design.md` and the phase `VT-` criteria
   - note where behaviour and design diverge
4. Raise every finding as a ledger entry — `doctrine review raise RV-NNN
   --severity <S> --title <what was expected vs observed> --detail <the
   evidence>`. The raiser owns `severity`/`title`/`detail`, fixed at raise
   (append-only). Severity is `blocker | major | minor | nit`:
   - **`blocker`** is the only severity that gates `/close` — an unresolved
     blocker on an active RV targeting this slice refuses the `audit→reconcile`
     and `reconcile→done` transitions (the close-gate teeth, enforced in the
     binary; D-C9b). Reserve it for findings that must not ship unreconciled.
   - `major`/`minor`/`nit` record the finding but never block close.
5. Disposition every finding explicitly via `doctrine review dispose RV-NNN
   --finding F-n --disposition <vocab> --response <rationale> --as responder`,
   then close it with `verify` (accept) or `contest` (hand back) — do not leave
   closure-grade findings undispositioned. The recommended `disposition`
   vocabulary (free-text, but use these consistently):
   - **aligned** — observed behaviour is already correct; no follow-up.
   - **fix-now** — reconcile inside this slice before closing.
   - **design-wrong** — the design, not the code, is the defect; reconcile
     `design.md` (and the slice scope) so canon tells the truth.
   - **follow-up** — owned future work is the correct route; capture it
     (`backlog new`).
   - **tolerated** — explicit unresolved drift, with rationale, only when the
     tradeoff is consciously accepted.

   Then accept it: `doctrine review verify RV-NNN --finding F-n --as raiser`
   (terminal). A finding **raised in error** is retracted with `doctrine review
   withdraw RV-NNN --finding F-n --as raiser` (terminal) — not disposed. A finding
   you disagree with after disposition goes back via `review contest` (answered →
   contested) for re-disposition. `--note` on verify/contest is ephemeral handoff
   chatter for the baton log, NOT durable rationale — durable justification
   belongs in the finding's `response` or a new finding (D10).
6. Resist easy escapes: do not pick **follow-up** just because the fix feels
   large, and do not normalise **tolerated** without a real rationale. Do not
   downgrade a true `blocker` to dodge the close-gate. If the correct route is
   ambiguous after reading `design.md` and governance, stop and `/consult`.
7. Write the audit's reasoning as the review's `## Synthesis` (append it to
   `review-NNN.md`) — the prose that the old `audit.md` carried: the closure
   story, the standing risks, the tradeoffs consciously accepted. Then harvest
   durable risks, decisions, and gotchas from the disposable phase sheets into
   `notes.md`, and promote reusable facts via `/record-memory`. Capture durable
   follow-up **work** the audit surfaced — risks, issues, chores — as backlog
   items with `backlog new`, alongside that harvest (the work / knowledge /
   decision boundary: `using-doctrine.md`).
8. Hand off to `/close` only when the ledger is resolved and the story is
   coherent — not merely when the tests pass. The review is **done** when every
   finding is terminal (`verified` or `withdrawn`, D-C9a); `review status RV-NNN`
   reports `done · await=none`. An unresolved `blocker` will be refused at the
   close seam, so resolve it (via `verify` or `withdraw`) before handing off.

## Outcomes

- Audit evidence is a structured RV ledger (`review-NNN.toml` + the review's
  `## Synthesis`), not a hand-made `audit.md`.
- Every finding ends terminal with an explicit disposition (or is withdrawn).
- No unresolved `blocker` remains — the close-gate would refuse it.
- Design and governance are reconciled before closure handoff.
- `/close` receives work that is actually audit-ready.
