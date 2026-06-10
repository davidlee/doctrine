# Inquisition — SPEC-001 (Graph-Derived Priority Engine)

> **HERESIS URITOR; DOCTRINA MANET**

Adversarial review of the first technical spec in the corpus, `draft`, before any
code rides on it. Target read via `doctrine spec show` (both tiers). Doctrine
established from PRD-011 (parent), ADR-001/ADR-004, the seed docs
(`vk/docs/bough-dag-prio-architecture.md` §9, `scratch/dag.local.md` §7/§8),
`src/retrieve.rs` (SL-008 scope predicates), the phase-tracking schema, and
CLAUDE.md (no parallel implementation; storage rule).

Status: **reconciled.** Operator dispositioned all charges; SPEC-001 / requirement
tomls / PRD-009 amended. Disposition log at the foot of this file.

## Disposition log (post-operator)

- **Charge I (union cycle)** — UPHELD. Operator: `dep` authoritative, `seq` yields.
  New **D9** authored; `order_key` = dep-topology → seq-rank → fallback; the
  union-closing `seq` edge is evicted. REQ-092 gains a union-cycle criterion.
- **Charge II (D6 vapor)** — UPHELD + enriched. Demoted to dependency-bearing; OQ-009
  reopened resolved-pending. Operator's insight folded in: **both** (a) plan
  declared-paths (prospective, at the planning gate — enables prefactor before the
  code exists) **and** (b) audited touched-paths (retrospective) are necessary; their
  divergence flipping a trigger is a **caught oops** / drift signal. Three
  prerequisites named; matcher contract pinned to the leaf glob predicates (not
  `match_scope`). REQ-093 amended; new OQ-7.
- **Charge III (age)** — DOWNGRADED (operator "not sold"; conceded). A clock-free
  stable ordinal *is* constructible (source id + append position); REQ-077
  determinism never at risk. Kept as honesty note: contract = total + recompute-stable,
  derivation deferred to the adapter slice, "creation-sequence" → "stable authoring
  ordinal." D5 / REQ-096 reworded.
- **Charge IV (multi-cycle eviction)** — UPHELD. D5 / REQ-092 now iterate eviction to
  a fixpoint.
- **Charge V (unexplained eviction)** — UPHELD. D5 / REQ-092 surface every evicted
  edge in provenance.
- **Charge VI (duplication)** — UPHELD. REQ-094 narrowed to the staleness stamp
  (cites REQ-078); REQ-095 narrowed to the boundary test + forbidden-core list (cites
  REQ-079); REQ-092 ac1 now cites REQ-076 as the obligation it mechanises.
- **Charge VII (FR-005 boundary smear)** — UPHELD. Operator: push schema to PRD-009.
  D4 / REQ-096 mark the authored edge schema PRD-009-owned + optional; **PRD-009
  OQ-007** raised.
- **Charge VIII (ADR-004)** — VACATED. The malformed `related = [2]` was repaired by a
  concurrent session before reconciliation; `doctrine adr show ADR-004` now loads
  clean. No action.

---

## 1. Charges

### Charge I — The mixed `dep`/`seq` union cycle: a hole the per-overlay policy provably cannot see. (HIGH)

**Doctrine violated.** PRD-011 invariant "a relation kind declared acyclic reports
cycles as diagnostics rather than silently producing a misleading order"; SPEC
REQ-076/REQ-092; the determinism concern.

**Evidence.** D5 enforces acyclicity **per overlay**: `dep` rejects, `seq` evicts.
Consider `A —dep→ B` and `B —seq→ A`. The `dep` overlay holds only `A→B` (acyclic);
the `seq` overlay holds only `B→A` (acyclic). **Neither overlay's policy fires** —
yet the union of the two orderings is cyclic (A must precede B by dependency; B is
preferred before A by sequence). The spec nowhere states how the `dep` partial order
and the `seq` partial order **compose** into `order_key`, nor what happens when their
union cycles. If `next`/`survey` topo-sort the union, they cycle on input each
overlay swore was acyclic; if they do not, then `seq`'s evict-on-cycle policy is
unmotivated (a pure scalar rank needs no acyclicity). The mechanism is incoherent
at the seam where its two halves meet.

**Risk.** This is the load-bearing path — ordering and blocking are the entire
capability. A false order or a panic on a union cycle is exactly the failure PRD-011
forbids. Confessed under cross-examination: the eviction and rejection rules are
provably blind to it.

