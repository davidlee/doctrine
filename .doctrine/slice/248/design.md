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
///
/// **Every field is private and the only constructor validates.** A public
/// struct literal would let a caller assemble a placement that satisfies every
/// property the suite asserts and still reaches host state the suite never
/// looked at — the mount set is the confinement, so an unvalidated one is an
/// unconfined capsule with a confined shape.
pub struct CapsulePlacement {
    root: TransactionRoot,
    /// The per-base export, as its own field rather than an element of a
    /// vector, bound to the identity it is an export of (sec-3).
    source: SourceExport,
    writable: Vec<MountedPath>,
    readable: Vec<MountedPath>,
    working_directory: InnerPath,
    network: NetworkPosture,
}

impl CapsulePlacement {
    /// The one constructor. `scopes` names the host regions this placement may
    /// never reach; it is supplied by the caller because a confinement type
    /// cannot know where a particular host keeps its canonical repository.
    pub fn try_new(
        parts: PlacementParts,
        scopes: &ForbiddenScopes,
    ) -> Result<CapsulePlacement, PlacementRefusal>;
}

/// The host regions a capsule may never reach, named by the trusted side.
pub struct ForbiddenScopes {
    /// The canonical repository and its object store.
    pub canonical_repository: PathBuf,
    /// Control-plane state — the whole `.doctrine/` tree at `root`.
    pub control_plane_state: PathBuf,
    /// The capsule root, so no capsule can reach a sibling capsule.
    pub capsule_root: PathBuf,
    /// Credential locations: the git config, the ssh directory, the harness
    /// credential store, and anything else the operator names.
    pub credentials: Vec<PathBuf>,
}

pub struct MountedPath {
    /// **Fully resolved.** See "resolution precedes validation" below.
    host: PathBuf,
    inner: InnerPath,
}

pub enum NetworkPosture { Denied, Permitted }
```

`try_new` refuses, each with a named variant:

- an inner destination in the **reserved set** — `/proc`, `/dev`, `/tmp`,
  `/source`, `/capsule`, `/agent` — supplied as an ordinary readable or writable
  entry, so nothing can shadow a mount the profile owns or substitute the
  contracted export;
- two entries with the same inner path, or one inner path an ancestor of
  another, so mount order can never decide what is visible;
- a resolved host path that **overlaps a `ForbiddenScopes` member in either
  direction** — equal to it, an ancestor of it, *or a descendant of it* — with
  the single carve-out below;
- a resolved host path that is the filesystem root.

**Overlap is bidirectional, and the descendant half is the half that matters.**
An earlier draft refused only equality and ancestry, on the reasoning that
provisioning computes every path in a placement and therefore computes only its
own. `RV-346` `F-10` refuted it: `readable-roots` entries and closure-resolver
output are *also* provisioning-computed in that sense, and they can name
anything. `repository/.git/config`, `credentials/token` and
`capsule-root/tx/<another-transaction>` are all descendants of a forbidden scope,
all pass an equal-or-ancestor test, and each is exactly what the corresponding
scope exists to deny. Provenance cannot carry the argument, so the geometry does.

```rust
/// True when either path lies on the other's root-ward chain. Both arguments
/// are already resolved, so this is a component comparison and not a textual
/// prefix test — `/var/lib/doctrine-other` is not under `/var/lib/doctrine`.
fn overlaps(a: &Path, b: &Path) -> bool { a == b || a.starts_with(b) || b.starts_with(a) }
```

**The carve-out is the transaction's own writable state, and it is typed rather
than described.** `TransactionRoot` is minted by `sec-3` step 9, by the call that
exclusively created that directory. `try_new` accepts a writable entry that is a
descendant of *this placement's* `root` and refuses every other overlap with
`capsule_root`. So the capsule area is closed to a placement except through the
one transaction root the placement carries, and a sibling transaction — which is
a descendant of `capsule_root` but not of this `root` — is refused by the same
test that admits `capsule/` and `agent/`. Readable entries get no carve-out at
all: nothing under the capsule root is ever a declared readable input.

```rust
/// One run inside a capsule.
pub struct Execution {
    /// A typed argument vector. Never a shell string — SPEC-030 § contract
    /// and interpretation provenance says the trusted plan uses typed argv,
    /// and a string would put quoting on a security boundary.
    argv: Argv,
    /// The environment, from a **closed vocabulary** (below).
    env: CapsuleEnv,
    /// Wall-clock bound, enforced trusted-side.
    timeout: Duration,
    /// Per-file write bound, enforced trusted-side.
    file_size_cap: ByteCount,
}

/// The environment a capsule may be given, at this slice's altitude.
///
/// A `BTreeMap<String, String>` was the first shape and is rejected:
/// `--clearenv` stops *inheritance* only, so an open map is a second, unguarded
/// route for exactly the credentials the mount set denies — a caller passing
/// `GITHUB_TOKEN` by value satisfies every mount assertion in sec-7. Credential
/// denial cannot be claimed to rest on absence from the mount set while another
/// public field carries values in.
pub enum CapsuleEnvVar {
    /// Derived from the bound readable paths (below). Never the host's.
    Path,
    /// Always the inner agent home.
    Home,
    /// Always `dumb`.
    Term,
    /// The capsule's own git identity — never the host's (sec-3).
    GitAuthorName, GitAuthorEmail, GitCommitterName, GitCommitterEmail,
}
```

Every variant's *value* is computed trusted-side from the placement; none is
caller-supplied text. This slice needs no others: its only capsule executions are
the provisioning git commands and `sec-7`'s probes.

**Widening this vocabulary is a later slice's decision and must arrive with its
own governed rule.** The launch slice will need to give a harness a credential —
the spike does it by binding one file read-only inside the agent home
(`sandbox.sh:248-250`), deliberately narrower than the `~/.claude` bind it
replaced. That is a scoped credential *capability*, not an open map, and
`SPEC-030` grants no exception that would justify one here.

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

The contract's meaning is the eight properties `sec-7` proves. They are named
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
| 7 | process-tree teardown | that no descendant outlives the `execute` call, observed by the parent |
| 8 | trusted observation of resource limits and termination | `timeout`, `file_size_cap`, and every `Termination` variant being correctly distinguished |

The eight rows stand in **one-to-one correspondence** with `SPEC-030` §
Platform backend contract's eight clauses. Nothing is merged, and that is a
correction rather than the original shape: an earlier draft folded rows 7 and 8
together, on the reasoning that teardown and observation are one question — what
the trusted parent can establish about a process it did not trust.

`RV-346` `F-2` refuted it, and the refutation turns `DEC-156`'s own principle on
that record's arithmetic. The two are independent enforcement claims: a backend
can reap every descendant and still misclassify a timeout as a signal, or
classify termination perfectly and leave a grandchild alive. A control removing
both axes at once cannot establish which guard produced the paired result, which
is exactly what one-property-removed exists to prevent — so a seven-row suite
would have admitted a backend failing one of the eight clauses.

`DEC-156` stated the count as seven and has been **corrected to eight**
(2026-08-06, authorised by the human author). Only the number moved; every
argument in that record stands. `sec-7` builds eight rows, each with its own
probe and its own one-axis control.

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

#### The readable set is declared, and on a closure-managed host it is the closure

The spike binds one root, `/nix/store`, and filters `PATH` to entries under it
(`sandbox.sh:141-153`). **The product does not do this, and the reason is safety
rather than `POL-002`.** A Nix store holds every package ever realised on that
host: other projects' toolchains, every historical build, every tool the operator
ever ran once. Binding it whole gives a capsule a larger executable set than an
ordinary unconfined process on a conventional distribution has, which inverts
`REQ-459` property 2 — an *explicit* input set — into an implicit and enormous
one. That the spike did it is a fact about a rig proving feasibility on a
disposable fixture, not a posture to carry into the product.

Nix is also what makes the correct posture cheap. A closure is exactly the
transitive runtime set of a realised derivation: what the toolchain specifies and
nothing else. So the capsule gets the closure, not the store.

Two declared lists, and one declared host capability:

```toml
[capsule]
# Bound exactly as declared. Directories or single files.
readable-roots = ["/bin/sh", "/usr/bin/env"]

# Expanded through `closure-resolver`; every path it returns is bound read-only.
closure-roots = [".doctrine/capsule-toolchain"]

# The host capability that performs the expansion. Absent on hosts that need none.
closure-resolver = ["nix-store", "--query", "--requisites"]
```

- **`readable-roots`** is bound as declared. A single-file entry is what covers
  shebang interpreters, which the kernel resolves before `PATH` exists and which
  cost the spike two separate findings (`sandbox.sh:171-188`, `/usr/bin/env` and
  `/bin/sh`). Binding the interpreter file rather than `/bin` is portable by
  construction; binding the directory is a posture that silently widens on a host
  where `/bin` holds hundreds of binaries.
- **`closure-roots`** entries are resolved through their symlinks to a realised
  path, then handed one at a time to `closure-resolver`. Each path the resolver
  returns is bound read-only, individually. On NixOS a project builds its
  toolchain out of band — `nix build .#capsuleToolchain -o .doctrine/capsule-toolchain`
  — and declares that one stable symlink; the store paths behind it change on
  every rebuild and are never written into config.
- **At least one of the two lists must be non-empty**, or provisioning refuses:
  a capsule with no readable inputs can execute nothing, and silently producing
  one would report a confinement success that is really a configuration failure.

Every entry of both lists is **probed at provisioning**. An absent entry, an
unrealised `closure-roots` path, or a declared `closure-roots` with no
`closure-resolver` each refuse, naming the entry and the config key — `POL-002`
facet 3, and the same shape `jail.rs:302-393` already uses for jail policy.

#### The resolver is a query, never a build — and the contract enforces it

`closure-resolver` runs **trusted-side**, which is the one place this design
executes an external command outside a capsule, so its limits are stated rather
than assumed.

1. **It is given an already-realised path, never a flake reference or a
   repository path.** `nix-store --query --requisites <store-path>` reads the Nix
   database; it does not evaluate anything. `nix build` *would* evaluate
   `flake.nix` — a repository-controlled file, and one this project's own
   `interpreted_paths` names — and evaluating it trusted-side is precisely the
   hazard `trusted_side_forbidden_executables` exists to deny. Provisioning
   therefore never realises a toolchain: a `closure-roots` entry that does not
   already resolve to an existing realised path refuses, and the operator builds
   it out of band.
2. **The resolver's executable is checked against the transaction's own
   forbidden list.** After `sec-4` resolves the policy, provisioning refuses when
   the normalized basename of `closure-resolver[0]` appears in
   `trusted_side_forbidden_executables`. This is `SPEC-030`'s rule applied to
   this slice's only external-command step — *"the trusted transaction plan …
   refuses any external-command step whose normalized executable matches the
   list"* — and it makes `REQ-449`'s forbidden list a live constraint in this
   slice rather than a field with no consumer until launch exists. A project that
   forbids `nix-store` must declare a different resolver or no `closure-roots`.
