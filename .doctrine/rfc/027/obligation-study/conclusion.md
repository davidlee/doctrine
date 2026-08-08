# Obligation graph — study conclusion

Study of brief 03 from the external design/proof pack
(`scratch/2026-08-08/03-obligation-graph-integration-brief.md`), commissioned to
test whether Doctrine should model an explicit graph of phase-level obligations
with hard `needs` / soft `after` edges.

Evidence: `raw/thread-ab-incumbent.md` + `thread-ab-verification.md` (incumbent
concepts and actionability semantics), `raw/thread-c-frontier.md` +
`thread-c-verification.md` (frontier comparison over three real slices), plus
direct corpus checks recorded here. Every load-bearing agent claim was verified
against primary sources; two were wrong and are corrected in the verification
notes.

**Recommendation: NARROW.** Reject the stored obligation graph. Retain two much
smaller repairs the evidence does indict.

---

## 1. Current-state map

| Concern | Where it lives |
|---|---|
| Authored phase plan | `plan.toml` — `[[phase]]` rows; read by `Plan` / `PlanPhase` (`src/plan.rs:20-43`) |
| Phase objective | `objective: String` (`src/plan.rs:37`) — free text, consumed by no gate; seeds the disposable phase sheet |
| Obligations | `exit_criteria: Vec<Criterion>` — `EX-N` rows, immutable ids, authored order |
| Preconditions | `entrance_criteria: Vec<Criterion>` — `EN-N` rows |
| Proof | `verification: Vec<VerificationCriterion>` — VT/VA/VH; only VT carries a structured mandate, checked by `src/vtgate.rs` |
| Phase sequencing | Authored `[[phase]]` array order. No `needs`/`after` at phase altitude anywhere |
| Readiness authority | `compute_next_phases()` over `plan_next_rows()` (`src/dispatch.rs:5195-5225`) — scans **plan order** |
| In-flight verb selection | `select_next()` (`src/dispatch.rs:6620`) — rungs 1–3 over funnel rows in **phase-id order**; rung 4 defers to the readiness authority |
| Runtime phase status | `.doctrine/state/slice/<N>/phase-nn.toml` — gitignored, disposable |
| Entity dep/seq | `src/dep_seq.rs` — `needs` (hard, payload-free) / `after` (soft, per-edge `rank`). ADR-001 leaf, **kind-neutral** |
| Graph engine | `crates/cordage` — `NodeId(u32)`, an opaque dense index. Identity-agnostic |
| Key↔node binding | `Projection<K: Copy + Ord>` (`src/projection.rs:20`), instantiated at `Projection<EntityKey>` |
| Corpus actionability | `src/priority/channels.rs` — `eligible` / `blocked` / `actionable`; consumed by `next`, `survey`, `blockers`, `explain`, `inspect` |

**Two altitudes, no bridge.** Corpus actionability is entity-level and rides
`dep_seq` + cordage. Phase actionability is intra-slice, rides plan array order
plus gitignored runtime status, and touches neither.

## 2. Fact ownership

The test each candidate field must pass: does it own a distinct fact, or restate
one already owned? (RFC-003 derivability; `REQ-447` AC-1.)

| Candidate | Distinct fact? | Already owned by |
|---|---|---|
| Obligation identity | **No** | `EX-N` — immutable id, authored order, phase-qualified reference (`REQ-441`) |
| Obligation statement | **No** | `EX-N.text` |
| Obligation → requirement | **No** | Per-phase `requirements` array — authored in 10 plans, discarded on read (ISS-321) |
| Obligation → proof | **YES — unowned** | Nothing. No authored field links a VT row to the EX row it proves |
| Obligation refinement / split | **No** | `REQ-442` — replacement, withdrawal, one-to-many split, many-to-one merge, cross-phase relocation |
| Obligation `needs` edge | **Partly** — semantics owned, altitude not | `dep_seq` owns `needs`/`after` semantics at entity altitude; nothing carries them intra-plan |
| Phase blocked/actionable | **No** | `compute_next_phases()` derives it from plan order + runtime status |
| Cycle / dangling-reference validation | **No** | `REQ-443` already requires detection of dangling and cyclic lineage |

