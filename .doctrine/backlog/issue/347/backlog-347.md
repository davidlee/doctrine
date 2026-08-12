# ISS-347: REQ-288 and SPEC-021 misstate the dispatch env-marker as .claude/ presence

Found during `SL-254`'s research round (thread 2, verified by the assembling
agent).

`REQ-288` (`SPEC-021` `FR-002`) states the dispatch arm is selected by *"the
env-marker (`.claude/` presence)"*. `SPEC-021` responsibility 2
(`spec-021.toml:16`) repeats it.

The implementation tests **`CLAUDECODE=1`**, not `.claude/` presence —
`.agents/skills/dispatch/SKILL.md:29-30`. `.claude/` presence *is* read, but at
`boot.rs:709`, for harness-pair detection during install-target selection, which
has nothing to do with dispatch arm routing. Two distinct signals were conflated
in the spec text.

## Why this is filed rather than absorbed

`SL-254` retires `REQ-288` outright, so the error is fixed by deletion — but
that closes the drift without recording that it happened. The interesting part
is not the wrong clause; it is that a requirement carried a false mechanism
claim undetected, which is a signal about how requirement text gets reviewed
against implementation. And if `SL-254` stalls or narrows, the defect survives
with nothing pointing at it.

Discharges when `SL-254` retires `REQ-288`, or by direct correction if that
slice does not land.
