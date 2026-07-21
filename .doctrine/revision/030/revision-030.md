# REV REV-030 — ADR-012 D4 operator-ingested candidate merge

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

### Decision (bottom line)

> Doctrine may **perform or ingest** a 3-way merge while *constructing* a
> candidate. It **never composes** a merge while *publishing* an admitted
> candidate to trunk. Integration stays publication-only: expected-tip CAS +
> fast-forward of the admitted OID. A non-FF/moved trunk is resolved by a
> superseding candidate on the new base — never auto-merged at integrate.

This REV makes the narrowest governance change that unblocks SL-212 (IMP-127,
hand-resolved candidate-conflict ingest): it extends ADR-012 D4's existing
candidate-merge exception to admit an **operator-produced** `merge_oid`, under
the *identical* provenance contract already in force. It **reaffirms** D4's
FF-only publication posture unchanged. It is **not** a reversal of D2/D4, and
therefore does **not** enact the non-FF-auto-merge-at-integrate capability that
RFC-006 deliberated.

### The mistaken dependency this dissolves

SL-212's scope (and the `research.local.md` §1) carried a gate:
`IMP-127 → non-FF trunk at integrate → RFC-006 (reverse ADR-012 D2/D4 FF-only)`,
requiring an ADR-012 *reversal* before SL-212 could be designed. That arrow is
**false**: the mechanism already *permits* an FF-only route the dependency
claimed was impossible. A hand-resolved merge whose parents are the recorded
`(base_oid, source_oid)` is a descendant of `base_oid`; when the candidate `base`
is chosen as the **current trunk tip**, the resulting `merge_oid`
**fast-forwards trunk** exactly like a Doctrine-produced clean candidate merge.
This is a graph fact, conditional on base selection — the code does not *enforce*
`base == the eventual --trunk tip` (base is resolved once at create; trunk
independently at integrate, `dispatch.rs:1338`/`:2617`). If they diverge,
integrate safely **refuses** (FF-check), never force-lands. So the reversal of
D2/D4 is not *required*: the FF-only route is available, and the non-FF case
degrades to a refusal, not a corruption.

Two problems were conflated into one decision and must be separated:

1. **A candidate merge conflicts and needs human resolution** (SL-212 / IMP-127)
   — a *candidate-construction* gap. Fixed here.
2. **Trunk moves after a candidate is admitted** (the shared-trunk-race, RFC-006's
   motivation) — a *publication-concurrency* concern. Left as-is: supersede on
   the new base (ordinary optimistic concurrency), not a merge-resolution deficiency.

### Code evidence (the seam already supports this)

Verified against `src/dispatch.rs` (v0.25.4):

- `candidate create --base` is **caller-supplied**, not pinned to a historical
  fork-point (CLI arg `dispatch.rs:287` → `CreateRequest.base` `:418` →
  `resolve_commit(&req.base)` `:1341`). Passing `--base refs/heads/main` (current
  trunk) is already legal. The pinned fork-point at `:1998` is `prepare-review`'s
  *bundle projection* leg — a different concern, not the candidate base.
- `candidate admit` **already validates precisely the provenance contract** an
  ingested merge needs (`:1620-1634`): `parents(merge_oid) == {base_oid, source_oid}`
  (an order-independent `BTreeSet`) plus `merge_oid` is-ancestor-of the admitted
  tip. A hand-resolved `git merge source` commit has exactly those two parents and
  passes admit **unchanged**.
- `integrate` is **FF-only** on the admitted `merge_oid` (`:2622`), and already
  prescribes the residual recovery: a non-FF planned OID → "re-anchor to a new
  base and re-admit … not auto-resolved" (`:2609`, `:2624`), with `--supersedes`
  the mechanism (`:294`).

The only thing standing between the current code and SL-212 is a **verb** to
record an operator's resolved commit as the `merge_oid` of a `Conflicted` row
(`:1409`, `merge_oid=""`).

