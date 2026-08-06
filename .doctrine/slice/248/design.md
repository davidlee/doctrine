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
  the two typed carve-outs below;
- a `source` whose host path does not descend from `<capsule_root>/export/`, or
  whose carried base identity is not the placement's accepted base;
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

**Both carve-outs are typed rather than described**, and neither is a carve-out
in the `readable`/`writable` vectors at all — each is a distinct field whose type
can only have been minted by the provisioning step that exclusively created the
directory it names. That is what makes them safe to admit: the vectors stay
carve-out-free, so no declared entry of any kind is ever admitted beneath a
forbidden scope.

*The transaction's own writable state.* `TransactionRoot` is minted by `sec-3`
step 9. `try_new` accepts a writable entry that is a descendant of *this
placement's* `root` and refuses every other overlap with `capsule_root`. So the
capsule area is closed to a placement except through the one transaction root
the placement carries, and a sibling transaction — a descendant of
`capsule_root` but not of this `root` — is refused by the same test that admits
`capsule/` and `agent/`.

*The source export.* `SourceExport` is minted by `sec-3` step 8, by the publish
-or-adopt protocol, and carries the base OID it is an export **of**. `try_new`
accepts the `source` field when it descends from `<capsule_root>/export/` and
its carried identity equals the placement's accepted base; every other value
refuses. So a sibling export — an export of a *different* base, which is a
descendant of `capsule_root` and a perfectly well-typed `SourceExport` — is
refused here, and not only later at `sec-3` invariant 3.

**This second rule is `RV-346` `F-25`, and the class it belongs to is worth more
than the rule.** The design's own layout puts the export at
`<capsule_root>/export/<base-oid>`; `capsule_root` is a `ForbiddenScopes` member;
overlap is bidirectional. So the validator refused the design's only lawful
source placement, and `sec-7` row 9's fixture — which builds its own per-run
export — could not have been constructed either. Every conformance row would
have failed before running, for a reason that has nothing to do with any
property under test.

It is the over-denial mirror of `F-10`, which was the under-denial defect in the
same rule, and the two together are the general lesson: **a refusal rule is half
a specification until the lawful case is asserted positively.** `F-10` was found
because nothing proved the rule denied enough. `F-25` was possible because
nothing proved it admitted enough. Only one of those two failure directions had
a test.

So the discipline, stated once and applied to every refusal rule in this
section: **each refusal is paired with a positive case that the refusal must not
capture**, and the pair is what gets written, never the refusal alone. The
placement tests below are ordered to make the pairing visible. The one that
already existed —
`a_writable_entry_under_this_placements_own_transaction_root_is_admitted`,
annotated *asserted positively so a fix that refuses everything under the capsule
root cannot pass* — had exactly the right instinct, and `F-25` is what it costs
to have that instinct once rather than as a rule.

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

The contract's meaning is the properties `sec-7` proves. They are named here
because the trait's shape is answerable to them, and `sec-7` is where each
acquires a probe, a control and a decoy.

**The count is stated in exactly one place — `sec-7`'s Table A — and no other
section restates it as a numeral.** That is not fastidiousness; it is the fix
for a defect this design has now had twice. The count has moved twice under
external review, and both times the correction reached the sections that were
being reviewed and missed the ones that were not: `RV-346` `F-28` found `sec-2`
still instructing eight properties, still asserting one-to-one correspondence,
and still reporting `DEC-156` as *corrected to eight*, three rounds after the
first correction and one after the second. Those were not historical framing —
they were current implementation instructions contradicting `sec-7`. A numeral
repeated across five sections is a constant with five definitions (`STD-001`),
so the numeral now lives where the rows do.

| # | property | what the contract owes it |
|---|---|---|
| 1 | fresh mutable state | `CapsulePlacement.root`, distinct per transaction |
| 2 | bounded input set | `readable` is a declared list; there is no implicit floor |
| 3 | denial of canonical state and credentials | absence from `readable`/`writable`, not read-only presence |
| 4 | bounded host filesystem visibility | the same absence, generalised to everything undeclared |
| 5 | explicit network posture | `NetworkPosture`, `Denied` by default |
| 6 | deterministic working directory | `working_directory`, with no inherit value |
| 7 | process-tree teardown | that no descendant outlives the `execute` call, observed by the parent |
| 8 | trusted observation of resource limits and termination | `timeout`, `file_size_cap`, and every `Termination` variant being correctly distinguished |
| 9 | immutable input set | that a declared readable path cannot be written through |
| 10 | closed descriptor set | that no open file descriptor crosses `execute` except the ones the contract deliberately owns |
| 11 | closed environment | that the capsule's environment is exactly `CapsuleEnv`, with nothing inherited from the trusted-side process |

#### The rows are derived from channels, not from clauses

This is the correction `RV-346` `F-26` forced, and it is the one worth reading
before the arithmetic below.

`SPEC-030` § Platform backend contract states eight clauses, and the natural way
to build a conformance suite is one row per clause. That is what this design did,
and it is **wrong in a way that is invisible from inside the suite**. A clause is
a requirement written in the vocabulary of what the capsule must not *have*. A
row has to be written in the vocabulary of the mechanism by which a capsule
*gets* it. Those two vocabularies are not in correspondence, and where a clause
names an outcome reachable by more than one mechanism, one row per clause leaves
every mechanism but one unproven.

Clause 2 — *an explicit base and input set* — is the whole of the problem. It
says nothing about the channels by which input arrives, so a row written against
it proves whichever channel its author happened to picture. Rows 2, 9, 10 and 11
are all clause 2, and each was found by someone constructing the backend the
existing rows would wrongly pass rather than by re-reading them.

So the design keeps a **channel ledger**: the closed enumeration of ways
authority crosses `execute`, each naming the row that proves it. A channel with
no row is then a visible hole rather than an absent thought — which is the only
structural defence available, because nothing inside a suite can notice a
mechanism nobody wrote down.

| channel | how authority would cross | proven by |
|---|---|---|
| the mount set — presence | a path the capsule was not given is readable | rows 3, 4 |
| the mount set — extent | a path it *was* given reaches further than declared | row 2 |
| the mount set — mutability | a path it was given read-only can be written | row 9 |
| open file descriptors | one inherited across `execute` is already-open authority no mount test sees | row 10 |
| the environment | a value inherited from the trusted-side process, rather than computed for the capsule | row 11 |
| the network | a socket to anything off-host or trusted-side | row 5 |
| the process tree | a descendant that outlives the call, or a trusted-side process reachable from inside | row 7, and `sec-7` B5 |
| `argv` | not a channel: it is typed, computed trusted-side, and never capsule-influenced | by construction (`Argv`) |
| the working directory | an inherited cwd makes the capsule's view depend on trusted-side state | row 6 |
| the clock and resource bounds | the capsule reports its own limits rather than the parent observing them | row 8 |

**Two channels on this ledger had no executed row until `RV-346` round 4**, and
they had none for the same reason: each was believed closed by something that is
not enforcement. Descriptors were closed by nobody having thought of them.
The environment was closed by `CapsuleEnv` being a closed *type* — which stops a
caller passing text in, and says nothing about what a backend inherits, as this
section's own note that `--clearenv` stops inheritance only already conceded.
Both had shape assertions over the argv. `DEC-156` is explicit that shape
assertions are necessary and never sufficient, and these two are what that rule
was warning about.

#### The arithmetic, and what correspondence survives

The rows cover every clause and three clauses are covered more than once, so
there is **no one-to-one correspondence** and the design does not claim one. The
correspondence that does hold is the ledger above: every channel has a row, and
every row removes exactly one mechanism.

`DEC-156` has taken this correction three times, each authorised by the human
author rather than by an agent mid-run:

- **seven → eight** (`F-2`): rows 7 and 8 had been merged, on the reasoning that
  teardown and observation are one question — what the trusted parent can
  establish about a process it did not trust. They are independent: a backend can
  reap every descendant and still misclassify a timeout as a signal, or classify
  termination perfectly and leave a grandchild alive. A control removing both
  axes at once cannot say which guard produced the paired result.
- **eight → nine** (`F-19`): clause 2's boundedness and immutability are two
  claims, and a backend binding exactly the declared inputs *writable* passed
  every row.
- **nine → eleven** (`F-26`, and the channel ledger that finding required):
  descriptors and the environment are two further clause-2 channels, neither
  with an executed row.

The pattern across all three is one pattern, and it is `sec-9`'s `R3` in its
sharpest form: **every count correction has come from an adversary building the
backend the suite would wrongly pass, and none from anyone reading the rows.**
The channel ledger exists so that the next such gap is at least visible as a
blank cell rather than as nothing at all.

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

**Those four are the profile's own. A declared readable input keeps its resolved
host path as its inner path** — `sec-3` step 10 binds each entry at the path it
was validated as, which is what makes the derived inner `PATH` below computable
at all. So the transaction's own state is at a fixed inner layout and the
readable set is identity-mapped, and the paragraph above is about the former.

The `try_new` rules are therefore stated over inner paths that come from two
different places, and the gap between them is deliberate: an entry *naming* a
reserved destination is refused, an entry *beneath* one is not. A
`readable-roots` entry resolving under `/tmp` is lawful and lands inside the
`--tmpfs` that precedes it in the assembly order. Refusing it instead would
refuse a host whose toolchain sits beneath a profile-owned path — the `F-25`
shape one level down, where a rule written only as a refusal takes the lawful
case with it. Whether bubblewrap creates that mountpoint inside the tmpfs rather
than refusing the bind is **a phase obligation to execute**, not a claim this
design makes on its behalf.

#### The profile

The twelve flag tokens are named once in this module and nowhere else. Assembly,
in order:

```
--unshare-all                      # namespaces; network included
--proc /proc  --dev /dev           # the two pseudo-filesystems a process needs
--tmpfs /tmp                       # TRANSIENT scratch: per-execution, dies with the capsule
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

#### Descriptors are closed outside the sandbox too, and by no flag

Invariant 12 is the only enforcement in this section with **no flag behind it**.
bubblewrap does not close inherited descriptors, and neither `--unshare-all` nor
`--clearenv` reaches them: an already-open descriptor is not a namespace, a
mount or an environment entry. The mechanism is the trusted parent's, in the
same place as `timeout -k` and `RLIMIT_FSIZE`, and it is named here because the
invariant it discharges was added under `RV-346` `F-26` with nothing behind it
but the word *closed*.

The backend enumerates `/proc/self/fd` in the parent, **before the fork**, and
sets `FdFlags::CLOEXEC` on every descriptor above 2 through
`rustix::io::fcntl_setfd`. `rustix::io` is ungated, so this adds no crate and no
feature to the root package's edge, and `src/worktree/claim_lock.rs:92` is the
precedent for handing rustix a descriptor directly. The new crate's own rustix
edge is not the root package's and is `sec-6`'s to declare.

Enumerate-and-mark in the parent rather than closing after the fork, for one
reason: a post-fork closure runs between `fork` and `exec` where allocation is
unsafe, and reading a directory there is precisely the allocation to avoid.
`close_range` would be the direct form and needs `libc`, which is not a
dependency and which `Cargo.toml:77`'s zero-new-crates posture rules out —
verified this session against `rustix` 1.1.4, which does not carry it.

Two facts bound what the sweep is for, both verified by execution this session.
Rust opens its own files `O_CLOEXEC`, so the backend's *own* handles are already
closed by construction; what the sweep exists for is anything the trusted-side
process inherited from *its* parent, or that an FFI path opened without the
flag. And the residual is a descriptor opened between the sweep and the spawn —
provisioning is single-threaded through `sec-3`'s step list, so that window has
no writer, but the claim rests on `sec-7` row 10 having seen the sweep fire
rather than on this paragraph.

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
5. **Credential denial is claimed per channel, and only where a row proves it.**
   The channel ledger above is the statement of this invariant: for each way
   authority can cross `execute`, a named row establishes that it does not. It
   is written that way because the version it replaces was **false**, and
   instructively so. That version read *credential denial has no second route —
   the environment is a closed vocabulary whose values are computed
   trusted-side, so absence from the mount set is the whole of it rather than
   most of it*, which enumerates two channels and asserts completeness over
   them. `RV-346` `F-26` executed the third: an inherited file descriptor is an
   already-open credential that no mount test and no environment test can see,
   and a backend preserving one satisfied every row the suite then had. A
   completeness claim is only as good as the enumeration behind it, so this
   invariant now points at an enumeration that is written down and rowed rather
   than at a sentence that sounded exhaustive.
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
11. **What is readable is read-only, and that is an executed claim.** Every
   entry in `readable`, and the source export, are bound so that a capsule's
   write to them fails. This is not implied by any of the invariants above:
   invariant 4 says the canonical repository and credentials are *absent*, and
   absence says nothing about how the paths that are present are attached. A
   backend binding exactly the declared set and binding it writable satisfies
   1–10 and violates `DEC-157`. `sec-7` row 9 is where it is proven, with the
   writable binding as its control (`RV-346` `F-19`).
12. **No descriptor crosses `execute` that the contract did not open.** The
   capsule receives the three standard streams and nothing else; every other
   descriptor held by the trusted-side process is marked close-on-exec before
   the capsule starts, by the parent-side sweep above — no flag in the profile
   reaches this channel. Like invariant 11, this has no pure
   test that could establish it — whether a descriptor survived an `exec` is a
   property of the running capsule, not of the argv — so it is executed only, in
   `sec-7` row 10, with an inheritable decoy as its control (`RV-346` `F-26`).
13. **The capsule's environment is computed, never inherited.** Its content is
   exactly the `CapsuleEnv` the placement carries. `CapsuleEnvVar` being a
   closed enum establishes that no *caller* can add to it; it establishes
   nothing about what a *backend* passes through, and those are different
   claims with different failure modes. Executed in `sec-7` row 11.

**Invariants 11, 12 and 13 are one shape repeated**, and the repetition is the
point rather than an accident of drafting. Each names a property that no pure
test can reach, that an argv shape assertion appears to cover and does not, and
that a backend can violate while satisfying every other invariant here. Each was
found by execution rather than by reading. A fourth of the same shape is more
likely than not, and the channel ledger is where it would show up first.

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
- `every_descriptor_above_two_is_marked_close_on_exec_before_the_exec` — the
  parent-side half of invariant 12, asserted over a descriptor the test opens
  *without* `O_CLOEXEC`, since a Rust-opened one is already closed and would
  pass against a backend that swept nothing
- `a_readable_entry_beneath_a_profile_owned_mount_is_admitted` — the lawful case
  the reserved-destination rule must not capture, `/tmp` being the one a host is
  most likely to hit

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

The source-export carve-out, its refusals and — the `F-25` class — the lawful
case each refusal must not capture:

- `the_lawful_source_export_for_this_base_is_admitted` — the positive control,
  and the one whose absence let `F-25` stand. It fails against a validator that
  refuses everything beneath `capsule_root`, which is what this design specified
  until round 4
- `a_source_export_of_a_different_base_refuses` — a well-typed `SourceExport`
  under the export directory, carrying the wrong identity
- `a_source_outside_the_export_directory_refuses` — including one under this
  placement's own transaction root, which the *other* carve-out would otherwise
  seem to admit
- `a_readable_entry_naming_the_export_directory_refuses` — the export reaches a
  capsule only as `source`, never as a declared readable input, so the two
  carve-outs cannot be composed into a third
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

**The three executed-only invariants have no pure test here, and the reason is
the same for each**: they are properties of the running capsule rather than of
the argv, so a pure suite can assert the flag that ought to produce them and
never the thing itself. Invariant 11 (read-only attachment) is `sec-7` row 9;
invariant 12 (descriptor closure) is row 10; invariant 13 (a computed
environment) is row 11.

The environment case is the one to be careful about, because it is the one this
design got wrong. The two pure tests above are real and they prove something
worth proving — that no *caller* can put arbitrary text into `CapsuleEnv`. Read
quickly, they look like they cover the environment. They do not touch inheritance
at all, and inheritance is the channel a backend controls. A pure test over a
closed type and an argv assertion over `--clearenv` were the whole of the
environment's proof for four rounds.

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

**`backend` documents the second one, and it is stated rather than left
implied.** The field records the mechanism that created the capsule and step 11
executes through it; `provision` does **not** check that mechanism ever passed
`sec-7`'s suite. The admission journal is `REQ-455` and a Non-Goal for the same
reason the accepted-ref check is, so the obligation is the caller's — and it is
named in the same words as the base's so a later slice inherits two stated
obligations rather than one stated and one silent. `backend verify` is what a
caller runs to discharge it today; nothing yet records that it did.

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
      tmp/                            #   RETAINED scratch: per-transaction
    agent/                            # rw → /agent, the agent home
```

