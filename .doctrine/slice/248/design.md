<!-- doctrine:section sec-1 -->
## Governing context

What binds this design, and how each binds. The records hold their own content;
this cites and judges, and restates nothing. Applicability — including the
checks set aside and why — is in `.doctrine/slice/248/research/research.md`,
whose header names the two places it is superseded.

### The target architecture

- `ADR-020` — the execution capsule is the dispatch authority boundary. Items 1
  and 2 of its decision are this slice; the rest is later slices'.
- `SPEC-030` — the container specification, under `SPEC-003`, from `PRD-015`.
  Four requirements are this slice's: `REQ-449` (contract and interpretation
  provenance), `REQ-459` (platform backend contract), `REQ-461` (advisory
  capacity), and `REQ-450` criterion 1 (fresh mutable state). `REQ-448`'s
  *denial* half rides `REQ-459`'s suite. Where this design and `SPEC-030`
  disagree, `SPEC-030` wins.
- `REV-046` — the governance cutover Revision, `proposed` and unapplied
  throughout. This slice retires nothing and applies no Revision.
- `RFC-027` — open, and it proposes reshaping selectors, plans, phase gates and
  criteria. `sec-3` keeps every criteria-bearing field out of the transaction
  for that reason, so this slice's exposure to that reshaping is one stable
  reference field.

### The decisions this design implements

Seven records settle the shape before a line of it was written. They are the
design's premises, not its conclusions, and re-deriving them here would produce
a second copy free to disagree.

| record | what it fixes | where it lands |
|---|---|---|
| `DEC-153` | two binaries, split at canonical mutation authority: `doctrine` unchanged, a new `doctrine-control` workspace member over a lib target on the root package | `sec-6` |
| `DEC-155` | the bubblewrap backend is self-contained — its own flag constants, its own profile builder, no import from `src/worktree/`; and it is *a* backend, not *the* profile | `sec-2` |
| `DEC-156` | the backend contract is property-defined; admission is an executed conformance suite proving denial by one-property-removed controls against decoy targets | `sec-2`, `sec-7` |
| `DEC-157` | a capsule's immutable input is a per-base bare export, bound read-only, cloned from *inside* the sandbox | `sec-3` |
| `DEC-158` | capacity is advisory, probed by `statvfs`, rooted outside the repository, configured in a new `[capsule]` table | `sec-5` |
| `DEC-159` | the phase refinement is consumed by provisioning; the transaction binds phase *identity* and no plan schema | `sec-3`, `sec-4` |
| `DEC-160` | the conformance suite ships in the product behind a `backend verify` verb, with the integration test as a second caller | `sec-6`, `sec-7` |

Three earlier records constrain without being implemented here. `DEC-136` fixes
provenance — the `[interpretation]` block stays in `.doctrine/doctrine.toml`,
resolved once from the contracted base and never re-resolved from a capsule
checkout. **Its handoff item 1 is wrong on the reader**, expecting a direct
extension of the existing loader; `sec-4` states why a typed projection beside
the shared reader is the only available shape, and the correction is owed to the
reconciliation brief rather than to a Revision, because the decision itself
stands. `DEC-134` fixes the headless worker. `DEC-133` and `DEC-137` hold that a
harvested capsule is live work, which is why `sec-5`'s capacity handling may
warn and refuse but never delete, and why `sec-3`'s rollback is scoped to a
directory that holds no work.

### The rules

- `ADR-001` — leaf ← engine ← command, no cycles. `sec-6` classifies the new
  modules and confronts the one place the existing gate cannot see: a
  `doctrine::…` import from a second crate is not a `crate::` edge, so the
  `syn` extractor in `tests/architecture_layering.rs` is blind to it.
- `POL-001` — required, and it reaches this design's vocabulary harder than
  most. Module, type, error and constant names all sit in its scope.
- `POL-002` — platform independence from host-project conventions. Three facets
  reach here: the capsule root default may not be a host convention (`sec-5`);
  the readable-input set may not be the spike's NixOS-shaped literals but a
  declared input with a probe behind it (`sec-2`); and bubblewrap's absence must
  produce a message naming what was missing and what would satisfy it, which is
  what `backend verify` is for (`sec-7`).
