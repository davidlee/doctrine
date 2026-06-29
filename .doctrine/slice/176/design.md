# SL-176 Design — Finish Axis B: retire `slices` / `drift`

<!-- Reference forms: entity ids padded (ADR-016, SPEC-018); doc-local refs bare
     (A.1 section, Q1 question, R1 risk, B.3 class). -->

## Status

Design draft for **RFC-003 § "Finish Axis B"** — the unfinished work→backlog half of
the relation-vocabulary collapse SL-149 began for work→canon. The *what* is decided
(RFC-003, direction locked 2026-06-29); this design resolves the **mechanism**.

**Governance ratification is deferred to reconciliation** (per scope + RFC-003): the
ratifying ADR (amend ADR-016 / ADR-010, or a sibling) is authored at P5/reconcile, after
this design is adversarially reviewed — *not* in design or plan. This differs from SL-149,
which folded its ADR into design.

Governing context already in canon: **ADR-016** (the closed role dimension —
structure/intent split; this slice extends it), **ADR-010** (unify contract + write seam,
storage bespoke), **ADR-004** (outbound-only, reciprocity derived), **SPEC-018** (the
cross-corpus relation contract), **RFC-003** (the deliberation this slice implements).

## Decision ledger (locked)

| # | Decision | Source |
|---|---|---|
| **Q1 tier** | `fulfils` = **Tier-1 + `degree: Option<Degree>` column** + `link --degree`. Mirrors SL-149's `role` Option-column; degree is *non-keyed* payload (never enters `lookup`). Tier-2 typed rejected — that shape is for collection/per-row payloads; `fulfils` is one scalar facet per single-target edge. | user-locked |
| **Q2 rename** | Rename `Role::ScopedFrom` → `OriginatesFrom` **in place** (wire `scoped_from`→`originates_from`); widen source/target. Parallel naming with the existing REV→RFC `originates_from` **label** is **accepted** (different namespaces: `label=` vs `role=`; same English meaning "born from"; different tiers). | user-locked (option 1) |
| **Q3 inbound** | Degree-aware inbound: degree annotates each **source** within the `fulfilled by` bucket (per-edge suffix), not a separate bucket key. New `inbound_degree_index` parallel to `inbound_role_index`. | user-locked |
| **Q4 author-end** | Author-at-mutable-end = **convention** + source-set partial fence (`fulfils` SL-only structurally forces slice-end). No lifecycle-aware enforcement. | user-locked |
| **Q5 cascade** | Close-cascade hint (`doctor`/`/close` reading `fulfils(full)`) **spun out → IMP-210**. Not in this slice. Hint-not-auto regardless (RFC-003 F-6). | user-locked |
| **Q6 drift** | Name the mapping now, re-census at execution: entity "carved out from" → `originates_from`; "feeds into" → `needs`/`after`; 5 free-text non-entity stay `drift` (deferred). | user-locked |
| **D-degree-default** | `None` degree ≡ **Full**. `partial` is the marked exception. Does NOT repeat SL-149's banned `unspecified` (that was banned because role keys the target gate; degree keys nothing). | design |
| **D-edge-identity** | Edge identity = `(label, role, target)`; **degree excluded** (one `fulfils` edge per (slice,item)). `append_edge` **upserts** degree; `unlink` ignores degree. | design |

---

## Section A — Model & code impact (`src/relation.rs` unless noted)

Layering (ADR-001): `Role`/`Degree` + rules + `lookup`/`validate` are leaf/engine;
`link --degree` is command; cordage stays unaware. Extends ADR-016, composes with
ADR-010 / ADR-004 (no supersession).

### A.1 New `Degree` enum (mirrors `Role`)

```rust
/// The completion facet on a `fulfils` edge: how much of the target backlog item this
/// slice satisfies. Pure per-edge annotation — NOT a lookup key (unlike Role), so it
/// never enters the target gate. Copy+Ord → canonical order = declaration order (REQ-077,
/// no HashMap on the relation path). Does NOT aggregate (two Partial ≠ Full; ADR-016 §2 / F-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub(crate) enum Degree { Full, Partial }   // wire "full" | "partial"; name()/from_name round-trip
```

