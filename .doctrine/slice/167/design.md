# Design SL-167: Accept prefixed canonical ids on all CLI id:u32 args

## 1. Design Problem

Five entity-kind command enums use `id: u32` CLI argument fields. Clap parses these as raw integers, rejecting the canonical prefixed form (`SL-163`, `ADR-007`, `POL-001`, `STD-001`, `RFC-011`) with an opaque `invalid digit found in string` error. Meanwhile every `Show`/`Inspect` verb already uses `reference: String`, and the boot snapshot + AGENTS.md mandate canonical ids everywhere in prose, commits, and comments. The inconsistency costs agents wasted invocations (surfaced via IMP-189, cross-ref RFC-011 case-notes).

**Affected verbs:**
| Command | Variant | Prefix |
|---|---|---|
| `slice` | `design`, `plan`, `phases`, `notes`, `phase`, `status`, `conformance`, `record-delta`, selector `add`/`upsert`/`list`/`rm` | SL- |
| `adr` | `status` | ADR- |
| `policy` | `status` | POL- |
| `standard` | `status` | STD- |
| `rfc` | `status` | RFC- |

## 2. Current State

- `slice::parse_ref` exists — accepts both `SL-NNN` and bare numbers. Prefixed IMP-189 quick-fix already wired it to all 12 SliceCommand `id: u32` fields via `parse_cli_id`.
- ADR/Policy/Standard/RFC status commands still take raw `id: u32` with no value_parser.
- `integrity::parse_canonical_ref` exists but rejects bare numbers (by design — it's for canonical-only contexts).
- All Show/Inspect/Paths verbs across all kinds use `reference: String`.

## 3. Forces & Constraints

- Backward-compatible: bare numbers must still work.
- DRY: each kind should reuse its own `parse_ref`-equivalent (currently only slice has one).
- Error messages must name the expected forms.
- Must not change `integrity::parse_canonical_ref` semantics (its bare-number rejection is tested and depended on).

## 4. Guiding Principles

- CLI is the source of truth — uniform id acceptance.
- Ride existing seams — add `value_parser` to existing `id: u32` fields.
- Small, local change per kind module.

## 5. Proposed Design

### 5.1 Approach

Add a `parse_cli_id` function to each kind module (ADR, policy, standard, RFC) mirroring `slice::parse_cli_id`. Each wraps a kind-local `parse_ref` that strips the canonical prefix before parsing.

For ADR: add `parse_ref` (strip `ADR-`/`adr-`) + `parse_cli_id` (clap wrapper).
For policy: add `parse_ref` (strip `POL-`/`pol-`) + `parse_cli_id`.
For standard: add `parse_ref` (strip `STD-`/`std-`) + `parse_cli_id`.  
For RFC: add `parse_ref` (strip `RFC-`/`rfc-`) + `parse_cli_id`.

### 5.2 Interfaces

Each `parse_ref` function:
```rust
pub(crate) fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("ADR-")
        .or_else(|| reference.strip_prefix("adr-"))
        .unwrap_or(reference);
    digits.parse::<u32>().with_context(|| {
        format!("not an ADR reference: `{reference}` (expected `ADR-007` or `7`)")
    })
}

fn parse_cli_id(s: &str) -> Result<u32, String> {
    parse_ref(s).map_err(|e| format!("{e:#}"))
}
```

Wire `#[arg(value_parser = parse_cli_id)]` on each `id: u32` field.

### 5.3 Quality

Existing tests to extend: each kind's module tests get a `parse_ref_accepts_prefixed_padded_and_bare_ids` test mirroring slice's. Existing CLI split tests in `main.rs` already pass (they use bare `"0"`).

## 6. Open Questions & Unknowns

None — solution is mechanically identical to the already-committed slice fix.

## 7. Decisions

**D1:** Per-kind `parse_ref` + `parse_cli_id` (not a global generic). Reuses the proven slice pattern; each kind controls its own error message formatting.

**D2:** No change to `integrity::parse_canonical_ref`. Its contract is canonical-only; mixing concerns would be a regression risk.

## 8. Risks & Mitigations

- **Low risk:** Pure additive — no existing callers change, no existing tests break.

## 9. Quality Engineering & Validation

- Unit tests: `parse_ref_accepts_prefixed_padded_and_bare_ids` per kind.
- Manual smoke: `doctrine adr status ADR-001 accepted`, `doctrine rfc status RFC-011 accepted`, etc.
- Gate: `cargo clippy --workspace` + `cargo test` zero failures.
