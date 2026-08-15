# CHR-065: Re-measure inquiry-map use once the map is surfaced

## Why this exists separately from ISS-299

`QUE-218` (*Does the inquiry map earn a semantic tier?*) turns on which of three
readings is true: the tier is **under-featured**, it is **under-surfaced**, or it
is **not needed at slice altitude**. `RFC-026` **E8.3** is the only measurement —
SL-243's run reached nine nodes with **zero edges of either kind** across
seventeen revisions, driven by a competent agent never told not to use them —
and it cannot separate the three, because nothing put the map in front of that
agent (`ISS-299`: zero references to `frontier`, `map`, `design show` or a
decision tree across every shipped design-prompt asset, including the
`inquiry.md` fragment delivered every turn).

`ISS-299` fixes the surfacing. It does **not** re-measure, and its fix landing is
exactly the moment the measurement becomes possible and the moment everyone stops
thinking about it. Hence a separate item with `needs ISS-299`, so it surfaces in
`doctrine next` when it becomes actionable and not before.

## What to do

On the **next real design run** after `ISS-299` lands — a live slice, not a
fixture, because E8.3's finding is about what an agent actually does when nothing
compels it:

1. Read the run state at close: node count, `parent` edge count, `needs` edge
   count, how many frontier entries carried a non-zero `needs_in_degree`, and
   whether `active_path` was ever non-empty.
2. Record whether the agent consulted the map without being told to, and whether
   any edge it authored changed what it did next.
3. Compare against E8.3's baseline (9 nodes / 0 edges / 0 in-degree / empty
   `active_path`) and settle `QUE-218` with the disposition the numbers support.

**One run is a data point, not a result.** If the first run is ambiguous, say so
and take a second rather than settling on n=1 — E8.2's own reading was revised
at revision 28 after "never fired" turned out to be a snapshot stated as a
property.

## What settles on the answer

`IMP-386` (cascade), `IMP-387` (cauterised-by edge), `IMP-388` (structured
rejected alternatives) and `IMP-389` (derived next candidate) all carry
`needs QUE-218`. They are one slice's worth of work, blocked on warrant rather
than on effort.

`ISS-300` and `ISS-303` are deliberately **not** gated — correctness defects in
the moves that already exist — and `ISS-303`'s fix (retain the prior disposition
across a reopen) is the `prior` capability `IMP-386` would build on, so it is
worth having either way.

## References

- `QUE-218` — the question this settles
- `ISS-299` — the prerequisite
- `RFC-026` E8.3, E8.5, E8.6 — the baseline, the capability table, the asymmetry
- `SPEC-029` — Design run engine