3. **Its output is data, not trust.** One absolute path per line. A relative
   path, an empty line, a non-existent path, or output exceeding a named bound
   refuses; the resolver's own nonzero exit refuses, naming it.
4. **It comes from the control plane's own `.doctrine/doctrine.toml` at `root`**
   — `[capsule]` is operational config (`DEC-158`), never resolved from a capsule
   checkout or a harvested tree, and never from the contracted base.

#### The derived inner `PATH`

The capsule's inner `PATH` is *derived*: the host `PATH` entries that lie beneath
**any** bound path, after closure expansion, in host `PATH` order, deduplicated.
Nothing else. Several bound paths contributing is the ordinary case:

- A conventional host declaring `readable-roots = ["/usr/bin", "/opt/toolchain",
  "/bin/sh"]` against `PATH=/home/u/.local/bin:/usr/bin:/opt/toolchain/bin`
  derives `PATH=/usr/bin:/opt/toolchain/bin`. `/home/u/.local/bin` is beneath no
  bound path and is dropped. `/bin/sh` is a file, so no `PATH` entry can be
  beneath it; it is declared for the shebang reason above.
- A NixOS host declaring one `closure-roots` entry derives the `…/bin`
  directories of the closure members that are on the host `PATH`, and drops
  `~/.cargo/bin` and every store path outside the closure.

#### Resolution precedes validation, and what is validated is what is bound

An earlier draft of this section said containment was lexical and that symlinks
were deliberately not followed. That was wrong, and wrong in the direction that
matters: **bubblewrap dereferences the source path of a `--ro-bind`.** `RV-346`
`F-1` demonstrated it by execution — a declared directory that was a symlink to
an out-of-tree target made the target readable inside the sandbox. A declared
root of `/opt/tools` pointing at the canonical repository would therefore have
bound the canonical repository while passing every lexical check.

The rule is now the opposite, and it is one rule rather than a policy:

1. **Resolve.** Every declared entry, and every path a closure resolver returns,
   is fully resolved — all symlink components, to a real path — before anything
   else looks at it.
2. **Validate the resolved path** against `ForbiddenScopes` and the reserved
   inner destinations, in `CapsulePlacement::try_new`.
3. **Bind the resolved path.** What the backend receives is the path that was
   validated, so there is no window in which the two differ.

The `PATH` containment test above runs over resolved paths on both sides, for the
same reason. Note what this does *not* do: it does not follow links found
*inside* a bound directory at execution time — nothing can, since the capsule may
create them. It closes the class that matters, which is a declared entry
resolving somewhere other than where it reads.

A `closure-roots` entry pays nothing for this: a closure is transitively complete,
so every target a closure member links to is itself a closure member and is
already bound. Closure expansion narrows the visible set *and* removes the
link-escape class, which is why it is the preferred form rather than a NixOS
accommodation.

Step 1 also has a limit worth naming: resolution happens at provisioning, and a
declared path could in principle be re-pointed afterwards. That is a
control-plane-side race on operator-owned configuration, not a capsule-reachable
one — a capsule cannot write any declared path, by rule 2 — and closing it would
need the bind to be performed against an open file descriptor rather than a path.
Recorded here as a known limit rather than solved; `sec-9` carries it.

`sec-7` property 2 additionally carries a row that *executes* a binary from each
bound path rather than stat-ing it, which is the spike's `F-P05-17` lesson: the
shebang class was found by running the project's own suite, not by inspecting the
profile.

Only the two declared lists and the per-base export (`sec-3`) ever become
readable inputs. There is no host-shaped default and no fallback: *"Doctrine
supplies no ecosystem default"* is `SPEC-030`'s rule for `[interpretation]`, and
the same discipline applies here for the same reason.

#### Bounds are enforced outside the sandbox

`timeout -k <grace> <secs>` wraps the `bwrap` exec from outside, and the
per-file cap is set on the child before it (`setrlimit(RLIMIT_FSIZE)`, the typed
equivalent of the spike's `ulimit -f`). Neither is reachable from inside the
namespace, which is what makes property 8 an observation rather than a report;
the reaping that property 7 asserts is the same parent's responsibility.
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
2. **No unvalidated placement exists.** `CapsulePlacement` has private fields and
   one validating constructor, so the reserved-destination, overlap and
   forbidden-scope checks cannot be bypassed by assembling the struct directly.
3. **What is validated is what is bound.** Every host path is fully resolved
   before validation, and the resolved path is the one handed to the backend.
4. **Absence, not read-only presence, is how denial is achieved.** The canonical
   repository, other capsules, and the credential store are never in `readable`.
   `DEC-157` makes this structural for the repository by provisioning from an
   export instead.
5. **Credential denial has no second route.** The environment is a closed
   vocabulary whose values are computed trusted-side, so absence from the mount
   set is the whole of it rather than most of it.
6. **No package store is ever bound whole.** Every readable input is a declared
   path or a member of a declared closure. There is no configuration — and no
   fallback taken on missing configuration — under which `/nix/store` or an
   equivalent host-wide artefact store becomes readable in its entirety.
7. **Provisioning realises nothing.** The only external command it runs outside
   a capsule is `closure-resolver`, on an already-realised path, and its
   executable is checked against the transaction's own
   `trusted_side_forbidden_executables`.
8. **Every termination fact is the parent's observation.** No `Termination`
   variant is derivable from capsule-written state.
9. **The default is refusal.** Empty argv, empty mounts, both declared lists
   empty, an absent or unrealised declared entry, `closure-roots` with no
   resolver, an unresolvable capsule root, a reserved or overlapping inner
   destination, a resolved path inside a forbidden scope: each refuses. There is
   no arrangement of missing configuration that produces an unconfined
   execution, and none that produces a *wider* readable set than the
   configuration names.
10. **`src/worktree/` is not imported.** Enforced by `sec-6`'s export set, which
   carries no bubblewrap surface at all.

### Verification alignment

Shape assertions are necessary and never sufficient (`DEC-156`; risk `R3`), so
this section owes both kinds. Pure, hermetic:

- `bubblewrap_argv_denies_network_unless_posture_is_permitted`
- `bubblewrap_argv_places_share_net_only_after_unshare_all`
- `empty_mount_vector_refuses_rather_than_running_unconfined`
- `empty_argv_refuses`
- `inner_path_is_derived_only_from_bound_paths`
- `inner_path_draws_from_every_bound_path_in_host_path_order`
- `a_file_readable_root_contributes_no_inner_path_entry`
- `absent_readable_root_refuses_naming_the_entry_and_the_config_key`
- `both_declared_lists_empty_refuses`
- `closure_roots_without_a_resolver_refuses_naming_both_keys`
- `unrealised_closure_root_refuses_and_provisioning_never_realises_it`
- `resolver_whose_basename_is_forbidden_by_the_policy_refuses`
- `resolver_returning_a_relative_or_absent_path_refuses`
- `resolver_nonzero_exit_refuses_naming_the_resolver`
- `closure_expansion_binds_each_returned_path_individually_never_their_common_parent`

Placement validation — one per refusal variant, and the `RV-346` `F-1` class
first because it was found by execution rather than by reading:

- `declared_root_that_resolves_into_the_canonical_repository_refuses`
- `declared_root_that_resolves_into_the_credential_scope_refuses`
- `declared_root_that_resolves_into_the_capsule_root_refuses`
- `declared_root_that_resolves_to_the_filesystem_root_refuses`

Descendant mutants, one per forbidden scope — the `F-10` class, which an
equal-or-ancestor test admits:

- `a_file_inside_the_canonical_repository_refuses` (`…/.git/config`)
- `a_file_inside_the_control_plane_state_refuses`
- `a_file_inside_a_credential_location_refuses`
- `a_sibling_transaction_root_under_the_capsule_root_refuses`
- `a_writable_entry_under_this_placements_own_transaction_root_is_admitted` —
  the carve-out, asserted positively so a fix that refuses everything under the
  capsule root cannot pass
- `a_readable_entry_under_this_placements_own_transaction_root_refuses` — the
  carve-out is writable-only
- `a_sibling_directory_whose_name_extends_a_forbidden_scope_is_admitted`
  (`/var/lib/doctrine-other` against `/var/lib/doctrine`) — the control that
  fails if overlap is implemented as a textual prefix test
- `reserved_inner_destination_supplied_as_a_readable_entry_refuses` — one case
  per reserved path, `/source` included, since shadowing it substitutes the
  contracted export
- `two_entries_with_the_same_inner_path_refuse`
- `an_inner_path_that_is_an_ancestor_of_another_refuses`
- `the_bound_host_path_is_the_resolved_path_not_the_declared_one`

Environment vocabulary:

- `capsule_env_is_a_closed_vocabulary_with_trusted_side_values`
- `no_public_route_carries_caller_supplied_environment_text`
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
    /// The commit this capsule is provisioned from, as a type the trusted
    /// boundary mints rather than a bare oid a caller can pass anything for.
    pub base: AcceptedBase,
    /// The admitted mechanism that created this capsule and must execute it.
    /// Recorded because provisioning itself calls `execute` (step 11), so the
    /// choice is made inside this slice and an observation that cannot be
    /// attributed to a backend cannot be attributed to an admission verdict
    /// either.
    pub backend: BackendId,
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

**Two fields are additions beyond `DEC-159`'s enumeration**, and both are
justified in that decision's own terms — it binds "only what has a consumer
now", and each of these has one *in this slice*. `backend` has provisioning's own
`execute` calls; `AcceptedBase` replaces a bare `CommitId` because nothing else
would let step 8 check that the export it is about to bind is an export of the
base it claims. Neither anticipates a later slice's needs.

**`AcceptedBase` documents an obligation it does not check.** It carries the
commit and the assertion that a trusted-side caller resolved it as accepted; it
does **not** verify the commit against the canonical accepted ref, because the
accepted-ref check belongs to admission (`REQ-455`), an explicit Non-Goal here.
What this slice does verify is the weaker and entirely local claim that the
export bound at `/source` contains exactly this base — see step 8.

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
    host: &dyn HostFacts,       // statvfs, resolution, path existence, environment (sec-5)
    backend: &dyn CapsuleBackend,
) -> Result<CapsuleTransaction, ProvisionRefusal>;

