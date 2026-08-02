# ISS-297: prepare-review gate misreads phases planned mid-drive

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`dispatch sync --prepare-review` refuses a legitimate landing when a slice gained
phases *during* its drive, and the refusal names the wrong cause.

## What happens

`src/dispatch.rs` step (4) upserts every committed-ledger row into the primary
registry; step (5) then gates on

```rust
crate::state::registry_completeness(&primary, &primary, slice)
```

whose completed-set is `completed_phase_ids(primary)` — the **primary tree's
runtime phase sheets**. That is sound only when the primary knows every phase in
the plan.

A dispatched slice can gain phases mid-drive: the plan is authored in the
coordination worktree on `dispatch/<slice>`, so a phase added there never reaches
the primary until integrate projects the authored state. Integrate runs *after*
prepare-review. So the primary's sheet set is a strict subset of the real phase
set, every phase added mid-drive reports as `Extra`, and the gate bails.

## Where it bit

SL-233 landing, 2026-08-02. Phases were split into PHASE-13…16 during the drive
(`4758da7a6`, `001e28a81`). At landing the coord tree had a 16-phase `plan.toml`
and 16 completed sheets; `edge` still had 12 of each. The registry held all 16
rows, the primary's completed-set held 12 — so:

```
recorded row for PHASE-13, which is not a completed phase; …PHASE-14…; …PHASE-15…;
…PHASE-16…; record-delta the missing phase(s) before audit
```

Four `Extra` gaps and **zero** `Missing` — the signature of this bug, since a
genuine record-delta gap produces `Missing`, not `Extra`.

## Why the message costs so much

It is actively misleading in two ways, and both were paid in full:

- **It prescribes the wrong repair.** "record-delta the missing phase(s)" sends
  you to record a delta. Nothing was unrecorded — all 16 rows were present. The
  defect is on the *other* side of the comparison.
- **The word "registry" is ambiguous on disk.** Two same-named files exist:
  `.doctrine/dispatch/<NNN>/boundaries.toml` (the committed ledger, authored-looking,
  the one you find first) and `.doctrine/state/slice/<NNN>/boundaries.toml`
  (`boundaries_path` → runtime, the one the gate reads). The error names neither
  path, so the obvious file is the wrong file.

## Shape of a fix

Three candidates, cheapest first — none scoped yet.

1. **Fix the message.** An `Extra` gap with zero `Missing` is diagnosable: say so,
   name the resolved registry *path*, and name the primary's sheet set. Cheap, and
   it converts a dead end into a signpost even if the rest is deferred.
2. **Fix the completed-set source.** The composite phase-truth resolver already
   knows a phase landed from the committed `phase/<slice>-NN` ref
   (`RefPresent`). Reading completion from composite truth rather than the
   primary's sheets removes the staleness assumption at its root.
3. **Make the gate tolerate the known-good case.** A recorded row whose phase is
   completed *in the coordination tree* is not an anomaly during a live drive.

Preference is (1) unconditionally, plus (2) if it is as cheap as it looks — (3)
narrows the gate's meaning and should be the fallback, not the first move.

## Workaround used

Copied the four runtime phase sheets from the coordination tree into the primary's
`.doctrine/state/slice/233/phases/`. Gitignored, disposable, exact copies rather
than hand-authored — and it clears the gate *for the right reason*: all 16 phases
genuinely are completed and all 16 genuinely do have recorded rows. `prepare-review`
then succeeded, creating 9 refs.

Captured as a friction observation at the moment it bit:
`.doctrine/observations/records/a6/019fc03f-a729-7e73-a2a4-b8c1a635fba6.toml`.
