# Design: SL-103 — Estimate graph exposure

## 1. Overview

Wire the estimate (and value) facets shipped by SL-101 into the catalog/graph
hydration pipeline so graph tooling (Cordage) consumes facet metadata through a
stable, policy-free contract. The facet *models*, *parse*, *validate*, and *unit
resolution* already exist (`src/estimate.rs`, `src/value.rs`); SL-101 wired them
onto the show path (`SliceDoc`) only. This slice wires them onto the **scan/catalog
path**, which today carries no facet data.

Realises **FR-006 (REQ-274)** — per estimated node: id, kind, `lower`, `upper`,
project unit, relations/edges, lifecycle state — as an additive, policy-free
projection. No aggregation, traversal, or interpretation (PRD-014 non-goals).

## 2. Current vs target behaviour

**Current.** `Catalog::from_scanned` and `CatalogGraph::from_catalog` are pure
projections that carry identity, kind, path, title, status, edges, diagnostics —
**no facet data**. The scan shell (`scan_entities`) reads `(status, title)` via
`read_meta` and outbound edges via `outbound_for`; it never reads `[estimate]` /
`[value]`. The `catalog graph` / `catalog scan` JSON dumps therefore expose no
facet metadata. `EstimationConfig`/`ValueConfig` are parsed by `dtoml` but unused
(dead-code-expected); `estimate::resolve_unit`/`parse_optional` and the value
equivalents are dead-code-expected pending this consumer.

**Target.** The scan shell reads facets kind-agnostically off every entity TOML
and carries them `ScannedEntity → CatalogEntity → CatalogNode`. Project-wide units
are resolved once from `doctrine.toml` and exposed as a top-level `units` block on
`Catalog` / `CatalogGraph`. The `catalog graph` JSON gains `units` plus per-node
`estimate` / `value`. A malformed facet on one entity yields a loud diagnostic and
a node carried *without that facet* — never a silently repaired bound, never a
dropped node, never a failed scan.

## 3. Decisions

- **D1 — Expose estimate AND value.** The slice's FR-006 (REQ-274) names estimate
  only, but the generic scan-side reader carries both facets naturally and a
  symmetric contract is more coherent than a near-identical follow-up for value.
  *Governance: value graph exposure is **deliberate, traced-pending** scope, not
  an oversight — no existing REQ covers it (REQ-274 is estimate-only; REQ-278/279/
  280 govern the value **model/validation/unit**, SL-101's territory, not graph
  exposure). The requirement is authored and given a spec home at reconcile —
  see §7. The value unit this slice resolves (§5.4) realises REQ-280; SL-103
  traces to it at reconcile alongside the new exposure REQ. [RV-094 F-1]*
- **D2 — Units are a top-level block, not per-node.** The estimation/value units
  are project-wide constants (`doctrine.toml`), not properties of any node.
  Carrying them once on `Catalog`/`CatalogGraph` is DRY and avoids encoding false
  per-node ownership. FR-006 is satisfied — the unit is in the contract and
  reachable from every node via the graph the consumer already holds.
- **D3 — Read facets in the scan shell (A1), isolated from the `Meta` seam.** A
  dedicated `read_facets` helper reads `[estimate]`/`[value]` off any
  `<stem>-NNN.toml`, kind-agnostic, reusing the leaf `parse_optional`. It is *not*
  folded into `read_meta`/`Meta` — SL-101 deliberately kept the list path
  facet-free, and folding would perturb the RV/REC special-cases. Cost: a second
  small TOML parse per entity (≈corpus size, negligible on a tooling/map surface).
- **D4 — Malformed facet → diagnostic + node-without-that-facet, per-facet.**
  `EstimateFacet::Deserialize` hard-fails on a malformed present table (SL-101
  "fail loud, never repair"). On the corpus-wide scan, failing the whole scan or
  dropping the node is brittle. Instead each facet parses independently; a failure
  pushes an `Error` `CatalogDiagnostic` (loud) and drops *that* facet to `None`
  (no bound coercion → no silent repair), leaving the node and the sibling facet
  intact. This mirrors the existing per-entity diagnostic+continue pattern.
  *Contract note [RV-094 F-2]:* on the wire, a malformed-present facet is
  **indistinguishable from an absent one** — both omit the key (`skip_serializing_if`);
  the corruption is observable only out-of-band in the `Error` `CatalogDiagnostic`
  stream. This is the **conscious v1 contract**: the loud diagnostic is the signal,
  the graph stays policy-free, and "never a repaired bound" is honoured (no coercion).
  A consumer that must distinguish the two reads the diagnostics. If a future
  consumer needs an in-band present-but-invalid marker, that is an additive contract
  change, not a silent one.
