# Design SL-204: Extract per-kind validation contract to break integrity coupling

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-204, ADR-001, IMP-266); doc-local refs bare — OQ-1, D1, R1. -->

## 1. Design Problem

RSK-227's coupling map names `integrity` (out=13, in=13) the symmetric nexus of
the 23-node command-tier core SCC (IMP-266, Tier-2). `integrity` imports all 13
kind modules to assemble `KINDS` — the corpus-wide id table — and 4 of those
kinds (`backlog`, `rec`, `review`, `revision`) import `integrity` back for its
ref-resolution helpers, closing 4 two-cycles that feed the core SCC.

Invert the dependency: the kind-identity data and the ref-resolution helpers
move to a **leaf**; `integrity` keeps only the validate/reseat machinery and
depends downward. Pure structural inversion — zero validation-behaviour delta;
the existing suites are the behaviour-preservation proof.

Precedent inherited from SL-203 (`mem.pattern.lint.mcp-server-entangled-with-core`):
RSK-227's SCC topology claims are point-in-time inferences and its numbers were
stale — **all tangle counts in this design are deferred to execute-time
measurement**; the current tight baseline is `command = 86`.

## 2. Current State

### 2.1 What integrity actually consumes from the 13 kinds (OQ-1 resolved: thin)

`src/integrity.rs:19-34` imports exactly the `*_KIND` statics — no validation
logic. They populate:

```rust
pub(crate) struct KindRef {
    pub(crate) kind: &'static entity::Kind,
    pub(crate) state_dir: Option<&'static str>,   // runtime tree reseat must not strand
}
pub(crate) const KINDS: &[KindRef] = &[ /* 24 numbered kinds */ ];
```

`KindRef` is a *referencing view* by deliberate SL-031 decision
(`mem.pattern.entity.numbered-kind-identity-table`): prefix/dir are **not
copied**, they derive from the owning module's `entity::Kind` static. Any
design that duplicates the identity into a second table regresses that.

### 2.2 Why the statics are pinned in kind modules

`entity::Kind` (engine, `src/entity.rs:89`) is identity **plus behaviour**:

```rust
pub(crate) struct Kind {
    pub dir: &'static str,       // .doctrine/slice
    pub prefix: &'static str,    // SL   (already sourced from leaf kinds::SL)
    pub stem: &'static str,      // slice → slice-204.toml
    pub scaffold: fn(&ScaffoldCtx<'_>) -> anyhow::Result<Fileset>,  // ← the pin
}
```

`scaffold` points at kind-module template fns, so a leaf can hold neither the
statics nor references to them. Wart: `rec`/`review`/`revision` materialise
eagerly (`materialise_fresh_prebuilt`, which never calls scaffold) and define
**panic-stub scaffolds** existing "only to satisfy the `Kind` descriptor
`integrity::KINDS` references".

### 2.3 The back-edges and the wider consumer surface

The 4 two-cycles are all calls to integrity's ref helpers, not the validator:
`parse_canonical_ref` (backlog ×3, revision ×3), `ensure_ref_resolves` (rec,
backlog, review, revision), `parse_resolvable_ref`.

Crate-wide usage census: `KINDS` ×31, `parse_canonical_ref` ×30,
`kind_by_prefix` ×16, `parse_resolvable_ref` ×11, `ensure_ref_resolves` ×10 —
vs integrity-proper items (`run_validate`, `run_reseat`, `scan_kind`,
`is_disposable_prose`, `id_integrity_findings*`) ×~8. **~90% of integrity's
consumed surface is the table + parsers.** 26 files import `crate::integrity`;
engine-tier hits (`relation.rs`, `supersede.rs`) are `#[cfg(test)]`-only.

## 3. Decisions

### D1 — Mechanism: scaffold-extraction ("C"), not identity-wrap

`Kind` **drops the `scaffold` field**, becomes pure identity data, and moves to
the leaf. `materialise` takes the scaffold fn as an explicit parameter.

Rejected alternatives, sized empirically:

- **A. identity-wrap** (`Kind { id: KindIdentity, scaffold }`, leaf table of
  identities): does NOT dodge the signature churn — table rows feed
  `entity::id_path` / `relation::read_block` at consumer sites, so the 8
  `&Kind`-threading fns re-signature either way. Adds a ~200-site
  `kind.prefix → kind.id.prefix` rewrite across 28 files, keeps the 3 panic
  stubs and a permanent two-type indirection. A→C later re-pays the 200-site
  rewrite in reverse: strictly more total churn than C now.
- **B. init-from-leaf field copies**: falls apart — consumers thread
  `kref.kind: &Kind` into engine fns, which a leaf identity table can't hold.
- **duplicate leaf table + drift test**: regresses SL-031's no-copy decision.
- **trait / push-registration**: forbidden by design intent —
  `mem.pattern.entity.kind-is-data-not-trait` ("Kind is data, not a trait";
  ~2 dozen kinds, compile-time known, no plugin story). Registration order and
  non-const tables buy nothing here.