**Validation is NOT parent-binding alone.** Admit today checks only
`parents == {base_oid, source_oid}` (an order-independent set) and ancestry
(`:1620-1634`) — it never inspects the *tree*. That is sufficient for a
Doctrine-produced merge (Doctrine authored the tree) but **not** for an
operator-produced one: a commit with the right two parents but an arbitrary tree
(unrelated source deleted or rewritten) passes admit and, on a clean FF, reaches
trunk. This directly contradicts SL-212's own safety objective ("a true 3-way
merge … **not an arbitrary tree**"). This REV therefore does **not** claim
parent-binding is a sufficient contract for ingest. It governs the **boundary**
and defers the **mechanism** to SL-212 design:

- **"True 3-way" ≙ non-conflict paths are determinate, conflict loci are not.**
  A conflict resolution has no unique correct tree *at the conflict hunks* — but
  every non-conflicting path has the deterministic `merge-tree` result. The ingest
  check must hold the resolved tree to the mechanical merge on non-conflict paths
  and admit operator freedom only at conflict loci (exact predicate = SL-212
  design).
- **Parent order.** The set-comparison in admit is lax; `merge_oid`'s *first*
  parent should be `base_oid` (the trunk-side lineage). SL-212's ingest check
  should require ordered parents, not just the set.

Legitimacy = this content-bounded provenance + explicit operator action +
preserved candidate branch + admit + expected-tip CAS.

### The amendment (D4, SL-068 candidate-admission clause)

**Before** (ADR-012 §Decision 4, "Candidate interaction/admission"):

> "Admission records an immutable `admitted_oid` after validating provenance:
> **the Doctrine-created `merge_oid`** has the recorded base/source parents, and
> `merge_oid` is an ancestor of the admitted tip."

The load-bearing word here is **"Doctrine-created."** It is *not*, however, the
only place governance encodes Doctrine authorship — see the **Downstream cascade**
below; SPEC-022 and the SL-068 design carry the same assumption. This REV amends
the *decision* (ADR-012) now and **tracks** the *model* (SPEC-022) for
reconciliation at SL-212 ship-time. The amendment is narrow in *substance* (one
provenance clause), not in *surface*.

**After** (amended intent):

> The candidate `merge_oid` is validated by provenance, not by authorship: it has
> the recorded `(base_oid, source_oid)` parents and is an ancestor of the admitted
> tip. It **may be Doctrine-produced** (the internal 3-way at `candidate create`)
> **or operator-ingested** (a hand-resolved 3-way of the recorded base+source,
> adopted via a candidate-ingest verb when `create` parks a conflict). The
> provenance contract is identical in both cases; authorship is not part of it.

Everything else in D4 stands. In particular the FF-only publication posture is
**reaffirmed verbatim**: "if the admitted OID does not fast-forward the target,
close refuses and requires a superseding candidate; it never creates, updates,
rebases, merges, or repairs a candidate at close time."

**Trunk honesty — the honest, bounded claim.** The operator-ingest path holds to
**exactly the same** inspect-before-trunk bar as today's clean Doctrine
close_target merge — it is provably **no weaker**. It does *not*, on its own,
*guarantee* that the exact admitted combination was reviewed: `integrate` requires
only that a `close_target` admission exists, and admit stores `--review` as
optional metadata without binding it to the admitted OID (`dispatch.rs:2172-2189`,
`:1653`). But that gap is **pre-existing** — it affects the clean merge equally —
not introduced here; it is filed as **IMP-303** (bind admitted OID → audit RV at
the close gate), which should land before/with SL-212's close path.

The genuine distinction from RFC-006 is therefore **enablement, not a bright
line**: the candidate path *materialises an admittable, re-auditable ref* that a
close gate can check; RFC-006's `plan_trunk_row` manufactures the merge at
integrate *after* admit with **no such surface at all**. This REV keeps composition
on the inspectable candidate; closing the inspected-vs-inspectable gap for *both*
authorship modes is IMP-303's job.

### Residual (consciously accepted)

This does **not** self-heal a concurrent trunk land: another slice landing
between candidate-create and integrate costs a supersede cycle (re-create on the
new base, possibly re-resolve, re-admit, re-integrate). Two honest bounds:

