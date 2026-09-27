# To revise a verified RV finding, reopen it — do not amend in prose

`verified` is terminal for a review's `done` status, but it is no longer
absorbing. The act table (`can()` in `src/review_ledger/transition.rs`) has a
`reopen` act: raiser, `verified → contested`.

**When a verified decision changes** — the operator pulls a `follow-up` to
fix-now before close, or later evidence undoes an accepted answer:

1. Raiser: `doctrine review reopen RV-NNN --finding F-n --note - <<'EOF' … EOF`.
   The note is required and says what changed and why. It is stored as a
   durable `[[finding.turn]]` row, so the audit-time disposition is not lost:
   the earlier dispose turn keeps its snapshot of disposition, route and response.
2. Responder: `review dispose` the now-`contested` finding with the new
   disposition. (`review amend` covers an `answered` finding the raiser has not
   acted on yet; it needs `--note` too.)
3. Raiser: `review verify`, then `review conclude RV-NNN --basis …` again —
   reopen, like raise, clears the pass's `concluded` marker.

**When prose amendment is still right.** Only when the ledger's raiser is
unavailable to reopen, or to explain legacy history on a ledger that predates
`reopen`. Then add a `## Post-verification amendment` section to the RV `.md`
(what changed, why, where the fix landed), cross-referenced from the
`## Reconciliation Outcome`.

Supersedes `mem.pattern.review.verified-is-terminal-amend-in-prose`, whose
premise ("no verb transitions a finding out of verified") stopped holding when
SL-268 added `reopen`.

Related: [[mem.pattern.dispatch.admit-fix-on-top-not-supersede]],
[[mem.pattern.review.done-requires-concluded]].
