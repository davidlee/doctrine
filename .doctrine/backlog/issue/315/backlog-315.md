# ISS-315: Retired ChangeEvent variants make historical design-run snapshots unreadable

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

`doctrine design show 244` fails outright:

```
Error: parse .doctrine/state/slice/244/design.toml
  TOML parse error at line 4991, column 9
  4991 | event = "integrated_review_recorded"
  unknown variant `integrated_review_recorded`, expected one of `node_created`, …
```

`ChangeEvent` is a closed enum deserialised strictly, so **one** unrecognised row
fails the whole snapshot — not just the change log, and not just that row.

## Why it happened

`SL-244` `PHASE-04` (`4943333a`, `T2`/`EX-2`) retired `IntegratedReview` whole,
including `ChangeEvent::IntegratedReviewRecorded`. That is correct for the
**writer**: nothing can emit the row any more, and `EX-2`'s *"a retirement that
leaves a refusal nothing can raise is half a retirement"* is right about the
write path.

But the change log is **append-only history**. `SL-244`'s own live design run
already held one such row, written months earlier when the run declared `int-1`.
Retiring the variant retro-actively invalidated a snapshot that was valid when
written.

Scope at time of filing: `SL-244` 1 row, `SL-243` 0 rows.

## The general defect

The vocabulary a change log **writes** and the vocabulary it must **read** are
not the same set, and the type currently conflates them. Every future retirement
of a `ChangeEvent` member has this same effect on any run that recorded one.
The same class applies to any strictly-deserialised closed vocabulary persisted
into long-lived state.

Note the near-sibling that got this right: `RecoveryIntent.subject` keeps its
pre-`DEC-125` key as `#[serde(alias = "checkpoint")]` precisely because
snapshots outlive binaries — see `SL-244` `7e2b768d`, which reverted an attempt
to drop it and pinned it with
`a_snapshot_written_before_the_intent_subject_key_still_parses`
(`src/design_run/snapshot.rs`). That test is the shape this issue's fix wants.

## Candidate resolutions (not decided)

1. **Retired-but-readable members.** Keep deleted variants as read-only,
   unconstructible history — the write path still cannot emit them, so `EX-2`'s
   intent survives. Needs a convention so "retired" is visible rather than
   looking like an incomplete retirement.
2. **Lenient row deserialisation.** An unknown `event` degrades that one row
   rather than failing the file. Loses fidelity; may be right for a log.
3. **Migrate on read.** A version-keyed rewrite pass. Heaviest; the storage
   model calls runtime state disposable, which argues against a migration
   framework for it.

Option 1 is the cheapest and preserves the most, but the choice is a governance
question about what a closed vocabulary owes its own history — `ADR-009`'s
neighbourhood — so it wants deciding, not improvising.

## Immediate impact

`SL-244`'s locked design run cannot be read via `design show` until this is
resolved. The run's authored outputs (`design.md`, `plan.toml`, the slice's
`notes.md`) are unaffected — this is a runtime-state read, and no authored
artefact is at risk. Implementation of `PHASE-05` onward does not depend on
reading the run.
