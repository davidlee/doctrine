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
**false**, and the code proves it. A hand-resolved merge whose parents are the
recorded `(base_oid, source_oid)` is a descendant of `base_oid`; if the
candidate `base` is the **current trunk tip**, the resulting `merge_oid`
**fast-forwards trunk** exactly like a Doctrine-produced clean candidate merge.
No non-FF trunk mutation is required, so no reversal of D2/D4 is required.

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
(`:1409`, `merge_oid=""`), gated by re-running admit's existing parent-set check.
The validation contract is parent-binding + descends-from — **no** content /
tree-diff validation, because a conflict resolution has no unique "correct" tree
(legitimacy is parent binding + explicit operator action + preserved candidate
branch + admit + CAS).

### The amendment (D4, SL-068 candidate-admission clause)

**Before** (ADR-012 §Decision 4, "Candidate interaction/admission"):

> "Admission records an immutable `admitted_oid` after validating provenance:
> **the Doctrine-created `merge_oid`** has the recorded base/source parents, and
> `merge_oid` is an ancestor of the admitted tip."

The single load-bearing word is **"Doctrine-created"** — the only clause in
governance excluding an operator-produced merge.

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
rebases, merges, or repairs a candidate at close time." The trunk-honesty claim
(§Consequences, "unreviewed code never touches trunk") is **preserved** — the
operator-ingested combination lands on an inspectable, admittable candidate
surface *before* trunk mutation, meeting the same bar today's clean candidate
merge meets. RFC-006's rejected capability would have composed the merge at
integrate, *after* admit — never on an inspectable surface; that is the boundary
this REV declines to cross.

### Residual (consciously accepted)

This does **not** self-heal a concurrent trunk land: another slice landing
between candidate-create and integrate costs a supersede cycle (re-create on the
new base, possibly re-resolve, re-admit, re-integrate). Accepted because:

- The **edge/main split** already insures against routine concurrency —
  integration lands on `main` while working trunk is `edge`; a non-FF `main` is
  usually only an agent promoting `edge → main` as dispatch phase hygiene, not
  organic churn.
- A cheap future hardening (out of scope here): an **advisory close-lock file**
  during an active close that orchestrators honour before touching `main` — at
  worst slightly more code merged later.
- The ergonomic tail routes to **RFC-016** (make supersede a prescribed,
  mechanical `dispatch next` action) and Path C (discrete-clone topology), neither
  of which reverses a publication boundary to remove retry friction.

### RFC-006 disposition

RFC-006 (`originates_from`) resolves **without** enacting non-FF auto-merge at
integrate. Its ergonomic motivation survived only as friction (SL-166 closed the
H2 correctness residuals); reversing the FF-only publication boundary to remove
retry friction is disproportionate when (a) SL-212 fills the real
candidate-construction hole under the existing exception, and (b) RFC-016 can make
the residual mechanical. RFC-006 is transitioned to `resolved` with this outcome.

### Change payload

- **`modify ADR-012`** (primary) — the D4 amendment above (surfaced-for-manual at
  `revision apply`).

Not in this REV (deliberately): the SL-212 scope un-gate is a per-slice authored
direct edit, not a `[[change]]` row; RFC-006's status move is a `rfc status`
transition referencing this REV as its enacted outcome.
