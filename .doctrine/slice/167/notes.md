# Notes SL-167: Accept prefixed canonical ids on all CLI id:u32 args

## Implementation notes

- All four status verbs (`adr|policy|standard|rfc status`) now accept both
  `PREFIX-NNN` and bare `NNN` via `parse_cli_id` value parsers.
- Each module exports its own `parse_ref` + `parse_cli_id`. The `parse_ref`
  functions delegate to `governance::parse_entity_ref(prefix, kind_label, reference)`
  — shared logic, per-kind identity.
- `integrity::parse_canonical_ref` was not touched (D2).

## Review
- RV-182 (code-review): doc-comment paste artifact fix + stronger test assertions.
- RV-197 (reconciliation audit): design.md §5.1/§5.2/D1 updated to reflect
  delegation pattern; no governance changes needed.

## Harvested facts
- The pattern of per-module wrapper + shared parameterized delegate is cleaner
  than five mechanically identical copies and preserves per-kind identity.
- `governance` was already a dependency of all five modules — no new coupling.
