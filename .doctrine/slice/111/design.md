# Design SL-111: Hoist kind identity to a leaf `kinds` module to break relation layering cycles

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§8), Q1. -->

## 1. Design Problem

The relation engine reaches **up** into command-tier modules for kind identity,
violating ADR-001 (`leaf ← engine ← command, no cycles`). `relation.rs:247-274`
declares 21 `const X: &Kind = &crate::<cmd>::*_KIND` aliases; every owning command
module imports `relation::tier1_edges` back → **7 confirmed cycles**. Break the
cycles by inverting ownership: the engine should borrow kind identity from a tier
*below* it, not reach up.

This is the enabling change for SL-112 — a compiler-enforced engine crate boundary
cannot compile while these cycles exist.

## 2. Current State

`relation.rs` aliases each kind's engine `entity::Kind` const by importing the
command module that declares it:

```rust
const SLICE: &Kind = &crate::slice::SLICE_KIND;     // …21 of these (L247-266)
const GOV:     &[&Kind] = &[ADR, POL, STD];          // 3 groupings (L271-274)
const BACKLOG: &[&Kind] = &[ISS, IMP, CHR, RSK, IDE];
const RECORD:  &[&Kind] = &[ASM, DEC, QUE, CON];
```

`RELATION_RULES` (22 rows) keys on these `&'static Kind` refs via `sources:
&[&Kind]` and `TargetSpec::Kinds(&[&Kind])`. **All four** consumer sites compare
by `.prefix` only — the `&Kind` is a pure prefix-carrier:

- `lookup` (L477) · `canonical_position` · `writable_labels_for` —
  `r.sources.iter().any(|k| k.prefix == source.prefix)`
- `validate_target` (`TargetSpec::Kinds`) — `set.iter().any(|k| k.prefix == target_prefix)`

The prefix is the canonical kind identity throughout the codebase (`relation_graph`
already does `kref.kind.prefix == "SPEC"` with a bare literal). The full `Kind`
(carrying `scaffold: fn`) **cannot** move down — `scaffold` binds each `Kind` to a
function in its command module, so an engine-tier `const Kind` would recreate the
exact upward edge (and memory `kind-is-data-not-trait` forbids restructuring the
seam). Therefore only **identity** — the prefix — is hoisted.

## 3. Forces & Constraints

- **ADR-001** — dependencies point downward only; no cycles. The target source of
  identity must sit at or below the engine tier.
- **`entity` is deliberately kind-blind** (`entity.rs:5`) — the concrete kind list
  must not land there.
- **`Kind` is data-not-trait, `scaffold` lives in the command tier** (memory
  `mem.pattern.entity.kind-is-data-not-trait`) — do not move scaffolds, do not
  trait-ify.
- **Behaviour-preservation gate** (AGENTS.md) — changing the relation engine means
  the existing suites are the proof; they stay green unchanged.
- **No parallel implementation** (CLAUDE.md) — the prefix literal must live in
  exactly one place, not be copied into the new module beside the existing
  `Kind.prefix`.

## 4. Guiding Principles

- **Hoist identity, not behaviour.** The engine needs the prefix; it does not need
  the scaffold. Move the minimum.
- **Prefix is identity; `&str` is its honest type.** No new wrapper type buys a
  consumer anything today (rejected alternative D2).
- **One authority for the literal.** `kinds.rs` owns each prefix string; both the
  relation table *and* the command `Kind.prefix` field consume it.
- **Zero caller churn.** Public relation signatures keep `&Kind` and read `.prefix`
  internally — only the rule-table *element type* changes.

## 5. Proposed Design

### 5.1 System Model

A new **leaf-tier** module `src/kinds.rs` owns the kind-identity vocabulary:
the canonical prefix per kind, plus the three relation source/target groupings.
It depends on **nothing** in-crate.

```
  command (slice, adr, …, relation_graph)  ──prefix:──┐
                                                       ▼
  engine (relation, integrity, …)  ──sources/target──▶ kinds (leaf)
```

`relation` (engine) now borrows identity downward from `kinds`; it imports no
command module. Command modules borrow the same literal downward for their
`Kind.prefix`. `kinds` is exactly the unit SL-112 pulls into the engine crate.

### 5.2 Interfaces & Contracts

`src/kinds.rs` (new):

