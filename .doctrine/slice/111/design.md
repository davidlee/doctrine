# Design SL-111: Hoist kind identity to a leaf `kinds` module to break relation layering cycles

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§8), Q1. -->

## 1. Design Problem

The relation engine reaches **up** into command-tier modules for kind identity,
violating ADR-001 (`leaf ← engine ← command, no cycles`). `relation.rs:247-274`
declares 20 `const X: &Kind = &crate::<cmd>::*_KIND` aliases; every owning command
module imports `relation::tier1_edges` back → **7 confirmed cycles**. Break the
cycles by inverting ownership: the engine should borrow kind identity from a tier
*below* it, not reach up.

This is the enabling change for SL-112 — a compiler-enforced engine crate boundary
cannot compile while these cycles exist.

## 2. Current State

`relation.rs` aliases each kind's engine `entity::Kind` const by importing the
command module that declares it:

```rust
const SLICE: &Kind = &crate::slice::SLICE_KIND;     // …20 of these (L247-266)
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
- **Prefix is identity; `&str` is the right type *for this slice*.** The relation
  engine compares by prefix only, so no wrapper or richer struct buys an in-scope
  consumer anything (D2). This is a deliberately **narrow** interface, not a claim
  that `&str` is the permanent end-state — the leaf identity module is expected to
  *widen* (→ `KindCore { dir, prefix, … }`) when SL-112 brings an engine-side
  consumer that needs more than the prefix (§8 R4).
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
  `&str` would be insufficient — not the case in-scope; that need is what triggers
  the widening to `KindCore` (§8 R4), a planned extension, not a re-discovery.
- **Edge** — the `SLICE` alias in the rule table renames to `SL`; no semantic change.
- **Edge** — the rule-table *element type* change ripples into every site that reads
  `RelationRule.sources` / `TargetSpec::Kinds(ks)` and then `.prefix` on an element.
  These are all **mechanical accessor edits** (`|k| k.prefix == p` → `|p2| *p2 == p`;
  `ks.iter().map(|k| k.prefix)` → `ks.iter().copied()`), no assertion or behaviour
  change. The complete reader set (re-swept, external pass §10): `relation.rs`
  matchers (L477/621/858/927), **two** `relation.rs` test blocks —
  `sources_match_shipped_accessors` (L1060, reads `.sources…k.prefix` at L1096) and
  `target_spec_matches_design` (L1267-1403) — and **one** test reader in
  `relation_graph.rs:1615`. Variant-only reads
  (`matches!(r.target, TargetSpec::Unvalidated)` at `relation_graph:361/1562/1581`)
  are untouched.

## 6. Open Questions & Unknowns

None blocking. (Resolved during design: representation = `&str` for this slice
(narrow by scope; widening to `KindCore` deferred — §8 R4, D2/D3); placement = new
leaf `kinds.rs` not `entity`/`registry`; `relation_graph` behavioral edges excluded
— SL-112-contingent, D5.)

## 7. Decisions, Rationale & Alternatives

- **D1 — New leaf module `kinds.rs`.** *Alt:* fold into `entity` (rejected — kind-blind
  by design) or `registry` (rejected — that is the FK-index seed, a semantic
  mismatch). A dedicated single-responsibility module is the cohesive home and the
  natural SL-112 crate member.
- **D2 — `&str` prefix constants for this slice, not a `KindId` newtype.** *Alt:*
  `struct KindId(&'static str)` for nominal typing. Rejected on idiom grounds — the
  codebase already treats prefix-as-`&str` as the identity idiom
  (`relation_graph:651`); a table-only newtype adds boundary friction (wrap/unwrap
  at every bare-`&str` comparison site) for marginal nominal gain, and no in-scope
  consumer needs more than the prefix. **This is narrow by *scope*, not a product
  position** (User, 2026-06-19): the leaf identity interface is expected to widen
  once it proves out. The widening target is not the newtype but a richer identity
  struct (D3, §8 R4) — so `KindId` would be a throwaway intermediate. Accepted cost:
  if/when widening lands, the `&str` table re-keys to the richer type — mechanical
  and compile-enforced (R4).
- **D3 — Keep the full `Kind` consts in the command tier; hoist *only* prefix now,
  not a `KindCore` identity split.** Two separable claims:
  - *The full `Kind` cannot hoist (forced).* `scaffold: fn` couples each `Kind` to
    its command module; an engine-tier `const Kind` would recreate the upward edge
    and violate memory `kind-is-data-not-trait`. This is genuinely forced — see §2.
  - *A leaf `KindCore { dir, prefix }` split is deferred, not impossible (chosen).*
    `Kind`'s only non-fn fields are `dir`+`prefix` (`entity.rs`), and identity-only
    consumers already exist (`integrity::KindRef` composes `&Kind` for dir+prefix+
    stem; `relation_graph:430` reads `g.kind.dir`). So a `KindCore` the command
    `Kind` *embeds* and the engine borrows is structurally live. Deferred because
    this slice's engine consumer (`relation`) compares **prefix only** — hoisting
    `dir` reads nothing in-scope and would re-expand into the just-narrowed
    `relation_graph` work. It becomes the right move when SL-112 places
    `relation_graph` engine-side (then `dir` needs a non-command source); recorded
    as the widening trajectory (§8 R4, D2).
- **D4 — Single-source the prefix via `Kind.prefix = kinds::X`.** *Alt:* leave
  command literals untouched (smaller diff). Rejected — leaves "SL" in two places,
  a parallel-implementation smell, and fails the slice's "command modules consume
  the hoisted constant" objective.
- **D5 — `relation_graph.rs` behavioral sites excluded.** Its three
  command-*imports* (`:82` `backlog::dep_seq_for`, `:423` gov supersession needing
  `dir` + `governance::supersession_pair`, `:652` `spec::interaction_types`) are
  **behavior-entangled**, not pure identity — a prefix swap cannot remove them. And
  `relation_graph` contributes **zero cycles** (verified external pass: no command
  module imports `relation_graph` back; its only inbound refs are `main` and an
  `integrity` comment). So **the 7 confirmed `relation.rs` cycles are fully broken
  by `relation.rs` alone.** This is *not* the same as "SL-112 is fully unblocked":
  ADR-001 is directional, so if SL-112 classifies `relation_graph` engine-side, its
  `:82/:423/:652` upward edges are still illegal and need their own resolution (the
  `KindCore` widening, §8 R4, supplies the leaf `dir` that `:423` wants). Those
  sites ride a contingent follow-up (§8 R1), resolved iff SL-112 classifies
  `relation_graph` engine-internal.
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
- **R4 (narrow-interface re-key)** — `&str` is narrow by scope (D2/D3); the leaf
  identity module is expected to widen to `KindCore { dir, prefix, … }` once SL-112
  brings an engine-side `relation_graph` that needs `dir` from a non-command source.
  At that point the `&str` rule table re-keys to the richer type — "pay twice"
  (external pass). *Mit:* the re-key is **mechanical and compile-enforced** (same
  type-safety net as R3), and the alternative (build `KindCore` now) would re-expand
  into the deliberately-cut `relation_graph` scope and hoist `dir` no in-scope
  consumer reads. Accepted: pay the small mechanical re-key later rather than carry
  out-of-scope `dir` plumbing now. SL-112's design must consume this as its starting
  identity shape so the widening is planned, not accidental.

## 9. Quality Engineering & Validation

- **Behaviour-preservation gate (primary).** The existing `relation` unit suite +
  corpus/relation integration tests keep **every assertion unchanged** and stay
  green. Test/code that reads the rule table internals — the `relation.rs` matchers,
  its `sources_match_shipped_accessors` (L1096) and `target_spec_matches_design`
  blocks, and the one `relation_graph.rs:1615` test reader — take the mechanical
  `&Kind`→`&str` accessor edit (`|k| k.prefix` →
  `|p| *p` / `ks.iter().copied()`); no assertion or expected value moves. Tests that
  go through `lookup`/`tier1_edges` are untouched (signatures stable).
- **New micro-test (`kinds.rs`).** Pin `GOV`/`BACKLOG`/`RECORD` membership to the
  documented sets (mirrors `integrity::kinds_table_*`).
- **Closure check.** No production-graph alias remains:
  `grep -nE 'const \w+: &Kind = &crate::\w+::\w*_KIND' src/relation.rs` → empty
  (the 20 hoisted aliases gone). The broader `grep -nE 'crate::\w+::\w*_KIND'`
  intentionally still matches the `#[cfg(test)]` `*_KIND` imports (L1015-1025,
  L1727): public signatures stay `&Kind`, so tests must construct real `Kind`
  instances (`lookup(&SLICE_KIND, …)`). Those are legitimate dev-dependency edges —
  acyclic for the SL-112 crate split (cargo permits dev-dep cycles) — not the
  command-tier cycle this slice breaks.
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

### External pass — codex/GPT-5.5 (2026-06-19)

Hostile source-verified review (read-only). Five charges put; dispositions:

- **Ripple completeness — ACCEPTED (design was wrong).** The internal sweep missed
  a reader: `relation.rs:1096` (`sources_match_shipped_accessors`, L1060) does
  `r.sources.iter().map(|k| k.prefix)` — a third in-file mechanical reader, distinct
  from `target_spec_matches_design`. Loud compile-fail, not silent, but §10/§5.5/§9
  claimed a complete sweep. *Fix:* reader set corrected in §5.5/§9; this is now the
  authoritative inventory (matchers L477/621/858/927 + two test blocks + the one
  `relation_graph:1615` line).
- **D3 "cannot hoist" overclaimed — ACCEPTED (rationale honesty).** True that the
  full `Kind` can't hoist (scaffold-coupled); false that bare `&str` is therefore
  forced. A `KindCore { dir, prefix }` split is structurally live (identity-only
  consumers already exist: `integrity::KindRef`, `relation_graph:430` reads `dir`).
  *Fix:* D3 split into forced-claim + chosen-claim; the `KindCore` alternative is
  named and **deferred by scope**, not dismissed as impossible.
- **D2 `&str` a tactical patch / dead-end — ACCEPTED as framing, kept as decision.**
  Narrow by scope, not a product position (User steer 2026-06-19: wide interface is
  the likely destination once the minimal hoist proves out). The dead-end "pay
  twice" risk is real but **contingent** on SL-112 classifying `relation_graph`
  engine-side and wanting leaf `dir`. *Fix:* recorded as R4 (widening trajectory,
  mechanical re-key) + D2 reframed. `KindId` newtype still rejected (idiom friction,
  and the real widening target is `KindCore`, not a newtype).
- **D5 SL-112-unblock overclaim — ACCEPTED (wording).** Zero-cycles-today confirmed
  by the reviewer (no inverse imports). But "cycle-break fully achieved" conflated
  *breaking the 7 confirmed `relation.rs` cycles* (true) with *unblocking the engine
  crate* (contingent on `relation_graph`'s classification). *Fix:* D5 now separates
  the two claims explicitly.
- **D4 single-source compile risk — REFUTED (reviewer concurs SOUND).** No `match`/
  const-context where `prefix: kinds::X` fails or changes meaning; consumers compare
  runtime strings. The one const-pattern wrinkle (`TargetSpec::Kinds(RECORD)` unstable
  in patterns, L1295) pre-exists and is unrelated to the `kinds::X` re-point.
- **Count slop — ACCEPTED.** "21 aliases" → 20 (§1/§2).

No new blocker. All accepted findings were rationale/inventory corrections or a
recorded contingent risk (R4); none changes the chosen mechanism (prefix-only `&str`
hoist for this slice). Reviewer ran no `cargo check` (read-only jail) — the
behaviour-preservation gate (§9) remains the implementation-time proof.

Doctrinal alignment unchanged: ADR-001 satisfied; `kind-is-data-not-trait` and
`entity` kind-blindness respected; R4 widening is forward-compatible with SL-112,
not a conflict.
