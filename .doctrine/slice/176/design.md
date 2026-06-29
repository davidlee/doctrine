# SL-176 Design — Finish Axis B: retire `slices` / `drift`

<!-- Reference forms: entity ids padded (ADR-016, SPEC-018); doc-local refs bare
     (A.1 section, Q1 question, R1 risk, B.3 class). -->

## Status

Design draft for **RFC-003 § "Finish Axis B"** — the unfinished work→backlog half of
the relation-vocabulary collapse SL-149 began for work→canon. The *what* is decided
(RFC-003, direction locked 2026-06-29); this design resolves the **mechanism**.

**Review state (2026-06-29):** revised through external pass 1 (codex F1–F6), then a second
independent external pass (codex G1–G4), then a **design session resolving G1–G4**.
**G1 resolved option (a)** (user-locked) — D-backlog-inbound; **G2 resolved** —
D-uniqueness-seam (`read_block`, local); **G3** re-classed as deliberate content
(enumerate at plan); **G4** census add. Mechanism complete. **Optional: one confirming
third external pass on the G1(a) backlog-inbound mechanism, else lock → `/plan`.** See the
decision ledger + the two "Adversarial review (external pass …)" sections.

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
| **D-edge-identity** | Edge identity = `(label, role, target)`; **degree excluded**. Single-edge guaranteed by a **per-entity `read_block` `DuplicateEdge` check** on `(label, role, target)` (G2 / D-uniqueness-seam), not corpus validate. Degree set at author time; **changed via `unlink`+`relink`**, NOT upsert (codex F1 — no mutation path exists in `append_relation_row`). `link` of an existing triple with a *different* degree **errors** ("exists with degree X; unlink to change"); identical → `Noop`. `unlink` matches `(label, role, target)`, degree ignored. | design (codex F1/F2) |
| **D-inspect-shape** | `RelationGroup` targets change from `Vec<String>` to **structured entries carrying `Option<Degree>`** (codex F3 — the flat `Vec<String>` cannot hold per-target degree). Affects outbound + inbound + `inspect --json` schema. A render-side side-index is insufficient. | design (codex F3) |
| **D-backlog-inbound** | (G1, user-locked **a**) `backlog show`/`show --json` render `fulfilled by` as **derived inbound**, computed via the **same relation-graph inbound machinery `inspect` uses** (`in_edges` + `inbound_role_index`/degree, threaded with `root`) — NOT the item's own outbound (which the migration deletes). `backlog show` thereby becomes **corpus-aware** (was item-local, `backlog.rs:1363-65`); this is a deliberate posture **refinement, ADR-004-consistent** (inbound is always derived; ADR-004 defers the reverse *field*, not derived render). The `slices` outbound read is removed, not swapped. `doctor` (`:2201`) takes the same inbound set. | design (codex G1) |
| **D-uniqueness-seam** | (G2) The `(source, label, role, target)` uniqueness invariant is **per-entity and local** — `source` is one entity, so a duplicate logical edge is two `fulfils` rows in **one slice's own toml**. Enforced in **`read_block`/per-entity row validation** (new `DuplicateEdge` finding, match on `(label, role, target)`, **degree-agnostic**), NOT corpus `validate_relations` (which would need degree threaded into `CatalogEdge`). The write-seam degree-conflict error (§A.5) is the author-time guard; this is the at-rest backstop for hand-authored dupes. | design (codex G2) |

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

