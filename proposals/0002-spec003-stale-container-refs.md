---
seq: 0002
scope: spec
target: SPEC-003
confidence: med
reversible: yes (proposal only; no spec edit performed — authored-tier fence)
---
## What
SPEC-003 ("Doctrine", the whole-system C4 **context** spec) enumerates its 11
containers at `.doctrine/spec/tech/003/spec-003.md:19-39`, but cross-references
the owning tech spec for only **3 of 11**:

- cited: Entity engine **(SPEC-004)** `:19`, Priority engine **(SPEC-001)** `:34`,
  Reconciliation **(SPEC-002)** `:36`.
- uncited but a tech spec now exists (title matches the container name 1:1):
  - Spec composition `:21` → **SPEC-006** "Spec composition machinery"
  - Memory `:23` → **SPEC-007** "Memory engine"
  - Id lifecycle `:25` → **SPEC-008** "Id lifecycle"
  - Install & distribution `:26` → **SPEC-009** "Install & distribution"
  - Skills distribution `:28` → **SPEC-010** "Skills distribution"
  - Boot snapshot `:30` → **SPEC-011** "Boot snapshot"
  - uncited and **no tech spec exists**: Dispatch & worktree `:32`, CLI surface `:38`.

The three cited specs are the lowest-numbered (001/002/004) — authored before the
others. The pattern is staleness: the root context spec was written when only those
three container specs existed and was never back-filled as SPEC-006…011 landed. The
spec's own stated job is to "name the containers … and how they fit … this root
defers to them for the *how*" (`:11-15`) — deferral without a citation is a dead
pointer. A reader navigating top-down from the root cannot reach 6 of the existing
container specs.

Note: derived reciprocity (ADR-004, outbound-only relations) means the *child*
specs point up via `parent: SPEC-003` (confirmed: SPEC-008 shows `parent: SPEC-003`),
so the structural edge exists. This is purely a **prose navigability** gap, not a
relation-graph gap.

## Options
1. **Back-fill the 6 existing refs only.** Add `(SPEC-006)`…`(SPEC-011)` inline at
   `:21-30`, matching the existing `(SPEC-004)` style. Leave Dispatch & worktree and
   CLI surface bare (no spec to cite). Tradeoff: smallest, purely corrective edit;
   closes the dead-pointer defect, defers the coverage question.
2. **Back-fill + flag the 2 gaps.** Same as (1), plus a one-line note that Dispatch
   & worktree and CLI surface have no tech spec yet (coverage gap → candidate
   backlog items). Tradeoff: surfaces a real architecture-coverage question, but
   mixes a corrective doc edit with a scoping decision.
3. **Do nothing — rely on the spec tree.** `spec show` and the parent edges already
   let a reader walk down. Tradeoff: leaves the root spec actively misleading (cites
   3, implying the other 8 have no spec), and the asymmetry reads as intentional when
   it is not.

## Recommendation
Option 1 now, as a self-contained doc correction; Option 2's gap-flag split into
backlog rather than folded into the spec edit. Rationale: the 6 back-fills are
unambiguous and reversible (the spec titles match the container names exactly), and
they restore the root spec's navigational contract. The two genuinely-missing specs
(Dispatch & worktree, CLI surface) are a separate, larger question — whether those
containers warrant their own tech spec — and should not ride a citation fix.

Decision deferred to YOU: (a) whether the root context spec *should* carry downward
citations at all, or deliberately relies on derived child→parent edges (if the
latter, then SPEC-004/001/002's refs are the anomaly and should arguably be
*removed* for consistency — the inverse of this proposal); (b) whether Dispatch &
worktree and CLI surface are tech-spec coverage gaps worth a backlog item.

## Next doctrine move
```
# inspect both tiers before editing:
doctrine spec show SPEC-003

# corrective edit is an authored-tier change to spec-003.md — route it:
/route            # → likely a small slice or, per project governance, a
                  #   "small backlog item" quick-design edit (boot.md Governance).
# if pursuing the coverage-gap question (option 2 / deferred-b):
doctrine backlog new improvement "Tech-spec coverage: Dispatch & worktree and CLI \
  surface containers (SPEC-003) have no owning tech spec" --tag area:spec
```
(Verbs described, NOT executed — the fence forbids me editing authored spec state
or transitioning backlog.)

## Illustration (optional) — ILLUSTRATIVE, not applied
Hand-authored (no worker), shows the option-1 shape only:
```diff
--- a/.doctrine/spec/tech/003/spec-003.md
+++ b/.doctrine/spec/tech/003/spec-003.md
@@ -19,13 +19,13 @@ The system decomposes into these containers:
 - **Entity engine** (SPEC-004) — the kind-agnostic scaffolding and identity
   substrate every authored kind is materialised through.
-- **Spec composition** — the spec family, its requirement peers, membership edges,
+- **Spec composition** (SPEC-006) — the spec family, its requirement peers, membership edges,
   reassembly, and corpus validation.
-- **Memory** — the scope-aware durable-knowledge store, recorded and retrieved
+- **Memory** (SPEC-007) — the scope-aware durable-knowledge store, recorded and retrieved
   out-of-band of any one task.
-- **Id lifecycle** — next-id allocation, corpus-wide integrity, and reseat repair.
+- **Id lifecycle** (SPEC-008) — next-id allocation, corpus-wide integrity, and reseat repair.
-- **Install & distribution** — the embedded sources, manifest, and templates the
+- **Install & distribution** (SPEC-009) — the embedded sources, manifest, and templates the
   installer lays into a target repo.
-- **Skills distribution** — the routing skills shipped from `plugins/` into the
+- **Skills distribution** (SPEC-010) — the routing skills shipped from `plugins/` into the
   installed skill tree.
-- **Boot snapshot** — the cache-friendly governance projection assembled for
+- **Boot snapshot** (SPEC-011) — the cache-friendly governance projection assembled for
   session start.
 - **Dispatch & worktree** — the isolation and orchestrator-sole-writer machinery
   for concurrent work.
```
