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
  proof *unrelated* machinery is unchanged; they must stay green. REQ-084's
  rewrite of the interaction target-kind check is **not** a divergence from this
  gate — it is an **intended behaviour change** mandated by PRD-012 §6 (the
  contract moved from dangling to invalid-kind), outside the gate's reach. The
  one real caveat the gate must absorb is the new `build_registry` parse
  (Charge I): its error surface widens, and the hand-built `Registry` unit suites
  do not exercise it — §9 adds a `build_registry`-level test so that change is
  proven, not assumed.
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
unresolvable ref is a `validate` finding. A *second parent* is **doubly
precluded** (REQ-087, finding D): the scalar field makes the in-model state
unrepresentable (a duplicate `parent =` key or a `parent = [...]` array fails
`toml::from_str`), and a **pre-parse guard** in `build_registry` (§5.3) detects
that same malformation first and emits a **named hard finding** ("spec X declares
a second parent") rather than an opaque `"Failed to parse"`. The guard is what
satisfies AC1 *literally* (a hard finding, non-zero exit); the scalar shape is
defense-in-depth, not the sole mechanism.

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
- `parent_findings` — `parent` ∈ `tech_specs` → clean; ∈ `product_specs` →
  *invalid kind* (a parent must be a tech spec — symmetry with descent /
  interaction); ∈ neither → *dangling* (REQ-083). Excludes the self case (owned by
  `self_parent`).
- `self_parent` — `parent == spec` → hard. **Sole reporter** for the 1-cycle
  A→A; `parent_cycle` skips a node whose parent is itself, so A→A yields exactly
  one finding (REQ-087).
- `parent_cycle` — walk the child→parent map from each node with a visited
  `BTreeSet`; a revisit (chain length ≥ 2) is a cycle. Terminates at a root (node
  with no parent edge) or a dangling parent (target absent as a key) — neither is
  a cycle (REQ-087). **One finding per cycle, not per node** (finding A's "one
  finding per defect" law): a k-cycle is reported once, canonicalised by emitting
  only when the walk's starting node is the cycle's least id. (A naive
  walk-from-every-node yields k findings for one ring — the relapse the cycle-2 /
  cycle-3 tests must pin by asserting the *count*, not mere existence.)
- `second_parent` — **HARD**, pre-parse (§5.3). A `spec-NNN.toml` whose raw text
  carries a duplicate `parent =` key or an array `parent = [...]` → one named
  finding ("second parent declared in spec X"), non-zero exit. This is REQ-087
  AC1's literal mechanism (see finding D); the scalar field is the defense-in-depth
  backstop, not the diagnostic.
- interaction split — rewrite `dangling_interaction_targets`: target ∈
  `product_specs` → *invalid kind*; ∈ neither → *dangling* (REQ-084). **An intended
  behaviour change** — PRD-012 §6 moved the contract (a product target is now
  invalid-kind, not dangling), so the current behaviour is incorrect and the
  existing test asserts the old, now-wrong contract. Not a "divergence" from the
  behaviour-preservation gate — that gate guards *unrelated* machinery from
  *accidental* change and does not reach a deliberate, spec-mandated contract move.
- `descent_on_product` — **WARN** — a product spec carrying `descends_from`
  (REQ-082 open edge; see §6).

**Severity split.** `validate(scope) -> Vec<String>` keeps its signature and
returns **hard findings only** — so every SL-015 `validate` test stays valid. A
**sibling** `warnings(scope) -> Vec<String>` returns the soft set (scope-aware
like the rest). `run_validate` calls both, prints warnings prefixed `warning:`,
and bails non-zero only when the hard set is non-empty; a corpus clean-but-for-
warnings exits zero.

The severity tier is retained deliberately (finding D5; Charge IV answered). Its
sole *current* consumer is `descent_on_product`, which traces to no acceptance
criterion — but the tier is kept as **durable corpus machinery**, not built for
that one case: a hard/soft split is the correct shape for advisory integrity the
moment any check needs to *flag without blocking* (the anchor-integrity findings
PRD-012 §6 foreshadows, evergreen-altitude drift, importer reconciliation), and
warning over hard-error here preserves open question Q1 (a hard error pre-empts a
future product-spec hierarchy; the warn does not). Cost is one sibling method and
a `warning:` prefix — accepted as a one-time investment, not per-case scope creep.

### 5.3 Data, State & Ownership

**Correction (Charge I).** `build_registry` today reads **only** `members.toml`
and `interactions.toml` per spec (`src/spec.rs:705-743`); it does **not** parse
`spec-NNN.toml` — the lone `Spec` parse in the subsystem lives in `run_show`
(`src/spec.rs:675-677`). So `parent` / `descends_from` are **not** "already
parsed" on the validate path. Harvesting them is a **new, fallible per-spec read**:

`build_registry` already loops `[Product, Tech]`; each arm gains a
`read_to_string(spec-NNN.toml)` + `toml::from_str::<Spec>` (mirroring `run_show`):
- Product arm: `product_specs.insert(ref)`; if the parsed spec carries
  `descends_from`, push a `DescentEdge { on_product: true }` (warn only).
- Tech arm (existing `tech_specs.insert` + interactions): read the parsed
  `Spec.parent` / `Spec.descends_from`; push `ParentEdge` / `DescentEdge { on_product:
  false }`, canonicalising the ref via the existing `resolve_spec_ref` path.
- Both arms: before the parse, run the **pre-parse `second_parent` guard** over the
  raw text (duplicate `parent =` key or `parent = [...]` array → named hard
  finding); on a hit, record the finding and skip the structural parse for that
  spec so a malformed file yields a *named* finding, not an opaque parse `Err`.

**Error-surface change (behaviour-preservation, Charge I).** This new parse means
a malformed `spec-NNN.toml` that today does **not** break `doctrine spec validate`
(validate never opened it) **will** now surface — caught and named where it is a
second-parent violation, propagated as a `"Failed to parse"` context error
otherwise. This is a genuine, intended widening of `build_registry`'s failure
behaviour; the SL-015 unit suites stay green only because they build `Registry`
by hand and bypass this seam, so their greenness is **not** proof this path is
unchanged — §9 carries a `build_registry`-level test for the new behaviour.

The child→parent inversion needed for cycle detection is built inside
`parent_cycle`, ephemeral, never persisted (storage rule). The raw second-parent
scan reads no extra file — it inspects the text already read for the parse.

### 5.4 Lifecycle, Operations & Dynamics

Author edits `spec-NNN.toml` by hand → `doctrine spec validate` resolves every
FK and walks the parent chain → hard findings block (exit non-zero), warnings
advise (exit zero) → `doctrine spec show <ref>` reassembles the spec with its
outbound descent / parent / peer / anchor lines. The reverse views (children, a
PRD's realising specs) are a future registry/`inspect` surface concern (§6).

### 5.5 Invariants, Assumptions & Edge Cases

- A tech spec has **at most one** parent (scalar field + pre-parse `second_parent`
  guard → named hard finding; the dup key is also rejected at parse as a backstop)
  and the parent chain is **acyclic** (`parent_cycle`, one finding per cycle).
  Together: the decomposition is a forest of single-parent trees.
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

## 6a. REQ-087 reconciliation (Charge III, User-sanctioned)

REQ-087 AC1 — *"a second parent is rejected as a hard finding"* — and AC3 —
*"the integrity pass returns a non-zero result"* — are satisfied **literally** by
the pre-parse `second_parent` guard (§5.2/§5.3): a duplicate / array `parent`
yields a **named hard finding** in `validate`'s list and a non-zero `run_validate`
exit. The scalar field makes the state additionally **unrepresentable** in the
parsed model — defense-in-depth, not the diagnostic. The earlier draft satisfied
AC1 only by an opaque parse error (a literal-reading deviation); the User ruled
(a) accept structural impossibility *and* (b) add the named diagnostic — which
together promote the case from deviation to literal satisfaction. No open
reconciliation remains.

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
/ invalid-kind; parent clean / dangling; self-parent; **cycle-2 / cycle-3 each
asserting the finding *count* is exactly one** (Charge II — guards the
one-finding-per-cycle dedup, not mere existence); clean chain-root; interaction
invalid-kind (rewriting `non_tech_interaction_target…`) / dangling;
descent-on-product warns and is absent from hard findings; severity gate
(warnings-only → empty hard set). Layer B — `spec.rs` parse (`parent` /
`descends_from` present→`Some`, absent→`None`) and render (tech emits the lines in
order; product with `descends_from` omits the line; no children line). Layer C —
`build_registry` over a temp corpus (the seam Charge I exposed, untouched by the
hand-built-registry layers): (i) a well-formed corpus parses and collects
parent/descent edges; (ii) a **second-parent** spec (duplicate / array `parent`)
yields the named hard finding and `run_validate` exits **non-zero** (REQ-087 AC1
+ AC3, end-to-end, not at `toml::from_str` level); (iii) an otherwise-malformed
`spec-NNN.toml` now surfaces as a parse error where before it was invisible to
`validate` (the deliberate error-surface widening).

Behaviour-preservation, precisely: the SL-015 `registry.rs` *pure-check* methods
and their hand-built-registry tests are untouched (additive set + new sibling
methods) **except** the REQ-084 rewrite of
`non_tech_interaction_target_is_flagged_tech_only` — an **intended behaviour
change** (PRD-012 §6 moved the contract), not a gate divergence (flagged in commit
+ audit). The one **real** machinery change the unit suites do not cover is
`build_registry`'s new per-spec `Spec` parse (Charge I): it widens the impure
scan's error surface, so it is proven by Layer C, not assumed from green unit
tests. Two mechanical, non-behavioural edits are unavoidable and are *not* gate
breaches: (a) the `spec.rs` test `Spec { … }` constructors (`tech_spec` helper and
peers) gain `descends_from: None, parent: None` because `Spec` derives no
`Default`; (b) nothing else. No existing assertion changes value. Closure:
`doctrine spec validate` non-zero on each crafted hard violation (including the
second-parent case, Layer C), zero on a clean-but-warned corpus; `just check`
green, clippy zero.

## 10. Review Notes

Internal adversarial pass (integrated above):

- **A — self-parent / cycle double-report.** A→A is both a self-parent and a
  1-cycle. Resolved: `self_parent` is the sole reporter; `parent_cycle` skips
  self-loops (§5.2). One finding per defect. *(Extended by inquisition finding F:
  the same "one finding per defect" law applies to k-cycles too — see F.)*
- **B — `validate()` signature creep.** Folding warnings into `validate`'s return
  would break the SL-015 `validate` tests. Resolved: `warnings()` is a sibling
  method; `validate()` returns hard findings only (§5.2).
- **C — overstated behaviour-preservation.** Adding two non-`Default` fields
  forces the `spec.rs` `Spec { … }` test constructors to add `None, None` — a
  mechanical, non-behavioural edit, but not "unchanged." Claim corrected (§9).
- **D — REQ-087 AC1 mechanism (RESOLVED by inquisition + User ruling).** The
  draft satisfied "second parent rejected as a hard finding" only by an opaque
  TOML parse error (`build_registry` → `Err`) — a literal-reading deviation. The
  inquisition tried this in full (Charge III) and surfaced two unconfessed sins:
  the failure was *undiagnosable* (generic `"Failed to parse"`) and the AC3
  non-zero exit was tested only at `toml::from_str` level. User ruled: accept
  structural impossibility **and** add a named diagnostic. Synthesis: a pre-parse
  `second_parent` guard emits a **named hard finding** with non-zero exit (literal
  AC1 + AC3), the scalar field is defense-in-depth. The `Vec`-parent alternative
  stays rejected (reintroduces the representable-but-invalid state). See §6a, §5.2,
  §5.3, Layer C. **No deviation remains.**
- **E — parent target-kind symmetry.** A `parent` pointing at a product spec is
  now reported *invalid kind*, not *dangling* — consistent with descent and
  interaction (§5.2). `product_specs` was already in scope, so zero marginal cost.

Inquisition pass (`inquisition.md`, integrated above):

- **F — k-cycle multiplicity (MAJOR, was unconfessed).** A walk-from-every-node
  `parent_cycle` reports a k-node cycle k times, breaking finding A's
  "one-finding-per-defect" law for every ring larger than the self-loop. Resolved:
  dedup to one finding per cycle (canonicalise by least node-id); cycle-2 / cycle-3
  tests assert the *count* (§5.2, §9 Layer A). User ruling: dedup.
- **G — `build_registry` false "already parsed" (CRITICAL, was unconfessed).** The
  draft claimed `parent` / `descends_from` ride a spec already parsed on the
  validate path; in truth `build_registry` never parses `spec-NNN.toml` (only
  `run_show` does). Resolved: §5.3 corrected, the new per-spec parse and its
  widened error surface owned, Layer C added. User: corrected as a fix.
- **H — severity-tier warrant (MINOR).** The `warnings()` tier traces to no AC.
  User ruled: keep it as durable corpus machinery (justified §5.2, D5), not built
  for the single product-descent warn case.
- **(wording) REQ-084 framing.** Recast from "sanctioned divergence" to an
  intended, spec-mandated behaviour change (§3, §9).

Doctrinal alignment: ADR-004 (outbound-only, derived reciprocity, show
outbound-only, §5 carve-out deferral), ADR-001 (registry stays pure leaf; impure
scan + verb in command), storage rule (no derived data persisted) — all checked,
no conflict found.