> **G3 [MAJOR — RESOLVED as content; enumerate at plan] the source/target widening
> deliberately FLIPS shipped rule-contract
> tests; these are *content*, not "machinery green unchanged" (external pass 2).** Widening
> the source set `[SL]`→`{SL+backlog}` and target `Kinds(BACKLOG)`→`Kinds(BACKLOG+[SL])`
> inverts assertions the current contract pins: `relation.rs:2696-2701` asserts a **backlog
> item CANNOT author `scoped_from`** (err→ok under source-widening); `relation.rs:2743-2748`
> asserts `scoped_from` **refuses a non-backlog target** (the `SPEC` case still refuses, but
> an `SL` target flips err→ok under target-widening); `VT-2 sources_match_shipped_accessors`
> (`relation.rs:1457-1477`) pins the exact per-label source set and **must** churn. §C frames
> these under "machinery stays green unchanged" — wrong: they are **deliberate
> rule-contract content changes** and belong in the machinery-vs-content split (R5) on the
> *content* side, enumerated and rewritten, not discovered as regression. No mechanism
> breakage; a verification-accounting correction. Re-class at design, list at plan.

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
`fulfils` edge per (slice, item); you do not fulfil the same item both full and partial.
**Single-edge is guaranteed by a per-entity `read_block` uniqueness check** (G2 /
D-uniqueness-seam — `read_block` legalises rows independently and never detects duplicate
logical edges, so absent the check two `fulfils` rows with differing degree could coexist and
the inbound index would pick an arbitrary winner). The check is **local** (identity's `source`
is one entity → dupes live in one toml): a new **`DuplicateEdge`** finding on a repeated
`(label, role, target)`, degree-agnostic — NOT corpus `validate_relations`. Mechanics — **no upsert**
(codex F1: `append_relation_row` is append-or-`Noop`, `relation.rs:911/949`; there is no
mutation path, and inventing one is real seam work, not a payload thread):