pub struct ProvisionRequest {
    pub repository_root: PathBuf,
    /// The base, as the typed value the trusted boundary mints. Never a bare
    /// `CommitId` — see `AcceptedBase` below.
    pub base: AcceptedBase,
    pub phase: PhaseIdentity,
    /// The transaction id, **allocated by the caller**, not by `provision`.
    pub id: TransactionId,
    /// An optional phase-refinement document, by path (sec-4).
    pub refinement: Option<PathBuf>,
    pub network: NetworkPosture,
}
```

**`HostFacts` carries no clock**, and the transaction id is an input rather than
something `provision` mints. Both follow the project's pure/imperative rule that
the impure value is read in the outer shell and passed in — the same pattern the
date and uid inputs already use, with `src/clock.rs` as the single home for
wall-clock reads. `RV-346` `F-9` and `F-14` are what forced this to be stated
rather than left as "a collision-resistant source": with no allocator named in
the signature, the collision case could not be reached through the stated API,
so `provision_onto_an_existing_transaction_root_refuses_and_removes_nothing` was
a test that could not be written. With the id as a parameter, the test hands the
same id twice, and no entropy is involved in reaching the branch.

`TransactionId` is minted by the `provision` verb from a collision-resistant
source (`sec-6`); the type refuses a value that is not a single path component,
so an id can never carry a separator into the layout.

1. **Read `[capsule]`** from `.doctrine/doctrine.toml` at `root` — capsule root,
   resource bounds, the two declared readable lists and the closure resolver
   (`sec-2`, `sec-5`). Unresolvable capsule root refuses, naming the key; both
   readable lists empty refuses; `closure-roots` with no `closure-resolver`
   refuses, naming both keys.
2. **Probe each declared entry.** A `readable-roots` entry that does not exist
   refuses; a `closure-roots` entry that does not resolve through its symlinks to
   an existing realised path refuses. Provisioning never realises one (`sec-2`).
3. **Probe capacity** by `statvfs` on the capsule root. Below one expected size,
   refuse; between one and two, warn and continue; unknown, report and continue
   (`sec-5`).
4. **Resolve the base policy** from the blob at `base` (`sec-4`). This is the
   *only* read of `.doctrine/doctrine.toml` from the contracted base, and it is
   `DEC-136`'s read-once invariant made physical.
5. **Apply the refinement**, when one was supplied — the pure monotonic
   restriction algebra of `sec-4`. Any widening refuses.
6. **Admit the closure resolver against the policy just bound.** Refuse when the
   normalized basename of `closure-resolver[0]` appears in the policy's
   `trusted_side_forbidden_executables`. This step is *after* 5 and not before,
   because a phase refinement may add a forbidden entry, and the entry it adds
   must bind the transaction it was supplied for.
7. **Expand the closures.** One resolver invocation per `closure-roots` entry,
   each output line validated as an existing absolute path. Every returned path
   becomes an individual read-only input — never their common parent, which is
   how a closure collapses back into a whole store.
8. **Publish or adopt the per-base export**, by the atomic protocol below. Ends
   with an export validated to contain exactly `base`.
9. **Create the transaction root**, exclusively, by the ownership protocol below.
10. **Assemble the placement** through `CapsulePlacement::try_new` — export
   read-only at the reserved `/source`, `capsule/` and `agent/` writable, every
   bound readable path read-only at its resolved host path, working directory
   `/capsule`, network posture as requested.
11. **Clone inside — three executions, not one.** `Execution` carries a single
   typed argv by design (`sec-2`), so provisioning issues three
   `backend.execute` calls and checks the `Observation` after each, refusing on
   the first that does not exit zero:

   | # | argv |
   |---|---|
   | 1 | `git clone --no-hardlinks --quiet -c user.name=… -c user.email=… -- /source /capsule/repo` |
   | 2 | `git -C /capsule/repo switch --detach --quiet <base>` |
   | 3 | `git -C /capsule/repo remote remove origin` |

   The alternatives were considered and rejected. `sh -c` reintroduces the
   quoting hazard on a security boundary that `sec-2` exists to avoid. Running
   commands 2 and 3 trusted-side would put trusted Git in a capsule-authored
   repository, which `SPEC-030` forbids outright. A control-plane-authored
   helper mounted read-only outside writable state — the spike's `/rig` posture
   (`sandbox.sh:36-43`) — is the right answer when a *sequence* must be
   capsule-side and conditional; for three unconditional commands it buys a
   mount and a shipped script to avoid two extra sandbox starts. Three
   executions also make each step's termination separately observable, which is
   better evidence than one aggregate status.
12. **Assert the capsule identity persisted** into the clone's config, by a
   fourth execution reading it back. `git clone -c` persisting is what keeps
   every later capsule git operation cheap, and a git that stopped doing so
   would restore the resolver stall silently (`provision.sh:68-72`).
13. **Return the transaction.**

Steps 1, 4, 5 and 6 are pure given `HostFacts`; steps 2, 3, 7–12 are the impure
part, in the thin outer part of the module per the project's pure/imperative
split. Step 7 is the one place provisioning executes an external command outside
a capsule, and step 6 is the check that governs it.

#### Publishing the export atomically (step 8)

"Idempotent reuse" was one sentence and hid four questions. The protocol:

1. **Adopt, if a valid export exists.** `export/<base-oid>/` is adopted only when
   it is a real directory (never a symlink — a symlink here is a `sec-2` `F-1`
   escape by another route), is a bare repository, has **no** `objects/info/alternates`,
   holds `base` as a complete object closure, and holds **no ref other than** the
   single ref naming `base`. Anything else refuses rather than being repaired:
   a partially-built or widened export binds more host object state than
   `DEC-157` permits, and repairing one in place is how a shared artefact
   silently acquires a second writer.
2. **Otherwise build in a temporary directory this call exclusively created** —
   `export/.building-<transaction-id>/`, created with the same `mkdir` semantics
   that fail when the path exists — then `git init --bare` and a fetch of `base`
   from the repository root. An existing path refuses; it is never entered,
   never cleaned, and never reused.

   The name is *derived* from the transaction id but ownership does not rest on
   it. `RV-346` `F-8` is why: this step runs before step 9, so at this point
   nothing has established that this call owns the id — collision-resistant is
   not collision-free, and a retry with the same id is not even improbable. The
   exclusive create is what establishes ownership, and it establishes it here,
   independently of step 9. Two calls holding the same id therefore both refuse
   rather than one silently building into the other's directory.
3. **Validate the temporary export** by the same rules as step 1, so what is
   published and what is adopted are checked by one function.
4. **Publish by no-replace rename.** `renameat2(RENAME_NOREPLACE)` — the loser of
   a concurrent race gets `EEXIST` rather than replacing a live export that other
   capsules already have bound.
5. **The loser adopts the winner**, re-running step 1 against the published path,
   and removes **only its own** temporary directory. It never removes the
   published export, and it never removes another builder's temporary directory.

The result is that concurrent provisioning on one base converges on one export
without either builder repairing or replacing anything.

#### Owning the transaction root (step 9)

The root is created with **`mkdir` semantics that fail when the path exists** —
never a recursive create-if-missing. Provisioning holds a creation token for the
root it made, and rollback (below) refuses to remove a path unless that token
says this call created it. The token is minted by the create, not by the id:
exclusivity is a fact the filesystem establishes, and the id's entropy only
makes the refusing case rare rather than impossible.

Without this the rollback rationale inverts: on an id collision or a retry with
the same id, a recursive create would enter an existing transaction root, and the
failure path would then delete another transaction's work — the exact automated
deletion of live work `REQ-461` criterion 3 and `DEC-133`/`DEC-137` forbid.

The same reasoning governs step 8's temporary export directory, which is created
the same way and holds its own token. Both are reachable in test by handing
`provision` an id that is already in use, since the id is a request field rather
than something the call mints (`F-9`).

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

A provisioning call that fails after step 9 removes the transaction root it
created, and only that path, and only on the creation token step 9 issued it.
A call that failed *because* the root already existed holds no token and removes
nothing. `REQ-461` criterion 3 forbids automated deletion of
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
2. **The export is immutable once published, and never repaired.** It is built in
   a uniquely owned temporary directory, validated, and published by no-replace
   rename. A partially-fetched export is never observable at
   `export/<base-oid>/`; an existing one is adopted only if it validates, and
   otherwise refuses rather than being mutated into shape.
3. **The export at `/source` is an export of the transaction's own base.**
   Verified before the clone, so a placement pairing base B with an export of A
   refuses instead of producing a capsule whose contracted base is a fiction.
4. **Two transactions share no mutable state.** Distinct `tx/<id>/` roots,
   distinct clones, distinct processes, distinct temporary areas. The export is
   shared and read-only — not shared *mutable* state, so `REQ-450` criterion 1
   is untouched.
5. **The transaction id is allocated trusted-side and supplied to `provision`;
   every directory keyed by it is created exclusively.** A call that did not
   create a path holds no token over it, and that holds for the temporary export
   directory as much as for the transaction root.
6. **A refused provision leaves no transaction, and removes nothing it did not
   create.** Either a `CapsuleTransaction` is returned, or the root this call
   exclusively created is gone and nothing else changed.

### Verification alignment

`REQ-450` criterion 1 names **five** axes — checkout, repository, runtime,
process, temporary state — and each is discharged **by writing or acting through
one transaction and observing absence in the other**, never by comparing paths.

The axes are not all of one kind, and an earlier draft of this section treated
them as if they were: it listed storage tests only, and offered one control —
provisioning the second transaction into the first's root — for all five.
`RV-346` `F-3`… `F-6` is right that this establishes nothing about the process
axis, because two capsules sharing a directory still do not share a process
namespace. Four axes are storage and take the storage control; the fifth is not
and takes its own.

| axis | what is written or done | control (one property removed) |
|---|---|---|
| checkout | a file in the working tree at `/capsule/repo` | second transaction provisioned into the first's root |
| repository | an object and a ref in `/capsule/repo/.git` | as above |
| runtime | a sentinel in the agent home `/agent` — the state a harness accumulates across a run | as above |
| temporary state | a file in the capsule's `/tmp` | as above |
| **process** | capsule A enumerates `/proc` and attempts `kill -0` on a pid the trusted side observed capsule B running under, while both run concurrently | **`--unshare-pid` alone removed**, every other property intact |

The process row is what makes the fifth axis carry information. Its green arm
asserts two things positively — B's process does not appear in A's `/proc`, and
the signal fails — and its control asserts that with the pid namespace shared,
and *only* that changed, both succeed. That is `DEC-156`'s discipline applied to
an axis that has no storage to compare.

Test titles:

- `two_transactions_on_one_base_share_no_writable_checkout`
- `two_transactions_on_one_base_share_no_repository_objects`
- `two_transactions_on_one_base_share_no_runtime_state_in_the_agent_home`
- `two_transactions_on_one_base_share_no_temporary_state`
- `control_second_transaction_in_the_first_root_does_share_each_storage_axis`
- `concurrent_capsules_cannot_see_each_others_processes`
- `concurrent_capsules_cannot_signal_each_others_processes`
- `control_with_the_pid_namespace_shared_both_become_possible`
- `export_is_adopted_across_transactions_on_the_same_base`
- `export_holds_the_contracted_history_and_no_other_ref`
- `export_with_an_alternates_file_refuses_rather_than_being_adopted`
- `export_that_is_a_symlink_refuses`
- `concurrent_publication_converges_on_one_export_and_the_loser_adopts_it`
- `placement_pairing_a_base_with_another_bases_export_refuses`
- `clone_inside_leaves_no_working_tree_trusted_side`
- `capsule_identity_persists_into_the_clone_config`
- `a_failing_execution_in_the_three_step_clone_refuses_at_that_step`
- `failed_provision_removes_only_the_root_it_created`
- `provision_onto_an_existing_transaction_root_refuses_and_removes_nothing` —
  reached by handing the same `TransactionId` twice, deterministically, since it
  is a request field
- `provision_onto_an_existing_temporary_export_directory_refuses_and_removes_nothing`
- `a_transaction_id_carrying_a_path_separator_refuses_at_construction`
- `failed_provision_leaves_an_existing_export_intact`

`REQ-450` criteria 2 and 3 need candidate identity and harvest from later
slices. This slice records a contributing `--change` against criterion 1 and
reports the requirement as **partial** in the reconciliation brief, not quietly
claimed. `REQ-448`'s denial half is the same shape and is proven by `sec-7`.

<!-- doctrine:section sec-4 -->
## Interpretation policy: resolution, normalization, and restriction

`REQ-449` binds each transaction to an interpretation policy resolved from the
contracted base, and permits a phase contract only to *narrow* it. This section
builds the typed value, the canonical hash, the refusal set, and the restriction
algebra. `sec-3` step 4 and step 5 are its callers.

### Current behaviour, and why the shared reader cannot be extended

`src/dtoml.rs` is leaf-tier (`.doctrine/adr/001/layering.toml:32`) and holds
`DOCTRINE_TOML` (`dtoml.rs:80`) and `read_doctrine_toml_text(root)`
(`dtoml.rs:87`). It is **deliberately tolerant**: unknown top-level tables are
ignored, absent values default.

`DEC-136`'s handoff item 1 expects "a direct implementation in the existing
`doctrine.toml` loader rather than a new configuration subsystem". That is not
available, for two independent reasons:

1. **The source differs.** `read_doctrine_toml_text` reads a file from disk at
   `root`. `REQ-449` resolves a **blob at the contracted base OID** —
   `git::read_path_at(root, base, DOCTRINE_TOML)` (`src/git.rs:790`), which is
   working-tree-free and returns `Ok(None)` for an absent path. Verified this
   session; it is the entire impure surface the resolution needs.
2. **The strictness differs, in opposite directions.** The shared reader must
   tolerate what it does not know, or every unrelated table breaks it.
   `REQ-449` criterion 1 must **refuse** an unknown key. One type cannot be both.

`DEC-136`'s decision — the `[interpretation]` block stays in
`.doctrine/doctrine.toml` and is resolved once from the base — **stands
unchanged**. Only that supporting note is wrong, which makes it a record
correction owed to the reconciliation brief rather than a Revision.

`src/reserve.rs:78-97` is the precedent and the shape to follow: a tolerant
outer document projecting one table, a pure `parse_*(text)` function, and a thin
loader that supplies the text. This design differs from it in exactly one
respect — the projected table is strict inside — and in where the text comes
from.

### Target behaviour

`src/interpretation.rs`, leaf-tier in the root package beside `dtoml`, exported
to `doctrine-control` through the lib target (`sec-6`). Pure throughout: it
takes text and returns a typed value or a refusal. Nothing in it reads a file, a
clock, or a repository.

```rust
/// The v1 schema version. The only accepted value (STD-001).
pub const INTERPRETATION_SCHEMA: u64 = 1;