- `STD-001` — single-source named constants. Every `[interpretation]` key, the
  schema version, the inner layout paths, the capacity defaults and the twelve
  bubblewrap flag tokens are named once. `DEC-155` records a **departure** for
  the flag tokens specifically: they are not shared with `src/worktree/jail.rs`,
  and the record carries the argument so review meets an adjudication rather
  than re-litigating it.
- `ASM-009` — Git is assumed and structurally privileged. `sec-3`'s export and
  clone-inside are Git objects, and `REQ-449` resolves a blob at a commit OID.
  The export is mechanism-neutral with respect to *confinement* and is not
  neutral with respect to *version control*; `ASM-009` is where that distinction
  is kept.

### What this design is not

Everything downstream of a provisioned capsule: launch, notification, snapshot,
harvest, quarantine ingestion, trusted conformance, verification capsules,
normalization, the admission journal, candidate adaptation, freeze and repair,
retention, and the cutover. The slice's Non-Goals enumerate them against their
requirement ids. Two consequences are visible inside this design rather than
outside it: the backend exposes an execution *primitive* and not a launch verb
(`sec-2`), and the transaction binds only fields that have a consumer in this
slice (`sec-3`).

<!-- doctrine:section sec-2 -->
## The platform backend contract

`REQ-459` asks for one contract that several confinement mechanisms can satisfy,
and an admission gate that a second mechanism passes without editing. `DEC-156`
fixes its form: the contract is stated in observable properties, the mechanism
is behind a trait, and admission is an executed suite. This section defines the
contract and the bubblewrap implementation of it; `sec-7` defines the suite that
admits one.

### Current behaviour, and why none of it is reused

`src/worktree/jail.rs` already builds a bubblewrap invocation. It is not the
starting point. `DEC-155` compares the two profiles axis by axis and finds
nothing structural in common:

| axis | worktree jail | capsule |
|---|---|---|
| filesystem floor | `--ro-bind / /` — everything readable, writes denied | an allowlist — the canonical repository, other capsules and the credential store are *absent* |
| namespaces | none unshared | `--unshare-all` |
| network | open by default, `--unshare-net` on opt-out (`jail.rs:806`) | denied by default, `--share-net` on opt-in |
| paths | the host path reproduced inside | a fixed inner layout |
| environment | inherited | `--clearenv` plus explicit `--setenv` |
| session | absent | `--new-session` |

`bwrap_core_argv` (`jail.rs:537`) is additionally a fixed vector asserted
byte-equivalent to `scripts/pi-spawn-confined.sh` (`jail.rs:1215`, `SL-182`
`VT-7`), so widening it in place fails the test whose purpose is to notice —
risk `R4`. The capsule backend therefore imports nothing from `src/worktree/`,
and `have_bwrap` (`pretooluse.rs:314`, six lines of `PATH` scan) is restated
rather than published, because publishing it would mean publishing a
PreToolUse-hook module and because the capsule's question is not *is bwrap on
`PATH`* but *does this host's backend pass the suite*.

### Target behaviour: the contract

```rust
/// One confinement mechanism, admitted by passing the property suite.
///
/// Every method is total with respect to the host: an unavailable backend
/// reports that it is unavailable rather than failing at execution time.
pub trait CapsuleBackend {
    /// The stable identity recorded in an admission verdict. Never a
    /// display string.
    fn id(&self) -> BackendId;

    /// Whether this host can run this backend, and what is missing when it
    /// cannot (POL-002 facet 3).
    fn availability(&self) -> Availability;

    /// Make `inputs` readable and `writable` writable inside a capsule, run
    /// `execution` there, and return what the trusted parent observed.
    fn execute(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
    ) -> Result<Observation, BackendError>;
}
```

`execute` is the whole capability, and it is deliberately smaller than a launch
verb: no work contract, no notification, no harvest, no result publication.
`DEC-156` requires it because `REQ-459`'s process-teardown and
resource-observation criteria are claims about *this* primitive and have nowhere
else to attach, and because a suite driving `bwrap` directly would certify a
path production never takes.

#### What a backend is told

