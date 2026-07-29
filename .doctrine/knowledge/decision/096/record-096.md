# DEC-096: Narrow phase truth to landed-vs-sheet; retire --across-trees

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

`resolve_phase_truth` (`src/state.rs:1114`) **narrows, it does not die**. It loses
the coord-vs-local axis and keeps landed-vs-sheet:

```rust
resolve_phase_truth(landed: &[(String, LandedSignal)], sheets: &[(String, StemStatus<'_>)])
```

- The `coord: Option<&[…]>` parameter is dropped.
- **The `Some`-branch matrix is what survives.** The `None` branch is deleted.
- `note_disagreement` (`src/state.rs:1208`) and `Divergence::disagreements` are
  deleted — with one file, `local == coord`, so `local_status != coord_status`
  can never hold and the field is structurally empty.
- `Divergence::conflicts` (landed vs sheet), `phase_set_mismatch`, `unknown` and
  `anomalies` all survive, so `--assert` does **not** go vacuous.

`slice status <ID> --across-trees` keeps its behaviour but is **renamed
`--truth`** — after this slice there are no trees to be across, and the new name
matches `resolve_phase_truth` and the existing "composite truth" vocabulary.

## The load-bearing detail: which branch survives

`resolve_phase_truth` has two branches, and collapsing to the wrong one silently
guts the verb:

- The **`None`** branch (today's no-live-coord-tree path) resolves *landed ⇒
  Landed*, unconditionally — it never compares the sheet against the oracle.
- The **`Some`** branch runs the full `(landed, sheet)` matrix and produces
  `Conflict(Rework)` / `Conflict(ReworkReset)`.

Falling back to `None` would delete conflict detection outright and leave
`--assert` permanently green. Keeping the `Some` matrix makes the verb *strictly
better on the common path*: standing in the primary with no live coord tree, a
landed-but-`in_progress` phase reports `Landed` silently today, and
`Conflict(Rework)` afterwards.

## Accepted consequences

- **`--assert` will newly fail where it passed.** That is the improvement
  working, not a regression, but it is a real behaviour change on the
  no-coord-tree path. Nothing automated consumes the flag (verified: no
  justfile, skill, or CI caller outside `src/slice.rs`), so the blast radius is
  operator surprise only.
- **`reconcile-phases` keeps its refuse-when-live bail**
  (`src/slice.rs:1236-1244`). The pre-design round noted that bail makes the
  verb's coord branch permanently dead — true, and it is why this narrowing
  costs it nothing. The guard itself stays correct and arguably matters more:
  with one file, an operator reconciling during a live drive now races the
  drive's writer on the *same* file rather than a different one.

## Why not retire the composite outright

The scope doc's objective 5 says "retire". That would take `reconcile-phases`
with it — but that verb's real job is rewriting sheets from the **landed**
oracle after a drive, and that job survives this slice untouched. Retiring the
composite would discard working recovery machinery for a reason that does not
apply to it. Objective 5 is corrected to "narrow" in `slice-237.md`.

Decided by the user in `/design`, 2026-07-29.
