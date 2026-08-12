# Test bands are carved before the split

`SL-253` `inq-6` asked how 186 tests are carved along the seam, and warned that
this is unbudgeted work the phase plan must carry explicitly rather than absorb.

## The premise, checked

There is exactly one `#[cfg(test)]` at `conformance.rs:5339` and **no inner
`mod` declarations at all** below it — 186 test functions, flat, to EOF. The
crate is bin-only by declared intent (`Cargo.toml`: *"every test here is a
`#[cfg(test)]` module inside the unit it tests"*), so a `tests/` directory is not
available and the tests must follow their code into the kernel and payload
modules.

## Sizing the work

A crude symbol-based triage over the 186 — an estimate for budgeting, **not** a
classification:

| | count |
|---|---|
| names only kernel symbols | 46 |
| names only payload symbols | 73 |
| names both — needs a judgement | 28 |
| names neither — helper, needs reading | 39 |

So roughly **67 need individual triage** and 119 sort themselves. That number
goes into the phase plan rather than being discovered inside a phase.

## Carve first

The flat `mod tests` is sub-moduled into two bands — kernel and payload —
**before any code moves**. It is a pure reorganisation against stable types, and
its exit criterion is mechanical: same 186 test names, all green, nothing
semantic changed. The split then moves whole sub-modules rather than rewriting a
test file.

This lands in the pre-split phase `DEC-199` already requires for the
characterisation test, so it adds a task to an existing phase rather than adding
a phase.

The second reason to front-load it is the 67 judgement calls. Taken together in
one phase they can be compared against each other; spread across the phases that
move code, each one is a small decision made while doing something else, which is
where they get waved through.

## Two bands, not three

A third band — splitting the payload into live-`bwrap` and neutral, so
`just capsule-check` runs meaningfully on a host without `bwrap` — is **deferred,
not refused**. It is scope creep against a slice already carrying a split, a
rename and a revision, and a third band must never become the skip `EX-14` and
`DEC-156` forbid. `ISS-342`'s fail-loud posture is unchanged by this decision.