/// A normalized, validated interpretation policy.
///
/// Construction is through `parse` alone — there is no public constructor and
/// no public field mutation, so an unnormalized value of this type cannot
/// exist. That is what lets `canonical_hash` and `restrict` assume
/// normalization rather than re-checking it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpretationPolicy {
    schema: u64,
    /// Byte-sorted after duplicate rejection.
    forbidden_executables: Vec<ExecutableName>,
    /// Byte-sorted after duplicate rejection.
    interpreted_paths: Vec<PathPattern>,
    /// Order preserved. Non-empty.
    verification: Vec<VerificationRow>,
}

/// A normalized executable basename: non-empty, no slash, no whitespace,
/// not `.` or `..`.
pub struct ExecutableName(String);

/// A normalized repository-relative gitignore-style pattern: not absolute,
/// no backslash, no NUL, no lexical `..` component.
pub struct PathPattern(String);

/// One verification row. Argument order preserved; every argument non-empty.
pub struct VerificationRow { argv: Vec<String> }
```

```rust
pub fn parse(text: &str) -> Result<InterpretationPolicy, PolicyRefusal>;
pub fn canonical_hash(policy: &InterpretationPolicy) -> PolicyHash;
pub fn restrict(
    base: &InterpretationPolicy,
    refinement: &InterpretationPolicy,
) -> Result<InterpretationPolicy, RestrictionRefusal>;
```

### Why the table is walked rather than deserialized

The obvious shape is a `#[derive(Deserialize)]` struct with
`deny_unknown_fields`. It is rejected: `REQ-449` criterion 1 requires **six
distinguishable refusals** — missing block, missing required key, unknown key,
unsupported schema version, invalid normalized value, empty verification
sequence — and serde produces a formatted *string* for the first four. Turning
those back into typed refusals means pattern-matching a dependency's error
message, which is a classification that breaks silently when `toml` rewords
itself, in exactly the code whose job is to refuse precisely.

So: parse to `toml::Table` — tolerant at the top level, so `[dispatch]`,
`[capsule]` and `[reservation]` are simply not looked at — then project
`interpretation` and walk its keys explicitly against the known set. Absent
table is `BlockMissing`; a key in the table but not in the known set is
`UnknownKey { key }`; a key in the known set but not in the table is
`KeyMissing { key }`.

That distinction is required and not incidental: `SPEC-030` says *"the two lists
may be explicitly empty when the project genuinely has no such instances;
omission is not equivalent to emptiness."* A derive with `#[serde(default)]`
collapses exactly that difference.

### Validation

Applied per field, after presence is established.

| field | rule | refusal on violation |
|---|---|---|
| `schema` | integer equal to `INTERPRETATION_SCHEMA` | `UnsupportedSchema { found }` |
| `trusted_side_forbidden_executables` | each entry non-empty, contains no `/`, no whitespace, and is neither `.` nor `..` | `InvalidExecutable { entry, reason }` |
| | no two entries equal | `DuplicateEntry { field, entry }` |
| `interpreted_paths` | each entry non-empty, not absolute, no `\`, no NUL, no lexical `..` component | `InvalidPathPattern { entry, reason }` |
| | no two entries equal | `DuplicateEntry { field, entry }` |
| `verification` | at least one row | `EmptyVerificationSequence` |
| | each row is a table | `MalformedVerificationRow { row }` |
| | each row holds `argv` | `RowKeyMissing { row, key }` |
| | each row holds **no key but** `argv` | `UnknownRowKey { row, key }` |
| | each row's `argv` is an array of strings | `MalformedVerificationRow { row }` |
| | each row's `argv` non-empty | `EmptyArgv { row }` |
| | each argument non-empty | `EmptyArgument { row, index }` |

**The walk is recursive, and the row is where an earlier draft stopped
walking.** `RV-346` `F-11`: the explicit key walk covered the `interpretation`
table and then validated rows only for non-emptiness, so a row spelled
`{ argv = [...], extra = true }` had no defined refusal and would have been
projected silently out of the typed value — the exact silent-acceptance that
`REQ-449` criterion 1's unknown-key refusal exists to prevent, one level down.
Strictness that stops at the first level is not strictness; it is a strict
outer table wrapping a tolerant inner one. Each row is therefore walked against
its own known key set — `argv`, and nothing else — with `RowKeyMissing` and
`UnknownRowKey` distinguished for the same reason `KeyMissing` and `UnknownKey`
are at the table level.

`SPEC-030` says arguments are "non-empty UTF-8 strings". TOML strings are UTF-8
by construction, so the only executable half of that rule is non-emptiness —
stated here rather than left as an assertion that cannot fail.

**Duplicate rejection precedes sorting, and both precede the hash.** A
`BTreeSet` would have made the sort free and the duplicate *invisible*, which is
the opposite of `REQ-449` criterion 2's requirement to sort "after duplicate
detection". The typed value therefore holds `Vec`, sorted, with the duplicate
check as an explicit refusing step.

### The canonical hash

`canonical_hash` is SHA-256 over a **deterministic encoding of the typed value**,
never over source text or a re-serialized TOML document. Re-serializing would
let formatting choices — quoting style, key order, whitespace, integer spelling
— re-enter a value whose entire purpose is to be stable across them.

```
domain  := "doctrine.interpretation.v1"
encode  := domain
         ‖ u64_le(schema)
         ‖ u64_le(count forbidden) ‖ for each: u64_le(len) ‖ utf8 bytes
         ‖ u64_le(count paths)     ‖ for each: u64_le(len) ‖ utf8 bytes
         ‖ u64_le(count rows)      ‖ for each row:
                u64_le(count argv) ‖ for each arg: u64_le(len) ‖ utf8 bytes