`None` degree ≡ **Full** (D-degree-default). Full is the common case; `partial` is marked.
No persisted "unspecified": degree keys no gate, so a defaulting `None` punctures no
validation (contrast SL-149's role `unspecified` ban, which would have punctured the
role-keyed target gate).

### A.2 `Role` — rename in place (Q2)

`Role::ScopedFrom` → `Role::OriginatesFrom`; wire `"scoped_from"` → `"originates_from"`;
inbound `"scoped into"` → **"originated from"** (born-end reading; exact wording settled in
P1 — golden-load-bearing, R6). The `references(originates_from)` rule widens:

```
references | [SL, ISS,IMP,CHR,RSK,IDE] | OriginatesFrom | "originated from" | Kinds(BACKLOG + [SL]) | One | Writable | degree_bearing:false
```

- **Sources** widen `[SL]` → `{SL + backlog kinds}`: a backlog item authors "born from SL"
  (IMP-207's `spawned_from`); a slice authors "born from idea / from sibling slice."
- **Target** widens `Kinds(BACKLOG)` → `Kinds(BACKLOG + [SL])`: SL→SL splits, backlog→SL
  provenance.
- **Collision (R4):** `originates_from` also exists as a tier-2 **label** (REV→RFC,
  SL-122, inbound "precursor of"). Distinguished in storage by field (`label=` vs `role=`)
  and tier; both mean "born from." Accepted parallel naming.

### A.3 `RelationLabel` — add `Fulfils`, retire `Slices`

Add `Fulfils`; `Slices` retained through migration then dropped (SL-149 retain-then-cut
cadence). `Drift` **stays** (non-entity rows deferred). New rule:

```
fulfils | [SL] | role:None | "fulfilled by" | Kinds(BACKLOG) | One | Writable | degree_bearing:true
```

SL-only source → **structurally enforces** author-at-slice-end (Q4 fence). New variants
land at the source kind's axis-run tail (R2-C1 / VT-1 lockstep discipline).

### A.4 `RelationRule` — one new column

```rust
pub(crate) struct RelationRule {
    // … existing: sources, label, role, inbound_name, target, tier, link …
    pub(crate) degree_bearing: bool,   // NEW — true ONLY on the fulfils row
}
```

Table-driven (ADR-010 D2 spine): `validate_link` reads `degree_bearing` rather than
hardcoding `label == Fulfils`. A table invariant asserts it true on exactly the `fulfils`
row.

### A.5 Edge / row shapes — thread degree

```rust
struct RelationEdge { label: RelationLabel, role: Option<Role>, degree: Option<Degree>, target: String }
struct RelationRow  { label: String, role: Option<String>, degree: Option<String>, target: String } // serde skip_if None
// TOML: [[relation]] label="fulfils" degree="partial" target="IMP-141"
//       [[relation]] label="fulfils" target="IMP-005"          # no degree key ≡ full
```

**Edge identity = `(label, role, target)`** — degree **excluded** (D-edge-identity): one
`fulfils` edge per (slice, item); you do not fulfil the same item both full and partial. So:

- `append_edge` matches `(label, role, target)` and **upserts degree** (re-link updates).
  Discipline: degree is set once at slice close; partial→full is a *new slice's* edge,
  never an edit to a closed slice's edge (RFC-003).
- `unlink` matches `(label, role, target)`, degree ignored.
- `read_block` parses the optional `degree`.

### A.6 Functions re-keyed

- `lookup` — **unchanged** (degree not a key; only `role` keys the rule, as SL-149).
- `validate_link(source, label, role, degree)` — new **`DegreeNotApplicable`** (degree given
  on a non-`degree_bearing` label, e.g. `references --role concerns … --degree full`),
  symmetric to SL-149's `RoleNotApplicable`. **No `MissingDegree`** (absent ≡ full).
  Target-kind mismatch still refused via `check_target_kind` against the role-keyed
  `TargetSpec` (now widened for `originates_from`).
- **Graph/projection** carry degree (like role rode SL-149 F1): `RelationEdge` →
  `CatalogEdge` → `InspectView`. New `inbound_degree_index: (source,label,target) → Degree`
  parallel to `inbound_role_index`. `render_inbound` suffixes degree per-source:
  `fulfilled by: SL-A · SL-B (partial)`. **Overlay allocation stays label-keyed** —
  `fulfils` is one overlay-backed label; cordage stays vocabulary-unaware (ADR-016 §3, R8).
  Inbound grouping key stays `(label, role)`; degree is rendered inline within a source's
  entry, not a grouping dimension (Q3).

### A.7 CLI

```
doctrine link  <SL> fulfils <BACKLOG> --degree partial    # degree optional; absent ≡ full
doctrine link  <SL> fulfils <BACKLOG>                      # ≡ full
doctrine link  <SL> references --role originates_from <BACKLOG|SL>
doctrine link  <BACKLOG> references --role originates_from <SL>   # born-end = backlog
doctrine unlink <SL> fulfils <BACKLOG>                     # matches (label,role,target), degree ignored
# --degree on a non-fulfils label → DegreeNotApplicable
```

---

## Section B — Migration (SL-149 redux + a direction-flip)

Re-census live at execution (SL-149 AR-1): the P1/IMP-207 lists are **reference, not
input** — concurrent authoring on shared `main` keeps minting fallout. Counts below are
the live census snapshot (~2026-06-29).

### B.1 Population & classes

| # | today | count | → becomes | mechanic | triage |
|---|---|---|---|---|---|
| 1 | `references(scoped_from)` (SL→backlog) | 19 | `references(originates_from)` | **in-place wire rename** (`role=` string); same file/direction/target | none — deterministic |
| 2 | `slices` **provenance** (BACKLOG→SL) | ~19 (IMP-207) | `references(originates_from)` | **relabel in place** in backlog toml; same direction (born-end = backlog) | classify prov-vs-fulfil |
| 3 | `slices` **fulfillment** (BACKLOG→SL) | ~63 | `fulfils` (SL→backlog) | **two-file flip**: delete from backlog toml, add to slice toml; degree default full | classify + degree |
| 4 | `drift` entity "carved out from" | ~1 | `references(originates_from)` | relabel/relocate per born-end | per-row |
| 5 | `drift` entity "feeds into" | ~1 | `needs` / `after` (dep-seq layer) | author dep edge (different subsystem) | per-row |
| 6 | `drift` free-text (memory/file) | 5 | **untouched** | stays `drift` (deferred non-entity edge) | none |

**Class 3 is the novel mechanic** SL-149 lacked: SL-149 relabelled in place (same file).
Here 63 edges *relocate* backlog→slice toml (direction flip). The old backlog-end `slices`
inbound view re-materialises as the slice-end `fulfils` outbound + the backlog item's
derived `fulfilled by` inbound.

**Prov-vs-fulfil triage criterion (SR-3)** — the load-bearing judgement separating class 2
from class 3 on the 82 `slices` edges. A `slices` edge (backlog→SL) is:

- **provenance** (class 2 → `originates_from`) when the backlog item was *born from* that
  slice — discovered/spawned as a side-effect of the slice's work (the item post-dates the
  slice; the `doctor` lifecycle check flagged exactly these as IMP-207);
