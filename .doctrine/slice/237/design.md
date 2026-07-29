# Design SL-237: Primary-resolve runtime phase state

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`phases_dir` (`src/state.rs:135`) is a bare path join. `boundaries_path`
(`src/state.rs:716`) — same tier, same slice, one directory apart — resolves
through `crate::git::primary_worktree`. Nothing in either name, signature, or
type says which you get. The divergence was never decided; it is two call sites
that chose independently, and it is the single generator behind a five-patch
history (ISS-212, IMP-272, IDE-028, ISS-269, SL-228 / RV-312 F-6).

This slice removes the generator, not another instance: it makes **runtime-state
scope a property of the type**, homes repo-scoped runtime state in the primary
worktree, and retires the machinery that existed only to paper over the split.

## 2. Current State

Runtime phase sheets live at `<root>/.doctrine/state/slice/<NNN>/phases/` where
`<root>` is whatever the caller handed in — usually cwd. The source-delta
registry lives at `<primary>/.doctrine/state/slice/<NNN>/boundaries.toml`. So a
linked worktree reads its own phase status against the primary's boundaries, and
`slice conformance` reports the exact inverse of the primary (ISS-269).

Compensating machinery in place today:

| Site | What it does | Fate |
|---|---|---|
| `mirror_completion_into_primary` (`state.rs:518`) | mirrors `Completed` flips from a coord tree into the primary | deleted (§5.4) |
| `resolve_phase_truth` (`state.rs:1114`) | composite truth over landed / coord / local | narrowed (DEC-096) |
| `slice status --across-trees` (`slice.rs:1121`) | renders a per-tree divergence table | renamed `--truth`, table narrowed |
| `gather_truth_inputs` (`slice.rs:1174`) | gathers landed + coord + local maps | loses the coord arm |

Two verified facts about today's code shape that constrain everything below:

- **`primary_worktree` (`src/git.rs:564`) forks a git subprocess per call,
  uncached** — `git worktree list --porcelain` plus a `canonicalize`.
- **`list_rows` (`src/slice.rs:2145`) fans `phase_rollup` over every slice
  meta** at `:2161`. This repo has 221 slice state dirs.

## 3. Forces & Constraints

- **ADR-001 (leaf ← engine ← command).** A git subprocess inside a pure path
  constructor inverts the layering. `boundaries_path` already does this; it is
  precedent for the violation, not for the design.
- **Cost.** Naive in-constructor resolution turns `doctrine slice list` from
  zero git subprocesses into ~221 — a user-visible regression on a hot listing
  path.
- **ADR-006 D2/D9 (worker-sole-writer).** No worker-side phase-status writer
  exists; all writes funnel through the orchestrator. Primary resolution
  therefore raises no worker *write* problem.
- **ADR-006 Context force 3** names cross-worktree invisibility of gitignored
  runtime state as a **hazard**. This slice serves the ADR's problem statement
  rather than contesting a decision.
- **The non-repo fallback is an invariant, not error handling.** ~20 existing
  tests exercise `phases_dir` against a plain `tempfile::tempdir()`
  (`state.rs:1382,1425,1484,1515,1540,1566,1589,1606,1631,1713,1721,1735,1749,1953,2529,2896`
  among them). No repo ⇒ the given root is its own primary.
- **POL-002.** Primary-worktree resolution is generic git topology, not a
  host-project convention. Does not bind.
- **STD-001.** Any new path segment or type name is a named constant.

**Two independent constraints select one design.** Layering says "no git
subprocess in a leaf path constructor"; cost says "don't resolve 221 times".
Both are satisfied by the same fix — resolve once in the impure shell, thread
the result down — and neither is satisfied by putting `primary_worktree` inside
`phases_dir`. That convergence is the strongest signal available here.

## 4. Guiding Principles

1. **Make the scope rule unrepresentable-if-wrong, not merely fixed once.** The
   bug is that a path silently resolved somewhere plausible and wrong. A sixth
   correct call site does not remove the generator; a type does.
2. **One resolution per invocation, minted in the impure shell.**
3. **Delete compensation, don't extend it.** Machinery whose only purpose is
   reporting or patching the split goes when the split goes.
4. **Refuse where a degrade would recreate the bug class.** Not "warn loudly" —
   a warning does not stop the write that follows it (RV-322 F-6).

## 5. Proposed Design

### 5.1 System Model

```
command tier          engine tier              leaf tier
────────────          ───────────              ─────────
run_list      ─┐
run_status    ─┤                               git::PrimaryRoot
run_phases    ─┼─ PrimaryRoot::resolve(cwd) ──▶  ::resolve  ──▶ primary_worktree
dispatch_*    ─┤        (once)                                  (1 subprocess)
mcp dispatch  ─┘            │
                            ▼
                   phases_dir(&PrimaryRoot, id)   ← pure, total
                   boundaries_path(&PrimaryRoot, id) ← pure, total
```

`PrimaryRoot` lives in `src/git.rs`, beside the resolver it wraps — the newtype
is about git topology, so that is where its cohesion lies. `src/state.rs`
consumes it. (`state.rs` retains other git dependencies — ancestry checks,
`resolve_ref` in boundary capture — so this does not make it git-free, and the
design does not claim so.)

### 5.2 Interfaces & Contracts

```rust
/// The repo's primary worktree, resolved ONCE. Repo-scoped runtime state —
/// phase sheets, the source-delta registry — is homed here so every worktree
/// names one file. Construct at the command tier; thread the value down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PrimaryRoot(PathBuf);

/// A root for READING repo-scoped runtime state. Weaker than `PrimaryRoot`: it
/// may have come from a verified resolution OR from the read fallback, and the
/// type deliberately cannot tell you which — because no reader may act on the
/// difference. There is NO conversion back to `PrimaryRoot` (RV-322 F-10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReadRoot(PathBuf);

impl PrimaryRoot {
    /// Impure: one `git worktree list --porcelain`. The ONLY way to mint the
    /// write capability.
    /// TOTAL for the designed invariant — no repo at all ⇒ the given root is
    /// its own primary. FALLIBLE for a genuine resolution fault (I-2).
    pub(crate) fn resolve(cwd: &Path) -> anyhow::Result<Self>;
    /// TEST SEAM ONLY — the caller asserts `path` is already the primary.
    /// No production consumer (F-C); `#[cfg(test)]`.
    pub(crate) fn assume(path: PathBuf) -> Self;
    pub(crate) fn as_path(&self) -> &Path;
}

impl ReadRoot {
    /// TOTAL. A resolution fault falls back to `cwd` as its own primary —
    /// byte-for-byte today's behaviour, since `phases_dir` is a pure join off
    /// the invoked root (I-2b).
    pub(crate) fn resolve_for_read(cwd: &Path) -> Self;
    pub(crate) fn as_path(&self) -> &Path;
}