```

Length prefixes are what make the encoding injective. Plain concatenation lets
`["ab", "c"]` and `["a", "bc"]` hash identically, and those are two different
forbidden-executable sets — one of which forbids an executable the other
permits. The domain prefix keeps a future v2 encoding from colliding with a v1
one over the same bytes.

`PolicyHash` is a newtype over the digest with no public constructor other than
`canonical_hash`, so a hash cannot be fabricated from anything but a validated
policy.

### The restriction algebra

`DEC-159` puts `REQ-449` criterion 4 here, as a pure function over typed values
— never over source text, which is `SPEC-030`'s explicit instruction.

**The refinement document states the refined policy in full**, in the same
`[interpretation]` schema, and goes through the same `parse`. Two consequences,
both deliberate:

- Subset validation compares like with like through one parser. This is DRY on a
  genuine invariant — the two documents describe the same typed values and are
  obliged to change together, which is the shared referent `DEC-155` found
  absent in the bubblewrap flag names and present here.
- A **delta** document was rejected because it cannot be refused for the things
  `REQ-449` criterion 4 names. "Remove a project verification row" and "reorder
  project verification" have no spelling in an additions-only document, so an
  author who drops a project check would be silently granted the removal. Stated
  in full, dropping a check *is* the refusal.

Rules, evaluated in order:

| # | check | result |
|---|---|---|
| 1 | `refinement.schema == base.schema` | else `SchemaMismatch { base, refinement }` |
| 2 | `refinement.forbidden_executables ⊇ base.forbidden_executables` | else `ForbiddenEntryRemoved { entry }` |
| 3 | `refinement.interpreted_paths ⊇ base.interpreted_paths` | else `InterpretedPathRemoved { entry }` |
| 4 | `base.verification` is a **prefix** of `refinement.verification`, row by row, `argv` byte-equal | else the diagnosis below |
| — | all hold | `Ok(refinement.clone())` |

Adding a forbidden executable or an interpreted path is a *restriction*: both
lists name things the trusted plan refuses to run or treats as hostile, so a
superset is strictly narrower. That is why rules 2 and 3 are superset checks in
the same direction rather than one of each.

Rule 4's failure is diagnosed rather than reported as one opaque mismatch,
because the three cases have different fixes. The diagnosis is an **ordered**
classification, and the order is what makes it deterministic:

1. some base row appears nowhere in the refinement → `VerificationRowRemoved { index }`
2. otherwise, every base row appears and the base is a *subsequence* of the
   refinement — so the additions are interleaved rather than appended →
   `VerificationRowInserted { index }`, `index` being the first refinement
   position not consumed by the base subsequence
3. otherwise, every base row appears but the base is not a subsequence — the
   rows moved relative to each other → `VerificationRowReordered { index }`
4. otherwise → `VerificationRowReplaced { index }`

`RV-346` `F-17` is why the classification is ordered and why case 2 exists. The
earlier statement tested for a prefix and then fell through to `Replaced`, and
called an insertion `Reordered` in the prose — but inserting a row before or
between base rows leaves the base rows in their original *relative* order, so
neither the reordering test nor the replacement test describes what happened.
Three predicates over the same pair of sequences were producing two
contradictory answers. Prefix and subsequence are now distinct checks in a fixed
order: prefix decides *acceptance*, subsequence decides *which refusal*.

The refusal is the same in every case — anything the refinement adds must come
**after** the base sequence, so only a strict prefix match is accepted. The
diagnosis exists to tell the operator which edit to undo, and it is worth
getting right for that reason alone rather than for a security one.

### Invariants

1. **Monotonicity.** For any `Ok(result) = restrict(base, r)`:
   `result.forbidden_executables ⊇ base.forbidden_executables`,
   `result.interpreted_paths ⊇ base.interpreted_paths`, and
   `base.verification` is a prefix of `result.verification`. There is no input
   for which `restrict` returns a policy weaker than `base` on any axis.
2. **Identity.** `restrict(p, p) == Ok(p)` for every valid `p`.
3. **Normalization is a type property.** Every `InterpretationPolicy` in
   existence came out of `parse`, so `canonical_hash` and `restrict` never see
   an unsorted list or an unvalidated entry.
4. **Hash injectivity over the typed value.** Two policies with the same
   canonical hash are equal as typed values. (Modulo SHA-256, which is the
   assumption every content-addressed part of the corpus already makes —
   `ASM-009`.)
5. **Read-once.** The only call site that supplies `text` from a repository is
   `sec-3` step 4, reading a blob at the contracted base OID. Nothing in this
   slice reads `.doctrine/doctrine.toml` from a capsule checkout or a harvested
   tree.

### The forbidden list has a consumer in this slice

`trusted_side_forbidden_executables` would otherwise be a validated field nothing
reads until the launch slice builds a transaction plan. It is not: `sec-2`'s
closure resolver is an external-command step the trusted side runs outside a
capsule, and `sec-3` step 6 refuses when its normalized basename appears in the
list — `SPEC-030`'s rule, applied to this slice's only such step.

The ordering matters and is stated in `sec-3`: the check runs **after** the
refinement is applied, so a phase contract that adds a forbidden entry binds the
transaction it was supplied for. A check against the base policy alone would let
a refinement forbid an executable that this very provisioning had already run.

### Verification alignment

`REQ-449`'s refusal cases are `VT` tests over the real parser, one per row of
the validation table plus the block/key/schema cases:

- `missing_interpretation_block_refuses`
- `missing_required_key_refuses_naming_the_key`
- `unknown_key_refuses_naming_the_key`
- `unsupported_schema_version_refuses_naming_the_version`
- `empty_verification_sequence_refuses`
- `unknown_key_inside_a_verification_row_refuses_naming_the_row_and_the_key`
- `verification_row_missing_argv_refuses_naming_the_row`
- `verification_row_that_is_not_a_table_refuses`
- `argv_that_is_not_an_array_of_strings_refuses`
- `explicitly_empty_list_is_accepted_and_is_not_the_same_as_omission`
- `executable_with_a_slash_refuses` / `..._with_whitespace_...` /
  `..._that_is_dot_or_dotdot_...` / `..._that_is_empty_...`
- `absolute_path_pattern_refuses` / `backslash_...` / `nul_...` /
  `lexical_dotdot_component_...`
- `duplicate_forbidden_executable_refuses` /
  `duplicate_interpreted_path_refuses`
- `empty_argv_row_refuses` / `empty_argument_refuses`

Normalization and hash:

- `set_valued_lists_sort_by_raw_utf8_bytes`
- `verification_row_and_argument_order_are_preserved`
- `canonical_hash_is_stable_across_key_order_and_whitespace`
- `canonical_hash_distinguishes_split_boundaries_in_adjacent_entries` — the
  `["ab","c"]` versus `["a","bc"]` case, which is what the length prefixes buy
- `canonical_hash_is_not_computed_over_source_text` (asserted by reformatting the
  document and comparing hashes)

Restriction, one per refusal plus the two invariants:

- `refinement_may_add_forbidden_entries`
- `refinement_may_append_verification_rows`
- `refinement_removing_a_forbidden_entry_refuses`
- `refinement_removing_an_interpreted_path_refuses`
- `refinement_removing_a_project_verification_row_refuses`
- `refinement_reordering_project_verification_refuses`
- `refinement_replacing_a_project_verification_row_refuses`
- `refinement_inserting_a_row_before_a_project_row_refuses_as_inserted`
- `refinement_inserting_a_row_between_project_rows_refuses_as_inserted`
- `refinement_swapping_two_project_rows_refuses_as_reordered`
- `the_diagnosis_is_classified_in_the_stated_order` — a refinement that both
  removes a row and reorders the rest reports `Removed`, not `Reordered`
- `refinement_with_a_different_schema_refuses`
- `restrict_is_identity_on_its_own_base`
- property: `restrict_never_weakens_any_axis` over generated policy pairs

Read-once (`REQ-449` criterion 3), which is an executed test in `sec-7`'s
environment rather than a pure one:

- `rewriting_doctrine_toml_inside_a_capsule_does_not_change_the_bound_policy`

<!-- doctrine:section sec-5 -->
## Operational configuration: the `[capsule]` table, the capsule root, and capacity

`REQ-461` asks provisioning to compare available space against a configurable
expected capsule size, warn conspicuously below a threshold, halt on exhaustion,
and to do none of pre-reservation, backpressure, eviction or rescue-archive
construction. `DEC-158` settles the mechanism (`statvfs`), the home (a new
`[capsule]` table), the two tiers, and the capsule root's default.

This section builds that table in full. It is the one place the operator's
configuration enters this slice: `sec-2`'s two declared readable lists and its
closure resolver are read here, `sec-3`'s `ResourceBounds` are read here, and
`sec-3` step 1 and step 3 are this section's callers.

### Current behaviour

Doctrine reads two operational tables from `.doctrine/doctrine.toml` today, both
through `dtoml`'s tolerant text read (`src/dtoml.rs:87`) and both by serde
projection out of an otherwise-ignored document:

| table | reader | shape |
|---|---|---|
| `[dispatch]` | `src/dispatch_config.rs:51` | `#[serde(rename_all = "kebab-case", default)]` |
| `[reservation]` | `src/reserve.rs:63-92` | tolerant outer `ReservationDoc`, pure `parse_*(text)`, thin loader |

Neither is a precedent for *capacity*, because nothing in Doctrine has ever
looked at free space. The spike does not either: `capsule_disk`
(`lib/measure.sh:58`) is a `du` of a finished tree, and there is no `df`,
`statvfs` or equivalent anywhere in the rig. `REQ-461` asks for a check *before*
provisioning, against a number the project supplies, which is net-new.

What the spike does supply is measurements, and they are the only real data this
design has for sizing a default (`R1` altitude — one host, two fixtures):

- a Node-shaped capsule was bounded at 256 MiB and 300s and did not overrun;
- a Rust workspace peaked at **4.4 GiB and 352s**, overran both, and was
  re-bounded to 8 GiB and 900s "with headroom over the measurement, not to it"
  (`lib/fixtures.sh:70-92`);
- the kill grace between `SIGTERM` and `SIGKILL` was 5s
  (`capsule/sandbox.sh:68`).

### Why a new table, and why kebab-case

`DEC-158` rules out both incumbent homes: `[interpretation]` is hashed into the
work contract, so an operational edit to a capacity threshold would move the
canonical hash; `[dispatch]` is the worktree arm's configuration, which this
slice sits beside rather than replaces.

The key spelling is **kebab-case**, which puts this table with `[dispatch]` and
`[reservation]` rather than with `[interpretation]`. `[interpretation]`'s
snake_case is not a house style — `SPEC-030` fixes those key spellings verbatim
(`trusted_side_forbidden_executables`, `interpreted_paths`), and a governed
contract vocabulary is not free to be renamed. `[capsule]` has no such
constraint and is operational configuration in exactly the sense the other two
are, so it follows the convention both of their readers already declare.

**This amends `sec-2`.** That section's sample was written snake_case
(`readable_roots`, `closure_roots`, `closure_resolver`); the keys are
`readable-roots`, `closure-roots` and `closure-resolver`, and `sec-2`'s sample
and prose are corrected to match. Named here rather than silently, because it
changes text a review round has already read.

### The table, in full

```toml
[capsule]
# Where capsules live. Absent ⇒ the platform data directory (below).
root = "/var/lib/doctrine/capsules"

# The readable inputs (sec-2). At least one of the two lists must be non-empty.
readable-roots = ["/bin/sh", "/usr/bin/env"]
closure-roots = [".doctrine/capsule-toolchain"]
closure-resolver = ["nix-store", "--query", "--requisites"]

# Capacity, advisory (REQ-461). Warn below expected × multiplier; refuse below
# expected.
expected-capsule-size-mib = 4096
capacity-warn-multiplier = 2

# The execution bounds sec-2's `Execution` carries, enforced trusted-side.
# The first two are REQUIRED — there is no measurement to default them from.
execution-timeout-seconds = 900
file-size-cap-mib = 512
execution-kill-grace-seconds = 5
```

| key | default | absent means |
|---|---|---|
| `root` | the platform data directory (below) | resolve it; refuse if nothing resolves |
| `readable-roots` | `[]` | nothing bound as declared |
| `closure-roots` | `[]` | no closure expansion |
| `closure-resolver` | none | no host capability declared |
| `expected-capsule-size-mib` | `4096` | the default sizing applies |
| `capacity-warn-multiplier` | `2` | warn below twice expected (`REQ-461` criterion 2) |
| `execution-timeout-seconds` | **required** | refuse, naming the key |
| `file-size-cap-mib` | **required** | refuse, naming the key |
| `execution-kill-grace-seconds` | `5` | the spike's measured grace (`sandbox.sh:68`) |

