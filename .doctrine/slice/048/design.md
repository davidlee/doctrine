# Design SL-048: Structural cross-corpus relation edges — the `link` writer, uniform `[[relation]]` storage, and the relation contract

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-048, ADR-010, REQ-074); doc-local refs bare — D1 (§Decisions), R1
     (§Risks), C1 (§Open). RelationLabel wire names in `code font`. -->

## 1. Design Problem

Cross-corpus relations are **prose-only** where the capture surface is missing,
and **inertly stored** where it exists. SL-046 shipped the *reader* (the graph
spine: `outbound_for` → `RelationEdge{label,target}`, derived inbound) and SL-047
the *rank*. SL-048 is the third and final slice: it **mints the missing authored
edges and the cross-kind writer** so the connective tissue is structured,
validated, and queryable — born live, not inert, because the reader already
exists (the anti-inert ordering).

The forcing decision — *which kinds may link to what, the edge semantics, the
storage shape, validation policy, composition with ADR-004 outbound-only* — was
taken in **ADR-010 (accepted)**. SL-048 *implements* ADR-010 and, per the design
thrash recorded below, carries its opportunistic tier-1 migration (D3)
**corpus-wide** rather than per-kind.

## 2. Current State (what shipped, what's missing)

SL-046 shipped, in `src/relation.rs` + `src/relation_graph.rs`:

- `RelationLabel` enum (13 variants) + `name()` wire form — `src/relation.rs:27`.
- `RelationEdge{label, target}` — `src/relation.rs:86`.
- `outbound_for(root, kind, id)` cross-kind dispatch — `src/relation_graph.rs:39`.
- per-kind `relation_edges()` **read** accessors: slice/spec/governance/backlog/
  review/rec.

Bespoke per-kind `[relationships]` storage (ADR-010 survey):

- **slice** — `{specs, requirements, supersedes}` (`Vec<String>`).
- **governance** (ADR/POL/STD) — `{supersedes, superseded_by, related, tags}`.
- **backlog** — `{slices, specs, drift, needs}` + `after{to,rank}` + `triggers{globs,note}`.
- **spec** — lineage `descends_from`/`parent` (`Option`, arity ≤1); `members`
  (members.toml); `interactions` (typed `[[edge]]`, free-text `type`).
- **review** — `[target].ref` (arity 1). **rec** — `owning_slice` (arity ≤1),
  `decision_ref` (free-text).

**Missing (= SL-048 build surface):**

| Need | Today | Gap |
|---|---|---|
| Legal-set vocab table `(src,label)→targets` | **does not exist** | the ADR-010 D2 spine |
| `link`/`unlink` write verb | only `spec req add` | cross-kind writer |
| slice→governance edge (IMP-035) | no field | `governed_by` |
| spec↔ADR edge | no field | `governed_by` |
| product↔product (PRD↔PRD) | no field | `related` |
| forward-edge validation at write | `ensure_ref_resolves` exists | wire writer through it |
| uniform tier-1 storage | bespoke per kind | `[[relation]]` corpus-wide |

No supersession verb exists — `superseded_by` is read/displayed but never
verb-written (scaffold seeds only `supersedes = []`). The reverse carve-out is a
**pre-existing gap (IMP-032), out of SL-048 scope.**

## 3. Forces & Constraints

- **ADR-010** — the governing decision: unify the relation contract + cross-kind
  write seam; tier partition (tier-1 clean multi-ref / tier-2 constrained-or-payload
  / tier-3 free-text); code-authoritative vocabulary over `integrity::KINDS`;
  opportunistic tier-1 storage migration (D3); `superseded_by` is the kept ADR-004
  §5 carve-out (D4); outbound-only enforced in code (D5); identifiers not bound (D7).
- **ADR-004** — relations stored **outbound-only**; reciprocity **derived**, never
  authored on the target. §5 carve-out (`superseded_by`) verb-written only.
- **ADR-001** — module layering leaf ← engine ← command. Vocabulary + accessors
  are leaf/engine, the verb is command.
- **Storage rule** — never queried/derived data in prose. The `RELATION_RULES`
  table is **code-authoritative**; the contract spec describes semantics + policy
  and *points at* the code, never mirrors the enumeration.