### D2 — Placement: extend leaf `kinds` as an umbrella

`src/kinds.rs` (already "the kind-identity vocabulary", already the source of
every `prefix:`) becomes:

```
src/kinds/
├── mod.rs      pure data: prefix consts + groupings (existing 147 lines)
│               + Kind struct + 27 Kind statics (+ *_DIR consts)
│               + KindRef + KINDS + the membership pin-test
└── resolve.rs  probes: kind_by_prefix, parse_canonical_ref,
                parse_resolvable_ref, ensure_ref_resolves, scanned_kinds
                (imports kinds data + fsutil + listing — all leaf)
```

- All-leaf umbrella: no `layering.toml` sub-classification rows needed
  (MixedUmbrella fires only on mixed altitudes;
  `mem.pattern.lint.module-split-needs-layering-entry`).
- `kinds/mod.rs` does `pub(crate) use resolve::*` — one retarget target for
  consumers.
- fsutil/listing are leaf; leaf-tier impure seams are established precedent
  (`root`, `git`, `tty`). The purity split is visible at file level.
- Rejected: separate sibling leaf for the resolvers (two retarget targets,
  splits kind-identity cohesion); brand-new module for everything (leaves
  `kinds` a stub vocabulary; `registry` name taken by the spec registry).

### D3 — Scaffold seam: explicit parameter on `materialise`

```rust
// entity.rs (engine) — ScaffoldCtx and Fileset stay here
pub(crate) type Scaffold = fn(&ScaffoldCtx<'_>) -> anyhow::Result<Fileset>;

pub(crate) fn materialise(
    root: &Path, kind: &Kind, scaffold: Scaffold, /* …existing args */
) -> anyhow::Result<Materialised>
```

- ~14 production + ~10 test call sites gain one argument, compiler-driven.
  Closure statics (backlog `|c| backlog_scaffold(ItemKind::Issue, c)`,
  knowledge ×7) move verbatim to the call site — non-capturing closures coerce
  to fn pointers.
- `materialise_fresh_prebuilt` never calls scaffold → **signature unchanged**;
  the 3 panic-stubs delete outright.
- Internal scaffold invocations (`entity.rs:374`, `entity.rs:477`) read the
  parameter.
- `materialise_named` (memory) doesn't ride `Kind.scaffold` — untouched.
- **Governance spine (RV-261 F-1):** `governance::run_new` materialises off the
  descriptor (`entity::materialise(&g.kind, …)`, `governance.rs:485`) — the
  scaffold it needs moves onto `GovKind` (D4), and `run_new` passes
  `g.scaffold`. The spine's other consumers (`list_rows`, `run_show`, boot
  projection) touch only identity fields — unaffected.

### D4 — Re-exports pin the blast radius

- `entity.rs`: `pub(crate) use crate::kinds::Kind;` — existing `entity::Kind`
  imports and all ~200 `.prefix`/`.dir`/`.stem` field accesses stay untouched
  (fields stay flat on `Kind`).
- Each kind module with a **plain `Kind` static** re-exports it
  (`pub(crate) use crate::kinds::SLICE_KIND;`) — call sites like
  `entity::rel_path(&SLICE_KIND, …)` untouched. Existing consumer→kind-module
  edges persist (command-tier, harmless); **no new** edges appear.
- **Governance/RFC kinds are NOT plain re-exports (RV-261 F-2).** `ADR_KIND`,
  `POLICY_KIND`, `STANDARD_KIND`, `RFC_KIND` are `GovKind` descriptors (inline
  `Kind` value + scaffold today), consumed *as `GovKind`* by the governance
  spine, boot projection, and relation scanners. Dual-surface plan:
  - leaf `kinds` gains four plain identity statics (`kinds::ADR_KIND` etc. —
    `Kind` only), which the `KINDS` rows reference directly (today's
    `&ADR_KIND.kind` → `&kinds::ADR_KIND`);
  - `GovKind` (`governance.rs:37`) becomes
    `{ kind: &'static Kind, scaffold: Scaffold, statuses, hidden }` — borrows
    the leaf identity, carries the scaffold D3 evicts from `Kind`;
  - the four `GovKind` descriptor statics **stay in their kind modules** under
    their existing names (they hold the scaffold fns, which cannot leave).
  Field access through `g.kind.dir` etc. auto-derefs — spine call sites
  compile unchanged; only `run_new`'s materialise call gains `g.scaffold` (D3).
  One identity, two viewers, still no copy.

### D5 — What stays in `integrity`

`run_validate`, `run_reseat`, `scan_kind`, `extract_entity_id`,
`is_disposable_prose`, `id_integrity_findings*` — the validate/reseat
machinery. Residual imports: `entity`, `meta` (engine) + `finding`, `fsutil`,
`git`, `listing`, `root` (leaf) → **`integrity` re-tiers command → engine** in
`layering.toml`.