- `append_relation_row` noop-guard matches `(label, role, target)`: identical (incl.
  degree) → `Noop`; **same triple, different degree → hard error** ("already `fulfils` X
  with degree=full; `unlink` to change") — a new `AppendOutcome`/error branch, the one
  honest extension to the seam; else `Wrote`.
- **Degree set at author time; changed via `unlink` + `relink`.** Aligns with the discipline:
  degree is set once at slice close; partial→full is a *new slice's* edge, never an edit to
  a closed slice's edge (RFC-003).
- `unlink` matches `(label, role, target)`, degree ignored.
- `read_block` parses the optional `degree` and **enforces the `DuplicateEdge` uniqueness
  check** (G2 / D-uniqueness-seam — per-entity, degree-agnostic match on `(label, role, target)`).

> **G2 [MAJOR — RESOLVED, see D-uniqueness-seam] the uniqueness invariant lives in
> per-entity `read_block` validation, not corpus `validate_relations`.** `validate_relations`
> (`relation_graph.rs:341+`) today emits only dangler + corruption findings and `CatalogEdge`
> carries no degree — but it doesn't need to: identity is `(source, label, role, target)` and
> `source` is **one entity**, so a duplicate logical edge is two `fulfils` rows in **one
> slice's own toml** — detectable **locally at `read_block`** with no corpus scan and no
> degree thread into `CatalogEdge`. New **`DuplicateEdge`** finding, match on
> `(label, role, target)` **degree-agnostic** (degree is excluded from identity by design).
> The write-seam degree-conflict error (§A.5) guards author-time; `read_block` is the at-rest
> backstop for hand-authored dupes. Finding text/category pinned at plan.

### A.6 Functions re-keyed

- `lookup` — **unchanged** (degree not a key; only `role` keys the rule, as SL-149).
- `validate_link(source, label, role, degree)` — new **`DegreeNotApplicable`** (degree given
  on a non-`degree_bearing` label, e.g. `references --role concerns … --degree full`),
  symmetric to SL-149's `RoleNotApplicable`. **No `MissingDegree`** (absent ≡ full).
  Target-kind mismatch still refused via `check_target_kind` against the role-keyed
  `TargetSpec` (now widened for `originates_from`).
- **Graph/projection — a data-model change, not a render-side index (codex F3).** Today
  `RelationGroup = ((RelationLabel, Option<Role>), Vec<String>)` (`relation_graph.rs:521`);
  both `InspectView.outbound` and `.inbound` flatten targets to **bare `Vec<String>`** with
  no per-target metadata slot. A side `inbound_degree_index` would (a) only reach the human
  *inbound* render, leaving *outbound* `fulfils (partial)` and `inspect --json`
  unexpressible, and (b) on a duplicate row pick an arbitrary winner (F2). **So the target
  representation itself changes:**
  ```rust
  struct RelationTargetView { target: String, degree: Option<Degree> }   // NEW
  type RelationGroup = (RelationKey, Vec<RelationTargetView>);            // was Vec<String>
  ```
  - Outbound: `outbound_for` already carries `degree` on the edge (A.5) → populate
    `RelationTargetView.degree` (`inspect_from` `relation_graph.rs:619-626`).
  - Inbound: recover degree from the SOURCE entity's outbound payload, exactly as role is
    recovered via `inbound_role_index` (`relation_graph.rs:633-665`) — extend that index (or
    a parallel `inbound_degree_index`) to also carry degree, attached per-source.
  - `render_outbound`/`render_inbound` (`relation_graph.rs:~756/797`) suffix `(partial)` per
    target where degree is `Some(Partial)`; `None`/`Full` render bare (no golden churn for
    non-degree groups).
  - **`inspect --json` schema decision (D-inspect-shape):** degree-bearing groups
    (`fulfils`) emit target entries as objects `{ "ref": "...", "degree": "partial" }`
    (degree `skip_if None` ⇒ full omitted); all other groups keep bare-string targets
    (heterogeneous-by-label, keyed by the group label — avoids churning every inspect-json
    golden). Enumerate the affected goldens at plan.
  - **Overlay allocation stays label-keyed** — `fulfils` is one overlay-backed label;
    cordage stays vocabulary-unaware (ADR-016 §3, R8). Grouping key stays `(label, role)`;
    degree rides per-target within the group, not as a grouping dimension (Q3).

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

## Section A′ — Live consumers of the retired labels (codex F4/F5 — blast radius)

Retiring `slices` and renaming `scoped_from` is **not** vocab-table-local: shipped
consumers read both. These are **in scope** (re-pointed in this slice), not deferrable to
IMP-210 — that item is only the *new* close-cascade hint, not the *existing* behaviour.

### A′.1 `RelationLabel::Slices` consumers → re-point at `fulfils`

| consumer | file:line | today | becomes |
|---|---|---|---|
| **priority scoring** | `src/priority/graph.rs:190, 201` | `Slices` is in **both** the reference-label and consequence-label sets — a backlog item's "optionality" credit counts the slices that reference it | the optionality signal moves to the **derived `fulfils` inbound** (a slice `fulfils` the item). Re-point both label-set memberships from `Slices` to `Fulfils`; preserve the scoring behaviour (behaviour-preservation on the *numbers*, not the label). |
| **backlog show (human)** | `src/backlog.rs:1420, 1443` | `targets_for(tier1, Slices)` → a `("slices", …)` line | render the derived **`fulfilled by`** inbound instead (the `slices` *outbound* row no longer exists post-migration). |
| **backlog show (JSON)** | `src/backlog.rs:1574` | `"slices": targets_for(…, Slices)` field | replace with the `fulfils`-derived shape; **public JSON schema change** — enumerate goldens at plan. |
| **lifecycle findings** | `src/backlog.rs:2201` (doctor "all linked slices terminal") | reads `Slices`-linked slices | read the `fulfils`/derived-inbound set. Keep the finding's semantics; swap the edge it reads. |

> **G1 [BLOCKER — RESOLVED (a), see D-backlog-inbound] the backlog re-point is NOT a
> label swap; it is an inbound-derivation read-path `backlog show` structurally lacks.**
> `format_show`/`format_json` are documented as a **pure fn of the item's OWN tier1**,
> explicitly *not* inbound — "the reverse view is the deferred registry surface's,
> ADR-004" (`src/backlog.rs:1363-1365`); the three reads (`backlog.rs:1420` human,
> `:1574` json, `:2201` lifecycle) all call `targets_for(&item.tier1, Slices)` on the
> item's *own outbound* rows. **Post-migration the `slices` outbound row no longer
> exists on the backlog item** — `fulfils` is authored at the *slice* end and surfaces
> on the item only as **derived inbound**. So "render the derived inbound instead"
> requires giving `backlog show` an inbound scan/index it does not have, and **reverses
> the ADR-004 posture this surface deliberately holds** (backlog show does not compute
> inbound by design). **RESOLVED — option (a)** (D-backlog-inbound): `show`/`json` gain
> the inbound derivation via the **same relation-graph machinery `inspect` uses**
> (`in_edges` + `inbound_role_index`/degree, threaded with `root`); the `slices` outbound
> read is **removed**, not swapped; `fulfilled by: <SL> (degree)` renders from derived
> inbound. **ADR-004-consistent** — inbound is always derived; the ADR defers the reverse
> *field*, not a derived render. `backlog show` becomes **corpus-aware** (deliberate
> refinement of the `:1363-65` item-local posture, recorded). `doctor` (`:2201`) takes the
> same inbound set. Exact wiring (reuse `InspectView` inbound vs a focused `fulfils`-inbound
> query) pinned at plan.