- **D5 — Reuse the leaf serialisers; no new contract types.** `EstimateFacet`
  (`{lower, upper}`) and `ValueFacet` (`{value}`) already derive `Serialize`. The
  contract is those types behind `Option` with `skip_serializing_if`, plus a new
  `Units`. No DTO duplication. *Coupling note [RV-094 F-7]:* because `/api/graph`
  serves `CatalogGraph` raw (§6.3 F-3), these leaf structs **are** the external
  contract — a field added to `EstimateFacet`/`ValueFacet` for internal reasons
  changes the public wire shape. Accepted consciously: the facet structs are
  **change-controlled contract types**, not free-to-mutate internals. A
  serialisation-DTO seam is deferred unless/until model and contract need to
  diverge.

## 4. Data structures (the contract)

### 4.1 `ScannedEntity` (scan.rs)

```rust
pub(crate) struct ScannedEntity {
    // …existing: key, kind, status, title, outbound…
    pub(crate) estimate: Option<crate::estimate::EstimateFacet>,
    pub(crate) value: Option<crate::value::ValueFacet>,
}
```

### 4.2 `CatalogEntity` (hydrate.rs)

```rust
pub(crate) struct CatalogEntity {
    // …existing fields…
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) estimate: Option<crate::estimate::EstimateFacet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<crate::value::ValueFacet>,
}
```

### 4.3 `CatalogNode` (graph.rs) — projected from the entity

```rust
pub(crate) struct CatalogNode {
    // …existing: title, status, kind_label, memory_type…
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) estimate: Option<crate::estimate::EstimateFacet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<crate::value::ValueFacet>,
}
```

### 4.4 `Units` — top-level (hydrate.rs)

```rust
#[derive(Debug, Clone, serde::Serialize)]  // match the containers' derives
pub(crate) struct Units {
    pub(crate) estimation: String,  // resolved; default "espresso_shots"
    pub(crate) value: String,       // resolved; default "magic_beans"
}
```

Carried on `Catalog { …, units: Units }` and copied onto
`CatalogGraph { …, units: Units }`.

### 4.5 Resulting `catalog graph` JSON contract

```json
{
  "units": { "estimation": "espresso_shots", "value": "magic_beans" },
  "nodes": {
    "SL-101": {
      "title": "Estimate & Value facets",
      "status": "done",
      "kind_label": "SL",
      "estimate": { "lower": 2.0, "upper": 8.0 },
      "value": { "value": 5.0 }
    },
    "SL-072": { "title": "…", "status": "…", "kind_label": "SL" }
  },
  "edges": [ /* unchanged CatalogEdge list */ ]
}
```

Non-faceted nodes omit `estimate`/`value` entirely. Edges unchanged
(relations/edges already satisfy that contract element); lifecycle = `status`
(already present). Field vocabulary (`units`/`estimate`/`value`/`lower`/`upper`)
is clear of the whole-word denylist (`project`/`task`/`schedule`/`capacity`).

## 5. Read path & wiring

### 5.1 `read_facets` (scan.rs) — kind-agnostic, per-facet isolation

```rust
fn read_facets(
    root: &Path, kref: &integrity::KindRef, id: u32,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> (Option<EstimateFacet>, Option<ValueFacet>) {
    let name = format!("{id:03}");
    let path = root.join(kref.kind.dir).join(&name)
        .join(format!("{}-{name}.toml", kref.stem));
    // The status read already validated this file parses; a vanished/garbled file
    // here is a concurrent-edit window (the status read is the authority and will
    // re-diagnose next scan). v1 returns absent rather than re-report [RV-094 F-6].
    // NOTE: the cleaner fix is to read each entity TOML ONCE and derive
    // status/title/facets from the one table — eliminating both this second parse
    // (D3) and the divergent-read window. Recorded for reconcile; out of scope here.
    let Ok(text) = std::fs::read_to_string(&path) else { return (None, None) };
    let Ok(table) = text.parse::<toml::Table>() else { return (None, None) };

    let estimate = parse_facet("estimate", table.get("estimate"),
        crate::estimate::parse_optional, root, kref, id, diagnostics);
    let value = parse_facet("value", table.get("value"),
        crate::value::parse_optional, root, kref, id, diagnostics);
    (estimate, value)
}
```

