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
feature DAG"). Four HARD checks returning `Vec<String>` findings: three are
membership tests (`dangling_member_fks`, `dangling_interaction_targets`,
`duplicate_labels`) and one is the corpus-only `orphan_requirements` — and they
are **not** uniformly shaped (codex F6a): `duplicate_labels` is a `BTreeMap`
counting pass, and `orphan_requirements` takes **no** `scope` parameter (a
requirement's membership is unknowable from one spec, so it runs corpus-only).
`validate` runs them and `run_validate` bails non-zero on any.
(`mem.system.spec.composition-seam`.)

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
(ADR-004 §3). Kind-gated so a (now hard-invalid) `descends_from` / `parent` on a
product spec is not legitimised by rendering — `spec validate` flags it; `spec
show` does not display it.

**`Registry`** gains:

```rust
product_specs:  BTreeSet<String>,       // descent + interaction-kind resolve here
parents:        Vec<ParentEdge>,        // { spec, parent, on_product }
descents:       Vec<DescentEdge>,       // { spec, target, on_product }
build_findings: Vec<BuildFinding>,      // { spec, message } — pre-parse hard findings (carrier)
```

`on_product` (on **both** edges) is set true when the subject is a product spec;
the check turns it into a hard *invalid-kind* finding — `descends_from` and
`parent` are tech-only fields, so either on a product is a wrong-subject error
(codex F5; symmetry with the target-kind checks). `build_findings` is the carrier
for findings born at scan time before a pure check can run — the `second_parent`
guard (codex F1) records into it; `validate(scope)` includes it (scope-filtered by
`spec`). The field is inert data, so `registry.rs` stays a pure leaf (ADR-001) —
only `build_registry` (in `spec.rs`, the impure shell) populates it.

**New pure checks** — all **HARD**, each `Vec<String>`, scope-aware (filter by
`spec == scope` like the existing checks):

- `descent_findings` — `on_product` → *invalid kind* ("`descends_from` is tech-only,
  found on product X", codex F5); else (tech subject) target ∈ `product_specs` →
  clean, ∈ `tech_specs` → *invalid kind*, ∈ neither → *dangling* (REQ-082).
- `parent_findings` — `on_product` → *invalid kind* ("`parent` is tech-only, found
  on product X", codex F5b — the previously-ignored case); else (tech subject)
  `parent` ∈ `tech_specs` → clean, ∈ `product_specs` → *invalid kind* (a parent
  must be a tech spec — symmetry), ∈ neither → *dangling* (REQ-083). Excludes the
  self case (owned by `self_parent`).
- `self_parent` — `parent == spec` (tech subject) → hard. **Sole reporter** for the
  1-cycle A→A; `parent_cycle` skips a node whose parent is itself, so A→A yields
  exactly one finding (REQ-087).
- `parent_cycle` — for each tech node, walk the child→parent map keeping an
  **ordered path** (`Vec`) plus a first-seen index map (a bare visited `BTreeSet`
  is insufficient — codex F3). On revisiting a node already on the path, **recover
  the cycle slice** = `path[first_index_of_revisited ..]` (the ring only, not the
  tail that fed it), and emit **one** finding **only when the start node is the
  least id within that recovered slice**. This dedups correctly even for a tail
  feeding a ring (`T → A → B → A`: the slice is `{A,B}`, least `A`; `T` is not in
  the slice so its walk never emits; only `A`'s walk does). Terminates at a root
  (no parent edge) or a dangling parent (target absent as a key) — neither is a
  cycle (REQ-087). Tests pin the **count** (one per cycle) for cycle-2, cycle-3,
  and the tail-fed-cycle case, not mere existence.
- `second_parent` — **HARD**, born at scan time (§5.3), carried via
  `build_findings`. A `spec-NNN.toml` declaring `parent` twice (duplicate-key) or
  as an array (`parent = [...]`) cannot deserialize to the scalar field; rather
  than line-scanning the raw text (not comment- or string-aware — would false-hit
  the scaffold's own commented `# parent = …` example, codex F2),
  `build_registry` **classifies the `toml::from_str` error**: a duplicate-key or
  wrong-type error *at `parent`* becomes a named finding ("second parent declared
  in spec X") pushed to `build_findings`; any other parse error propagates as the
  generic `"Failed to parse"`. This is REQ-087 AC1's literal mechanism (a named
  hard finding + non-zero exit); the scalar field is the defense-in-depth backstop.
  The exact error-kind match is pinned by a test (toml-version-fragile).
- interaction split — rewrite `dangling_interaction_targets`: target ∈
  `product_specs` → *invalid kind*; ∈ neither → *dangling* (REQ-084). **An intended
  behaviour change** — PRD-012 §6 moved the contract (a product target is now
  invalid-kind, not dangling), so the current behaviour is incorrect and the
  existing test asserts the old, now-wrong contract. Not a "divergence" from the
  behaviour-preservation gate — that gate guards *unrelated* machinery from
  *accidental* change and does not reach a deliberate, spec-mandated contract move.

**No severity tier (codex F5 — supersedes the earlier warn).** The earlier draft
made product `descends_from` a soft *warning* to "preserve Q1." Codex showed the
rationale is broken: a future product-spec hierarchy would use **`parent`**, not
the cross-family **`descends_from`**, so warning on product `descends_from`
preserves nothing — and product `parent` was being **silently ignored** entirely.
Both tech-only fields on a product are now plain **hard invalid-kind** findings
(symmetry, no special case). `validate(scope) -> Vec<String>` keeps its signature
and returns all hard findings (including `build_findings`, scope-filtered); every
SL-015 `validate` test stays valid. **No `warnings()` sibling, no `warning:`
prefix, no soft exit tier** — the speculative machinery is dropped (YAGNI; the
drift / anchor-integrity consumers that might justify it are out of scope). When
product specs genuinely gain a hierarchy, `parent` is added to the product family
then — the hard finding forecloses nothing.

### 5.3 Data, State & Ownership

**Correction (Charge I).** `build_registry` today reads **only** `members.toml`
and `interactions.toml` per spec (`src/spec.rs:705-743`); it does **not** parse
`spec-NNN.toml` — the lone `Spec` parse in the subsystem lives in `run_show`
(`src/spec.rs:675-677`). So `parent` / `descends_from` are **not** "already
parsed" on the validate path. Harvesting them is a **new, fallible per-spec read**:

`build_registry` already loops `[Product, Tech]`; each arm gains a
`read_to_string(spec-NNN.toml)` + `toml::from_str::<Spec>` (mirroring `run_show`).
The subject `ref` is known from the dir scan (`subtype.canonical_id(id)`)
independent of the parse, so a finding can name a spec **even when its parse
fails**. Per spec:
- Attempt the parse. On **Ok**: harvest `Spec.descends_from` / `Spec.parent`,
  pushing `DescentEdge` / `ParentEdge` with `on_product = (subtype == Product)`,
  canonicalising the ref via the existing `resolve_spec_ref` path. (The product
  arm also `product_specs.insert(ref)` and the tech arm keeps its existing
  `tech_specs.insert` + interactions.) **Both** arms harvest **both** fields — a
  product carrying either tech-only field must be seen so the check can flag it
  (codex F5b), not silently dropped.
- On **Err**: classify the error (codex F1/F2). A duplicate-key or wrong-type
  error at `parent` → push a `BuildFinding { spec: ref, message: "second parent…" }`
  to `build_findings` and continue (skip this spec's edges). Any other parse error
  → propagate via `?` with the `"Failed to parse"` context. No raw line-scan — the
  parser is the comment/string-aware authority.

`validate(scope)` aggregates `build_findings` (filtered by `spec == scope`) into
its hard set alongside the pure checks; `run_validate` bails non-zero on any hard
finding exactly as today — so the second-parent case is a genuine findings-list
entry with a non-zero exit, end-to-end.

**Error-surface change (behaviour-preservation, Charge I).** This new parse means
a malformed `spec-NNN.toml` that today does **not** break `doctrine spec validate`
(validate never opened it) **will** now surface — named where it is a second-parent
violation, propagated as a `"Failed to parse"` context error otherwise. This is a
genuine, intended widening of `build_registry`'s failure behaviour; the SL-015
unit suites stay green only because they build `Registry` by hand and bypass this
seam, so their greenness is **not** proof this path is unchanged — §9 carries a
`build_registry`-level test for the new behaviour.

The child→parent inversion needed for cycle detection is built inside
`parent_cycle`, ephemeral, never persisted (storage rule).

### 5.4 Lifecycle, Operations & Dynamics

Author edits `spec-NNN.toml` by hand → `doctrine spec validate` resolves every
FK and walks the parent chain → any hard finding blocks (exit non-zero); a clean
corpus exits zero → `doctrine spec show <ref>` reassembles the spec with its
outbound descent / parent / peer / anchor lines. The reverse views (children, a
PRD's realising specs) are a future registry/`inspect` surface concern (§6).

### 5.5 Invariants, Assumptions & Edge Cases

- A tech spec has **at most one** parent (scalar field — a duplicate / array
  `parent` fails to deserialize, classified into a named `second_parent` hard
  finding) and the parent chain is **acyclic** (`parent_cycle`, one finding per
  cycle). Together: the decomposition is a forest of single-parent trees.
- A root tech spec has no `parent` — valid.
- `descends_from` resolves to a **product** spec; a tech-spec target is an
  invalid kind, an absent target is dangling.
- A peer `interaction` resolves to a **tech** spec; a product target is an
  invalid kind, an absent target is dangling.
- Reciprocity (children, realising specs) is **derived, never stored, never
  rendered by `spec show`** (ADR-004 §3).
- Product specs carry neither tech-only field in v1; a `descends_from` **or**
  `parent` on a product is a **hard invalid-kind** finding (codex F5), never
  silently dropped and never rendered.
- A tech `descends_from` restates no product intent — it stores only the target's
  durable id, no prose field exists to hold a restatement (REQ-082 AC3, satisfied
  by construction; D2). Authoring discipline, not a code gate.
- Assumption: `c4_level`, `[[source]]`, membership/label seams are final for the
  slice (PRD-012 settled).

## 6. Open Questions & Unknowns

- **Q1 — Product-spec hierarchy (open, undesigned).** v1 treats a tech-only field
  (`descends_from` / `parent`) on a product as a **hard invalid-kind** finding
  (codex F5 corrected the earlier warn). This forecloses nothing: a future
  product-spec hierarchy would add `parent` to the **product** family — a new,
  deliberate surface — at which point product `parent` stops being invalid by
  definition. The plausible future case stands; v1 simply refuses the field on the
  wrong subject today. (Slice Follow-Ups.)
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
the `second_parent` guard (§5.2/§5.3): a duplicate / array `parent` fails to
deserialize, `build_registry` **classifies that parse error** into a **named hard
finding** carried in `build_findings`, which `validate(scope)` surfaces in its
list with a non-zero `run_validate` exit. The scalar field makes the state
additionally **unrepresentable** — defense-in-depth, not the diagnostic. The
earlier draft satisfied AC1 only by an opaque parse error (a literal-reading
deviation); the User ruled (a) accept structural impossibility *and* (b) add the
named diagnostic — promoting the case to literal satisfaction. Codex F1/F2 then
fixed *how*: the finding rides a `build_findings` carrier (it had nowhere to live)
and is born by classifying the parse error, **not** a raw line-scan (which would
false-hit the scaffold's commented `parent` example). No open reconciliation
remains.

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
- **D5 — Hard invalid-kind (not warn, not ignore) on a tech-only field on a
  product.** *(Reversed by codex F5; supersedes the earlier warn-tier decision.)*
  Both `descends_from` and `parent` are tech-only; on a product they are a
  wrong-subject error, flagged hard for symmetry with the target-kind checks. The
  earlier warn rationale ("preserve Q1") was incoherent — a product hierarchy uses
  `parent`, not `descends_from` — and product `parent` was being silently ignored.
  Hard-flagging forecloses no future (Q1); it deletes the speculative severity tier
  (YAGNI), keeping `validate`'s signature and the whole exit contract unchanged.
- **D6 — Cycle detection local to decomposition.** A bounded child→parent walk
  (ordered path + first-seen index for correct per-cycle dedup, codex F3), not a
  general DAG framework; the feature-DAG cache stays deferred (ADR-004 §4).

Alternatives rejected: edge-table per relation (D1); producer verbs in v1 (D1);
**warn (not hard) on a tech-only field on a product** (D5, codex F5); a
`Vec`-typed `parent` to make the second parent a list finding (reintroduces the
representable-but-invalid state, finding D); a raw-text second-parent scan (not
comment-aware, codex F2); rendering children in `show` (D3).

## 8. Risks & Mitigations

- **R1 — new registry fields perturb existing tests.** The four SL-015 *checks*
  are untouched, but the additive `Registry` fields break the test `clean()`
  struct literal (`registry.rs:175` — full literal, no `..Default`, codex F6b).
  Mitigation: `Registry` derives `Default`, so the fix is mechanical
  (`..Default::default()` or the new fields); §9 owns this as a disclosed,
  non-behavioural existing-test edit (not a silent gate breach).
- **R2 — error-classification fragility.** The `second_parent` finding is born by
  matching the `toml::from_str` error kind (duplicate-key / wrong-type at
  `parent`), which is toml-version-sensitive. Mitigation: pin the match with a test
  over a real duplicate-`parent` and array-`parent` doc; if the match ever fails,
  the file falls through to the generic `"Failed to parse"` — degraded message,
  still a non-zero exit, never a silent pass.
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
/ invalid-kind (tech target) / **invalid-kind on product subject** (codex F5);
parent clean / dangling / invalid-kind (product target) / **invalid-kind on
product subject** (codex F5b, the previously-ignored case); self-parent;
**cycle-2 / cycle-3 / tail-fed-cycle each asserting the finding *count* is exactly
one** (codex F3 — guards the per-cycle dedup against the tail-into-ring case, not
mere existence); clean chain-root; interaction invalid-kind (rewriting
`non_tech_interaction_target…`) / dangling. Layer B — `spec.rs` parse (`parent` /
`descends_from` present→`Some`, absent→`None`) and render (tech emits the lines in
order; product with `descends_from` / `parent` omits the line; no children line).
Layer C — `build_registry` over a temp corpus (the seam Charge I exposed,
untouched by the hand-built-registry layers): (i) a well-formed corpus parses and
collects parent/descent edges; (ii) a **second-parent** spec (duplicate `parent`
*and* an array `parent`, separately) lands a `build_findings` entry that
`validate` surfaces and `run_validate` exits **non-zero** on (REQ-087 AC1 + AC3,
end-to-end, not at `toml::from_str` level); (iii) the scaffold's commented
`# parent = …` example does **not** trip the guard (codex F2 regression); (iv) an
otherwise-malformed `spec-NNN.toml` surfaces as a parse error where before it was
invisible to `validate` (the deliberate error-surface widening).

REQ-082 AC3 ("does not restate product intent") is **satisfied by construction**,
not by a test: the field stores only a target id and there is no prose field to
hold a restatement (D2). Authoring discipline — verified by review (`VA`), not a
machine gate. The slice's "every requirement traces to a test" is corrected to
"every machine-checkable AC traces to a test; authoring-discipline ACs are
satisfied by construction."

Behaviour-preservation, precisely: the SL-015 `registry.rs` *pure-check* methods
are untouched (additive fields + new methods) **except** the REQ-084 rewrite of
`non_tech_interaction_target_is_flagged_tech_only` — an **intended behaviour
change** (PRD-012 §6 moved the contract), not a gate divergence (flagged in commit
+ audit). The machinery change the unit suites do not cover is `build_registry`'s
new per-spec `Spec` parse (Charge I): it widens the impure scan's error surface,
proven by Layer C, not assumed from green unit tests. Three mechanical,
non-behavioural existing-test edits are unavoidable and are *not* gate breaches:
(a) the `spec.rs` test `Spec { … }` constructors gain `descends_from: None,
parent: None` (`Spec` derives no `Default`); (b) the `registry.rs` test `clean()`
`Registry { … }` literal (`registry.rs:175`) gains the three new fields or
`..Default::default()` (`Registry` *does* derive `Default`) — codex F6b, the
previously-undisclosed edit; (c) nothing else. No existing assertion changes value.
Closure: `doctrine spec validate` non-zero on each crafted hard violation
(dangling/invalid-kind descent incl. product-subject, dangling/invalid-kind parent
incl. product-subject, self-parent, cycle, second-parent, dangling/invalid-kind
interaction), zero on a clean corpus; `just check` green, clippy zero.

## 10. Review Notes

Internal adversarial pass (integrated above):

- **A — self-parent / cycle double-report.** A→A is both a self-parent and a
  1-cycle. Resolved: `self_parent` is the sole reporter; `parent_cycle` skips
  self-loops (§5.2). One finding per defect. *(Extended by inquisition finding F:
  the same "one finding per defect" law applies to k-cycles too — see F.)*
- **B — `validate()` signature creep.** *(Mooted by codex F5.)* The concern was
  folding warnings into `validate`'s return; with the warn tier dropped (F5),
  `validate()` returns hard findings only and there is no `warnings()` sibling at
  all — signature unchanged for the simplest reason.
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
- **H — severity-tier warrant (MINOR).** *(SUPERSEDED by codex F5 below.)* The
  `warnings()` tier traced to no AC; the User initially ruled keep-it. Codex then
  showed the warn's only justification was incoherent, so the tier is **dropped**
  entirely — see F5.
- **(wording) REQ-084 framing.** Recast from "sanctioned divergence" to an
  intended, spec-mandated behaviour change (§3, §9).

Codex MCP pass (`gpt-5.2-codex`, read-only; verdict "must be revised first" — all
six accepted, each verified against source):

- **F1 — second-parent finding had no carrier (CRITICAL).** `validate()` is a pure
  `Registry` method; the build-time guard's finding had nowhere to live. Resolved:
  `Registry.build_findings` carrier, populated by `build_registry`, surfaced
  (scope-filtered) by `validate` (§5.2, §5.3). *(Supersedes finding D's "pre-parse
  guard" hand-wave, which never said where the finding was stored.)*
- **F2 — raw line-scan not comment-aware (MAJOR).** A raw `parent =` scan would
  false-hit the scaffold's own commented `# parent = …` example on every fresh
  spec. Resolved: classify the `toml::from_str` error instead of scanning text
  (§5.2 `second_parent`, §5.3); Layer C(iii) regression-tests the commented case.
- **F3 — cycle dedup unsound for a tail feeding a ring (MAJOR).** "Least id over
  the visited `BTreeSet`" cannot identify which suffix is the actual cycle.
  Resolved: ordered path + first-seen index, recover the cycle slice, canonicalise
  over the slice; tail-fed-cycle VT added (§5.2, §9). *(Refines finding F's
  fix, which named the symptom but not the correct algorithm.)*
- **F4 — traceability / PHASE-03 proof overclaim (MAJOR).** REQ-082 AC3 is
  authoring discipline (no VT) — reclassified satisfied-by-construction/`VA`;
  PHASE-03's end-to-end exit proof scoped (second-parent proven there, the
  findings-list cases swept in the closure phase). §9 + plan corrected.
- **F5 — subject-kind policy incoherent (MAJOR).** Warning on product
  `descends_from` preserved nothing (a product hierarchy uses `parent`), and
  product `parent` was silently ignored. **User ruled: hard invalid-kind for both,
  drop the severity tier.** Reverses D5/H (§5.2, §5.3, D5, Q1). The single largest
  simplification this pass.
- **F6 — design misstated current code (MINOR).** (a) the four checks are not all
  `BTreeSet::contains`/scope-aware (`duplicate_labels` counts, `orphan_requirements`
  is corpus-only) — §2 corrected; (b) the `clean()` `Registry` literal needs the
  new fields — §9/R1 disclose this third existing-test edit.

Doctrinal alignment: ADR-004 (outbound-only, derived reciprocity, show
outbound-only, §5 carve-out deferral), ADR-001 (registry stays pure leaf — the
`build_findings` carrier is inert data populated only by the impure
`build_registry`; the verb in command), storage rule (no derived data persisted) —
all checked, no conflict found.