- **Behaviour-preservation gate** — `backlog order` byte-identical; existing
  per-kind + cordage + backlog_order suites stay green. SL-046 reader output for
  already-authored edges unchanged post-migration (same edges, new on-disk shape).
- **Determinism** — no clock/RNG/`HashMap` iteration order; BTree only.
- **No parallel implementation** — ride `ensure_ref_resolves`, the edit-preserving
  toml_edit seam (`backlog::append_relationship`, `governance::set_status`),
  `integrity::KINDS`, the SL-046 read seam.

## 4. Guiding Principles

- **Corpus-wide uniformity for tier-1; typed guarantees kept for tier-2/3.** One
  on-disk idiom for clean multi-ref edges; arity/payload/free-text edges stay typed
  (genericizing them is flattening, not consistency — ADR-010 rejected alt A).
- **The graph layer is already uniform.** `RelationEdge` normalises every edge
  before cordage; storage shape never reaches the graph. Uniform storage buys
  *writer/reader code* simplicity, not graph correctness.
- **One source of truth.** `RELATION_RULES` drives writer, validator, and the
  SL-046 reader in lockstep — a test forbids divergence (ADR-010 verification).
- **Write-strict, read-tolerant.** The writer refuses to *create* a dangling
  numbered-kind edge; the reader/`validate` *tolerate* danglers that arise later.
- **Outbound-only, in code.** The vocabulary admits outbound labels only; no
  inverse/derived label is authorable. Inbound stays derived from `in_edges`.

## 5. Proposed Design

### 5.1 Storage shape — uniform `[[relation]]`, corpus-wide tier-1

Tier-1 edges move to a single on-disk idiom; tier-2/3 stay typed:

```toml
[[relation]]
label  = "supersedes"
target = "ADR-003"
[[relation]]
label  = "related"
target = "ADR-004"
```

`label` (not ADR-010's prose `kind=` — collides with `entity::Kind`; D7 leaves
identifiers open). Multiple same-label edges = multiple rows. `link` appends one
row; `unlink` removes one.

**Migration inventory** — corpus-wide (the design-thrash decision; ADR-010 D3
taken corpus-wide, recorded in the ADR amendment, deliverable 8):

| Kind | → `[[relation]]` (tier-1) | stays typed |
|---|---|---|
| governance (ADR/POL/STD) | `related` | `supersedes` + `superseded_by` (the supersession **pair**, OD-3/X7), `tags` (classification) |
| slice | `specs`, `requirements`, `supersedes`, **+`governed_by`** | — |
| backlog | `slices`, `specs`, `drift`¹ | `needs`, `after{to,rank}`, `triggers{globs,note}` (SL-047 owns) |
| spec | **+`governed_by`**, **+`consumes`** (PRD↔PRD) | `descends_from`, `parent` (arity≤1), `interactions` (free-text+payload), `members` (members.toml) |

¹ `drift` migrates shape; target free-text-unvalidated (no `DRIFT` kind).

**OD-3/X7 — governance supersession excluded from migration.** `supersedes` and
`superseded_by` stay typed **together**: the forward edge is meaningless to
migrate while its sanctioned reverse carve-out (`superseded_by`, ADR-004 §5) has
no transactional owning verb — that verb is IMP-006's cross-kind lifecycle axis,
not SL-048's (building it gov-only here = parallel implementation). Only
governance `related` migrates. Slice `supersedes` (no carve-out partner) migrates
normally.

**Governance worked example** (spans all cases):

```toml
# BEFORE                          # AFTER
[relationships]                   [relationships]                 # typed leftovers FIRST (F1)
supersedes    = ["ADR-003"]       supersedes    = ["ADR-003"]     # OD-3: supersession pair NOT migrated
related       = ["ADR-004"]       superseded_by = ["ADR-009"]     # ADR-004 §5 carve-out (its pair)
superseded_by = ["ADR-009"]       tags          = ["layering"]    # classification, NOT a relation
tags          = ["layering"]
                                  [[relation]]                    # tier-1 array-of-tables, at EOF
                                  label  = "related"              # only `related` migrates
                                  target = "ADR-004"
```

