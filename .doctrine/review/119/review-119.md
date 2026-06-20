# Review RV-119 — reconciliation of SL-129

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Review surface:** `refs/heads/dispatch/129` (bb6a242) via `sync --prepare-review` → `refs/heads/review/129`. Trunk drifted 12 commits post-fork; slice verified in isolation against the dispatch tip.

### Lines of attack

1. **Structural integrity** — does every `entity::Kind` const initializer have `stem:`? Are sub-kinds (DESIGN/PLAN/NOTES) correctly marked `stem: ""`? Verified via grep of all 36 sites.
2. **KindRef::stem removal completeness** — `integrity::KindRef` struct drops `stem`; all KINDS rows updated. No remaining `kref.stem` references in production code.
3. **GovKind::stem removal completeness** — `governance::GovKind` struct drops `stem`; all `g.stem` refs migrated to `g.kind.stem`. `read_doc`/`set_status` signatures changed from `gov_root` to `root`.
4. **Format! replacement completeness** — all ~85 production path construction sites converted to `entity::id_path` or `entity::rel_path`. Exclusions per design: `meta.rs` internals, test full-path assertions, phase-state files, memory UUID paths, `BACKLOG_STEM` constant.
5. **Behaviour preservation** — every replacement produces the identical byte path. Verified via: all 2117 tests pass, clippy zero warnings, build green.
6. **Excluded-surface audit** — verify `lazyspec.rs` (3 excluded format! sites) and `meta.rs` internals are genuinely out-of-scope, not missed sites.
7. **Test-only site boundary** — test-only format! sites in `main.rs` and module-level tests replaced; assertion strings left alone per design.

## Synthesis

**Verdict: clean conformance — zero findings.**

Every design decision was faithfully implemented:

- **`Kind.stem`** added to the struct with `#[serde(skip)]` (catalog JSON shape preserved). 36 Kind initializers seeded (30 production + 6 test). Sub-kinds correctly get `stem: ""`.
- **`Ext` enum**, **`make_file_name`** (with `debug_assert!` guard), **`id_path`** and **`rel_path`** helpers all present and correct.
- **`KindRef::stem`** removed; all 22 KINDS rows updated. No stale `kref.stem` references.
- **`GovKind::stem`** removed; all ~10 `g.stem` → `g.kind.stem` migrations done. `read_doc`/`set_status` now accept `root` (project root) and use `entity::id_path` internally.
- **~85 production format! sites** replaced with data-driven helpers across 20 files. Exclusions (meta.rs, lazyspec.rs, test assertions, phase-state files, BACKLOG_STEM) all genuinely out-of-scope per design.
- **Behaviour-preservation gate**: 2117 tests pass, 0 failed, 1 ignored (expected); `cargo clippy` zero warnings; `cargo build` clean.
- **PHASE-01 → PHASE-02 dead_code transition**: all `#[expect(dead_code)]` annotations from PHASE-01 removed in PHASE-02, confirming the helpers are consumed.

**Tradeoffs consciously accepted** (all per design):
- Sub-kind stem-less guard is `debug_assert!` only (debug-mode check, acceptable for single-author tool).
- `BACKLOG_STEM` constant left in place (harmless dead constant).
- Test assertion full-path strings left unabstracted (failure output readability).
- `meta.rs` internals excluded (already abstracted behind stem parameter with kind-root callers).

No blocker, major, minor, or nit findings.

## Reconciliation Brief

No spec/governance findings — every design decision was implemented as specified. The reconciliation brief is empty.

## Reconciliation Outcome

All 0 findings were resolved as clean conformance. No writes needed.
Reconcile pass complete — handoff to /close.
