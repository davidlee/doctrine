# ISS-464: verify-vt reports keyword present for absent keywords

`doctrine slice verify-vt` short-circuits on **attribution**, so a `VT` row whose
test file exists but whose mandated keywords are **absent** can still print
"keyword present". The row resolves as `UNATTRIBUTABLE` where it should be
`FAIL`.

It does distinguish a *missing file* from a present one. It does not distinguish
a *missing keyword* from a present one once attribution fails.

## Why this matters

`VT` rows are the slice lifecycle's machine-checkable verification tier. Their
whole job is to answer "does the test that proves this criterion actually
exist?" A tool that answers "yes" when the answer is "no" lets a phase close
carrying a verification that proves nothing — and it does so in the one place an
agent is *supposed* to trust the tool over its own reading.

Discovered in SL-246 PHASE-03/04, where an orchestrator had to be warned not to
read `verify-vt` as evidence that tests exist.

## Related exit-code defect, same surface

`verify-vt`'s **exit code is unusable per-phase**. In SL-246 it reports `FAIL`
for PHASE-05 `VT-1`..`VT-3` purely because that phase's test file does not exist
yet — correct for those rows, but it means a slice mid-execution always exits
non-zero and the individual rows must be read by eye. Any gate wired to the exit
code would be permanently red until the last phase lands.

## The shape of a fix

Separate the three outcomes the surface currently collapses: file missing,
file present but keyword absent (`FAIL`), keyword present (`PASS`). Keep
`UNATTRIBUTABLE` for the case it actually names, and do not let it absorb a
genuine keyword miss. Consider scoping the exit code to the phases a caller
names, so a mid-flight slice can be gated at all.

Surfaced during SL-246 PHASE-03/04 (capsule-driver).