**F1 — TOML ordering is load-bearing.** Bare keys (`superseded_by`, `tags`) after
an array-of-tables header bind to the *last `[[relation]]` table*, not top-level —
silent corruption (the `edit-preserving-status-transition` tail-insert trap).
Typed leftovers therefore stay in a `[relationships]` **table that precedes** the
`[[relation]]` arrays; `append_edge` always appends arrays at **EOF**, which is
unconditionally valid. A migrated kind with no typed leftovers (slice) drops
`[relationships]` entirely and carries only `[[relation]]`.

The supersession pair (`supersedes` + `superseded_by`, OD-3) and `tags` stay typed
beside the block. The `[[relation]]` array admits **outbound validated labels
only**, so `superseded_by` is structurally un-authorable there (ADR-010 D4/D5
satisfied by construction). spec legitimately carries both idioms — it spans tier-1 (new edges) *and* tier-2
(lineage/interactions); ADR-010-sanctioned, not the bad same-tier mix.

**Migration mechanism — out-of-band, no shipped surface.** doctrine is
dogfood-only today, so there is no client back-compat obligation. Parsers cut
**hard** to `[[relation]]` (no dual-read branch, no shipped migrate verb). The
one-time corpus rewrite is a throwaway script/LLM pass over this repo's TOML,
run once, verified by round-trip `doctrine <kind> show` + `validate` across the
whole corpus before commit. Cleanest final shape, zero permanent migration code.

### 5.2 The legal-set vocabulary table (ADR-010 D2 spine)

One const table in `src/relation.rs` (beside `RelationLabel`):

```rust
enum TargetSpec { Kinds(&'static [Kind]), AnyNumbered, Unvalidated } // F4: AnyNumbered for RV reviews; Unvalidated = free-text
enum Tier       { One, Typed }                               // One → [[relation]]; Typed → bespoke
enum LinkPolicy { Writable, LifecycleOnly, TypedVerbOnly }   // does `link` admit it

struct RelationRule {
  sources:      &'static [Kind],   // F2: source-set, not one row per kind
  label:        RelationLabel,
  inbound_name: &'static str,      // X5: derived-inbound display ("governed_by" → "governs")
  target:       TargetSpec,
  tier:         Tier,
  link:         LinkPolicy,
}
const RELATION_RULES: &[RelationRule] = &[ … ];   // lookup keyed by (source ∈ sources, label)
```

Five axes, all from this one table: `target` → forward validation; `tier` →
storage shape; `link` → whether the verb admits the triple; `inbound_name` → how
the derived reciprocal renders on the target (X5 — generalises the `supersedes` →
"superseded by" special-case in `relation_graph.rs`, which today is the *only*
inverted label; every other inbound currently renders its raw outbound name,
which is backwards for asymmetric labels).

**This table is the sole driver of FIVE consumers, asserted by EXACT coverage
(X3), not subset:** (a) the `read_block` parser's per-kind legality; (b) the
`link`/`unlink` writer dispatch; (c) forward-edge validation; (d) the SL-046
reader's emitted edges; (e) **cordage overlay allocation** (today hand-maintained
at `relation_graph.rs:117` — the drift source X3 names). A test asserts every
`Writable`/reader-reachable rule has an overlay and a reader path, and that no
reader emits a label absent from the table. Subset (⊆) is insufficient: it cannot
catch a rule with no overlay (silent dangler) or a source-legality mismatch.

**Full vocabulary** (★ = new in SL-048):