- **fulfillment** (class 3 → `fulfils`) when the slice was *created to do* the item's work
  (the item pre-dates the slice and scoped it).

Heuristic: creation order + intent (did the item scope the slice, or did the slice spawn
the item?). Recorded per row in the disposition artifact; never inferred silently.

**Coexistence (SR-2):** `originates_from` and `fulfils` may both hold on the *same* (SL,
backlog) pair — a slice scoped *from* an improvement (`SL references(originates_from) IMP`)
that also *does* its work (`SL fulfils IMP`). Distinct labels, distinct directions of
intent; not mutually exclusive, no conflict.

### B.2 Disposition artifact (SL-149 F2 — oracle verifies ASSIGNMENT, not survival)

Edge-set preservation alone is insufficient: a bug mapping all `slices`→provenance would
preserve `(source,target)` while corrupting the exact classification this slice exists to
set. Committed `migration-dispositions.md` records per row:

- **prov vs fulfil** classification + one-line rationale (the load-bearing judgement);
- **degree** per fulfil row. **Every class-3 edge is affirmatively examined for partial
  (SR-4)** — `full` is the default *after* examination, never a blind default. Blind-
  defaulting would lose exactly the completion signal this slice exists to add (the RFC
  named IMP-141 as a partial exemplar). Each `partial` carries a one-line rationale.

