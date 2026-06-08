# Design SL-022: Technical-spec system support: descent, decomposition & integrity

<!-- Reference forms (doc/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

PRD-012 ("Technical Specifications") settled the v1 surface of the `tech` spec
family. SL-015 shipped the *at-rest* half of it — identity, requirements as peer
entities, membership labels, `c4_level`, `[[source]]` anchors, `interactions`
peer edges, `spec validate`, `spec show`. The *relational spine* is missing: a
tech spec cannot descend from the product capability it realises (REQ-082),
cannot decompose into a single-parent hierarchy (REQ-083), the registry runs no
decomposition integrity (REQ-087), and a peer interaction that points at a
product spec is mis-reported as a dangling reference instead of an invalid target
kind (REQ-084). Until that spine exists, SL-021 (the tech-spec corpus backfill)
has no complete surface to author against.

This slice builds the spine — and only the spine. It is deliberately narrowed:
supersession lineage (REQ-086), the importer (REQ-088 / OQ-2), transform verbs
(OQ-4), and the inbound/reverse views are out (see §6, slice Non-Goals).

## 2. Current State

`src/spec.rs` parses `spec-NNN.toml` into a `Spec` (flat fields incl. the
`Option<C4Level>` and `Vec<Source>` precedents), reads `members.toml` /
`interactions.toml` edge tables, renders a read-only reassembly in `render()`,
and scans the three trees into a `Registry` in `build_registry`. `run_validate`
consumes the registry; `run_show` reassembles one spec from its own files.

`src/registry.rs` is a pure, cache-independent snapshot: `requirements`,
`tech_specs`, `members`, `interactions` — **no product-id set** ("no check
resolves against one"), no graph traversal ("cycle detection deferred to the
feature DAG"). Four HARD checks, all direct `BTreeSet::contains`, all
scope-aware, all returning `Vec<String>` findings; `validate` runs them and
`run_validate` bails non-zero on any. (`mem.system.spec.composition-seam`.)

Governing canon: **ADR-004** — relations stored outbound-only on the durable
entity, reciprocity derived by registry scan, the sync-free reader (`spec show`)
renders *outbound only* (§3), and a reverse field is sanctioned *only* under the
§5 supersession carve-out. **ADR-001** — module layering leaf ← engine ←
command. The storage rule — structured data in TOML, no derived data in the
authored tier.

## 3. Forces & Constraints

- **ADR-004 outbound-only.** `parent` and `descends_from` are authored on the
  child / the tech spec; their reciprocals (children, realising specs) are
  derived, never stored, and are *not* the plain reader's to render (§3). This
  removes "show derived children" from scope.
- **Behaviour-preservation gate.** The shared machinery's SL-015 suites are the
  proof it is unchanged; they must stay green. The one sanctioned divergence is
  REQ-084's deliberate reframing of the interaction target-kind check.
- **Storage rule + cardinality.** The three relations are single-valued with no
  edge-local data, so they are flat scalar fields (the `c4_level` precedent), not
  edge-table files (which exist for many-cardinality + mobile edge data).
- **Additive registry.** The new `product_specs` set and edge collections must
  not perturb the existing four checks.
- **ADR-001 layering.** `registry.rs` (engine/leaf) stays pure; the impure scan
  and the verb wiring stay in `spec.rs` (command). No new module.

## 4. Guiding Principles

- Build the missing spine, reuse every shipped seam unchanged.
- Direction and reciprocity are ADR-004's to dictate, not this design's.
- Hard findings block; a genuinely-open policy question warns rather than
  pre-empting a decision (`descends_from` on a product spec).
- Keep graph machinery local to decomposition — no premature feature-DAG.

## 5. Proposed Design

### 5.1 System Model

Three single-valued outbound scalar fields on a tech spec's `spec-NNN.toml`:

- `descends_from = "PRD-NNN"` — cross-family descent (REQ-082). Tech-only.
- `parent = "SPEC-NNN"` — decomposition containment (REQ-083). Tech-only.
- (`superseded_by` — **deferred**, ADR-004 §5 carve-out; not in v1.)

Authoring is hand-edited TOML; `spec validate` is the integrity gate; `spec
show` renders outbound-only. No producer verb, no new edge file. The field name
is **`descends_from`**, not `realises` (`mem.concept.spec.descent-descends-from`:
code realises intent, a spec describes the *how* — `realises` overclaims).

### 5.2 Interfaces & Contracts

**`Spec` struct** gains:

```rust
#[serde(default)] pub(crate) descends_from: Option<String>, // → PRD-NNN, tech-only
#[serde(default)] pub(crate) parent:        Option<String>, // → SPEC-NNN, tech-only
```

`#[serde(default)]` → absent yields `None` (product specs and unfilled tech
specs parse unchanged, exactly as `c4_level`). No shape validation at parse; an
unresolvable ref is a `validate` finding. A duplicate `parent =` key is a TOML
parse error — this is how a *second parent* is precluded (REQ-087).

**`render()`** adds, after the `c4_level` line, kind-gated to tech:

```rust
if spec.kind == SpecSubtype::Tech {
    if let Some(p) = &spec.descends_from { parts.push(format!("descends from: {p}\n")); }
    if let Some(p) = &spec.parent        { parts.push(format!("parent: {p}\n")); }
}
```

Verbatim stored ref (mirrors interaction-target render). Order: `c4 level →
descends from → parent → responsibilities → sources`. **No children line**
(ADR-004 §3). Kind-gated so a (warned) `descends_from` on a product spec is not
legitimised by rendering.

**`Registry`** gains:

```rust
product_specs: BTreeSet<String>,        // descent + interaction-kind resolve here
parents:       Vec<ParentEdge>,         // { spec, parent }  tech-only
descents:      Vec<DescentEdge>,        // { spec, target, on_product: bool }
```

`on_product` lets the descent check separate the hard tech-spec cases from the
soft product-spec warning without a second collection pass.

**New pure checks** (HARD unless noted), each `Vec<String>`, scope-aware:

- `descent_findings` — for a tech-spec descent: target ∈ `product_specs` → clean;
  ∈ `tech_specs` → *invalid kind*; ∈ neither → *dangling* (REQ-082).
- `dangling_parent` — `parent` ∉ `tech_specs` → dangling (REQ-083).
- `self_parent` — `parent == spec` → hard 1-cycle (REQ-087).
- `parent_cycle` — walk the child→parent map from each node with a visited
  `BTreeSet`; a revisit is a cycle (REQ-087).
- interaction split — rewrite `dangling_interaction_targets`: target ∈
  `product_specs` → *invalid kind*; ∈ neither → *dangling* (REQ-084).
- `descent_on_product` — **WARN** — a product spec carrying `descends_from`
  (REQ-082 open edge; see §6).

**Severity split.** `validate` returns hard findings (as today) and a new soft
`warnings` set. `run_validate` prints both (warnings prefixed `warning:`) but
bails non-zero only on hard findings; a corpus clean-but-for-warnings exits zero.

### 5.3 Data, State & Ownership

`build_registry` already loops `[Product, Tech]`:
- Product arm: `product_specs.insert(ref)`; if the parsed spec carries
  `descends_from`, push a `DescentEdge { on_product: true }` (warn only).
- Tech arm (existing `tech_specs.insert` + interactions): read the parsed
  `Spec.parent` / `Spec.descends_from`; push `ParentEdge` / `DescentEdge { on_product:
  false }`, canonicalising the ref via the existing `resolve_spec_ref` path.

No new file reads — both fields ride `spec-NNN.toml`, already parsed. The
child→parent inversion needed for cycle detection is built inside `parent_cycle`,
ephemeral, never persisted (storage rule).

### 5.4 Lifecycle, Operations & Dynamics

Author edits `spec-NNN.toml` by hand → `doctrine spec validate` resolves every
FK and walks the parent chain → hard findings block (exit non-zero), warnings
advise (exit zero) → `doctrine spec show <ref>` reassembles the spec with its
outbound descent / parent / peer / anchor lines. The reverse views (children, a
PRD's realising specs) are a future registry/`inspect` surface concern (§6).

### 5.5 Invariants, Assumptions & Edge Cases

- A tech spec has **at most one** parent (scalar field; dup key rejected at
  parse) and the parent chain is **acyclic** (`parent_cycle`). Together: the
  decomposition is a forest of single-parent trees.
- A root tech spec has no `parent` — valid.
- `descends_from` resolves to a **product** spec; a tech-spec target is an
  invalid kind, an absent target is dangling.
- A peer `interaction` resolves to a **tech** spec; a product target is an
  invalid kind, an absent target is dangling.
- Reciprocity (children, realising specs) is **derived, never stored, never
  rendered by `spec show`** (ADR-004 §3).
- Product specs do not descend in v1; a `descends_from` on one is a **warning**,
  not a hard finding and not silently dropped.
- Assumption: `c4_level`, `[[source]]`, membership/label seams are final for the
  slice (PRD-012 settled).

## 6. Open Questions & Unknowns

- **Q1 — Product-spec hierarchy (open, undesigned).** v1 *warns* on
  `descends_from` on a product spec rather than forbidding the concept. There is a
  plausible future case for decomposition among product specs. Left undesigned;
  revisit before hardening the warning into an error. (Slice Follow-Ups.)
- **Q2 — Supersession lineage (deferred to a backlog item).** `superseded_by`
  under the ADR-004 §5 carve-out + a `spec supersede` verb (atomic status-flip +
  reverse co-write) + orphan-on-superseded-parent integrity. Out of v1; backlog
  once SL-020's backlog entity lands (REQ-086).
- **Q3 — Reverse/inbound views (deferred).** "A parent's children", "which tech
  specs realise this PRD" belong to a registry-backed `inspect`/survey surface,
  not the sync-free reader (ADR-004 §3). v1 computes the inversion only internally
  for cycle detection.
- **Q4 — REQ-082 prose.** REQ-082's title still reads "...the product capability
  it realises". Reconciling that wording to `descends_from` is PRD-012 territory,
  not SL-022.

## 7. Decisions, Rationale & Alternatives

- **D1 — Flat scalar fields, not edge tables or producer verbs.** Single-valued
  + no edge-local data ⇒ scalar field (the `c4_level` precedent). Edge tables
  carry cardinality and mobile data these relations lack; a producer verb breaks
  the established hand-edit-then-`validate` pattern and adds command surface this
  slice avoids. ADR-004 fixes the *direction*; cardinality fixes the *shape*.
- **D2 — `descends_from`, not `realises`.** Overclaim; see §5.1 and
  `mem.concept.spec.descent-descends-from`.
- **D3 — `spec show` outbound-only; no derived children.** ADR-004 §3 is binding;
  an earlier draft that scanned for children in the reader was withdrawn.
- **D4 — Supersession out of v1.** ADR-004 §5 frames it as a verb (status-flip +
  carve-out co-write), structurally distinct from the static hand-edited fields;
  pulling it out keeps v1 coherent and lets it be backlogged properly.
- **D5 — Warn (not error, not ignore) on product-spec descent.** Preserves the
  open question Q1 — a hard error pre-empts a possible future; silent ignore hides
  a likely misauthoring. Cost: `validate` gains a minimal severity tier (reusable).
- **D6 — Cycle detection local to decomposition.** A bounded child→parent walk,
  not a general DAG framework; the feature-DAG cache stays deferred (ADR-004 §4).

Alternatives rejected: edge-table per relation (D1); producer verbs in v1 (D1);
hard-error on product descent (D5); rendering children in `show` (D3).

## 8. Risks & Mitigations

- **R1 — `product_specs` perturbs existing checks.** Additive set + new methods
  only; the four SL-015 checks untouched. Mitigation: the SL-015 registry suite
  runs unchanged (gate).
- **R2 — Severity split leaks into existing exit semantics.** A warning must not
  flip a clean corpus to non-zero. Mitigation: explicit test — warnings-only →
  exit zero; any hard finding → non-zero.
- **R3 — rust-embed template footfault.** Editing `spec-tech.toml` alone does not
  re-embed until the embedding crate recompiles
  (`mem.pattern.embed.rustembed-recompile-and-symlinks`). Mitigation: note for
  `/execute`; verify the scaffold emits the new comments.
- **R4 — clippy ceilings.** New find-methods assemble strings under the repo's
  `push_str(&format!)` ban and collection bans (BTree only). Mitigation: follow
  `mem.pattern.lint.string-build-no-push-format` and `…disallowed-types-collections`.

## 9. Quality Engineering & Validation

Unit-test driven (no spec e2e harness exists; SL-015 set the precedent). Layer A
— `registry.rs` pure checks over hand-built registries: descent clean / dangling
/ invalid-kind; parent clean / dangling; self-parent; cycle-2 / cycle-3 / clean
chain-root; interaction invalid-kind (rewriting `non_tech_interaction_target…`) /
dangling; descent-on-product warns and is absent from hard findings; severity
gate (warnings-only → empty hard set). Layer B — `spec.rs` parse (`parent` /
`descends_from` present→`Some`, absent→`None`; duplicate `parent` key → read
errors) and render (tech emits the lines in order; product with `descends_from`
omits the line; no children line).

Behaviour-preservation: every SL-015 `registry.rs` / `spec.rs` test stays green
**unchanged** except the one deliberate REQ-084 rewrite (flagged in commit +
audit). Closure: `doctrine spec validate` non-zero on each crafted hard
violation, zero on a clean-but-warned corpus; `just check` green, clippy zero.

## 10. Review Notes
