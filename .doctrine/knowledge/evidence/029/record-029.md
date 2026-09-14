The live suspect for ISS-361's symptom once EVD ruled out the parse site. Confidence is medium deliberately: the ordering is read off the code and is certain, but that it is what the single ISS-361 witness saw is not established.


## Qualified by SPEC-029 (added at `explore.scope`)

The ordering reported above is accurate and re-verifiable. Its framing as a
hole is too strong for the part `SPEC-029` prescribes: the reserve-then-journal
protocol *specifies* that the record materialises before the snapshot completes,
recovery is the journal, and no failure path may delete authored knowledge to
repair a runtime error. The watermark re-check is likewise specified to run
immediately before the write it guards, with journalled effects remaining and
staying recoverable on divergence.

What remains a defect after that subtraction is narrower and still real: a check
that could have run in pass 1 and did not. See `DEC-250`'s correction section.
