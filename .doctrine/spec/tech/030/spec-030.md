# SPEC-030: Dispatch execution capsules

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See glossary.md § reference forms. -->

## Overview

Dispatch execution capsules are the forward-intent isolation container beneath
the whole-system root (SPEC-003), descending from the concurrent isolated-dispatch
product intent in PRD-015. The container makes a **fresh execution transaction**,
not a Git worktree or harness identity signal, the dispatch authority boundary.

A persistent trusted control-plane orchestrator retains interactive context across
phases. Each phase runs as a headless worker in newly provisioned mutable capsule
state; verification runs in another fresh capsule. The worker may use ordinary
tools, modify local Doctrine files, and make local commits, but it cannot mutate
canonical Git state or control-plane storage. Only the control plane interprets,
normalizes, journals, and admits the result.

This specification is **forward-intent**. ADR-020 selects the target architecture;
RFC-025 and SL-241 supply bounded Linux/bubblewrap and result-ingestion evidence.
ADR-011, SPEC-012, SPEC-021, and SPEC-022 still describe shipped worktree dispatch
until REV-046's implementation cutover. At cutover SPEC-021 becomes this container's
process component, SPEC-012 narrows to surviving solo-worktree mechanism, and
SPEC-022 remains the sibling Git/admission substrate.

The container deliberately does not select retention durations, quota hierarchy,
macOS confinement, egress policy, non-Git build-input provisioning, cache strategy,
or remote execution. Those remain additive policies or separately evidenced
mechanisms; none may weaken the authority boundary defined here.

## Responsibilities

The structured responsibilities are authoritative. They divide into four owned
contracts: transaction authority, trusted ingestion and admission, recovery and
lifecycle, and backend equivalence.

### Transaction authority and lifecycle

The control plane creates a transaction from an immutable accepted base, a project
interpretation policy resolved from that base, a phase work contract, a capsule
identity, and explicit resource/backend choices. The ordinary lifecycle is:

`resolve → provision → launch → notify → snapshot → harvest/freeze → conform →
verify → normalize → journal → admit/integrate → close → explicit cleanup`.

Every transition has one trusted writer and a durable state. Notification is a
payload-minimal doorbell, not a verdict or identity source. A capsule process exit
can end local execution but cannot authorize the next trusted transition. Formal
execution never resumes in a harvested capsule.

Provisioning starts from the exact accepted commit and only explicit immutable
inputs. Mutable phase state is fresh. Immutable toolchain layers and caches may be
reused if their identity is recorded and they cannot carry mutable transaction
state or grant canonical authority.

### Contract and interpretation provenance

Capsule dispatch requires this v1 schema in `.doctrine/doctrine.toml`:

```toml
[interpretation]
schema = 1
trusted_side_forbidden_executables = ["cargo", "nix", "direnv", "just", "rustc"]
interpreted_paths = ["build.rs", "**/build.rs", "flake.nix", ".envrc", "rust-toolchain.toml"]

[[interpretation.verification]]
argv = ["just", "web-build"]

[[interpretation.verification]]
argv = ["cargo", "test"]
```

The three fields are explicit and have no manufactured defaults:

- `trusted_side_forbidden_executables` is a duplicate-free list of normalized
  executable basenames. Entries contain no slash, whitespace, empty value, `.` or
  `..`. The trusted transaction plan uses typed argument vectors, not a general
  shell, and refuses any external-command step whose normalized executable matches
  the list. Doctrine-owned Git/object operations are separately constrained by
  the ingestion contract; wrappers and interpreters the project uses must be named.
- `interpreted_paths` is a duplicate-free list of normalized repository-relative
  gitignore-style patterns. Absolute paths, backslashes, NUL, and lexical `..`
  components are refused. This field instantiates ecosystem-specific hostile probes
  and documents build-system/toolchain auto-load surfaces; trusted-side safety does
  not depend on matching it because harvested trees are never materialized there.
- Each `interpretation.verification` row contains one non-empty `argv` array of
  non-empty UTF-8 strings. Rows run sequentially, without shell expansion, from the
  verification capsule's repository root; all must succeed. Complex verification
  lives behind an explicitly named executable/script and remains confined to the
  verification capsule.