Oracle: (1) exact expected `(source,target) → (label, role|degree)` per row; (2)
disposition artifact match; (3) edge-count + **class-aware** `(source,target)` multiset
(classes 1/2/4 preserve the pair; class 3 flips source↔target; class 5 moves label-space;
class 6 unchanged) — *secondary* sanity, not the primary proof.

### B.3 Atomicity (SL-149 F3 — no-dual-read ⇒ no valid intermediate)

Full in-memory transform of every affected `[[relation]]` row across the whole corpus
(deterministic class 1 + hand-dispositioned 2/3/4/5) → **single atomic apply** →
`validate` only after. Parser changes (`Fulfils`, `OriginatesFrom`, `degree` column,
`Slices` dropped) + rewritten corpus land in the **same commit** — no commit where code
and corpus disagree. Vehicle: out-of-band one-shot (gated test or throwaway bin; **no
shipped `migrate` verb** — SPEC-018 dogfood precedent); plan picks.

### B.4 Scaffold templates + dogfooding

- `install/templates/{slice,backlog}.toml` — strip/rewrite scaffold rows referencing
  `slices`/`drift` (pin exact at plan).
- **Dogfooding:** SL-176's own toml migrates like any other. Its `references(concerns)`
  edges (RFC-003, IMP-207, IMP-149) are unaffected. The `IMP-210 references(concerns)
  SL-176` edge authored during design is provenance-shaped → migration may retcon it to
  `originates_from` (minor; dispositioned).

### B.5 What does NOT migrate

`references(implements)` (95), `references(concerns)` (112), `related` (130),
`governed_by`, `members`, `reviews`, etc. — untouched, goldens byte-identical. Only the
`slices` / `scoped_from` / `drift`-entity goldens change by design.

---

## Section C — Verification alignment

**Behaviour-preservation gate (machinery vs content split, SL-149):** the entity-engine
*machinery* (generic seam, read/write dispatch, `outbound_for`, `validate_relations`,
`lookup`) stays behaviour-preserving — those suites green **unchanged**. The *vocabulary
content* changes deliberately — `slices`/`scoped_from`/`drift`-entity goldens update. Proof
obligation: an explicit list of which goldens legitimately change vs which stay
byte-identical.

### Tests to change / add

- **Lockstep (VT family):**
  - VT-1 enum order — `RelationLabel` gains `Fulfils`, drops `Slices` (post-migration);
    `Role` renames `ScopedFrom`→`OriginatesFrom`; new `Degree` order pinned.
  - VT-2 `sources_match_shipped_accessors` — `fulfils` source `[SL]`; widened
    `originates_from` source set.
  - VT-4 exact-coverage — fully-populated fixture authors one edge of every legal
    `(label, role)` incl. `fulfils` + widened `originates_from`; **degree** asserted on the
    `fulfils` edge.
  - VT-3 `inbound_name` identical per `(label, role)` — add `fulfils`→"fulfilled by",
    renamed `originates_from`→"originated from".
  - New: `degree_bearing` true on exactly the `fulfils` row; `Degree` name/from_name
    round-trip; canonical degree order.
- **Validation:** `DegreeNotApplicable` (degree on non-`degree_bearing` label); corpus
  `validate` clean post-migration (no `IllegalRow`, no dangler regression); a hand-edited
  bad-degree `fulfils` row flagged.
- **Storage round-trip:** author `fulfils --degree partial` → row → read back → `inspect`
  slice outbound `fulfils (partial)` + backlog inbound "fulfilled by … (partial)";
  **upsert** — re-link same `(label,target)` new degree updates not duplicates;
  degree-absent edge serialises with no `degree` key (≡ full); `unlink` matches
  `(label, role, target)` ignoring degree.
- **Migration oracle (B.2):** exact `(source,target)→(label, role|degree)` per row;
  disposition match; class-aware multiset. Render **not** byte-identical — after-migration
  goldens assert new vocabulary (`slices` inbound "slices" → `fulfils` "fulfilled by";
  "scoped into" → "originated from"); storage-level post-check guards on-disk row order
  (render launders it — SPEC-018 concern).
