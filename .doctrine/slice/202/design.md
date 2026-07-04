# Design SL-202: Memory body wikilinks as catalog edges

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Memories cross-reference two disjoint ways. **TOML relations** (`[[relation]]`
rows in `memory.toml`) are read by the catalog and drawn as web-map edges. **Body
wikilinks** (`[[mem.<key>]]` in the `.md` body) are read by the CLI
(`links::extract_wikilinks` → `resolve-links`/`backlinks`/`show`) but **never by
the catalog**. A memory whose associations live only in body prose — the shipped
onboarding corpus does exactly this (`See [[mem.signpost.doctrine.lifecycle-
start]]`) — draws zero web-map edges. Make body wikilinks a first-class catalog
edge source alongside TOML relations.

Originates from ISS-214; enabled by ISS-213 (landed `93880c77`).

## 2. Current State

- `scan_memory_entities` (`src/catalog/scan.rs:449`) walks both corpora and calls
  `read_catalog_record` (`src/memory.rs:1291`) per memory. `MemoryCatalogRecord`
  (`src/memory.rs:1278`) carries `uid/key/title/status/memory_type/relations/path`
  — **no body**.
- `Catalog::from_scanned` (`src/catalog/hydrate.rs:206`) is a **pure** projection.
  The memory loop (`hydrate.rs:293-367`) hydrates the node with `body: None`
  (`hydrate.rs:305`) and emits edges solely from `record.relations`, each a
  `CatalogEdgeLabel::Raw(relation.label)` with `role: None`.
- `classify_target` (`hydrate.rs:389`) resolves a raw target string: canonical ref
  → `Numbered`; else memory `mem_<uid>` or `mem.<key>` (via `mem_key_map`) →
  `Resolved(CatalogKey::Memory)`; else `UnvalidatedText`. **Reused verbatim.**
- `extract_wikilinks(body) -> Vec<Wikilink { target, is_uid }>` (`src/links.rs:16`)
  is fence-aware and inline-code-aware, regex-gated to `mem.`/`mem_` prefixes.
  **Reused verbatim.**
- ISS-213: `mem_key_map` spans both corpora keyed on full `mem.…` `memory_key`;
  `classify_target` already resolves both target forms — no new resolver needed.

## 3. Forces & Constraints

- **ADR-001** (leaf ← engine ← command, no cycles): scan/hydrate/memory are engine
  tier; the change adds no new dependency edge.
- **ADR-016** (relation intent = closed role dimension): a `Raw` edge carries no
  typed role. Body wikilinks are untyped → `role: None`. Aligned.
- **STD-001** (no magic strings): the synthesized edge label is a single-source
  named constant, never an inline literal.
- **Purity gate** (slices-spec § Architecture; mirrored by `resolve_units`): no
  disk I/O in the pure `from_scanned`. Body read lives in the impure scan layer
  (`read_catalog_record`); `from_scanned` stays pure over record data.
- **Behaviour-preservation gate**: existing TOML-edge and CLI-backlink suites are
  the proof — they must stay green unchanged.

## 4. Guiding Principles

- **Ride existing seams — no parallel implementation.** Reuse `extract_wikilinks`,
  `Wikilink`, `classify_target`, `EdgeTarget`. No new resolution logic.
- **One I/O pass.** Body read folded into the existing `read_catalog_record` read,
  not a second directory walk.
- **Exact dedup on resolved identity**, not on raw string.

## 5. Proposed Design

### 5.1 System Model

Two edge sources feed the memory node, unified in one hydration loop:

```
read_catalog_record  ──reads──▶ memory.toml (relations) + memory.md (body)   [impure]
        │
        ▼
MemoryCatalogRecord { …, relations, body: Option<String> }
        │
        ▼
Catalog::from_scanned  (pure)
  memory loop, per record:
    (a) TOML relations  → classify_target → Raw(author label) edges   [existing]
        └─ record each Resolved target into `seen`
    (b) body wikilinks  → extract_wikilinks → classify_target
        └─ skip if Resolved target already in `seen`  (dedup)
        └─ else Raw(MEMORY_WIKILINK_EDGE_LABEL) edge, origin field = "body"
```

### 5.2 Interfaces & Contracts

- `MemoryCatalogRecord` gains `pub(crate) body: Option<String>`.
- `read_catalog_record(toml_path)` additionally reads the sibling `memory.md`
  (`toml_path.parent()/memory.md`); a missing/unreadable body → `None` (tolerant,
  mirroring numbered-entity `read_body`). Read error on the `.md` is NOT fatal —
  a memory with no body is legal.
- `from_scanned` signature unchanged. The memory loop internally builds a
  per-record `seen: BTreeSet<CatalogKey>` and runs the two passes above.
- Two new constants (STD-001), in `hydrate.rs` beside the edge-construction site:
  `const MEMORY_WIKILINK_EDGE_LABEL: &str = "related";` and the origin marker
  `const MEMORY_WIKILINK_ORIGIN_FIELD: &str = "body";`. Filename literals
  (`"memory.md"`) are NOT constant-ized — the codebase inlines them by convention
  (`corpus.rs:370`: `fs::read_to_string(dir.join("memory.md")).ok()`, the exact
  missing→`None` idiom to mirror).

