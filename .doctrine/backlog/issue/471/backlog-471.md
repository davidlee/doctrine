# ISS-471: Capsule tier leaves notes.md to the disposable phase sheets

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced as `RV-372` `F-12` at `SL-246`'s reconciliation audit. Same seam as
`ISS-470`; this is the half with no detector.

## Observed

`SL-246` was driven by `/capsule-driver`. When the audit opened, `notes.md`'s
harvest line still read:

    fresh-as-of: 2026-09-18 · design run at `reviewing` (revision 59, materialised)

Execution ran on the 19th. Six phases of material sat **only** in
`.doctrine/state/slice/246/phases/` — 4,457 lines of runtime phase sheet, which
the storage rule makes `rm -rf`-able by contract. Among what existed nowhere
else:

- `PHASE-01` `F-1`, the blankness predicate — an empty facet list is `List([])`
  and can never be `Absent`, so the authored unfilled-marker rule was
  unsatisfiable for two of seven record kinds. It became `PHASE-03`'s `EX-2`
  amendment.
- `PHASE-03` `F-a` … `F-f`, six escalations; `F-a` became `EX-4`'s arity
  amendment and `F-b` became `CHR-074` and this audit's `F-3`.
- `PHASE-06`'s traced deletion chain — four symbols (`scaffold_design_doc`,
  `design_scaffold`, `render_design`, `DESIGN_DEPRECATION_NOTICE`) named in **no**
  exit criterion and in **no** `VA` grep list, plus the planner's note on why a
  deletion phase needs a call graph rather than a grep.

## The seam

| path | who keeps `notes.md` |
|---|---|
| `/execute` | itself — its stated outcome (`SKILL.md:112`) is *"Notes and durable memory stay current throughout execution"* |
| `/capsule-driver` | **nobody**. Three agent definitions plus the driver skill contain zero occurrences of *notes*, at any spelling |

`/audit` step 6 does own a sweep of the phase sheets into `notes.md` — *"the
audit-specific lens"* — so the material has a rightful collector. But only
**downstream**, after a window in which the sole copy is disposable runtime
state. On this slice that window was days wide.

Structural, not a lapse: the orchestrator holds no `Edit`/`Write` by design, the
driver is *"Do not implement"*, the worker is scoped to one phase and is never
told `notes.md` exists.

## Why this is the worse half

`ISS-470`'s divergence is printed by `slice status` unprompted. **Nothing detects
this one.** It was found because an auditor went looking. Clear runtime state
first — which the storage rule expressly permits — and the loss is both
unrecoverable *and* invisible: `notes.md` reads as merely thin, not as truncated.

## Candidate fix — a scribe seat

Proposed by the repository owner at `SL-246`'s close: the super-orchestrator
spawns a **dedicated agent whose sole purpose is maintaining slice-tier state —
`notes.md`, and the lifecycle — from the phase sheets.**

What recommends it over the alternatives:

- It **respects the orchestrator's no-`Edit`/`Write` bound** rather than
  breaking it. That bound is load-bearing (an orchestrator that patches things
  burns the budget the tiering protects), so a fix that widens it trades the
  problem for a worse one.
- It is a **fresh, cheap context** that reads sheets and writes one file — it
  does not spend the orchestrator's accumulated budget, which is the resource the
  whole tier is shaped around.
- Run **per charter** (~3 phases), it bounds the disposable window to a charter
  rather than to the whole slice.
- It **subsumes `ISS-470`** — a seat that owns slice-tier state owns the
  lifecycle flip too.

Open questions for whoever takes it:

1. **Cadence** — per charter, or per phase? Per phase is safer and costs a spawn
   each time; per charter bounds the loss to ~3 phases.
2. **Scope** — `notes.md` only, or the lifecycle as well? If both, `ISS-470`
   closes with it and the `audit` transition needs the same care `ISS-470`
   describes: moving the slice is not beginning the audit.
3. **Division with `/audit` step 6** — audit's sweep is explicitly *the
   audit-specific lens*. A scribe writing the same file needs a stated division
   of labour, or the two produce duplicate or contradictory harvest.
4. **Ordering against state cleanup** — the scribe needs the phase sheets to
   still exist. Whatever runs it must be ordered before anything that may clear
   `.doctrine/state/`.

Related: `RV-372` `F-12`, `SL-246`, `ISS-470`, `/capsule-driver`, `/audit` step 6.