| label | source(s) | target | tier | `link`? |
|---|---|---|---|---|
| `specs` | SL, backlog | PRD·SPEC | 1 | Writable |
| `requirements` | SL | REQ | 1 | Writable |
| `supersedes` | SL | SL | 1 | Writable |
| `supersedes` | ADR·POL·STD | same-gov | 1* | LifecycleOnly — storage excluded (OD-3); verb → IMP-006 |
| `governed_by` ★ | SL·PRD·SPEC | ADR·POL·STD | 1 | Writable |
| `related` | ADR·POL·STD | same-gov | 1 | Writable |
| `consumes` ★ | PRD | PRD | 1 | Writable |
| `slices` | backlog | SL | 1 | Writable |
| `drift` | backlog | *free-text* | 1 | Writable (unvalidated) |
| `descends_from` | SPEC | PRD | 2 (arity≤1) | TypedVerbOnly |
| `parent` | SPEC | SPEC | 2 (arity≤1) | TypedVerbOnly |
| `members` | SPEC | REQ | 2 (members.toml) | TypedVerbOnly (`spec req add`) |
| `interactions` | SPEC | SPEC | 2 (free-text+payload) | TypedVerbOnly |
| `reviews` | RV | any | 2 (arity1 `[target]`) | TypedVerbOnly (`review`) |
| `owning_slice` | REC | SL | 2 (arity≤1) | TypedVerbOnly |
| `decision_ref` | REC | *free-text* | 3 | TypedVerbOnly |

`1*` — tier-1 *by shape*, but **storage excluded** from migration (OD-3): stays
typed with its `superseded_by` carve-out pair until IMP-006 builds the
transactional supersede verb. Never `link`-writable (`LifecycleOnly`).

**Label naming.** `governed_by` — one shared label for SL→gov and SPEC/PRD→gov
(one overlay, as `supersedes` already spans SL+gov); reads right on the source
("this slice is governed by ADR-010"); inbound renders via `inbound_name` =
"governs" (X5). `consumes` (OD-1) for PRD→PRD — **its own** label/overlay (fixes
X4, no overlay-model change): "PRD-011 **consumes** a seam/contract PRD-009
exposes" (consumer → provider, directional); inbound on the provider renders
`consumed_by`. Chosen over `builds_on`/`related` for crispness — it names the
seam/interface consumption the PRDs actually describe, not vague adjacency, and
does **not** collide with the work-item `depends_on`/`needs` axis (SL-047).

### 5.3 The seam — the uniformity dividend

Because tier-1 storage is now uniform, the ADR-010 per-kind *write* accessors
collapse to **one generic function each** for tier-1:

```rust
relation::append_edge(root, source_kind, id, label, target) -> Result<Wrote|Noop>  // idempotent
relation::remove_edge(root, source_kind, id, label, target) -> Result<Removed|Absent>
relation::read_block(source_kind, toml) -> (Vec<RelationEdge>, Vec<IllegalRow>)     // X2: source-kind-aware
```

Path resolved by kind via `integrity::KINDS`. The SL-046 per-kind `relation_edges`
accessors **shrink**: tier-1 becomes the shared `read_block`; each kind's accessor
keeps only its **tier-2/3** typed edges (lineage, members, interactions, reviews,
owning_slice). Net: less per-kind code, no per-kind write code for tier-1.

**X2 — `read_block` takes the source kind and enforces legality.** Generic
storage must not mean a generic *parser that emits anything*. Today a slice
**cannot** emit `related` and a backlog item **cannot** emit `governed_by` —
that legality lives in code shape, and hand-edited authored TOML is part of the
model (read-tolerant). So `read_block(source_kind, …)` checks each row's
`(source_kind, label)` against `RELATION_RULES`: legal rows → `RelationEdge`s;
illegal rows → `IllegalRow` **validation findings, never live graph edges**. The
generic seam must preserve the per-kind legality the hardcoded readers had for
free.

**X1 — emitted order is axis-major, not storage order.** `read_block` groups and
emits edges in **canonical label order** (the `RELATION_RULES` order for that
source kind), *independent of `[[relation]]` row order on disk*. This is
load-bearing: SL-046's reader contract is axis-major and byte-pinned (slice
`specs→requirements→supersedes` `slice.rs:1182`; governance `supersedes→related`
`governance.rs:219`; backlog `slices→specs→drift` `backlog.rs:761`; spec
`descends_from→parent→members→interactions` `spec.rs:506`; goldens at
`relation_graph.rs:725`). If emitted order tracked storage order, the first
`append_edge`-at-EOF would reorder edges and break the goldens. Decoupling emit
order from storage order also fixes the tier-1/tier-2 **merge order** (F5/R2):
each accessor concatenates `read_block`'s axis-ordered tier-1 edges, then its
typed tier-2/3 edges, in the kind's pinned axis sequence.

