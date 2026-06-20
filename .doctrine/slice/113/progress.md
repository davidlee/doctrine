## Review

**Verdict**: revision-required. The core design is sound, but the call-site table is factually inaccurate.

### What's correct
- All 18 production `std::fs::write` sites to authored entities are correctly identified by line number
- The clippy guard strategy (gated non-test, two noted exceptions) is correct
- ADR-001 (layering), ADR-003 (change loop), ADR-004 (outbound-only), ADR-010 (relation modelling) are respected
- Pure/imperative split is respected; change confined to leaf tier
- Storage-tier distinction correctly applied (ledger.rs exception justified)
- VT criteria are testable and cover the surface

### Fixed: 6 function-name errors in the call-site table (F-1 through F-4, F-7, F-9)

| Design says | File | Line(s) | Actual enclosing function |
|---|---|---|---|
| `set_authored_status` | dep_seq.rs | 178 | `append` |
| `remove_after` IO | dep_seq.rs | 246 | `remove` |
| `apply_with` | dep_seq.rs | 356 | `set_authored_status` |
| `add_requirement` | spec.rs | 799 | `append_member` |
| `handle_edit_concept_map_edge` | map_server/routes.rs | 412 | `mutate_concept_map` |
| `apply_supersede` | main.rs | 4795,4856,4858 | `run_supersede` |

Additionally, `concept_map.rs` rows use prose descriptions ("add-edge", "add-edge-force", "remove-edge", "rename-node") instead of Rust function names — inconsistent with the rest of the table.

### Note: Additional findings (F-5 through F-19)
- VT-6 error-wrapper analysis is misleading — `anyhow::Error::to_string()` and `io::Error::to_string()` give different message fragments. Cosmetic change, no tests assert on error format, but should be documented
- Missing ADR-001 tier analysis (§2.1) — confirm each module's `crate::fsutil` import is a valid downward dependency
- Missing acknowledgment of existing `write_atomic` usage (~11 sites in memory, review, boot, skills, coverage_store)
- VT-2 concurrent test underspecified — needs assertions on both-ok, no stray temps, deterministic content result
- VT-5 negative test is manual-only — tolerated as disproportionate automation
- §5 guarantee scope undersells the power-loss risk from missing `fsync` before rename

### Ledger
Full findings, dispositions, and resolution on **RV-102** (19 findings, all verified).
