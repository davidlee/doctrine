# ISS-342: Live-bwrap conformance tests keep just gate permanently red off-jail

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`just release` reaches `just gate` (`justfile:22`), which runs `just test-all` =
`cargo test --workspace` (`justfile:77`). On the owner's host, unjailed, nine
`doctrine-control` conformance tests fail, so **the release gate is red**. In the
development jail the same suite is 299 passed / 0 failed / 9 ignored (measured
2026-08-10; the nine ignored are re-execution instruments and are a disjoint set
from the nine failures).

This is a `cargo test` run on the host, not a `nix build` check phase —
`flake.nix:311` sets `doCheck = false` and the flake exposes no `checks` output.

## The structural finding, which is the item

The conformance suite's rows are proved by **live `bwrap` integration tests
wearing `#[test]`**. `SL-248` `PHASE-09` `EX-14` deliberately forbids them from
skipping when the host cannot oblige — *no skip stands in for a claim* — and
that rule is correct for `doctrine-control backend verify`, the shipped verb
whose whole job is to refuse a green transcript it did not earn.

The consequence nobody decided: `cargo test` is permanently red on any host that
cannot host a capsule, and therefore so are `just gate` and `just release`. A
rule that is right at the verb's altitude was inherited by the test harness,
where it means something different — `backend verify` reporting *this host cannot
prove these rows* is information, whereas `cargo test` reporting it is a broken
build.

**The decision this item owns is what `cargo test` should mean for a
host-capability-dependent claim.** Not the nine test names; they are symptoms of
that one unmade decision, and fixing them individually would leave the next
capability-dependent row to rediscover it.

## The nine, and who owns them

Three sit in `SL-252`'s neighbourhood (the fixture's readable-input posture) and
are that slice's to fix or not, as a consequence of its narrowing:

- `bounded_input_set_is_proven`
- `a_binary_is_executed_from_each_bound_path`
- `the_shipped_backend_is_admitted_on_this_host`

Six are **this item's**, and none is reached by `SL-252`'s scope:

- `concurrent_capsules_cannot_see_each_others_processes`
- `control_with_the_pid_namespace_shared_both_become_possible`
- `the_observed_pid_is_the_one_the_parent_reported_not_one_the_subject_printed`
- `the_sweep_reaches_what_row_b5s_control_leaks`
- `a_write_through_every_readable_mount_fails`
- `every_shipped_rows_control_is_seen_to_fail`

The first four are process/namespace rows; the last two are unclassified and
want a diagnostic pass before anyone theorises about them.

## What must not happen

Making the tests skip, or narrowing them until they pass, without deciding the
question above. That is `EX-14`'s prohibition arriving by the back door, and it
is the same reflex `SL-252`'s scope rejects for `ISS-340` — turning the
transcript green while the defect stands.

## Provenance

Raised from `SL-252`'s design run while resolving its `inq-9` scope question;
the scoping decision is `DEC-184`. Same family as `ISS-339` (the suite has never
run off-jail), which is where the measurement discipline that surfaced this
came from.