```rust
//! The kind-identity vocabulary: canonical prefix per kind + the relation
//! source/target groupings. Leaf tier (ADR-001) — depends on nothing in-crate,
//! so the engine borrows identity without reaching up into command modules.
//! The prefix is the canonical kind identity (compared by `==` everywhere).

pub(crate) const SL: &str = "SL";
pub(crate) const PRD: &str = "PRD";
pub(crate) const SPEC: &str = "SPEC";
pub(crate) const CM: &str = "CM";
pub(crate) const REQ: &str = "REQ";
pub(crate) const ADR: &str = "ADR";
pub(crate) const POL: &str = "POL";
pub(crate) const STD: &str = "STD";
pub(crate) const RV: &str = "RV";
pub(crate) const REC: &str = "REC";
pub(crate) const REV: &str = "REV";
pub(crate) const ISS: &str = "ISS";
pub(crate) const IMP: &str = "IMP";
pub(crate) const CHR: &str = "CHR";
pub(crate) const RSK: &str = "RSK";
pub(crate) const IDE: &str = "IDE";
pub(crate) const ASM: &str = "ASM";
pub(crate) const DEC: &str = "DEC";
pub(crate) const QUE: &str = "QUE";
pub(crate) const CON: &str = "CON";

/// Every governance kind — `supersedes`/`related` source-set + `governed_by` targets.
pub(crate) const GOV: &[&str] = &[ADR, POL, STD];
/// Every backlog item kind — they share one `relation_edges` accessor.
pub(crate) const BACKLOG: &[&str] = &[ISS, IMP, CHR, RSK, IDE];
/// Every knowledge-record kind.
pub(crate) const RECORD: &[&str] = &[ASM, DEC, QUE, CON];
```

`relation.rs` element-type change (public fn signatures **unchanged**):

```rust
pub(crate) enum TargetSpec {
    Kinds(&'static [&'static str]),   // was &[&'static Kind]
    SameKind, AnyNumbered, Unvalidated,
}
pub(crate) struct RelationRule {
    pub(crate) sources: &'static [&'static str],   // was &[&'static Kind]
    // …rest unchanged
}
// matchers: `.any(|k| k.prefix == p)` → `.any(|p2| *p2 == p)`;
// diagnostic `set.iter().map(|k| k.prefix)` → `set.iter().copied()`.
pub(crate) fn lookup(source: &Kind, label: RelationLabel) -> …   // unchanged sig
pub(crate) fn tier1_edges(source_kind: &Kind, text: &str) -> …   // unchanged sig
pub(crate) fn rels_block(source: &Kind, …) -> String             // unchanged sig
```

### 5.3 Data, State & Ownership

- **`kinds.rs` owns every prefix literal.** Single authority.
- Each command `*_KIND` const re-points its `prefix:` field to `kinds::<X>`
  (`slice.rs` `prefix: "SL"` → `prefix: kinds::SL`; ~20 consts across slice, spec
  (×2), concept_map, requirement, adr, policy, standard, review, rec, revision,
  backlog (×5), knowledge (×4)). The "SL"-lives-twice parallel copy is removed.
- `integrity::KINDS` is a *referencing view* over those `Kind` consts (memory
  `mem.pattern.entity.numbered-kind-identity-table`); it inherits the re-pointed
  prefix transparently — **no change** there.
- No runtime state; all `const`.

### 5.4 Lifecycle, Operations & Dynamics

Pure compile-time data. The cycle-break is a property of the import graph after
the edit: `relation.rs` no longer names any `crate::<cmd>::…`. Nothing changes at
runtime — same rule table, same lookups, same outputs.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** — `kinds::GOV/BACKLOG/RECORD` membership equals the documented sets
  (pinned by a unit test). A drift here silently changes relation legality.
- **INV-2** — every prefix in `kinds.rs` matches the `prefix` its owning `Kind`
  const carries. After §5.3 re-pointing this is *structural* (the const reads the
  same symbol), not merely asserted.
- **ASM-1** — every relation consumer compares kinds by `prefix` only (verified:
  all four sites). If a future axis needed `dir`/`scaffold` from a `sources` entry,
  `&str` would be insufficient — not the case today.
- **Edge** — the `SLICE` alias in the rule table renames to `SL`; no semantic change.
- **Edge** — the rule-table *element type* change ripples into every site that reads
  `RelationRule.sources` / `TargetSpec::Kinds(ks)` and then `.prefix` on an element.
  These are all **mechanical accessor edits** (`|k| k.prefix == p` → `|p2| *p2 == p`;
  `ks.iter().map(|k| k.prefix)` → `ks.iter().copied()`), no assertion or behaviour
  change: `relation.rs` matchers + its `target_spec_matches_design` test block, and
  **one** test reader in `relation_graph.rs:1615`. Variant-only reads
  (`matches!(r.target, TargetSpec::Unvalidated)` at `relation_graph:361/1562/1581`)
  are untouched.

## 6. Open Questions & Unknowns

None blocking. (Resolved during design: representation = `&str` not a wrapper;
placement = new leaf `kinds.rs` not `entity`/`registry`; `relation_graph` excluded.)

## 7. Decisions, Rationale & Alternatives

- **D1 — New leaf module `kinds.rs`.** *Alt:* fold into `entity` (rejected — kind-blind
  by design) or `registry` (rejected — that is the FK-index seed, a semantic
  mismatch). A dedicated single-responsibility module is the cohesive home and the
  natural SL-112 crate member.
- **D2 — `&str` prefix constants, not a `KindId` newtype.** *Alt:* `struct
  KindId(&'static str)` for nominal typing / future `dir`. Rejected — no consumer
  needs more than the prefix today; the codebase already treats prefix-as-`&str`
  as the identity idiom (`relation_graph:652`). "As simple as possible."
