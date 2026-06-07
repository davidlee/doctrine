# Backlog work-intake membership test

The `backlog_item` entity (PRD-009, SL-020) admits a kind **only** if it passes
the work-intake membership test:

> A backlog item is a latent unit of work intent that can be triaged,
> prioritised, and promoted into a slice, and whose standing fits the work-status
> lifecycle (`open|triaged|started|resolved|closed`).
>
> **If a candidate does not fit the work-intake lifecycle, it is not a backlog
> item.**

This is a normative invariant, not advice. It governs every future entity-kind
decision, not just backlog.

**Admitted kinds (the five glossary-reserved):** issue, improvement, chore, risk,
idea. `risk` is admitted because it is *unresolved work-risk* — uncertain future
harm that may need mitigation/acceptance/expiry/promotion — not as a general
epistemic record. `problem` is excluded until it earns a reserved id + a
decomposition boundary distinguishing it from `issue`.

**Excluded (fail the test — epistemic/governance, different lifecycles):**
assumptions (held→validated), decisions (proposed→accepted→superseded — and the
decision/governance family already owns these; ADR ships), questions, findings,
tradeoffs, constraint statements. Putting any of these in the backlog would break
the uniform work-status lifecycle and duplicate an existing family. Their home is
a risk facet, the decision/governance family, or a future epistemic entity group
(PRD-009 OQ-005).

**Two further rules from the same model:**
- Each kind carries a discriminating boundary; when several fit, precedence
  resolves deterministically: `risk > issue > improvement > chore > idea`.
- `status` (whether active) is orthogonal to `resolution` (why it stopped);
  no close-reason — accepted, expired, duplicate, promoted, wont_do — is ever
  encoded as a `status` state.

Why: the backlog is the capture layer for *work intent*. Conflating it with
truth-state (assumptions) or governance-state (decisions) is the drift this rule
prevents — "backlog is not the home for every unresolved thing; it is the home
for unresolved work intent."
