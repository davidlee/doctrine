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
| governance (ADR/POL/STD) | `supersedes`, `related` | `superseded_by` (carve-out), `tags` (classification) |
| slice | `specs`, `requirements`, `supersedes`, **+`governed_by`** | — |
| backlog | `slices`, `specs`, `drift`¹ | `needs`, `after{to,rank}`, `triggers{globs,note}` (SL-047 owns) |
| spec | **+`governed_by`**, **+`related`** (PRD↔PRD) | `descends_from`, `parent` (arity≤1), `interactions` (free-text+payload), `members` (members.toml) |

¹ `drift` migrates shape; target free-text-unvalidated (no `DRIFT` kind).

**Governance worked example** (spans all cases):

```toml
# BEFORE                          # AFTER
[relationships]                   [[relation]]
supersedes    = ["ADR-003"]       label = "supersedes"
related       = ["ADR-004"]       target = "ADR-003"
superseded_by = ["ADR-009"]       [[relation]]
tags          = ["layering"]      label = "related"
                                  target = "ADR-004"
                                  superseded_by = ["ADR-009"]  # ADR-004 §5, typed, verb-written only
                                  tags          = ["layering"] # classification, NOT a relation
```

`superseded_by` and `tags` are not tier-1 → stay typed beside the block. The
`[[relation]]` array admits **outbound validated labels only**, so `superseded_by`
is structurally un-authorable there (ADR-010 D4/D5 satisfied by construction).
spec legitimately carries both idioms — it spans tier-1 (new edges) *and* tier-2
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
enum TargetSpec { Kinds(&'static [Kind]), Unvalidated }   // Unvalidated = free-text, no KINDS row
enum Tier       { One, Typed }                              // One → [[relation]]; Typed → bespoke
enum LinkPolicy { Writable, LifecycleOnly, TypedVerbOnly }  // does `link` admit it

struct RelationRule {
  source: Kind,
  label:  RelationLabel,
  target: TargetSpec,
  tier:   Tier,
  link:   LinkPolicy,
}
const RELATION_RULES: &[RelationRule] = &[ … ];
```

Three axes, all from this one table: `target` → forward validation; `tier` →
storage shape; `link` → whether the verb admits the triple.

**Full vocabulary** (★ = new in SL-048):

| label | source(s) | target | tier | `link`? |
|---|---|---|---|---|
| `specs` | SL, backlog | PRD·SPEC | 1 | Writable |
| `requirements` | SL | REQ | 1 | Writable |
| `supersedes` | SL | SL | 1 | Writable |
| `supersedes` | ADR·POL·STD | same-gov | 1 | LifecycleOnly (carve-out, IMP-032) |
| `governed_by` ★ | SL·PRD·SPEC | ADR·POL·STD | 1 | Writable |
| `related` | ADR·POL·STD | same-gov | 1 | Writable |
| `related` ★ | PRD | PRD | 1 | Writable |
| `slices` | backlog | SL | 1 | Writable |
| `drift` | backlog | *free-text* | 1 | Writable (unvalidated) |
| `descends_from` | SPEC | PRD | 2 (arity≤1) | TypedVerbOnly |
| `parent` | SPEC | SPEC | 2 (arity≤1) | TypedVerbOnly |
| `members` | SPEC | REQ | 2 (members.toml) | TypedVerbOnly (`spec req add`) |
| `interactions` | SPEC | SPEC | 2 (free-text+payload) | TypedVerbOnly |
| `reviews` | RV | any | 2 (arity1 `[target]`) | TypedVerbOnly (`review`) |
| `owning_slice` | REC | SL | 2 (arity≤1) | TypedVerbOnly |
| `decision_ref` | REC | *free-text* | 3 | TypedVerbOnly |

**Label naming.** `governed_by` — one shared label for SL→gov and SPEC/PRD→gov
(one overlay, as `supersedes` already spans SL+gov); reads right on the source
("this slice is governed by ADR-010"); reader derives the inbound reciprocal on
the target. `related` reused for PRD↔PRD — vocab stays tight, the table
disambiguates by source; the `related` overlay carries gov→gov + PRD→PRD.

### 5.3 The seam — the uniformity dividend

Because tier-1 storage is now uniform, the ADR-010 per-kind *write* accessors
collapse to **one generic function each** for tier-1:

```rust
relation::append_edge(root, source_kind, id, label, target) -> Result<Wrote|Noop>  // idempotent
relation::remove_edge(root, source_kind, id, label, target) -> Result<Removed|Absent>
relation::read_block(toml) -> Vec<RelationEdge>                                      // shared tier-1 parse
```

Path resolved by kind via `integrity::KINDS`. The SL-046 per-kind `relation_edges`
accessors **shrink**: tier-1 becomes the shared `read_block`; each kind's accessor
keeps only its **tier-2/3** typed edges (lineage, members, interactions, reviews,
owning_slice). Net: less per-kind code, no per-kind write code for tier-1.

### 5.4 The `link` / `unlink` verbs (command layer)

```
doctrine link   <SOURCE-ID> <LABEL> <TARGET-ID>     # doctrine link SL-048 governed_by ADR-010
doctrine unlink <SOURCE-ID> <LABEL> <TARGET-ID>
```

Positional triple mirrors `(source, label, target)`. Dispatch:

1. Parse `<SOURCE-ID>` → `(Kind, id)` via `integrity::parse_canonical_ref`.
2. Look up `(Kind, label)` in `RELATION_RULES`. Absent → refuse, list legal labels
   for the source. `link ≠ Writable` → refuse, name the owning verb
   (`LifecycleOnly`/`TypedVerbOnly`).
3. Validate `<TARGET-ID>` (§5.5).
4. `append_edge` / `remove_edge` — edit-preserving toml_edit, idempotent (no-op if
   present / absent).

`unlink` folded in — symmetric on the same seam, near-free.

### 5.5 Forward-edge validation — write-strict, read-tolerant

- **`link` write:** `TargetSpec::Kinds` → target **must** resolve via
  `integrity::ensure_ref_resolves`, else hard-refuse (never create a dangler);
  also assert the target's kind is in the legal set. `TargetSpec::Unvalidated`
  (`drift`, `decision_ref`) → accept free-text as-is.
- **`validate` corpus check:** report `[[relation]]` danglers that arise later
  (target deleted post-authoring); never rewrite — the reseat precedent. Extend
  the existing dangling-citation logic over the new block; do not duplicate.

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
- **PHASE-04** — `link`/`unlink` command + forward-edge validation wiring.
- **PHASE-05** — one-shot corpus migration (out-of-band script) + whole-corpus
  round-trip verification; ADR-010 amendment; SPEC-005/006/016 references.

(Indicative — `/plan` owns the authoritative phase decomposition + EN/EX/VT.)

## 9. Risks & Open Questions

- **R1 — migration correctness.** The out-of-band rewrite mutates committed
  authored TOML. Mitigation: round-trip `show` + `validate` diff across the whole
  corpus before commit; small corpus, reversible via git.
- **R2 — `read_block` vs tier-2/3 cohabitation.** A kind's accessor must merge
  shared tier-1 edges with its typed tier-2/3 edges without double-counting or
  ordering drift vs the SL-046 golden. Mitigation: behaviour-preservation gate.
- **R3 — `governed_by` semantics on the inbound side.** Reader must render the
  derived reciprocal sensibly (ADR "governs" SL). Confirm SL-046's inbound label
  derivation handles a label whose natural inverse differs from its name.
- **C1 — does `validate` already walk `[relationships]`?** PHASE-05 must extend,
  not duplicate, the existing dangling-citation logic (`integrity.rs`).
- **C2 — overlay identity for reused `related`.** gov→gov + PRD→PRD share the
  `related` overlay; confirm SL-047 ranking / graph queries don't conflate them
  harmfully (label+source is the disambiguator).

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

Pending — internal self-review then external hostile pass (codex-mcp, GPT-5.5)
before lock. Findings recorded here.
