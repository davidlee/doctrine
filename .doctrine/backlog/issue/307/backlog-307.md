# ISS-307: Phase completion flip captures HEAD as code_end_oid, so a shared tree records a foreign commit

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`doctrine slice phase <id> <PHASE> --status completed` captures the phase's
`code_end_oid` from **`HEAD` at the moment of the flip**. In a tree where several
agents commit concurrently — this repo's normal condition — `HEAD` at that
instant may belong to an entirely different slice.

Observed closing SL-241 PHASE-05 (2026-08-03):

```toml
[[boundary]]
phase = "PHASE-05"
code_start_oid = "8e962656…"
code_end_oid    = "5c78c892…"   # design(SL-244): DEC-121 — another slice's commit
```

The phase's real code tip is `fdebae1e` (its last source-touching commit); the
three tasks after it were authoring only. Corrected by hand in
`.doctrine/state/slice/241/boundaries.toml` (runtime tier, hand-editable).

## Why the existing warning does not cover it

The flip does warn:

> PHASE-05 boundary spans 12+ commits — any that are not this phase's are
> attributed to it; review before audit
> correct with: `doctrine slice record-delta <id> <PHASE> --commit <code tip>`

That describes **range width**, which for any multi-commit phase in a shared tree
is expected and already documented per-slice (SL-241's notes carry a "111 foreign
commits" section). So it reads as known-and-fine. It does **not** say *the
endpoint I just wrote belongs to another slice*, which is a different failure and
a worse one: it silently moves the phase's own boundary rather than widening it.

The suggested correction also points at `record-delta`, a **different store**.
In SL-241's case `record-delta` was already correct — it takes explicit
`--start`/`--end`. The two stores disagreed, and only the registry asks for its
endpoints.

## Scope to settle

- **Where should `code_end_oid` come from?** Candidates: the last commit touching
  the slice's `design-target` selectors; the `record-delta` registry's `end` when
  one is recorded; or an explicit flag on the flip. The registry option is
  attractive because it already carries a guarded, author-supplied range.
- **Should the flip refuse rather than guess** when `HEAD` is not attributable to
  the slice? A refusal that names the likely tip is cheap and this is a
  hard-to-notice wrong value, not a stylistic one.
- **The four already-closed boundaries in SL-241 are unverified** —
  PHASE-01..04's `code_end_oid`s were captured the same way. Worth checking
  before their ranges are believed at audit. Other slices likewise.

## Related

- SL-241 PHASE-05 F-P05-47 — where this was found, with the near-miss.
- SL-241 `notes.md` § PHASE-05 boundary — the 111-foreign-commit context that
  makes the width warning read as benign.
- F-P04-6 — the shared-tree hazard record this joins.
- ISS-306 — the other tooling gap found in the same phase.

## Second sighting — SL-250 audit (RV-350 `F-5`, 2026-08-08)

The same defect, at a scale that made the conformance report unusable as a
scope-creep signal rather than merely inaccurate.

`doctrine slice conformance 250` reported **27 undeclared** paths. Attributing
them commit-by-commit, **20 were foreign** — committed by other agents on the
shared `edge` branch during SL-250's execution window:

- `src/plan.rs` — SPEC-031 / IMP-382 / ISS-321
- `.doctrine/rfc/027/**`, `.doctrine/rfc/029/**`
- `.doctrine/slice/248/{notes.md,plan.md,plan.toml,slice-248.toml}`
- `.doctrine/backlog/improvement/411/**`, `.doctrine/backlog/issue/321/**`
- `.doctrine/knowledge/decision/181/**`
- two `.doctrine/observations/records/**`, one `.doctrine/memory/items/**`

Also absorbed: a `.claude-plugin/marketplace.json` edit from the `chore: v0.37.0`
release bump (`88d6ecfc9`). SL-250 `PHASE-06` `EX-5` requires that file to be
**untouched** — it *is* untouched by the slice, but the report cannot say so, so
a criterion's own evidence is inverted by the capture.

**Why this is worse than a wide range.** The audit skill designates the
undeclared cell as the *highest-signal* lead and instructs that each entry be
treated as a finding candidate. At 20/27 foreign the cell inverts: the auditor
must clear it by hand before any entry can be read as a lead, and an auditor who
does not will raise findings against other slices' work. The existing width
warning does not help — it describes span, and every entry here sits inside a
legitimately wide span.

Ranges here come from `code_start_oid` per phase
(`.doctrine/state/slice/250/phases/phase-0N.toml`), so the same root cause
applies to the *start* boundary as to `code_end_oid` — every concurrent commit
between two phase starts lands in the earlier phase's delta. Worth settling
alongside `IMP-175` (stale `code_start_oid`) and `IMP-282` (exclude slice-own
process artifacts), which together cover three faces of one problem.
