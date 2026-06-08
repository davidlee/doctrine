# lazyspec read-only front-end for doctrine

Thread: bolt **lazyspec** (a Rust TUI spec framework,
https://github.com/jkaloger/lazyspec, local checkout `../lazyspec`)
on as a **read-only** front-end for doctrine. Status: pre-slice; research
outstanding. Full assignment in `lazyspec-research-spec.md` (repo root) — that file
is the live placeholder; this memory is the durable pointer + decisions.

## Decided architecture (do not relitigate)
- **Core mismatch.** lazyspec is *document-centric* (a spec **is** a file —
  `DocMeta` = markdown + YAML frontmatter). doctrine is *composition-centric* (specs
  **lazily assembled** from requirements + outbound edges — no spec file exists).
  See [[mem.system.spec.composition-seam]].
- **Wire = doctrine CLI JSON**, never doctrine's TOML/MD. Keeps lazyspec ignorant of
  the storage rule ([[mem.concept.doctrine.storage-model]]); keeps doctrine ignorant
  of YAML/Ratatui. CLI is source of truth ([[mem.fact.doctrine.cli-source-of-truth]]).
- **Adapter lives in lazyspec, not doctrine.** lazyspec has no plugin loader —
  backends are a compile-time `StoreBackend` enum. doctrine gains at most **one leaf
  command** emitting the composed-spec projection as JSON; guaranteed no internal
  dependents by ADR-001 layering.
- **RO-first.** lazyspec backend returns `Err(ReadOnly)` from all four
  `DocumentStore` mutation methods. No doctrine mutation verbs needed for v1;
  selectively re-enable as doctrine grows them.
- **doctrine is the brain** — owns ids, edges, sequencing, lifecycle, validation.
  lazyspec's own validation rules disabled in the preset (two validators must not
  disagree).
- **Composed specs** project into lazyspec's `DocMeta.virtual_doc` (read-only views).

## Pieces
1. **Research** (`lazyspec-research-spec.md`) → produces the integration brief.
2. **doctrine wire-format lock + tests + thin generic adapter** — schema *specified
   by* the brief (DocMeta field needs). Pin with canary tests
   ([[mem.pattern.parse.toml-error-classification-fragile]] — same medicine: wire
   schemas are version-fragile).
3. **doctrine emits lazyspec brief** (the projection JSON).
4. **lazyspec fork + doctrine backend (off the brief) + doctrine-compat
   `.lazyspec.toml` preset.**

Ordering: 1 gates 2 and 3 (wire can't lock until DocMeta needs known); 2‖3 then 4.

## Research findings (resolved — brief: `../lazyspec/research/lazyspec-doctrine-integration-brief.md`)
- **RO tolerance: nearly free, one patch.** 4/6 mutation paths degrade acceptably
  (create/provenance show inline errors; delete/status/link silent no-op). The
  **editor key `e` is destructive** — opens an editor on the synthetic path, then
  `event_loop` re-ingests any file created. MUST gate: add
  `StoreBackend::is_writable()` → false for Doctrine, suppress `e`. Only true
  blocker for RO-safe.
- **`rules = []` does NOT empty validation** — TOML array-of-tables parses to `None`
  → `default_rules()`. So validation-off shifts entirely to the **emitter**: every
  projected entity carries `validate_ignore: true`. Residual: `TypeConstraintChecker`
  still runs even then — emitted types must be **non-singleton** in the `types[]`
  config. This is the load-bearing wire-format constraint.
- **DocMeta fidelity: lossy v1 accepted.** 12 core fields map 1:1. Richer edges
  flatten to the 4 RelationTypes (exotic → RelatedTo); phases + EN/EX/VT criteria
  dropped or pushed into body text. Do NOT extend DocMeta — keeps schemas decoupled.
- **Body is inline in the wire**, materialized to `.lazyspec/cache/doctrine/{kind}/{id}.md`
  (GitRef cold-cache precedent). Composed-spec body is *Critical* — emitter must
  assemble + serialize it or preview breaks.
- **Backend = fork, ~8–9 files**, match arms, GitRef precedent. No trait registry
  (that's a v2 upstream proposal to lazyspec). `loader.rs` unchanged.
- **id minting**: `numbering = "reserved"` + create-arm bails before numbering →
  double-mint impossible.
- **New doctrine command** (piece 3, working name): `doctrine emit-lazyspec-brief --json`.
  Wire schema = brief §3; preset `.lazyspec.toml` = brief §5.

## v1 limitations (eyes-open, not blockers)
- **Graph = implements-tree only.** lazyspec's graph follows `Implements` edges
  only. doctrine sequencing survives **only if expressed as `implements`**;
  `blocks`/`supersedes` show in tree + relations panel but NOT the graph. doctrine's
  full edge-DAG (its distinctive value) is partially lost in the graph view.
- **Silent no-op UX** on delete/status/link — dialog just closes, nothing happens.
  Adequate v1; wants inline-error parity later.

## Open design Q for piece 3 (emitter)
Node set: emit requirements AND composed-specs as separate nodes (composed
`implements` its constituents, per brief example), or composed-only? Avoid
double-representation in the doc tree. Settle in piece-3 design, not now.

## Doctrine-side work (this repo)
Pieces 2 (lock wire schema §3 + JSON conformance tests) and 3 (the emitter command)
are **doctrine slices** — route through `/slice` when picked up. Coordinate with the
in-flight JSON-output slice (currently touching `src/spec.rs`): the emitter should
**ride that JSON substrate, not build a parallel one**.
