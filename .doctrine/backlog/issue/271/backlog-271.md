# ISS-271: verify-vt reports FAIL for a test_file absent from the tree, not UNCHECKABLE

Surfaced at the SL-231 audit (RV-318 F-9).

## Observed

Run from the primary tree after the coordination worktree is removed and before
integration, `slice verify-vt 231` reports 16 of 17 rows FAIL:

```
PHASE-01: ✗ FAIL VT-1 — mandated test_file `src/observation/wire.rs` not found
PHASE-03: ✗ FAIL VT-3 — keyword `observation_write_class_split` absent from `src/main.rs`
... 16 rows
```

The same binary against the same `plan.toml`, run from the candidate worktree,
reports **17/17 PASS**. Nothing is wrong with the slice: the implementation
lives on `review/231` and is simply not in the primary tree yet.

## Cause

The vtgate treats "mandated `test_file` does not exist" as a criterion failure.
The taxonomy already distinguishes these: FAIL is halting and "HALTS handover";
`Uncheckable` is visible and non-halting. A file absent from the tree under
inspection is definitionally uncheckable — the gate has no evidence either way —
whereas FAIL asserts the criterion *was* checked and not met.

## Impact

The window is structural, not operator error. The conclude cadence removes the
coordination tree (the only tree holding both the plan and the code) before
`/audit` runs, and integration is `/close`'s job. So between conclude and close
there is no tree where `verify-vt` is both runnable and meaningful except a
freshly created candidate. The `/audit` skill does prescribe the candidate, but
nothing warns that running the VT gate from the parent tree instead produces a
wall of FAILs — the verb's most alarming output is also its most likely false
one at exactly the moment an auditor reaches for it.

## Candidate fix

Report `Uncheckable` with "not present in this tree" when the mandated
`test_file` is absent; keep FAIL for a file that IS present but whose mandated
keywords are missing. That preserves the halting semantics where they mean
something and removes the false alarm where they do not.

Related: ISS-269 (the sibling root-resolution split in `slice conformance`).