The block, all three keys, and at least one verification row are required. The two
lists may be explicitly empty when the project genuinely has no such instances;
omission is not equivalent to emptiness. Unknown keys or schema versions refuse
capsule provisioning. Normalization preserves verification-row and argument order,
sorts the two set-valued lists by raw UTF-8 bytes after duplicate detection, and
hashes the canonical typed value for the work contract and admission journal.

The control plane reads this value from the contracted base once, binds the
normalized value into the work contract and admission journal, and never re-resolves
it from the capsule checkout or harvested result. Doctrine supplies no ecosystem
default when the block is absent.

A phase contract may reduce allowed paths or execution, add forbidden executable or
interpreted-path entries, and append verification rows after the project-declared
sequence. It may not remove or reorder project verification, remove a forbidden
entry, widen another permission, replace the declaration, or make its absence
acceptable. Subset validation compares normalized typed values, never source text.

### Result publication, ingestion, and conformance

V0 result transport is a Git bundle at a control-plane-selected location. The
worker completes publication before sending the result-ready notification. The
control plane treats the path and bytes as hostile: it rejects symlinks and unsafe
path resolution, enforces explicit byte/time/object bounds, requires quiescence,
and makes one immutable snapshot into parent-owned storage. Later capsule writes
cannot change what trusted Git reads.

A new disposable quarantine repository verifies and imports only the snapshot,
performs object-integrity checks, resolves exactly one result identity, and rejects
unexpected refs or object shapes. Trusted code never invokes Git with the
capsule-authored repository as repository or working context. Fetching from that
repository is not a fallback.

Against the pinned result, the trusted conformance layer checks ancestry from the
contracted base, allowed merge shape, actual changed paths, declared phase scope,
forbidden paths, file modes, submodule policy, and interpretation-policy obligations.
These are computed from objects, not accepted from capsule prose.

### Separate verification and normalization

The control plane constructs the exact candidate that could be admitted, including
any configured history normalization. It provisions a fresh verification capsule
from that immutable candidate and the same resolved policy, runs the contracted
verification, and treats the verifier process result plus trusted evidence capture
as the verdict. Worker-authored logs remain evidence, not authority.

Flattened and full-history normalization are policy choices. Whichever is selected,
the admitted commit identity must be the exact identity verified. Normalization
runs on the trusted side without executing candidate code.

### Admission, staleness, and repair

Before any canonical mutation, the admission journal durably records the transaction,
base, policy identity, source snapshot, pinned source, normalization, verifier
attestation, candidate identity, expected old accepted tip, and intended transition.
The control plane checks the accepted ref before transferring objects into the
canonical repository, then performs one expected-old-object compare-and-swap. A stale
precheck transfers nothing; a lost final race may leave unreachable objects but never
advances the wrong ref. Re-entry classifies already-applied, still-applicable, and
diverged state without force or automatic resolution.

A stale pinned result enters the existing candidate engine through explicit capsule
provenance: current accepted commit, pinned source commit, contracted base, and
verification attestation. An object-only three-way classification either creates a
candidate or records a durable conflict. Clean, hand-resolved, or fix-on-top candidates
are verified in a fresh capsule before immutable-commit admission. A second accepted-
ref movement creates an explicitly superseding candidate against the new base.

The source capsule stays frozen. Repair is a new transaction in a new capsule based
on the current accepted commit, with the frozen result supplied as input. A similar
tree is not enough to prove incorporation: cleanup requires a mechanically recorded
lineage or an explicit operator disposition.

### Retention, cleanup, and capacity

The compact trusted admission journal is durable. Before successful integration and
formal closure, the frozen source capsule and its result are live work: v0 performs
no automatic deletion, eviction, or garbage collection, including after a
superseding attempt. Explicit operator abandonment is the only early cleanup path.

After integration and closure of the result or an incorporating repair, the capsule,
bundle, logs, and permitted transcript material become forensic exhibits. They may
expire under a separately-owned archive policy without rewriting the admission
journal, which records their lifecycle honestly.

V0 checks available capacity at provisioning and emits a conspicuous warning against
a configurable expected capsule size; an initial implementation may warn below twice
that expectation. It does not reserve space, throttle throughput, or destroy work to
free capacity. Exhaustion stops progress for manual intervention.

### Platform backend contract

Every backend must enforce equivalent observable properties: fresh mutable state;
an explicit base and input set; no writable canonical repo, shared object store,
control-plane state, or credentials; bounded host filesystem visibility; explicit
network/egress posture; deterministic working directory; process-tree teardown; and
trusted observation of resource limits and termination.

