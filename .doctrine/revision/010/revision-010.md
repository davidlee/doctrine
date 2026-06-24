# REV REV-010 — reconcile SL-148

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile narrative for **SL-148** (git-ref reservation backend), driven by the
**RV-152** audit reconciliation brief. SL-148 ships the previously-deferred remote
reservation reach; four authored governance/spec statements now lag the as-built
truth and are amended here. All four rows are `modify` (surfaced-for-manual at
apply). Each cites the RV-152 finding that drove it.

### [RV-152 R7] SPEC-008 §"Trunk-aware fork safety" — modify

The section closes by naming the `git-ref` / shared-backend generalisation a
"specified-but-deferred extension." SL-148 **ships** it. Append a note recording
the now-shipped remote reservation primitive: the ref class
`refs/doctrine/reservation/<prefix>/<NNN>` (empty-tree commit, content-free per
REQ-024) and the three new remote git ops in `git.rs` (`fetch_refspec`,
`push_ref_cas`, `for_each_ref`); reach is config-selected (`local | shared | auto`)
and the trunk-union local path remains the degraded reach.

**Before:** "… is a specified-but-deferred extension of the same algorithm."
**After:** adds a following paragraph — "**Shipped (SL-148).** The deferred
extension now exists: a `GitRef` claim backend reserves an id by creating
`refs/doctrine/reservation/<prefix>/<NNN>` (an empty-tree, content-free commit)
at a shared remote under a zero-oid create-CAS, linearizing the claim across every
clone. Three remote ops in `git.rs` (`fetch_refspec`, `push_ref_cas`,
`for_each_ref`) back it; reach is selected by `[reservation] reach = local | shared
| auto`, and the trunk-union local scan remains the degraded single-tree reach."

### [RV-152 R7/F-4] SPEC-022 §"ref taxonomy" — modify

The taxonomy enumerates two local classes (mutable refs; immutable evidence refs)
and scopes coordination/evidence refs as local. SL-148 adds a **remote, permanent**
ref class. Add a note so the taxonomy is not silently widened.

**Before:** the "Immutable evidence refs (R2)" block ending "…the SL-067 trap the
candidate layer was built to close."
**After:** adds a following paragraph — "**Permanent reservation refs (SL-148).**
A third class, distinct from both above: `refs/doctrine/reservation/<prefix>/<NNN>`
is created exactly once under a **zero-oid create-CAS** like an evidence ref, but is
**pushed to a shared remote** and is **permanent** (never deleted, never reissued;
an abandoned reservation is a harmless gap). This is the model's first *remote* ref
mutation — PRD-005 / SPEC-008 ratify the reach; SPEC-022's local-only framing of
coordination refs is widened accordingly."

### [RV-152 F-3/D8] PRD-005 §6 "Reach selection" — modify

§6 states `auto` "falls back to single-tree reach with a one-time signal" when the
remote is unreachable. SL-148's D8 **tightens** this: the literal fall-back governs
only the **structurally single-tree** (no remote configured) case; a *configured*
remote that fails is a **hard error** by default (a silent transient downgrade would
mint a colliding local id), the operator opting into local fallback explicitly.

**Before:** "… otherwise allocation falls back to single-tree reach with a one-time
signal that cross-team reach is off."
**After:** "… otherwise allocation falls back to single-tree reach with a one-time
signal that cross-team reach is off. This automatic fall-back governs the
structurally single-tree case (no remote configured); a *configured* remote that
fails is treated as a hard error rather than silently downgraded — the operator
opts into reduced-reach local allocation explicitly — so a transient failure can
never silently mint an id that collides with another clone's accepted reservation."

### [RV-152 F-6] ADR-001 `layering.toml` — modify (cosmetic)

The `reserve = "engine"` inline comment "→ entity only (out=1)" is stale: `reserve`
imports `entity`/`git`/`dtoml` (out=3) since PHASE-03. Comment-only correction; the
classification and the enforced layering test are unchanged.

**Before:** `reserve = "engine"          # SL-148: claim-backend selection seam; → entity only (out=1)`
**After:** `reserve = "engine"          # SL-148: claim-backend selection seam; → entity/git/dtoml (out=3)`