Sizes are configured in **mebibytes and seconds, spelled in the key**, rather
than as suffixed strings. A `"4GiB"` spelling would need a parser, a refusal
vocabulary for malformed units, and tests for both, to buy nothing this slice's
callers need; the unit in the key name is unambiguous to a human writing it and
costs no code. Both are converted once, at construction, into the typed
`ByteCount` and `Duration` the rest of the design uses, **through checked
arithmetic**: a mebibyte figure above `u64::MAX / 1_048_576` refuses with
`BoundOverflows { key }` rather than wrapping. `RV-346` `F-12` found that hole —
overflow was specified for the two multiplications downstream and not for the
conversion that feeds them, which left construction undefined at the top of the
range and every downstream saturation argument resting on an unchecked value.

**The two enforced bounds are required keys, because there is no measurement to
default them from.** This is `RV-346` `F-15`, and it lands. The spike used a
single `disk_cap` for *both* `ulimit -f` and its whole-tree `du`
(`capsule/sandbox.sh:287,313`), mapping either leg to one outcome, so its 256 MiB
figure measures neither the largest file a capsule writes nor which limit fired;
and the 300s figure is not merely unmeasured but *known to fail*, at 352s on a
Rust workspace. An optional default terminates work, so shipping either number
as a default would be presenting an unmeasured — in one case falsified — value
as generally usable. `execution-timeout-seconds` and `file-size-cap-mib` are
therefore required: absent, provisioning refuses naming the key. The sample above
shows the heavy fixture's re-bounded figures because those are the ones with
headroom over a real measurement, but they are a project's choice, not a
platform default. `execution-kill-grace-seconds` keeps its default of 5, which
*is* directly measured (`sandbox.sh:68`) and is a window between two signals
rather than a bound that ends work.

**The advisory default survives that argument, and is re-anchored by it.**
`expected-capsule-size-mib` denotes whole-tree size, which is exactly the
quantity the spike's 4.4 GiB peak measured — so unlike the per-file cap, this
default rests on a measurement of the right thing. It stays, because `REQ-461`
criterion 2 explicitly contemplates a default here and because the tier it feeds
warns rather than terminates. `4096` sits at the upper order of the two measured
points without asserting that a Rust workspace is the platform's shape;
`POL-002` facet 1 is why the number is not simply the 8192 this repository
should set for itself. The residual tension is worth naming: a Node project on a
laptop with 3 GiB free is refused by a default sized for something larger, and
the answer is the key rather than a cleverer default.

`capacity-warn-multiplier` is a plain integer **strictly greater than 1**.
`RV-346` `F-13`: at exactly 1 the warning region `expected ≤ available <
expected × multiplier` is empty, so no amount of free space can ever emit the
conspicuous warning `REQ-461` criterion 1 requires — the policy would be
accepted and the requirement silently unmet. A value below or equal to 1
refuses with `WarnMultiplierNotAboveOne { found }`, which is a stronger check
than the inversion argument alone would have produced.

### Reading it

```rust
/// A byte quantity. Defined here because this section owns the arithmetic that
/// can overflow; `sec-2`'s `file_size_cap` and `disk_used` are this type.
pub struct ByteCount(u64);

/// The validated `[capsule]` table.
///
/// Private fields and one fallible constructor, for `sec-2`'s reason: a public
/// literal would let a caller assemble a configuration that never went through
/// the readable-list rule or the root resolution.
pub struct CapsuleConfig {
    /// Absolute and resolved. `sec-2`'s `ForbiddenScopes::capsule_root`.
    root: PathBuf,
    readable_roots: Vec<PathBuf>,
    closure_roots: Vec<PathBuf>,
    closure_resolver: Option<Argv>,
    capacity: CapacityPolicy,
    bounds: ResourceBounds,
}

pub struct CapacityPolicy {
    expected_capsule_size: ByteCount,
    warn_multiplier: u32,
}

/// The resource choices `sec-3`'s transaction binds and `sec-2`'s `Execution`
/// carries.
pub struct ResourceBounds {
    timeout: Duration,
    kill_grace: Duration,
    file_size_cap: ByteCount,
}
```

```rust
/// PURE. Projects and validates everything that does not need the host.
pub fn parse_capsule_config(text: &str) -> Result<UnrootedCapsuleConfig, ConfigRefusal>;

/// Resolves the capsule root, which is the only part that needs the host.
pub fn root_capsule_config(
    parsed: UnrootedCapsuleConfig,
    host: &dyn HostFacts,
) -> Result<CapsuleConfig, ConfigRefusal>;
```

The split is the project's pure/imperative rule applied at the one place this
table touches the host: every list rule, every bound, and the multiplier are
decided from text alone and are testable without a filesystem; only the root
default reads the environment.

**Serde projection here, an explicit table walk in `sec-4`, and the difference
is not inconsistency.** `sec-4` walks `[interpretation]`'s keys by hand because
`REQ-449` criterion 1 demands six *distinguishable* refusals and serde would
hand back a formatted string for four of them. `[capsule]` has no such
requirement. What it does need is that a mistyped key be refused rather than
silently defaulted — `execution-timeout-secconds` quietly restoring a 300s bound
on a project that configured 900 is the hazard — and
`#[serde(deny_unknown_fields)]` supplies exactly that, naming the offending key
in its own message. So this reader is `src/reserve.rs:78-103`'s shape with
unknown keys denied, plus a validating constructor. Two readers, two
requirements, one of them met by a derive.

Refusals, each a named variant carrying the key:

| refusal | condition |
|---|---|
| `MalformedTable { detail }` | not valid TOML, wrong value type, or an unknown key |
| `NoReadableInputs` | `readable-roots` and `closure-roots` both empty (`sec-2`) |
| `ClosureRootsWithoutResolver` | `closure-roots` non-empty, `closure-resolver` absent |
| `EmptyResolverArgv` | `closure-resolver = []` |
| `RelativeCapsuleRoot { path }` | a configured `root` that is not absolute |
| `UnresolvableCapsuleRoot` | no `root`, and no platform data directory resolves |
| `ZeroBound { key }` | any bound or size configured as `0` |
| `BoundMissing { key }` | `execution-timeout-seconds` or `file-size-cap-mib` absent |
| `BoundOverflows { key }` | a mebibyte figure above `u64::MAX / 1_048_576` |
| `WarnMultiplierNotAboveOne { found }` | `capacity-warn-multiplier ≤ 1` |

`NoReadableInputs` and `ClosureRootsWithoutResolver` are `sec-2` invariant 9's
first three refusals, discharged here because this is where the lists are read.
The remaining probes in that invariant — an entry that does not exist, a
`closure-roots` entry that is not realised — need the filesystem and stay at
`sec-3` step 2.

### The capsule root

`DEC-158`: not `${HOME}/capsules`, which is the spike's host convention;
not the repository's runtime state tier, which is characterised as disposable
scratch and would contradict `DEC-133`/`DEC-137`'s holding that a harvested
capsule is live work. An owned path derived from the platform's data directory,
outside the repository, overridable by `root`, and where nothing resolves,
provisioning refuses naming the key rather than falling back.

```rust
/// The subdirectory of the platform data directory that Doctrine owns.
const CAPSULE_ROOT_LEAF: &str = "doctrine/capsules";
const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
```

Resolution order, through `HostFacts` so it is testable: `XDG_DATA_HOME` if set,
non-empty and **absolute**; else `HOME` under the same three conditions, joined
with `.local/share`; else refuse. Both reads are `var_os` — this repository bans
`std::env::var` through `disallowed_methods` (`src/tty.rs:41`), and
`src/install.rs:1818` is the precedent for the `HOME` read and its refusal
message.

**Both variables are held to the same test, and an earlier draft was not.**
`RV-346` `F-16`: the draft rejected a relative `XDG_DATA_HOME` and then joined
`HOME` unconditionally, so `HOME=relative` or `HOME=` produced a relative
capsule root — contradicting the type's own claim to be absolute and resolved,
defeating invariant 4, and in the empty case rooting capsules at
`.local/share/doctrine/capsules` **under the current working directory**, which
on an ordinary invocation is the repository. A relative default that lands
inside the repository is the precise outcome `DEC-158` rules out, arrived at
through a variable nobody validated. An unusable value in either variable is
skipped rather than joined, and the arm falls through to the refusal.

Two things this deliberately is not. It is **not a new dependency**: a
platform-directory crate would buy macOS and Windows arms for a slice whose
backend is Linux, against a `Cargo.toml` whose `rustix` note records "ZERO new
compiled crates" as the standing posture. And it is **not a fallback chain that
ends in a guess** — the last arm refuses. A capsule root silently landing in a
temporary directory is how large live work ends up somewhere a reboot removes.

The resolved root is what populates `sec-2`'s `ForbiddenScopes::capsule_root`,
and `sec-2`'s overlap test is **bidirectional**: a placement path that is equal
to, above, *or below* the capsule root is refused, with one typed carve-out for
writable entries under the placement's own `TransactionRoot`. A sibling
transaction is below the capsule root and not below this root, so it is refused
by the same test that admits `capsule/` and `agent/`.

An earlier draft of this paragraph argued the descendant case away — that
provisioning computes every path in a placement and computes only its own, so
`sec-2` invariant 1 kept siblings out. `RV-346` `F-10` refuted it, and the
refutation is worth keeping visible because the reasoning was the appealing
kind: `readable-roots` entries and closure-resolver output are also computed by
provisioning in that sense, and they can name any path at all. Provenance was
doing no work. What denies a sibling is geometry, checked in both directions,
which is now `sec-2`'s rule.

### `HostFacts`

The impure inputs provisioning is given, named as one contract so every pure
step above it is testable against a fixture rather than a filesystem:

```rust
pub trait HostFacts {
    /// Available bytes at `path`, or why the probe could not answer.
    fn available_bytes(&self, path: &Path) -> Result<ByteCount, CapacityUnknown>;
    /// Fully resolve a path through its symlink components (sec-2's rule 1).
    fn resolve(&self, path: &Path) -> io::Result<PathBuf>;
    fn path_exists(&self, path: &Path) -> bool;
    fn env_var(&self, name: &str) -> Option<OsString>;
}
```

**It carries no clock**, which corrects the sketch in `sec-3`'s signature
comment. Provisioning needs no wall-clock read — the transaction id comes from a
collision-resistant source, not a timestamp — and `src/clock.rs` is already this
project's single home for wall-clock reads, with the pure/imperative rule that
the value is passed in rather than a clock handed down. A `now()` here would be
a second one.

`SystemHost` is the production implementation. A fixture implementation in tests
supplies available bytes, resolutions and environment values from a table, which
is what makes the tier boundaries below assertable without a disk of a
particular size.