**Sentencing.** Author a new decision (append; do not renumber): fix overlay
composition and the union-cycle policy. The natural reading — `dep` is authoritative,
`seq` yields — means a `seq` edge that closes a cycle **against the resolved `dep`
order** is evicted by the same `(rank, age)` rule; `seq` never overrides a `dep`
ordering. State it, and state that `order_key` composes dep-topology first, seq-rank
within, fallback last. Verification: a test seeding `A —dep→ B`, `B —seq→ A` yields a
stable order with the `seq` edge surfaced as evicted, no panic, no false topo.

### Charge II — D6's architectural-trigger mask is vapor on BOTH input sides; it over-claims OQ-009 closure and "acceptance fixtures." (HIGH)

**Doctrine violated.** "Use the CLI / authored state, don't guess"; the testability
the spec asserts ("if the mask cannot express their triggers and surface them at the
gate, D6 is not done").

**Evidence.** Three confessions:
1. **Query side does not exist.** D6 runs the scope match "over the phase's
   declared paths." The phase-tracking TOML (`.doctrine/state/.../phase-NN.toml`)
   carries `schema/status/started/completed/progress` only — **no `paths`/`files`/
   `globs` field**. There is no "phase's planned/touched file set" in the authored
   model.
2. **Authored side does not exist.** D6 names IMP-013/IMP-014 as "the acceptance
   fixtures." Their TOML carries `tags = [… "trigger" "deferred"]` and **nothing
   else** — no `{ globs = […], note = "…" }` capture field. D6 itself admits the
   field is "PRD-009's capture seam" (unbuilt). The "two-path / single-path"
   characterisation lives in the author's head, not in authored structure.
3. **The matcher is overstated.** "Reuse the `retrieve.rs` scope predicates … no new
   matcher." The actual scope engine `match_scope(m: &Memory, q: &QueryContext)`
   (`src/retrieve.rs:143`) is **typed to `Memory` + `QueryContext`**. Only the leaf
   free functions `glob_admits(pattern, query)` (`:126`) and `path_admits` (`:117`)
   are generic `&str` matchers. Reusing them is real; reusing "the scope-admittance
   engine" is not — a new caller/adapter is required.

**Risk.** "Closes PRD-011 OQ-009" and "IMP-013/014 are acceptance fixtures" are
**not evaluable** — the triggers are unauthored and the gate input is unbuilt. The
first tech spec in the corpus sets precedent; an unfalsifiable acceptance criterion
is a precedent for vapor.

**Sentencing.** Demote D6 from "closed" to dependency-bearing. Name the two
prerequisites explicitly: (a) PRD-009 trigger-capture field `{ globs, note }`;
(b) a phase declared-path field (or the touched-path source the gate will read).
Specify the matcher contract precisely — trigger globs are the **pattern**, the
phase file set the **subject**, matched by `glob_admits`/`path_admits` (the free
fns, not `match_scope`). Keep OQ-009 honest: either reopen it as
"resolved-pending-prerequisites" or fold the prerequisite into OQ-6.

### Charge III — `age` rests on a monotonic creation source that does not exist; the label "creation-sequence" bears false witness. (HIGH)

**Doctrine violated.** Determinism concern (REQ-077); pure/imperative split; "ask,
don't infer."

**Evidence.** D5/REQ-096: eviction tie-breaks on `(rank asc, age asc)`, where `age`
is "a clock-free creation-sequence stamp the adapter supplies." But: relationship-
array edges carry **no per-edge timestamp**; entity `created` is **day-granular**
(IMP-013 `created = "2026-06-10"` — ties everything authored that day); there is no
`link` verb minting a creation counter. **No monotonic creation source exists.** The
only deterministic source available is the adapter's **scan order** (entity-id sort
+ edge array position) — which is not "creation" order at all, and makes the
equal-rank eviction an arbitrary structural artifact the operator cannot predict or
control.

**Risk.** Either age is undefined (non-deterministic eviction → REQ-077 violated) or
it is scan-order wearing a "creation-sequence" mask (deterministic but semantically
dishonest — "evict oldest-first" implies authoring age it cannot deliver).

**Sentencing.** Name the source. State `age` is a **deterministic adapter scan-order
index** (specify the scan ordering: entity-id ascending, then edge position), drop
or qualify the "creation-sequence" framing, and confess that the equal-rank eviction
tie-break is **structural, not authoring intent**. Verification: two equal-rank
`seq` edges in one cycle resolve identically across recomputes.

### Charge IV — `seq` eviction is underspecified for multiple/disjoint cycles. (MEDIUM)

**Evidence.** D5/REQ-092: "evict **the** participating edge minimal under
`(rank, age)`" — singular. Two **disjoint** `seq` cycles require one eviction each; a
single global-minimal eviction breaks only one, leaving the other to poison the
"always acyclic" claim.

