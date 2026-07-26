# ISS-251: slice selector doctor reports healthy when a phase objective or exit criterion names an undeclared file

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`doctrine slice selector doctor <id>` returns

```
SL-230: selectors healthy — no findings
```

for a slice whose PHASE-02 **objective** and **EX-2** both name
`bench/index.md`, while no `design-target` selector covers that path. Importing
that phase's delta would refuse `undeclared-scope`.

## Cause

IMP-256 ("plan-time selector completeness check") closed the case it was scoped
to: VT `test_file` paths not present in the design-target set. The paths a phase
names in its **objective** and **exit criteria** are not checked, so a file the
plan explicitly promises to write can be entirely absent from the selector set
and still read as healthy.

## Evidence

Observed twice, independently, in SL-228 PHASE-07's memory-blind benchmark. Both
blind orchestrators ran `selector doctor`, received "healthy — no findings", and
then declared the missing selector anyway by comparing `plan.toml` EX-2 against
the selector list themselves — one of them recording the reason in its commit:

```
slice(SL-230): declare bench/index.md as a design target
plan(SL-230): declare bench/index.md as a design-target selector
             (note: "PHASE-02 cross-link target (plan.toml EX-2)")
```

The gap was closed by operator competence, not by the check that exists to close
it. A less careful operator reaches `undeclared-scope` at import instead — the
rework loop IMP-256 was raised to prevent (case-notes-analysis #4, 8+
occurrences).

## Fix direction

Extend the doctor's source set from VT `test_file` paths to **any repo-relative
path mentioned in a phase's objective / entrance / exit criteria**. A path-shaped
token (contains `/`, has a file extension, resolves in-tree) is a cheap and
precise enough heuristic; false positives are advisory, and the verb is already
advisory.

Worth pairing with a test that a phase naming a file only in EX text is flagged.

Relates to IMP-256, case-notes-analysis #4, and the SL-228 PHASE-07 benchmark
(`.doctrine/slice/228/benchmark.md`).
