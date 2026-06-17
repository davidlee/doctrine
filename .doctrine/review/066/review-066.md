# Review RV-066 — reconciliation of SL-092

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Audit surface.** Reviewed the `review/092` branch (impl-bundle) diffed against
`main` — the published candidate interaction surface per dispatch `--prepare-review`.

**Lines of attack.**

1. **PHASE-01: numeric inbound sort (RSK-007).** Does `inspect_from` sort inbound
   sources by `EntityKey::Ord` (prefix lexicographic, id numeric) instead of
   lexical canonical-ref string sort? Is the test extended with ids ≥ 1000? Is
   cross-prefix ordering preserved (implicit via `BTreeSet<EntityKey>`)?

2. **PHASE-02: graceful scan degradation (IMP-036).** Is `scan_entities` signature
   updated with `&mut Vec<CatalogDiagnostic>`? Do per-entity failures skip with
   diagnostics instead of aborting? Are non-per-entity failures still fatal? Are all
   9 call sites updated? Are there two separate match arms (D4)?

3. **D3 stderr surface.** Does `run_inspect` collect diagnostics and print them to
   stderr before normal output? Is stdout byte-identical for a well-formed corpus?

4. **D5 dangler amplification.** Is the trade-off documented in design.md? Does
   the implementation avoid any dangler-suppression logic?

## Synthesis

SL-092 is a clean, mechanical implementation of two deferred findings from the
SL-046 code review. The changes are tightly scoped, ride existing infrastructure
(`EntityKey::Ord`, `CatalogDiagnostic`), and touch no new types or dependencies.

**PHASE-01 (numeric sort).** A single ~5-line change in `inspect_from` replaces
`Vec<String>` canonical-ref sort with `Vec<EntityKey>` sort — the derived `Ord`
(prefix lexicographic, id numeric) was already correct, just unused at this site.
The test extension seeds SL-998..SL-1001 out of filesystem order and asserts
numeric ordering, closing the RSK-007 gap definitively. Cross-prefix ordering
(ADR before SL) is covered implicitly by `BTreeSet<EntityKey>` construction in
scan-order and projection tests.

**PHASE-02 (graceful degradation).** `scan_entities` now accepts `&mut
Vec<CatalogDiagnostic>`, matching the existing `scan_memory_entities` precedent.
Both per-entity read sites (`status_and_title_for`, `outbound_for`) use separate
`match` arms (D4) with distinct diagnostic messages. A skipped entity contributes
nothing to the returned `Vec` — identical semantics to an absent entity. All 9
call sites are updated (3 non-test, 6 test). `scan_catalog` propagates
diagnostics through `Catalog.diagnostics`. `run_inspect` surfaces them to stderr
before normal output.

The diagnostic `file` path points to the entity directory (e.g.
`.doctrine/slice/002`) rather than the specific TOML file — a slight imprecision
but functionally adequate; the `entity_key` and message provide full
identification. The stderr format uses `io::stderr()` with `writeln!` rather than
the `eprintln!` shorthand mentioned in D3 — functionally equivalent.

**Gate evidence.** Full test suite (1642+ tests) passes with zero failures.
`cargo clippy` emits zero warnings. `just gate` is green. All plan criteria (EX/
VT) are satisfied.

**Standing risks.** The D5 dangler-amplification trade-off (skipped-entity
references become danglers) is documented in design.md but deferred to a future
`--strict` flag. The stderr diagnostic is the only warning. No regression risk.

**Tradeoffs consciously accepted.** The diagnostic `file` field points to
directories, not individual TOML files — the cost of pinpointing the exact file
(TOML vs MD) would require per-parse-site path tracking, out of proportion to the
diagnostic value gained.

## Reconciliation Brief

Clean audit — zero findings raised. The implementation conforms to design.md,
plan.toml, and all extant ADRs. No per-slice edits or governance REV changes
required.

### Per-slice (direct edit)

None.

### Governance/spec (REV)

None.

## Reconciliation Outcome

Clean audit — zero findings raised. No writes needed. Reconcile pass complete —
handoff to /close.
