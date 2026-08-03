# DEC-134: Interactive orchestration persists outside fresh phase capsules

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Context

RFC-025 requires token-efficient orchestration across several phase
transactions while also requiring fresh mutable capsule state per phase. Its
red-team topology finding identified two possibilities: keep the interactive
session in the trusted control plane, or attempt to preserve an interactive
session inside a capsule while replacing its workspace underneath it.

SL-241's locked design treated the first option as a binding v0 constraint. The
post-spike cleanup promotes that already-approved topology into durable
knowledge rather than leaving it implicit in a completed slice.

## Decision

The persistent interactive session lives in the trusted control plane. Each
phase runs as a headless, fresh-context worker process in fresh mutable capsule
state. No in-session control-plane subagent is the capsule worker.

The topology preserves token efficiency because the control-plane
orchestrator's conversational context persists while phase workers already have
per-phase context lifetimes. Mutable capsule state does not need to survive to
preserve that context.

Escalation from a headless worker is a payload-minimal notification followed by
a halt. The control plane may answer through a separately-proven session-resume
mechanism or provision a new worker with an amended contract; session resume is
not established by this decision.

The capsule boundary is the worker trust unit. The target design does not retain
disk-marker or harness-subagent identity choreography inside the capsule. This
is a target-state decision: ADR-011's incumbent Claude `Agent` arm and its marker
remain authoritative until a capsule implementation cuts over.

## Consequences

- Replacing or destroying phase-local files does not invalidate the interactive
  orchestrator's process or working directory.
- Human pairing inside a capsule is a later, separate design.
- Immutable provisioning layers or caches may be reused, but mutable phase
  execution state is fresh in v0.
- Uniform subprocess launch is a migration target, not a claim about current
  harness parity.

## Origin

Promotes RFC-025's red-team topology finding and SL-241's locked v0 design. It
shapes the RFC cleanup in CHR-053 and the later governance revision scoped by
CHR-054.
