# A fixture that satisfies `meta::Meta` is not necessarily *scannable*

Measured SL-238 PHASE-04 T0, against `backlog::tests::lifecycle_*`, which reach
their fixture through `relation_graph::scan_entities` rather than through
`meta::read_meta`.

## The two conditions

`meta::Meta` requires only `id`, `slug`, `title`, `status` (`tags` defaults). A
toml carrying exactly those parses fine and `authored_status::read` is happy with
it. **A corpus scan is a stricter consumer.** Two further conditions are
*independently* necessary — each was isolated with the other held fixed:

1. **the `.md` sibling** must exist beside `<stem>-<NNN>.toml`;
2. **`created` and `updated` keys** must be present in the toml.

Neither is a `Meta` field, so neither is visible in the struct you would check.

## Why it costs more than a compile cycle

Omitting either takes `lifecycle_findings` from one finding to **zero, with no
error**. The reverse-`fulfils` appender skips a file it cannot read rather than
failing, so an under-seeded entity is simply *absent from the graph* and every
edge anchored on it silently vanishes.

The symptom is therefore **a wrong count, not a failure** — which reads as a
logic bug in the code under test, not as a fixture defect. Two hypotheses were
tried and disproved before the real cause was isolated.

## The rule

- Testing a *reader* (`authored_status::read`, `meta::read_meta`)? Seed the toml
  alone — `authored_status::test_support::seed_status_bearing` is the minimal
  fixture, and its minimality is the point.
- Testing anything that reaches the entity through a **scan** or the **relation
  graph**? Add `seed_md` and the two date keys.
- When a scan-backed test reports a wrong count, **suspect the fixture before the
  code**, and isolate one variable at a time — changing the toml body and the
  `.md` together proves nothing about either.

See [[mem.pattern.harness.grep-negative-needs-positive-control]] for the same
shape in a different tool: an empty result that is not demonstrated absence.