The priority re-point (A′.1 row 1) is the load-bearing one: it must keep the *scoring
numbers* identical (a fulfils inbound credits optionality exactly as a `slices` reference
did), proven by the priority suite staying green on equivalent fixtures (fixtures re-authored
from `slices=[…]` to a `fulfils` edge).

### A′.2 `scoped_from` → `originates_from` output surfaces

The role rename moves named output fields + CLI text, each golden-load-bearing:

| surface | file:line |
|---|---|
| slice show (human) | `src/slice.rs:1677` (`"references(scoped_from)"` label + `targets_for_role(References, ScopedFrom)`) |
| slice show (JSON) | `src/slice.rs:1758` (`"scoped_from":` field) |
| backlog show (human) | `src/backlog.rs:1428, 1447` |
| backlog show (JSON) | `src/backlog.rs:1581` (`"scoped_from":` field) |
| CLI `--role` help (clap) | `src/commands/cli.rs:552` (the role-name doc) |
| CLI unknown-role diagnostic | `src/commands/relation.rs:42-47` (G4 — the runtime parse-error string hardcodes `scoped_from`, **distinct** from the clap help above; missed by the first pass) |

All become `originates_from`. The JSON field renames are **public schema changes** — list
the affected `show --json` goldens at plan; the human-render goldens move with the wording
(R6).

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

**Oracle scope — honest framing (codex F6).** The automated oracle proves the migration
**faithfully APPLIES the recorded dispositions** (mechanical correctness), NOT that the
dispositions are *correct* (the prov-vs-fulfil + degree judgement). The latter is human and
is reviewed separately — the disposition artifact is itself the adversarial-review subject,
per-row rationale committed as evidence; the automated test cannot self-certify it (it would
be checking the rewrite against its own input). So:
- (1) **mechanical:** each row lands at exactly the `(label, role|degree)` its disposition
  records (deterministic class 1 emits its planned map; classes 2/3/4/5 assert against the
  artifact); (2) **class-aware** edge-count + `(source,target)` multiset (classes 1/2/4
  preserve the pair; class 3 flips source↔target; class 5 moves label-space; class 6
  unchanged) — *secondary* sanity; (3) `validate` clean (incl. the new `fulfils` uniqueness
  invariant). The multiset check **cannot** detect a misclassification (a wrong prov-vs-fulfil
  call preserves the pair) — that is what the human review of the disposition artifact is for;
  the design does not claim otherwise. Call it an **editorial migration with a mechanical
  faithfulness oracle**, not an oracle-validated classification.

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
byte-identical. **Caveat (G3):** the `originates_from` source/target *widening* also flips
**rule-contract tests** (`relation.rs:2696`/`:2743`/VT-2 accessor census `:1457`) — those
sit on the *content* side of this split (deliberate change), despite exercising machinery;
they do **not** stay green-unchanged.

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
- **Validation:** `DegreeNotApplicable` (degree on non-`degree_bearing` label); **`fulfils`
  `DuplicateEdge`** — a second `fulfils` row for an existing `(source, target)` in one
  entity's toml flagged at `read_block`, degree-agnostic (codex F2 / G2 / D-uniqueness-seam); `link` of an existing triple with a different degree errors (codex F1); corpus
  `validate` clean post-migration (no `IllegalRow`, no dangler regression).
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
- **Surfaces:** `inspect` mixed (fulfils+degree as JSON target objects, originates_from,
  label-only) golden + the structured-target schema (D-inspect-shape); `relation
  list`/`census` (`slices` gone, `fulfils`+degree present, `scoped_from`→`originates_from`);
  web-graph edge label; slice/backlog `show --json` field renames (`scoped_from`→
  `originates_from`; `slices`→`fulfils`-derived) — enumerate goldens at plan.