```rust
/// Where a capsule's state lives on the host and what may be read into it.
/// Computed entirely trusted-side; a backend chooses no path.
pub struct CapsulePlacement {
    /// The per-transaction root. Its children map to the fixed inner layout.
    pub root: TransactionRoot,
    /// The writable areas, each with the inner path it appears at.
    pub writable: Vec<MountedPath>,
    /// The read-only inputs, each with the inner path it appears at. The
    /// per-base export (sec-3) is one of these; the declared readable roots
    /// (below) are the rest.
    pub readable: Vec<MountedPath>,
    /// The working directory inside the capsule. Deterministic by
    /// construction — there is no "inherit" value.
    pub working_directory: InnerPath,
    /// Egress. `Denied` unless the caller says otherwise.
    pub network: NetworkPosture,
}

pub struct MountedPath {
    pub host: PathBuf,
    pub inner: InnerPath,
}

pub enum NetworkPosture { Denied, Permitted }
```

```rust
/// One run inside a capsule.
pub struct Execution {
    /// A typed argument vector. Never a shell string — SPEC-030 § contract
    /// and interpretation provenance says the trusted plan uses typed argv,
    /// and a string would put quoting on a security boundary.
    pub argv: Vec<String>,
    /// The environment, in full. The host environment never crosses.
    pub env: BTreeMap<EnvName, String>,
    /// Wall-clock bound, enforced trusted-side.
    pub timeout: Duration,
    /// Per-file write bound, enforced trusted-side.
    pub file_size_cap: ByteCount,
}
```

#### What a backend reports

```rust
/// What the trusted parent observed. Every field is observed by the parent —
/// nothing here is reported by the capsule (REQ-448 criterion 3: worker
/// output is evidence, never authority).
pub struct Observation {
    pub termination: Termination,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    /// Bytes resident beneath the transaction root after the run.
    pub disk_used: ByteCount,
}

/// How the run ended, as distinguishable states rather than a status code the
/// caller has to classify. The spike proved the classification matters:
/// `sandbox.sh:297,309` maps 127 and 128+SIGXFSZ to distinct tokens, because
/// "the runner refused" and "the runner never ran" otherwise read identically.
pub enum Termination {
    Exited { code: i32 },
    Signalled { signal: i32 },
    TimedOut,
    FileSizeExceeded,
    /// The command could not be executed at all — a missing binary, or a
    /// shebang interpreter outside the readable set.
    NotExecutable,
}

pub enum Availability {
    Available,
    /// POL-002 facet 3: what was missing, and what would satisfy it.
    Unavailable { missing: String, remedy: String },
}
```

`BackendError` is reserved for failures of the *backend*, never for a capsule's
own nonzero exit: a capsule that exits 1 is `Ok(Observation { termination:
Exited { code: 1 }, .. })`. The spike states the same rule as a shell idiom
(`sandbox.sh:284` — "a nonzero status from the capsule is DATA here").

### The properties, stated once

The contract's meaning is the seven properties `sec-7` proves. They are named
here because the trait's shape is answerable to them, and `sec-7` is where each
acquires a probe, a control and a decoy.

| # | property | what the contract owes it |
|---|---|---|
| 1 | fresh mutable state | `CapsulePlacement.root`, distinct per transaction |
| 2 | explicit input set | `readable` is a declared list; there is no implicit floor |
| 3 | denial of canonical state and credentials | absence from `readable`/`writable`, not read-only presence |
| 4 | bounded host filesystem visibility | the same absence, generalised to everything undeclared |
| 5 | explicit network posture | `NetworkPosture`, `Denied` by default |
| 6 | deterministic working directory | `working_directory`, with no inherit value |
| 7 | process-tree teardown and trusted observation | `timeout`, `file_size_cap`, and every `Termination` variant |

`SPEC-030` § Platform backend contract lists eight clauses; rows 6 and 7 above
absorb *deterministic working directory*, *process-tree teardown*, and *trusted
observation of resource limits and termination* into two rows, because teardown
and observation are one question — what the trusted parent can establish about a
process it did not trust — and splitting them would give the suite two rows with
one control between them.

### The bubblewrap backend