**One unowned fact in eight.** That is the study's central quantitative result and
it does not justify an ontology.

## 3. Historical dependency corrections

Brief 03 § D asked for real cases of dependency omission or correction.
Instrument: `grep 'id = "EN-'` over all `plan.toml`, filtered for `AMENDED`;
2,362 entrance criteria scanned, **3 distinct corrections** (each doubled in raw
counts by the slug symlink).

1. **SL-233 PHASE-11 EN-1** — *edge retarget, one-to-many.* Named PHASE-06; the
   2026-07-29 split moved the marker machinery, so "both successors are required
   — PHASE-13 for the grammar and the watermark, PHASE-14 for the parse-a-human-
   edited-document path".
2. **SL-233 PHASE-12 EN-1** — *edge retarget, one-to-one.* Same split; PHASE-06's
   successor for section fingerprints is PHASE-13.
3. **SL-182 PHASE-06 EN-1** — *edge removal.* `[AMENDED — RETIRES the
   SubagentStop premise]`: a live probe proved the worker tree persists
   post-return, so the dependency was retired outright.

**All three were handled by hand-editing EN prose.** No lineage, no record of
what else the change invalidated, no way to query which phases were affected.

**But the rate is the finding.** Three corrections in 2,362 entrance criteria —
about 0.13%. Brief 03's premise is that *"missed dependencies should be a common
execution discovery and cheap to reconcile"*. In this corpus they are rare.
Instrument limit: only `AMENDED`-tagged edits are counted, so this is a lower
bound; silent corrections would not appear. Even allowing an order of magnitude
of undertagging, the premise is not supported.

## 4. Model comparison

**Model 1 — Phase DAG.** Only phases carry `needs`/`after`.
*Cost:* new authored edges duplicating information already carried by array order.
*Consumer:* `compute_next_phases` could read edges instead of order.
*Verdict:* rejected — array order already expresses it, and no evidence shows the
order is wrong.

**Model 2 — Obligation DAG inside phase envelopes.** Stable obligation identities
carry the edges; phase status derives from contained obligations.
*Cost:* a second dependency implementation (the `dep_seq` seam is entity-bound),
a widened projection key, edge upkeep, plus a naming collision with the shipped
obligation-runbook vocabulary.
*Consumer:* none demonstrated. Thread C found the frontier unchanged across
SL-233 (16 phases), SL-057 and SL-229. The one case where obligation granularity
differed — SL-233 PHASE-10 being runnable after PHASE-04 — was an architectural
deferral the plan states deliberately, not a coarseness artefact.
*Verdict:* **rejected on absence of consumer.**

**Model 3 — No stored graph; improve derived reads.** Keep plan.toml as-is.
*Cost:* near zero.
*Consumer:* existing.
*Verdict:* **retained**, with the two narrow repairs in § 6.

**Model 4 (emergent) — graft, don't promote.** Load slice-local nodes into the
existing cordage graph for the focus slice only, via a different loader; store
nothing new. Verified cheap: cordage is identity-agnostic, `Projection<K>` is
already generic, so the change is a widened key plus a loader arm, with no cost
on corpus-wide loads (see `thread-ab-verification.md` § Direction).
*Verdict:* **technically cheap and currently unmotivated.** Model 4 is how Model
2 should be built **if** a consumer ever earns it. It is not a reason to build it
now.

## 5. Revision semantics

Largely moot under the recommendation — with no stored edges, there is no edge
revision problem. Recorded for the record, since Model 4 would inherit it:

| Change | Pending obligation | Running | Completed |
|---|---|---|---|
| Edge added | Re-derive; may become blocked | Continues; flagged for revalidation | Untouched — evidence is Git-anchored to a state that already happened |
| Edge removed | Re-derive; may become actionable | Unaffected | Untouched |
| Target split (1→N) | Re-derive against all successors | Flagged | Untouched |

The invariant, already RFC-027 P1/P5 and consistent with `REQ-439` AC-3: derived
actionability recomputes freely; observed evidence is never rewritten. A
per-load projection makes this trivial — nothing is stored to migrate.

