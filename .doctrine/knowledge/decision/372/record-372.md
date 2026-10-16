Installing every agent def in a harness's def set under an explicit --agent leg is lawful under SPEC-009 D7: the leg is user-requested and the def path is operationally necessary for the harness to load it (ADR-019 position 1). D7's 'not projected by default' governs the base fileset. SL-281's REV records this reading as a D7 clarification. Capsule defs remain published (DEC-010 complement unchanged — they are not base backings). Rejected: reading B (D7 forbids; would leave IMP-468 open).


## Amendment (2026-10-16, RV-420 F-8)

The lawful trigger is wider than an explicit `--agent`: install also runs the
full-def-set leg for a harness it detected and the user confirmed (at the
prompt, or under `-y`). Confirmation is a request, so the same reading holds —
the leg is user-requested and operationally necessary. The user chose this
reading over restricting the code to explicit `--agent` only.