`crates/doctrine-control/src/backend/bubblewrap.rs`, self-contained per
`DEC-155`. Naming is plural-ready throughout: this is `BubblewrapBackend` under
`CapsuleBackend`, never "the confinement profile".

#### The inner layout, named once (STD-001)

```rust
const INNER_CAPSULE: &str = "/capsule";
const INNER_SOURCE: &str = "/source";
const INNER_HOME: &str = "/agent";
const INNER_TMP: &str = "/tmp";
```

Fixed inner paths rather than the host's. `--bind "$D" "$D"` — the worktree
arm's form — would reproduce the capsule's host path inside, which reads the
parent directory of every other capsule into existence as a mountpoint chain
(`sandbox.sh:58-66`).

#### The profile

The twelve flag tokens are named once in this module and nowhere else. Assembly,
in order:

```
--unshare-all                      # namespaces; network included
--proc /proc  --dev /dev           # the two pseudo-filesystems a process needs
--tmpfs /tmp                       # writable temporary area, dies with the capsule
--ro-bind <host> <inner>           # once per readable input, in declared order
--bind <host> <inner>              # once per writable area, in declared order
--chdir <working_directory>
--die-with-parent
--new-session
--clearenv
--setenv <name> <value>            # once per env entry, in sorted order
--share-net                        # ONLY when NetworkPosture::Permitted
```

Two guards, both fail-closed, both taken from the spike's own findings:

- an empty mount vector refuses rather than executing unconfined
  (`sandbox.sh:276`, itself the seed's `pi-spawn-confined.sh:115` guard);
- an empty `argv` refuses.

`--share-net` last and conditional is the inversion `DEC-155` names and the
harvest note at `jail.rs:806` warns about: the worktree arm is default-open, and
a capsule must be default-denied. `sec-7`'s property 5 asserts the default
**positively** — a probe that reaches a trusted-side loopback listener only when
the posture is `Permitted` — because a negative-only assertion passes on a host
with no network at all.

#### The readable roots are a declared input, not a literal

The spike filters `PATH` to entries under `/nix/store` and binds that one root
(`sandbox.sh:141-153`). That is a host convention, and `POL-002` facet 1 forbids
the product resting on one. The product form:

- `[capsule] readable_roots` is a required, non-empty list of host paths. Each
  entry may be a directory or a single file — the file case is what covers
  shebang interpreters, which the kernel resolves before `PATH` exists and which
  cost the spike two separate findings (`sandbox.sh:171-188`, `/usr/bin/env` and
  `/bin/sh`). Binding the interpreter file rather than `/bin` is portable by
  construction; binding the directory is a posture that silently widens on a
  host where `/bin` holds hundreds of binaries.
- Each entry is **probed at provisioning**. An absent entry refuses, naming the
  entry and the config key — `POL-002` facet 3, and the same shape `jail.rs:302-393`
  already uses for jail policy.
- The capsule's inner `PATH` is *derived*: the host `PATH` entries that lie
  beneath a declared readable root, in host order. Nothing else. This keeps the
  spike's behaviour — the capsule gets the declared toolchain and not the
  control plane's own binaries — while making the root a declared input rather
  than a NixOS-shaped literal.

Only `readable_roots` and the per-base export (`sec-3`) ever become readable
inputs. There is no host-shaped default and no fallback: `Doctrine supplies no
ecosystem default` is `SPEC-030`'s rule for `[interpretation]`, and the same
discipline applies here for the same reason.

#### Bounds are enforced outside the sandbox

`timeout -k <grace> <secs>` wraps the `bwrap` exec from outside, and the
per-file cap is set on the child before it (`setrlimit(RLIMIT_FSIZE)`, the typed
equivalent of the spike's `ulimit -f`). Neither is reachable from inside the
namespace, which is what makes property 7 an observation rather than a report.
The whole-tree disk figure is computed trusted-side after the run, because a
per-file limit does not catch a capsule that writes many small files
(`sandbox.sh:311-317`).

Rust owns the process orchestration rather than generating shell. `DEC-160`
records the trade: process orchestration is the one act materially more verbose
in Rust than in bash, and generated shell trusted-side would reintroduce quoting
hazards on a security boundary, where the typed refusals and the canonical hash
live.

### Invariants

1. **A backend chooses no path.** Every host path in `CapsulePlacement` is
   computed by `sec-3`'s provisioning. A backend that resolved its own paths
   could satisfy the suite and still place a capsule somewhere the suite never
   looked.
2. **Absence, not read-only presence, is how denial is achieved.** The canonical
   repository, other capsules, and the credential store are never in `readable`.
   `DEC-157` makes this structural for the repository by provisioning from an
   export instead.
3. **Every termination fact is the parent's observation.** No `Termination`
   variant is derivable from capsule-written state.
4. **The default is refusal.** Empty argv, empty mounts, absent readable root,
   unresolvable capsule root: each refuses. There is no arrangement of missing
   configuration that produces an unconfined execution.
5. **`src/worktree/` is not imported.** Enforced by `sec-6`'s export set, which
   carries no bubblewrap surface at all.

### Verification alignment

Shape assertions are necessary and never sufficient (`DEC-156`; risk `R3`), so
this section owes both kinds. Pure, hermetic:

- `bubblewrap_argv_denies_network_unless_posture_is_permitted`
- `bubblewrap_argv_places_share_net_only_after_unshare_all`
- `empty_mount_vector_refuses_rather_than_running_unconfined`
- `empty_argv_refuses`
- `inner_path_is_derived_only_from_declared_readable_roots`
- `absent_readable_root_refuses_naming_the_entry_and_the_config_key`
- `working_directory_has_no_inherit_value` (a type-level property, asserted by
  construction rather than by test)

Executed, and the reason the shape assertions are not enough: all of `sec-7`.

<!-- doctrine:section sec-3 -->
## The capsule transaction and how it is provisioned

`SPEC-030` § Transaction authority says the control plane creates a transaction
*from* a base, a resolved policy, a work contract, a capsule identity and
resource choices. This slice builds that creation and nothing after it. The
transaction is the value provisioning returns; `sec-4` supplies its policy
fields, `sec-2` consumes its placement, `sec-5` gates it on capacity.

### What the transaction binds

`DEC-159` fixes the field set to what has a consumer *now*. Later slices add
fields; they do not rework these.

```rust
/// One capsule transaction, as provisioning returns it.
pub struct CapsuleTransaction {
    /// Allocated trusted-side. Never chosen by a capsule — which is why this
    /// does not touch the parked QUE-208 (capsule-side entity id allocation).
    pub id: TransactionId,
    /// A durable reference to the phase this serves, and nothing more.
    pub phase: PhaseIdentity,
    /// The accepted commit this capsule is provisioned from.
    pub base: CommitId,
    /// The canonical hash of the base policy, as resolved from `base`.
    pub base_policy: PolicyHash,
    /// The policy actually in force: the base policy, or the result of
    /// applying a phase refinement to it (sec-4).
    pub policy: InterpretationPolicy,
    /// The canonical hash of `policy`. Equal to `base_policy` when no
    /// refinement was applied — recorded rather than derived, so the
    /// admission journal a later slice writes has both without re-hashing.
    pub policy_hash: PolicyHash,
    /// Where the capsule lives, and what is readable inside it (sec-2).
    pub placement: CapsulePlacement,
    /// The resource choices read from `[capsule]` (sec-5).
    pub bounds: ResourceBounds,
}

/// A phase, by durable identity only.
pub struct PhaseIdentity {
    pub slice: SliceId,
    /// The immutable `PHASE-NN`.
    pub phase: PhaseNumber,
}
```

**Phase identity is bound; plan schema is not.** `RFC-027` is open and proposes
reshaping selectors, plans, phase gates and criteria. Provisioning reads neither
`plan.toml` nor a phase sheet: the refinement arrives as its own document in the
`[interpretation]` schema (`sec-4`). Everything criteria-bearing — where
`RFC-027`'s churn lands — therefore stays with the later launch slice by
construction rather than by intention, and this slice's exposure to that
reshaping is the two fields of `PhaseIdentity`.

### The immutable input: a per-base bare export

`DEC-157` settles the input form. A capsule is provisioned from a **per-base
bare export** — a repository holding exactly the contracted commit's history,
built trusted-side once per base, shared read-only by every capsule on that
base, and cloned from *inside* the sandbox.

**Why not the canonical repository.** Binding it is what the spike's fixture
does and costs nothing trusted-side, but it places the canonical object store
inside the mount set. `REQ-448` criterion 1's denial of shared object storage
would then rest on a read-only mount rather than on absence, and the capsule
could read every ref and branch rather than only its contracted history. With an
export, the canonical repository is outside the mount set under *every* arm of
`sec-7`'s suite — including the controls — which is the structural form of the
same denial. `REQ-450`'s "explicit immutable inputs" also becomes a physical
hashable object rather than a description. A bundle file is the same idea
carrying a format question and no advantage.

**Why cloned inside.** No working tree is materialised trusted-side. The
mechanism is `SL-241`'s, proven at its `EX-8`.

**`--no-hardlinks` is a mitigation, not tuning.** A local clone hardlinks object
files by default, so a hostile capsule corrupting a shared object would corrupt
its source (`provision.sh:56-61`). The read-only binding makes the write fail
rather than corrupt; the flag is what survives someone making the export
writable.

**The export is mechanism-neutral; the read-only binding is not.** The export is
a plain Git artefact with no confinement mechanism in it. *How it becomes
readable inside the capsule at a known path, and nothing else of the host with
it,* is a capability of the backend contract (`sec-2`): bubblewrap realises it
as a read-only bind, a container backend as a volume, a virtual machine as a
share. A future backend inherits the export and the clone-inside posture
unchanged.

Building it rides `git init --bare` plus the existing `fetch_refspec`
(`src/git.rs:2718`). There is no clone or bundle helper in `src/git.rs` to
reuse — verified — so this half is net-new whatever input form is chosen.

### Layout

```
<capsule_root>/                       # from [capsule] root, outside the repo (sec-5)
  export/<base-oid>/                  # bare, immutable once built, shared read-only
  tx/<transaction-id>/                # the per-transaction root
    capsule/                          # rw → /capsule
      repo/                           #   the clone, created INSIDE the sandbox
      out/                            #   the output area
      tmp/                            #   the writable temporary area
    agent/                            # rw → /agent, the agent home
```

Distinct host directories mapping to `sec-2`'s fixed inner layout. The
transaction root is on real disk, not tmpfs: `sec-5`'s free-space probe must
read real available space, and property 7's resource observation on a tmpfs
would measure the mount's size rather than the disk's (`DEC-156`).

The agent home is a sibling directory rather than the spike's tmpfs
(`sandbox.sh:248`) for the same two reasons — it is capsule state, and capsule
state is what the disk figure is about.

### Provisioning, step by step

```rust
pub fn provision(
    request: &ProvisionRequest,
    host: &dyn HostFacts,       // clock, statvfs, path existence — the impure inputs
    backend: &dyn CapsuleBackend,
) -> Result<CapsuleTransaction, ProvisionRefusal>;

pub struct ProvisionRequest {
    pub repository_root: PathBuf,
    pub base: CommitId,
    pub phase: PhaseIdentity,
    /// An optional phase-refinement document, by path (sec-4).
    pub refinement: Option<PathBuf>,
    pub network: NetworkPosture,
}
```

1. **Read `[capsule]`** from `.doctrine/doctrine.toml` — capsule root, resource
   bounds, readable roots (`sec-5`). Unresolvable root refuses, naming the key.
2. **Probe each declared readable root.** Absent entry refuses, naming the entry
   and the key.
3. **Probe capacity** by `statvfs` on the capsule root. Below one expected size,
   refuse; between one and two, warn and continue; unknown, report and continue
   (`sec-5`).
4. **Resolve the base policy** from the blob at `base` (`sec-4`). This is the
   *only* read of `.doctrine/doctrine.toml` from the contracted base, and it is
   `DEC-136`'s read-once invariant made physical.
5. **Apply the refinement**, when one was supplied — the pure monotonic
   restriction algebra of `sec-4`. Any widening refuses.
6. **Ensure the per-base export.** Idempotent: an existing
   `export/<base-oid>/` that already holds `base` is reused. Building it is
   `git init --bare` then a fetch of `base` from the repository root.
7. **Create the transaction root** and its four children.
8. **Assemble the placement** — export read-only at `/source`, `capsule/` and
   `agent/` writable, declared readable roots read-only at their host paths,
   working directory `/capsule`, network posture as requested.
9. **Clone inside.** One `backend.execute` running
   `git clone --no-hardlinks -c user.name=… -c user.email=… -- /source /capsule/repo`
   followed by `git switch --detach <base>` and `git remote remove origin`.
10. **Return the transaction.**

Steps 1–5 are pure given `HostFacts`; steps 6–9 are the impure part, in the thin
outer part of the module per the project's pure/imperative split.

**Capsule identity on the clone, pinned by `-c`.** `git clone -c` takes effect
after init and before the fetch, so it covers the clone's own reflog writes.
Configured only afterwards, git guesses an identity, which means resolving the
hostname — and inside `--unshare-all` that is a DNS query for an unshared UTS
name that blocks until the resolver gives up. The spike measured ~3.9s per
ident-needing operation against 40ms with the ident pinned (`provision.sh:41-52`).
The identity is *asserted* after the clone, not assumed, because a git that
stopped persisting `-c` would restore the stall silently.

**No remotes.** The capsule has nowhere to push. Harvest, when a later slice
builds it, is a control-plane pull from the capsule, never a capsule-initiated
write outward.

### Rollback, which is not eviction

A provisioning call that fails after step 7 removes the transaction root it
created, and only that path. `REQ-461` criterion 3 forbids automated deletion of
a capsule or result, and the slice's Non-Goals forbid eviction; neither reaches
a directory this call created before any transaction exists in it. It holds no
work, and leaving it accumulates partial trees on a disk already short of space.

Rollback is a private step of `provision`, not a product-side delete capability.
`DEC-133` and `DEC-137` hold that a harvested capsule is live work, and
`DEC-156` names the hazard precisely: a delete primitive introduced here for
tidiness is what a later slice would reach for.

### Invariants

1. **The policy is resolved once, from the contracted base.** Nothing in this
   slice re-reads `.doctrine/doctrine.toml` from a capsule checkout, and nothing
   in a capsule can reach the resolution (`DEC-136`, `REQ-449` criterion 3).
2. **The export is immutable once built.** It is created, fetched into, and
   thereafter only read. Concurrent provisioning on the same base builds it
   under a temporary name and renames into place, so a partially-fetched export
   is never observable at `export/<base-oid>/`.
3. **Two transactions share no mutable state.** Distinct `tx/<id>/` roots,
   distinct clones, distinct processes, distinct temporary areas. The export is
   shared and read-only — not shared *mutable* state, so `REQ-450` criterion 1
   is untouched.
4. **The transaction id is allocated trusted-side.**
5. **A refused provision leaves no transaction.** Either a `CapsuleTransaction`
   is returned or the path this call created is gone.

### Verification alignment

`REQ-450` criterion 1 is discharged by asserting distinctness on each of the
five axes the criterion names — checkout, repository, runtime, process,
temporary state — **by writing through one transaction and observing absence in
the other**, never by comparing paths. Under `DEC-156`'s discipline the paired
control provisions the second transaction into the first's root and observes
that the writes *are* visible, which is what makes the green arm carry
information.

Test titles:

- `two_transactions_on_one_base_share_no_writable_checkout`
- `two_transactions_on_one_base_share_no_repository_objects`
- `two_transactions_on_one_base_share_no_temporary_state`
- `control_second_transaction_in_the_first_root_does_share_state`
- `export_is_reused_across_transactions_on_the_same_base`
- `export_holds_the_contracted_history_and_no_other_ref`
- `clone_inside_leaves_no_working_tree_trusted_side`
- `capsule_identity_persists_into_the_clone_config`
- `failed_provision_removes_only_the_root_it_created`
- `failed_provision_leaves_an_existing_export_intact`

`REQ-450` criteria 2 and 3 need candidate identity and harvest from later
slices. This slice records a contributing `--change` against criterion 1 and
reports the requirement as **partial** in the reconciliation brief, not quietly
claimed. `REQ-448`'s denial half is the same shape and is proven by `sec-7`.