## 6. Recommendation — narrow

**Reject** the stored obligation graph (Models 1 and 2). **Retain** Model 3, and
pursue exactly two repairs the evidence indicts:

**R1 — the EX→VT link.** The one unowned fact in § 2. No authored field connects
a verification criterion to the exit criterion it proves; the mapping lives only
in the author's prose. Every proof-bearing consumer RFC-027 contemplates needs
it, and it is a field on an existing row, not a new entity. Belongs in the
"Phase plan surface" component spec that IMP-382's `/spec-tech` half will author.

**R2 — stop discarding authored requirement links.** Ten plans populate per-phase
`specs`/`requirements`; `Plan` deserializes only `phases`. `REQ-439` AC-2
(pending) already requires phases to state their canonical links. Captured as
ISS-321; same spec home.

Neither needs an obligation concept, a graph, or a new entity kind.

**Two constraints on any future revisit.**

- *Naming.* `DEC-101` is accepted and SL-233 PHASE-16 shipped an **obligation**
  primitive — `src/design_run/runbook.rs`, ordered steps with per-step discharge.
  A plan-level "obligation" collides with live user-visible vocabulary (STD-002).
- *Precedent.* That shipped machinery already answers questions RFC-027 lists as
  open: digest-bound discharge staleness (`EX-2` — *"an id solves reference, not
  equivalence"*), versioned canonical encoding (`EX-18`), attested-vs-verified
  distinction (`EX-15`), and a closed-vocabulary scope fence (`EX-19`). Mine it
  before designing anything adjacent.

## 7. Falsifiers and unknowns

**Falsifiers the study actively tried, and their outcomes:**

- *Obligations cannot be identified without duplicating criteria* — **confirmed
  as duplication.** EX rows already are obligations.
- *Real dependencies are predominantly phase-wide* — **confirmed.** Phases share
  files; SL-233's plan states the serialism is forced, not chosen.
- *Existing orchestration already derives the same frontier* — **confirmed.**
- *Dependency upkeep exceeds its value* — **supported** by the 0.13% correction
  rate.
- *Late edge changes entangle completion semantics* — **not reached**; moot
  without stored edges.

**Unknowns:**

- Three slices of ~90 completed. A genuinely file-disjoint slice with parallel
  obligations may exist and was not sampled. Mitigation: the largest slice in the
  corpus was chosen deliberately, on the reasoning that if granularity does not
  help at 16 phases it will not help at three.
- The `AMENDED`-tag instrument undercounts silent dependency corrections by an
  unknown factor.
- Whether R1 (EX→VT) should be a field on the VT row, on the EX row, or a
  separate mapping is a spec question, not settled here.

**Retired.** An earlier note flagged a possible ordering defect —
`src/dispatch.rs:6826` sorts funnel rows by phase id while `compute_next_phases`
scans plan order, and SL-233's plan order is deliberately non-numeric. Traced:
**not a defect.** `select_next` rungs 1–3 operate over `mid` (rows already in the
funnel), where id order is a determinism tie-break among in-flight work; rung 4,
which decides what to spawn, defers to the plan-order readiness authority.
Authored order governs admission. One residual nuance, not worth an item: among
*concurrently* in-flight phases, rung 2 picks the lowest id, which on a
non-numerically-ordered plan need not be the earliest authored — harmless while
phases run serially.

## 8. Consequence for the RFC-027 patch

The external pack's RFC-027 patch brief proposes `H9` (obligation-level
dependency improves actionability) as a new open hypothesis with confirm/kill
conditions. **It should not land as open.** Its kill condition —

> most meaningful dependency is genuinely phase-wide and obligation-level edges
> add only ceremony

— is met. `H9` should be recorded as **tested and not supported**, citing this
study, in the same register RFC-027 already uses for Stage 0 / EVD-004 ("the
proposed missing concept was not earned").

`H10` (recursive refinement of obligations) is likewise weakened: `REQ-442`
already supplies split/merge/relocation lineage for criteria, so the refinement
case has an owner.

`H11`, `H12` and `H13` are untouched by this study — `H13` in particular is brief
04's subject and remains live.