- **Surfaces:** `inspect` mixed (fulfils+degree, originates_from, label-only) golden;
  `relation list`/`census` (`slices` gone, `fulfils`+degree present, `scoped_from`→
  `originates_from`); web-graph edge label; slice/backlog `show --json` (the `slices`-
  derived field, if any, → `fulfils`/derived-inbound shape — enumerate affected goldens at
  plan).
- **Determinism:** BTree ordering only; canonical `(label, role, degree)` order =
  declaration order; no `HashMap` on the relation path (REQ-077).

---

## Section D — Phasing, risks, carried opens

### Phasing sketch (plan refines; **no ADR phase** — ratification deferred to reconcile)

1. **P1** — `Degree` enum + `Role` rename + `RelationLabel::Fulfils` + rules
   (`fulfils`, widened `originates_from`) + `degree_bearing` column + lockstep VT tests.
   Leaf/engine. `Slices` retained.
2. **P2** — storage (`RelationEdge`/`RelationRow`/`read_block`/`append`-upsert/`remove`) +
   `validate_link` `DegreeNotApplicable` + `check_target_kind` for widened `originates_from`.
3. **P3** — surfaces (`CatalogEdge` degree, `inspect`, `inbound_degree_index`,
   `render_inbound`, `relation list`/`census`, web graph, `show --json`) + `link --degree`.
4. **P4** — migration: full in-memory transform (classes 1–5) + disposition artifact +
   single-shot apply + class-aware oracle + **drop `Slices`** + scaffold templates. Hard
   cut, same commit.
5. **P5** — docs + reconcile (see Reconciliation intent below): SPEC-018 +
   `relation-vocabulary.md` updated to describe `fulfils`/degree/`originates_from` and the
   retired `slices`; governance ratification ADR authored **here**; RFC-003 closed.

### Risks

- **R1** — class-3 direction-flip (two-file move): drop/double-author hazard → class-aware
  multiset oracle.
- **R2** — prov-vs-fulfil triage is human judgement over 82 `slices`; IMP-207 list stale →
  re-census live, disposition per row.
- **R3** — degree-as-mutable **upsert** departs from idempotent append → test the update
  path explicitly. **Gate-compliance (SR-6):** for roleless/degreeless edges the upsert is
  a strict no-op (degree always `None`, match → no change), so existing `append_edge`
  suites stay green unchanged; the upsert is a superset path exercised only by `fulfils`.
- **R9** — `originates_from` is authored at the born end by *convention* (Q4), and both
  source kinds are legal, so a mis-authored reverse edge (origin authoring toward the born
  entity) is not kind-catchable by `validate` → accepted residual; cleaned at reconcile if
  the dogfood census surfaces any. Lifecycle-aware enforcement deferred.
- **R4** — `originates_from` label(REV)/role(references) name collision → name→meaning
  paths + diagnostics disambiguate by field; accepted (Q2 option 1).
- **R5** — golden-churn volume → machinery-vs-content split explicit so a reviewer tells
  deliberate change from regression.
- **R6** — inbound wording ("originated from", "fulfilled by") load-bearing for goldens →
  settle in P1 before surface goldens harden.