**Two scratch areas, and they are not the same thing.** The design called both
"the writable temporary area" until the naming hid a defect (`sec-7` B4), so
they are distinguished by name from here on:

| inner path | backed by | lifetime | in `disk_used` |
|---|---|---|---|
| `/tmp` — **transient** | `--tmpfs`, anonymous | one `execute`; gone when that capsule exits | no |
| `/capsule/tmp` — **retained** | `tx/<id>/capsule/tmp/`, host disk | the transaction, across all its executions | yes |

*Transient* and *retained* rather than *isolated* and *shared*: `/capsule/tmp`
is shared only across the executions of **one** transaction, and bare "shared"
collides head-on with `REQ-450` criterion 1's vocabulary, whose whole claim is
that two transactions share no such state. Retained is what actually
distinguishes it — it is on host disk, it survives the capsule that wrote it,
`Observation.disk_used` counts it, and it is transaction state a later slice's
harvest can read. The transient area is none of those.

Only the retained area is a declared writable entry in the placement. `/tmp` is
the profile's own mount (`sec-2`), which is why no placement-level delta reaches
it and why `sec-7` row 1 — writing to every *declared* writable location — does
not name it.

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
   distinct clones, distinct processes, distinct retained scratch. The export is
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
| checkout | a file in the working tree at `/capsule/repo` | both placements carry the first transaction's `TransactionRoot` |
| repository | an object and a ref in `/capsule/repo/.git` | as above |
| runtime | a sentinel in the agent home `/agent` — the state a harness accumulates across a run | as above |
| temporary state | a file in the **retained** scratch at `/capsule/tmp` — never the transient `/tmp`, which no placement delta reaches | as above |
| **process** | capsule A enumerates `/proc` and attempts `kill -0` on a pid the trusted side observed capsule B running under, while both run concurrently | **the pid namespace alone removed**, every other property intact |

**The storage control is a placement, not a second provision.** This table said
*second transaction provisioned into the first's root* until `sec-7` was drafted
against it, and that control cannot be set up: step 9 creates the transaction
root exclusively, and
`provision_onto_an_existing_transaction_root_refuses_and_removes_nothing`
asserts the refusal three rows below. A control the system refuses to build
proves nothing. The removal therefore happens one level down, at the placement —
the fixture builds a second `CapsulePlacement` carrying the *same*
`TransactionRoot`, which `try_new` admits because its carve-out accepts a
writable entry descending from this placement's own root and both placements
satisfy it against the same root. `sec-7` runs it.

The process row is what makes the fifth axis carry information. Its green arm
asserts two things positively — B's process does not appear in A's `/proc`, and
the signal fails — and its control asserts that with the pid namespace shared,
and *only* that changed, both succeed. That is `DEC-156`'s discipline applied to
an axis that has no storage to compare.

Its control is named by the property rather than by the flag, because
bubblewrap has **no `--share-pid`**: `--share-net` is its only re-share flag.
Removing the pid namespace means assembling the explicit `--unshare-*` set
without it, which is `sec-7`'s `PropertyRemoval::ProcessVisibility`. That
removal also disables process-tree teardown as a side effect — measured, in
`sec-7` — so this control leaks a descendant and depends on the harness reaper
`DEC-156` provides for.

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
                              #   its own rustix edge — see below
                              #   publish = false — see Nothing ships
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
  src/conformance.rs          # the property table (count: sec-7 Table A) (sec-7)
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
| `conformance` | engine | `provision`, `transaction`, `backend`, `config`, `host` |
| `main` | command | all of the above |

`conformance`'s out-edges are the ones its own signatures force, not the ones it
conceptually depends on: `sec-7`'s `verify` names `HostFacts`, its fixture
synthesizes a `CapsuleConfig`, and its freshness delta reaches a provisioned
transaction's `TransactionRoot`. An earlier draft listed `provision` and
`backend` alone, which the gate below would have failed — the edge check reads
imports, and a transitive path through `provision` is not one.

**The new crate declares its own `rustix` edge, and cannot inherit the root
package's.** `capacity` calls `statvfs` (`sec-5`) and `backend` performs the
descriptor sweep (`sec-2`), so the manifest carries
`rustix = { version = "1", default-features = false, features = ["fs", "std"] }`.

The `std` in that list is the part worth writing down, because `sec-5` reads as
if this dependency were already paid for. The root package declares
`features = ["fs"]` and gets `std` **only by unification** — from `crossterm`,
and from `which` and `tempfile`, which are dev-dependencies. `doctrine-control`
depends on none of the three. Without `std`, `rustix::fd::AsFd` is a `no_std`
polyfill that `std::fs::File` does not implement and every call site fails
`E0277`; reproduced on a minimal package this session, and cleared by adding the
feature. It is still zero new compiled crates — the same widening-its-own-surface
argument `Cargo.toml:77` already records — but it is a second manifest that has
to say so, and a claim about the root package's edge does not transfer to it.
This is `sec-6`'s standing trap class, not a new one: like the `crate::kinds`
`E0432` below, it is invisible to a reading of the production graph and surfaces
only at the first build of the second crate.

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

**`doctrine-control` has no library target, and that is a decision rather than
an omission.** `DEC-160`'s second caller for the conformance suite is therefore
a `#[cfg(test)]` module inside `src/conformance.rs`, not a `tests/` file — a
`tests/` file cannot link a bin-only package at all (`E0433`), and adding a lib
target to rescue it would force `verify` and everything in its signature to be
`pub` (`E0603` on `pub(crate)`), taking `sec-7`'s weakening vocabulary public
with it. All three claims verified by execution. The root package's lib target
above exists because a *second crate* has to reach it; nothing has to reach
into `doctrine-control`, so it acquires no published surface.

**Nothing ships, and that takes two changes rather than a convention.**
`publish = false` in the new crate's manifest, **and** the publish recipe
path-limited to `cargo publish -p doctrine`. Both, because neither alone is
enough and the failure modes are opposite ones — measured on a minimal
workspace while remediating `RV-346` `F-21`:

- `default-members` selects the new crate for *packaging and publishing* as
  well as for building, so a bare `cargo publish` — which is what `just publish`
  runs today — would reach for it. The manifest key is what states the
  intent durably rather than leaving it to a flag someone may drop.
- But `publish = false` does not make a bare `cargo publish` *skip* the member;
  it makes the whole command fail with ``error: `doctrine-control` cannot be
  published``. So the manifest key alone would break releases instead of
  protecting them, and the recipe must name its package.

`just publish` is therefore in this slice's touch-set (`sec-8`), which an
earlier draft of that section denied by listing the release arrangement among
what does not change. `pkg-check` already passes `-p doctrine` and needs
nothing.

The distribution contract itself — the nix `srcWithDist` graft, the binstall
asset name, `install.sh`, `release.yml` — remains a close-time Follow-Up for
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


<!-- doctrine:section sec-7 -->
## The conformance suite: the properties, their controls, and admission

`REQ-459` criterion 1 asks for a shared conformance suite, `DEC-156` fixes its
discipline, and `DEC-160` fixes where it lives and who calls it. This section is
that suite: the harness, the removal vocabulary, the three tables it runs, and
the verdict it returns. It is the only part of this design that executes
anything, which is why every other section's `Verification alignment` ends by
pointing here.

### Where it lives, and its two callers

`crates/doctrine-control/src/conformance.rs` — engine tier, direct out-edges
`provision`, `transaction`, `backend`, `config` and `host` (`sec-6`).

```rust
/// The whole suite, parameterised by backend. `REQ-459` criterion 3: a second
/// backend is admitted by passing these assertions, never by editing them.
pub(crate) fn verify(
    backend: &dyn ConformanceBackend,
    host: &dyn HostFacts,
    today: Date,
) -> AdmissionVerdict;
```

`today` is a parameter rather than a read, for the reason `sec-5` gave for
`HostFacts` carrying no clock: `src/clock.rs` is the single home for wall-clock
reads and `main.rs` is the shell that performs it. The verdict is dated because
`DEC-156` requires admission to be a recorded verdict naming backend, host and
date, so `REQ-459` criterion 3's *independently* has an artefact to point at.

**No `CapsuleConfig` parameter.** An earlier draft took the operator's
`[capsule]` table, which is wrong twice over: the suite would then be testing
the operator's configuration rather than the backend's enforcement, and a
table whose `readable-roots` happened to omit a shell would make every probe
`NotExecutable` and the whole run indeterminate for a reason it never named. The
fixture synthesizes its own `CapsuleConfig` over its own root instead, so the
only host facts admission depends on are the backend's availability and a
working shell. A host with no usable shell is reported through the existing
`NotAdmitted::Unavailable`, naming what is missing.

`DEC-160`'s two callers:

- **`backend verify`** calls it for the on-host verdict, exiting nonzero with
  structured output when the backend is not admitted.
- **A `#[cfg(test)]` test inside this module** calls it so CI asserts *admitted*.

**The test is a unit test, not a `tests/` integration test, and that is forced.**
`doctrine-control` is a binary crate (`sec-6`), and a `tests/` file cannot link
one — verified by execution: `E0433, use of unresolved module or unlinked
crate`. Adding a library target does not rescue it either, because everything
the test touches would then have to be `pub`: with a lib target and
`pub(crate) fn verify`, the same test fails `E0603, function is private`, and
only promoting it to `pub` compiles. That promotion would take
`ConformanceBackend` and `PropertyRemoval` public with it and destroy the
sealing below — the suite's weakening vocabulary would become constructible by
anything that depends on the crate. A `#[cfg(test)]` module reaches `pub(crate)`
and `cargo test` runs it in a bin-only crate, both verified the same way.
`DEC-160` rejected a **`tests/`-only** suite because a skipping test reports
green; it is indifferent to where the second caller sits, and this placement
changes nothing about that argument.

### The removal is named by the property, never by a flag

A control removes one property. What it costs in flags is the backend's
business, so the vocabulary is property-shaped — a flag-shaped one would be
bubblewrap's and could not be asked of a second backend.