`parse_facet` (a small generic helper over the two leaf `parse_optional`
signatures): if the key is present but not a TOML table → `Error` diagnostic +
`None` (fail-loud, not silent-absent); else `.and_then(Value::as_table)` →
`parse_optional`; `Ok(f) => f`; `Err(e) => { push Error diagnostic; None }`. The
diagnostic carries the entity dir, the entity key, `field = Some("estimate")`/
`Some("value")`, and the leaf's error message verbatim.

### 5.2 `scan_entities`

After the existing status/title and outbound reads, call `read_facets` and push
`estimate`/`value` onto the `ScannedEntity`. No change to the status/relation
diagnostic+continue arms.

### 5.3 `from_scanned` (hydrate.rs)

New signature `from_scanned(root, scanned, memory, mem_key_map, units)`. Copies
`se.estimate.clone()` / `se.value.clone()` onto each numbered `CatalogEntity`;
memory entities carry `None` (memory has no facets). Stays pure — `units` is an
input, not a read. Stores `units` on the returned `Catalog`.

### 5.4 `scan_catalog` (signature unchanged)

Resolves units in the shell, mirroring `coverage_store::load_config` **faithfully**
— default only on `NotFound`, propagate every other read error and all parse
errors. A swallow-all `Err(_) => default` (the prior draft) would mask permission
/ transient-I/O faults as "no config → default units", silently mis-resolving the
contract on a corrupt environment [RV-094 F-4]:

```rust
let cfg = match std::fs::read_to_string(root.join("doctrine.toml")) {
    Ok(text) => crate::dtoml::parse(&text)?,                 // parse errors propagate
    Err(e) if e.kind() == std::io::ErrorKind::NotFound =>
        crate::dtoml::DoctrineToml::default(),               // absent file → defaults
    Err(e) => return Err(e.into()),                          // permission/I-O → loud
};
let units = Units {
    estimation: crate::estimate::resolve_unit(&cfg.estimation),
    value: crate::value::resolve_unit(&cfg.value),
};
let mut catalog = Catalog::from_scanned(root, &scanned, &memory, &mem_key_map, units);
```

### 5.5 `CatalogGraph::from_catalog`

Project `estimate`/`value` onto each `CatalogNode`; copy `catalog.units`.

### 5.6 Dead-code `expect` cleanups (now live)

These symbols gain live call sites — their `#[cfg_attr(not(test), expect(dead_code,…))]`
must be removed or clippy fires *unfulfilled-expect*:

- `estimate::parse_optional`, `value::parse_optional`
- `estimate::resolve_unit`, `value::resolve_unit`
- `dtoml::DoctrineToml::{estimation, value}` fields

**Stay dead (leave their expects):** confidence consts / `resolve_confidence`,
`estimate::display` module — owned by SL-102, not this slice.

## 6. Verification

### 6.1 By test (VT)

- **VT-1** Faceted slice → graph node carries `estimate{lower,upper}` + `value{value}`;
  top-level `units` resolved.
- **VT-2** Non-faceted entity → node present; `estimate`/`value` omitted
  (skip_serializing_if).
- **VT-3** Unit resolution: configured `[estimation].unit`/`[value].unit` surface;
  absent → `espresso_shots`/`magic_beans`.
- **VT-4** Malformed estimate (`upper < lower`) → `Error` diagnostic + node carried
  with estimate `None`, **and** a valid `[value]` on the same entity still present
  (per-facet isolation).
- **VT-5** Round-trip durability (FR-004 tie): normalized bounds identical
  scan → catalog → graph.