- **Consumer re-point (A′ — codex F4/F5):** `priority/graph.rs` suite stays green on
  fixtures re-authored from `slices=[…]` to a `fulfils` edge (optionality numbers identical
  — the load-bearing behaviour-preservation proof for the re-point); `backlog` show/JSON +
  lifecycle-findings goldens move to the `fulfils`-**derived-inbound** shape (G1(a) — new
  test: a slice authors `fulfils IMP-x --degree partial` → `backlog show IMP-x` renders
  `fulfilled by: SL-… (partial)` from derived inbound, item's own toml carrying no `fulfils`
  row); `slice`/`backlog` `scoped_from`→`originates_from` field/wording goldens.
- **Rule-contract churn (G3 — deliberate content):** widened `originates_from` flips
  `relation.rs:2696` (backlog may now author it), revises the `:2743` target assertion (SL
  legal, SPEC still refused), and churns VT-2 accessor census `:1457`. Enumerate + rewrite at
  plan; these are content-side, not behaviour-preservation regressions.
- **Determinism:** BTree ordering only; canonical `(label, role, degree)` order =
  declaration order; no `HashMap` on the relation path (REQ-077).

---

## Section D — Phasing, risks, carried opens

### Phasing sketch (plan refines; **no ADR phase** — ratification deferred to reconcile)

1. **P1** — `Degree` enum + `Role` rename + `RelationLabel::Fulfils` + rules
   (`fulfils`, widened `originates_from`) + `degree_bearing` column + lockstep VT tests.
   Leaf/engine. `Slices` retained.
2. **P2** — storage (`RelationEdge`/`RelationRow`/`read_block` incl. the `DuplicateEdge`
   uniqueness check (G2) / `append` degree-conflict-error / `remove`) + `validate_link`
   `DegreeNotApplicable` + `check_target_kind` for widened `originates_from`.
3. **P3** — surfaces + **consumer re-point (A′)**: `RelationTargetView` structured targets
   (`InspectView` outbound+inbound, `render_*`, `inspect --json` schema), `CatalogEdge`
   degree **only if a catalog/census consumer needs it** (the G2 uniqueness check does NOT —
   it is `read_block`-local), `relation list`/`census`, web graph, `link --degree`; **re-point
   `priority/graph.rs` from `Slices`→`fulfils`**; **`backlog.rs` show/JSON/lifecycle gain the
   derived `fulfils`-inbound read-path (G1(a) / D-backlog-inbound — `inspect`-style inbound
   derivation, not the deleted `slices` outbound)**; `scoped_from`→`originates_from` output
   fields in `slice.rs`/`backlog.rs`/`cli.rs`/`commands/relation.rs`.
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
- **R10 (codex F4)** — the `priority/graph.rs` re-point is the highest-risk consumer change:
  a wrong move silently shifts work-ordering scores. Mitigation: the optionality numbers must
  stay identical (fulfils inbound credits exactly as a `slices` reference did), proven by the
  priority suite green on fixtures re-authored `slices`→`fulfils`. NOT a behaviour change to
  scoring, only to the edge it reads.
- **R11 (codex F3)** — `RelationGroup` target-type change (`Vec<String>`→`Vec<RelationTargetView>`)
  touches every inspect render/JSON path; mitigated by degree `skip_if None` (degreeless
  groups render/serialise byte-identically) — but the type signature ripples; machinery-vs-
  content split must distinguish the type-thread (machinery, behaviour-preserving for None)
  from the fulfils content.
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

---

## Adversarial review (external pass — codex/GPT-5.5, integrated)

Hostile external review of the committed draft. All six findings verified against the live
source before integration; none overturned a locked decision (the *what*), all corrected the
*mechanism / blast radius*.

- **F1 [BLOCKER] "upsert degree" not grounded in the write seam** — `append_relation_row` is
  append-or-`Noop` (`relation.rs:911/949`), no mutation path. *Fixed:* D-edge-identity +
  §A.5 — no upsert; degree set at author time, changed via `unlink`+`relink`; `link` of an
  existing triple with a different degree **errors** (one new branch, not a redesign).
- **F2 [BLOCKER] degree-as-non-keyed payload is lossy without an invariant** — `read_block`
  legalises rows independently; duplicate `fulfils` with differing degree → arbitrary winner.
  *Fixed:* §A.5 + §C — uniqueness check on `(source, fulfils, target)` (later refined by G2 →
  per-entity `read_block` `DuplicateEdge`, not corpus `validate`).