### 5.4 The `link` / `unlink` verbs (command layer)

```
doctrine link   <SOURCE-ID> <LABEL> <TARGET>     # doctrine link SL-048 governed_by ADR-010
doctrine unlink <SOURCE-ID> <LABEL> <TARGET>
```

`<TARGET>` (not `<TARGET-ID>`, X8) — canonical ref for `Kinds`/`AnyNumbered`
labels, free text for `Unvalidated` (`drift`); help text documents both. Positional
triple mirrors `(source, label, target)`. Dispatch:

1. Parse `<SOURCE-ID>` → `(Kind, id)` via `integrity::parse_canonical_ref`.
2. Look up `(Kind, label)` in `RELATION_RULES`. Absent → refuse, list legal labels
   for the source. `link ≠ Writable` → refuse, name the owning verb
   (`LifecycleOnly`/`TypedVerbOnly`).
3. Validate `<TARGET>` (§5.5).
4. `append_edge` / `remove_edge` — edit-preserving toml_edit, idempotent (no-op if
   present / absent).

`unlink` folded in — symmetric on the same seam, near-free.

### 5.5 Forward-edge validation — write-strict, read-tolerant

- **`link` write:** `TargetSpec::Kinds` → target **must** resolve via
  `integrity::ensure_ref_resolves`, else hard-refuse (never create a dangler);
  also assert the target's kind is in the legal set. `TargetSpec::Unvalidated`
  (`drift`, `decision_ref`) → accept free-text as-is.
- **`validate` corpus check:** report (never rewrite — the reseat precedent) both
  (a) `[[relation]]` danglers that arise later (target deleted post-authoring) and
  (b) **`read_block` `IllegalRow`s** — hand-edited rows whose `(source,label)` is
  not in `RELATION_RULES`, or whose target-kind is outside the rule's `TargetSpec`
  (X2). Extend the existing dangling-citation logic over the new block; do not
  duplicate.
- **Inbound rendering** uses the rule's `inbound_name` (X5), not the raw outbound
  label — so an ADR's derived inbound shows `governs: SL-048`, not the backwards
  `governed_by: SL-048`. The existing `supersedes`→"superseded by" special-case
  collapses into this table-driven path.
- **Supersession cross-check (OD-3, ADR-010 D4).** `validate` reports where a
  governance entity's stored `superseded_by` disagrees with the reciprocal derived
  from `supersedes` `in_edges` — report drift, never rewrite (the reseat
  precedent). Pure read; independent of the (unbuilt, IMP-006) transactional
  supersede verb. This is the honest guard ADR-010 D4 named after reclassifying
  IMP-032 — it may surface pre-existing hand-authored drift, which is the point.

### 5.6 The contract doc + ADR amendment

- **Deliverable 1 — new tech spec "Cross-corpus relation contract"** (`/spec-tech`),
  authored as **PHASE-01**. Describes the model, label semantics, tier rationale,
  validation policy, outbound-only, graph composition, and *where authority lives*
  — pointing at ADR-010 + the `RELATION_RULES` code, **never** mirroring the
  enumeration (storage rule). `descends_from` the graph product spec; cites
  ADR-004 + ADR-010. SPEC-005/006/016 updated to *reference* it for the relation
  story rather than each re-telling it (deliverable 9).
- **Deliverable 8 — amend ADR-010**: a one-line note that SL-048 exercised D3's
  opportunistic tier-1 migration **corpus-wide** (incl. backlog), with the
  rationale (consistency over minimal churn; dogfood-only, no client back-compat).

## 6. Data, State & Ownership

- **`relation.rs` owns** `RelationLabel`, `RELATION_RULES`, the generic
  `append_edge`/`remove_edge`/`read_block`.
- **Each kind's module owns** only its tier-2/3 typed parsing (cohesion).
- **`integrity.rs` owns** id resolution + dangling-citation validation.
- **cordage owns** nodes/overlays/edges/reverse-index (unchanged).
- **Nothing owns a stored reverse field** — inbound recomputed from `in_edges`
  (ADR-004); the `superseded_by` carve-out is the sole typed reverse, projected by
  no reader.

