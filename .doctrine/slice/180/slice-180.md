# Selector conformance hardening: design-time dry-run + import-belt scope-creep refusal

## Context

RFC-004 defines path-intent selectors (`[[selector]]` with `design-target` /
`scope-relevant` intents) that declare what files a slice is expected to touch.
RFC-005's dispatch funnel imports worker deltas onto the coordination branch,
but the import belt (R-5, `src/worktree/import.rs`) only rejects `.doctrine/`
and `.claude/` prefixes — it does **not** check whether the delta's files are
within the slice's declared design-target selector set.

IMP-199 and IMP-204, both surfaced in the SL-168 postmortem (§5b.4), form a
coherent defense-in-depth pair:

- **IMP-204** — design-time conformance dry-run. Before locking a design, diff
  the design's Code-impact table against the planned/committed file set. Prompt
  for compile-fallout (CLI/MCP/reconcile/sibling goldens) that the author may
  have omitted from the selectors. Catches under-declaration early.

- **IMP-199** — import-time scope-creep refusal. Extend the import belt
  (`classify_import`) to refuse worker deltas that touch files outside the
  declared design-target selector set. Catches worker scope creep before it
  lands on the coordination branch.

Both ride the same conformance engine (`src/conformance.rs`), which already
partitions paths into conformant / undeclared / undelivered but is only used at
audit time (`slice conformance`). This slice wires it into two new surfaces.

## Scope & Objectives

1. **PHASE-01 (IMP-204):** Design-time conformance dry-run.
   - New subcommand or flag on `slice conformance` that diffs design's
     Code-impact table against the committed/planned file set.
   - Warns (or fails, under `--strict`) when the declared selectors miss
     compile-necessary fallout files.
   - Pure: the conformance compute exists; the dry-run is a thin shell over it.

2. **PHASE-02 (IMP-199):** Import-belt scope-creep refusal.
   - Extend `classify_import` with a new `UndeclaredScope` refusal variant.
   - `run_import` reads the slice's design-target selectors, computes
     conformance on the worker's `B..<fork>` delta, and refuses if undeclared
     paths exist.
   - The slice id must be passed to `run_import` (or resolved from the
     coordination context).

## Non-Goals

- Changing the selector model itself (RFC-004 territory).
- Guarding `edge:main` promotion (that's H6/g4, separate slice).
- Auto-suggesting selectors from code analysis — just gap detection.
- Modifying the `prepare-review` or `integrate` stages.

## Affected Surface

- `src/conformance.rs` — may need minor extensions (e.g. accepting raw path
  lists alongside the full `BTreeMap<String, Vec<Status>>`)
- `src/worktree/import.rs` — new `UndeclaredScope` refusal + selector plumbing
- `src/slice.rs` — conformance shell may gain a `--dry-run` mode
- `src/dispatch.rs` — `run_import` call site may need slice-id plumbing

## Risks

- **False positives at import time.** An author may genuinely need to touch a
  file not in the selectors (compile fallout). Mitigation: IMP-204's design-time
  dry-run closes the gap *before* import; and the import refusal should be
  overridable (e.g. `--allow-undeclared` flag) for emergency unblocking.
- **Selector read in the import path.** `run_import` currently takes only
  `--base` and `--fork`; adding slice-id resolution to it couples the import
  verb to the slice registry. Low risk — the coordination context already knows
  its slice.

## Verification / Closure Intent

- VT: `classify_import` with selector-aware `UndeclaredScope` test (pure,
  unit-testable).
- VT: `run_import` integration test — worker touches an undeclared path → import
  refused.
- VT: design-time dry-run flags missing selectors for a known under-declared
  slice fixture.
- VA: the import belt still passes for a conformant delta (no regression on
  existing paths).

## Follow-Ups

None — this is a self-contained hardening. IMP-204 closes the design-time gap;
IMP-199 closes the import-time gap.
