## Problem

`slice verify-vt` matches `keywords` over the raw test file. That's POL-002 compliant (no host-language comment/string stripping), but it has a blind spot: if the keyword already appears in production code, comments, or string literals, the gate passes **even when the worker added zero new tests** and the mandated file was never touched by the slice.

Concrete example from RFC-011 case notes (SL-176 PHASE-03):
- Plan VT-3 mandated `test_file = "tests/e2e_relation_migration_storage.rs"`, `keywords = ["Fulfils", "burndown"]`
- Worker added 0 new test functions (count 32→32)
- But `"Fulfils"` and `"burndown"` appear throughout production code (`src/relation.rs`, `src/priority/burndown.rs`)
- `verify-vt` reported PASS — the gate was inert

The gate's threat model is worker **omission** (keyword absent entirely). But when the author picks keywords that pre-exist in production code, the gate false-passes on work that was never done.

## Proposed fix

**Intersect `test_file` with the slice's source-delta.** If the mandated file is NOT in the set of files modified by this slice (recorded in the source-delta registry via `record-delta`), the keyword match carries zero signal — it was there before the slice did any work.

Add a new verdict: `Unattributable` — distinct from both `Pass` (non-halting, but no credit) and `Fail` (halting).

```
Check order (revised):
  1. waived → Waived (unchanged)
  2. no test_file → Uncheckable (unchanged)
  3. file missing → Fail (unchanged)
  4. [NEW] test_file NOT in slice's source-delta → Unattributable
     "keyword present but file not modified by this slice"
  5. keyword absent → Fail (unchanged)
  6. otherwise → Pass (unchanged)
```

`Unattributable` is **non-halting** (like `Uncheckable` / `Waived`; INV-4) — the absence of evidence is not evidence of absence. But it surfaces a distinct glyph (suggest: `~`) so the orchestrator/auditor sees the signal and can manually inspect.

## POL-002 compliance

Source-delta tracking is doctrine's **own mechanism** (RFC-004; `code_start_oid`/`code_end_oid` per phase, recorded by `slice record-delta`; `slice conformance` already reads this registry). It is not a host-language convention, not a directory-layout assumption, not a transient local state shape. Adding a gate that consults it adds no new host-coupling burden.

The residual weakness (keyword added in a comment within a genuinely-modified test file) is the same POL-002 tradeoff already accepted and documented in `vtgate.rs` — out of scope for this fix.

## Implementation surface

- **Pure core** (`src/vtgate.rs`):
  - Add `Unattributable { reason: String }` variant to `VtVerdict`
  - Add `modified_files: &HashSet<String>` (or `&[String]`) parameter to `check_vt` and `check_phases`
  - Insert the source-delta check at step 4 in the order above
  - Update `has_failure` (still `Fail`-only; `Unattributable` is non-halting)
  - Update `render_summary` / `render_line` with a new glyph and label
  - Add unit tests: keyword in unmodified file → `Unattributable`; keyword in modified file → `Pass`; keyword absent in modified file → `Fail`

- **Impure shell** (`src/slice.rs`, the `run_verify_vt` function):
  - Already loads the slice's plan; now also read the source-delta registry (same place `slice conformance` reads it) to build the modified-files set
  - Pass it into `check_phases`
  - Gate exit code unchanged (only `Fail` → non-zero)

- **No plan.toml schema change needed** — source-delta is a runtime registry, not an authored field.

## Interactions / edge cases

- **Solo slices that never ran `record-delta`:** the modified-files set will be empty → every VT reports `Unattributable`. This is correct — the gate can't attribute the keywords to slice work. The auditor runs `record-delta` (already a standard F-2 backstop step documented in the /audit skill) which populates the registry, then re-runs `verify-vt` for a meaningful signal.
- **Phases not yet in_progress/completed:** same as above — no source-delta → `Unattributable`. Expected for pre-implementation phase-plan dry-runs.
- **Multi-phase slices:** each phase's `code_start_oid`/`code_end_oid` independently contributes to the modified set. A file modified in PHASE-01 but referenced by PHASE-02's VT is still `Unattributable` for PHASE-02 (the keyword was added in a prior phase — not attributable to PHASE-02's work). This is correct — each phase's VT gates that phase.

## Companion process fix (non-code)

The plan-authoring skill/template should **prefer `patterns` over bare `keywords`** when the token might exist in production code. `patterns: ["fn\\s+\\w*burndown"]` gives a stricter signal than `keywords: ["burndown"]` and is equally POL-002 compliant (the regex is author-owned, language-agnostic). `keywords` remains the cheap floor for genuinely novel tokens. This is a skill/template update, outside this item's scope — capture as a separate chore or fold into the next plan-skill revision.