## 7. Verification / Closure Intent

- A `governed_by` / PRD↔PRD / governance `related` edge authored via `link` is
  **validated**, persisted as `[[relation]]`, surfaced by `show` and the SL-046
  query, and appears in the target's **derived inbound** view (ADR-004).
- `link` **refuses** an illegal `(source, label, target-kind)` triple and a
  dangling numbered-kind target; `unlink` round-trips.
- `RELATION_RULES` rejects every inverse/derived label; a test asserts the SL-046
  reader's labels ⊆ the table (cannot diverge).
- **Behaviour preservation:** `backlog order` byte-identical (`needs`/`after`/
  `triggers` untouched); existing per-kind + cordage + backlog_order suites green;
  post-migration SL-046 reader emits the same edges for already-authored relations.
- Whole-corpus round-trip `doctrine <kind> show` + `validate` clean after the
  one-shot migration.

## 8. Phasing sketch (for `/plan`)

- **PHASE-01** — author the relation-contract tech spec (`/spec-tech`); the design
  contract re-homed. Settles semantics before code.
- **PHASE-02** — `RELATION_RULES` table + `RelationLabel` new variants + the
  reader-labels-⊆-table invariant test (pure, no storage change yet).
- **PHASE-03** — generic `[[relation]]` parser (`read_block`) + writer
  (`append_edge`/`remove_edge`); SL-046 reader accessors rewired to `read_block`
  for tier-1. Behaviour-preservation gate.
- **PHASE-04** — `link`/`unlink` command + forward-edge validation wiring; extend
  `validate` for `IllegalRow`s + the **supersession cross-check** (OD-3).
- **PHASE-05** — **deterministic** one-shot corpus migrator (unshipped, excludes
  the governance supersession pair) gated by **before/after black-box goldens** on
  `inspect`/`show`/`show --json` (OD-2); ADR-010 amendment (D3 corpus-wide +
  supersession-excluded); SPEC-005/006/016 references; **reclassify IMP-032**,
  **record the supersede-verb follow-up under IMP-006**.

(Indicative — `/plan` owns the authoritative phase decomposition + EN/EX/VT.)

## 9. Risks & Open Questions

- **R1 — migration correctness.** The out-of-band rewrite mutates committed
  authored TOML. Mitigation: round-trip `show` + `validate` diff across the whole
  corpus before commit; small corpus, reversible via git.
- **R2 — `read_block` vs tier-2/3 cohabitation.** A kind's accessor must merge
  shared tier-1 edges with its typed tier-2/3 edges without double-counting or
  ordering drift vs the SL-046 golden. Mitigation: behaviour-preservation gate.
- **R3 — `governed_by` inbound — RESOLVED (X5).** `inbound_name` on the rule
  renders the reciprocal ("governs"); generalises the `supersedes` special-case.
- **C1 — does `validate` already walk `[relationships]`?** PHASE-04 must extend,
  not duplicate, the existing dangling-citation logic (`integrity.rs`).
- **C2 — overlay identity — RESOLVED (OD-1).** `consumes` is its own label/overlay,
  distinct from gov `related`; no conflation.
