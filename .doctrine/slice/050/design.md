# SL-050 — Priority surface cleanup: design

Canonical technical design for the seven-finding cleanup of the SL-047 priority
surfaces. Scope, provenance, and non-goals live in `slice-050.md`; this file is
the *how*. No behaviour the operator relies on changes except (a) the `explain`
order label is removed and (b) the keyed read surfaces refuse a non-existent id
instead of rendering a clean empty result.

The seven findings collapse into two design sections: §1 the shared-scan +
existence-gate restructure (F2 + F6 — the foundational signature change), §2 the
six smaller findings that ride on it (F1, F3, F4, F5, F7).

---

## §1 — Shared scan (F2) + keyed-surface existence gate (F6)

### Current behaviour

A single-id `inspect` runs **two** corpus `scan_entities` walks:

- `relation_graph::render`/`inspect` → `build_relation_graph(root)` → `scan_entities` (#1)
- `priority::surface::actionability_block(root)` → `graph::build(root)` → `scan_entities` (#2)

Walk #1 also derives `status_for` + `title_for` for every entity, neither of
which `build_relation_graph` consumes (it reads only `key` + `outbound`).

Separately, every keyed read surface that resolves to nothing renders a clean,
*indistinguishable-from-a-real-isolated-node* result: `inspect` an empty-section
view (its VT-5 contract), `explain`/`blockers` empty lists, `actionability_block`
an all-empty block. `parse_key` validates ref *shape* only; a well-formed ref to
an unminted id (`SL-999`) sails through every `None`-returning lookup. By
contrast `doctrine <kind> show <missing>` **errors** — so `inspect`'s own "mirror
a show-like read surface" rationale (VT-5) is already self-violated.

### Target behaviour

One corpus scan per `inspect` composition, performed at the command layer
(`run_inspect`). ADR-001 permits `main` to depend on both `relation_graph` and
`priority`; the two modules still may not call each other. Builders gain
pre-scanned `_from` entry points; the existing `root` wrappers delegate (scan
then `_from`), so the five standalone priority surfaces and standalone
`inspect`/`render` are untouched (each still scans once — that was never the
defect).

All four keyed surfaces refuse a non-existent id with one pinned message.

### Seam shape

Thin delegating wrappers over `_from` bodies — the real work moves into the
`_from` fns; the `root` wrappers become `scan_entities(root)?` + delegate:

```rust
// relation_graph.rs — build_relation_graph touches no disk beyond the scan,
// so its _from takes only the slice.
fn build_relation_graph_from(scanned: &[ScannedEntity]) -> anyhow::Result<RelationGraph>;
fn build_relation_graph(root: &Path) -> anyhow::Result<RelationGraph>;          // scan + _from

pub(crate) fn inspect_from(scanned: &[ScannedEntity], root: &Path, id: &str)
    -> anyhow::Result<InspectView>;                                             // keeps root: the
pub(crate) fn inspect(root: &Path, id: &str) -> anyhow::Result<InspectView>;   // queried entity's
                                                                               // own outbound re-read
pub(crate) fn render_from(scanned: &[ScannedEntity], root: &Path, id: &str, fmt: Format)
    -> anyhow::Result<String>;
pub(crate) fn render(root: &Path, id: &str, fmt: Format) -> anyhow::Result<String>;

// priority/graph.rs — build_from still needs root for the per-backlog dep_seq
// reads (step 3b); those are not part of scan_entities.
pub(crate) fn build_from(scanned: &[ScannedEntity], root: &Path) -> anyhow::Result<PriorityGraph>;
pub(crate) fn build(root: &Path) -> anyhow::Result<PriorityGraph>;             // scan + _from

// priority/surface.rs
pub(crate) fn actionability_block_from(scanned: &[ScannedEntity], root: &Path, id: &str)
    -> anyhow::Result<ActionabilityBlock>;
pub(crate) fn actionability_block(root: &Path, id: &str) -> anyhow::Result<ActionabilityBlock>;
```

`inspect_from` still performs the queried entity's *own* `outbound_for` re-read +
`render_human`'s interaction-type re-read. Those are per-entity, not corpus —
outside F2's "two full corpus scans" target — and are left as-is to bound churn.

### Existence gate — `require_minted`

One shared helper, pinned message, in `relation_graph` (where `EntityKey` lives;
`priority` already imports it):

```rust
// Err: "{}: no such entity", key.canonical()   e.g.  "SL-999: no such entity"
pub(crate) fn require_minted(scanned: &[ScannedEntity], key: EntityKey) -> anyhow::Result<()>;
```

Applied at all four keyed surfaces:

- `inspect` — replaces the empty-view early return (which also guarded
  `outbound_for` off a missing file; the bail does the same, earlier).
- `explain` / `blockers` / `actionability_block` — after `graph::build`, via the
  projection: `g.projection.resolve(key).is_none()` ⇒ the same error. (These hold
  a `PriorityGraph`, not the scanned slice; the projection is the existence
  oracle — it contains exactly the scanned/minted keys. A second
  projection-based overload or a shared inner check keeps the message single-
  sourced.)

`run_inspect` checks once, up front, so its two `_from` calls cannot trip it:

```rust
let key = parse_key(id)?;                              // shape-validate
let scanned = relation_graph::scan_entities(&root)?;   // ONE walk
relation_graph::require_minted(&scanned, key)?;        // F6 gate, once
// table: relation_graph::render_from(&scanned, &root, id, Table)
// json:  relation_graph::inspect_from(&scanned, &root, id) → inspect_value
// block: priority::surface::actionability_block_from(&scanned, &root, id)
```

### Decisions

- **D1.** The gate lives *in the keyed surfaces*, not only the command layer, so
  standalone `explain`/`blockers` refuse identically. One pinned message.
- **D2.** Message `<CANONICAL>: no such entity` — cross-kind, no path. Diverges
  deliberately from `show`'s per-kind `slice 999 not found at <path>` wording
  (these surfaces are kind-agnostic; a path would have to pick a kind tree).
- **D3.** Single-scan property is a *structural* guarantee of the seam (the two
  redundant `scan_entities` calls are gone from `run_inspect`); not asserted via
  a scan counter. Existing real-id goldens staying byte-identical is the
  behavioural proof.

### Invariants preserved

- Scan order (KINDS table / id-ascending, permutation-invariant — REQ-077)
  unchanged: the single scan IS `scan_entities`, same order both consumers saw.
- `inspect` relation-portion bytes for *existing* ids unchanged (the non-goal
  byte-identical gate); only the missing-id path changes (empty view → error).
- Pure/imperative split: `_from` bodies are pure over the scanned slice (plus
  `build_from`'s explicit `root` for dep_seq); disk stays in `scan_entities` +
  the per-entity re-reads.

---

## §2 — The remaining six findings

### F1 — kill the double parse (`relation_graph::scan_entities`)

`status_for` deserializes the full `meta::Meta` (which carries `title`), then
`title_for` re-opens and re-parses the *same* toml into a `TitleOnly` struct —
two parses per non-RV/REC entity per scan. Merge into one combined reader:

```rust
fn status_and_title_for(root: &Path, kref: &integrity::KindRef, id: u32)
    -> anyhow::Result<(Option<String>, String)> {
    match kref.kind.prefix {
        "REC" => Ok((None, title_for(root, kref, id)?)),               // lenient title; no status
        "RV"  => Ok((Some(crate::review::derived_status_string(root, id)?),
                     title_for(root, kref, id)?)),                     // derived status + lenient title
        _     => { let m = crate::meta::read_meta(&root.join(kref.kind.dir), kref.stem, id)?;
                   Ok((Some(m.status), m.title)) }                     // ONE parse → both
    }
}
```

`title_for` survives as the RV/REC lenient reader (their strict `read_meta` fails
for lack of a top-level `status`). `scan_entities` calls `status_and_title_for`
once per entity. Common path: one `<stem>-NNN.toml` parse. RV/REC: one lenient
parse (+ RV's separate finding-ledger read, which is a different file). "One
parse per entity" is structural (see D3 — not instrumented).

### F3 — `survey` decorate-sort-undecorate

`sort_by` calls `actionability` (→ `blocked` → `blocked_by`: `in_edges` walk +
`BTreeSet` + per-predecessor `class_of`) and `consequence` for *both* operands on
every comparison; the subsequent `map` recomputes the same per row. Materialise
each node's signals once, then sort + map over the decorated set:

```rust
struct Row { key: EntityKey, act: Actionability, consequence: u32, blockers: Vec<String> }
let mut rows: Vec<Row> = keys.into_iter().map(|k| Row {
    key: k,
    act: actionability(&g, k),
    consequence: channels::consequence(&g, k),
    blockers: refs(&channels::blocked_by(&g, k)),
}).collect();
rows.sort_by(|a, b| act_rank(a.act).cmp(&act_rank(b.act))
    .then(b.consequence.cmp(&a.consequence))
    .then_with(|| a.key.cmp(&b.key)));
// map → SurveyRow reusing a.act / a.consequence / a.blockers; reasons built from them
```

The comparator does zero graph work. Output order + bytes identical — pure
refactor; the survey goldens hold unchanged.

### F4 + F5 — drop `OrderContrib`, leaving one transitive walk

`OrderContrib` carried `dep_level` (mislabelled "dep-topology level"; it is the
*count* of transitive non-terminal blockers — and it equals `len(blocker_chain)`,
which `explain` already emits) plus `seq_rank`, which is *always* `None`. It
carries no information not already in `blocker_chain`. Drop it whole:

- **view.rs** — remove `ReasonKind::OrderContrib`; remove the `order_contrib`
  field from `Explanation`.
- **surface.rs `explain`** — delete the order-contrib block. That block contained
  the *second* `blocked_by_transitive(&g, key)` walk (the F4 double walk); deleting
  it leaves only the first `chain` walk that feeds `blocker_chain`. F4 is resolved
  as a consequence of the F5 drop — no "reuse the result" plumbing needed.
- **render.rs** — remove the `OrderContrib` human arm (`order: dep-level …`), its
  JSON arm, the `order_contrib` field in the JSON object, and
  `parts.push(reason_line(&ex.order_contrib))`.

**Goldens:** `explain` human output loses the `order:` line; `explain --json`
loses the `order_contrib` field.

### F7 — resolve the remaining dead vocabulary (drop, not wire)

Wiring a consumer for these would be a new per-entity dangler query — a non-goal.
Drop:

| Item | Action |
|---|---|
| `view::ReasonKind::Fallback` | drop variant + its human/JSON render arms (with F5) |
| `OrderContrib.seq_rank` always `None` | drops with `OrderContrib` (F5) — kills render's dead `Some` arm + JSON field |
| `graph::Dangling` struct + `dangling` field | drop struct + field; the edge pass's `else`/`None` arms become no-ops (an unresolved target already contributes no edge — we only stop *recording* it) |
| `graph::ref_overlays: Vec<OverlayId>` field | drop the field + the `Vec`; keep the local `ref_by_label` map + per-label overlay creation (the edge pass needs them) |

All **five `#[expect(dead_code)]` suppressions retire** (the two view-layer +
three graph-layer tells the review flagged).

**Verification impact:** the graph tests assert the dead artifacts
(`pg.ref_overlays.len()` and the `ref_overlays`-vs-dep/seq disjointness asserts
~512–520; `pg.dangling` contents ~697–745). Rewrite: drop the dead-artifact
assertions; **preserve the behavioural ones** in the same tests — the
`dangling`-target cases were really asserting "an unresolved target produces no
edge"; re-express that against the graph's edge set (the dep/seq overlays), where
the assertion still carries behavioural weight, rather than against a removed
`dangling` Vec. Detail deferred to plan/execute.

---

## Verification summary

| Surface | Change | Evidence |
|---|---|---|
| `explain` (human + json) | `order:` line / `order_contrib` field removed (F5) | golden update |
| `inspect` missing id | empty view → `… : no such entity` error (F6) | VT-5 flips |
| `explain` / `blockers` missing id | empty result → error (F6) | new test |
| `survey` / `next` / `blockers` / `inspect` real ids | byte-identical (F1/F2/F3/F4/F7) | existing goldens hold (13 priority + 9 inspect) |
| graph tests | dead-artifact assertions dropped, behavioural ones re-expressed (F7) | unit tests |

Gate: `just check` green; `cargo clippy` zero warnings (the five `dead_code`
suppressions are gone, not relocated).

## Doctrinal alignment

- **ADR-001 (layering)** — the single scan lives at the command layer
  (`run_inspect`); `relation_graph` and `priority` still never call each other.
  `require_minted` lives in `relation_graph` (owns `EntityKey`); `priority`
  imports downward, no up-call.
- **ADR-004 (outbound-only relations)** — untouched; consequence/inbound stay
  derived, nothing stores a reverse field.
- **REQ-072 (reasons are the render source of truth)** — preserved and tightened:
  one fewer reason kind, still built once in the surface/view layer, the renderer
  only formats.
- **REQ-077 (determinism / permutation-invariance)** — preserved: scan order
  unchanged; the F3 decorate-sort uses the same total order
  (`act → consequence desc → canonical-id asc`).
- **Non-goal amendment** — `slice-050.md` "No `inspect` relation-portion output
  change" gains a carve-out: missing-id output legitimately changes (empty →
  error) per F6; existing-entity output stays byte-identical.