### 5.3 Data, State & Ownership

- Body string owned by `MemoryCatalogRecord`, read once, dropped after hydration.
  The memory `CatalogEntity.body` stays `None` — bodies are consumed transiently
  for edge extraction; no consumer renders memory prose (non-goal).
- `seen` is per-record, local to the loop iteration — no cross-record state.
- Dedup identity = `CatalogKey` (the `Resolved` variant's payload). Unresolved
  targets carry no dedup key and never suppress an edge (same as the TOML path).
- **Naming (F-3):** `MemoryCatalogRecord.body` (populated, transient, source of
  wikilinks) and `CatalogEntity.body` (stays `None`) coexist in one loop with
  opposite lifecycles. Disambiguate at the site with a one-line comment
  (`// record.body feeds edge extraction; the entity node carries no prose`) so a
  reader does not mistake one for the other.

### 5.4 Lifecycle, Operations & Dynamics

Per memory record, in `from_scanned`:

1. TOML relation pass (existing) — for each emitted edge whose target is
   `Resolved(k)`, `seen.insert(k)`.
2. Body wikilink pass (new) — `extract_wikilinks(body)`; for each `Wikilink`:
   - `let target = classify_target(&link.target, &key_set, mem_key_map);`
   - if `Resolved(k)` and `!seen.insert(k.clone())` → **continue** (deduped). The
     `seen.insert` here also dedups **body-vs-body** — a second body wikilink to
     the same resolved target is suppressed.
   - if `UnvalidatedText` → push a `Warning` diagnostic. **This is the ONLY
     unresolved outcome a `mem.`-prefixed body wikilink can produce** (F-1):
     `parse_canonical_ref` fails on a `mem.…` string, the memory branch misses,
     and `classify_target` returns `UnvalidatedText` — the `UnresolvedRef` arm is
     **unreachable** from this pass (it requires a *numbered* ref) and is not
     handled here. The warning is a **deliberate divergence** from the TOML path,
     which is **silent** on `UnvalidatedText` (`hydrate.rs:341` warns only on
     `UnresolvedRef`) — NOT "parity". Justified by the `mem.word.word` shape-gate:
     a well-formed-but-unresolvable memory key is near-certainly a real dangling
     reference, not prose (owner ruling). **Scoped to the body pass** — neither
     `classify_target` nor the shared TOML diagnostic is touched, so TOML
     `UnvalidatedText` targets stay silent and behaviour-preservation holds.
   - push `CatalogEdge { source: Memory(uid), label: Raw(MEMORY_WIKILINK_EDGE_
     LABEL), role: None, descriptor: None, target, origin: EdgeOrigin { file:
     memory.md, field: MEMORY_WIKILINK_ORIGIN_FIELD } }`.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (one-directional dedup — F-2):** a **body-wikilink** edge is suppressed
  when a TOML relation OR a prior body wikilink already draws an edge to the same
  resolved target — body defers to TOML, and body-vs-body collapses. This is NOT a
  global uniqueness rule: **TOML-vs-TOML multiplicity is unchanged** — two TOML
  relations to one target still draw two edges (pre-existing behaviour,
  `hydrate.rs:312-366`, preserved; the TOML pass populates `seen` but never checks
  it). Dedup is enforced only in the body pass.
- **INV-2 (F-1):** a body wikilink whose target is `UnvalidatedText` emits exactly
  one edge + one `Warning`, never suppressed by dedup (unresolved targets carry no
  dedup key). `UnresolvedRef` does not arise in this pass.
- **EDGE:** wikilink whose target is the memory itself (self-link) — resolves and
  draws a self-edge; acceptable, matches TOML self-relation behaviour (no special
  case). Rare in practice.
- **EDGE:** duplicate identical body wikilink lines → one edge (dedup covers it).
- **EDGE:** wikilink inside a fenced code block → excluded by `extract_wikilinks`
  (fence-aware) — no edge. Correct.
- **BOUNDARY:** body wikilinks are **memory-target-only** — the extractor regex
  gates on `mem.`/`mem_` prefixes, so a body link can never edge to a numbered
  entity (`SL-`/`ADR-`/…). `classify_target`'s canonical-ref branch is unreachable
  from this pass. This is a construct boundary (wikilinks are a memory-corpus
  device), not a limitation to fix. Numbered cross-refs stay in TOML relations.
- **ASM:** by the time this ships, SL-200 has relinked the shipped corpus body
  wikilinks to `mem.`-prefixed form the extractor requires (confirmed done, paper-
  work pending). Prefix-less links are a corpus concern, not this slice's.

## 6. Open Questions & Unknowns

None open. Prior forks resolved:
- Unresolved body wikilink → **`Warning` on `UnvalidatedText`, a deliberate
  divergence** from the silent TOML path (F-1), scoped to the body pass. The
  `mem.word.word` extractor shape excludes prose false-positives, so warnings do
  not misfire (owner ruling).
- Edge label → **`"related"`** with `EdgeOrigin.field = "body"` for origin
  traceability (owner ruling).
- Clickable focus-link renderer → **deferred** (non-goal; follow-up).

## 7. Decisions, Rationale & Alternatives

- **D1 — body read in `read_catalog_record`, not `scan_memory_entities`.** It is
  the existing single I/O site per memory and already returns the record; folding
  the body read there keeps one pass and one impurity boundary. *Alt:* second read
  in the scan loop — rejected (splits I/O, no gain).
- **D2 — dedup on `EdgeTarget::Resolved(CatalogKey)`, per-record `seen` set.**
  Exact identity dedup; a TOML relation and a body wikilink to the same memory draw
  one edge. *Alt:* dedup on raw string — rejected (a `mem.<key>` and `mem_<uid>`
  to the same memory would evade it).
- **D3 — synthesized label `"related"`, origin field `"body"`.** Consistent with
  existing untyped memory relations; body-origin stays traceable via origin field
  without minting a new vocabulary word. *Alt:* distinct label (`"mentions"`) —
  rejected (new term for no consumer benefit today).
- **D4 — `body` stays `None` on the memory `CatalogEntity`.** Bodies consumed
  transiently; no renderer needs memory prose. *Alt:* populate it — rejected (dead
  data, non-goal).

## 8. Risks & Mitigations

- **R1 — diagnostic noise** from unresolved body wikilinks on a large corpus.
  *Mitigation:* extractor shape gates out prose; owner accepts warnings as
  near-zero false-positive. Revisit only if noise observed.
- **R2 — behaviour drift** in existing TOML-edge output from the loop refactor.
  *Mitigation:* the TOML pass logic is unchanged except an additive `seen.insert`;
  existing edge/backlink suites stay green as the proof.
- **R3 — perf** of reading every memory `.md` at scan. *Mitigation:* one extra
  file read per memory, already reading the sibling `.toml`; negligible, same
  order as numbered-entity body reads under `include_bodies`.

## 9. Quality Engineering & Validation

New tests (VT):
- **VT-1** body-only `[[mem.<key>]]` wikilink → one resolved edge (`from_scanned`).
- **VT-2** body wikilink + TOML relation to same target → one edge (dedup, INV-1).
- **VT-3** two body wikilinks to same target → one edge (dedup).
- **VT-4** unresolved body wikilink → `UnvalidatedText` → one edge + one `Warning`
  (INV-2, body-pass divergence); the TOML path's `UnvalidatedText` handling stays
  silent (behaviour-preservation).
- **VT-5** `read_catalog_record` populates `body`; missing `memory.md` → `None`.
- **VT-6 (behaviour-preservation)** existing TOML-edge + backlink suites green
  unchanged.

## 10. Review Notes

Internal adversarial pass (design stage):

- **A1 — every-call body read is safe.** `read_catalog_record` has one production
  caller (`scan.rs:468`); the rest are tests. No hot path pays for the sibling read.
- **A2 — filename literal is convention, not magic string.** `"memory.md"` is
  inlined across the codebase (`corpus.rs:370`); mirror the `.ok()` missing→`None`
  idiom rather than mint a `MEMORY_MD` constant against the grain (§5.2).
- **A3 — origin marker constant-ized.** The novel `"body"` semantic marker becomes
  `MEMORY_WIKILINK_ORIGIN_FIELD` (STD-001); folded into §5.2.
- **A4 — memory-target-only boundary** stated in §5.5. Body links cannot edge to
  numbered entities; not a defect.
- **A5 — test-infra gap.** `seed_memory` writes only `memory.toml` (its `body`
  param is TOML content, misnamed). VT-1..5 need a helper that also writes
  `memory.md`. Carried to /plan as a build-the-fixture task, not a design change.

External inquisition (RV-245, design stage) — all three findings accepted,
design reconciled:

- **F-1 (major, design-wrong → fixed).** The "diagnostic parity with the TOML
  path (`hydrate.rs:341`)" claim was false: a `mem.`-prefixed body wikilink
  resolves to `UnvalidatedText`, not `UnresolvedRef`, and the TOML path is silent
  on `UnvalidatedText`. Rewrote §5.4/§5.5-INV-2/§6/§9-VT-4: warn on
  `UnvalidatedText` only, scoped to the body pass, named a **deliberate
  divergence** (not parity); struck the dead `UnresolvedRef` arm and the false
  citation. Plan EX-3/VT-4 moved in lockstep.
- **F-2 (minor → fixed).** INV-1 overclaimed global edge-uniqueness; the TOML pass
  self-dedups nothing. Rescoped INV-1 to one-directional dedup (body defers to
  TOML; TOML-vs-TOML multiplicity unchanged). Plan EX-2 already stated this
  correctly — design now matches the plan.
- **F-3 (nit → adopted).** The two opposite-lifecycle `body` fields get a one-line
  disambiguating comment at the loop site (§5.3).

Informal plan review (owner) folded alongside: plan EX-2 reworded so `seen` is
populated by the TOML pass **and** each emitted body edge (covers VT-3
body-vs-body); plan EX-3/VT-4 aligned to the F-1 divergence.
