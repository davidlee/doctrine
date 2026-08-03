# Archived phase sheets — SL-241

**These are frozen exhibits. They are not live tracking, and they are not
authoritative for anything.**

Copied verbatim from `.doctrine/state/slice/241/phases/` at the end of PHASE-05,
2026-08-03 — refreshed as the last authored act of T9, so the sheets carry their
own close. `boundaries.toml` rides along because its PHASE-05 row was
hand-corrected at close and the correction is part of the record (F-P05-47). That directory is **runtime tier** — gitignored, disposable, and
`rm -rf`'d at slice close. These copies exist so that the `F-P05-nn` / `D-P05-nn`
citations in `../README.md`, `../matrix.md`, `../guards.md` and in EVD-006…011
still resolve after the originals are gone.

## Why copied rather than distilled

The findings here were expensive. Several took a session each to reach, and at
least one records its own first reading being wrong and corrected further down
the same entry. A tidied summary would keep the conclusion and lose the
reasoning — and **F-P05-39** is the standing lesson about what that costs: the
falsification drivers for T4a–T4e were never tracked, and their loss took
re-runnability with them even though the scored results survived.

So this is a straight copy. It is cheap, it is complete, and it does not spend
judgement deciding in advance which reasoning a future reader will need.

## How to read them

- **`phase-05.md`** is the substantial one — ~3,000 lines. `## Findings` is
  **newest first** (F-P05-45 at the top, F-P05-1 at the bottom); `## Decisions` is
  oldest first. Grep to the id you are chasing rather than reading forward.
- **`phase-01.md` … `phase-04.md`** carry `F-P0n-nn` / `D-P0n-nn` for the earlier
  phases. `phase-06.md` is an unexpanded stub.
- The `.toml` sheets carry each phase's criteria ids. The **authoritative**
  criteria are `.doctrine/slice/241/plan.toml`, not these.
- Checkboxes, task lists and "next unit is…" prose are **stale by construction**.
  They record where the work stood when the sheet was frozen.

## What is authoritative instead

| for | read |
|---|---|
| slice state, phase status | `doctrine slice show 241` |
| the durable harvest — decisions, findings, evidence lifted out of these sheets | `.doctrine/slice/241/notes.md` |
| phase criteria (`EN-`/`EX-`/`VT-`/`VA-`) | `.doctrine/slice/241/plan.toml` |
| the design these all serve | `.doctrine/slice/241/design.md` |
| the probe results themselves | `../results-c3.tsv`, `../results-guards.tsv` |