- **C3 — supersession cross-check may fire on existing corpus.** Pre-existing
  hand-authored `superseded_by` may already disagree with `supersedes`; the
  cross-check will report it (intended — reveals drift, doesn't rewrite).

## 10. Decisions Log (the thrash)

- **D1** — tier-1 migration carried **corpus-wide** (incl. backlog), not per-kind.
  Rationale: final-shape consistency over minimal churn; dogfood-only.
- **D2** — migration is **out-of-band** (script/LLM), no shipped migrate verb, no
  dual-read; parsers cut hard to `[[relation]]`.
- **D3** — `label`/`target` field names; `governed_by` shared SL/SPEC/PRD→gov;
  `related` reused for PRD↔PRD.
- **D4** — `RelationRule` table with `TargetSpec`/`Tier`/`LinkPolicy` as the single
  source driving writer/validator/reader.
- **D5** — contract lives **inline in this design**, re-homed to a tech spec at
  PHASE-01 (not a new ADR — ADR-010 owns the decision).
- **D6** — `link` + `unlink` both; write-strict / read-tolerant validation.
- **D7** — governance `supersedes` is `LifecycleOnly` (no plain-`link`) — the
  reverse carve-out transaction is the pre-existing IMP-032 gap, out of scope.

## 11. Adversarial review

### Internal pass (recorded; F1/F2/F4 fixed above)

- **F1 (critical, fixed §5.1)** — TOML ordering: bare carve-out/`tags` keys after
  `[[relation]]` arrays bind to the last table (corruption). Typed leftovers go in
  a `[relationships]` table *before* the arrays; writer appends at EOF.
- **F2 (fixed §5.2)** — `RelationRule.sources: &[Kind]` (source-set), not one row
  per source kind — avoids `specs`/`slices` row explosion across backlog kinds.
- **F3 (open, for external)** — is `related` semantically adequate for PRD↔PRD, or
  do PRDs want a directional `reads`/`refines`/`depends_on`? "PRD-011 reads PRD-009"
  is dependency-flavoured; `related` flattens it.
- **F4 (fixed §5.2)** — `TargetSpec::AnyNumbered` for RV `reviews` (was prose "any").
- **F5 (→ R2)** — merge order: each accessor's `read_block` (tier-1) + typed
  (tier-2/3) merge must reproduce the SL-046 golden edge order; migration-script
  row order couples to it. Plan must pin the merge contract.

### External pass — codex-mcp (GPT-5.5), verdict: revision-required

Core critique: *"treating storage uniformity as if it were behaviour uniformity."*

**Adopted (design revised above):**
- **X1 (CRITICAL→§5.3)** — emit order was mutation-history; now axis-major,
  storage-independent. Fixes SL-046 golden breakage + tier-1/2 merge order.
- **X2 (CRITICAL→§5.3/§5.5)** — generic `read_block` lost per-kind legality; now
  source-kind-aware, illegal rows → validation findings not live edges.
- **X3 (MAJOR→§5.2)** — ⊆ invariant was fake; now exact coverage, overlay
  allocation table-driven (was hand-maintained `relation_graph.rs:117`).
- **X5 (MAJOR→§5.2/§5.5)** — `governed_by` rendered backwards inbound; added
  `inbound_name` to the rule, generalising the `supersedes` special-case.
- **X8 (MINOR→§5.4)** — `<TARGET>` (free-text `drift` isn't an id).

**Resolved with the user (round-2 escalations, §12):**
- **X4 (MAJOR→OD-1)** — *adopted.* Minted `consumes` (PRD→PRD), own label/overlay;
  inbound `consumed_by`. No overlay-model change; names the seam-consumption the
  PRDs describe, no collision with the work-`depends_on` axis.
- **X6 (MAJOR→OD-2)** — *adopted.* Deterministic in-repo one-shot migrator
  (unshipped) + before/after black-box goldens on `inspect`/`show`/`show --json`
  across the corpus, asserted byte-identical.
- **X7 (MAJOR→OD-3)** — *adopted.* Governance supersession pair excluded from
  migration (stays typed); `related` only migrates; **+ `validate` supersession
  cross-check** (§5.5); transactional verb → IMP-006; IMP-032 reclassified.

## 12. Decisions after external review — RESOLVED

- **OD-1 (X4) → `consumes`** for PRD→PRD (own overlay), inbound `consumed_by`.
  Grounded in PRD-011 "consumes a seam PRD-009 owns"; crisper than `builds_on`,
  no collision with the work-dependency axis.
- **OD-2 (X6) → deterministic script + before/after goldens** (unshipped).
- **OD-3 (X7) → exclude supersession from migration + validate cross-check.**
  Transactional supersede verb is IMP-006's cross-kind-lifecycle axis (a gov-only
  build now = parallel implementation). IMP-032's "derive, don't store" framing is
  stale (rejected by ADR-010 D4) — reclassified to the cross-check, addressed by
  this slice. **Follow-ups recorded: IMP-006 (verb), IMP-032 (corrected).**
