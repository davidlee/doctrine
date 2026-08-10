## Premise, confirmed

`just release` reaches `just gate`, which runs `just test-all` = `cargo test --workspace` (`justfile:22`, `justfile:77-78`). Unjailed on the owner's host that is nine red conformance tests, so the release gate is red. This is a host run of `cargo test`, not a `nix build` check phase — `flake.nix:311` sets `doCheck = false` and there is no `checks` output.

In-jail the same suite is 299 passed / 0 failed / 9 ignored (measured this session; the nine ignored are re-execution instruments, a disjoint set from the nine failures).

## The split

Three are this slice's neighbourhood: `bounded_input_set_is_proven`, `a_binary_is_executed_from_each_bound_path`, `the_shipped_backend_is_admitted_on_this_host`.

Six are not: `concurrent_capsules_cannot_see_each_others_processes`, `control_with_the_pid_namespace_shared_both_become_possible`, `the_observed_pid_is_the_one_the_parent_reported_not_one_the_subject_printed`, `the_sweep_reaches_what_row_b5s_control_leaks`, `a_write_through_every_readable_mount_fails`, `every_shipped_rows_control_is_seen_to_fail`.

## The decision

SL-252 fixes what its readable-input narrowing fixes and no more. The six are captured as one sibling item.

## Why not fix the gate here

The cheap route — stop these tests running where they cannot pass — is a packaging change wearing this slice's name, and it is the same reflex SL-252's own scope rejects for `ISS-340`: turning the transcript green while the defect stands. Keeping it out also keeps `inq-8` honest — whether `ISS-340` dissolves under the shape this design picks has to be re-derived, not aimed at.

## The structural finding the sibling carries

These are live-`bwrap` integration tests wearing `#[test]`, and `SL-248` `PHASE-09` `EX-14` deliberately forbids them from skipping when the host cannot oblige — no skip stands in for a claim. That is right for `backend verify`. It also means `cargo test` is permanently red on any host that cannot host a capsule, and therefore so are `just gate` and `just release`. Nobody decided that; it fell out of a correct local rule. The sibling owns the decision, not the six symptoms.

## Provisionality

Held until the design locks. Scope may widen if the shaping work leaves room; the owner rates that unlikely.