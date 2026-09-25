# ISS-487: review new --target rejects the <ref>@PHASE-NN spelling its own records display

`review new --target SL-261@PHASE-05` → `DANGLING_REF`. The accepted spelling is
`--target SL-261 --phase PHASE-05`, but the stored and displayed target is
`SL-261@PHASE-05` (`RV-375`/`RV-376`). The canonical target the ledger writes is
not an accepted input for the verb that writes it (obs `01a0d200`).

Fix: accept `@PHASE-NN` as a target spelling (split into ref + phase), or stop
displaying it. See RFC-032 `research.md` F4.
