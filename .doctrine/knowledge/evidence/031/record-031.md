# EVD-031: Plan claims are frozen prose that the tree outruns

## Observation

Source: the friction-observation corpus as of 2026-10-01 — 758 active `friction`
records, 2026-07-28 → 2026-09-29 (`doctrine observation list --kind friction`).

By manual read of the summaries, roughly 100 records (the largest single cluster)
report a plan, phase sheet, design passage or worker brief that stated something
about the tree which execution then found false. Keyword matching over the
summaries, with the recording skill's prefix stripped, gives these **overlapping
lower bounds** — a heuristic, not a classification:

| class | matches | typical record |
|---|---|---|
| inventory undercount (touch sets, call sites, tests, mutation targets) | ≥29 | "the survey undercounted an eleventh time, 4× (46 → 191 regions)"; a reading list omitting `src/commands/guard.rs`, whose exhaustive match makes a new CLI variant a compile error |
| asserted from reading, not compiling | ≥21 | mutation arms that cannot compile under `-D unused` (three consecutive phases); a signature clippy `unnecessary_wraps` rejects; derives transcribed from recollection; one fabricated symbol |
| criterion wrong as written | ≥19 | VA checks vacuous as worded; predicted red of the wrong kind; a VT `test_file` sited wrongly four times in one slice, the same way, after DEC-140 had already settled it |
| positional citations rot | ≥17 | `plan.toml` / `design.md` line citations invalidated by an earlier phase of the same slice; multi-tranche sheets keyed to the pre-phase commit |
| artefacts disagree | ≥16 | runtime sheet stale against an amended `plan.toml` while `slice phases` reports "up to date"; sheet and orchestrator brief naming different notes shards |
| size estimates | (few) | sheet sizing about 2× low; workers over their token bound |

Each defect is paid at execution: typically one ~3-minute gate or compile cycle,
a probe round, or a full worker re-dispatch. Several records note the same class
recurring after an explicit warning against it.

A related multiplier: several lint traps (`dead_code` / `unused` under staged
modules) recurred although memories describing them existed — retrieval did not
surface them (see ISS-507; `memory search` matches no memory-body text).

## Bearing

The common root: a plan states facts about the tree as **prose, frozen when it is
written**, with no recorded derivation, and nothing re-checks them as earlier
phases move the tree. That supports RFC-027 P1 (predictions are authoritative but
provisional) with a measured failure distribution, and suggests a concrete
mechanism: every factual claim in a sheet (count, file set, "compiles",
signature) carries the command that produced it, re-run at phase entry. That one
mechanism would address the undercount, rot and drift classes together.

For RFC-023 it ranks the directions by observed demand: B (declare the edit set,
check it at plan time) and G (`check plan` lints) meet the undercount and
criterion classes; F (a durable home for the negative control) meets the
mutation-arm class, where workers currently hand-roll an apply/compile/red
driver and two failure modes — a mutation that silently did not apply, and one
that cannot compile — recur.

Not tool-addressable: fabricated gate results and ignored worker rules belong to
worker contracts and prompts.
