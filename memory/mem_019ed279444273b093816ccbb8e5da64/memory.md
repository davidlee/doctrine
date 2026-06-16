# Doctrine review ledger (RV kind)

The review ledger is a **first-class entity** for structured adversarial
review (ADR-007). It is turn-based: two parties alternate between raising and
resolving findings, with an explicit baton that tracks whose turn it is.

The RV kind is the substrate for audit reconciliation, design review,
code review, and any structured finding-tracked dialogue.

## Lifecycle

1. `doctrine review new --facet <facet> --target <ENTITY>` — open a ledger
   targeting an entity (slice, spec, ADR, etc.). The target is validated
   before any id is allocated.
2. `doctrine review raise <ID>` — raise a finding (severity + title + detail).
   The finding is `open`; the baton flips to the responder.
3. `doctrine review dispose <ID>` — answer an `open`/`contested` finding with
   a disposition and response. The finding is `answered`; the baton returns to
   the raiser.
4. `doctrine review verify <ID>` — accept the disposition (terminal:
   `verified`).
5. `doctrine review contest <ID>` — reject the disposition and hand it back
   (status returns to `contested`; baton flips again).
6. `doctrine review withdraw <ID>` — retract an `open`/`answered` finding
   (terminal: `withdrawn`).

Every finding carries a severity (`blocker | major | minor | cosmetic`) and
an owner-owned status.

## Coordination

- `doctrine review status <ID>` — report the derived state and rebuild the
  baton (cache recompute).
- `doctrine review prime` — populate the reviewer context warm-cache from a
  curated `domain_map`; `--seed` emits git-changed candidate paths to curate.
- `doctrine review unlock <ID>` — remove a stale per-review lock left by a
  hard kill (escape hatch).

## Viewing

- `doctrine review show <ID>` — derived status, the `reviews` edge, and the
  brief.
- `doctrine review list` — id, derived status (+ await), facet, target, title.

## Where it fits

The RV kind is driven by the audit phase (`/audit`), the design review gate
(`/inquisition`), and the code review skill (`/code-review`). Closeout expects
that any RV targeting the slice has no unresolved blockers.

See [[signpost.doctrine.audit]] for the audit phase that uses the RV ledger,
[[signpost.doctrine.lifecycle-start]] for the full lifecycle,
[[signpost.doctrine.file-map]] for the `.doctrine/review/nnn/` layout, and
[[concept.doctrine.reading-entities]] for the read-via-show rule.