- **VT-6** Kind-agnostic: `[estimate]` authored on a non-slice TOML (e.g. ADR)
  surfaces in the graph — proves the generic read, not slice-only.
- **VT-7** Contract JSON shape: serialize the graph; assert `units`,
  `nodes[*].estimate|value`, `edges` keys and graph-neutral field names.

### 6.2 By agent (VA)

- **NF-001 structural non-blocking:** attest no dispatch/execute/audit/close
  predicate reads facet presence — proven by absence of such a read, not a passing
  run.
- **Vocabulary:** contract field names clear of the whole-word denylist.

### 6.3 Behaviour preservation

Existing hydrate/graph/map_server suites stay green. The additive `units` key and
the new struct fields require updating direct construction sites
(`map_server/routes.rs` `CatalogNode { … }`, graph/hydrate test literals) and the
single `from_scanned` call site (`scan_catalog` — its only caller; no test invokes
it directly) — additive contract evolution, not regression.

**HTTP surface — corrected [RV-094 F-3].** Two map_server endpoints, different
exposure:
- `/api/map` (and the rendered web view) maps `CatalogNode` → its own
  `{key,label}` DTO explicitly and picks no facet fields — **unaffected**.
- `/api/graph` (`map_server/routes.rs` `graph()`) serialises the whole
  `CatalogGraph` directly (`Json(snapshot)`), so the additive top-level `units`
  and per-node `estimate`/`value` **will surface on this endpoint**. This is
  **accepted as in scope** (per the F-3 ruling): `/api/graph` is the Cordage-facing
  raw-contract endpoint, and exposing the same policy-free facet contract there as
  on `catalog graph` is coherent, not a leak. The earlier "HTTP view unaffected,
  out of scope" claim was wrong and is retracted. The rendered *web map UI* remains
  out of scope (it reads the `{key,label}` DTO, not the facets).

## 7. Open / governance

- **Value graph exposure — orphan requirement, homed at reconcile [RV-094 F-1].**
  D1 deliberately exposes value past REQ-274 (estimate-only). No existing REQ
  covers value *graph exposure*: REQ-274 is estimate-only, and REQ-278/279/280
  govern the value *model / validation / unit resolution* (SL-101), not its
  projection onto the graph. This is **intentional scope**, recorded here so it
  does not ship silently un-traced.
  - **Reconcile obligation (the orphan):** author a value-graph-exposure
    requirement (sibling in intent to REQ-274) and bind SL-103 to it; decide its
    spec home then — SPEC-020 is titled "Estimate graph exposure", so value may
    warrant a renamed/extended SPEC-020 or its own spec. *(The CLI cannot mint an
    orphaned, un-homed requirement — `spec req add` couples reservation to a spec
    membership — so the requirement is captured here as intent and formally minted
    + homed during reconcile, where the spec home is chosen.)*
  - **Also trace at reconcile:** SL-103 → REQ-280 (value unit resolution), which
    §5.4 realises.
- **D2 unit literalism [RV-094 F-5].** D2 satisfies REQ-274's "project unit" by
  reachability (top-level block), not literal per-node duplication. Sound (units
  are project-wide constants), but ratify REQ-274's intent at reconcile rather
  than assume it.

## 8. Code impact summary

- `src/catalog/scan.rs` — `ScannedEntity` +2 fields; new `read_facets` + helper;
  `scan_entities` call.
- `src/catalog/hydrate.rs` — `CatalogEntity` +2 fields; new `Units`; `Catalog`
  +`units`; `from_scanned` +param & projection; `scan_catalog` unit resolution.
- `src/catalog/graph.rs` — `CatalogNode` +2 fields; `CatalogGraph` +`units`;
  `from_catalog` projection.
- `src/estimate.rs`, `src/value.rs` — remove now-fulfilled `expect(dead_code)` on
  `parse_optional`/`resolve_unit`.
- `src/dtoml.rs` — remove `expect(dead_code)` on `estimation`/`value` fields.
- `from_scanned` call site (`scan_catalog`, its sole caller) + `CatalogNode`
  struct literals (`map_server/routes.rs`, graph/hydrate tests) updated for the
  new fields.
- No change to `Meta`/list path, RV/REC readers, or the map_server HTTP view.
