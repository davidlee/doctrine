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

## Resolution — fixed 2026-08-11 (RV-353 `F-4`)

### The decision this item owned

**`cargo test`'s default selection asserts the claims that hold on any developer
host. A claim that depends on a host *capability* is not a claim the default run
makes — it is a separate instrument, invoked explicitly, and it fails rather than
skips when the host cannot oblige.**

The reason the nine tests were red off-jail is not that they are wrong. They are
right, and `EX-14` / `DEC-156` are right that no skip may stand in for a claim —
this crate executes that ruling on itself at
`no_ignored_test_in_this_crate_stands_in_for_a_claim`, so `#[ignore]`, an
availability guard and a narrowed row were each unavailable to a fix here, and
none was used. The defect was **selection**: `cargo test --workspace` was
asserting a host-capability-dependent claim in a run whose contract is
host-independence, and `just gate` / `just release` inherited the consequence
nobody chose.

Selection is not skipping, and the difference is the whole of the fix. A skip
makes a claim and then abstains from proving it, which is what `EX-14` forbids.
Deselection does not make the claim in that run at all. The claim is still made,
unconditionally and un-skippably, where it can be proved.

### Enacted

- `justfile` — `test-all` names its packages: `cargo test -p doctrine -p cordage`.
  Verified: 0 `doctrine_control` binaries in the gate set, 117 test binaries
  present (the positive control — an empty selection would also report 0).
- `justfile` — new `capsule-check`: `cargo clippy -p doctrine-control` then
  `cargo test -p doctrine-control`. Not wired into `check` or `gate`. Measured
  in-jail: 299 passed, 0 failed, 9 ignored, matching this item's baseline.
- `Cargo.toml` — `default-members = ["."]`, which is `ISS-343`'s fix and also
  removes the crate from bare `cargo test` / `clippy` / `build`.
- `just gate` green in-jail after the change.

### What is NOT claimed

The nine still fail on a host that cannot host a capsule — `just capsule-check`
there will be red, by design. Nothing was diagnosed and nothing was repaired
about the six process/namespace and unclassified rows this item listed as its
own; they are host-capability facts awaiting the `cluster:capsule` triage
(`RV-353` `F-8`, sorted by the `F-13` complexity partition). Their off-jail
behaviour is now *information* rather than a broken build, which is exactly the
altitude distinction this item identified.

Consequence recorded because it is a real loss: the gate's test set is now a
named list, so a new workspace member is no longer auto-gated by `--workspace`.
The `test-all` comment says so and
`mem.pattern.build.just-check-workspace-gates-members` was corrected.
