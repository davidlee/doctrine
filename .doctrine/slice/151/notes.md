# Notes SL-151: Non-contiguous TOML sections cause opaque parse failures

## Implementation summary

### PHASE-01 — Shared entity-parse wrapper + canonical-id threading

- Added `dtoml::parse_entity_toml<T>(text, prefix, id) → T` — pure leaf wrapping
  `toml::from_str` with canonical-id error context (D1)
- Routed six entity read paths through wrapper: `read_meta`, `read_id`,
  `read_slice`, `read_item`, `read_record`, `read_doc`
- Threaded `prefix` param through `read_metas` and all compile-forced callers
  (13 modules including lazyspec.rs:468 and catalog/scan.rs:429)
- Commit: `7f46024f`

### PHASE-02 — Proactive validate detection via scan_kind

- Added `diagnostics: &mut Vec<String>` param to `scan_kind`
- Schema-agnostic `toml::Value` parse before `read_id` catches non-contiguous
  TOML and pushes canonical-id-tagged diagnostic
- Wired through `id_integrity_findings` → `doctrine validate`
- No catalog (`scan_entities`) performance impact — `scan_kind` is validate-only
- Commit: `0f57a084`

### Test coverage

- VT-1: `parse_entity_toml` passes valid TOML identically to raw `toml::from_str`
- VT-2: `parse_entity_toml` on non-contiguous TOML errors with canonical id
- VT-3: `scan_kind` flags non-contiguous `[relationships]` header
- VT-4: `scan_kind` produces no diagnostics on valid TOML (incl string-body guard)
- VT-5: e2e — `doctrine validate` exits non-zero on non-contiguous fixture
- VT-6: e2e — `doctrine show SL-NNN` errors with canonical id in message
- VT-7: `read_id` on malformed id-only TOML surfaces canonical-id context
- VA-1: `just gate` — 2492 passed, 0 failed, 0 warnings

### Design decisions

- `toml::Value` parse placed BEFORE `read_id` (not after as originally designed):
  `read_id` would fail on non-contiguous TOML before the diagnostic could run.
  The gate ensures diagnostic is pushed first.
- No string-matching on error text — wrapper adds context around existing error
  with zero conditional logic (mem.pattern.parse.toml-error-classification-fragile)
- Prefix descends from callers — meta.rs does not reach up to integrity::KINDS
  (ADR-001)

## Reconciliation Outcome

No findings — implementation fully conforms to design. All VT criteria met,
behaviour-preservation gate held (zero test assertion changes). No spec or
governance changes required. No-op reconcile — handoff to /close.