- **R7** — degree `None`≡full conflates unspecified with full → accepted (degree keys no
  gate; unlike SL-149's banned `unspecified` role).
- **R8** — cordage overlay stays **label-keyed** (`fulfils` = one overlay; ADR-016 §3) →
  confirm vocabulary-unaware.

### Carried opens (deferred, named)

- Close-cascade hint consumer → **IMP-210**.
- Create-time provenance flag (`--originates-from`) → **IMP-156** follow-up.
- Non-entity-target edge (5 `drift` free-text) → IMP-012 / IDE-015.
- Sub-roles on `originates_from` (`scoped` vs `follow_up`) → deferred until an edge demands.
- Lifecycle-aware author-end enforcement → convention now (source-set partial fence).
- Governance ratification ADR (amend ADR-016 / ADR-010, or sibling) → reconciliation.

### Reconciliation intent (P5 — authored at reconcile, not design/plan)

The doc + governance surface that lands when the mechanism is proven:

- **Ratifying ADR** — amend ADR-016 / ADR-010 or a sibling, ratifying the `fulfils` label,
  the `{full, partial}` degree facet, the generalised `originates_from` role, and the
  author-at-mutable-end property (a refinement of ADR-004's outbound-only). This is the
  governance gate RFC-003 deferred.
- **SPEC-018** — the cross-corpus relation contract: record `fulfils` (+ degree facet) and
  the generalised `originates_from`; remove `slices`; note `drift` is now entity-row-empty
  but retained for the deferred non-entity edge. Points at `RELATION_RULES`, never
  transcribes (storage rule).
- **`relation-vocabulary.md`** — the semantic-classes companion: re-class the work→backlog
  edges onto the universal grammar; retire the `slices`/`drift` noun-labels from the
  vocabulary narrative; record the degree facet as the Axis-C completion answer.
- **RFC-003** — **close** it. Per its own disposition ("stays open as the decision-of-record
  until finish-Axis-B's ADR ratifies, then closes"), this slice's ratification is its
  terminal event.

### Discharges

- **IMP-207** — the 19-row provenance retcon rides class 2.
- **IMP-149** — `slices` ambiguity dissolved (provenance / fulfillment / completion split).
- **Axis C completion hole** — the rejected `partially_addresses` predicate lands as the
  `fulfils` degree facet.

---

## Adversarial review (internal pass — integrated)

Hostile self-review of the draft. Substantive findings integrated above; recorded here for
the audit trail.

- **SR-1 — inbound degree render mechanism under-pinned.** Inbound bucket key stays
  `(label, role)` = `(Fulfils, None)`; degree is per-source, fetched at render from
  `inbound_degree_index(source, Fulfils, inspected)`. *Adequate at design altitude* (§A.6);
  plan pins the `Vec<EntityKey>`→degree join.
- **SR-2 — `originates_from` + `fulfils` coexistence.** Both may hold on one (SL, backlog)
  pair (scoped-from *and* does-the-work). *Integrated* (§B.1 coexistence note) — distinct
  labels/directions, no conflict.
- **SR-3 — prov-vs-fulfil triage criterion was unstated** (the crux of the migration).
  *Integrated* (§B.1) — creation-order + intent heuristic, recorded per row.
- **SR-4 — degree default-full could silently mismark partials**, losing the exact signal
  the slice adds. *Integrated* (§B.2) — every class-3 edge affirmatively examined; full is
  the post-examination default.
- **SR-5 — `validate_link` signature ripple** (adds `degree` param; callers update). Real
  but minor seam churn, mirrors SL-149's role add. *Accepted, noted* (§A.6).
- **SR-6 — `append_edge` upsert is a shared-machinery behaviour change** vs the behaviour-
  preservation gate. *Integrated* (R3) — no-op for roleless/degreeless edges → existing
  suites green; superset path exercised only by `fulfils`.
- **SR-7 — retiring `Slices` + adding `Fulfils` shifts enum order / overlay count.**
  Precedented (SL-149 dropped Specs/Requirements, added References); cordage label-keyed,
  `fulfils` = one overlay. *Covered* (VT-1, surfaces).
- **SR-8 — `drift` "feeds into" → `needs`/`after` assumes dep-layer legality** for the
  row's source/target. ~1 row, hand-dispositioned; *per-row legality checked at execution*
  (§B.1 class 5).
- **SR-9 — STD-001 (no magic strings).** Wire strings `"full"`/`"partial"`/`"fulfils"`/
  `"originates_from"` are single-sourced via `name()`/`from_name()` round-trip (mirrors
  `Role`). *Compliant.*
- **SR-10 — author-end convention is not enforceable** → reverse-author hazard. *Integrated*
  as residual risk R9.
- **SR-11 — the design-authored `IMP-210 references(concerns) SL-176` edge is itself
  provenance-shaped.** *Covered* (§B.4 dogfooding) — migration may retcon to
  `originates_from`.
- **SR-12 — the ~19/~63 split rests on a stale IMP-207 count.** *Covered* — re-census live
  at execution; disposition authored then (SL-149 AR-1 discipline).
