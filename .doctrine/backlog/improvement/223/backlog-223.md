# IMP-223: Live auto-provision test for non-dispatch (Agent-tool Passthrough) jail-policy spawn path

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Surfaced by SL-183 /audit (RV-209 F-5). The macOS Seatbelt confinement CONSUMER
(resolve → wrap → materialize `.sb` → confine) is proven LIVE end-to-end through
the shipped `worktree pretooluse` hook. But the live EX-2 battery used the
Agent-tool spawn, which takes the create-fork **Passthrough** path
(`cwd_is_arming=false`), so the per-worktree `jail/<name>.toml` was provisioned
**manually**. The dispatch **Fork** path auto-copies `spawn/jail.toml` →
`jail/<name>.toml`; that auto-provision trigger on the non-dispatch spawn path
is currently proven only by unit test
(`provision_jail_policy_copies_declaration_to_named_file`, create.rs).

## Detail

A spawn-path seam, not a containment gap — the floor itself is live-verified.
Close it with a live (or integration-level) test that exercises the
auto-provision trigger on the non-dispatch/Passthrough spawn path, so both the
provisioning trigger and the consumer are covered by more than a unit test.

Refs: SL-183, RV-209 F-5, notes.md "EX-2 live battery — RESOLVED"
(provisioning-trigger caveat), mem.thread.sl-183.ex2-live-battery-resolved-macos.
