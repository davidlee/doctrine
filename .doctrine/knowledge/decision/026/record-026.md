# DEC-026: Observations use a repository-wide sink with arm-specific write delivery

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Observations belong to one repository-wide primary corpus, not to the invoking
worker branch. Capture from a worktree routes to that shared sink so observations
survive rejected, abandoned, or deleted worker forks.

SL-231 exposes the validated create-only observation writer through a narrow MCP
tool for Claude workers. The Claude MCP server runs outside the worker's additional
bwrap jail and can write the primary corpus even though the worker cannot.

SL-231 does not claim equivalent subprocess-arm delivery. A subprocess worker's
stdio MCP process inherits its jail and cannot use the Claude-arm bypass.
IMP-319 owns the follow-up for a brokered subprocess egress path; RSK-225 governs
the security constraint on writable worker MCP tools.

## Rationale

Branch-local observations would be sample-biased: the friction most likely to be
lost is friction from work that never integrates. A shared sink preserves those
signals independently of change outcome.

The delivery mechanism must nevertheless respect confinement. Treating all MCP
deployments as unconfined would silently generalise a Claude-arm property onto the
subprocess arm and produce a non-working or unsafe design.

## Consequences

- The CLI writes to the primary corpus when invoked in a context that can reach it;
  the receipt reports the actual sink path.
- The Claude worker surface grants only the purpose-built observation-write tool,
  not a broad writable Doctrine MCP surface.
- Worker tool allowlisting and the remaining general MCP escape risk stay visible
  under RSK-225.
- A project without shared Git/worktree context uses its current project root as
  the repository sink.
- Subprocess parity is not an SL-231 acceptance criterion; it is explicit follow-up
  work in IMP-319.
