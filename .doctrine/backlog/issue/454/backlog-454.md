# ISS-454: Displacing a coverage-dead act reports it invalidated twice

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`SL-259` `PHASE-01` gave both retain-by-kind act stores a displacement report
(`ISS-367`), and `admit_and_record` turns it into an `ActInvalidated` row.
`RV-367` `F-1` found the mirror defect: the row fires for an act that was
**already dead and already reported dead**.

## Mechanism

`admit_and_record` (`src/design_run/run.rs:768-785`) emits `ActInvalidated`
whenever `record.insert(next)` reports a displacement.
`CheckpointActGroup::record` and `AgentDeclarationGroup::record`
(`src/design_run/snapshot.rs:361-370`, `:404-413`) report displacement from the
**store**, and the store retains acts that `live_acts` has already dropped for
coverage — `live_acts` (`run.rs:1962-1982`) *filters*, it does not prune. So a
coverage death and a later displacement of the same corpse both emit a row.

## Reproduction

A scratch probe in `run.rs`'s own suite, three applies over `run_with_a_map()`:

```text
rev 2: declare inq-9                    -> ActInvalidated (3, 1, cpa-sufficiency-accepted)  [coverage]
       live_acts = {} , stored acts = 1 -- the corpse is retained
rev 3: record SufficiencyAccepted again -> ActInvalidated (4, 0, cpa-sufficiency-accepted)  [displacement]
```

A delta reader sees the same `(ActKind, DesignId)` slot die twice with no
intervening `ActRecorded`.

## The claim it contradicts

`invalidation_rows` (`run.rs:2027-2032`) argues no dedup is owed because "the
two cases are disjoint". That holds **within** one revision — an act absent from
both sides of the difference produces no derived row. It does not hold across
revisions, which is where the doubling lives.

## Which side is wrong is the open question

Leg 2 as `SL-259`'s design states it is *every material change emits its row*.
That invariant is **not** violated: the second row is spurious, not missing. So
either

1. the code is wrong — thread `live_acts_before` into `admit_and_record` and
   suppress the row when the displaced slot was not live; or
2. the prose is wrong — state the limit at `invalidation_rows` and
   `admit_and_record` instead of claiming disjointness.

`RV-367` recommends (1) and does not decide it. Either way the fix is small; it
is a behavioural change with a test, which is why it is here rather than inside
`SL-259`'s reconcile.

Raised on `RV-367` (code-review of `SL-259`) as `F-1`, disposed follow-up.