### Capacity

```rust
pub enum CapacityVerdict {
    /// At or above expected × multiplier.
    Ample { available: ByteCount },
    /// At or above expected, below expected × multiplier — warn and continue.
    Low { available: ByteCount, expected: ByteCount, threshold: ByteCount },
    /// Below expected — refuse.
    Insufficient { available: ByteCount, expected: ByteCount },
    /// The probe could not produce a usable figure — report and continue.
    Unknown { reason: CapacityUnknown },
}

/// PURE, given the probe's answer.
pub fn assess_capacity(
    probe: Result<ByteCount, CapacityUnknown>,
    policy: &CapacityPolicy,
) -> CapacityVerdict;
```

**The probe.** `rustix::fs::statvfs(path)`
(`rustix-1.1.4/src/fs/abs.rs:288`), available bytes as `f_bavail × f_frsize`.
`rustix` is already a direct dependency for `flock`
(`src/worktree/claim_lock.rs`), at `default-features = false, features =
["fs"]`, so the probe adds no compiled weight. `f_bavail` is the count available
to an unprivileged process, which is the honest figure — `f_bfree` includes the
reserved blocks a capsule cannot have. `f_frsize` is the fragment size and is
the unit `f_bavail` counts in; `f_bsize` is the preferred I/O size and is the
wrong multiplier.

**The probe runs against the capsule root, not the repository.** They are
different filesystems on any host that separates `/home` from `/var`, and the
figure that matters is where the transaction root will be written. On a root
that does not exist yet, the probe runs against its nearest existing ancestor;
a root whose ancestors do not exist either is a configuration error caught at
`sec-3` step 1.

**`CapacityUnknown` is a named outcome, not an error swallowed into ample.**
`POL-002` facet 3 lands here (`DEC-158`): filesystems are diverse and not every
field is meaningful on every one. Three constructors —
`ProbeFailed { errno }` when `statvfs` itself fails, `UnusableFigure` when
`f_frsize` is zero, and `FigureOverflows` when the product exceeds `u64`.
Capacity is advisory under `REQ-461`, so unknown does not refuse; it is
reported, and it is *distinguishable in the report* from a probe that returned
an ample figure, because those two are the same to a silent implementation and
opposite to an operator.

**The tiers, exactly.** With `expected = expected-capsule-size-mib × 1 MiB` and
`threshold = expected × capacity-warn-multiplier`, saturating at `u64::MAX`
rather than overflowing:

| available | verdict | provisioning |
|---|---|---|
| `≥ threshold` | `Ample` | continues silently |
| `expected ≤ a < threshold` | `Low` | warns conspicuously, continues |
| `< expected` | `Insufficient` | refuses at `sec-3` step 3 |
| unusable probe | `Unknown` | reports, continues |

`REQ-461` criterion 2's "may warn below twice the expected capsule size without
reserving that space" is the default multiplier and the whole of the reservation
story: nothing is written, claimed or held at `Low`. The next capsule provisioned
sees the same free space this one did.

**The warning is structured, not a sentence.** `REQ-461` criterion 1 says
conspicuous; a formatted line is neither machine-readable nor greppable by an
operator triaging a stalled queue. It carries named fields — `available_bytes`,
`expected_bytes`, `threshold_bytes`, `capsule_root`, and the config key to
change — through the same reporting the refusals use, so a warning and a refusal
about the same condition differ in severity and not in vocabulary.

**Exhaustion mid-provision halts identically.** A write that fails `ENOSPC`
after the probe passed is the same condition observed later, and it produces the
same refusal class rather than a distinct one an operator has to learn: the
probe is advisory precisely because it can be overtaken. What follows is
`sec-3`'s rollback of the transaction root this call created, on the creation
token that call holds — which matters most here, since a partial tree left
behind is on a disk that was already short.

### What V0 does not do

`REQ-461`'s negative half is a requirement, not an omission, and each of its
four names a mechanism that would be visible in this design if it were present:

- **no pre-reservation** — nothing is written or claimed at `Low`, and
  `CapacityPolicy` holds no reserved figure;
- **no throughput backpressure** — `assess_capacity` is a function of one
  probe and one policy; it does not know how many capsules exist, and nothing
  queues or delays on its verdict;
- **no automatic eviction** — there is no delete capability in this slice at
  all beyond `sec-3`'s token-guarded rollback of a directory that call created
  before it held any work (`DEC-156`, `DEC-133`, `DEC-137`);
- **no rescue-archive construction** — nothing is copied or compressed anywhere
  on the refusal path.

The whole-tree disk figure is **observed and not capped**. `sec-2`'s
`Observation.disk_used` is computed trusted-side after a run, as the spike's is
(`capsule/sandbox.sh:311-317`); the enforced bound is `file_size_cap`, per file,
through `RLIMIT_FSIZE`. Capping a tree needs either polling or a filesystem
quota, and both are the backpressure `REQ-461` scopes out of V0.

### Invariants

1. **Capacity never deletes.** No verdict, and no refusal path in this section,
   removes anything. The single removal in provisioning is `sec-3`'s rollback,
   guarded by a creation token.
2. **An unusable probe is reported, never read as ample.** `Unknown` is a
   distinct verdict with a reason, and it is distinguishable in the report from
   `Ample`.
3. **The configuration cannot silently default.** An unknown key refuses. There
   is no arrangement of missing configuration that produces a capsule root
   inside the repository, an empty readable set, or a bound of zero.
4. **The capsule root is absolute, resolved, and forbidden to capsules in both
   directions.** It is `ForbiddenScopes::capsule_root`, so `sec-2`'s validating
   constructor refuses any placement path that contains it *or lies under it*,
   except writable entries under the placement's own transaction root. No
   environment value can produce a relative root, and none can produce one
   inside the repository.
5. **Capacity is advisory in one direction only.** It may refuse before work
   starts; it may never permit what another rule refuses, and it is not consulted
   after `sec-3` step 3.

### Verification alignment

Pure, against a fixture `HostFacts` — the tiers are arithmetic and need no disk:

- `available_at_the_threshold_is_ample_and_one_byte_below_it_is_low`
- `available_at_expected_is_low_and_one_byte_below_it_is_insufficient`
- `insufficient_refuses_and_names_the_capsule_root_and_the_config_key`
- `low_warns_with_named_fields_and_provisioning_continues`
- `low_reserves_nothing_so_a_second_assessment_sees_the_same_figure`
- `unknown_is_reported_and_is_not_ample`
- `unknown_carries_which_of_the_three_reasons_produced_it`
- `threshold_saturates_rather_than_overflowing_on_a_large_multiplier`

Configuration:

- `unknown_key_refuses_naming_the_key`
- `both_readable_lists_empty_refuses`
- `closure_roots_without_a_resolver_refuses_naming_both_keys`
- `empty_resolver_argv_refuses`
- `relative_configured_root_refuses`
- `zero_valued_bound_refuses_naming_the_key`
- `absent_execution_timeout_refuses_naming_the_key`
- `absent_file_size_cap_refuses_naming_the_key`
- `a_mebibyte_figure_above_the_conversion_ceiling_refuses_naming_the_key`
- `the_largest_convertible_mebibyte_figure_is_accepted` — the boundary either
  side, so the check is not off by one
- `warn_multiplier_of_one_refuses_because_the_warning_region_would_be_empty`
- `warn_multiplier_of_zero_refuses`
- `absent_optional_keys_take_the_named_default_constants` — grace and the two
  capacity keys only; the enforced bounds have no default to take
- `sizes_in_the_key_unit_convert_once_into_bytes_and_seconds`

Capsule root resolution:

- `configured_root_wins_over_the_environment`
- `xdg_data_home_is_used_when_set_and_absolute`
- `home_supplies_the_default_when_xdg_data_home_is_unset`
- `relative_xdg_data_home_is_ignored_rather_than_joined`
- `relative_home_is_ignored_rather_than_joined`
- `empty_home_is_ignored_rather_than_joined` — the case that would otherwise
  root capsules under the working directory
- `no_resolution_path_yields_a_relative_root` — asserted over every combination
  of the two variables
- `neither_variable_usable_refuses_naming_the_config_key`
- `the_resolved_root_is_the_forbidden_scope_a_placement_is_validated_against`
- `a_sibling_transaction_under_the_resolved_root_is_refused_by_the_placement`

Executed, in `sec-7`: nothing in this section is a confinement property, so the
suite carries no capacity row. The one executed claim it owes is that the probe
reads the filesystem the capsule root is on rather than the repository's — a
single test that runs only where the two differ, and reports skipped rather than
passed where they do not.

<!-- doctrine:section sec-6 -->
## Crate topology, the export set, and layering enforcement

`DEC-153` settles that this code lands in a second binary, `doctrine-control`,
in the same workspace, with the split line at canonical mutation and nothing
migrating out of the existing binary. `DEC-160` settles that two verbs hang off
it, built but not released. This section builds the crate, classifies its
modules under `ADR-001`, specifies what the root package must export for the new
crate to reach the two pieces it needs, and rules on the enforcement gap that
crossing opens.

### Current behaviour

```
doctrine/                     # the workspace
  Cargo.toml                  # [workspace] members = [".", "crates/cordage"]
  src/main.rs                 # 94 `mod` declarations — the whole product
  crates/cordage/             # a zero-dependency leaf crate; doctrine → cordage
  tests/architecture_layering.rs
  .doctrine/adr/001/layering.toml
```

Three facts about it decide most of this section.

1. **The root package has no lib target.** `src/lib.rs` does not exist; the
   package builds one binary from `src/main.rs`, which declares all 94 modules.
   Nothing outside the package can reach any of them today.
2. **`cordage` is the precedent, and it points the other way.** It is a
   product-neutral leaf that `doctrine` depends on. `doctrine-control` is the
   reverse direction — it needs two things the root package already has — so
   `cordage` is a precedent for the workspace split and not for the dependency.
3. **The layering gate walks one tree.** `discover_units` and `extract_edges`
   (`tests/architecture_layering.rs:40,123`) are already parameterised by a
   source directory, but the gate calls them on `Path::new("src")`
   (`:1148`), and `check`'s module filter hardcodes the same path (`:558`).

### The new crate

```
crates/doctrine-control/
  Cargo.toml                  # depends on the doctrine lib target, path
  src/main.rs                 # the two verbs
  src/host.rs                 # HostFacts, SystemHost                    (sec-5)
  src/config.rs               # CapsuleConfig, the [capsule] reader      (sec-5)
  src/capacity.rs             # CapacityVerdict, assess_capacity         (sec-5)
  src/backend.rs              # CapsuleBackend, CapsulePlacement,
                              #   Execution, Observation, Termination,
                              #   ForbiddenScopes                        (sec-2)
  src/backend/bubblewrap.rs   # the Linux profile and its argv           (sec-2)
  src/transaction.rs          # CapsuleTransaction, TransactionId,
                              #   AcceptedBase, PhaseIdentity            (sec-3)
  src/provision.rs            # provision, export publication, root
                              #   ownership, rollback                    (sec-3)
  src/conformance.rs          # the eight-property table                 (sec-7)
```