/// One-way: a verified primary is always a sound read root. Never the inverse.
impl From<&PrimaryRoot> for ReadRoot { /* … */ }
```

**Why two types and not one — RV-322 F-10.** An earlier form of I-2b gave
`resolve_for_read` the *same* `PrimaryRoot` return type. That silently forged the
write capability: a value obtained by fallback could be passed to
`set_phase_status` and it would compile. I-2's refusal would then hold only by
caller discipline — which is precisely the thing this slice exists to stop relying
on, so the patch quietly abandoned the design's own thesis. Splitting the types
makes the unsound direction **unrepresentable** rather than merely discouraged:

| Direction | Status |
|---|---|
| `PrimaryRoot` → `ReadRoot` | sound; provided as `From` |
| `ReadRoot` → `PrimaryRoot` | **does not exist** — a fallback root can never reach a writer |

This is the one place in the slice where a type buys *impossibility* rather than
visibility (§10 R-3's distinction), because the hazard is a single conversion
rather than a call-site pattern like a fan-out.

**The repo/no-repo discriminator is `exists()`, never `is_dir()` — RV-322-B F-A.**
`resolve` distinguishes I-1 (no repo ⇒ total) from I-2 (fault ⇒ `Err`) by looking
for `.git` in `cwd.ancestors()`. That check MUST accept a `.git` **file** as well
as a directory: in a linked worktree `.git` is a *file*, and an `is_dir()` spelling
would classify **every linked worktree** as "no repo", hand it I-1's fallback, and
home phase state in the fork — the exact defect this slice exists to remove,
reintroduced by its own fallback. Verified: in this repo every linked worktree's
`.git` is a file; only the primary's is a directory.

Ride `root::find_from` (`src/root.rs:43`), which already walks ancestors with
`exists()` — but with `.git` **alone**, never `default_markers()`: those include
`Cargo.toml` / `.project`, which would read a non-repo Cargo project as a repo and
invert I-1.

**I-2, restated after RV-322 F-6.** An earlier draft made `resolve` total and
emitted a warning on any failure. That was the slice's own confusion at the new
seam: **observability is not prevention**. A warning does not stop the write that
follows it, so a genuine git fault would still home state in the linked tree. The
discriminator is unchanged — a `.git` in `cwd.ancestors()`, a filesystem check
with no second subprocess — but its consequence is promoted from *warn* to
*`Err`*. **A caller that cannot resolve a primary does not get to write
primary-owned state.** Totality survives only where the invariant genuinely
holds: no repo at all.

**I-2b — the refusal is scoped to writers. RV-322-B F-B.** I-2's argument is a
*write* argument, and F-6 promoted it at a seam that pure reads also cross. Today
`phases_dir` is a pure join (`state.rs:135`), so `slice list` (`slice.rs:2161`) and
the map projection (`lazyspec.rs:535`) carry **no git dependency at all**. Routing
them through a fallible `resolve` would newly hard-fail them on any git fault —
verified reproduction: a linked worktree whose admin dir was pruned still has its
`.git` file, but `git worktree list` there exits `fatal: not a git repository`.
Today `doctrine slice list` works in such a tree; under a single fallible resolve it
would not, and stale forks are ordinary residue of this repo's own dispatch
workflow.

So the two consequences are split by intent, not by luck:

| Caller intent | Constructor | Yields | Fault behaviour |
|---|---|---|---|
| will write primary-owned state | `PrimaryRoot::resolve` | `PrimaryRoot` | `Err` — no write occurs (I-2) |
| read/projection only | `ReadRoot::resolve_for_read` | `ReadRoot` | falls back to `cwd` — **exactly today's pure-join result** |

`resolve_for_read` is not a re-run of the warn-and-continue mistake F-6 killed:
that draft let a *write* proceed after a signal. This one degrades a **read** to
the answer it already gives today, and writes nothing. The distinction F-6 drew —
observability is not prevention — is preserved, because there is nothing to
prevent on a read path.

**Two roots, not one — RV-322 F-3.** `project_root` is today one identifier
serving **two meanings**: *where the state file lives* (the primary) and *which
worktree's git am I asking* (the invoked tree). `capture_phase_boundary` uses it
for both — `live_worktree_for_ref(project_root, …)` (`state.rs:617`) and
`resolve_ref(project_root, "HEAD")` (`:629`). Collapsing it to `&PrimaryRoot`
alone would record the **primary's** HEAD as a solo linked worktree's
source-delta boundary: wrong revision identity, silently. That is this slice's
own defect — one name for two jobs — reproduced in its design. So both roots are
carried explicitly, and only the functions that genuinely ask git take the second.

Signature ripple:

The split is decided **per function by whether it actually asks git** — verified
call-site by call-site, not inferred from the tier (RV-322-B F-E). A function that
touches no git gets the primary alone; widening it to both roots would be F-3's
defect mirrored (two names where one meaning suffices).

| Function | Change |
|---|---|
| `phases_dir` (`state.rs:135`) | `(&ReadRoot, u32) -> PathBuf` — pure, total. Takes the **weaker** type so readers need no write capability; writers pass `(&primary).into()` (F-10) |
| `boundaries_path` (`state.rs:716`) | `(&ReadRoot, u32) -> PathBuf` — **loses its `Result`** (DEC-098) |
| `phase_rollup`, `completed_phase_ids`, `read_phase_statuses`, `read_source_deltas` | first param `&Path` → `&ReadRoot` — pure readers. **Verified git-free** |
| `init_phases`, `forget_source_delta` (`state.rs:857`) | first param `&Path` → `&PrimaryRoot` — these **write** (`init_phases` materialises sheets; `forget_source_delta` evicts a row) |
| `edit_phase_sheet` (`state.rs:367`), `reconcile_phase_status` (`state.rs:1296`) | `&PrimaryRoot` **only** — verified zero `git::` calls in either; an earlier draft gave them both roots for nothing |
| `registry_completeness` (`state.rs:981`) | **collapses `(cwd, project_root)` → one `&PrimaryRoot`** — it already takes two roots that now mean the same tree (F-E) |
| `set_phase_status` (`state.rs:399`), `capture_phase_boundary` (`state.rs:598`) | **both roots**: `(primary: &PrimaryRoot, git_cwd: &Path, …)` (F-3) |
| `record_source_delta` (`state.rs:771`) | **both roots** — it is not a pure path consumer: `is_ancestor` (`:776`) and `parents` (`:783`) are git queries on the *invoked* tree (F-D) |
| `single_commit_boundary` (`state.rs:825`) | **unchanged, `&Path` = git cwd** — it resolves a possibly-symbolic ref (`resolve_ref(root, commit)`, `:831`). Listed so its caller (`slice.rs:3004`) is not "tidied" onto the primary |
| `resolve_phase_truth` (`state.rs:1114`) | drops `coord`; becomes `(landed, sheets)` (DEC-096) |
| `refresh_symlink` (`state.rs:345`) | both roots — mint in the primary, **retire a stale link** in a linked worktree (DEC-097, F-8) |
| `integrity.rs:322-323` (reseat guard) | hand-built `root.join(state_dir)` → resolve via `ReadRoot` — but **kind-scoped**, see below (F-5, corrected by F-13) |

**The reseat guard's migration must be kind-scoped, not generic — RV-322 F-13.**
`integrity.rs:322` iterates `kind.state_dir`, and **two** kinds declare one:
`SLICE` → `.doctrine/state/slice` (`kinds/mod.rs:358`) and `REVIEW` →
`.doctrine/state/review` (`:410`). A generic "resolve every `state_dir` through
the root type" edit would therefore silently primary-scope **review** runtime
state too — which §6 OQ-2 explicitly defers to CHR-050. The slice would ship the
deferred change while claiming not to. The guard resolves through `ReadRoot` for
`SLICE` only; every other kind keeps today's invoked-root join until CHR-050
classifies it. V-11 asserts the slice kind moves and V-17 asserts the review kind
does **not**.

**`registry_completeness` must collapse, not half-migrate (F-E).** Its two params
are documented as coinciding "in the primary worktree" (`state.rs:979`); after this
slice they *always* coincide, so keeping both preserves the very
one-name-two-jobs hazard F-3 was raised about — inverted. A call site left passing
`(&primary, &invoked_root)` would read the completed-set from a tree that no longer
holds sheets: **spurious `Incomplete`** when rows exist, and **vacuous `Complete`**
when they do not (`check_completeness` returns `Complete` for empty/empty,
`state.rs:947`). Call sites: `slice.rs:2861`, `dispatch.rs:3477`.

**`record_source_delta`'s param does three jobs, not two (F-D).** Migrating it to
`&PrimaryRoot` alone would ask the *primary* whether `code_start` is an ancestor of
`code_end`. That happens to agree today only because linked worktrees share one
object database — an unrecorded, load-bearing assumption. It is recorded here, and
the git guards take `git_cwd` so the agreement is not relied on.

CLI surface: `slice status <ID> --across-trees [--assert]` becomes
`slice status <ID> --truth [--assert]`. After this slice there are no trees to
be across; the new name matches `resolve_phase_truth` and the existing
"composite truth" vocabulary.

### 5.3 Data, State & Ownership

The unrecognised axis this slice names: the storage tiers say how *durable* a
file is, never *whose fact it is*. Within the runtime tier —

| Scope | Meaning | Examples |
|---|---|---|
| **repo** | true of the *slice*, whichever tree you stand in | `boundaries.toml`, `phases/` |
| **tree** | true of *this checkout* | `boot.md`, **`.doctrine/state/dispatch/worker`** |
| **session** | true of *this agent run* | `handover.md`, `mem-surface-seen-<uuid>.txt` |

**The worker marker is tree-scoped by necessity — RV-322-B F-H.** It is called out
in the table rather than left to inference because it sits *inside* the subtree OQ-2
defers, and the deferral is where the mistake would be made.
`worker_mode(root) = (is_linked_worktree(root) && marker_present(root)) OR env`
(`worktree/marker.rs:158-166`) asks a question about **this checkout**; resolving it
through `PrimaryRoot` would look for the marker in the primary, where it is absent
by construction, and `worker_mode` would go false in **every** fork. On the claude
arm that is the CLI's only leg (see I-3), so the effect would be to silently disable
worker write refusal. Verified live: the fork at
`.dispatch/SL-233/.worktrees/agent-…` carries the marker, and `worktree status`
there reports *"worker fork: yes — writes refused; signal: marker"*.

"PHASE-03 is completed" and "PHASE-03 spans oids X..Y" are the same kind of fact.
`PrimaryRoot` is the type that carries the repo-scoped rule. Promoting this
vocabulary to governance is explicitly **out of scope** (it is design rationale
here, not a proposed tier change).

**Sole writer is unchanged.** ADR-006 D2/D9 keep the orchestrator the only
phase-state writer; single-homing does not introduce a second writer, and
concurrent drives touch disjoint `<NNN>` directories.

### 5.4 Lifecycle, Operations & Dynamics

**Resolution points.** One resolution per invocation at the command tier: the
`run_*` entry points in `slice.rs`, `dispatch.rs`, and `mcp_server/dispatch.rs`
(including `phase_receipt`, `mcp_server/dispatch.rs:1085`, which is not a `run_*`
but resolves `phases_dir` off `coord.root` today), plus `lazyspec::load_slices`
(`lazyspec.rs:495`).

**Corrected from `lazyspec.rs:535` — RV-322-B F-F.** An earlier draft named `:535`
as the lazyspec resolution point while the fan-out table two paragraphs down said
"resolve before the loop". Those contradict: `load_slices` loops `scan_ids` at
`:498` and calls `load_plan` at `:507`, so `:535` is **inside** the loop.
Resolving there *is* fan-out #2. The resolution point is `:495`, threaded through
`load_plan`.

**There are TWO fan-outs, not one** (R-2 of the adversarial pass). Both resolve
once, outside their loop:

| Site | Loop | Fix |
|---|---|---|
| `list_rows` (`slice.rs:2145`) | `.map()` at `:2161` over every surviving `Meta` | resolve before the `.map()` |
| `load_slices` (`lazyspec.rs:495`) | `for id in scan_ids(&tree)` → `load_plan` (`:518`) → `phase_rollup` (`:535`) | resolve before the loop; thread through `load_plan` |

The pre-design round found only the first. Missing the second would have left
half the regression in place — `lazyspec` projects **every** slice.

**What the type does and does not buy.** It makes each resolution an explicit,
reviewable act at the command tier instead of a hidden cost inside a path join.
It does **not** make a fan-out impossible — a caller can still put
`PrimaryRoot::resolve` inside a loop. R-2 is the proof: a second fan-out existed
and a type would not have prevented it, only made it visible at review. The
design claims visibility, not impossibility.

**`run_phases` splits deliberately (`slice.rs:796`).** It reads `plan.toml` from
the **invoked** root and writes sheets to the **primary**. Today both are one
tree; afterwards they diverge on purpose. This is what dissolves SL-228 /
RV-312 F-6: `slice phases 228` run from a coord tree whose `plan.toml` carries
mid-drive-appended PHASE-08/09 materialises those sheets where
`registry_completeness` (`state.rs:981`) reads them. **The asymmetry must carry
a comment saying it is intentional**, or a later reader will "fix" it back.

**Deletions.** `mirror_completion_into_primary` (`state.rs:518`) with its
distinct-primary and live-coord guards and its call site at `:512`;
`resolve_phase_truth`'s `None` branch; `note_disagreement` (`state.rs:1208`);
`Divergence::disagreements`; the coord-gathering arm of `gather_truth_inputs`
and `TruthInputs::coord_rows`; the per-tree column of `render_across_trees`.

**Retained.** `reconcile_phases`' refuse-when-live bail (`slice.rs:1236-1244`)
stays. The pre-design round observed it makes the verb's coord branch
permanently dead — true, and why the narrowing costs it nothing — but the guard
itself matters *more* now: with one file, an operator reconciling during a live
drive races the drive's own writer on the **same** file.

### 5.5 Invariants, Assumptions & Edge Cases

- **I-1 — Non-repo fallback.** No repo ⇒ the given root is its own primary.
  Load-bearing, not defensive polish; ~20 existing tests are its acceptance
  evidence. This is the **only** case in which `resolve` is total.
- **I-2 — A fault refuses; it does not degrade.** See §5.2. Rewritten after
  RV-322 F-6: the earlier "warn and continue" form mistook observability for
  prevention.
- **I-3 — Worker write exclusion is OS-enforced on BOTH arms.** *Rewritten after
  RV-322 F-2; the earlier version was wrong in both directions.*
  - **Subprocess arm.** `scripts/pi-spawn-confined.sh:110` sets
    `--setenv DOCTRINE_WORKER 1` **unconditionally**, so `worker_mode` is true
    via the env leg irrespective of the marker, and the CLI guard
    (`src/commands/guard.rs:78,80`) refuses `slice phase` / `slice phases`
    **before** any write is attempted. The `--ro-bind / /` mount (`:104-107`) is
    a second, independent floor. **The EROFS path the earlier draft designed an
    error message for is unreachable on this arm.**
  - **Claude arm.** `.agents/skills/dispatch-agent/SKILL.md:177-179` installs two
    PreToolUse hooks — a nested bwrap (rw worktree, ro everything else) and
    `Edit|Write` denial outside the worktree. The earlier draft asserted this arm
    had no OS floor, on the strength of ADR-008 D-B3's text that claude's `Agent`
    tool "cannot be wrapped". **That governance is stale relative to the shipped
    skill**; the staleness is recorded as a follow-up (ISS-278), not silently
    corrected here.
  - **The claude arm's CLI refusal is the MARKER leg, not the env leg — RV-322-B
    F-G.** The hooks are the OS floor; they are not what makes `doctrine slice
    phase` refuse. That is `worker_mode`, and on this arm it is satisfied by
    `is_linked_worktree && marker_present` (`worktree/marker.rs:158-166`), *not*
    by `DOCTRINE_WORKER` — nothing in the claude spawn path sets that env var
    (verified: no `--setenv` in `worktree/pretooluse.rs`). Verified live on the
    fork at `.dispatch/SL-233/.worktrees/agent-…`: marker present, `worktree
    status` reports *"writes refused; signal: marker"*. This matters because the
    marker is the thing §5.3 now pins as tree-scoped.
  - **The refusal set is the whole `WriteClass::Write` family**, not just `slice
    phase` / `slice phases`: `record-delta`, `reconcile-phases` and `slice status`
    are all classified `Write` (`commands/guard.rs:74-101`). The floor is broader
    than the earlier draft claimed.
  - **The one documented hole is `worker_commit`, and it is why the invariant
    holds rather than a counterexample to it.** SKILL.md is explicit that the
    *unconfined server* commits on the worker's behalf — "the tool's belts, not
    the wall, are the security boundary there". An invariant asserted without its
    known exception is the F-2 failure again, so the exception is named and
    discharged on evidence: `worker_commit` lands funnel rows onto the coord ref
    through the object db (`mcp_server/worker_commit.rs:313-338`) and writes **no
    phase sheet and no registry row**. Any future change that gives it one would
    breach I-3 through the only door left open — V-8b checks this, not just the
    two arms.
  - **Consequence for this design.** No new named EROFS error is required,
    because no reachable state produces one. What the design *does* owe is that
    phase-state writes **fail rather than degrade** if they ever do hit a
    read-only primary — unlike boundary capture, a lost status flip is not
    recoverable. Stated as a property, not as a message for an unreachable path.
- **I-4 — Symlink honesty, including on upgrade.** The `phases` convenience
  symlink is minted only in the primary, **and a stale link already present in a
  linked worktree is retired** (DEC-097 + RV-322 F-8). Today's `refresh_symlink`
  mints into the invoked root, so existing worktrees already carry one; not
  removing it would leave exactly the misleading runtime projection DEC-097
  exists to prevent.
  - **Retirement is on-encounter, and that is a bound, not a sweep — RV-322
    F-14.** `refresh_symlink` runs where it is invoked, so it can only retire the
    link in *that* tree. Sibling worktrees that are never invoked in keep their
    stale links indefinitely. Sweeping them was considered and **rejected**:
    enumerating `git worktree list` and writing into other people's trees breaks
    ADR-006's sole-writer posture, races a live drive, and can touch a worktree
    whose owner never ran this slice's code. So the residual is accepted and made
    *visible* instead of silently carried: `doctrine doctor` reports a `phases`
    link in any non-primary worktree as a stale-projection finding, so the
    operator retires it by visiting (or removing) the tree. **Fresh forks are not
    affected at all** — `WITHHELD` carries `Tier::PhaseLink`
    (`worktree/allowlist.rs:74`), so provisioning never creates one. The exposure
    is finite and shrinking: only worktrees that predate this slice.
- **I-5 — Shared-file concurrency is bounded by contract, not by a lock.**
  *Added after RV-322 F-4.* `edit_phase_sheet` is an unlocked read-modify-write
  and the registry is unlocked shared-mutable. Two copies could previously
  diverge but not race; one file can race. **No locking is designed.** The bound
  is: ADR-006 D2/D9 make the orchestrator the sole phase-state writer during a
  drive, and the single-operator precondition already documented on
  `run_reconcile_phases` is promoted to a design-level assumption covering the
  shared file. Named as a **relaxation with a governed risk**, not concealed
  inside a "restatement".
- **E-1 — Worker read exposure.** A fork can now *read* primary phase sheets.
  Accepted explicitly (§7 D5), not silently.

## 6. Open Questions & Unknowns

- **OQ-1 — CLOSED in the adversarial pass (R-1), with evidence.** The question
  was whether DEC-098's contract change (non-repo cwd stops erroring, yields an
  empty registry) turns a real misconfiguration benign at the two propagating
  sites. It does not:
  - `slice.rs:3028` (`record-delta`, a **write**) calls
    `crate::git::resolve_ref(&root, …)` *before* `record_source_delta`, so a
    non-repo root fails there first. The write is unreachable in the changed
    case.
  - `slice.rs:2855` (`conformance_outcome`, a **read**) already documents
    *"Fails closed: an empty registry → `Unavailable`"*. A non-repo root yielding
    empty lands on an outcome the function is already designed around.

  No action needed in the DEC-098 phase beyond a regression test.
- **OQ-2** — `.doctrine/state/dispatch/` and `.doctrine/state/review/` are
  suspected of the same defect. Deliberately out of scope; audited under
  CHR-050. **Carries a warning, not just a deferral (RV-322-B F-H):**
  `.doctrine/state/dispatch/worker` is inside that subtree and is **tree-scoped
  by necessity** — see §5.3. Anyone applying this slice's repo-scoped reasoning
  across `dispatch/` wholesale disables worker write refusal on the claude arm.
  CHR-050 must classify each file in the subtree by *whose fact it is* before
  moving any of them; "same tier" is not "same scope".

## 7. Decisions, Rationale & Alternatives

Full rationale, alternatives, and accepted costs live in the knowledge records;
summarised here so the design reads standalone.

- **D1 (DEC-095) — `PrimaryRoot` newtype minted in the impure shell.**
  Rejected: in-constructor resolution with memoisation (keeps the ADR-001
  violation and delivers no type-visibility); per-call-site resolution (the
  sixth patch — re-scatters the decision this slice exists to centralise).
  **Accepted cost:** ~10 production sites thread the value; ~50 test
  construction sites in `state.rs` change shape. The claim "~20 tests stay green
  unchanged" therefore weakens to *assertions unchanged, construction adapted
  mechanically* — named here rather than left implicit.
- **D2 (DEC-096) — narrow `resolve_phase_truth` to landed-vs-sheet; rename the
  verb `--truth`.** The load-bearing detail is **which branch survives**: the
  `None` branch resolves *landed ⇒ Landed* unconditionally, so collapsing onto it
  would delete conflict detection and leave `--assert` permanently green. Keeping
  the `Some` matrix makes the verb strictly better on the common path.
  **Accepted:** `--assert` will newly fail where it passed. Nothing automated
  consumes the flag (verified — no justfile, skill, or CI caller outside
  `src/slice.rs`), so the blast radius is operator surprise.
- **D3 (DEC-097) — mint the `phases` symlink in the primary only.** Rejected:
  absolute link (falsifies ADR-006 D4 clause 3 for a convenience feature);
  dangling link (a dangling link reads as "this slice has no phases").
- **D4 (DEC-098) — migrate `boundaries_path` in-slice.** Deferring would ship
  SL-237 with two primary-resolution mechanisms in one module — the asymmetry it
  exists to remove, relocated. Also deletes an existing double resolution at
  `dispatch.rs:3454,3471`.
- **D5 — accept worker read exposure explicitly.** ADR-006 D2 already grants
  workers free read of the authored tier, which includes the slice design —
  strictly more sensitive than a phase sheet's status string. Read exposure adds
  no new *class* of information. The residual is prompt discipline (a worker may
  see sibling phases and act outside its own), not an invariant breach.

### Governance dependency — one REV, ADR-006 + SPEC-012

Three load-bearing statements are falsified; they are one claim at two
altitudes, so they are restated together (ADR-013 routes governance dependency
through a Revision).

*Scope widened after RV-322 F-1. The claim "no REQ requires change" — inherited
from the pre-design sweep, which checked REQ-297 and stopped — was **false**.*

| Where | Text | Status under SL-237 |
|---|---|---|
| **ADR-006 D2** | "*(the coordination/runtime tier is **withheld**, D9)*" | **falsified** — no longer withheld from a fork's **reach** |
| **ADR-006 D4** | "runtime state stays gitignored/**per-worktree**" | **falsified** — one home |
| **SPEC-012** § Tier merge-safety by construction, + the responsibility bullet | defends D4 "not by trust but by **absence**" | mechanism drifts |
| **PRD-015 Invariant 2** | "The coordination/runtime tier **never crosses the isolation boundary** into a worker" | **falsified** — a *read* crosses it |
| **PRD-015 FR-001 (REQ-296)** | "the coordination/runtime tier **absent by construction**" | **REVISED, wording only** — see below (changed from PRESERVED after RV-322 round 3) |
| **PRD-015** §2 in-scope, §3 principle, verification basis | assert the same "absence" | **falsified** — three surfaces a reader reaches before the invariant (F-12) |
| **PRD-015 NF-003 (REQ-304)** | "tier exclusion guaranteed **by construction rather than a trusted check**" | **PRESERVED**, construction relocated |

**Why the two requirements are preserved rather than revised** — the finding that
raised them assumed otherwise, and the narrower answer is the evidenced one:

- **REQ-296 — revised, wording only.** *This bullet previously read "PRESERVED";
  RV-322 round 3 re-contested it and the contest lands.* The mechanism does
  survive untouched — the provisioning allowlist `is_withheld`
  (`src/worktree/allowlist.rs:170`; `WITHHELD` covers `.doctrine/state/slice/**`)
  is not modified, so no copy is provisioned into a fork. But the requirement's
  **words** are "absent by construction", and *absent* reads as unreachable,
  which is now false: a path resolves outward and the tier is readable. Keeping
  the sentence because its implementation still works would conflate mechanism
  with promise and leave a product-level guarantee the product no longer makes.
  REV-043 row 3b revises the wording to "no copy … provisioned into it" — the
  force is unchanged, because the corruption guarded against requires a second
  mutable copy and there is none.
- **REQ-304.** Write exclusion remains by construction, at the OS layer, on
  **both** arms (I-3). The construction moved from provisioning-absence to
  mount-level read-only; it did not become a trusted check.

So the REV **adjudicates** REQ-296 / REQ-304 (recording why they hold) and
**revises** PRD-015 Invariant 2 alongside ADR-006 D2/D4 and SPEC-012.

**This is a restatement of mechanism plus one named relaxation** — the earlier
draft claimed pure restatement, which RV-322 F-4 correctly called overreach.
Merge safety is preserved (no copy ⇒ nothing to diverge); a **new concurrent-RMW
exposure** is introduced and governed by I-5 rather than concealed. Three attacks
were run against the governing content and none lands:

1. **D4's "any future central index reintroduces conflict."** The phases dir is
   gitignored (`.gitignore:39`) so git never merges it; it is per-slice and
   id-derived so concurrent drives touch disjoint directories; ADR-006 D2/D9
   make the orchestrator the sole writer. Nothing is minted or counted, so it is
   not an index in D3's sense either.
2. **Is withholding load-bearing beyond merge safety?** No. D2's load-bearing
   claim is *"workers write none of it"*; withholding was the **means**, not the
   invariant, and the invariant survives with a different mechanism (REQ-297
   stays enforced at `src/commands/guard.rs:78,80`).
3. **Does primary resolution work from a confined fork?** Yes — verified, not
   assumed. `git worktree list --porcelain` lists the main worktree first
   (confirmed live: `/workspace/doctrine` on `edge` precedes `.dispatch/SL-209`),
   and under `--ro-bind / /` the primary is present and readable.

Merge safety is **not weakened and is arguably strengthened**: the named hazard
was a *copied* sheet diverging, and single-homing makes no copy — one file cannot
diverge from itself. On the subprocess arm the write prohibition additionally
gains an OS floor. ADR-006 D4 clause 3 (the symlink is relative) needs **no**
change under D3.

## 8. Risks & Mitigations

- **R1 — Test-churn dilutes the behaviour-preservation proof.** ~50 construction
  sites change. *Mitigation (revised, RV-322 F-9):* `PrimaryRoot::assume(path)`.
  The earlier "one-line helper" mitigation was unimplementable: the tuple
  field is private to `git.rs`, so no `state.rs` test module could write it, and
  routing the tests through the impure `resolve` would have converted pure
  path-join tests into git-dependent ones — changing what the suites prove.
  Assertions are reviewed for unchanged-ness explicitly at audit (V-1b).
  **`assume` is a TEST SEAM and is spelled as one — RV-322-B F-C.** The draft
  integrating F-9 claimed it was "a real constructor, not a test-only hatch"
  because `dispatch.rs:3449-3471` needed it too. That claim does not survive
  reading the site: `prepare_review` does `git::primary_worktree(root)?` — which
  is exactly what `resolve(root)` does, at identical cost — so `resolve` fits
  there and `assume` has **no** production consumer. Shipping an
  invariant-bypassing public constructor on a production justification that does
  not exist is worse than shipping an honest test hatch, so it is `#[cfg(test)]`.
  This also re-grades §10 R-3: the visibility argument now rests on `resolve` /
  `resolve_for_read` being the only production constructors.
- **R2 — A silent fallback homes state in the wrong tree.** *Mitigation
  (revised, RV-322 F-6):* I-2 **refuses**. A warning was never a mitigation —
  observability does not prevent the write that follows it.
- **R3 — A later reader "fixes" the intentional `run_phases` split.**
  *Mitigation:* the comment required in §5.4, plus the RV-312 F-6 test.
- **R4 — Landing code that falsifies accepted governance.** *Mitigation
  (revised, RV-322 F-7):* the **gate**, not the raising. `SL-237 needs REV-043`
  is authored, and REV-043 reaching `approved` is an **entry criterion for
  PHASE-01** (§8a). Opening a REV is not a mitigation if implementation can
  start alongside it.
- **R5 — DEC-098's contract change hides a real misconfiguration.** *Retired* —
  closed with evidence in the adversarial pass (§10 R-1); both propagating sites
  fail closed for other reasons. Carried only as regression test V-10.
- **R6 — a fan-out site is missed.** One already was (§10 R-2), and the type does
  not prevent it (§10 R-3). *Mitigation:* V-9 reviews **both** known sites at the
  call site; any new `phase_rollup` caller must be checked for loop context at
  review, not assumed safe because it takes a `&PrimaryRoot`.
- **R7 — a hand-built runtime-state path is missed.** *Added, RV-322 F-5.* The
  census enumerated `phases_dir` callers, which structurally cannot find a path
  assembled by hand. One such site existed (`integrity.rs:322-323`, via
  `kind.state_dir`) and would have silently stopped guarding. *Mitigation:* the
  census axis is now `.doctrine/state` path construction, not `phases_dir`
  call sites; V-11 covers the reseat guard.
- **R8 — concurrent RMW on the now-shared file.** *Added, RV-322 F-4.* No lock
  is designed. *Mitigation:* bounded by contract (I-5), and named as a
  relaxation in REV-043 rather than absorbed into a "restatement".
- **R9 — a read-only verb gains a fatal git dependency.** *Added, RV-322-B F-B.*
  `slice list` and the map projection have none today; a single fallible
  `resolve` would hard-fail them in a repo whose git state is broken — verified
  against a pruned linked worktree, where `.git` survives as a file but `git
  worktree list` exits fatal. *Mitigation:* `resolve_for_read` (I-2b), and V-13
  pins the degradation instead of leaving it to inference.
- **R10 — the fallback discriminator reintroduces the defect.** *Added, RV-322-B
  F-A.* An `is_dir()` spelling of the `.git` ancestors check reads **every**
  linked worktree as "no repo" and homes state in the fork. *Mitigation:* the
  `exists()` requirement is stated in §5.2 as a contract, and V-14 tests it from
  a linked worktree specifically — the case an `is_dir()` bug passes every
  primary-tree test unscathed.

### 8a. Sequencing and the entry gate

**PHASE-01 does not start until REV-043 is `approved`** (R4). The phases
themselves:

1. `PrimaryRoot` + `ReadRoot` (`resolve` / `resolve_for_read`, the one-way
   `From`, and the `#[cfg(test)]` `assume`),
   `phases_dir` migration, and the two-root split for the git-asking functions
   (F-3) — decided per function against its actual `git::` calls, not by tier
   (F-D / F-E). The `exists()` discriminator (F-A) and the read/write
   constructor split (F-B) are both PHASE-01: they are contract, and every later
   phase builds on the contract being right.
2. Retire the mirror; the `run_phases` plan/sheets split.
3. `resolve_phase_truth` narrowing; `--across-trees` → `--truth`.
4. `boundaries_path` + `forget_source_delta` migration (DEC-098) — kept
   separable.
5. Symlink mint-and-**retire** (DEC-097 + F-8); `integrity.rs` reseat guard (F-5).

## 9. Quality Engineering & Validation

| # | Mode | What |
|---|---|---|
| V-1a | VT | Existing suites green (behaviour-preservation gate) |
| V-1b | VA | Diff review confirms existing **assertions** are unchanged and only construction sites were adapted — a judgement, not a test, so it cannot ride V-1a's mode (R-4) |
| V-2 | VT | Non-repo tempdir: `phases_dir` resolves to the given root (I-1) |
| V-3 | VT | Resolution failure under a visible `.git` returns `Err` and **no primary-owned write occurs** (I-2, revised per F-6 — the earlier form asserted a warning) |
| V-4 | VT | e2e: `slice conformance <ID>` agrees from the primary and a linked worktree — **ISS-269's reproduction inverting** |
| V-5 | VT | e2e: phase appended to a coord tree's `plan.toml`; `slice phases` there lands the sheet in the primary (RV-312 F-6) |
| V-6 | VT | Landed-but-`in_progress`, no coord tree → `Conflict(Rework)`; `--truth --assert` non-zero (DEC-096) |
| V-7 | VT | Symlink: minted in the primary; **a pre-existing link in a linked worktree is retired** — the upgrade case, not a fresh tree (DEC-097 + F-8) |
| V-8 | VT | **Solo linked worktree**: `set_phase_status` records the *invoked* tree's HEAD as the source-delta boundary, not the primary's (F-3 — the regression the two-root split exists to prevent) |
| V-8b | VA | Worker write exclusion is argued per arm from the actual mechanism — **not** by testing an unreachable EROFS path (I-3, revised per F-2, and again per F-15 which found this row still describing I-3 as it read *before* the F-G correction). All four legs must be argued: (i) the CLI refusal is the whole `WriteClass::Write` set, `commands/guard.rs`; (ii) `worker_mode` holds via the **env** leg on codex/pi (`pi-spawn-confined.sh:110`) and via the **marker** leg on claude (`worktree/marker.rs:158-166`) — the arms differ and the row must say so; (iii) each arm's OS floor; (iv) `worker_commit`'s unconfined-server exception writes no phase-state or registry file (`mcp_server/worker_commit.rs:313-338`) |
| V-9 | VA | **Both** fan-out sites resolve once outside their loop — `list_rows` (`slice.rs:2161`) and `load_slices` (`lazyspec.rs:495`). Reviewed at the call site, **not** inferred from the type: R-2 showed the type makes a fan-out visible, not impossible |
| V-10 | VT | `read_source_deltas` against a non-repo root yields an empty registry, and `conformance_outcome` reports `Unavailable` rather than erroring (OQ-1 / R-1) |
| V-11 | VT | The reseat guard (`integrity.rs:322-323`) refuses when live phase state exists **in the primary**, invoked from a linked worktree (F-5) |
| V-12 | VA | Census re-run on the corrected axis — `.doctrine/state` path *construction*, not `phases_dir` call sites — with every production site dispositioned (F-5 / R7). **Already run once in the RV-322-B pass: `integrity.rs:322` is the only production hand-built site** (`lazyspec.rs:1600`, `dispatch.rs:8300` are test helpers). Re-run at audit to catch sites the slice itself adds |
| V-13 | VT | A read verb (`slice list`) in a repo whose `git worktree list` fails still renders, resolving to the invoked root — and no primary-owned write occurs on that path (I-2b / R9). Reproduce the fault with a linked worktree whose admin dir is pruned |
| V-14 | VT | `resolve` **from a linked worktree** (`.git` is a *file*) returns the primary, not the linked tree — the `exists()`-vs-`is_dir()` discriminator (I-1 / R10). Must be a linked worktree: an `is_dir()` bug passes every primary-rooted test |
| V-15 | VA | `registry_completeness` takes ONE root, and both call sites (`slice.rs:2861`, `dispatch.rs:3477`) pass the primary (F-E) |
| V-16 | VA | `PrimaryRoot::assume` has no non-test caller (F-C); `edit_phase_sheet` / `reconcile_phase_status` take the primary alone, and `record_source_delta`'s git guards take the invoked tree (F-D / F-E) |
| V-17 | VT | The reseat guard still resolves the **review** kind's `state_dir` against the invoked root — the deferred-subtree boundary holds and OQ-2 was not silently implemented (F-13). Paired with V-11, which asserts the slice kind *did* move |
| V-18 | VA | No `ReadRoot → PrimaryRoot` conversion exists anywhere in the tree — the write capability cannot be forged from a fallback root (F-10). A compile-fail test is preferable to a review assertion if one is cheap |
| V-19 | VT | `doctor` reports a `phases` symlink present in a non-primary worktree as a stale-projection finding (F-14's named residual — the bound is only honest if it is visible) |

## 10. Review Notes

### Internal adversarial pass — 2026-07-29, `/design`

Eight attacks run against this design. Four land; all four are integrated above.

**R-1 — DEC-098's contract change could turn a real misconfiguration benign.
LANDS as an improvement — OQ-1 closes early rather than deferring.** Both
propagating sites were read. `slice.rs:3028` resolves a git ref *before* the
registry write, so the non-repo case never reaches it; `slice.rs:2855` already
documents empty-registry → `Unavailable` as fail-closed behaviour. Deferring this
to the DEC-098 phase (as first written) would have carried a resolvable unknown
through planning for no reason. **§6 OQ-1, V-10.**

**R-2 — the design named only ONE fan-out. LANDS; a second exists.**
`lazyspec::load_slices` (`src/lazyspec.rs:495`) loops `scan_ids` → `load_plan`
(`:518`) → `phase_rollup` (`:535`) over every slice. The pre-design round found
`list_rows` only. Fixing one and not the other would have left half the
regression in place. **§5.4.**

**R-3 — "a 221× fan-out becomes inexpressible" was an overclaim, and R-2
refutes it empirically.** A caller can still call `PrimaryRoot::resolve` inside a
loop; the type buys *visibility at review*, not impossibility. That a second
fan-out existed and no type would have caught it is the proof. The doc now claims
visibility only, and V-9 is re-graded from "argued structurally from the type" to
"reviewed at the call site". **§5.4, §9 V-9.**

**R-4 — V-1 conflated two verification modes.** "Suites green" is VT; "assertions
unchanged, only construction adapted" is a human/agent judgement (VA). Split into
V-1a / V-1b, so the behaviour-preservation gate cannot be signed off by a green
run alone — which is exactly the risk R1 in §8 flags. **§9.**

Attacks that did **not** land, recorded so they are not re-run:

- **The `--truth` rename breaks a shipped consumer.** Searched `.agents/skills`,
  `install`, `docs`, `memory` with a positive control (`slice status` *is* found
  in those trees; `across-trees` is not). Only `src/slice.rs` and SL-190's own
  artefacts reference it.
- **I-3's named error should degrade like boundary capture.** No — boundary
  capture is recoverable (the registry can be re-recorded); a lost status flip is
  not. The asymmetry is deliberate.
- **`PrimaryRoot` in `git.rs` inverts ADR-001.** No — `state.rs` already imports
  `crate::git`, so this is engine ← leaf, the sanctioned direction.
- **ISS-269's mechanism is asserted, not verified.** Verified: `conformance_outcome`
  (`slice.rs:2852`) says so in its own doc comment — *"`root` resolves both the
  shared registry (via the primary worktree) and the local phase-sheet state
  tree."* That sentence is the defect. V-4 is grounded.

### External adversarial pass — RV-322 (raiser: codex/GPT-5.5), 2026-07-29

Nine findings: six blockers, three majors. **All nine accepted**, two with
corrections to the reviewer's mechanism. Full dispositions on the ledger
(`doctrine review show 320`); the design changes they forced are integrated
above and summarised here.

| # | Sev | What it caught | Where it landed |
|---|---|---|---|
| F-1 | blocker | "No REQ requires change" was false — PRD-015 REQ-296/REQ-304 and Invariant 2 were never swept | §7 table; REV-043 gains a PRD-015 row |
| F-2 | blocker | I-3 traced an **unreachable** EROFS path and asserted a nonexistent claude-arm gap | §5.5 I-3 rewritten; V-8b |
| F-3 | blocker | `PrimaryRoot` erases the git cwd `capture_phase_boundary` needs | §5.2 two-root split; DEC-095 amendment 1; V-8 |
| F-4 | blocker | Non-duplication ≠ conflict freedom; unlocked RMW on one shared file | §5.5 I-5; R8; REV-043 reframed |
| F-5 | major | Census missed **hand-built** state paths (`integrity.rs:322-323`) | §5.2; R7; V-11, V-12 |
| F-6 | blocker | A warning is not a mitigation | §5.2 I-2 fallible; principle 4; R2; V-3 |
| F-7 | blocker | `concerns` edge where ADR-013/ADR-017 want gating `needs` | `needs` authored; §8a entry gate; R4 |
| F-8 | major | Nothing retires symlinks already minted in existing worktrees | §5.5 I-4; V-7 |
| F-9 | major | The proposed test helper was unimplementable (private tuple field) | `PrimaryRoot::assume`; DEC-095 amendment 3; R1 |

**Two corrections to the reviewer**, both narrowing rather than dismissing:

- **F-1's remedy is smaller than claimed.** REQ-296 ("absent by construction") is
  **preserved** — the provisioning allowlist `is_withheld`
  (`src/worktree/allowlist.rs:170`) is untouched, so no copy enters a fork.
  REQ-304 is **preserved** with the construction relocated to the OS floor. Only
  PRD-015 **Invariant 2** is falsified. The REV adjudicates the two requirements
  and revises the invariant.
- **F-5's mechanism is wrong; its conclusion is right and its scope is larger.**
  `integrity.rs` never calls `phases_dir` (0 matches, positive control 25 `fn`).
  It hand-builds the path from `kind.state_dir`. So the site breaks by a route
  the census could not have found — which makes it a missed *class*, not an
  instance. `src/mcp.rs` does not exist (mis-cite); `forget_source_delta` is a
  genuine omission.

**What the internal pass missed and this one caught.** F-3 and F-6 are both
instances of *the slice's own defect committed inside its own design* — one
identifier serving two meanings, and a signal mistaken for a safeguard. The
internal pass found gaps in coverage; the external pass found errors in
reasoning. That is the difference worth recording.

### Rigour pass on the integration — RV-322-B, 2026-07-29

The nine RV-322 findings were integrated in one sitting; this pass audited **the
integration itself** against source before returning the ledger to the raiser.
Every claim was verified at a call site. Eight findings land.

| # | Sev | What it caught | Where it landed |
|---|---|---|---|
| F-A | blocker | The I-1/I-2 `.git` discriminator is unspecified between `exists()` and `is_dir()`; `is_dir()` reads **every** linked worktree as "no repo" and homes state in the fork | §5.2 contract; R10; V-14 |
| F-B | blocker | I-2's `Err` is a *write* argument applied at a seam pure reads cross — `slice list` gains a fatal git dependency it has never had | §5.2 I-2b + `resolve_for_read`; R9; V-13 |
| F-C | major | `assume`'s sole claimed production consumer (`dispatch.rs:3449`) is a plain `primary_worktree(root)` — i.e. `resolve`. The "not a test hatch" justification is empty | §5.2; R1 rewritten; V-16 |
| F-D | major | `record_source_delta`'s one param does three jobs — two of them git queries (`:776`, `:783`) — so the table put it on F-3's wrong side | §5.2 table; V-16 |
| F-E | major | `registry_completeness` already takes two roots that now mean one tree; half-migrating preserves the hazard inverted. `edit_phase_sheet` / `reconcile_phase_status` were widened to two roots despite zero `git::` calls | §5.2 table; V-15, V-16 |
| F-F | major | §5.4 named `lazyspec.rs:535` a resolution point while its own table said "resolve before the loop" — `:535` is *inside* the loop, so the sentence reinstates fan-out #2 | §5.4 corrected |
| F-G | major | I-3 credits the claude arm's refusal to the PreToolUse hooks; it is actually the **marker** leg. And the invariant is asserted without its one documented hole (`worker_commit`'s unconfined server) | §5.5 I-3 |
| F-H | major | OQ-2 defers `.doctrine/state/dispatch/` without noting the worker **marker** inside it is tree-scoped by necessity — moving it would silently disable worker write refusal | §5.3 table + note; §6 OQ-2 |

**Four attacks did not land** — recorded so they are not re-run:

- *A third fan-out exists.* No. `dispatch.rs:4839`, `:5074`,
  `mcp_server/dispatch.rs:1085` and `slice.rs:998` are all single-slice. Exactly
  two.
- *The corrected census misses a second hand-built site.* No.
  `integrity.rs:322` is the only production one; the other two hits are test
  helpers. F-5's scope is right.
- *Fork provisioning re-mints the `phases` symlink, making F-8's retirement
  futile.* No — `WITHHELD` already carries `Tier::PhaseLink`
  (`worktree/allowlist.rs:74`), so a fresh fork never receives one. I-4's
  upgrade-only scope is correct.
- *`worker_commit` writes phase state from a fork, breaching I-3.* No — it lands
  funnel rows on the coord ref via the object db and touches neither sheet nor
  registry. (This is now recorded as I-3's evidence rather than left untested.)

**The pattern this pass confirms.** Three of the eight (F-A, F-D, F-E) are the
same defect the slice exists to remove — one name serving two meanings, or a
discriminator that cannot tell two things apart — committed *inside the fix for
the findings about committing it inside the design*. The external pass named this
about F-3 and F-6; it recurred one layer down, in the integration. Concentrating
that class into PHASE-01 as contract (§8a) is the structural answer.

### External pass, round 2 — RV-322 raiser verification, 2026-07-29

The raiser verified F-3, F-4, F-6, F-7, F-9; contested F-1, F-2, F-5, F-8; and
raised F-10..F-17 against the RV-322-B integration. Seven land. One is a race.

| # | Verdict | Where it landed |
|---|---|---|
| F-10 | **accepted — blocker** | `resolve_for_read` returned `PrimaryRoot`, forging the write capability. §5.2 splits `PrimaryRoot` / `ReadRoot` one-way; V-18 |
| F-11 (=F-2) | **already fixed** | The stale claude-arm text was corrected in REV-043 rows 1a and 2 at commit `7e92f41e`, before this round returned. The raiser read the pre-fix artefact |
| F-12 (=F-1) | **accepted** | PRD-015 asserts absence/non-leakage on more surfaces than Invariant 2. REV-043 row 3 widens rather than a second REV — one claim, one altitude, one approval gate |
| F-13 (=F-5) | **accepted — the responder's census was under-read** | `integrity.rs:322` is generic over `kind.state_dir`, and REVIEW declares one (`kinds/mod.rs:410`). A generic migration would have shipped OQ-2's deferred change. §5.2 kind-scopes it; V-17 |
| F-14 (=F-8) | **accepted with a bounded remedy** | Retirement is on-encounter and cannot reach sibling worktrees. Sweeping rejected (sole-writer, races); residual made visible via `doctor`. §5.5 I-4; V-19 |
| F-15 | **accepted** | V-8b still described I-3 as it read before the F-G correction. Rewritten to argue all four legs |
| F-16 | **accepted** | DEC-098's "the fallible half moves to `resolve`" predates the constructor split. Amended |
| F-17 | **accepted** | `slice-237.md` still claimed "restatement, not a relaxation" — the claim F-4 falsified. Corrected |

**The recurring failure this round names.** F-11, F-16 and F-17 are one defect
three times: a correction was written where it was *argued* and not everywhere it
was *asserted*. F-4's correction reached REV-043's rationale but not the scope
doc; F-2's reached the rationale but not the rows; F-B's reached §5.2 but not
DEC-098. The lesson is procedural, not analytical — **when a finding overturns a
claim, grep the corpus for the claim, not for the finding.** Recorded in the
harvest.

### External pass, round 3 — RV-322 raiser verification, 2026-07-29

Nine dispositions verified (F-2, F-5, F-8, F-11, F-13, F-14, F-15, F-16, F-17).
**No new findings** — the raiser stated explicitly that new ids would duplicate
the contested remedies, which is the right call and worth recording as evidence
the design is converging rather than churning. Three contested, resolving to two
defects, both accepted:

| # | Contest | Resolution |
|---|---|---|
| F-10 | The two-type design closes the hole, but **DEC-095 still specified the obsolete one-type API** — `PrimaryRoot::resolve_for_read`, `phases_dir(&PrimaryRoot)`, no `ReadRoot` | DEC-095 amendment 6; amendment 4's API superseded in place |
| F-1 / F-12 | REQ-296 still **promises absence** while the design concedes readability; and row 3d wrote slice history and OS detail into product-spec prose | REV-043 row 3b becomes a wording revision; row 3d's proposed text re-pitched to product altitude |

**F-10's contest is the §10 lesson landing on its author.** The round-2 pass
recorded, as a procedural finding, that *a correction gets written where it was
argued and not everywhere it was asserted* — and then committed exactly that:
the two-type split reached §5.2 and DEC-098 but not DEC-095, in the same commit
that wrote the lesson down. That is worth keeping rather than quietly fixing,
because it shows the failure is not inattention — it survives knowing about it.
The countermeasure has to be mechanical (grep the corpus for the claim), not
intentional.

**On REQ-296.** The round-1 adjudication — "preserved, because the allowlist is
untouched" — was a mechanism argument answering a wording question. A
requirement is a promise, and *absent* promises unreachability. The revision
changes only the words, so nothing about the isolation guarantee moves; but the
distinction between "the implementation still holds" and "the sentence is still
true" is the one that was missed, twice, before it was named.

### Method note

Two findings (R-1, R-2) came from reading call sites the pre-design round had
only inventoried. A third (R-3) came from checking the doc's own strongest claim
against the evidence the pass had just produced. The pattern worth carrying: the
research artefact's ✓ rows were reliable, but its *completeness* was not — and
its own header said so ("a strong start, not a closed sweep").
