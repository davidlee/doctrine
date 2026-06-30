# Review RV-197 — reconciliation of SL-167

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of attack:**

1. **CLI acceptance**: Do all four status verbs (`adr|policy|standard|rfc status`) accept both `PREFIX-NNN` and bare `NNN`? Is backward compatibility preserved?
2. **Design conformance**: Does the implementation match `design.md`? The refactor commit (11ca3f71) unified per-kind `parse_ref` into `governance::parse_entity_ref` — does this deviate materially from D1?
3. **Test coverage**: Does each kind module have `parse_ref_accepts_prefixed_padded_and_bare_ids` tests covering prefixed, lowercase, bare, padded, and invalid inputs?
4. **Gate integrity**: `integrity::parse_canonical_ref` must be untouched (D2).
5. **Slice metadata hygiene**: Status, selectors, conformance recording — is the slice ready for close?

**Invariants:**
- `cargo test` + `cargo clippy` zero failures/warnings
- Bare numbers unchanged (no regression)
- All prefix forms produce identical results (canonical, lowercase, bare, padded)

## Synthesis

SL-167 is a clean, low-risk CLI acceptance fix: four kind modules (adr, policy,
standard, rfc) can now accept `PREFIX-NNN` on their `status` verbs, matching the
convention already used by all `show`/`paths`/`inspect` verbs across the codebase.

**Implementation quality.** The initial implementation (2c08c594) followed the
design literally — five mechanically identical `parse_ref` copies. A subsequent
refactor (11ca3f71) eliminated the duplication by introducing a shared
`governance::parse_entity_ref(prefix, kind_label, reference)` that all five
per-kind `parse_ref` functions delegate to. The per-module `parse_ref` +
`parse_cli_id` contract is preserved; the shared function is parameterized, not
a generic that erases kind identity. This is a pure DRY improvement — the design
spirit (per-kind identity, per-kind error messages) is intact.

**Evidence.** All 2,814 doctrine tests pass (the lone cordage failure is a
pre-existing path-resolution issue in a different crate). Clippy is clean.
Manual smoke confirms all four verbs accept `PREFIX-NNN` and bare `NNN`
identically. Each module carries `parse_ref_accepts_prefixed_padded_and_bare_ids`
covering prefixed uppercase, prefixed lowercase, bare, zero-padded, and invalid
inputs. D2 (`integrity::parse_canonical_ref` untouched) holds.

**Standing risks.** None. The change is pure additive — no existing callers
changed, no existing tests broken.

**Tradeoffs accepted.** The DRY refactor introduces a dependency from five
module crates onto `governance::parse_entity_ref`. This is acceptable because
`governance` is already a dependency of all five modules for their other verbs
(`show`, `paths`, `inspect`). No new coupling introduced.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md §5.1** (F-1): The prose describes five independent `parse_ref`
  implementations. Consider updating to reflect the actual architecture
  (per-module wrappers delegating to `governance::parse_entity_ref`). Optional —
  the design *decision* (per-kind identity) is correct; only the mechanical
  description is stale.

### Governance/spec (REV)

None.

## Reconciliation Outcome

### Direct edits applied
- `design.md` §5.1 + §5.2 + §7 D1: updated to describe the actual delegation pattern
  (`governance::parse_entity_ref`) rather than five independent copies (RV-197 F-1).

### REVs completed

None — no governance/spec changes needed.

### Withdrawn / tolerated
- RV-197 F-1: aligned — per-kind contract preserved; DRY refactor is a strict improvement.
- RV-197 F-2: tolerated — no selectors; manual inspection substituted for this small CLI change.
