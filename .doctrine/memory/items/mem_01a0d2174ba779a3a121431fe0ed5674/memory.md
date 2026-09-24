`doctrine slice phase <id> PHASE-NN --status in_progress` stamps
`code_start_oid` = the tip at that moment. The engine expects the phase span to
run to the completion stamp, so any commit another agent lands between the two —
in a shared primary worktree — falls inside the phase range and would be
attributed to this phase.

`doctrine slice record-delta --commit <S>` guards against this: it refuses a
narrower span that drops commits the sheet stamp covers, and its remedy line is
either "span the phase" or `--force`.

When the dropped commits are FOREIGN (not yours), do NOT span them. Name your
own span explicitly:

    doctrine slice record-delta <id> PHASE-NN \
      --start <your-first-own-commit>^ --end <your-own-tip>

`--start` names both ends deliberately and is never guarded, so it records
exactly your commits and leaves the foreign ones with their own slice. `--force
--commit <S>` gives the same span but reads as an override; the explicit
`--start/--end` states the intent.

Observed SL-263 PHASE-03: another agent's SL-261 commits c80c38599/78efda9d4
landed before the flip and 6d3811eee after, so the naive completion span
(78efda9d4..mine) carried a foreign commit.
