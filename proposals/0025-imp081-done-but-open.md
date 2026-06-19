---
seq: 0025
scope: backlog
target: IMP-081 (memory record --trust/--severity)
confidence: high
reversible: yes (proposal only; no backlog transition performed — yours to run)
---
## What
**IMP-081 ("memory record CLI lacks `--trust` and `--severity` flags", audit A-3)
is done but still `open`.** The flags it asks for now exist:
`doctrine memory record --help` lists `--trust <TRUST>` ("Trust level carried in
`[trust].trust_level`") and `--severity <SEVERITY>` ("Severity carried in
`[ranking].severity`") — exactly the two fields IMP-081's title names. The
implementation landed; the backlog item was never transitioned.

A done-but-open item is not free: it pollutes the actionability surfaces the project
ships — it shows up in `survey`/`next` as eligible work, inflates the open count, and
a human (or an agent routing to `next`) can pick up something already finished.
Doctrine's whole value proposition is a trustworthy work graph; a resolved item
sitting `open` is exactly the drift the corpus otherwise avoids (FK/orphan clean,
etc.).

The "audit A-3" tag suggests IMP-081 came from a structured audit (A-1, A-2, …).
Worth a quick spot-check that its siblings aren't *also* done-but-open — a single
landed change often satisfies several audit line-items at once.

## Options
1. **Verify then transition IMP-081 to resolved/done.** Confirm the flags fully
   satisfy the item (both fields, correct toml targets — they appear to), then close
   it with a pointer to the implementing slice/commit. Tradeoff: the correct, simple
   hygiene fix; the only caveat is confirming no sub-requirement of IMP-081 (beyond
   the two flags) is unmet.
2. **Transition + sweep the audit-A siblings.** Do (1) and check other `audit A-N`
   / same-vintage items for the same done-but-open state in one pass. Tradeoff: more
   thorough (a landed change often clears several audit findings); slightly more
   work; likely catches more drift.
3. **Leave open.** Tradeoff: zero effort; but `next`/`survey` keep advertising
   finished work, and the backlog's credibility erodes one stale item at a time.

## Recommendation
Option 2: transition IMP-081 (done), and while there, sweep the `audit A-*` /
2026-06-16-vintage backlog for siblings in the same state. Rationale: a stale-open
item is a direct hit on the actionability graph's trustworthiness — the cheapest
high-value grooming there is — and audit-batch items tend to land together, so the
sweep likely finds more than one. Verify-before-close (confirm the flags meet the
item's full intent) is the only gate.

Decisions deferred to YOU:
- (a) confirm the `--trust`/`--severity` flags **fully** satisfy IMP-081 (vs a
  partial — e.g. does it also require validation/defaulting the item implied?).
- (b) **transition target** — `resolved/fixed` vs `done` (per your backlog
  lifecycle vocabulary), and whether to cite the implementing change.
- (c) **sweep the audit-A siblings** now, or just close IMP-081.

## Next doctrine move
```
# verify done + find siblings (read-only):
doctrine memory record --help | grep -E 'trust|severity'   # both present
doctrine backlog show IMP-081
doctrine backlog list | grep -iE 'audit'                    # sibling audit items?

# transition (NOT executed — fence forbids backlog transition):
doctrine backlog <resolve/close verb> IMP-081   # verb per `doctrine backlog --help`
```
(Verbs described, NOT executed — fence forbids backlog state transitions.)

## Illustration (optional)
None — a status-hygiene transition, not a diff. Evidence is the `--help` output
matching the item title 1:1.
