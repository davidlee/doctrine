---
seq: 0016
scope: spec
target: PRD-005 (Reservation & Leasing) vs SPEC-008 (Id lifecycle)
confidence: med
reversible: yes (proposal only; no authored-tier edit — fence holds)
---
## What
PRD-005 ("Reservation & Leasing") presents a **configurable shared-remote claim
backend as an available capability**, but the realising tech spec (SPEC-008) says
that backend is **unbuilt** — and nothing tracks the gap.

PRD-005's product intent, present-tense / configuration-framed:
- `:19` — "a single working tree today, **every clone of a shared remote when teams
  demand it**".
- `:32-33` — "The reach of a claim as a **backend choice** — a single working tree,
  or every clone of a shared remote — **selected by configuration**, transparent to
  callers."
- `:106` — "**When a shared-reach backend is configured**, identities are unique
  across every clone."
- `:126-127` — "the broader-reach backend is used when its remote is reachable".
- Out-of-scope (`:37-41`) defers only **transient/expiring TTL** claims (edit-exclusion
  heartbeat/release) — it does **not** scope out the shared-remote backend; the PRD
  treats that as in-scope/available.

What SPEC-008 (descends-from target of PRD-005) actually says shipped:
- `:34,55` — allocation is one algorithm: `max(local ∪ trunk) + 1` (**trunk-id
  union**), lock-free, **no shared store** (`:132`).
- `:59-63` — no-trunk repos "degrade to local-only allocation… without cross-fork
  reach[;] the **permanent-claim-over-a-shared-backend generalisation (`git-ref`,
  leasing) is a** [future generalisation]" — i.e. the shared backend is explicitly
  **not built**.
- `:116-118` — current cross-fork guarantee *requires a trunk to union against*;
  there is no configured shared-reach backend today.

So the product spec advertises "leasing" and a "backend choice … selected by
configuration" as if available, while the tech spec is candid that only trunk-union
reservation shipped and the shared backend (the actual *leasing*) is a future
generalisation. A `backlog list` scan for leasing/shared-backend/git-ref finds
**nothing** — the gap between promised and shipped is untracked (the 0009 pattern,
but at PRD level). This is the team-scale half of the capability (multi-clone
coordination is exactly "when teams demand it"), and it is the part that isn't there.

Note: this is a *framing/coverage* gap, not necessarily a defect — trunk-union is a
genuinely clever lock-free mitigation, and the shared backend may be deliberate
roadmap. The issue is that PRD-005 doesn't say so, so a reader (or team evaluating
doctrine for multi-clone use) would believe `configuration` can turn it on today.

## Options
1. **Reconcile PRD-005's framing to shipped reality.** Mark the shared-remote
   backend as a stated future extension (like the TTL claims already are in
   out-of-scope), and describe trunk-union as the current cross-fork mechanism.
   Tradeoff: honest product spec; small authored edit; doesn't build anything.
2. **Capture the shared-backend as tracked work** (don't edit the PRD's ambition,
   give the gap a backlog item so the promise has a path). Tradeoff: keeps PRD-005's
   vision intact and makes the unbuilt half visible/sequenceable; leaves the PRD
   reading as "available" until built.
3. **Both** — reconcile the framing *and* capture the work. Tradeoff: most complete
   (honest now + tracked path); two actions instead of one.
4. **Leave as-is.** Tradeoff: zero effort; PRD-005 keeps implying a configurable
   shared backend that doesn't exist — misleading to exactly the multi-team audience
   the focus targets.

## Recommendation
Option 3: reconcile PRD-005's present-tense "backend choice … selected by
configuration" to a stated future extension (mirroring how TTL claims are already
scoped out), **and** capture the shared-remote/leasing backend as a tracked
improvement citing SPEC-008's named generalisation. Rationale: the product spec
should not over-claim against shipped reality (a trust issue for evaluators), and
the unbuilt half is genuinely valuable team-scale work that deserves a path, not
silent limbo. This mirrors proposal 0009 (designed-but-untracked) and 0013 (stale
spec prose) — same hygiene, PRD tier.

Decisions deferred to YOU:
- (a) **is the shared-remote backend intended roadmap or genuinely out-of-scope?**
  (sets whether it's "future extension" framing vs a real backlog item, or both).
- (b) **does a config knob already exist** for reach selection (PRD says "selected
  by configuration")? If yes, what does it do today with no backend — silently
  degrade? (worth confirming the config surface isn't itself half-wired).
- (c) PRD framing: soften to future tense now, or wait until the backend lands?

## Next doctrine move
```
# confirm the promised-vs-shipped gap (read-only):
doctrine spec show PRD-005           # the "backend choice / when configured" framing
sed -n '55,63p;110,120p' .doctrine/spec/tech/008/spec-008.md   # trunk-union; leasing = future
grep -rn 'reach\|backend\|lease' src/integrity.rs src/entity.rs # is any config knob wired?

# capture the unbuilt half (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "Shared-remote (git-ref) reservation/leasing \
  backend — the PRD-005 cross-clone 'backend choice' beyond trunk-union; SPEC-008 \
  names it the unbuilt permanent-claim-over-shared-backend generalisation" \
  --tag area:coordination --tag area:worktree
# PRD framing reconcile is authored-tier — route it:
/route                               # → small slice / governance quick edit to PRD-005
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — the substance is the promised-vs-shipped gap and its absence from backlog,
not a diff. (PRD framing edits and the backend build are separate, larger moves.)
