# IMP-161: Project-wide TOML config coefficient CLI — get/set priority coefficients, kind_weights, tag_coefficients

<!-- IMP-161: Project-wide TOML config coefficient CLI -->

## Context

RFC-002's priority scoring (ADR-015, SL-133) ships `[priority.coefficients]`,
`[priority.kind_weights]`, `[priority.tag_coefficients]`, and
`[priority.consequence]` in `doctrine.toml`. Currently these are hand-edited —
no CLI surface exists to inspect or modify them.

This creates a papercut gap:
- Operators must open `doctrine.toml` directly to tune scoring
- Kind weights and tag coefficients have no discoverable write path
- No `show` surface exists to preview active config
- STALE: hand-edit can silently drift from the clamped defaults the code expects

Other `doctrine.toml` sections (`[dispatch]`, `[conduct]`, `[verification]`,
`[estimation]`, `[value]`) share this gap — they too are hand-edited — but
priority coefficients are the highest-leverage first target because they
are the most frequently adjusted (tuning scoring during triage).

## Scope

A `doctrine config` CLI verb with subcommands that read and write specific
keys in `doctrine.toml` without reserialising the file (edit-preserving,
reusing `toml_edit` patterns from SL-136).

### Phase 1: Priority config surface

- `doctrine config show` — dump the active `[priority]` section with resolved
defaults shown for absent keys
- `doctrine config set priority.coefficients.value <f64>`
- `doctrine config set priority.coefficients.risk <f64>`
- `doctrine config set priority.consequence.dep_coeff <f64>`
- `doctrine config set priority.consequence.ref_coeff <f64>`
- `doctrine config get priority.coefficients.value` — read a single key

### Phase 2: Key-value map fields

- `doctrine config set priority.kind_weights.<KIND> <f64>` — upsert a kind weight
- `doctrine config set priority.tag_coefficients.<TAG> <f64>` — upsert a tag coeff
- `doctrine config unset priority.kind_weights.<KIND>` — remove a key
- `doctrine config unset priority.tag_coefficients.<TAG>`

### Phase 3 (deferred, speculative)

Extend to cover other `doctrine.toml` sections: `[dispatch]`, `[conduct]`,
`[estimation]`, `[value]`, `[verification]`. The `config` verb is the shared
surface; each section is just a new path namespace.

## Related

- RFC-002 — Consumption surfaces program (program coordinator)
- ADR-015 — Multi-dimensional priority scoring (the policy being tuned)
- SL-133 / IMP-118 — Priority scoring (producer of the config schema)
- SL-136 / IMP-134 — Tagging (built the `toml_edit` root-insert patterns we reuse)
- SL-134 / IMP-132 — Risk facet CLI (precedent for a small dedicated CLI verb)

## Open Questions

- **Validation scope**: Should `config set` validate the value against the
scoring engine's clamp ranges (0 ≤ coefficient ≤ 1e9, dep_coeff ∈ (0, 1]), or
only write and trust the clamp? Writing and trusting clamp is simpler and
matches the existing tolerance.
- **Atomicity**: Should writing multiple keys use a single `doctrine.toml` write
or one per invocation? Single write per invocation is simpler; batch edits can
use shell scripts.
- **`config show` scope**: Show only `[priority]` or the whole `doctrine.toml`?
Start with `[priority]`; whole-file is a future extension.

## Non-Goals

- Not editing `.doctrine/` entity TOML files (that's `doctrine tag`, `status`, etc.)
- Not editing hand-authored `governance.md`
- No web UI or MCP tool for config (CLI only)
- No history/diff tracking of config changes (future work)
- No validation beyond the existing clamp (no schema-driven validation)