`ADR-001`'s rule applies inside the new crate exactly as it does inside the root
package — tier is the highest altitude of any non-test file, and there are no
cycles:

| unit | tier | out-edges |
|---|---|---|
| `host` | leaf | none |
| `config` | leaf | `host` |
| `capacity` | leaf | `config`, `host` |
| `backend` | leaf | `config` |
| `transaction` | engine | `backend`, `config` |
| `provision` | engine | `transaction`, `backend`, `capacity`, `config`, `host` |
| `conformance` | engine | `provision`, `backend` |
| `main` | command | all of the above |

`backend` is leaf despite executing processes, because the tier rule classifies
by what a unit imports and not by whether it is pure — `git` is leaf in the root
package on the same reading. The pure/imperative split is a separate obligation,
discharged inside each unit: `sec-3` names which of provisioning's thirteen
steps are pure given `HostFacts`, and `sec-5` splits its reader into a pure
parse and a host-reading resolution.

**`src/worktree/` is absent from that table and stays absent.** `sec-2`
invariant 10 asserts it, and this is where it is enforced: `doctrine-control`
does not depend on the root package's worktree modules because the export set
below does not carry them, and it cannot reach around the export set.

### What the root package must export, and what that costs

`doctrine-control` needs exactly two things from the root package:

1. `git::read_path_at(root, refish, path)` (`src/git.rs:790`) — the
   working-tree-free blob read `REQ-449`'s resolution runs on, verified this
   session to be the whole impure surface that resolution needs;
2. the `[interpretation]` policy module `sec-4` builds, which lives in the root
   package rather than here because both binaries will need it at cutover.

It also needs the project config file's location and a raw read of it, for
`sec-5`'s `[capsule]` table.

**A lib target on the root package, with a curated export list.**

```rust
// src/lib.rs — the entire public surface of the doctrine library.
mod config_file;
mod git;
mod kinds;

pub mod interpretation;

pub use config_file::{DOCTRINE_TOML, read_doctrine_toml_text};
pub use git::{CaptureError, read_path_at};
```

Four consequences, none of them free, all of them small:

The three claims below were **checked by execution**, not by reading, on a
minimal package reproducing the arrangement — the `RV-346` `F-1` discipline
applied to a build-level claim, because each of these would otherwise be
discovered by whoever implements the phase.

- **The exported items change visibility, and the compiler requires it.**
  `read_path_at`, `CaptureError`, `DOCTRINE_TOML` and `read_doctrine_toml_text`
  are `pub(crate)` today and become `pub`. Nothing else does. This is not a
  stylistic choice: `pub use` of a `pub(crate)` item is `E0364` — *"only public
  within the crate, and cannot be re-exported outside"* — confirmed by
  execution. The list *is* the export contract, and the gate below asserts it.
- **`main.rs` keeps its own module tree.** It continues to declare
  `mod git;` rather than importing `doctrine::git`, so the modules the lib
  target names compile twice. A package declaring the same module in both its
  bin and its lib target builds cleanly — confirmed by execution — and the two
  copies never meet, because the binary touches no library type. The
  alternative, making `main.rs` a thin binary over the library, would require
  every `pub(crate)` item the command layer reaches to become `pub` (110 in
  `git.rs` alone, 203 in `memory.rs`), which publishes most of the product as
  library API to buy a build-time saving. The double compilation is the cheaper
  of the two, and it is bounded by the module list above.
- **The set must be transitively closed over `crate::` paths, including test
  code.** `git` is out-edge-free in the production graph, but its `#[cfg(test)]`
  module imports `crate::kinds` (`src/git.rs:2896`), and the lib target compiles
  that module under `cargo test`. Omitting `kinds` from `lib.rs` fails the
  library's own test build with `E0432: unresolved import crate::kinds` —
  confirmed by execution, and cleared by declaring the module. `kinds` is
  therefore declared privately, as a path target rather than an export. This is
  the trap in the whole arrangement: it is invisible to a production-graph
  reading, `cargo build` is green while it is present, and it surfaces only when
  the library's tests first run.
- **`dtoml` cannot be exported, and does not need to be.** It carries seventeen
  `crate::` references — `conduct`, `verify`, `estimate`, `value`,
  `dispatch_config`, `install_config` — because `DoctrineToml` projects each of
  their tables, and `verify` reaches `coverage`, which is engine-tier. Declaring
  it would pull the cascade into a library whose export set is meant to be leaf
  only.

**The config file's location and raw read move to their own leaf module.**
`DOCTRINE_TOML` and `read_doctrine_toml_text` sit in `dtoml` today and are used
35 times, so relocating them naively would touch 33 files. `src/config_file.rs`
takes both, and `dtoml` re-exports them under their existing names:

```rust
// src/dtoml.rs
pub(crate) use crate::config_file::{DOCTRINE_TOML, read_doctrine_toml_text};
```

Every existing `dtoml::DOCTRINE_TOML` call site is untouched, `STD-001`'s single
source is preserved rather than duplicated, and the new module is out-edge-free
so it can be exported. The split is also honest about what `dtoml` already is:
its own documentation describes `read_doctrine_toml_text` as the shared file
read used by consumers that project their own table out-of-band of
`DoctrineToml` — two altitudes in one module, and only the lower one crosses
the crate boundary.

### `layering.toml`

Four rows are added to the root package's map — `config_file = "leaf"` (out=0),
and `interpretation = "leaf"` (out=0) from `sec-4` — plus one edge, `dtoml →
config_file`, leaf to leaf. The new crate's eight units are recorded in their
own section rather than merged into the root map, because unit names are only
unique within a tree and merging them would make `config` ambiguous.

### The enforcement gap, and the ruling

Crossing a crate boundary steps outside what the existing gate can see, in two
independent ways:

1. **The second tree is never walked.** The gate runs on `src` alone, so
   `doctrine-control`'s eight units are unclassified and its internal edges
   unchecked.
2. **A cross-crate import is not a `crate::` path.** `CratePathCollector`
   (`tests/architecture_layering.rs:216`) accumulates the first component after
   `crate`, so `doctrine::interpretation::parse` produces no edge at all. Even
   walking both trees would leave every edge from `doctrine-control` into the
   root package invisible.

The two gaps take different answers, and the design's position is that
generalising the extractor to cross crates is the wrong one.

**For the cross-crate direction: assert the export list, not the edges.** The
public surface of the root library is the only route from `doctrine-control`
into the root package. Bounding that list bounds every cross-crate edge exactly,
with no graph analysis at all:

```rust
/// The root library's entire public export set (STD-001).
const EXPORTED: &[&str] = &[
    "interpretation", "DOCTRINE_TOML", "read_doctrine_toml_text",
    "read_path_at", "CaptureError",
];
```

A test asserts that the public items reachable from `doctrine`'s crate root are
exactly this set, and that every module contributing one is classified `leaf` in
`layering.toml`. This is stronger than an edge check — an edge check would
permit `doctrine-control` to reach any exported engine-tier item, and this
refuses the export in the first place — and it doubles as protection against
the published crate's API widening by accident. `interpretation` is exported as
a whole module because it is purpose-built and out-edge-free; the other four are
individual items behind private modules.

**For the intra-crate direction: run the existing gate a second time.**
`discover_units` and `extract_edges` already take a source directory. The one
change is `check`'s module filter (`:558`), which builds its on-disk existence
probe from a hardcoded `"src"` and must take the same directory as a parameter.
The gate then runs twice — once per tree, each against its own `[tiers]`
section — and `doctrine-control`'s eight units are enforced by the same
machinery, at the cost of one parameter and one call.

Naming this rather than letting it pass is `sec-1`'s obligation under
`ADR-001` discharged: the design confronts the place the existing gate cannot
see instead of inheriting an assertion that stopped being true when a second
crate appeared.

### The two verbs

`DEC-160`: `doctrine-control` exposes `provision` and `backend verify`, built
but not released.

- **`provision`** consumes contract resolution, the refinement algebra,
  capacity, layout and the backend — `sec-3`'s `provision` behind a CLI.
  A transaction `show` verb is deliberately omitted: nothing operates a
  transaction yet, and the tests inspect the returned value directly.
- **`backend verify`** runs `sec-7`'s conformance suite for an on-host verdict,
  exiting nonzero with structured output when the backend is not admitted. It
  is also `POL-002` facet 3's descriptive absence path for bubblewrap: absent,
  the verdict names what was missing and what would satisfy it, rather than the
  suite skipping green.

**Nothing ships.** The distribution contract — the nix `srcWithDist` graft, the
binstall asset name, `install.sh`, `release.yml` — is a close-time Follow-Up for
whichever slice first releases the binary, recorded as risk `R5` and not
discharged here. The binary is reachable from the build tree only, which is what
the scope means by tested machinery sitting beside the incumbent arms, unused.
`POL-002` is why that Follow-Up is named rather than done: the four artefacts
are this project's own release arrangement, and the platform does not acquire a
release step because one of its slices produced a second binary.

### Invariants

1. **The export list is the whole crossing.** `doctrine-control` reaches the
   root package through the items in `src/lib.rs` and through nothing else;
   every one of them is leaf-tier.
2. **Nothing migrates.** No existing verb, module or behaviour moves out of the
   root binary. The only changes to existing code are four visibility
   promotions, one module relocation behind re-exports, and one test parameter.
3. **Both trees are gated.** Every unit in both source trees is classified in
   `layering.toml` and checked by the same gate, and an unclassified unit fails
   it.
4. **`doctrine-control` is not mounted into a capsule.** `DEC-153` places it on
   the control host only; `sec-2`'s readable set is declared configuration, and
   nothing in this design adds the control binary to it.

### Verification alignment

- `the_root_library_exports_exactly_the_named_set`
- `every_exported_item_belongs_to_a_leaf_tier_module`
- `the_layering_gate_runs_over_both_source_trees`
- `an_unclassified_unit_in_the_new_crate_fails_the_gate`
- `a_command_tier_import_from_an_engine_unit_in_the_new_crate_fails_the_gate`
- `doctrine_control_does_not_depend_on_the_worktree_modules` — asserted through
  the export set, which carries no worktree surface at all
- `the_existing_layering_gate_is_unchanged_in_verdict_over_the_root_tree` — the
  behaviour-preservation obligation for a change to shared machinery: the
  parameterisation must not move any existing classification

The relocation of `DOCTRINE_TOML` and `read_doctrine_toml_text` is covered by
the existing suites unchanged, which is the point of doing it behind
re-exports: if any of the 35 call sites changes behaviour, tests that were
written for something else fail.

