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
4. **Degrade loudly where a silent degrade would recreate the bug class.**

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

impl PrimaryRoot {
    /// Impure: one `git worktree list --porcelain`. TOTAL — no repo ⇒ the
    /// given root is its own primary.
    pub(crate) fn resolve(cwd: &Path) -> Self;
    pub(crate) fn as_path(&self) -> &Path;
}
```

**The fault-vs-no-repo distinction (§5.5 I-2).** A total constructor must not
swallow a genuine git fault. "Not a repo" is the designed invariant; "git is
broken" degrading to `cwd` would home phase sheets in the wrong tree — SL-237's
own bug class reintroduced at the new seam. `resolve` therefore falls back **and
emits one named warning** when `cwd.ancestors()` contains a `.git`: a filesystem
check, no second subprocess. If there is a `.git` above you, git should have
answered.

Signature ripple:

| Function | Change |
|---|---|
| `phases_dir` (`state.rs:135`) | `(&PrimaryRoot, u32) -> PathBuf` — stays pure and total |
| `boundaries_path` (`state.rs:716`) | `(&PrimaryRoot, u32) -> PathBuf` — **loses its `Result`** (DEC-098) |
| `init_phases`, `phase_rollup`, `edit_phase_sheet`, `set_phase_status`, `completed_phase_ids`, `read_phase_statuses`, `reconcile_phase_status`, `registry_completeness`, `read_source_deltas`, `record_source_delta` | first param `&Path` → `&PrimaryRoot` |
| `resolve_phase_truth` (`state.rs:1114`) | drops `coord`; becomes `(landed, sheets)` (DEC-096) |
| `refresh_symlink` (`state.rs:345`) | gains the primary/not-primary test (DEC-097) |

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
| **tree** | true of *this checkout* | `boot.md` |
| **session** | true of *this agent run* | `handover.md`, `mem-surface-seen-<uuid>.txt` |

"PHASE-03 is completed" and "PHASE-03 spans oids X..Y" are the same kind of fact.
`PrimaryRoot` is the type that carries the repo-scoped rule. Promoting this
vocabulary to governance is explicitly **out of scope** (it is design rationale
here, not a proposed tier change).

**Sole writer is unchanged.** ADR-006 D2/D9 keep the orchestrator the only
phase-state writer; single-homing does not introduce a second writer, and
concurrent drives touch disjoint `<NNN>` directories.

### 5.4 Lifecycle, Operations & Dynamics

**Resolution points.** One `PrimaryRoot::resolve` per invocation at the command
tier: the `run_*` entry points in `slice.rs`, `dispatch.rs`,
`mcp_server/dispatch.rs`, and `lazyspec.rs:535`.

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
  evidence.
- **I-2 — Loud degradation.** Fallback under a visible `.git` warns once, named.
- **I-3 — Read-only primary (subprocess arm).** Under
  `scripts/pi-spawn-confined.sh:104-107` (`--ro-bind / /` then `--bind "$D"`) the
  primary is read-only inside a worker namespace. An **unstamped** worker
  (`worker_mode = (is_linked_worktree && marker_present) OR env DOCTRINE_WORKER`
  — the confessed ADR-011 D6/M2 false-clear) passes the CLI guard and will now
  attempt a write that hits EROFS. This is a strict improvement — a silent
  wrong-place write becomes a loud one — but it must surface as a **named error
  naming the primary and the worker-mode suspicion**, not a raw permission
  denial. Unlike boundary capture, a lost status flip is not recoverable, so
  this one must **fail**, not degrade.
- **I-4 — Symlink honesty.** The `phases` convenience symlink is minted only in
  the primary (DEC-097). A linked worktree gets none, because a dangling link is
  indistinguishable from a real empty one.
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
  CHR-050.

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

| Where | Text | What changes |
|---|---|---|
| **ADR-006 D2** | "*(the coordination/runtime tier is **withheld**, D9)*" | no longer withheld from a fork's **reach** |
| **ADR-006 D4** | "runtime state stays gitignored/**per-worktree**" | falsified — one home |
| **SPEC-012** § Tier merge-safety by construction, + the responsibility bullet | defends D4 "not by trust but by **absence**" | defence moves from absence to the CLI write guard |

**This is a restatement, not a relaxation**, and the REV must say why. Three
attacks were run against the governing content and none lands:

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
  sites change. *Mitigation:* one test helper `fn primary(p: &Path) ->
  PrimaryRoot`, so each site gains a wrapper call and nothing else; assertions
  are reviewed for unchanged-ness explicitly at audit.
- **R2 — A silent fallback homes state in the wrong tree.** *Mitigation:* I-2's
  named warning.
- **R3 — A later reader "fixes" the intentional `run_phases` split.**
  *Mitigation:* the comment required in §5.4, plus the RV-312 F-6 test.
- **R4 — Landing code that falsifies an accepted Decision.** *Mitigation:* the
  REV is opened in this slice, not deferred to reconcile.
- **R5 — DEC-098's contract change hides a real misconfiguration.** *Retired* —
  closed with evidence in the adversarial pass (§10 R-1); both propagating sites
  fail closed for other reasons. Carried only as regression test V-10.
- **R6 — a fan-out site is missed.** One already was (§10 R-2), and the type does
  not prevent it (§10 R-3). *Mitigation:* V-9 reviews **both** known sites at the
  call site; any new `phase_rollup` caller must be checked for loop context at
  review, not assumed safe because it takes a `&PrimaryRoot`.

## 9. Quality Engineering & Validation

| # | Mode | What |
|---|---|---|
| V-1a | VT | Existing suites green (behaviour-preservation gate) |
| V-1b | VA | Diff review confirms existing **assertions** are unchanged and only construction sites were adapted — a judgement, not a test, so it cannot ride V-1a's mode (R-4) |
| V-2 | VT | Non-repo tempdir: `phases_dir` resolves to the given root (I-1) |
| V-3 | VT | Fallback under a visible `.git` emits the named warning (I-2) |
| V-4 | VT | e2e: `slice conformance <ID>` agrees from the primary and a linked worktree — **ISS-269's reproduction inverting** |
| V-5 | VT | e2e: phase appended to a coord tree's `plan.toml`; `slice phases` there lands the sheet in the primary (RV-312 F-6) |
| V-6 | VT | Landed-but-`in_progress`, no coord tree → `Conflict(Rework)`; `--truth --assert` non-zero (DEC-096) |
| V-7 | VT | No `phases` symlink in a linked worktree; still minted in the primary (DEC-097) |
| V-8 | VT | Phase-status write against a read-only primary fails with the named error, not a raw permission denial (I-3) |
| V-9 | VA | **Both** fan-out sites resolve once outside their loop — `list_rows` (`slice.rs:2161`) and `load_slices` (`lazyspec.rs:495`). Reviewed at the call site, **not** inferred from the type: R-2 showed the type makes a fan-out visible, not impossible |
| V-10 | VT | `read_source_deltas` against a non-repo root yields an empty registry, and `conformance_outcome` reports `Unavailable` rather than erroring (OQ-1 / R-1) |

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

### Method note

Two findings (R-1, R-2) came from reading call sites the pre-design round had
only inventoried. A third (R-3) came from checking the doc's own strongest claim
against the evidence the pass had just produced. The pattern worth carrying: the
research artefact's ✓ rows were reliable, but its *completeness* was not — and
its own header said so ("a strong start, not a closed sweep").