- **F3 [BLOCKER] `inbound_degree_index` structurally insufficient** — `RelationGroup` flattens
  to `Vec<String>` (`relation_graph.rs:521`); cannot carry per-target degree on outbound or
  `--json`. *Fixed:* D-inspect-shape + §A.6 — target type becomes `RelationTargetView { target,
  degree }`; JSON degree-bearing groups emit objects; degreeless groups byte-identical.
- **F4 [BLOCKER] live `slices` consumers missed** — `priority/graph.rs:190/201` (optionality
  scoring), `backlog.rs` show/JSON/lifecycle. *Fixed:* new **§A′.1** — re-point all at
  `fulfils`; priority numbers held identical (R10); brought into scope + selectors.
- **F5 [MAJOR] `scoped_from` rename understated** — hardcoded in slice/backlog show+JSON +
  `cli.rs:552`. *Fixed:* new **§A′.2** — enumerated output surfaces; public JSON field renames.
- **F6 [MAJOR] migration oracle not independent** — certifies the rewrite against its own
  handwritten judgement. *Fixed (framing):* §B.2 — reframed as a **mechanical faithfulness
  oracle** (proves dispositions are *applied*, not *correct*); the classification is human,
  reviewed via the disposition artifact. No false oracle claim.

Codex verdict was "return to design"; the return is this revision (mechanism + blast-radius
corrections), not a re-decision of the locked direction.

---

## Adversarial review (external pass 2 — codex/GPT-5.5, integrated)

Second **independent** hostile pass on the F1–F6 revision (fresh thread; reviewer did not
see pass 1). Attacked the mechanism the revision *introduced* (no-upsert, `RelationTargetView`,
the A′ blast radius, the widening). All four findings verified against live source before
integration; none overturns a locked decision. Verdict: **RETURN-TO-DESIGN** on G1.

- **G1 [BLOCKER, NEW] backlog re-point is an inbound read-path `backlog show` lacks, not a
  label swap.** `format_show`/`format_json` are pure-on-own-tier1 by design, inbound
  explicitly deferred to the registry surface (ADR-004) — `backlog.rs:1363-1365`; the three
  reads (`:1420`/`:1574`/`:2201`) read the item's *own outbound* `slices`, which the
  migration deletes. Rendering `fulfilled by` = derived inbound the surface cannot compute
  + an ADR-004 posture reversal. *Integrated* §A′.1 (G1 callout). **Open — resolution
  required: (a) give show/json a scan-derived inbound, or (b) drop the line, defer to
  `inspect`.** Drives the verdict.
- **G2 [MAJOR, NEW — refines F2] the `(source,fulfils,target)` uniqueness invariant has no
  home in the validator seam.** `validate_relations` (`relation_graph.rs:341+`) reports only
  danglers + corruption; no duplicate-edge class, and `CatalogEdge` carries no degree.
  *Integrated* §A.5 (G2 callout) — needs a named finding class + a seam choice
  (`validate_relations` with degree threaded vs `read_block`/storage validation).
- **G3 [MAJOR, NEW] the `originates_from` source/target widening flips shipped rule-contract
  tests, mis-framed as "machinery green unchanged."** `relation.rs:2696-2701` (backlog can't
  author scoped_from), `:2743-2748` (non-backlog target refused), `VT-2` accessor census
  `:1457-1477`. *Integrated* §A.2 (G3 callout) — re-classed as deliberate *content* change
  (R5 split), enumerated at plan. No mechanism breakage.
- **G4 [MINOR, residue of F5] the unknown-role diagnostic string still hardcodes
  `scoped_from`.** `commands/relation.rs:42-47` — distinct from the `cli.rs:552` clap help the
  first pass named. *Integrated* §A′.2 (new row).

**Disposition (resolved in the same-day design session).** **G1 → option (a)**, user-locked
(D-backlog-inbound): backlog show/json gain the `inspect`-style derived inbound; ADR-004-
consistent; backlog show becomes corpus-aware. **G2 → D-uniqueness-seam**: per-entity
`read_block` `DuplicateEdge` finding, degree-agnostic, no corpus scan / no `CatalogEdge`
degree thread. **G3 → content**: the flipped rule-contract tests enumerated at plan (§C
caveat). **G4 → §A′.2** row. Mechanism now complete; lifecycle stays `design` pending an
optional confirming third pass on the G1(a) read-path, else lock.
