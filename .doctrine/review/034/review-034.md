# Review RV-034 — reconciliation of SL-071

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of attack:**

1. **Re-home correctness (Patch 1).** Verify all 6 items moved to `catalog::scan`
   with alias-only re-exports in `relation_graph.rs`. Confirm `dep_seq_for` and
   `require_minted` stayed home. No wrapper functions.

2. **Behaviour-preservation gate.** Existing inspect/priority/validate suites
   pass zero-diff — zero test changes needed for Patch 1 and Patch 5.

3. **Equivalence tests (Patch 2).** The 6 fixture tests pin scan order, entity
   shape, inspect golden output, priority graph node set, and validate findings
   — run them, inspect them.

4. **Catalog types (Patch 3–4).** `Catalog::from_scanned` is pure (no disk).
   Edge classification uses `integrity::parse_canonical_ref` — no new parsing
   path. `CatalogGraph` has no cordage dependency. `outgoing`/`incoming`
   semantics match D10.

5. **Consumer migration (Patch 5).** `relation_graph` consumes via re-exports
   (zero change). `validate_relations` dangler detection migrated to
   `Catalog.edges`. `priority` stays on `ScannedEntity` via re-exports.
   Exactly one KINDS walk remains outside `catalog/` — the IllegalRows re-read.

6. **PHASE-07 remediation.** Test helpers deduplicated, typo fixed, CLI
   `--json` flag removed, doc comments added, e2e assertions strengthened,
   `validate_relations` edge-source lookup documented.

**Invariants held:**

- `scan_entities` walks `integrity::KINDS` in table order, sorts IDs ascending
  per kind.
- `outbound_for` dispatches to per-kind readers — never re-parses TOML generically.
- `status_and_title_for` does one parse per entity (SL-050 F1).
- Fail-fast on first malformed entity (preserved; error-tolerant walk deferred).
- Re-exports are aliases, not wrappers (D7).
- `Catalog::from_scanned` is pure — no disk reads (D9).
- `CatalogGraph` uses `BTreeMap<NodeKey, CatalogNode>` + `Vec<CatalogEdge>` — no
  cordage (D4).

---

## Synthesis

### Evidence

- **1683 tests pass**, zero failures (unit + integration, full workspace).
- **`cargo clippy` zero warnings** (workspace-wide; `just gate` passes).
- **PHASE-02 6 equivalence tests** scan order, entity shape, inspect golden
  output (byte-identical), priority graph node set, validate findings.
- **PHASE-03 9 hydrate tests** entity hydration, resolved/unresolved/unvalidated
  edge classification, diagnostic generation, path derivation, `classify_target`
  edge cases.
- **PHASE-04 4 graph tests** node/edge counts, outgoing-includes-unresolved,
  incoming-excludes-unresolved, incoming-correctness.
- **PHASE-05 consumer migration** zero test changes; `va lidate_relations`
  dangler detection migrated to `Catalog.edges`; `priority` stays on
  `ScannedEntity` via re-exports.
- **PHASE-06 3 e2e CLI tests** valid JSON output, non-existent root non-zero exit.
- **PHASE-07 remediation** test helpers deduplicated, typo fixed, `--json` flag
  removed, doc comments added, e2e assertions strengthened, edge-source lookup
  comment corrected (F-1 fix).

### Conformance

All 6 patches (design §1) implemented faithfully. Design decisions D1–D12
respected:
- `dep_seq_for` and `require_minted` stay in `relation_graph.rs` (D1, D2).
- `CatalogEntity.path` derived from `EntityKey` + `Kind.dir` (D3).
- `SourceSpan` is file+field only (D4).
- Edge classification uses `integrity::parse_canonical_ref` (D5).
- No generic TOML relation reader (D6).
- Re-exports are aliases, not wrappers (D7).
- `Catalog::from_scanned` is pure — no disk reads (D9).
- `outgoing` includes unresolved/unvalidated targets; `incoming` excludes them
  (D10).
- Private helpers not re-exported (D11).
- CLI under `doctrine catalog` noun, debug-only (D12).

**Behaviour-preservation gate held.** Existing inspect/priority/validate suites
pass zero-diff — zero test code changes were needed for the re-home (Patch 1)
or consumer migration (Patch 5). This is the strongest conformance signal:
the move was mechanical and the aliases are transparent.

**KINDS-walk audit.** `rg 'for kref in integrity::KINDS' src/` hits exactly
two loops: `catalog::scan::scan_entities` (the scan) and
`relation_graph::validate_relations` (the IllegalRows re-read, a distinct
raw-TOML concern). No other entity-scanning walker remains outside `catalog/`.

### Findings dispositioned

**F-1 (minor):** `validate_relations` comment claimed `debug_assert` existed
when it did not. Fix: corrected the comment to document the invariant without
the false claim. No behavioural change. Verified.

### Standing risks

1. **Error-tolerant walk deferred.** `scan_entities` remains fail-fast on
   malformed TOML — the design explicitly deferred an error-tolerant scanner
   to a follow-up slice. The `CatalogDiagnostic::Error` variant is plumbed and
   ready. The slice scope says "Diagnostics, not panics" but the design
   narrowed scope — conscious tradeoff, no unresolved finding.

2. **`outgoing`/`incoming` not consumed in production.** They carry
   `#[expect(dead_code)]` — tested but no callers outside test. The first
   downstream consumer will naturally retire these expects. No risk of drift
   since tests pin behaviour.

3. **`Catalog::diagnostics` is sparse.** Only edge-classification diagnostics
   this phase (one Warning per `UnresolvedRef`, one Info per
   `UnvalidatedText`). The follow-up error-tolerant walk will fill the richer
   diagnostics (malformed TOML, duplicate ids, unknown labels).

### Verdict

The implementation conforms to `design.md`. The six patches were implemented in
order, the behaviour-preservation gate held, and the single finding (a misleading
comment) was corrected in-place. The slice is ready for closure.

**Recommendation:** `/close` — harvest notes, record memory, and transition
SL-071 to `done`.
