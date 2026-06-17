# Review RV-061 — reconciliation of SL-90

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reviewed the candidate interaction branch `candidate/090/review-001`
(merge-resolved from review/090 impl bundle onto main at ae5d26a9).

**Lines of attack:**

1. **PHASE-01 — resolve_memory_toml_path**: Does the resolver correctly gate
   shipped/ as read-only? Do uid, uid-prefix, and key resolution all work?
2. **PHASE-02 — append/remove helpers**: Is the F1 trap effective? Is
   idempotency correct? Are empty labels/targets refused before file mutation?
3. **PHASE-03 — CLI fork**: Does `MemoryRef::parse` correctly route memory
   sources without intercepting numbered entities? Is target validation
   best-effort per D3? Is CLI help text updated?
4. **Behaviour-preservation gate (VT-6)**: Does `link SL-048 governed_by
   ADR-010` still work unchanged? Do SL-087 boot_keys tests still pass?

**Invariants held to:** D4 (shipped/ read-only), D1 (fork before
parse_canonical_ref), D6 (F1 trap), D5 (outcome enum re-use), D3 (best-effort
validation), D2 (free-form labels). All 19 VT criteria across 3 phases.

## Synthesis

All three phases delivered against the design with full fidelity. The
candidate merge conflict in `src/memory.rs` was purely additive — SL-087's
boot_keys tests and SL-090's relation tests occupied adjacent regions with
no semantic overlap — and resolved by keeping both sets. `just check` passes
with 1608 tests, no clippy warnings, and no lint-js warnings.

### PHASE-01 — Memory resolution helper

`resolve_memory_toml_path` correctly implements the D4 policy: items/ probed
first (writable), shipped/ as a fallback diagnostic only (returns an error
that names "shipped corpus record" and "read-only"). All 6 VT criteria pass:
uid in items, uid only in shipped (error), uid nowhere (not-found error),
unique uid-prefix resolution, ambiguous prefix error, and key resolution.
The function reuses `resolve_uid_prefix` for prefix disambiguation and
`fsutil::safe_join` as the path-building chokepoint (H1).

### PHASE-02 — Memory relation write helpers

`append_memory_relation` and `remove_memory_relation` handle the raw
`[[relation]]` write seam correctly. The F1 trap (`trailing_typed_table_after_relation`)
guards against a `[trust]` or `[ranking]` table authored after `[[relation]]`,
refusing before touching the file. Idempotency is checked before any mutation:
re-linking returns `Noop` with byte-unchanged file; re-unlinking returns
`Absent`. Empty label/target are hard errors before touching the file.
`toml_edit::value()` escaping handles special characters in targets (VT-6
passes with quotes and backslashes). The helpers return `AppendOutcome` /
`RemoveOutcome` from the `relation` module (D5), keeping the CLI output
surface consistent.

### PHASE-03 — CLI fork

The fork in `run_link` and `run_unlink` is minimally invasive:
`MemoryRef::parse(source)` is tried first — "SL-048" returns `Err` (no
`mem_` prefix, no key pattern match), so numbered entities fall through
to the existing path unchanged. Memory sources route through PHASE-01+02.
Best-effort target validation (D3) uses `parse_canonical_ref(target).is_ok()`
to detect canonical-ref-shaped strings and then `ensure_ref_resolves` to
validate them; free-text and mem_* targets pass through unvalidated (the
catalog scanner surfaces dangling refs at scan time). CLI help text was
updated to document `mem_<uid>` / `mem.<key>` as valid source refs.

All 7 VT criteria pass, including the critical VT-6 behaviour-preservation
gate (`link SL-048 governed_by ADR-010` unchanged) and VT-7 (key-based
memory source).

### Standing risks

None. The implementation is fully additive — no shared infrastructure was
modified, no existing code paths were re-wired. The F1 trap duplication is
intentional (D6, different label types) and carries no maintenance risk at
this scale.

### Tradeoffs consciously accepted

- F1 trap duplication: `memory.rs` carries its own `trailing_typed_table_after_relation`
  rather than sharing with `relation.rs`. Justified by the label-type mismatch
  (`&str` vs `RelationLabel`), per design D6.
- No target-kind validation for memory edges: labels are free-form raw strings
  (D2), so there is no `RELATION_RULES` entry to consult. Target validation is
  limited to resolution checks for canonical-ref-shaped strings (D3). Dangling
  refs surface at catalog scan time.

## Reconciliation Brief

### Per-slice (direct edit)

None — implementation matches design.md exactly. All design decisions (D1–D6)
are faithfully expressed in code.

### Governance/spec (REV)

None — no ADR, spec, or governance document requires amendment. The slice's
scope is self-contained: a write surface for memory relations that the
catalog pipeline already reads.

## Reconciliation Outcome

All findings were withdrawn or tolerated with rationale. No writes needed.
Reconcile pass complete — handoff to /close.
