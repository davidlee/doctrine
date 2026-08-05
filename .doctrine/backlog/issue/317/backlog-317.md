# ISS-317: Boundary-span advisory prescribes --commit, which truncates the very multi-commit range it fires on

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

`span_advisory` (`src/state.rs:720-751`) fires on the `completed` flip when a
phase's recorded boundary spans two or more commits, warns that foreign commits
may have been attributed to the phase, and closes with:

```
correct with: doctrine slice record-delta <slice> <PHASE-NN> --commit <this phase's own code tip>
```

`--commit S` is documented — and implemented — as recording **exactly `S`'s own
patch, `[S^, S]`** (`doctrine slice record-delta --help`). So following the
advisory verbatim on a multi-commit phase replaces a range that was *too wide*
with one that is *one commit long*.

The advisory's own guard makes this exact: it returns `None` when
`commits.len() < 2`, so it **only ever fires in the case where its own remedy is
wrong**. There is no input on which the printed command is correct.

## Why it matters — it is worse than the problem it repairs

The two failure modes are not symmetric:

- A boundary that is **too wide** over-reports. Foreign paths show up in
  `slice conformance`'s `undeclared` cell, where a reader sees them and dismisses
  them.
- A boundary that is **too narrow** under-reports, silently. Paths the phase
  really touched vanish from `conformant`, real deliverables appear as
  `undelivered`, and `slice verify-vt` reports `UNATTRIBUTABLE` for criteria whose
  test files sit in the dropped commits. Nothing says a range was truncated.

So the advisory trades a visible error for an invisible one.

## Observed

SL-244 PHASE-08, seven code commits (`21ac1e7d..3aed3918`). The solo binding
recorded `(b8df2efa, 07cba2b2)` — correct start, end polluted by three trailing
`chore` commits — and the advisory fired. It was followed as printed
(`--commit 3aed3918`), yielding `(e8cc2dc1, 3aed3918)`: end repaired, six of
seven commits dropped.

Downstream, at audit (`RV-345` `F-1`):

- `slice conformance 244` reported `undelivered: install/design-run-stages.md` —
  the phase's headline deliverable, sitting on disk.
- `slice verify-vt 244` reported PHASE-08 `VT-5` `UNATTRIBUTABLE — keyword
  present but src/publication.rs not modified by this slice`.
- `src/publication.rs`'s missing selector (`RV-345` `F-3`) stayed invisible,
  because the commit that touched it was outside the recorded range.

Repaired at audit with `record-delta --start b8df2efa --end 3aed3918`, after
which conformance read 42 conformant / 1 undelivered and all 40 VT rows PASS.

## Candidate resolutions

1. **Print the correct shape.** Emit
   `--start <parent of this phase's first code commit> --end <this phase's own
   code tip>`, filled in from the range the advisory already walked — it holds
   the oneline list, so it can name real oids rather than a placeholder.
2. **Refuse the narrowing.** Have `record-delta --commit S` warn (or refuse
   without `--force`) when the sheet's stamped `code_start_oid` is an ancestor of
   `S^` — i.e. when the caller is about to record a range strictly narrower than
   the phase was stamped for. This catches the mistake wherever it is made, not
   only when the advisory prompted it.
3. **Both.** (1) removes the trap; (2) closes the class.

(2) is the load-bearing half: the stamped `code_start_oid` is already on the
sheet and is exactly the fact that would have caught this.

## Related

- `IMP-175` — solo phase-binding stamps a stale `code_start_oid` when edge
  advances before land. The *other* end of the same boundary-accuracy problem;
  the advisory here exists because of it.
- `mem.pattern.doctrine.conformance-needs-a-correct-boundary-row` — the practice
  memory recorded during SL-244 PHASE-06. Its step 3 already distinguishes the
  two modes correctly ("`--commit` for a single-commit phase; `--start`/`--end`
  for a multi-commit one"). The CLI's own advisory contradicts it, and the CLI is
  what an agent reads at the moment of the flip.
- `RV-345` `F-1` — the audit finding this was extracted from.