- **The supersede cycle is complete only for the *plan-time* refusal** — trunk
  moved *before* a trunk row is journaled (the common case; caught at the
  candidate-trunk FF check). It does **not** cover the *post-journal CAS race* —
  trunk moving between the journal commit and the ref mutation persists a `Failed`
  row (`dispatch.rs:2263-2264`), and replanning is `fresh`-gated / status-blind
  (`:2183`), so a superseding candidate's new admitted OID is never replayed. That
  case dead-ends into `record-integration` or manual journal surgery today. This is
  **pre-existing**, filed as **IMP-304** (let a supersede replace a `Failed`/
  `Pending` trunk row). Correctness is never at risk — CAS refuses; only recovery
  ergonomics degrade.
- **Concurrency is *reduced*, not eliminated** (not a correctness boundary; the
  CAS is). The **edge/main split** keeps most work off the integration ref — a
  non-FF `main` is usually an agent promoting `edge → main` as dispatch hygiene,
  not organic churn — and a future **advisory close-lock** (orchestrators honour it
  before touching `main` during an active close) would lower the frequency further,
  at worst slightly more code merged later. Both shrink the odds of hitting the
  supersede path; neither is insurance against it.

The ergonomic tail routes to **RFC-016** (make supersede a prescribed, mechanical
`dispatch next` action) and Path C (discrete-clone topology) — neither reverses a
publication boundary to remove retry friction.

### RFC-006 disposition

RFC-006 (`originates_from`) resolves **without** enacting non-FF auto-merge at
integrate. Its ergonomic motivation survived only as friction (SL-166 closed the
H2 correctness residuals); reversing the FF-only publication boundary to remove
retry friction is disproportionate when (a) SL-212 fills the real
candidate-construction hole under the existing exception, and (b) RFC-016 can make
the residual mechanical. RFC-006 is transitioned to `resolved` with this outcome.

### Change payload

- **`modify ADR-012`** (primary) — the D4 amendment above, **plus** a new
  operator-ingest case in ADR-012's §Verification (the current items cover only
  create/admit ancestry): assert the ingest verb rejects (a) an arbitrary tree on
  non-conflict paths and (b) reversed parent order, and accepts a genuine
  hand-resolved 3-way. Surfaced-for-manual at `revision apply`.

### Downstream cascade (tracked, not applied here)

SPEC-022 (Git interaction model) mirrors the amended clause and must be reconciled
**at SL-212 ship-time**, not now — SPEC-022 is *retrospective* ("describes shipped
behaviour; coverage reconciled, never inferred"), so amending it ahead of the code
would violate its own charter. Tracked targets:

- **REQ-316 (FR-006)** "Candidate admission by immutable OID" — the durable
  requirement encoding Doctrine-authored admission.
- SPEC-022 responsibilities line (`spec-022.toml:19`) and candidate-layer prose
  (`spec-022.md:180`) — "Doctrine no-ff 3-way `merge_oid`".

Recorded as an SL-212 follow-up; reconciled via a sibling REV when the ingest verb
lands. (SL-068's `design.md:195` also carries it, but a closed-slice design is a
point-in-time artifact — noted, not amended.)

### Not in this REV (deliberately)

- The SL-212 scope un-gate is a per-slice authored direct edit, not a `[[change]]`
  row.
- RFC-006's status move is a `rfc status` transition referencing this REV as its
  enacted outcome.
- **IMP-303** (audit-OID binding) and **IMP-304** (supersede clears a `Failed`
  trunk row) are pre-existing gaps this decision surfaced but does not introduce;
  filed for separate scheduling (IMP-303 should land before/with SL-212's close).

### Review provenance

Reasoning cross-checked by an external adversarial pass (codex, GPT-5.5) against
the code. Six findings; the core insight (SL-212 does not require RFC-006's
reversal) held. Two blockers (content-validation sufficiency; inspectable≠inspected)
and two majors (amendment surface breadth; post-journal recovery completeness) are
integrated above; the decision was unaffected. Adjudication logged to the SL-212
session.