```rust
/// One property a control arm removes.
///
/// `pub(crate)`, with `ConformanceBackend`, because every backend lives in this
/// crate (`sec-6`: `backend/bubblewrap.rs`, plural-ready) and the only callers
/// are `main.rs` and this module's own tests. That is an assumption, not a law
/// — if a backend ever ships from outside the crate this vocabulary becomes
/// public API and needs sealing behind a newtype over a private enum, so
/// nothing but the suite can ask a backend to weaken itself. `sec-9` carries it.
pub(crate) enum PropertyRemoval {
    WorkingDirectory,
    Teardown,
    ProcessVisibility,
    ResourceBound(Bound),
    /// Row 9. Every readable input and the source export become writable —
    /// the mechanism unique to input immutability, and the only removal that
    /// changes how an existing mount is bound rather than which mounts exist.
    InputsWritable,
    /// Row 10. The fixture's decoy descriptor is left inheritable across
    /// `exec` instead of close-on-exec. Changes no mount, no environment
    /// variable and no argv byte — the mechanism unique to descriptor closure.
    DescriptorsClosed,
    /// Row 11. The trusted-side environment is not cleared before the
    /// capsule's own is applied. Changes no mount and no descriptor.
    EnvCleared,
}

pub(crate) enum Bound { FileSize, Wall }

/// The admission instrumentation, deliberately not on `CapsuleBackend`.
///
/// Production uses `CapsuleBackend` and it carries no way to weaken anything.
/// A backend seeking admission implements this second trait as well, and
/// implementing it is not a favour: without controls the suite proves nothing,
/// so `DEC-156`'s discipline *is* this obligation.
pub(crate) trait ConformanceBackend: CapsuleBackend {
    fn execute_weakened(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
        removal: PropertyRemoval,
    ) -> Result<Observation, BackendError>;

    /// Run as `execute` does, and call `observer` exactly once — trusted-side,
    /// after the capsule's top-level process exists and before this returns —
    /// with that process's pid *in the host's pid namespace*.
    ///
    /// Row B5's seam, and it exists because nothing else can supply one.
    /// `CapsuleBackend::execute` is synchronous and yields only an
    /// `Observation` after the run is over, so a harness holding it in flight
    /// on a thread still cannot name the process it started; and a pid the
    /// capsule reports about itself is capsule-written state, which invariant 9
    /// and `REQ-448` criterion 3 forbid as evidence.
    fn execute_observed(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
        observer: &dyn Fn(HostPid),
    ) -> Result<Observation, BackendError>;
}

/// A pid as the trusted parent sees it, in the host's own pid namespace.
/// A newtype because the whole point of row B5 is that this number means
/// something different inside a capsule than it does outside one.
pub(crate) struct HostPid(pub(crate) i32);
```

**Why the seam is here and not on `CapsuleBackend`.** Invariant 6 holds that
the production trait carries no admission affordance, and an observer is an
admission affordance even though it cannot weaken anything: production has no
use for it and would pass a no-op. Putting it on `ConformanceBackend` also
makes it part of what admission *costs* — a backend that cannot name the
process it spawned cannot have its process-isolation property controlled, and
a property that cannot be controlled cannot be proven. Refusing to admit such a
backend is the correct outcome rather than an inconvenience.

An earlier draft of this section specified row B5 against `execute` alone and
was not implementable. `RV-346` `F-18` is right that the control added in
answer to `F-3`…`F-6` could not be built at all, which is the same defect as
the `SharedRoot` mistake two subsections below and is why both are recorded
here rather than quietly repaired.

**A dishonest implementation fails closed.** An `execute_weakened` that ignores
its argument and runs fully confined makes every control arm show the property
still holding, which the harness reports as `Unproven` — not admitted. There is
no lazy implementation that yields a green verdict.

`BubblewrapBackend`'s mapping, each delta measured rather than reasoned about
(`EVD-013`):

| removal | bubblewrap delta |
|---|---|
| `WorkingDirectory` | `--chdir <wd>` omitted |
| `Teardown` | `--die-with-parent` omitted |
| `ProcessVisibility` | `--unshare-all` replaced by `--unshare-user --unshare-ipc --unshare-uts --unshare-cgroup --unshare-net` |
| `ResourceBound(FileSize)` | `RLIMIT_FSIZE` not set on the child |
| `ResourceBound(Wall)` | the `timeout -k` wrapper omitted |
| `InputsWritable` | every `--ro-bind` carrying a readable entry or the source export becomes `--bind` |
| `DescriptorsClosed` | the fixture's decoy descriptor is passed with `--file-descriptor`-style inheritance instead of being closed on exec |
| `EnvCleared` | `--clearenv` omitted, the explicit `--setenv` list unchanged |

**The last two deltas are reasoned, not measured**, and are the only rows in
this table of which that is true — `EVD-013` covers the five above them. Under
`R1` that difference is recorded rather than smoothed over: measuring both is a
phase obligation, and a delta that turns out not to produce its row's control
failure means the row is wrong, not that the measurement is inconvenient.

`ProcessVisibility` enumerates rather than subtracting because **there is no
`--share-pid`**: `--share-net` is bubblewrap's only re-share flag and its help
states it "can only combine with `--unshare-all`". The removal is therefore the
one place the backend does not assemble its profile the usual way, and it is
worth the asymmetry — the alternative is a suite that cannot control its own
process rows.

### The arm is the unit of execution, not the capsule

Six rows across tables A and B are two-capsule — something is written or done in one
capsule and observed from another — and one of those runs its two capsules
concurrently. A runner keyed to a single placement cannot carry them, so the
unit the harness runs is the **arm**: everything that executes in order to
observe the property once.

```rust
/// What runs, in one arm. Identical on a row's probe and control arms — a
/// control that changed the payload would not be a control.
enum ArmShape {
    /// One capsule.
    Single(Probe),
    /// Two capsules in sequence: `writer` runs to completion, then `reader`
    /// observes. Rows 1 and B1–B4.
    Sequential { writer: Probe, reader: Probe },
    /// Two capsules concurrently. The trusted side observes the subject's pid
    /// and renders it into the observer's argv as a decimal integer — the only
    /// value any payload interpolates, and it is trusted-side-computed. Row B5.
    Concurrent { subject: Probe, observer: PidProbe },
}

/// Row B5's observer. The subject's pid is not known until the subject is
/// running, so the observer's argv is built from it rather than fixed — the
/// only value any payload interpolates. The pid is the one the *trusted side*
/// observed the subject running under, never one the subject reported
/// (`REQ-448` criterion 3).
struct PidProbe {
    argv: fn(pid: HostPid) -> Argv,
    observed: Observed,
}
```

**How the pid arrives, and why a host-namespace pid is the right one to send
in.** The harness runs the subject through `execute_observed` on its own
thread, handing an observer that publishes the pid to the main thread and
returns. The main thread waits for that pid, renders the observer's argv from
it, and runs the observer while the subject is still alive — the subject's
payload sleeps for a bounded interval so the window exists rather than being
raced for.

The number handed to the observer is a pid in the *host's* namespace, and that
is exactly what gives the row content. Under the probe arm the observer has its
own pid namespace, in which that number names nothing, so both assertions hold
for the reason the property claims: the subject is not visible. Under the
control arm — `ProcessVisibility` removed, and only that — the observer shares
the host's pid namespace, the same number resolves to the subject, and both
assertions become possible. A namespace-local pid would have made the probe arm
hold for an arithmetic reason instead of an isolation one.

Two failure modes the row must not read as success, both classified
`Indeterminate` rather than `Failed`: the subject exiting before the observer
runs, and `execute_observed` returning without ever having called the observer.
The second is a backend that did not implement the seam, and a suite that read
it as a passing probe would admit the backend it was least able to check.

### Both arms provision; the control applies exactly one delta

Every capsule in every arm — probe and control alike — is placed by
`sec-3`'s `provision`, and the probe arm's placement is **exactly what
`provision` returned, unmodified**. That is what makes the suite end-to-end in
`DEC-160`'s sense and what keeps it honest about `sec-2` invariant 1: the suite
does not hand a backend placements provisioning would never produce.

The control arm provisions the same capsules the same way and then applies one
typed delta to the placement.

```rust
/// How a control arm's placement differs from the probe arm's. Exactly one
/// value, so "differs by one thing" is a type rather than a promise.
enum Delta {
    /// The second capsule's placement is rebuilt on the *first* capsule's
    /// `TransactionRoot`. Both transactions are still provisioned normally;
    /// only the second placement is re-pointed, so the arms differ by this and
    /// nothing else. The freshness control.
    SharedRoot,
    /// The placement is widened by the row's declared readable entries, rebuilt
    /// through `try_new`. Rows 2, 3, 4.
    Widened(fn(&Fixture) -> Vec<MountedPath>),
    /// The placement's network posture becomes `Permitted`. Row 5.
    NetworkPermitted,
    /// The placement is unchanged; one profile property is removed from the
    /// backend. Rows 6, 7, 8, 9, B5.
    Removed(PropertyRemoval),
}
```

`SharedRoot` is where an earlier draft was simply wrong. It had the control
*provision* a second transaction into the first's root, which cannot be set up:
`sec-3` step 9 creates the transaction root exclusively and
`provision_onto_an_existing_transaction_root_refuses_and_removes_nothing`
asserts the refusal. A control the system refuses to build proves nothing. The
removal happens one level down instead — the second capsule is provisioned as
usual, and its *placement* is rebuilt carrying the first's root, which `try_new`
admits because its carve-out accepts a writable entry descending from this
placement's own root and both placements satisfy it against the same root.
`sec-3`'s freshness table is amended to match.

Every mount-set delta is rebuilt through `CapsulePlacement::try_new`, and that
the rebuilt placement is still lawful is itself evidence: it passes the same
validating constructor the probe's placement passed.

```rust
struct Row {
    id: RowId,
    shape: ArmShape,
    delta: Delta,
}

/// A row of either admission table. Table B's rows are freshness axes, not
/// properties, so one enum spans both rather than a `Property` key that cannot
/// hold half of what the verdict reports.
enum RowId {
    /// An enforcement claim of `SPEC-030` § Platform backend contract. One
    /// per channel rather than one per clause — see Table A, and `sec-2`'s
    /// channel ledger for why clause 2 carries four.
    Property(Property),
    /// One of `REQ-450` criterion 1's five freshness axes.
    Axis(Axis),
}

enum Property {
    FreshMutableState, BoundedInputSet, ImmutableInputSet,
    ClosedDescriptorSet, ClosedEnvironment,
    CanonicalAndCredentialDenial, BoundedFilesystemVisibility, NetworkPosture,
    WorkingDirectory, ProcessTreeTeardown, ResourceAndTerminationObservation,
}

enum Axis { Checkout, Repository, Runtime, TemporaryState, Process }
```

### Liveness first, observation second

A probe that was *denied* and a probe that never ran look identical on an empty
stdout. Reading the second as the first is how a suite certifies a backend it
never exercised, and it is worse than it first appears: if `Failed` is defined
as merely *not `Held`*, then a payload that breaks **on the control arm only**
reads as `Failed` — which is exactly what a row needs to report `Proven`. A
false green, through the very hole a token rule exists to close.

So classification is two-stage, and the order is the fix. **Unless the payload
proved it ran, the arm is `Indeterminate` regardless of what the observation
says.** Every payload emits a liveness marker before anything else:

```sh
/bin/sh -c 'echo LIVE; if cat /decoy/credential >/dev/null 2>&1; then echo HELD-NOT; else echo HELD; fi'
```

```rust
/// What the arm is read off, once liveness is established.
enum Observed {
    /// Exactly one of two tokens on stdout.
    Token { held: &'static str, failed: &'static str },
    /// A value line, compared for equality — row 6, where the observation is a
    /// value rather than a binary. A missing value line is `Indeterminate`,
    /// never `Failed`.
    Exactly(String),
    /// The termination itself is the observation — row 8. The marker still
    /// arrives, because `Observation` carries `stdout` whatever the
    /// termination: `sh -c 'echo LIVE; exec sleep 60'` prints before it is
    /// killed.
    Termination(Termination),
}

struct Probe { argv: Argv, observed: Observed }

/// What one arm showed about the property under test. Uniform across all three
/// `Observed` kinds, which is why the vocabulary is *held/failed* rather than
/// *reached/denied*: rows 6 and 8 observe a value and a termination, not a
/// reach.
enum ArmResult {
    Held,
    Failed,
    Indeterminate {
        reason: Indeterminacy,
        termination: Termination,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
}

enum Indeterminacy {
    /// No liveness marker: the payload did not run. Never folded into `Failed`.
    NoLiveness,
    /// Live, but neither token — or, for `Exactly`, no value line.
    NoObservation,
    /// Both tokens, which means the payload is wrong.
    AmbiguousObservation,
    BackendError(String),
}
```

**The one payload that cannot print its own marker** is row 8's
`NotExecutable` case, which by definition never runs. Its liveness comes from
the same capsule instead: a liveness execution precedes it, and only if that
succeeds is a subsequent `NotExecutable` meaningful. Without that ordering the
sub-row passes on any host where the shell is missing — for the wrong reason,
and looking exactly like success.

Payloads are `Argv` values holding fixed constants, not compiled helpers or
embedded assets, which is what keeps the product weight of shipping the suite
small (`DEC-160`). They do invoke `/bin/sh -c`, which the provisioning steps
deliberately avoid; the distinction is that `sec-2`'s rule exists to keep
caller-supplied text off a security boundary, and these payloads interpolate
nothing except B5's trusted-side-observed pid.

### The row verdict, and why *unproven* is not *violated*

```rust
enum RowVerdict {
    /// The probe held and the control failed. The property is enforced, and the
    /// control licenses the inference that the enforcement is what did it.
    Proven,
    /// The probe arm did not hold: the property is not enforced.
    Violated,
    /// The control arm still held, so removing the property changed nothing and
    /// the probe's result has no established cause.
    Unproven,
    /// Either arm was indeterminate. The row establishes nothing in either
    /// direction, which is not the same as either failing.
    Indeterminate { arm: Which, detail: Indeterminacy },
}

enum Which { Probe, Control }
```