- **D3 — Keep the full `Kind` consts in the command tier.** Forced, not chosen:
  `scaffold: fn` couples each `Kind` to its command module. Hoisting the full
  `Kind` is impossible without dragging scaffold logic down (out of proportion,
  behaviour-gate risk) — see §2.
- **D4 — Single-source the prefix via `Kind.prefix = kinds::X`.** *Alt:* leave
  command literals untouched (smaller diff). Rejected — leaves "SL" in two places,
  a parallel-implementation smell, and fails the slice's "command modules consume
  the hoisted constant" objective.
- **D5 — `relation_graph.rs` behavioral sites excluded.** Its three
  command-*imports* (`:82` `backlog::dep_seq_for`, `:423` gov supersession needing
  `dir` + `governance::supersession_pair`, `:652` `spec::interaction_types`) are
  **behavior-entangled**, not pure identity — a prefix swap cannot remove them. And
  `relation_graph` contributes **zero cycles** (its edges are non-cyclic). So the
  cycle-break is fully achieved by `relation.rs` alone; those sites ride a
  contingent follow-up (§8 R1), resolved iff SL-112 classifies it engine-internal.
  *Caveat (adversarial pass):* one **test** reader (`:1615`) reads
  `RELATION_RULES.sources` and takes the mechanical `&str` accessor edit — that is
  the rule-table type change rippling into a reader, **not** a behavioral change to
  an excluded site. "Excluded" means the upward `*_KIND`/behavioral edges, not "no
  line of `relation_graph` is touched."

## 8. Risks & Mitigations

- **R1 (scope drift)** — the slice scope originally named `relation_graph`. *Mit:*
  `slice-111.md` narrowed to `relation.rs`-only with the `relation_graph` work
  demoted to an explicit contingent follow-up; rationale recorded (D5).
- **R2 (silent legality change)** — a typo in a hoisted prefix or grouping would
  change relation validation silently. *Mit:* INV-1 membership pin; INV-2 made
  structural by §5.3; the existing relation suite re-exercises every rule.
- **R3 (missed matcher site)** — a `.prefix` comparison left keyed on the old type.
  *Mit:* the type change makes every stale site a **compile error**; no silent path.

## 9. Quality Engineering & Validation

- **Behaviour-preservation gate (primary).** The existing `relation` unit suite +
  corpus/relation integration tests keep **every assertion unchanged** and stay
  green. Test/code that reads the rule table internals — the `relation.rs` matchers,
  its `target_spec_matches_design` block, and the one `relation_graph.rs:1615` test
  reader — take the mechanical `&Kind`→`&str` accessor edit (`|k| k.prefix` →
  `|p| *p` / `ks.iter().copied()`); no assertion or expected value moves. Tests that
  go through `lookup`/`tier1_edges` are untouched (signatures stable).
- **New micro-test (`kinds.rs`).** Pin `GOV`/`BACKLOG`/`RECORD` membership to the
  documented sets (mirrors `integrity::kinds_table_*`).
- **Closure check.** `grep -nE 'crate::\w+::\w*_KIND' src/relation.rs` → empty.
- **`just gate`** green (clippy zero-warning, fmt).

## 10. Review Notes

### Internal adversarial pass (2026-06-19)

Empirically swept the full ripple of the `&Kind`→`&str` rule-table element-type
change (`grep -rn '\.sources\b|TargetSpec::Kinds|r\.target' src/`).

- **A — "`relation_graph` excluded entirely" was falsifiable.** `relation_graph.rs:1615`
  (a test) reads `r.sources … k.prefix` and needs the mechanical accessor edit.
  *Resolution:* §5.5 / D5 / §9 corrected — "excluded" scoped to the behavioral
  upward edges (`:82/:423/:652`), not "no line touched." One test line rides the
  mechanical re-key. Variant-only reads (`matches!(r.target, …Unvalidated)` at
  `:361/1562/1581`) confirmed untouched.
- **B — in-scope test churn is mechanical, not behavioral.** Read
  `relation.rs::target_spec_matches_design` (L1267-1403): every edit is accessor
  shape (`ks.iter().map(|k| k.prefix)` → `.copied()`); no assertion or expected
  set changes. §9 reworded from "green unchanged" to "every assertion unchanged."
- **C — stale doc-comment.** `TargetSpec`/`RelationRule` justify "No `Debug`" by
  "holds `&Kind`"; false after the swap. Flagged for update (no behaviour change).
- **No external construction leak.** Only `relation.rs` builds `RelationRule`/
  `TargetSpec`; all other modules read via `lookup` (stable sig) → the public type
  change is contained. `priority/graph.rs:356` reads `r.tier`/`r.link` only — safe.
- **Type-safety net (R3) holds.** Every stale `.prefix`-on-`&str` site is a compile
  error, not a silent path.

Doctrinal alignment: ADR-001 (downward-only, acyclic) satisfied — new edges are
`command→kinds` and `engine→kinds`, both downward to a leaf; no cycle. `entity`
kind-blindness and `kind-is-data-not-trait` respected (scaffold stays command-side).
No governance conflict surfaced; no `/consult` required.

### External pass

(Pending — offered to user after internal integration.)
