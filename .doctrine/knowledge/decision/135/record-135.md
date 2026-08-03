# DEC-135: Bundle ingestion removes trusted Git execution from capsule repositories

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Context

`QUE-200` asks for the minimal safe parent-side mechanism for ingesting a phase
result from a potentially hostile capsule Git repository. `SL-241` compared
fetch-from-capsule and bundle ingestion side by side. The measured downstream
behaviour was almost identical, but `DEC-128` established that the remaining
difference cannot be settled by more sampling: fetch makes the trusted parent
run Git inside a capsule-authored repository, and safety over Git's entire
repository-configuration surface is an unprovable universal.

Bundle ingestion carries its own trusted-side boundary. The parent parses
capsule-authored bytes, and those bytes may be absent, malformed, oversized or
changing during handoff. Unlike the hostile repository-configuration surface,
that boundary is finite enough to enumerate, bound and test. The bundle also
preserves the worker's Git history as a forensic exhibit, unlike final-tree
materialisation.

## Decision

The v0 capsule result transport is a Git bundle. The governing structural
invariant is:

> Trusted control-plane code never runs Git with a capsule-authored repository
> as its repository or working context.

The worker publishes the bundle at a fixed, control-plane-selected location
before ringing the result-ready doorbell. The parent treats the file as hostile
bytes, snapshots it once into parent-owned storage under no-symlink and resource
bounds, and permits Git to read only that snapshot from a fresh disposable
quarantine repository. A bundle is not intrinsically trusted or "safe"; this
choice replaces an open-ended execution surface with a bounded file-ingestion
surface.

Fetch from a capsule repository is not a v0 fallback. A future transport may
replace the bundle only if it preserves the structural invariant rather than
claiming that a finite sample cleared hostile Git configuration. Final-tree
materialisation remains excluded because it discards the worker history needed
for proportional short-horizon forensics.

## Implementation handoff

`SL-241` already demonstrated the enclosing four-stage pipeline. Product work
should implement the following protocol rather than commission another
feasibility probe:

1. **Publish and snapshot.** The worker creates the bundle and signals once it
   is complete. The parent refuses an unsafe path, missing artifact or resource
   limit breach, and copies one bounded snapshot into parent-owned storage so
   later capsule writes cannot change the bytes Git reads.
2. **Harvest.** A fresh disposable quarantine repository verifies the bundle,
   imports it, runs object-integrity checks and pins exactly one result object
   identity.
3. **Conform.** Against the pinned identity, trusted code checks ancestry from
   the contracted base, merge policy, actual changed paths, declared scope,
   forbidden paths and tree modes. The existing strict slice-conformance seam
   is reused.
4. **Verify.** A separate verification capsule runs the declared verification
   against exactly the pinned candidate. Its process exit status is the verdict;
   capsule-authored prose is not.
5. **Advance.** Trusted code first checks that the accepted ref is still at the
   contracted base. Only then does it transfer the pinned objects into the
   canonical repository and perform one expected-old-object compare-and-swap.
   A stale base transfers nothing; losing the final race leaves only
   unreachable objects for garbage collection.
6. **Record and dispose.** A durable admission journal records the contracted,
   pinned, verified and admitted identities and verdicts. The bundle and richer
   forensic material follow `DEC-133`'s separately-owned, potentially expiring
   exhibit lifecycle; quarantine is discarded.

The existing Git ancestry, object-only merge, ref compare-and-swap, strict
conformance, candidate identity and admission machinery are implementation
seams to reuse rather than rederive. `QUE-202` still owns the necessary
decoupling of conflict/staleness resolution from the incumbent dispatch
coordination journal. That question blocks the complete admission and cutover
design, but it does not block implementing or specifying bundle ingestion.

The hostile rows and stage assertions from `SL-241` become production
acceptance tests. Its disposable shell rig is evidence and a behavioural
oracle, not code to migrate directly.

## Consequences

- `QUE-200` is answered without pretending the spike proved a universal over
  Git configuration.
- The result boundary has a concrete producer/consumer protocol and therefore
  a direct path to an implementation slice after `REV-046` defines the target
  architecture.
- Bundle hygiene, quarantine lifecycle, time/resource bounds and refusal tokens
  are product obligations, not incidental hardening.
- `QUE-202` remains the largest target-design gap: safe refusal is proven, but
  admitting or superseding a second result from one base still needs an owning
  mechanism.
- `IMP-397` remains a separate provisioning and egress track; it does not reopen
  this transport choice.

## Origin

Answers `QUE-200` for `RFC-025` and is a decision input to the post-spike
governance revision `REV-046`. It consumes `DEC-128`'s ruling that the residual
is architectural, not evidential, and `DEC-133`'s journal/exhibit lifecycle
separation.