Only `Proven` on every row admits. `Violated` and `Unproven` are both
not-admitted and are reported distinctly because they name **different
repairs**: the first says the guard is broken, the second says the row is. This
is `SL-241`'s rule — a guard never seen to fire is not known to work
(`probe-guards.sh`, `EX-10`) — turned on the suite itself.

### Table A — the properties

**This table is where the count lives** (`sec-2`). `SPEC-030` § Platform backend
contract states eight clauses and this table has **eleven** rows, because clause
2 — *an explicit base and input set* — is a claim about inputs that names none
of the channels inputs arrive by. Rows 2, 9, 10 and 11 are all clause 2. `sec-2`
§ The rows are derived from channels, not from clauses carries the argument and
the channel ledger; this table is its realisation.

| # | property | shape | probe holds when | delta | control fails when |
|---|---|---|---|---|---|
| 1 | fresh mutable state | `Sequential`: A writes a sentinel into **every** writable location the placement declares — `/capsule/out`, `/capsule/tmp`, `/agent` — B reads all three | all absent in B | `SharedRoot` | any present in B |
| 2 | bounded input set | `Single`: exec a binary from each bound path, then exec the decoy | declared exec 0, decoy `NotExecutable` | `Widened` (decoy's directory) | decoy execs 0 |
| 3 | denial of canonical state and credentials | `Single`: read the decoy credential, then the decoy repository's secret blob | both reads fail | `Widened` (both decoys) | both reads succeed |
| 4 | bounded host filesystem visibility | `Single`: read the undeclared decoy; enumerate `/` | read fails, `/` holds only the profile's own entries | `Widened` (the undeclared decoy) | read succeeds |
| 5 | explicit network posture | `Single`: connect to the trusted-side TCP loopback listener, **then** to a trusted-side abstract unix socket | both refused | `NetworkPermitted` | both connected |
| 6 | deterministic working directory | `Single`: `pwd`, the arm run from two different trusted-side cwds | both equal `working_directory` | `Removed(WorkingDirectory)` | `pwd` tracks the trusted-side cwd |
| 7 | process-tree teardown | `Single`: fork a descendant that **leaves the original session and process group** (`setsid`) before its parent exits | no descendant outlives the `execute` call, established trusted-side | `Removed(Teardown)` | the escaped descendant survives |
| 8 | trusted observation of resource limits and termination | `Single` ×5, one per `Termination` variant | each variant correctly distinguished | `Removed(ResourceBound(..))` | the capped and timed-out payloads run to completion |
| 9 | immutable input set | `Single`: write through each readable mount, then write into `/source` | every write fails | `Removed(InputsWritable)` | the writes succeed |
| 10 | closed descriptor set | `Single`: the parent opens a decoy secret and marks it **inheritable**; the payload enumerates `/proc/self/fd` and reads every descriptor above 2 | no descriptor above 2 is readable, and the decoy's bytes appear nowhere in `stdout` | `Removed(DescriptorsClosed)` | the decoy's bytes are read |
| 11 | closed environment | `Single`: the parent sets a decoy variable in **its own** environment; the payload dumps its whole environment | the dump is exactly `CapsuleEnv`, and the decoy is absent | `Removed(EnvCleared)` | the decoy is present |

#### The payload must be the property's strongest negation, not a representative one

`RV-346` `F-27` is the rule's instance and the rule is worth more than the fix.
Row 7 read *orphan a descendant, then exit*, which is a **representative**
instance of the property's negation — a descendant that outlives its parent. It
is not the **strongest** one. A backend that reaps by killing the original
process group passes that probe, because a plain orphan is still in the group;
a descendant that has called `setsid` survives it. The row's title claimed *no
descendant outlives the `execute` call* and its payload proved only *no
descendant in the original process group does*, so the row was weaker than the
property it was named after, and the gap was invisible because both readings use
the same words.

Codex reproduced the mechanism without namespaces: a process-group kill removed
the same-group control and a `setsid` descendant remained alive until cleaned up
explicitly.

So each row's payload is written as the **hardest instance of the property's
negation the property admits**, and where a property has several distinguishable
negation mechanisms the payload exercises each of them. Applying that rule to
the whole table rather than to row 7 alone moved three rows, which is the return
on stating it as a rule:

- **Row 1** wrote a sentinel to `/capsule/out` only. A backend giving each
  transaction a fresh output area while sharing the agent home or the retained
  scratch passed. The payload now writes to every writable location the
  placement declares — which is `/capsule/out`, `/capsule/tmp` and `/agent`, and
  not the transient `/tmp`, since that is the profile's mount rather than a
  declared entry.
- **Row 5** attempted a TCP loopback connection only. A backend that denies by
  packet filter rather than by network namespace blocks TCP and leaves an
  abstract unix socket — a distinct mechanism in the same channel — reachable.
  The payload now attempts both. Under `--unshare-all` a fresh network namespace
  denies both, so bubblewrap's arm is unaffected; the row now discriminates
  between mechanisms that were previously indistinguishable to it.
- **Row 7** as above.

**Row 7's containment has to move with its payload**, and this is the part a
weaker reading would miss. `sec-7`'s hazard containment reaps the control arm's
survivor with a process-group kill — which is precisely the mechanism row 7 now
exists to defeat, so the harness would leak the very process its own control
creates. Containment for this row is therefore the outer `timeout -k` plus a
sweep that does not stop at the original process group: the fixture records the
capsule's session id trusted-side before the arm runs and kills the session on
the way out. A control arm the harness cannot clean up is a suite that leaks
processes on every run.

#### Clause 2 is four rows, one per channel

Clause 2 asks for *an explicit base and input set*, and every one of its four
rows was added after someone built the backend the previous rows admitted. Rows
2 and 9 are the mount channel: **bounded** — only declared paths reach a capsule
— and **immutable** — a declared path cannot be written through. Row 10 is the
descriptor channel and row 11 the environment channel. No two of the four imply
each other.

**Rows 10 and 11 are `RV-346` `F-26` and the channel ledger it forced.** The
finding built the tenth mutant the brief asked for: a backend identical to a
conforming one on every observation the suite then made, but preserving one
inherited file descriptor across `execute`. It passes rows 2, 3, 4 and 9 —
every one of them reasons about *paths*, and an already-open descriptor is not a
path — and it hands the capsule an open credential file or a preconnected socket.
Codex isolated the mechanism from its own sandbox before raising it: a direct
child given an inheritable descriptor read the decoy secret and exited 0, while
the bubblewrap-shaped profile exposed no bytes. So `BubblewrapBackend` closes
descriptors and the *suite* was what failed — the distinction `DEC-156` requires
of every finding at this altitude, and the one round 3's `F-20` got wrong.

Row 11 was not raised by the review. It came out of writing the channel ledger
that `F-26` required, which is the return on fixing the class instead of the
instance: the environment sat in the ledger with no executed row against it. Its
proof was `CapsuleEnvVar` being a closed enum — which binds *callers* and says
nothing about a backend — plus an argv assertion that `--clearenv` is present.
`sec-2`'s own note that `--clearenv` stops inheritance only is the concession
that inheritance is a separate channel; nothing executed ever checked it. A
backend that omits it passes every other row while handing the capsule whatever
the trusted-side process holds, which on a developer machine or a CI runner is
routinely a credential.

`RV-346` `F-19` found this by executing the mutant rather than reading the
profile — `bwrap --ro-bind / / --bind HOST HOST` followed by a capsule write
changed the host's marker and exited 0. As drafted, every row of tables A and B
passed that backend: row 2 only executes from bound paths, rows 3 and 4 only
establish that undeclared things are unreachable, and table B writes only to
transaction-local paths that are *supposed* to be writable. The suite had no row
that wrote to something it had asked to be read-only, so the one guard `DEC-157`
depends on was the one guard never seen to fire.

`DEC-156`'s count moves **nine → eleven**, authorised by the human author rather
than taken mid-run — the third such correction, on the same terms as the first
two. Its correspondence sentence stays withdrawn and is now generalised: the
rows cover every clause, three clauses are covered more than once, and the
structure that *is* one-to-one is the channel ledger, because one control can
only remove one mechanism.

**Each of the three new rows has a mechanism unique to it**, which is the rule
rows 3 and 4 are held to and the reason each is separately controllable:

- Row 9's is the read-onlyness of a bind that exists under both arms. Its delta
  changes no mount's presence and no path, only how an existing mount is
  attached, which is why it is a `PropertyRemoval` and not a `Widened`.
- Row 10's is descriptor closure at `exec`. Its delta leaves the mount set, the
  environment and the argv identical and changes only whether the decoy
  descriptor is marked close-on-exec, so a row-10 failure cannot be produced by
  any other guard in the table.
- Row 11's is the clearing of inherited environment. Its delta leaves every
  mount and every descriptor identical and changes only whether the trusted-side
  environment is cleared before the capsule's own is applied.

**Its hazard is that the control arm really does write to host state**, and
containment is structural rather than careful: the readable entries in this
row's placement are fixture-owned decoys under the fixture's own root, and the
`/source` it writes to is a **per-run export the fixture built for itself**,
never one adopted from a shared capsule root. A control arm that could corrupt
the export other transactions adopt would be a suite that damages the property
it is testing.

**Rows 10 and 11 carry their hazard the same way — the decoy is the fixture's,
never the host's.** Row 10's control arm really does hand a capsule an open
descriptor, so the descriptor is opened onto a fixture-created decoy file under
the fixture's own root, never onto anything the trusted side holds for real.
Row 11's control arm really does leak the trusted-side environment, so the
assertion is over a decoy variable the fixture sets in its own child before
`execute`, and the row asserts the *decoy's* absence rather than dumping and
diffing whatever the operator's shell happens to carry. Both are the same
structural containment row 9 uses: the control arm exercises the real mechanism
against a target the suite made for itself.

Row 11's assertion is set-equality against `CapsuleEnv`, not a search for known
credential names. A denylist of variable names would pass a backend leaking a
variable nobody thought to list, which is the defect this row exists to close
restated one level down.

**Row 5's TCP arm is measured; its abstract-socket arm is not.** Against a
trusted-side listener on `127.0.0.1`, the capsule is refused under
`--unshare-all` (`ConnectionRefusedError`) and connects under `--share-net`. A
fresh network namespace has its own loopback, so that distinction holds offline
and in CI with no external contact — which is what `DEC-156` requires of this
row's hazard. The abstract unix socket leg is reasoned, not executed: abstract
sockets are scoped to the network namespace, so `--unshare-all` should deny it
by the same mechanism. Under `R1` that is a claim this design does not get to
make on bubblewrap's behalf without running it, so **executing both legs is a
phase obligation**, and if the abstract leg turns out to be denied by a
different mechanism than the TCP leg it is a twelfth row rather than a second
assertion in this one.

#### Rows 3 and 4 share a mechanism and are still independent

Both deny by absence from the mount set (`sec-2` invariant 4), so the natural
objection is that they are one property. They are not, and the difference is
what row 3 reaches for: row 4 asks whether an *arbitrary* undeclared path is
unreachable, row 3 whether the specific shapes `SPEC-030` names — a repository
holding history the capsule was not given, and a credential file — are. A
backend could deny arbitrary undeclared files and still be handed a placement
containing a credential store.

Row 3's decoys are **not** members of `ForbiddenScopes`, which is what makes its
control a lawful single delta: widening `readable` by the decoy entries passes
`try_new` like any other. `ForbiddenScopes`' own job — refusing a placement that
*declares* the real canonical repository or credential scope — is a
construction-time refusal with no execution in it, and it is proven by `sec-2`'s
pure tests (`declared_root_that_resolves_into_the_credential_scope_refuses` and
the `F-10` descendant mutants). The suite does not repeat them, and could not:
a placement declaring a forbidden scope never gets built, so there is no arm to
run.

The related claim that the clone's object set is the contracted history alone
carries **no control** — nothing about it can be removed one axis at a time — so
it is not an admission row. `sec-3` owns it
(`export_holds_the_contracted_history_and_no_other_ref`); it appears in table C.

#### Row 7's control is `--die-with-parent`, and the choice is measured

`EVD-013` records the 2×2. Teardown needs **both** the pid namespace and
`--die-with-parent`; neither is redundant, because bubblewrap runs its own init
as pid 1 (the payload reports `pid=2`) so the namespace never collapses when the
command exits. Either removal therefore makes the control fail as required.

The row takes `--die-with-parent` — the mechanism **unique to teardown**. Taking
the pid namespace instead would remove the mechanism row B5 also depends on, and
a control removing two rows' guards at once cannot establish which one produced
the result. That is `RV-346` `F-2`'s objection applied one level down, and it
yields the rule the suite follows generally: *a row's control removes the
mechanism unique to that row's property; where a property has no unique
mechanism, it cannot be controlled independently and the rows must be re-cut.*

The converse is a hazard rather than a defect: row B5's control does remove the
pid namespace and so breaks teardown as a side effect, leaking a descendant.
`DEC-156` provides the containment; `EVD-013` is why it is needed on B5's arm
and not only on row 7's.

**The 2×2 was measured against the old payload, and one cell needs re-measuring
under the new one.** `EVD-013` established that both the pid namespace and
`--die-with-parent` are required for a descendant that merely outlives its
parent. `RV-346` `F-27`'s payload escapes the session as well, and the mechanism
by which bubblewrap reaps it is the pid namespace collapsing rather than any
process-group relationship — `setsid` changes a descendant's session, not its pid
namespace, so the namespace should still take it. *Should* is the operative word:
that is reasoning about the new payload from a measurement made against the old
one, which is exactly the altitude error `R1` forbids. **Re-running the 2×2
against the escaping payload is a phase obligation**, and if `--die-with-parent`
turns out not to be required for it, row 7's control is wrong and this
subsection's argument — not just its number — has to be redone.

#### Row 6 observes a value, and the trusted-side cwd is the thing to vary

Running the arm twice from two different trusted-side working directories is
what gives row 6 content. Measured:

```
--chdir /tmp,  trusted cwd /workspace/doctrine  →  pwd = /tmp
no --chdir,    trusted cwd /workspace/doctrine  →  pwd = /workspace/doctrine
no --chdir,    trusted cwd /tmp                 →  pwd = /tmp
```

Without `--chdir` the capsule's working directory *tracks the trusted side's* —
exactly the inheritance the property forbids, and invisible to a probe run from
a single cwd. `sec-2`'s `working_directory_has_no_inherit_value` is the
type-level half of the same property, asserted by construction; this is the
executed half.

#### Row 8 is five payloads, one per `Termination` variant

| variant | payload | control |
|---|---|---|
| `Exited { code }` | `exit 7` | none |
| `Signalled { signal }` | raises `SIGTERM` on itself | none |
| `TimedOut` | sleeps a small fixed multiple of the configured timeout | `ResourceBound(Wall)` |
| `FileSizeExceeded` | writes just over `file_size_cap` | `ResourceBound(FileSize)` |
| `NotExecutable` | an argv naming a path outside the readable set | none |

`NotExecutable` and `Exited { code: 127 }` are the pair the spike had to
separate by hand (`sandbox.sh:297,309`) — *the runner refused* and *the runner
never ran* otherwise read identically — so the row asserts them as distinct
outcomes rather than merely as distinct codes.

**Three of the five carry no control, and that is stated rather than papered
over.** They are observations of what the OS reports, not enforcement claims, so
there is no guard whose firing is in question. The enforcement half of row 8 is
the two bounded payloads, and the row is `Proven` on those; the other three
contribute to the probe arm only.

The timeout payload sleeps a **small fixed multiple** of the configured timeout
rather than an open-ended duration. Under `ResourceBound(Wall)` the bound is
gone, so whatever the payload sleeps is what the control arm costs — an
open-ended sleep would hang `backend verify` rather than fail it.

### Table B — the five freshness axes

`REQ-450` criterion 1 and `DEC-157`, driven by the same harness. Four storage
rows share one delta; the process row has its own (`sec-3`).

| id | axis | shape | delta |
|---|---|---|---|
| B1 | checkout | `Sequential`: a file in the working tree at `/capsule/repo` | `SharedRoot` |
| B2 | repository | `Sequential`: an object and a ref in `/capsule/repo/.git` | `SharedRoot` |
| B3 | runtime | `Sequential`: a sentinel in the agent home `/agent` | `SharedRoot` |
| B4 | temporary state | `Sequential`: a file in the **retained** scratch at `/capsule/tmp` | `SharedRoot` |
| B5 | process | `Concurrent`: A enumerates `/proc` and attempts `kill -0` on a pid the trusted side observed B running under | `Removed(ProcessVisibility)` |

B5's probe asserts two things positively — B's process is absent from A's
`/proc`, and the signal fails — and its control asserts that with the pid
namespace shared and only that changed, both become possible.

**B4 named `/tmp` until this round, and its control could not fail.** `/tmp` is
`--tmpfs`: an anonymous mount the profile makes fresh on every `execute`, backed
by nothing beneath the transaction root. `SharedRoot` re-points a placement's
root, so it reaches everything the placement *declares* writable and cannot
reach `/tmp` at all — A's file died with A's capsule and B saw an empty tmpfs
under **both** arms. That is `Unproven` by this section's own algebra, and since
`Admission` is computed from tables A and B alone, the suite as specified could
never return `Admitted`. B4 now targets the retained scratch, which is the
transaction state `REQ-450` criterion 1 is about and which `SharedRoot` does
reach.

Two things this is worth noticing about. It is the residual of the `SharedRoot`
repair two subsections above: moving the removal down to the placement fixed a
control that could not be *built*, and left one that could not *fail* — the same
`F-25` lesson a third time, that a rule checked in only one direction is half a
rule. And it survived four review rounds because two different areas answered to
the same words; `sec-3` now names them.

The transient area carries no admission row and that is deliberate rather than
an omission. Its privacy is `--tmpfs`'s by construction, it holds no transaction
state, `Observation.disk_used` does not count it, and `REQ-450` criterion 1's
temporary-state axis is a claim about state two transactions could share. A row
for it would need its own `PropertyRemoval` — binding a shared host directory at
`/tmp` — which buys a proof of something no requirement asks for at the cost of
widening the weakening vocabulary `sec-9` residual 2 already tracks as growing
every round.

**Row 1 keeps writing to the retained scratch, and the overlap with B4 is
intended.** Tables A and B discharge different requirements — row 1 is
`SPEC-030`'s backend property, B1–B4 are `REQ-450` criterion 1's named axes —
and this section's non-overlap rule binds a row's *control* to a mechanism
unique to it, not a row's payload to untouched ground. Narrowing row 1 to avoid
the overlap would undo `F-27`'s strengthening, which widened it to every
declared writable location precisely because a backend freshening the output
area while sharing the agent home passed the narrower payload.

### Table C — the executed claims other sections owe

Not admission rows. They ride the same fixture because they need a provisioned
capsule, and they are reported separately so nothing conditional can reach the
verdict.

| owed by | claim | test |
|---|---|---|
| `sec-4` | read-once (`REQ-449` criterion 3) | `rewriting_doctrine_toml_inside_a_capsule_does_not_change_the_bound_policy` |
| `sec-3` | the clone's object set is the contracted history alone | `the_clones_object_set_is_exactly_the_exports` |
| `sec-5` | `SystemHost` reads real available space at the path it is given | `the_capacity_probe_reads_real_space_at_the_path_it_is_given` |
| `sec-5` | and reads the capsule root's filesystem, not the repository's | `the_capacity_probe_reads_the_filesystem_the_capsule_root_is_on` |

The object-set claim compares trusted-side: the capsule prints its object names
with `git -C /capsule/repo cat-file --batch-all-objects --batch-check='%(objectname)'`
and the trusted side compares the set against the same query run on the export.
The `--batch-check` is not optional — bare `--batch-all-objects` is a fatal
error (`'--batch-all-objects' requires a batch mode`), verified by execution.

**`sec-5`'s two rows split a claim that was one row and could not carry it.**
The first is unconditional: the fixture calls `SystemHost` on its own capsule
root and compares the figure against a `statvfs` the test performs itself on
the same path, requiring agreement within one allocation unit. That rules out a
host that never calls `statvfs`, one that returns a manufactured or cached
figure, and one that reads the wrong *kind* of quantity — and it needs no
particular disk layout, so it never skips.

It cannot rule out a host that probes the **wrong path**, because where the
capsule root and the repository share a filesystem the two figures are equal
and the defect is unobservable. The second row is the discriminator, and the
fixture does not wait for the operator's disk layout to supply one: it compares
the capsule root — which `DEC-156` already requires to be on real disk, never
tmpfs — against a path on a second filesystem it selects at fixture build time
from the host's own mounts, and asserts the two figures differ and each matches
an independent `statvfs`. On Linux a tmpfs is present at `/dev/shm` or `/tmp`
and the pair is available in practice; measured here, the real-disk root and
the tmpfs reported 272 GiB against 32 GiB. Where no second filesystem can be
found the row reports *skipped* naming that reason, which is a report about the
host rather than a silent pass.

`RV-346` `F-24` is why this is two rows. As one skippable row it was the only
executed evidence `REQ-461` had, and `sec-8` was closing the requirement while
explicitly disclaiming it — pure tests over a fixture `HostFacts` exercise
arithmetic and cannot observe whether the real probe was ever called.

**Table C may skip; tables A and B may not.** A Table C row contributes to no
admission, and the distinction is structural rather than a promise: `Admission`
is computed from tables A and B alone, so a Table C row has no path to it. That
is what makes a skip there lawful where `DEC-156` forbids one in an admission.

### The fixture

```rust
/// Built once, trusted-side, before any row runs.
struct Fixture {
    /// Real disk, never tmpfs (`DEC-156`): `sec-5`'s probe must read real
    /// available space, and a resource observation on tmpfs would measure the
    /// mount's size rather than the disk's. `Drop` removes it.
    root: TempRoot,
    /// A self-contained control-plane root: a git repository that supplies the
    /// contracted base, and a `.doctrine/doctrine.toml` carrying the
    /// synthesized `[capsule]` table. `provision` reads this, never the
    /// operator's project.
    project_root: PathBuf,
    base: AcceptedBase,
    /// The host regions this fixture's placements may never reach. Names
    /// `project_root`, its `.doctrine/`, and `capsule_root` — so the fixture's
    /// own canonical repository stands in for the real one, which is never
    /// referenced by any arm.
    scopes: ForbiddenScopes,
    capsule_root: PathBuf,
    /// Row 3's targets. Deliberately **not** members of `scopes`, so the row's
    /// control is a lawful widening.
    decoy_credential: PathBuf,
    decoy_repository: PathBuf,
    /// Row 9's targets: a readable decoy the row writes through, and this
    /// fixture's **own** export — built for this run, adopted by nothing else,
    /// so the control arm's writes cannot reach an export a real transaction
    /// shares.
    decoy_readable_input: PathBuf,
    own_export: PathBuf,
    /// Table C's second capacity row: a path on a filesystem other than the
    /// one `capsule_root` is on, chosen from the host's mounts at build time.
    /// `None` where the host offers no second filesystem, which makes that row
    /// report *skipped* naming the reason rather than passing quietly.
    second_filesystem: Option<PathBuf>,
    /// Row 4's target, and row 2's.
    decoy_undeclared: PathBuf,
    decoy_executable: PathBuf,
    /// Trusted-side, row 5's target.
    listener: TcpListener,
}
```

**One transaction per capsule per arm.** A capsule that left state behind would
contaminate the next, and the `Sequential` and `Concurrent` shapes need two
within a single arm. The cost is bounded by `sec-3`'s export being built once
per base and adopted by every transaction after it, so a transaction is a local
clone from a warm export.

**Cleanup is the harness's.** `TempRoot`'s `Drop` removes the run's root.
`DEC-156` is explicit that this must not mint a product-side capsule-delete
capability for test convenience — `DEC-133` and `DEC-137` hold that a harvested
capsule is live work, and a delete primitive introduced here for tidiness is
what a later slice would reach for.

### Hazard containment, per row

| row | hazard | containment |
|---|---|---|
| 3 | reaching real canonical state or credentials | decoys only, inside the fixture's own root; the operator's repository and credentials are named by no arm and bound under neither |
| 5 | contacting the network | a trusted-side loopback listener, never the internet — measured refused under `--unshare-all`, so the row holds offline |
| 7, B5 | a leaked process | `timeout -k` outside the sandbox plus a **session** kill in teardown, not a process-group kill — see below (`EVD-013`) |
| 8 | filling the disk, or hanging | the unbounded write goes to the capsule's size-capped **transient** tmpfs at `/tmp`, never the retained scratch on host disk; the wall payload sleeps a small fixed multiple of the bound |
| 9 | the control arm writing to real host state | its readable entries are fixture-owned decoys under the fixture's own root, and its `/source` is an export this run built for itself — never one a real transaction adopts |
| 10 | the control arm handing a capsule a live descriptor | the descriptor is opened onto a fixture-created decoy file under the fixture's own root; no descriptor the trusted side holds for real is ever marked inheritable |
| 11 | the control arm leaking the trusted-side environment | the assertion is over a decoy variable the fixture sets in its own child, so nothing the operator's shell carries is read, compared or reported |

**Row 7's containment changed in round 4 and the reason generalises.** It read
*a process-group kill in teardown (`sandbox.sh:288`)*, which was containment for
the payload row 7 used to have. `RV-346` `F-27` strengthened that payload to a
descendant that leaves the original session — precisely so a process-group-only
backend cannot pass — and a process-group kill therefore cannot reap the
survivor its own control arm creates. The fixture records the capsule's session
id trusted-side before the arm runs and kills the session on the way out.

The general rule, because this will recur: **when a row's payload is
strengthened, its containment is part of the payload.** A harness reaping by the
mechanism the row exists to defeat is not containment, and the failure is
silent — a leaked process per run, discovered by a developer whose machine is
slowly filling with them rather than by a red test.

### The verdict

```rust
pub(crate) struct AdmissionVerdict {
    pub(crate) backend: BackendId,
    pub(crate) host: HostDescriptor,
    pub(crate) date: Date,
    pub(crate) outcome: Admission,
    /// Every row of tables A and B, including the proven ones — the artefact
    /// `REQ-459` criterion 3 points at.
    pub(crate) rows: Vec<(RowId, RowVerdict)>,
    /// Table C. Reported, never admitted on.
    pub(crate) auxiliary: Vec<(Claim, AuxOutcome)>,
}

pub(crate) enum Admission {
    /// Every row in tables A and B is `Proven`.
    Admitted,
    NotAdmitted { reason: NotAdmitted },
}

pub(crate) enum NotAdmitted {
    /// `POL-002` facet 3: what is missing and what would satisfy it. Read from
    /// `CapsuleBackend::availability`, or from the fixture failing to find a
    /// usable shell, before any row runs.
    Unavailable { missing: String, remedy: String },
    /// At least one row was not `Proven`.
    Rows,
}

/// A table C claim and the section that owes it.
pub(crate) struct Claim { pub(crate) section: &'static str, pub(crate) name: &'static str }

pub(crate) enum AuxOutcome { Passed, Failed(String), Skipped(String) }

/// The host an admission verdict was recorded on.
pub(crate) struct HostDescriptor { pub(crate) os: String, pub(crate) kernel: String, pub(crate) arch: String }

/// A directory removed on drop. Test-support, in this module.
struct TempRoot(PathBuf);
```

There is exactly one green path and it requires every row to have been run. An
unavailable backend takes `NotAdmitted::Unavailable` and exits nonzero, which is
`DEC-156`'s "never a green skip" made structural rather than remembered.

**The test asserts `Admitted` unconditionally.** Making it conditional on
availability would reintroduce precisely the green skip, so it does not: on a
host that cannot run the backend, this project's `cargo test` fails. That is a
real cost and it is accepted rather than hidden — the supported development
environment is Linux with bubblewrap, and `DEC-160` verified that nested
bubblewrap works inside this project's jail, re-measured while remediating
`RV-346` round 3: `--unshare-all`, `--unshare-net` alone, and the explicit
non-network unshare set all succeed one level down.

**What that measurement does not cover is a third layer.** `RV-346` `F-20`
reported the exact nested profile failing with `Failed to create NETLINK_ROUTE
socket: Operation not permitted`, and the reading it drew — that this project's
jail cannot run the suite — does not hold: the same commands succeed in that
jail, which is the positive control the claim needed. The observation was real
in the environment it was made in, an agent sandbox wrapping the jail whose
seccomp filter denies the socket bubblewrap opens to bring up loopback in a
fresh network namespace. The general fact survives the specific claim's
withdrawal: **any additional confinement layer that denies network-namespace
setup makes row 5's probe arm, and therefore admission, impossible** — and a
seccomp-filtered CI runner is that layer as much as an agent sandbox is.
`backend verify` is the path for such a host, reporting
`NotAdmitted::Unavailable` naming what is missing, and `sec-9` residual 3
carries the consequence for `cargo test` — which is not a macOS-only concern,
as an earlier draft of that residual assumed.

### Invariants

1. **Admission requires every row to have run.** No skip, no early return, and
   no conditional reaches `Admission::Admitted`; it is computed from tables A
   and B alone.
2. **Every property carries a control that was seen to fail.** A row whose
   control still held is `Unproven`, which is not admitted.
3. **A control differs from its probe by exactly one typed `Delta`,** and both
   arms run the same `ArmShape`. Mount-set deltas additionally prove their
   lawfulness by passing the same validating constructor.
4. **A probe arm's placement is what `provision` returned,** unmodified.
5. **No arm result is read before liveness is established.** `Failed` is a
   positive observation, never the absence of one.
6. **Nothing weakens through `CapsuleBackend`.** The production trait has no
   weakening surface; `ConformanceBackend` is a separate trait and is
   `pub(crate)`.
7. **Hazardous rows reach decoys** inside the fixture's own root. No arm names
   the operator's repository or credentials, and no arm — probe or control —
   can write to an export any real transaction adopts: row 9's writable-input
   control operates on an export this run built for itself.
8. **The suite creates no capsule-delete capability.** Cleanup is the fixture's
   own `Drop` over a temporary root it created.
9. **A backend's own report is never evidence.** Every arm result is read from
   the `Observation` the trusted parent returns — `REQ-448` criterion 3.
10. **A live pid is the trusted parent's observation.** Row B5's target comes
   from `execute_observed`'s callback, never from the subject's own output, and
   a backend that returns without calling it yields `Indeterminate` rather than
   a held probe.

### Verification alignment

The suite is the executed evidence for the rest of the design, so what *this*
section owes is tests of the part that can be wrong without any capsule
running: classification. `ArmResult` and `RowVerdict` are pure functions, and
they are where a suite silently degrades into one that always passes.

Pure, over classification:

- `no_liveness_marker_is_indeterminate_not_failed` — over all three `Observed`
  kinds, the `F-5` class
- `an_arm_specific_breakage_on_the_control_arm_does_not_yield_proven` — the
  false-green path stated directly
- `both_tokens_present_is_ambiguous_not_held`
- `a_missing_value_line_is_indeterminate_rather_than_unequal` — row 6's shape
- `a_termination_observation_reads_its_marker_from_a_killed_run`
- `an_observed_execution_that_never_calls_back_is_indeterminate_not_held` — the
  backend that did not implement the B5 seam, which must not read as a pass
- `a_subject_that_exited_before_the_observer_ran_is_indeterminate`

Pure, over the verdict algebra:

- `held_probe_and_failed_control_is_proven`
- `failed_probe_is_violated_even_when_the_control_failed`
- `held_control_is_unproven_rather_than_proven`
- `an_indeterminate_arm_is_never_proven` — over both arms
- `a_backend_ignoring_its_removal_yields_unproven_for_every_row` — the
  fails-closed claim, against a stub whose `execute_weakened` delegates to
  `execute`
- `admitted_requires_every_row_proven` — one row of each non-proven kind
- `an_unavailable_backend_is_not_admitted_and_runs_no_row`
- `a_host_without_a_usable_shell_is_unavailable_not_violated`
- `auxiliary_outcomes_do_not_reach_the_admission`
- `a_skipped_auxiliary_claim_leaves_the_verdict_admitted`
- `the_verdict_names_backend_host_and_date`
- `every_row_id_is_covered_by_exactly_one_table`

Executed, table A — each asserts `RowVerdict::Proven`, so each runs both arms:

- `fresh_mutable_state_is_proven`
- `bounded_input_set_is_proven`
- `denial_of_canonical_state_and_credentials_is_proven`
- `bounded_filesystem_visibility_is_proven`
- `explicit_network_posture_is_proven`
- `deterministic_working_directory_is_proven`
- `process_tree_teardown_is_proven`
- `resource_and_termination_observation_is_proven`
- `immutable_input_set_is_proven`
- `closed_descriptor_set_is_proven`
- `closed_environment_is_proven`

One title per row, and the list is asserted to be exactly the row set by
`every_row_id_is_covered_by_exactly_one_table` above — so a row added without a
title, or a title outliving its row, is a red test rather than a silent gap.
That assertion is what makes the count safe to state in one place.

Executed, the claims that need naming beyond their row:

- `a_binary_is_executed_from_each_bound_path` — row 2's `F-P05-17` half: the
  shebang class was found by running the project's own suite, not by inspecting
  the profile (`sec-2`)
- `every_termination_variant_is_distinguished` — row 8's five payloads
- `not_executable_is_distinct_from_exit_127`
- `not_executable_is_preceded_by_a_liveness_execution_in_the_same_capsule`
- `the_working_directory_does_not_track_the_trusted_side_cwd` — row 6 from two
  trusted-side cwds
- `the_descendant_escapes_the_original_session_before_its_parent_exits` — row
  7's payload precondition, asserted directly so a payload that silently stops
  escaping cannot weaken the row without failing (`F-27`)
- `no_descendant_outlives_the_execute_call` — row 7's probe arm, established
  trusted-side against the escaped descendant
- `a_process_group_only_reaper_fails_row_seven` — the `F-27` mutant itself, as a
  stub backend: it reaps the original process group and nothing else, and the
  row must report `Violated`. Without this, the strengthened payload is a claim
  no test defends
- `the_orphan_left_by_a_teardown_or_visibility_control_is_reaped_by_the_harness`
  — by session kill, not by process-group kill, which is the containment that
  had to move with the payload
- `a_probe_arm_placement_is_byte_identical_to_what_provision_returned` —
  invariant 4, which nothing else would catch
- `the_shared_root_delta_repoints_only_the_second_placement`
- `a_write_through_every_readable_mount_fails` — row 9's probe stated per
  entry, so a backend binding one entry read-only and another writable cannot
  pass on the first
- `a_write_into_the_source_export_fails` — the `DEC-157` half specifically
- `the_writable_inputs_delta_changes_no_mount_and_no_path` — that the control
  differs by attachment alone, which is what makes it a single delta
- `the_writable_inputs_control_writes_only_to_this_runs_own_export`
- `the_observed_pid_is_the_one_the_parent_reported_not_one_the_subject_printed`
- `no_descriptor_above_two_is_readable_in_the_capsule` — row 10's probe arm,
  enumerated from `/proc/self/fd` rather than guessed at by number
- `the_inheritable_decoys_bytes_appear_nowhere_in_the_capsules_output` — row
  10 stated as the thing that actually matters, so a backend that leaves the
  descriptor open but unreadable is still distinguished from one that closes it
- `the_descriptor_control_changes_no_mount_no_env_and_no_argv_byte` — that row
  10's delta is a single axis, the same assertion row 9 carries
- `a_descriptor_leaking_backend_fails_row_ten` — the `F-26` mutant as a stub:
  identical on every other observation, one inherited descriptor, must report
  `Violated`
- `the_capsule_environment_equals_capsule_env_exactly` — row 11's probe arm, set
  equality rather than a denylist of known credential names
- `a_trusted_side_variable_does_not_appear_in_the_capsule` — the decoy half
- `an_environment_passthrough_backend_fails_row_eleven` — the row 11 mutant as a
  stub

Executed, table B: the freshness titles `sec-3`'s `Verification alignment`
already names, driven by this harness rather than restated here.

Executed, table C: the three titles above.

**These tests live in the new crate,** which `just check` would not build by
default — it is root-package only, and a suite that is never built is green by
never running. `sec-8` rules on the checked set and owns the change.

`REQ-459` criterion 1 is discharged by table A in full, which as of `RV-346`
`F-19` means *including row 9* — without it the suite admitted a backend that
bound every declared input writable, and criterion 1 was not discharged at all.
Criterion 2 —
bubblewrap becoming *the supported* backend — needs production acceptance tests
this slice does not have, so it is recorded as a contributing `--change` and
reported **partial** in the reconciliation brief, the same shape `sec-3` uses
for `REQ-450`. Criterion 3 is discharged structurally: there is one suite, it is
parameterised by backend, and a second backend passing it edits nothing.


<!-- doctrine:section sec-8 -->
## Code impact and verification alignment

The touch-set in one place, the checked-set ruling `sec-7` deferred here, and the
evidence rollup — what is net-new, what must stay green unchanged, and which
requirements this slice can and cannot close. Each section's own `Verification
alignment` is the authority on its test titles; this restates none of them and
records only what sits between sections.

### The touch-set

New, in the root package:

| path | what it is | owner |
|---|---|---|
| `src/lib.rs` | the lib target and its entire export set — five items behind three private modules | `sec-6` |
| `src/config_file.rs` | `DOCTRINE_TOML` and `read_doctrine_toml_text`, relocated out of `dtoml` so they can cross a crate boundary; out-edge-free, leaf | `sec-6` |
| `src/interpretation.rs` | the `[interpretation]` typed projection: parse, normalize, canonical hash, restriction algebra; leaf, beside `dtoml`, the `reserve.rs` shape | `sec-4` |

New, the second crate — a bin-only package, no lib target (`sec-6`):

| path | what it is | owner |
|---|---|---|
| `crates/doctrine-control/Cargo.toml` | path dependency on the `doctrine` lib target, its own `rustix` edge (`fs` **and** `std`, not inherited from the root package), and `publish = false` | `sec-6` |
| `crates/doctrine-control/src/main.rs` | the two verbs, `provision` and `backend verify` | `sec-6` |
| `crates/doctrine-control/src/host.rs` | `HostFacts`, `SystemHost` | `sec-5` |
| `crates/doctrine-control/src/config.rs` | `CapsuleConfig` and the `[capsule]` reader | `sec-5` |
| `crates/doctrine-control/src/capacity.rs` | `CapacityVerdict`, `assess_capacity` | `sec-5` |
| `crates/doctrine-control/src/backend.rs` | `CapsuleBackend`, `CapsulePlacement`, `Execution`, `Observation`, `Termination`, `ForbiddenScopes` | `sec-2` |
| `crates/doctrine-control/src/backend/bubblewrap.rs` | the Linux profile, its twelve flag constants and its argv | `sec-2` |
| `crates/doctrine-control/src/transaction.rs` | `CapsuleTransaction`, `TransactionId`, `AcceptedBase`, `PhaseIdentity` | `sec-3` |
| `crates/doctrine-control/src/provision.rs` | `provision`, export publication, root ownership, rollback | `sec-3` |
| `crates/doctrine-control/src/conformance.rs` | the property table, the fixture, the verdict, and the second caller | `sec-7` |

Modified — and this is the whole of it, which is `sec-6` invariant 2 stated as a
diff rather than as a claim:

| path | change | why it is small |
|---|---|---|
| `src/git.rs` | `read_path_at` and `CaptureError` `pub(crate)` → `pub` | compiler-required: `pub use` of a `pub(crate)` item is `E0364` |
| `src/dtoml.rs` | two items removed, re-exported from `config_file` under their existing names | all 35 call sites across 33 files are untouched |
| `src/main.rs` | one `mod config_file;` declaration | the binary's `dtoml` reaches the relocated items through it |
| `Cargo.toml` | `crates/doctrine-control` joins `[workspace] members`; a `default-members` key is added; `include` gains `!/src/lib.rs` | see the checked set below, and `sec-9` `R7` for the exclusion |
| `Cargo.lock` | one package entry for the new member | generated, but committed and release-owned here |
| `justfile` | `publish` becomes `cargo publish -p doctrine`; `pkg-check` gains the assertion that `src/lib.rs` is absent from the packaged source | `sec-6` § Nothing ships; `sec-9` `R7` |
| `tests/architecture_layering.rs` | `check`'s module filter takes its source directory as a parameter; the gate runs twice; the export-set assertions are added | `discover_units` and `extract_edges` are already parameterised |
| `.doctrine/adr/001/layering.toml` | `config_file = "leaf"`, `interpretation = "leaf"`, the edge `dtoml → config_file`, and a second section for the new crate's eight units | unit names are unique per tree, so the new crate's map is not merged |
| `.doctrine/doctrine.toml` | the `[capsule]` table | this project's own operator configuration, not a platform default |

Three of those rows are `RV-346` round 3's. `Cargo.lock` was missing (`F-22`):
adding a workspace member rewrites it whether or not a registry dependency
arrives, `cargo build --locked` refuses against a stale one, and the crane build
consumes it. The `justfile` and `include` rows are `F-21` and `F-23`, and both
correct a claim made two paragraphs below this table in an earlier draft — that
the release arrangement does not change. It does, in two small places, and the
reason it does is that `default-members` reaches further than the checked set.

`src/main.rs` does **not** declare `mod interpretation;`. Nothing in the
agent-facing binary consumes the policy in this slice — `doctrine-control`
reaches it through the lib target — and declaring it in both targets to no
purpose would compile it twice for nothing. It is still classified in
`layering.toml` and still walked by the gate, which discovers units from the
file tree rather than from `mod` declarations, so it acquires no exemption by
being absent from the binary.

### What does not change, and why that is worth stating

**`src/worktree/` is untouched.** Not merely un-imported — unedited. `DEC-155`
gives the bubblewrap backend its own flag constants and its own profile builder,
so `bwrap_core_argv` and its byte-parity contract with
`scripts/pi-spawn-confined.sh` are outside this slice's diff entirely.

That retires the slice's authored risk `R4` — the parity contract as a second
behaviour-preservation obligation — and narrows `R2`, which named `jail.rs` as
the shared machinery this slice disturbs. `R2` survives with a different
subject: the shared machinery this slice actually changes is `dtoml`'s two
relocated items and the layering gate's directory parameter. Both are covered by
existing suites, which is the point of doing them behind a re-export and behind
a parameter. `sec-9` carries the corrected reading.

Also unchanged: every incumbent dispatch, worktree, marker and confinement
suite; `src/commands/`; `src/dispatch_config.rs`; and the *distribution* half of
the release arrangement — `flake.nix`, `install.sh`, `release.yml` — which
`sec-6` defers to whichever slice first releases the second binary. The two
release-adjacent paths that do change (`justfile`'s publish recipe and the
`include` allow-list) are in the table above; they are about not shipping the
new crate and not widening the old one, which is the opposite of a distribution
contract.

### The checked set

`sec-7` left one obligation on this section: the new crate's tests are green by
never running unless something brings it into the checked set. `just check` runs
`fmt`, `lint`, `build`, `validate`, `test`. Three of those — `lint`, `build`,
`test` — resolve to a bare `cargo` invocation that, with a package at the
workspace root and no `default-members`, selects the root package alone.
`crates/cordage` is outside the fast loop today for exactly this reason, and a
new crate would inherit it — including `cargo clippy`, which would leave the new
crate unlinted under `just gate` too, since `lint` is root-package-only in both
recipes.

**`fmt` is not one of the three.** `cargo fmt` walks every workspace member
regardless of `default-members` — measured, against a member excluded from the
default set, which it formatted anyway (`RV-346` `F-21`). An earlier draft of
this section listed `fmt` among what the ruling below extends, and it was wrong
twice: formatting already reaches the new crate, and `cordage` was never outside
that leg of the fast loop.

**The ruling: `default-members = [".", "crates/doctrine-control"]`.** One key,
in the workspace `Cargo.toml`, and `lint`, `build` and `test` reach the new
crate with no per-recipe flag. `cordage` stays out of those three exactly as
today; `test-all`'s `--workspace` is unchanged and still covers all three
packages.

**It is not the whole of the change, which is the other half of `F-21`.**
`default-members` is a *package selection* default, not a build-command one, so
it also selects the new crate for `cargo package` and `cargo publish` — and
`just publish` runs the bare form. `sec-6` § Nothing ships carries the two-part
answer (`publish = false` in the new manifest, and `-p doctrine` on the recipe)
and the measurement showing why each alone is insufficient. Both edits are in
the touch-set above.

Its consequence is honest and belongs here rather than in a phase note: the
executed conformance suite joins the fast inner loop. Tables A, B and C are on
the order of fifty capsule provisions and executions per run — each a local
clone from an export built once per base and adopted thereafter — plus two
payloads that sleep a small fixed multiple of a fixture-chosen bound. The
fixture sets that bound, so the wall clock is bounded by construction rather
than by the operator's `[capsule]` table, but it is not free.

**If measurement makes the inner loop unusable, the lawful adjustment is a
filter on `test:` alone** — `cargo test -- --skip <executed module path>` — with
`test-all` unfiltered, so the commit gate still runs every row. This is not the
green skip `DEC-156` forbids, and the distinction is worth being exact about:
`DEC-156` forbids an *admission verdict* reaching `Admitted` without every row
having run, which `sec-7` makes structural in `Admission`. A test binary that
some recipe does not invoke produces no verdict at all. What is not available is
`#[ignore]` on the admission test, which would put the skip inside the default
run where a reader would take its absence for a pass.

The measurement is a phase obligation, not a design one. The default is
unfiltered.

### Where the new evidence lives

The new crate has no lib target, so it has no `tests/` directory — a `tests/`
file cannot link a bin-only package (`E0433`), and `sec-6` declined the lib
target that would rescue it rather than publish `sec-7`'s weakening vocabulary.
Every test in `doctrine-control` is therefore a `#[cfg(test)]` module inside the
unit it tests, which reaches `pub(crate)` items and does run under `cargo test`.

| evidence | lives in | kind |
|---|---|---|
| `sec-2`'s argv, closure and placement-validation tests (≈37 titles) | `backend.rs`, `backend/bubblewrap.rs` | pure |
| `sec-3`'s export, clone, rollback and ownership tests (≈22) | `provision.rs`, `transaction.rs` | pure and trusted-side |
| `sec-4`'s parse, normalization, hash and restriction tests (≈42) | `src/interpretation.rs`, root package | pure |
| `sec-5`'s capacity, configuration and root-resolution tests (≈33) | `config.rs`, `capacity.rs`, `host.rs` | pure, over a fixture `HostFacts` |
| `sec-6`'s export-set and two-tree gate assertions (7) | `tests/architecture_layering.rs` | integration, root package |
| `sec-7`'s classification and verdict-algebra tests (≈20) | `conformance.rs` | pure |
| `sec-7`'s tables A, B (five) and C (four) | `conformance.rs` | executed, one fixture per run |

The counts are indicative for phase sizing; each section's own list is the
authority, and two titles are named by two sections and belong to one test.
`sec-2`'s figure moved by four in round 4 — the source-export carve-out's
refusals and the lawful case each must not capture (`F-25`) — and table A's
executed cost by two arms plus two controls (`F-26` and the channel ledger).
Neither is a large phase-sizing move; both are recorded because a count nobody
adjusts is a count nobody is reading.

`sec-6`'s assertions land in `tests/architecture_layering.rs` rather than a new
file because the export set is the cross-crate half of the same rule the file
already enforces, and because `tests/` can now link the root package's lib
target — which it could not before this slice, and which is the only reason the
export set is assertable at all.

### What must stay green unchanged

The behaviour-preservation obligation (AGENTS.md; `R2` as corrected above) has
four subjects, and each is proven by a suite written for something else:

1. **The 35 `DOCTRINE_TOML` / `read_doctrine_toml_text` call sites.** The
   relocation is behind re-exports under the existing names, so no call site
   changes; any behavioural difference fails tests that never heard of this
   slice.
2. **The layering gate's verdict over `src`.** Parameterising `check`'s module
   filter must move no existing classification —
   `the_existing_layering_gate_is_unchanged_in_verdict_over_the_root_tree` is
   the assertion, and it is the reason the parameter is added rather than the
   extractor generalised.
3. **The incumbent dispatch and confinement suites**, including the pi-spawn
   byte-parity test, which pass unchanged because nothing they cover is edited.
4. **The `dtoml` cascade stays inside the binary.** `dtoml` is not exported and
   must not become exportable; the export-set assertion is what notices if a
   later change tries.

One new gate arrives with the lib target and is easy to miss: **`cargo test` now
builds the library's own test targets.** The export set must be transitively
closed over `crate::` paths in `#[cfg(test)]` code as well as production code —
`git.rs`'s test module reaches `crate::kinds`, so `kinds` is declared privately
in `lib.rs` as a path target rather than as an export. `cargo build` is green
while this is wrong. It surfaces only when the library's tests first compile,
which is a `just check` away rather than a release away, but only because the
crate is in the default set.

### Requirement closure

The slice's authored closure intent expects `REQ-449`, `REQ-459` and `REQ-461`
to move `pending → satisfied`. The design agrees on two of the three and
**corrects the third**: `sec-7` establishes that `REQ-459` criterion 2 —
bubblewrap becoming *the supported* backend — needs production acceptance
evidence this slice does not produce. The correction is owed to the
reconciliation brief, alongside `DEC-136`'s handoff note (`sec-1`, `sec-4`).

| requirement | this slice | evidence |
|---|---|---|
| `REQ-449` | `satisfied` | `sec-4`'s refusal, normalization, hash and restriction tests; criterion 3 by `sec-7` table C's read-once row |
| `REQ-461` | `satisfied` | `sec-5`'s pure tests for the arithmetic and the configuration, **plus** table C's unconditional row — `SystemHost`'s figure agrees with a `statvfs` the test performs itself on the same path. The wrong-path discriminator is a second, conditional row |
| `REQ-459` | **contributing `--change`, stays `pending`** | criterion 1 by table A in full — every channel in `sec-2`'s ledger rowed, rows 9, 10 and 11 included; criterion 3 structurally, one suite parameterised by backend; criterion 2 unmet |
| `REQ-450` | contributing `--change`, stays `pending` | criterion 1 by table B. Criteria 2 and 3 need candidate identity and harvest from later slices |
| `REQ-448` | contributing `--change`, stays `pending` | the *denial* half only, and it is a claim per channel rather than per row: canonical state and credentials in the mount channel by row 3, arbitrary undeclared paths by row 4, the shared object store by rows 3 and 9 together — reachable-but-not-writable is not denial — egress by row 5, and credentials reaching the capsule *already open* or *already in the environment* by rows 10 and 11, which are the two channels this table cited nothing for before round 4 |

**Rows of this table have moved in each of the last two rounds, and always in
the same direction.** `REQ-461` could not have been `satisfied` as the table
first read it (`F-24`): its only executed evidence was a row that skips wherever
the capsule root and the repository share a filesystem, and the pure tests run
against a fixture `HostFacts` that cannot observe whether the real probe is ever
called. `sec-7` splits that claim into an unconditional leg and a discriminating
one, and closure now rests on the first.

`REQ-459` criterion 1 has now been claimed *in full* three times against three
different tables. It was false before row 9 (`F-19`, table A admitted a backend
binding every declared input writable) and false again before rows 10 and 11
(`F-26`, table A admitted a backend leaking an inherited descriptor, and the
channel ledger then found the environment unrowed as well). Each time the phrase
was a true statement about a table that was missing a row.

That is worth stating plainly rather than quietly fixing a third time: **the
phrase *in full* is only as strong as the enumeration it quantifies over**, and
until round 4 this design had no written enumeration for it to quantify over at
all. `sec-2`'s channel ledger is now that enumeration, and *in full* above means
*every channel in the ledger has a row* — a claim that can be checked against
something, and that fails visibly if a channel is added without one.

Coverage records name the discharging test per criterion (`doctrine coverage
record`), and the reconciliation brief reports `REQ-448`, `REQ-450` and
`REQ-459` as partial in those words rather than leaving a reader to infer it
from a `pending` status.


<!-- doctrine:section sec-9 -->
## Risks, residuals, and what stays open

The design's own account of what it did not settle. The slice's scope authored
`R1`–`R3` and its assumptions `A1`–`A3`; the pre-design triage added `R4`–`R5`
and `A4`. This section adjudicates each against the design that now exists —
one of them is retired, one changes subject, one is discharged — adds the two
the design itself created, and records the five residuals a reader implementing
from this document would otherwise meet unannounced.

### The authored risks, as they now stand

**`R1` — evidence altitude. Stands, unchanged, and binds this document.**
`SL-241` is Linux, bubblewrap, one client shape, `n = 1` on the real-agent leg:
feasibility evidence, and not performance, portability or production-readiness
evidence. Every measured claim in `sec-2`, `sec-3`, `sec-5` and `sec-7` is
attributed where it was measured, and none is generalised past it. The "16/16"
summary stays forbidden — fifteen rows reached model level, the env-file row is
unproven beyond the Rust fixture, structural `n/a` cells are not omissions, and
four `fail` rows are successful mutant detections. `R1` is also why `REQ-459`
criterion 2 cannot close here (`sec-8`).

**`R2` — behaviour preservation. Stands, with a different subject.** As
authored it named `src/worktree/jail.rs` as the shared machinery this slice
disturbs. Under `DEC-155` the bubblewrap backend is self-contained, and `sec-8`
records that `src/worktree/` is not merely un-imported but unedited. The
obligation survives against what this slice does change: `dtoml`'s two
relocated items, behind re-exports, and the layering gate's directory
parameter. Both are proven by suites written for something else, which is why
they take those shapes.

**`R3` — a property suite is only as good as its adversary. Stands, and is the
risk `sec-7` exists to answer.** One-property-removed controls against decoy
targets, liveness established before any observation is read, `Failed` as a
positive observation rather than the absence of one, and a verdict algebra in
which a control that still held yields `Unproven`. The residual is structural
rather than a matter of care: rows 3 and 4 share a denial mechanism and are
independent only because they reach for different things, and the rule that
kept them honest — *a row's control removes the mechanism unique to that row's
property* — is a rule a later row can be added in violation of. A property with
no unique mechanism cannot be controlled independently, and the rows must then
be re-cut rather than the control widened.

`R3` has now been realised **three times** rather than merely feared, and every
time by the external pass rather than by the author. `F-2` found two clauses
merged into one row. `F-19` found one clause carrying two claims of which only
one had a row — a suite that admitted a backend binding every declared input
writable. `F-26` found a clause-2 channel with no row at all, and writing the
fix found a second one beside it. None was a careless omission; each read as
complete until someone executed the mutant the suite did not have.

The standing form of this risk is that **the gap in a property suite is
invisible from inside it**, and the only reliable detector is an adversary
constructing the backend the suite would wrongly pass. Three data points now say
the same thing about *where* the gaps are: all four missing rows were clause 2,
and clause 2 is the clause that names an outcome without naming the mechanisms
that reach it.

`sec-2`'s **channel ledger** is round 4's structural answer, and it is worth
being exact about what it does and does not buy. It does not make the suite
complete. It changes the failure from *nobody thought of this channel* to *this
channel has no row*, which is a blank cell in a table someone can read. That is
a real improvement over the previous state, where the enumeration existed only
in whichever author's head last wrote a row, and it is strictly weaker than a
proof. The residual below states what it leaves open.

**`R4` — the `bwrap_core_argv` parity contract. Retired.** Its premise was that
this slice widens the shared bubblewrap builder. `DEC-155` gives the capsule
backend its own flag constants and its own profile, so the byte-parity test
against `scripts/pi-spawn-confined.sh` is outside this slice's diff. Nothing
here can fail it.

**`R5` — the distribution contract. Stands, deferred, and named as a
Follow-Up.** `doctrine-control` is built and not released, so the nix
`srcWithDist` graft, the binstall asset name, `install.sh` and `release.yml`
move together for whichever slice first ships it. `POL-002` is why that is
named rather than done here: those four artefacts are this project's own
release arrangement, and the platform does not acquire a release step because
one of its slices produced a second binary.

### Two risks this design created

**`R6` — the double compilation is safe only while the binary touches no
library type.** `src/main.rs` keeps its own module tree and `src/lib.rs`
declares three of the same modules, so `git`, `config_file` and `kinds` compile
twice. The two copies never meet today because nothing in the binary names a
`doctrine::` path. The day something does, `doctrine::git::CaptureError` and
`crate::git::CaptureError` become distinct types with identical names, and the
resulting mismatched-type error names one type twice. The alternative —
`main.rs` as a thin binary over the library — was rejected in `sec-6` because it
publishes most of the product as library API (110 `pub(crate)` items in `git.rs`
alone, 203 in `memory.rs`), so this is the accepted side of a considered trade
rather than an oversight. The mitigation is the rule, not a test: the binary
does not import from its own library.

**`R7` — the `doctrine` package would otherwise acquire a public library API,
and the design declines it.** `src/lib.rs` falls inside the published `include`
allow-list, so on the next release of `doctrine` the five exported items would
become semver surface any downstream consumer could depend on — for no benefit,
since the only consumer that needs them is `doctrine-control`, which lives in
this workspace and is never published.

A first draft of this risk accepted that exposure, on the ground that excluding
the lib target would mean replacing the `include` allow-list with a file
enumeration. **That premise was false.** `include` takes gitignore-style
patterns, negation included, so the exclusion is one line —
`include = ["/src/**", …, "!/src/lib.rs"]` — measured on a minimal package
while remediating `RV-346` `F-23`: `cargo package --list` omits `src/lib.rs`
and `cargo package` completes, warning that the library was ignored because its
source was not included.

So the design takes the exclusion. What remains is a genuine tradeoff rather
than a cost: **the published crate differs from the built one**, having a bin
target where the workspace package has both. That is tolerable because nothing
downstream builds `doctrine-control` from a published `doctrine`, and because
`cargo install doctrine` and `cargo binstall` want the binary and nothing else.
It is not free of hazard — a divergence between packaged and local source is
the same shape as the crane embed-strip trap that shipped a hollow binary at
`v0.5.0` — so the exclusion is asserted rather than trusted: `pkg-check`, which
already asserts that force-included embed roots survive packaging, gains the
opposite assertion for this one path.

The export-set test keeps its job either way. It is not there to protect
crates.io consumers who now do not exist; it bounds what `doctrine-control` can
reach across the workspace boundary, which is `sec-6`'s enforcement ruling.

### Assumptions

`A1` (`SPEC-030` and `ADR-020` win where this design disagrees), `A2` (`REV-046`
stays proposed and unapplied throughout) and `A3` (the existing
`.doctrine/doctrine.toml` reader is extended, not forked) stand as authored, with
one refinement: `A3`'s *extended* is `sec-4`'s typed projection beside the shared
reader, which is what `DEC-136`'s intent admits and what its handoff note
mis-describes.

`A4` is **discharged**. `git::read_path_at` was to be verified as the whole
impure surface `REQ-449`'s resolution needs; it was, at point of use, and
`sec-6` exports it on that basis.

### The five residuals

Each of these is a thing this design knows and does not fix. They are recorded
here rather than solved because solving them needs either a second backend, a
later slice, or a mechanism the backend contract cannot currently express.

**1. The resolution-time race.** Declared readable paths are fully resolved,
validated and then bound, and the window between validation and bind is real: a
path re-pointed inside it would bind a target that was never validated. It is
not capsule-reachable, so it is a control-plane-side race over operator-owned
configuration, and an operator who can re-point a declared path can also edit
the configuration that declares it.

**That dismissal rests on an executed claim, and until `RV-346` `F-19` it did
not.** An earlier draft cited `sec-2` invariant 4 — the canonical repository and
credentials are absent from the mount set — which is the wrong invariant: it
says nothing about whether the paths that *are* present can be written through,
and that is precisely what the race needs. Nothing in the suite proved it, and a
backend binding declared inputs writable would have passed every row. `sec-2`
invariant 11 now states the property and `sec-7` row 9 proves it, so the
dismissal stands on evidence rather than on an invariant that did not cover it.

Closing the race properly still means binding against an open descriptor rather
than a path, and whether the backend contract can express that at all — for
bubblewrap and for any later backend — is unanswered. Recorded, not solved.

**2. Out-of-crate backends break `sec-7`'s sealing, and the surface grows each
round.** `ConformanceBackend` and its weakening vocabulary are `pub(crate)`,
which is what stops a production backend from carrying a way to weaken itself.
That holds only while every backend lives inside `doctrine-control`. A backend
shipping from another crate would make the weakening vocabulary public API, and
the shape that survives it is a newtype over a private enum — deliberately not
built here, because building an extension point for a second backend that does
not exist is how the first one gets designed wrong.

What has changed is the size of what would be exposed. `ConformanceBackend`
carries two methods, and `PropertyRemoval` now carries seven variants, two of
them added in round 4. The trend is the point: every count correction widens
this residual, so the cost of deferring the seal rises with each round rather
than staying fixed. It is still the right deferral — the sealing shape is
decided by the second backend's needs and there is no second backend — but a
later slice inheriting this should expect a larger vocabulary than this one
described, not the same one.

**3. A host that cannot run the backend cannot run this project's tests.**
`sec-7`'s admission test asserts `Admitted` unconditionally, because
conditioning it on backend availability reintroduces exactly the green skip
`DEC-156` forbids. So on such a host `cargo test` fails — and `sec-8` sharpens
this by putting `doctrine-control` in `default-members`, which brings the
failure forward into `just check`.

**The affected set is wider than a first draft of this residual assumed.** It
named macOS, on the reasoning that Linux-with-bubblewrap is the supported
environment and nested bubblewrap was verified inside this project's own jail.
Both of those remain true — re-measured while remediating `RV-346` round 3 —
but they are not the whole population. `F-20` reported the nested profile
failing to create its network namespace, and although the specific claim did not
survive its positive control (the same commands succeed in this jail), the
mechanism it exhibited is real: **any confinement layer wrapping the jail whose
filter denies the socket bubblewrap opens to bring up loopback makes row 5's
probe arm impossible, and with it admission.** An agent sandbox is such a layer.
So is a seccomp-filtered CI runner. The residual is therefore not "macOS" but
*any host, or any nesting, that denies what the backend needs* — and it is
sharper than it looks, because the environments most likely to hit it are
exactly the automated ones a project relies on to notice breakage.

It is accepted rather than hidden. `backend verify` is the descriptive path,
naming what was missing and what would satisfy it (`POL-002` facet 3) instead of
failing opaquely; no `SPEC-030` requirement is met by making the suite
conditional; and the cost falls on a development or CI host rather than on a
capsule. What this design does **not** carry is a ruling on what such a CI
environment should do instead, and that is the open part: the choice is between
requiring a runner that permits the namespaces, and accepting that admission is
established on developer machines and release hosts only. Whichever slice first
runs this suite in CI owes that decision.

**4. The executed suite's cost lands on the fast inner loop.** `sec-8` rules
`default-members` and states the measurement obligation and the one lawful
adjustment. What stays open is the measurement itself, which is a phase
obligation. Round 4 added two arms and two controls, which moves the figure
without changing its shape.

**5. The channel ledger is an enumeration, and no one can prove it complete.**
This residual is created by round 4's own fix and would be dishonest to omit.
`sec-2` now lists the ways authority crosses `execute` and names the row proving
each, which is what turned an unthought-of channel into a readable blank cell.
It cannot establish that the list is exhaustive — an enumeration asserting its
own completeness is the exact shape of the invariant `F-26` refuted, one level
up.

Three channels are on the ledger but rowed by something other than a table A
row, and each is a place a later reader should look first. `argv` is closed by
construction rather than by execution — `Argv` is typed and computed
trusted-side — which is a real argument and is still not a probe. Process
reachability is split between row 7 and table B's `B5`, so no single row carries
it. And the ledger says nothing about **process credentials** — uid and gid
mapping, supplementary groups, capabilities, and whether a capsule can regain
privilege through a setuid binary in its own readable set. Under
`--unshare-all`'s user namespace bubblewrap makes that hard, which is a fact
about bubblewrap and not about the contract, and `SPEC-030` states no clause
that would make it a row. It is named here so the next reviewer starts from a
list that admits its own edge rather than from prose that sounds finished.

The honest statement of the ledger's value: it makes the *next* gap cheaper to
find and does not make it less likely to exist.

### Open questions, and where each belongs

None of these blocks this design.

- **`QUE-208` — capsule-side entity id allocation.** Parked 2026-08-06. Nothing
  in this slice mints an entity from inside a capsule. It becomes live for the
  ingestion slice and is unavoidable by the recovery slice. Its own first
  settling condition is upstream of every option in it: whether v0 permits a
  capsule to mint entities at all.
- **`ISS-319` — fresh-id allocation fails open when the trunk ref is
  unreachable.** A separable defect, fixable independently of `QUE-208` and of
  this slice.
- **`IMP-397` / `QUE-204` — egress allowlisting and non-Git build-input
  provisioning.** Out of scope, and adjacent to table A row 5: this slice
  establishes that the network posture is explicit and denied by default, not
  what a permitted posture may reach.
- **`IMP-404` — `SL-112`'s deferred engine/leaf crate extraction.** `src/lib.rs`
  is a small deliberate instance of the same shape — five leaf items, a curated
  list, an asserted boundary — and does not discharge it. Whether the export set
  is a first step toward that extraction or a special case beside it is the
  extraction's question, not this slice's.
- **Scope `OQ-1`** — the five-slice decomposition is provisional and later
  slices are deliberately unminted. **Scope `OQ-3`** — the three cross-cutting
  requirements; this slice's named share is `sec-8`'s closure table. **Scope
  `OQ-4`** — what replaces `review/*` and `phase/*` refs, which `REV-046`
  § `ADR-012` leaves a target-design question for the cutover slice.

### Corrections owed to the reconciliation brief

Collected here so the audit does not have to reassemble them from six sections.
None is a Revision: each is a correction to a record or to this slice's own
authored text, and the decisions themselves stand.

1. **`DEC-136` handoff item 1** expects a direct implementation seam in the
   existing `.doctrine/doctrine.toml` loader rather than a new configuration
   subsystem. Not available: the shared reader reads disk at `root` and is
   deliberately tolerant, `REQ-449` reads a blob at the contracted base OID and
   must be strict. A separate typed projection is required either way. The
   decision — the `[interpretation]` block stays in `.doctrine/doctrine.toml`,
   resolved once from the contracted base — stands.
2. **The slice's closure intent overstates `REQ-459`.** It is a contributing
   change against a requirement that stays `pending`, not a move to `satisfied`
   (`sec-8`).
3. **Risk `R4` is retired and `R2` changes subject** (above).
4. **The slice's Affected surface names `tests/**` for the acceptance tests.**
   They live in the new crate's `#[cfg(test)]` modules, because a bin-only
   package cannot be linked from `tests/`; only `sec-6`'s export-set and
   two-tree gate assertions land in `tests/`. The same table omits four paths
   `sec-8` adds: the workspace `Cargo.toml`, `Cargo.lock`, the `justfile`'s
   publish recipe, and the `[package] include` allow-list.

### Follow-Up at close

**The `doctrine-control` distribution contract** (`R5`). Whichever slice first
releases the binary owes the nix `srcWithDist` graft, the binstall asset name,
`install.sh` and `release.yml` together, and owes them as one change — a missing
embed graft ships a hollow binary with no compile error. An `RFC-025` § State of
play note carries it alongside the five-slice decomposition.