### D6 — Tangle numbers deferred to execute-time (SL-203 precedent)

No predicted count is hard-coded anywhere. At execute time: measure the live
tangle (deliberately-low baseline → read `TangleGrew { actual }`), ratchet to
the measured tight value. Integrity's ejection from the core SCC may cascade
(mcp_server ejected −4, not −2); expect a large drop, assert only
"measured, recorded, lower".

## 4. Consumer Retarget

~98 refs across 26 files: `integrity::{KINDS, KindRef, kind_by_prefix,
parse_canonical_ref, parse_resolvable_ref, ensure_ref_resolves, scanned_kinds}`
→ `kinds::…`. Mechanical, compiler-enforced. The 4 back-cycles retarget to the
leaf and break.

Not freed (explicitly): `relation_query` stays command (pinned by `catalog`);
`catalog::scan` / `relation_graph` stay command (import kind modules directly
for scaffolds/labels beyond identity).

## 5. Code-Impact Summary (design-target selectors)

| path | change |
|---|---|
| `src/kinds.rs` → `src/kinds/mod.rs` | +Kind struct, 27 statics, *_DIR consts, KindRef, KINDS, pin-test |
| `src/kinds/resolve.rs` | new — the 5 ref helpers |
| `src/entity.rs` | Kind def out / re-export in; `Scaffold` type; materialise param; test kinds |
| `src/integrity.rs` | −13 kind imports, −table, −helpers; keeps validate/reseat |
| `src/adr.rs` `src/backlog.rs` `src/concept_map.rs` `src/knowledge.rs` `src/policy.rs` `src/rec.rs` `src/requirement.rs` `src/review.rs` `src/revision.rs` `src/rfc.rs` `src/slice.rs` `src/spec.rs` `src/standard.rs` | plain statics → re-exports; adr/policy/standard/rfc `GovKind` descriptors rewire to leaf identity + own scaffold (D4); materialise call sites +arg; 3 stubs die |
| `src/governance.rs` | GovKind → `{ kind: &'static Kind, scaffold, statuses, hidden }`; `run_new` passes `g.scaffold` |
| `src/catalog/hydrate.rs` `src/catalog/scan.rs` `src/commands/cli.rs` `src/commands/dep_seq.rs` `src/commands/doctor.rs` `src/commands/facet.rs` `src/commands/map.rs` `src/commands/relation.rs` `src/commands/supersede.rs` `src/commands/tag.rs` `src/commands/validate.rs` `src/doctor_checks.rs` `src/map_server/markdown.rs` `src/map_server/routes.rs` `src/priority/graph.rs` `src/priority/surface.rs` `src/reconcile.rs` `src/relation.rs` `src/relation_graph.rs` `src/relation_query.rs` `src/search.rs` `src/supersede.rs` | retarget `integrity::` → `kinds::` |
| `.doctrine/adr/001/layering.toml` | `integrity = "engine"`; measured `command` ratchet |
| `tests/architecture_layering.rs` | expectation updates if any hard-code integrity's tier |

## 6. Verification Alignment

- **VT-A** `tests/architecture_layering.rs` green with: integrity's 13
  up-reaches gone, 4 back-cycles broken, `integrity = "engine"`, `command`
  tangle ratcheted to the measured value (D6).
- **VT-B** Full validation + entity suites green **unchanged** —
  behaviour-preservation gate.
- **VT-C** The `KINDS` membership pin-test relocated with the table, still
  green (R-b drift surface stays accepted and hand-maintained — unchanged
  scope).
- **VA-A** `doctrine slice conformance` clean against the §5 selectors.
- **VA-B** Design confirms re-tier outcomes for the Tier-3 follow-up:
  `integrity` → engine now; `catalog`/`relation_graph` still command (§4).

## 7. Risks & Boundary Conditions

- **R1 (sprawl, from scope):** 13-module migration in one slice. Mitigation:
  every step is compiler-enforced (missing param, missing symbol); phase the
  plan kinds-first → entity seam → retargets → layering measure, each ending
  green.
- **R2 (new leaf edges):** kind modules gain `use crate::kinds::…` — downward,
  legal. Watch the gate delta, not just integrity's local count (scope R2).
- **R3 (extractor quirks):** crate-root type edges are pre-filtered by
  source-file existence (`mem` — gate pre-filter subtlety); regenerate the map
  with `dump_real_graph`, never hand-guess.
- **R4 (test-local kinds):** entity.rs's 6 test statics also drop the field and
  pass scaffolds at call sites — kept inside VT-B's "suites green".
- **A1 (assumption):** no consumer outside integrity needs `scaffold` through
  the table — verified: scaffold is invoked only inside `entity.rs` (374, 477).

## 8. Open Questions

None blocking. Naming freedom left to implementation: `resolve.rs` fn names
stay verbatim to keep the retarget mechanical.