Linux/bubblewrap is the measured v0 backend. A macOS backend remains unselected until
its mechanism is independently specified and measured. Harness launch is uniformly
headless subprocess execution in the target architecture; harness-specific in-session
subagent identity is not part of the capsule contract.

## Concerns

- **Security posture is structural.** Configuration, retention, and monitoring may
  evolve, but no convenience path may restore capsule access to canonical Git or
  make trusted code execute Git in a capsule-authored repository.
- **Correctness precedes throughput.** V0 accepts cold provisioning, serial trusted
  admission, and manual disk intervention. Reservation, backpressure, and eviction
  are deferred because they are additive and could destroy work if guessed early.
- **Forensics are proportional.** Durable journal truth is small and permanent;
  rich post-close exhibits are valuable over a usually short horizon and need not
  live in authored Git.
- **Crash recovery is transaction recovery.** A durable journal and immutable object
  identities reconstruct progress; interactive agent context and capsule mutability
  are not recovery roots.
- **Disk loss is not solved here.** Backup or replication may protect capsule storage,
  but distributed durability is outside the v0 isolation mechanism.
- **Evidence has a declared altitude.** Linux confinement and one real-agent run are
  evidence for feasibility, not cross-platform parity, performance, or production
  readiness.

## Hypotheses

- Fresh mutable state per phase removes more identity and state-carry-over complexity
  than it adds in provisioning cost.
- One hostile-file ingestion boundary is more enumerable and testable than allowing
  trusted Git to interpret an untrusted repository configuration surface.
- The incumbent candidate engine's pure object semantics can be separated from its
  coordination-journal adapter without Git archaeology or a second admission model.
- Retaining frozen unresolved capsules is operationally cheaper and safer for v0 than
  building a rescue archive or automated eviction policy before usage evidence exists.

## Decisions

- **D1 — Fresh transaction, persistent control plane.** Interactive orchestration
  persists outside capsules; every phase and verification run receives fresh mutable
  capsule state.
- **D2 — Bundle snapshot boundary.** V0 ingests one bounded parent-owned bundle
  snapshot through a fresh quarantine repository; no fetch-from-capsule fallback.
- **D3 — Policy from accepted base.** The required project interpretation declaration
  lives in `.doctrine/doctrine.toml`, is resolved once from the accepted base, and can
  only be restricted by a phase contract.
- **D4 — Verify what can land.** The exact normalized candidate is verified in a
  separate capsule before journaled admission by immutable identity.
- **D5 — Candidate semantics, capsule provenance.** Stale/conflicted work reuses the
  candidate engine through explicit capsule inputs, not the incumbent coordination
  journal.
- **D6 — Frozen source, fresh repair.** Harvested source capsules never resume formal
  execution; unresolved work remains frozen and non-evictable, while repairs are new
  transactions.
- **D7 — Advisory capacity in v0.** Warn on low space and stop for manual intervention;
  defer reservation, backpressure, eviction, and a rescue archive.
- **D8 — Property-equivalent backends.** Linux/bubblewrap is measured; every other
  backend must independently prove equivalent authority and confinement properties.

## References

- ADR-020 — governing capsule authority and lifecycle decision.
- PRD-015 — isolated concurrent dispatch product intent; revised at cutover by
  REV-046.
- RFC-025 and SL-241 — mechanism census, Linux/bubblewrap spike, hostile-ingestion
  probes, and their explicitly bounded evidence claims.
- DEC-099 and DEC-136 — interpretation-surface ownership, schema home, immutable-base
  provenance, and monotonic phase refinement.
- DEC-133 through DEC-135 and DEC-137 — journal/exhibit separation, persistent
  orchestration, bundle ingestion, stale-result recovery, and frozen-source repair.
- SPEC-021 — incumbent orchestrator process and future component of this container.
- SPEC-022 — sibling Git object/ref, candidate, journal, and compare-and-swap
  substrate reused through a capsule-provenance adapter.
- ADR-006, ADR-008, ADR-011, ADR-012, and SPEC-012 — incumbent dispatch authority
  retained until REV-046's implementation cutover.
- IMP-397 and QUE-204 — separately-owned egress and non-Git build-input work.