**Sentencing.** Restate iteratively: "while the `seq` overlay contains a cycle, evict
the globally-minimal participating edge; repeat to fixpoint" (equivalently, per
strongly-connected component). Assert termination (each eviction strictly reduces
edge count) and determinism. Verification: a graph with two disjoint `seq` cycles
ends acyclic with exactly two evictions.

### Charge V — The evicted `seq` edge vanishes unexplained — magic, not structure. (MEDIUM)

**Doctrine violated.** PRD-011 principle "explanations are structured, not prose
magic"; REQ-072/REQ-077.

**Evidence.** D5 evicts an **authored** edge from the view. The spec mandates a
diagnostic for `dep` cycles (node ids + edge kinds) but is **silent** on surfacing
`seq` eviction. An operator's authored sequencing preference disappears from the
order with no reason emitted — the exact "magic" the PRD forbids.

**Sentencing.** Require the evicted edge to appear in the affected nodes'
explanation/provenance (a structured "seq edge evicted, lost the `(rank, age)`
contest" reason). Verification: `explain` over a node downstream of an evicted edge
names the eviction.

### Charge VI — Requirement duplication: SPEC REQs restate PRD REQs they were meant to complement. (MEDIUM)

**Doctrine violated.** CLAUDE.md "No parallel implementation"; reference discipline
(refine the durable REQ, don't clone it).

**Evidence.**
- **REQ-094 (NF-001) ⊃ REQ-078 (NF-002).** REQ-094 ac1/ac3 — "no derived value
  written into entity TOML as canonical truth … mutates no status/resolution/
  item_kind/priority/relation" and the recompute-equivalence ac2 — restate REQ-078
  ac1/ac2/ac3 near-verbatim. Genuinely new in REQ-094: **only** the policy-version +
  input-signature stamp (PRD OQ-008 / D7).
- **REQ-095 (NF-002) ⊃ REQ-079 (NF-003).** REQ-095 ac3 ("core test suite carries no
  product vocabulary; policy tests carry interpretations explicitly") restates
  REQ-079 ac1. Genuinely new: the forbidden-core **enumeration** + the boundary-test
  placement rule.
- **REQ-092 (FR-003) ac1 ↔ REQ-076 (FR-007).** "dep cycle reported as a diagnostic
  naming node ids + edge kinds, never silently ordered" is REQ-076's obligation
  restated.

**Risk.** Two entities now own the same invariant; they will drift, and a reviewer
cannot tell which is authoritative.

**Sentencing.** Narrow each SPEC requirement to its **genuinely-new mechanism**
obligation and **cite** the PRD requirement for the inherited invariant rather than
restating it: REQ-094 → keep only the version/signature stamp, cite REQ-078 for
disposability/no-mutation; REQ-095 → keep the forbidden-core list + boundary test,
cite REQ-079 for the generic-core outcome; REQ-092 → keep the dep/seq split +
eviction mechanism, cite REQ-076 for the diagnostic obligation.

### Charge VII — FR-005 mints new authored relation schema in a tech spec; the product-capture boundary is smeared and unreconciled with PRD §4. (MEDIUM)

**Doctrine violated.** PRD-011 §4 constraint "Capture must never require … dependency
modelling" (seed `dag.local.md:62` confirms); PRD §1/§2 framing — *derive from the
typed relations **already in the corpus***; product-vs-tech boundary.

**Evidence.** D4 / FR-005 (REQ-096): "**Doctrine authors** two new typed edge kinds
… a hard `dep` … and a soft `seq` … with an int `rank`." This is a **product-capture
decision** — a new authored relation schema, peer to PRD-009's priority seam — made
unilaterally inside TECH-001. The spec (a) never states these edges are **optional
enrichment** preserving cheap capture (leaving them in apparent tension with PRD §4's
"no dependency modelling required"), and (b) assigns the authored-edge schema to **no
product spec**, unlike D6 which correctly defers the trigger field to PRD-009.

**Risk.** A tech spec deciding product capture surface sets a boundary-eroding
precedent for every tech spec after it.

**Sentencing.** (a) State explicitly that `dep`/`seq` edges are optional — capture
stays cheap, unedged items still surveyed by derived context + fallback — preserving
PRD §4. (b) Assign the authored `dep`/`seq` schema to a product spec (PRD-009, as D6
does for the trigger field), or raise an Open Question that the new authored relation
schema needs product blessing before FR-005 is buildable.

### Charge VIII (collateral) — ADR-004 is unreadable; SPEC-001 leans on it. (LOW; out of SPEC scope)

**Evidence.** `doctrine adr show ADR-004` fails: `TOML parse error … line 16 …
related = [2] … invalid type: integer 2, expected a string`. REQ-091 cites ADR-004
as load-bearing (outbound-only edges) — the spec rests on an entity that will not
load.

**Sentencing.** Not a SPEC-001 defect; surfaced because the spec depends on it. Fix
the ADR file: `related = ["ADR-002"]` (id-string, per reference discipline).

---

## Recanted charges (logged, per SL-020 §10 discipline)

- **cordage cargo layout forces restructuring** — FALSE. `members = ["."]` →
  `members = [".", "crates/cordage"]` is a valid workspace; the root crate stays.
  cordage needs none of `bm25`/`regex-lite`/`glob` (the core is path-free by D6).
  The one residue: FR-001's "no **product** (bough) dependency" is **not**
  cargo-enforced (only the reverse-direction doctrine dep is, as a cargo cycle). Fold
  a manifest-assertion test into FR-001's verification rather than trusting prose.
- **REQ-09x `.md` bodies are hollow** — NOT CHARGED. Requirement description +
  acceptance live in the TOML tier by design; the empty `.md` is correct (SL-020's
  inquisition C4 was a false "hollow" charge for exactly this — bearing false witness
  by reading one tier).

---

## 2. Questions

1. **Composition (Charge I):** is `dep` authoritative over `seq` such that a `seq`
   edge closing a cycle against the resolved `dep` order is evicted? Or do you intend
   a different precedence?
2. **D6 (Charge II):** do you accept demoting D6 to dependency-bearing (two named
   prerequisites + a precise matcher contract), reopening OQ-009 as
   resolved-pending-prerequisites?
3. **`age` (Charge III):** confirm `age` = deterministic adapter scan-order index
   (id-asc, edge position), and that "creation-sequence" framing is dropped.
4. **FR-005 ownership (Charge VII):** does the new `dep`/`seq` authored schema belong
   to PRD-009, or should SPEC-001 raise an OQ pending product blessing?
5. Reconcile into SPEC-001 now (draft; append decisions, narrow requirements), or
   hold for a second pass?

---

## 3. Pronounce Judgement

**This spec is sound in spine but harbours heresy at three load-bearing seams.** The
three-layer split (D1), the boundary test (D2), the disposable-derived discipline
(D7/D8), and the seed absorption (§9 quoted faithfully) are doctrinally clean and
faithful to PRD-011. But the mechanism is **incoherent where its two overlays meet**
(Charge I), **unbuildable as written** at the trigger mask (Charge II — both inputs
vapor), and **rests on a phantom clock** (Charge III — no creation source exists).
Three further sins — multi-cycle eviction (IV), unexplained eviction (V), and
requirement cloning (VI) — and one boundary smear (VII) compound it. As the FIRST
tech spec, its form is precedent: an unfalsifiable acceptance criterion (D6) and a
cloned requirement (REQ-094/095) must not be canonised. **Heresy confirmed — not
mortal, but it must burn before any code rides on it.**

## 4. Sentencing (ordered)

1. **Charge I** — author a decision fixing overlay composition + union-cycle policy
   (`dep` authoritative, `seq` yields). Verify with the `A —dep→ B`, `B —seq→ A`
   test. *Penance: the heretic who let two acyclic overlays breed a cyclic union
   shall be broken on the wheel, one spoke per unhandled edge.*
2. **Charge III** — name `age` as a deterministic scan-order index; strike the
   "creation-sequence" mask. *He who invents a clock from nothing is fitted with the
   heretic's fork until he confesses the true ordinal.*
3. **Charge II** — demote D6; name the two prerequisites + the matcher contract;
   reopen OQ-009 honestly. *The false prophet of a gate that cannot fire shall walk
   the auto-da-fé barefoot over the globs he could not match.*
4. **Charges IV & V** — restate eviction iteratively; require evicted edges in
   provenance. *The edge cast silently into the void shall be exhumed and read aloud
   at the stake.*
5. **Charge VI** — narrow REQ-094/095/092 to new mechanism; cite the PRD invariants.
   *The forger of duplicate requirements shall have both copies nailed to the church
   door and the lesser struck through.*
6. **Charge VII** — declare `dep`/`seq` edges optional; assign schema ownership or
   raise an OQ. *The tech-scribe who trespassed onto product capture shall recant his
   schema in the chapter house.*
7. **Charge VIII** — repair `adr-004.toml` (`related = ["ADR-002"]`). *A swift
   penance; the malformed integer is cast out.*

Verification gate for the whole: `doctrine spec validate SPEC-001` clean,
`doctrine validate` clean, every narrowed requirement cites its PRD parent, and the
two new tests (union cycle, multi-cycle eviction) named in the design.

> **HERESIS URITOR; DOCTRINA MANET**
