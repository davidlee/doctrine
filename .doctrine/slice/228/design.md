# SL-228 — Design: Zero-rescue dispatch funnel

> Status: **drafted, pending User approval** (internal pass + RV-303 external
> inquisition + RV-304 round-2 external inquisition integrated). Design input:
> [`extraction.md`](extraction.md) (as-built graph, commit `0603f11f`) — this
> document works the **delta** over that graph and does not restate it.
> Requirements: SPEC-021 FR-008..011 (REQ-384..387), SPEC-022 FR-010/011
> (REQ-388/389), all `pending`, minted by REV-032.
>
> **Amendment (2026-07-26, pre-PHASE-08): D9 + D10** — the reap landing authority
> and the uniform refusal-as-data contract, closing ISS-245 / ISS-246. Touches
> §1, §2 (transition table `Concluded → Reap` row), §4 (`dispatch_reap` gate row),
> **new §4.1**, §10, §11, §13 (R6, I4), §14 (NEW-OQ-C). Authored **before**
> implementation because it contradicts two locked lines — the old `Reap` facts
> gate and §4's "landed-oracle via `run_gc`" — rather than merely describing them
> staler than the code. **Reviewed as RV-308** (external codex pass, design round 4,
> 7 findings incl. one blocker): the first draft authorised `branch -D` on branch
> name + position alone and would have deleted unimported work; the tip-binding
> proof, the typed outcome table, invariant I5, and the governance mapping are all
> RV-308 repairs.

## §0 Summary

Two moves, E before A, benchmark terminal:

- **Move E** — every funnel git read becomes a first-class read verb (four gaps
  built, one relocated, the rest reused/absorbed); every coord-tree write is bounded by a
  safe-commit verb + a coord-worktree pre-commit hook. Absorbs ISS-234.
- **Move A** — funnel position becomes persisted, per-phase, authoritative
  run-state (`funnel.toml` on `dispatch/<NNN>`), advancing through a pure
  transition machine that owns legality; every funnel verb gates on it; a new
  `dispatch verify` verb makes verification an authoritative, evidence-carrying
  transition; a new `dispatch next` oracle prescribes the single next action
  over the full ladder.
- **OQ-5 benchmark** — memory-blind orchestrator completes a standard run +
  top-5 quirk scenarios by verb output alone; measured against the Cluster-1
  baseline (SL-224/225). Harness detail is plan-phase work.

## §1 Decision log

| id | Decision | Rationale anchor |
|---|---|---|
| **D1** | Run-state home = **new committed sibling** `.doctrine/dispatch/<NNN>/funnel.toml`, landed by the same splice-into-tree + `commit_on_behalf` CAS machinery as `boundaries.toml`. NOT an extension of `BoundaryRow` (pre-conclude rows would break `registry_completeness`-class consumers), NOT the journal (ref-projection grain), NOT the gitignored sheet (not crash-safe). | OQ-2; extraction §5 |
| **D2** | **Partial absorb**: position is the *sole* authority for verb legality and `next`; `derive_receipt_status` gains `Option<Position>` as senior input; `ConcludeIncomplete` is repurposed as an **integrity alarm** (position vs sheet/boundary disagreement), no longer a normal gap state; solo path (`position = None`) stays byte-identical. One authority *per question*, not two truths. | extraction §5; no-parallel-implementation |
| **D3** | `dispatch verify` is **self-executing**: conditional forward-sync of the coord worktree (clean → ff to tip; dirty → typed refusal naming paths), run the configured suite (`[dispatch] verify_suite`, default `gate`, via the existing `check` pipeline), land evidence `{status, verified_oid, suite, at}` in one CAS commit. Ladder is **monotone**: fail leaves position at `imported` with red evidence; pass advances. Conclude gate is **strict**: `position == verified ∧ verify.status == pass ∧ tree-identity(verified_oid, coord tip) modulo the funnel record` — the evidence commit is a funnel-record-only child of the tested tip, so a fresh pass never self-stales; any non-funnel commit after verify refuses `conclude-verify-stale` (RV-303 F-1). | REQ-384/385; ISS-234 fix-direction (b) delivered as side effect |
| **D4** | `dispatch next` covers the **full ladder** (spawn prescription via the existing readiness authority → await-worker → import → verify → conclude → reap), terminal at all-reaped ("consult `dispatch status`"). `next` is read-only and never heals; drift is caught by verb gates whose refusals name the fixing verb. | REQ-386; OQ-5 forcing function |
| **D5** | Safe-commit guard = **verb + hook**: a pathspec-mandatory commit verb for the orchestrator's authored writes, plus a `dispatch setup`-installed per-worktree pre-commit hook (worktreeConfig) that refuses the funnel-reversion signature in the commit set (deletion arm + modification-reversion arm, RV-303 F-7). Hook **chains to the effective non-worktree hooksPath** (local → global → system — the operator's global hook must keep firing) with fallback to common-gitdir hooks. Coord worktrees only (POL-002). | REQ-389; ISS-234 (6 repros); ADR-011 mechanism-over-prose |
| **D6** | Machine = **pure leaf** `src/funnel_machine.rs` (ADR-001); persistence behind a **sole-writer module boundary** in `src/dispatch.rs` — **command tier** per ADR-001's authoritative map (the map's only change is `funnel_machine = leaf`; "writer role", not an ADR-001 "engine" tier — RV-304 F-6); every transport reaches the same sole writer, which consults the same leaf — projection additive for the deferred subprocess arm. Spec home: **inline SPEC-021 §** at reconcile; SPEC-022 FRs cross-reference (no new spec; REQ custody stays where REV-032 minted it). | REQ-387; NEW-OQ-A |
| **D7** | Drift resistance = **golden-pinned artifact**: the leaf's `const` transition table and a committed table+mermaid artifact under `.doctrine/spec/tech/021/` are held identical by a golden VT test; the SPEC-021 § embeds/references it. Structure pinned mechanically; semantics governed socially (REV). **The authoring direction is the reverse of what "code-derived" implies (RV-312 F-7), and the cause is the `.doctrine/` hard wall**: `classify_import` refuses a worker touch of `.doctrine/` before the selector leg even runs, so the artifact cannot be generated-then-committed by the phase that owns the renderer. It is **authored first, at the orchestrator's phase base, and the renderer is pinned to it** — the same wall behind F-3's unsatisfiable `.doctrine/` selector and behind the base beat whose content F-2 shows conformance cannot see. The pin is symmetric either way: the test fails if code and artifact disagree, whichever moved. | NEW-OQ-B |
| **D8** | Class-2 transitions (act on a non-coord ref) are recorded **post-act** by the acting verb, with **heal-forward**: a later verb that can *prove* the missing Class-2 transition — from the durable fork binding plus git facts (RV-304 F-2: the binding is a crash-durable local file, not a git fact) — records it itself, **in the same CAS commit as its own transition** (one splice, all or none — RV-305 F-1). No rescue verbs, no read-side healing. | REQ-384 crash-safety; zero-rescue |
| **D9** | **Reap landing authority = the funnel record, not git archaeology — bound to the exact fork tip.** For a fork the funnel recorded, the landing proof is the conjunction: **exactly one** row matches the branch, that row is at `concluded`, **and the live branch OID equals that row's `import.fork_tip`**. Position alone is NOT the proof (RV-308 F-1: a branch that advanced past its imported tip, or a recycled fork name matching an older row, would otherwise be deleted with unimported work on it). The `git cherry` patch-id oracle is **demoted, not deleted** — it remains the sole oracle for a fork with no funnel row (solo / pre-funnel / legacy). The proof is computed by a command-tier **landing-authority resolver** and passed to the engine-tier gc classifier as an already-proven fact; gc never imports the funnel (ADR-001 — a `worktree → dispatch` edge closes a same-tier cycle; probed at `fefa4d8f`: `TangleGrew{tier: Command, baseline: 76, actual: 79}`). Two of the three landing consumers adopt the resolver here — `dispatch_reap` and `worktree::inventory`, so `worktree list` cannot publish a verdict the funnel contradicts (RV-308 F-5); CLI `worktree gc` alone defers, because it derives no slice (NEW-OQ-C). | ISS-245; §4.1; RV-308 F-1/F-2/F-5 |
| **D10** | **Every funnel tool models an actionable verdict as data.** `Refused{reason, detail}` with an enumerated reason token and the CLI's remedy text carried **verbatim** in `detail` (single-sourced, STD-001); `Err` — which the MCP transport flattens to `-32603 Internal error`, dropping the message — is reserved for genuine internal faults. Uniform across `dispatch_import` / `dispatch_conclude_phase` / `dispatch_verify` / `dispatch_reap`. Uniformity is a **claim, and a claim must be discharged where it is made** (RV-308 F-6): each fallible path is classified internal fault / pre-act refusal / **post-act partial completion** / retryable CAS loss — the third class is the dangerous one, because it can hide behind a success outcome (F-3). **Discharged here for `dispatch_reap` only** (§4.1's outcome table). For the siblings D10 states the rule and schedules the audit (VA-1); one instance is already known and named — `dispatch_conclude_phase`'s trailing sheet projection can fail via `?` *after* the durable conclusion lands, surfacing an opaque `-32603` for an outcome the caller must retry. Until that sweep lands in this section, D10 is a **rule with one verb proven**, not a proven invariant across four. | ISS-246; REQ-385 (`FR-009`) — **as split by REV-039**: naming the expected next verb is normative and proven; a refusal's text being *by itself* a sufficient recovery procedure is a **goal**, not a held property (four counter-examples; vehicle IMP-321) |

## §2 The machine (`src/funnel_machine.rs`, new — pure leaf)

No git, disk, clock, or engine imports (ADR-001 leaf; pure/imperative split).
Single-sources every state/transition token (STD-001).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Position { Spawned, WorkerCommitted, Imported, Verified, Concluded, Reaped }

/// Each transition carries its typed proposed-event payload — the CANDIDATE
/// facts. The machine, not the shell, decides result-state and act-vs-replay
/// (RV-303 F-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Transition {
  Spawn { fork: String, base_oid: String },
  RecordWorkerCommit { fork_tip: String },
  Import { fork_tip: String, onto: String },  // INPUTS only — the import commit
                                              // cannot contain its own oid (F-2
                                              // contest); it is identified
                                              // post-hoc by funnel provenance
  Verify { evidence: VerifyEvidence },   // Pass ⇒ Verified; Fail ⇒ stays Imported
  Conclude,
  Reap,
}

/// STORED facts, gathered by the engine shell. Pure data.
pub(crate) struct TransitionFacts<'a> {
  /// The row as last landed (None ⇔ no row, pre-spawn): per-transition
  /// provenance incl. last verify evidence (§3).
  pub row: Option<&'a PhaseRow>,
  /// The live coord tip at gate time.
  pub coord_tip: &'a str,
  /// Paths changed in `verified_oid‥coord_tip` (conclude's tree-identity-
  /// modulo-funnel-record gate input — RV-303 F-1). Shell-supplied.
  pub paths_since_verify: Option<&'a [String]>,
}

pub(crate) struct VerifyEvidence {
  pub status: VerifyStatus,       // Pass | Fail
  pub verified_oid: String,       // coord tip the suite ran against
  pub suite: String,              // e.g. "gate"
  pub at: String,                 // shell-supplied timestamp
}

/// THE legality authority. `current == None` ⇔ no row (pre-spawn).
pub(crate) fn attempt(
  current: Option<Position>,
  t: Transition,
  facts: &TransitionFacts,
) -> Result<Position, IllegalTransition>

/// The act/replay discriminator — the REAL authority; `attempt` is a thin
/// by-value projection over it (`attempt_advance(..).map(|a| a.position)`),
/// kept verbatim as this section's published seam. `Result<Position, _>`
/// alone cannot tell the sole writer an ACT from a REPLAY, and the writer
/// must know: landing a replay would mint a spurious CAS commit per retry,
/// which is exactly what REQ-384's crash-retry promise forbids. So the
/// verdict carries the bit.
pub(crate) struct Advance { pub position: Position, pub replay: bool }

pub(crate) fn attempt_advance(
  current: Option<Position>,
  t: &Transition,                 // by REF — the writer inspects `t` after
  facts: &TransitionFacts,
) -> Result<Advance, IllegalTransition>

/// Kind-level legality — the pure PRE-ACT gate for verbs whose evidence exists
/// only after acting (verify: status/oid are post-suite facts, so a full
/// `Transition::Verify{evidence}` cannot be constructed pre-act — RV-304 F-3).
/// Derived from the SAME const table as `attempt` (its kind column); never a
/// second table. `attempt` remains the sole transition authority at landing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitionKind { Spawn, RecordWorkerCommit, Import, Verify, Conclude, Reap }

pub(crate) fn preflight(
  current: Option<Position>,
  kind: TransitionKind,
) -> Result<(), IllegalTransition>

/// The REQ-385 (`FR-009`) refusal payload — surfaced VERBATIM by verbs and
/// rendered by `next`. It always names the expected next verb; that its text
/// is BY ITSELF a sufficient recovery procedure is the GOAL, not a property
/// this payload holds today (REV-039; vehicle IMP-321). `attempted` is the
/// KIND, not the full payload (RV-304 F-3 contest: `preflight` must be able
/// to construct the refusal pre-act, when a full `Transition::Verify`
/// payload cannot exist; the refusal names the verb — `reason` + `expected`
/// carry the recovery, candidate facts are not part of the refusal).
pub(crate) struct IllegalTransition {
  pub current: Option<Position>,
  pub attempted: TransitionKind,
  pub expected: Expected,          // the machine's prescription from `current`
  pub reason: &'static str,        // distinct token, e.g. "conclude-unverified"
}
```

### Transition table (normative; the D7 artifact renders exactly this)

| current | legal transition(s) | facts gate | result |
|---|---|---|---|
| `None` | `Spawn` | — | `Spawned` |
| `Spawned` | `RecordWorkerCommit` | — | `WorkerCommitted` |
| `WorkerCommitted` | `Import` | — | `Imported` |
| `Imported` | `Verify` | suite ran; evidence recorded either way | pass ⇒ `Verified`; fail ⇒ **stays `Imported`** (evidence updated) |
| `Verified` | `Verify` (re-verify) | — | `Verified` (fresh evidence; a **failed** re-verify keeps position — monotone — but the red evidence blocks conclude and flips `expected_next` back to verify/triage) |
| `Verified` | `Conclude` | `verify.status == Pass ∧ paths_since_verify ⊆ {funnel record}` (tree-identity modulo `funnel.toml` — F-1) | `Concluded` |
| `Concluded` | `Reap` | **shell-side, not a machine fact gate**: the landing authority per D9/§4.1 — a funnel row certifies by position, a fork with no row falls to the `git cherry` oracle. A fork already absent is an idempotent completion, not a failure | `Reaped` |
| `Reaped` | — (terminal) | — | — |

Everything not in the table refuses with `IllegalTransition{expected, reason}`.
Distinct reason tokens (STD-001 consts): `not-spawned`, `worker-not-committed`,
`not-imported`, `conclude-unverified` (no evidence), `conclude-verify-failed`
(red evidence), `conclude-verify-stale` (non-funnel-record paths changed in
`verified_oid‥tip` — the D3 modulo-funnel-record gate, **not** bare
`verified_oid != tip`, which would stale every fresh pass on its own evidence
commit — RV-305 F-2), `not-concluded`,
`already-<position>` (see idempotent replay), `terminal`.

The `already-<position>` family is **wider than replay**: it is the
*milestone-already-passed* family, and it covers **backward** attempts too. The
replay leg fires only when the transition targets exactly `current` *and* its
stable input facts match; a transition whose milestone lies **behind** `current`
matches neither, so it refuses `already-<the position it is behind>` — e.g. a
second `Import` at `verified` refuses `already-imported`, prescribing `Conclude`.
One family, two causes: a replay whose facts disagree, and a move that has
already been passed.

### Expected-next map (shared by refusals and `next`)

`None → Spawn · Spawned → (await worker) · WorkerCommitted → Import · Imported →
Verify (or triage, on red evidence) · Verified → Conclude (or Verify, if stale) ·
Concluded → Reap · Reaped → terminal`. One function, `expected_next(current,
facts)`, consumed by both surfaces — the oracle *is* the machine's read
projection; no second table.

### Idempotent replay

Attempting a transition whose target equals `current` **with matching
replay-identity facts** returns `Ok` as a **no-op replay** (reported, not
refused) — crash-retry safe on every arm. **Replay identity is each
transition's *stable input facts* only**: `fork_tip` for `RecordWorkerCommit`
**and for `Import`** — `Import`'s `onto` is stored provenance, **excluded**
from replay identity, because a lost-response retry (CAS landed, reply lost)
re-resolves the live coord tip and derives a *fresh* `onto`; keying replay on
it would refuse the very retry REQ-384 promises (RV-304 F-1). `verified_oid`
for `Verify`. `RecordWorkerCommit`'s replay leg must be **reachable on its
acting arm**: `worker_commit`'s belts resolve the fork before `attempt` runs,
so its **retry signature** (§3/§4 — clean tree, exactly one commit `C`,
`C^ == base`) resolves as `Advanced` rather than refusing at the `AtBase`
guard; only then can the `fork_tip` comparison decide replay vs act
(RV-305 F-1 — a replay promise a verb's own belts refuse is no promise). Mismatched replay-identity facts refuse (`already-<position>` +
detail). A **first** `Verify{Fail}` at `Imported` is not a replay: act-vs-replay
is decided by comparing candidate evidence to *stored* evidence (none, or a
different `verified_oid`/outcome ⇒ act; identical ⇒ replay) — evidence
comparison, not position comparison (RV-303 F-2).

## §3 The run-state record (`.doctrine/dispatch/<NNN>/funnel.toml`)

Committed on `dispatch/<NNN>` alongside `boundaries.toml`; same storage tier,
same single-writer machinery. `boundaries.toml` stays "what was delivered";
`funnel.toml` is "where each phase stands and how we know". (It will surface in
`dispatch_authored_divergence` output exactly as `boundaries.toml` already does
— same pathspec precedent, not drift; noted so review doesn't flag it.)

```toml
schema = 1

[[phase]]
id = "PHASE-01"
position = "verified"            # kebab tokens from Position::as_str (STD-001)
updated_at = "2026-07-25T09:00:00Z"

  [phase.spawn]                  # per-transition provenance, filled as reached
  fork = "dispatch/worker-a"
  base_oid = "abc123…"
  at = "…"

  [phase.worker_commit]
  fork_tip = "def456…"
  at = "…"

  [phase.import]
  fork_tip = "…"                 # the fork commit imported (input fact;
                                 # the replay-identity key — RV-304 F-1)
  onto = "…"                     # the coord tip composed onto (stored
                                 # provenance; NOT replay identity — F-1 R2)
  at = "…"                       # NB no import_oid: the row rides the import
                                 # commit's own tree, so the commit cannot name
                                 # itself (F-2); it is identified post-hoc as
                                 # the funnel-provenance commit introducing this
                                 # row

  [phase.verify]                 # LAST attempt; pass required to advance
  status = "pass"                # "pass" | "fail"
  verified_oid = "…"
  suite = "gate"
  at = "…"

  [phase.conclude]
  at = "…"                       # boundary oids live in boundaries.toml (D1)

  [phase.reap]
  at = "…"
```

The sole writer (in `src/dispatch.rs` — command tier per ADR-001's map,
RV-304 F-6): `FunnelRecord::parse/render`, `read_funnel_at(root,
tip, slice)` (object-db read, absent ⇒ empty — mirroring `read_boundaries_at`),
and the sole writer `land_funnel_transition(...)` which: reads record at tip →
`funnel_machine::attempt` → on `Ok`, splices the updated record into the commit
tree → `commit_on_behalf` CAS. **No code path lands a position row the machine
did not approve** (REQ-387's "one authority" is enforced by module boundary, not
convention).

### Atomicity classes (REQ-384 crash-safety)

- **Class 1 — atomic by construction** (position row rides the act's own commit
  tree; no crash window exists): `Import` (worker delta ⊕ funnel.toml),
  `Verify` (funnel.toml is the act's only durable artifact), `Conclude`
  (boundary row ⊕ funnel.toml — `concluded ⟺ boundary present`, making the old
  ConcludeIncomplete gap structurally impossible in funnel flows — **requires
  the §4 conclude reorder**: the CAS commit lands first, the gitignored sheet
  is projected after; the current sheet-first order is exactly the crash window
  the claim would otherwise lie about, RV-303 F-5).
- **Class 2 — post-act record** (the act lives on another ref/medium):
  `Spawn` (worktree+branch creation; recorded by `arm-spawn`),
  `RecordWorkerCommit` (fork ref advance; recorded server-side by
  `worker_commit` immediately after it lands the fork commit),
  `Reap` (worktree/branch deletion; recorded by `dispatch_reap` after `run_gc`).
  Rule (D8): record lands strictly **after** the act. Crash window ⇒ position
  lags reality; recovery is **heal-forward over the provable prefix**: a verb
  that can prove missing earlier transitions records them **in the same CAS
  commit as its own transition** — one funnel-record splice, one tree, all
  or none (RV-305 F-1 second contest: a separately-landed prefix row would
  durably rest position at `worker-committed` without worker_commit's belts
  ever having run, forging the §3 replay induction's premise; the record is
  one file, so prefix⊕own is naturally a single splice — never two landings).
  `dispatch_import` at `position == spawned` with `fork tip ≠ base` lands
  `RecordWorkerCommit` ⊕ `Import` in that one commit; at `position == None`
  it heals the full prefix (`Spawn` + `RecordWorkerCommit`) the same way,
  **only from the durable fork binding**: `DispatchRecord` gains `slice` + `phase` fields, snapshotted by
  the trusted create-fork path exactly as `base` already is — and the record
  write moves **before** the worktree-mutating act (RV-304 F-2: today
  `create.rs` writes it *after* `fork_core`, leaving a crash window in which
  a live fork exists with no binding — permanently `unprovable-fork`), with
  the fork sequence restructured around **branch-as-claim** (F-2 contests:
  a plain reorder lets a colliding spawn clobber a live fork's binding, and
  a liveness-read-then-replace is a TOCTOU under concurrent same-name
  spawns — the claim must be one atomic op, and the repo already owns one):

  1. **claim** — create the branch ref at `base` (git ref creation is
     atomic; `fork_core`'s existing `branch exists` refusal *is* the claim
     gate — exactly one concurrent same-name spawn wins, the loser refuses);
  2. **bind** — write the `DispatchRecord` under the held claim: no
     concurrent writer for the name can exist (a spawn never adopts an
     existing branch), so no clobber path exists — the write keeps a
     no-clobber belt regardless;
  3. **act** — create the worktree last, so **a live fork implies its
     binding exists by construction**.

  The claim→bind→act window is held under a per-name **claim lock** — an OS
  advisory lock (`flock`) on `<coord>/.doctrine/state/dispatch/lock/<name>`
  (runtime tier; kernel-mediated, **auto-released on process death**, so no
  stale-lease problem exists by construction; single-host is already the
  dispatch topology, ADR-008) — taken before the claim, released after the
  worktree act. This closes the **spawn–gc race** (RV-304 F-2, third
  contest): the branch claim serializes spawn against spawn, but gc could
  otherwise mistake an *active claimant's* branch (claimed, worktree not yet
  created) for crash residue, sweep it, and let a second spawn re-claim the
  name while the first continues into bind/act — cross-pairing bindings.

  Crash residues (branch-only, or branch⊕record without a worktree) are
  inert — never a live fork, `worker_commit`/import refuse on their own
  gates; a same-name retry refuses at the claim and the refusal prescribes
  the gc sweep (gc already owns record deletion — "no record survives
  without a live worktree" — and gains the branch-residue sweep). The sweep
  **must hold the same per-name claim lock** before classifying or deleting
  a residue: a busy lock IS the active-claimant signal — gc skips that name
  this sweep (non-blocking acquire); a crashed spawn's lock is already
  kernel-released, so its residue sweeps freely. A residue can never
  acquire a worktree concurrently with its sweep — the sweep holds the very
  lock bind/act require (zero-rescue: refusal + prescription, never inline
  replace). Honest
  tier claim: the binding is a crash-durable *local file* in the coord's
  runtime-state tier, **not a git fact** — destroying that tier under a live
  fork ⇒ `unprovable-fork`, the deliberate triage beat (zero-rescue never
  guesses ownership). Heal reads the binding through the **single existing
  resolver** (`resolve_agent`/`classify_resolve`), which gains a
  caller-declared **expected fork state** — `AtBase` (worker_commit
  *first-commit* pre-act: `HEAD == base`) vs `Advanced` (import/heal — and
  `worker_commit`'s **retry signature**, RV-305 F-1: exactly one commit,
  `C^ == base`): today's classifier folds ANY `HEAD != base` to
  `stale-record`, which is precisely the advanced state heal-forward needs
  (RV-304 F-7); every other consistency check (live worktree, record present,
  dir exists, branch resolves) is preserved, and no parallel raw record read
  is built. A live fork with **no** binding is not
  provable ⇒ typed refusal `unprovable-fork` naming the fork (deliberate triage
  beat — zero-rescue never guesses ownership). Both arms create forks through
  paths that write the record (`arm-spawn`, `worktree fork --worker` — each
  gains the phase argument), which is also what lets the **deferred subprocess
  arm** enter the gated funnel from import onward before its own spawn verbs
  record (Non-Goals stays honest);
  `dispatch_reap` re-run at `concluded` with the fork already absent records
  `Reaped` (idempotent completion). Re-running the original recorder is a no-op
  replay (§2). `worker_commit` resolves fork → (slice, phase) via the same
  durable binding — the very `DispatchRecord` it already consumes for the base
  guard, so the lookup is one read, always available on its (claude) arm; a
  fork with **no** binding refuses `unprovable-fork` outright (consistent with
  import — there is no "skip and let import heal" arm: import refuses the same
  unbound fork, so a skip would promise a heal that never comes; F-3/F-4
  contests). Its own crash window is healed **by its own re-drive** (RV-305
  F-1): a retry after a lost response or a kill between fork commit and coord
  record arrives at the **retry signature** — worktree *clean*, fork advanced
  by exactly one commit `C` with `C^ == base` — which resolves `Advanced`
  (never the `AtBase` refusal) and forks two truthful legs: stored
  `worker_commit.fork_tip == C` ⇒ recorded **no-op replay** naming the landed
  tip; row absent / position `spawned` ⇒ the verb lands its own lagging
  Class-2 `RecordWorkerCommit` record now — it *is* the original recorder
  (D8), and import's heal-forward remains the backstop when no retry ever
  comes. The self-record leg is **belt-gated, provenance-indifferent**
  (RV-305 F-1 contest): before recording, it re-runs the same delta/scope/
  gate belts the first-commit leg enforces, against `C`'s delta — an
  *ungated* one-commit fork (a raw-git bypass wearing the retry signature)
  either fails a belt and refuses with that belt named (triage beat — never
  adopted, never recorded) or passes them all, in which case adoption is
  sound by construction: the belts, not authorship, are the content
  authority, and a commit they cannot distinguish from a legitimate one *is*
  legitimate. The no-op replay leg needs no re-run: a row at exactly
  `worker-committed` can only have been landed by a belt-running leg (the
  first-commit leg or the belt-gated self-record; import's heal lands the
  row only **in the same CAS commit as** `Import` — D8's one-commit heal
  rule — so position never durably *rests* at `worker-committed` via heal,
  and a crash mid-heal lands nothing at all), so the stored row is the
  belt-proof by induction. A *dirty* tree on an advanced fork is not a retry: refuse (late
  re-commit); position `imported`+ refuses `already-<position>` naming the
  imported tip — the worker's lost-response ambiguity always ends in a
  truthful terminal answer, never a `stale-record` misdiagnosis.

### CAS / concurrency contract (the OQ-2 durable half)

Single-writer authority = the doctrine process (CLI or MCP server) landing
through `land_funnel_transition`; workers never write position (worker-committed
is recorded *about* them, server-side). Concurrency guard = `commit_on_behalf`'s
`update_ref_cas`: a lost-ref-race refusal means another transition landed first
— recovery is re-read tip (position re-derived from the new commit), re-attempt;
the replay rule makes retries safe. Crash recovery = nothing: position is
whatever the last landed commit says, by construction.

## §4 Write-verb gates (target invariant table — delta over extraction §2)

Every write verb inserts one step after `resolve_coord`: gather `TransitionFacts`
→ `attempt(...)` → on `Err`, return `FunnelOutcome::Refused{reason, detail}` where
`detail` renders the `IllegalTransition` (current position, expected verb,
runnable command) — the existing refusal channel, richer payload, no new shapes.

| Verb | New gate (before existing belts) | Existing belts (unchanged) |
|---|---|---|
| `worker_commit` | `attempt(RecordWorkerCommit)` **pre-act, always** — the fork binding is the same `DispatchRecord` the verb already reads for its base guard (claude arm = its only arm), so there is no unbound-but-legitimate case: refuses at `imported`+ (no late re-commit) and refuses `unprovable-fork` on a missing binding (RV-303 F-3 contest — no ungated fallback); the pre-gate is advisory against races, the post-act CAS record stays authoritative — **adds** post-land `RecordWorkerCommit` record on coord; the **retry signature** (clean tree, one commit, `C^==B` — §3, RV-305 F-1) bypasses the `AtBase`/base-guard refusal into the replay/self-record leg — the self-record leg re-runs the content belts against `C` before recording (belt-gated adoption, F-1 contest) | delta/scope/base/gate belts (base guard `HEAD==B` gates the *first-commit* leg only; the content belts gate *both* acting legs) |
| `dispatch_import` | `attempt(Import)` (heal-forward per D8) | scope belt, merge compose, CAS |
| `dispatch verify` *(new)* | `preflight(Verify)` at entry (pure kind gate — evidence exists only post-suite, RV-304 F-3); the evidence-bearing `attempt(Verify)` runs at landing | — (see §5) |
| `dispatch_conclude_phase` | `attempt(Conclude)` — refuses `conclude-unverified` / `conclude-verify-failed` / `conclude-verify-stale` | **reordered** (RV-303 F-5): boundary⊕position CAS lands **first**; sheet flip + mirror becomes a trailing projection (crash after CAS ⇒ sheet lags position — benign, position is senior per §9) |
| `dispatch_reap` | `attempt(Reap)` | landing authority per D9/§4.1: a funnel-certified fork **skips** the `git cherry` read (the fact is injected, the oracle is not overridden); no funnel row ⇒ `run_gc`'s oracle unchanged. Every gc verdict maps to `Refused{reason, detail}` per D10, never `Err` |
| `dispatch record-boundary` (CLI escape hatch) | routes through `land_funnel_transition(Conclude)` — same authority, no bypass | — |

`arm-spawn` additionally lands the `Spawn` row (Class 2) after arming.

### §4.1 The reap landing authority (D9) — why a record may certify a `branch -D`

**The defect (ISS-245).** D1 makes import atomic: the worker delta and the
`Import` row land in ONE commit. So the import commit's patch is a strict
*superset* of the fork commit's, their patch-ids differ, and
`git cherry <coord> <fork>` reports `+` for **every** funnel-managed fork. Reap
refuses on its own prescribed path and the only exit is `--force` — precisely the
"operator learns a `--force` reflex and the safety gate collapses" failure
`dispatch-mechanics.md` cites as the *reason* patch-id was chosen over
delta-emptiness. It stayed latent until SL-228 PHASE-06 because PHASE-01…05
imported with no funnel row and so stayed patch-identical.

**The authority — a proof, not an assertion (RV-308 F-1).** For a fork the funnel
recorded, the record replaces git archaeology, but *position alone is not the
proof*. The injected fact is the conjunction of three checks, all cheap:

1. **exactly one** row's `spawn.fork` matches the branch (an ambiguous match
   refuses — `row_for_fork` currently returns the *first* match, so a recycled fork
   name could otherwise bind a new branch to an older phase's row);
2. that row is at `concluded` (or `reaped`, for replay); and
3. **the live branch OID equals that row's `import.fork_tip`.**

Check 3 is what makes the fact a proof. Without it, D9 authorises `branch -D` on a
branch that may have *advanced past* what was imported — the reap would destroy
unimported commits, which is exactly the harm the patch-id oracle exists to
prevent. The funnel machine refuses a second `Import` at `imported`, so the funnel
cannot itself re-import an advanced fork; that makes an advanced-and-concluded fork
an *out-of-band* state, which is precisely the state a safety gate must survive.
Any of the three checks failing ⇒ **no fact is injected** and the `git cherry`
oracle decides, unchanged. Fail-closed, never fail-open.

**Why this is sound where a runtime "import receipt" was not.**
`dispatch-mechanics.md` rejects an apply-time receipt on two grounds: it lives in
disposable state, and it is born *before* the commit — so it survives a
crash-before-commit and reads "landed" when nothing landed, letting a cleanup reap
the only surviving copy. The funnel record inverts **both**:

- it is **durable committed state** on the coordination ref, written only by
  `land_funnel_transition` (I1) — not a flag in disposable state; and
- since D1 the `Import` row lands **in the same CAS commit as the delta**, so a
  crash mid-import lands nothing, row included.

Therefore `imported` ⇒ the delta was committed on the coordination ref; positions
are monotone (I3), so `concluded` ⇒ it was landed. The patch-id oracle is
**demoted, not deleted**: it remains the sole oracle exactly where no such record
exists.

**The residual, stated rather than hidden (RV-308 F-2).** That argument proves the
delta *was* committed; it does not by itself prove the *current* coordination tip
still descends from that commit. The record is read at the tip (`read_funnel_at(root,
tip, slice)`), so record and history are co-located — a rewound ref rewinds the
record with it, and a *revert* leaves the import commit in ancestry (recoverable,
and `git cherry` would agree). The one uncovered case is a **rewritten** ancestry
that carries the record file forward without the commit that introduced it. No
sanctioned path does this — the coordination ref is orchestrator-sole-writer
(ADR-006/ADR-012), advanced only by CAS append, and `dispatch sync` projects rather
than rewrites (SPEC-021) — so this is promoted to an explicit invariant (**I5**)
rather than an unstated assumption: if the coordination ref ever becomes
rewritable, D9 must be re-derived, not inherited. A full provenance proof (locate
the coord commit that introduced the matching `Import` row and prove it is an
ancestor of the tip) is the sound-but-expensive alternative, rejected here as
disproportionate given I5 — recorded so a future reader can reverse the call.

**Where the proof is computed — one resolver, not a scattered predicate.**
`worktree::gc` is engine tier; `dispatch` — the funnel's sole-writer home (D6) — is
command tier, so gc may not read the funnel. The three checks live together in a
**command-tier landing-authority resolver** (one named seam, `fork → landing
verdict`), which hands gc's existing gathered-facts struct an already-proven fact;
the pure classifier consults it *before* the git verdict, and the shell skips the
`git cherry` read entirely when it holds. Precedent for the direction:
`WorktreeCommand::Import` resolves `--slice` selectors at the command tier and hands
engine-tier `run_import` an already-resolved list. Class: **gc consumes facts; it
never acquires knowledge of who produced them.** The resolver being a named seam —
rather than a predicate inlined into `dispatch_reap` — is what lets the other two
landing consumers adopt it without a second implementation (see NEW-OQ-C).

**What holds the position — the typed outcome set (RV-308 F-4).** "Reaped or
`gc-incomplete`" is too coarse: gc has outcomes that fit neither, and mapping them
wrongly either lies or strands. The reap verb classifies against a **total** set,
and each case declares whether it may advance the position:

| gc outcome | advances? | why |
|---|---|---|
| `Reaped` (acted) | yes | the fork is gone |
| `AlreadyAbsent` (branch gone before the act) | yes | idempotent completion (§3) — **not** contingent on gc finding the fork present |
| `Busy` (a live claimant holds the per-name claim lock) | **no** | today a `skipped` + `Ok(())`; mapping it to success would advance while the branch and worktree still live. Refuses; the claimant is transient, so re-issue completes |
| `NotLanded` (no funnel proof, `git cherry` refuses) | no | the legacy gate, now surfaced as data (D10) |
| `CriticalResidue{worktree, branch}` | **no** | `reaped` asserts the fork is gone; advancing would make the record lie |
| `AdministrativeResidue{dispatch_record}` | yes, **with the residue reported** | the fork itself IS gone; the leftover is a runtime-tier record. Blocking on it would strand the funnel forever behind a permissions fault that re-issuing cannot clear (F-4) |

**Totality is over the residue *set*, not a choice between variants (RV-308 F-4,
round 2).** The last two rows are not alternatives. gc accumulates `leftovers` as a
**vector** across three independent legs — worktree removal (`gc.rs:404`), branch
delete (`gc.rs:416`), dispatch-record delete (`gc.rs:431`) — and raises **one**
`gc-incomplete` carrying whatever survived, so critical and administrative leftovers
routinely co-occur (a permissions fault that blocks `worktree remove` will usually
also block the record delete under it). Classifying on a disjoint variant is therefore
not a total function over what gc can report. The classification is over the set:

> residue ⊆ {`worktree`, `branch`, `dispatch_record`}; **advance iff the set contains
> neither `worktree` nor `branch`.** Critical membership outranks administrative —
> a mixed set refuses **and reports every member**, administrative ones included, so
> the operator sees the whole cleanup in one refusal rather than discovering the
> record leg on a second pass.

`CriticalResidue` / `AdministrativeResidue` name the two *verdicts* this rule yields,
not two mutually-exclusive gc results. Empty set ⇒ `Reaped`.

**What the set is total over — and the three paths that are not outcomes at all
(RV-308 F-4, round 3).** The table classifies gc's *acting* results. `run_gc_to` has
three further fallible paths, and D10's "discharged for `dispatch_reap`" claim only
holds if each is **placed** rather than passed over:

- **Pre-act faults are `Err`, by D10's own classification.** Root resolution and
  canonicalization (`gc.rs:255-257`) and the gather reads fail through `?` before
  anything is classified or deleted. Nothing acted ⇒ nothing may advance, and there
  is no operator remedy to carry in a `detail`, so these are the *internal fault*
  class — the one case where `Err` is correct. Named here so the discharge is
  explicit rather than assumed.
- **Reporting is not an outcome.** `run_gc_to` `?`-propagates its report writes: the
  `Busy` notice (`gc.rs:280`), the post-delete stderr recompile warning
  (`gc.rs:443-446`), and the final success line (`gc.rs:448`). The last two fire
  **after** both deletes succeeded, so a failed write converts a completed reap into
  an `Err` — F-3's post-act-partial-completion harm arriving through a different
  door. The funnel's typed entry point therefore **returns the verdict as data and
  lets no report write alter or fail it**: write errors are dropped, exactly as
  `dispatch_reap` already does for its own Class-2 stderr warning. `Busy` becomes a
  *returned* outcome, not a printed one — which it must anyway, since the MCP caller
  passes `io::sink()` and cannot otherwise observe it.
- **`dry_run` is outside the funnel's outcome space by construction.** It genuinely
  collapses would-reap and would-refuse into `Ok(())` (`gc.rs:348-376`), which is not
  a usable verdict — but the reap verb passes `dry_run: false`, `force: false`,
  `superseded_head: None` (`src/mcp_server/dispatch.rs:726`, `:754`). The typed entry
  point takes **neither `dry_run` nor `force`**, so the collapse is unreachable from
  the funnel by signature rather than by discipline, and the operator overrides stay
  a CLI-only affordance. `run_gc` / `run_gc_to` keep their present shape for the CLI
  (R1).

So the outcome function is total over everything the funnel's entry point can return,
and the three paths above are excluded by type, not by omission.

**And the row must actually land.** The `Reap` transition is Class 2 — recorded
strictly after the act (D8) — so the CAS can lose after the fork is already
destroyed. Returning a bare `Reaped` there is a **post-act partial completion
hiding behind a success outcome** (F-3): the shipped driver proceeds on a
non-refusal (`install/workflows/drive-slice.js`), while `next` still prescribes
reap. The verb therefore reports that state distinctly — the fork is gone (saying
otherwise would lie, per D8) *and* the row is pending, so re-driving completes it.
A success outcome may never imply a landed row.

**Scope: one resolver, three consumers, two adopt here (RV-308 F-5, round 2).**
There are exactly three landing-oracle consumers — the funnel's `dispatch_reap`, the
CLI `worktree gc`, and `worktree list` via `worktree::inventory`
(`landed_against`'s only caller outside gc). **`dispatch_reap` and
`worktree::inventory` both adopt the resolver in this phase.** Only CLI `worktree gc`
keeps the patch-id oracle.

Inventory adopts *now* because the deferral had no cost-basis. F-5's stated minimum
was that inventory must stop publishing a false unlanded, and the first disposition
answered a *structural* remedy (ship the seam) while quietly dropping that minimum —
shipping a shared seam does not un-publish the contradiction. The one objection on
record ("no `--slice` available") does not apply here at all: `landed_cell`'s
`WorkerFork` arm already has the slice in hand — `resolve_row` derives it per row via
`slice_of` from the fork's own nested path (`.dispatch/SL-<N>/.worktrees/agent-*`),
not from the `--slice` *filter* (`src/worktree/inventory.rs:179`, `:200`, `:230`).
So the **semantic** change is a substitution at one call site: the arm consults the
landing-authority resolver first and falls through to `landed_against` when no funnel
row certifies the fork — the same fail-closed shape gc's classifier uses.

*Cost correction, found at phase-plan:* the **plumbing** is not one call site.
`worktree = command` in ADR-001's map (`layering.toml:117`) and `dispatch → worktree`
already exists, so `src/worktree/mod.rs` may no more import `crate::dispatch` than
`gc.rs` may — the same 2-cycle S1 forbids. The legal injection point is
`src/commands/cli.rs:1683`, which already imports `crate::dispatch`, so the resolver
reaches `landed_cell` as a **callback threaded through four signatures**
(`worktree::dispatch` → the `List` arm → `run_list` → `resolve_row` → `landed_cell`).
A callback rather than a pre-resolved map because rows are discovered inside
`run_list` by `git::list_worktrees`; pre-resolving would duplicate that walk. This is
an injected *capability* rather than an injected *fact* — a mild widening of the
"gc consumes facts" class, recorded rather than smuggled. The decision stands; only
its cost was understated. The
existing `LandedCell::Unknown` third state already covers an unresolvable resolver
read, so no new rendering vocabulary is needed and the cell tokens stay stable
(`landed_cell_tokens_are_stable`, `inventory.rs:489`).

CLI `worktree gc` stays deferred, and that deferral *does* have a basis: it takes no
slice and derives none, so it would need a cross-slice scan of
`.doctrine/dispatch/*/funnel.toml` — a different mechanism, not a wiring change.
Mitigations for the one remaining consumer: `next` prescribes reap as the MCP
literal, so no prescribed path routes an operator to CLI gc for a funnel-managed
fork; and gc's refusal message gains a signpost naming the funnel verb (the refusal
**token** is unchanged — the goldens are token-keyed). NEW-OQ-C narrows to that one
consumer.

Scope delta this adds: `src/worktree/inventory.rs` becomes a **design-target**
selector (today it is covered only by the `src/worktree/**` scope-relevant wildcard).

## §5 `dispatch verify` (new verb: CLI + MCP `dispatch_verify`)

`dispatch verify --slice N --phase PHASE-NN`:

1. `resolve_coord`; `preflight(Verify)` legality (must be `imported`/
   `verified`) — the pure kind-level gate (RV-304 F-3: full `VerifyEvidence`
   cannot exist pre-suite); an illegal verify refuses HERE, before sync or
   suite. The evidence-bearing `attempt(Verify)` is step 5's landing gate.
2. **Conditional forward-sync** of the coord worktree to the coord tip. NOT a
   `reset` — the ref advanced *under* the checkout, so HEAD already IS the tip
   and the tree/index lag it; `reset --keep` cannot resync this shape (known
   trap, memory-pinned). Mechanism: compute the **reverse-diff set** (paths
   changed by funnel-provenance commits the tree hasn't caught up to — the
   funnel record + `FUNNEL_MARKER` history make it derivable), then
   `git restore --source=HEAD --staged --worktree -- <that set>` (the ISS-234
   known-good idiom, verb-owned now). Restore is **proof-gated per path**
   (RV-303 F-6): a reverse-set path is restored only when its index entry AND
   worktree bytes are byte-identical to the **stale baseline** — the pre-advance
   tree the checkout last materialized (the parent of the oldest
   not-yet-materialized funnel-provenance commit; blob-oid compare via
   `blob_oid_at` + working-file hash). Any delta **outside** the set, any
   **unignored untracked path** (coord trees are funnel-owned — the
   untracked-aware clean check is §8 gap 1's `tree_clean_untracked`, reused
   here), or any reverse-set path **diverging from the stale baseline** (the
   operator touched it since), ⇒ typed refusal `verify-tree-dirty` naming the
   paths (never discard uncommitted work). This collapses the ISS-234
   reverse-diff window for the rest of the phase funnel as a side effect.
3. Run the configured suite: `[dispatch] verify_suite` (default `"gate"`)
   resolved through the `check` pipeline's resolver (`resolve_check` —
   project-configured recipes; POL-002: the bar is project posture, not
   framework doctrine). The current runner is command-tier and diverges via
   `process::exit` (true exit forwarding), so verify **extracts a
   status-returning runner** — engine-callable `run_suite(root, argv) ->
   SuiteStatus` (stdio inherited/streamed, no exit); `commands/check.rs`
   becomes a thin exit-forwarding shell over it (ADR-001; RV-303 F-10).
4. **Post-suite identity check** (RV-303 F-11): re-prove tracked index +
   worktree identity against the tip **and unignored-untracked emptiness**
   (the same proofs as step 2's gate, `tree_clean_untracked` included). A
   green-but-mutating suite (formatter, codegen) **or a suite that creates
   unignored untracked files** (which the green run may have consumed —
   RV-304 F-9) ⇒ `VerifyFailed{reason: suite-mutated-tree}` naming the paths —
   pass evidence is **never** landed on bytes (or extra files) `verified_oid`
   does not describe (I2 stays true). Ignored paths (build caches) stay
   outside evidence scope by definition — the honest boundary of what
   `verified_oid` attests.
5. Land evidence in one CAS commit: pass ⇒ position `verified`; fail ⇒ position
   stays `imported`, red evidence recorded, outcome `VerifyFailed{...}` — `next`
   then reports it and halts for triage (red-verify triage stays LLM-judgment,
   per scope).

Outcome arms: `Verified{coord_tip, suite}`, `VerifyFailed{suite, detail}`,
`Refused{reason, detail}`.

**The absent-facts case, stated (RV-312 F-7).** The evidence this verb lands is
read back by conclude through `TransitionFacts::paths_since_verify`, which is
`Option` — the shell supplies it, and may not be able to. When `verified_oid`
already equals the coord tip the question is moot; otherwise `None` means **no
tree-identity was established at all**, and the gate **fails closed**:
`conclude-verify-stale`, the same token as a genuinely stale tree. The design
originally left this silent, so the fail-closed choice read as an implementation
liberty. It is not — it is the intended reading of I2: conclude may only certify
bytes some `verified_oid` attests, and an unanswered question is not an
attestation. The remedy the token prescribes (re-verify) is correct for both
causes.

## §6 `dispatch next` (new read verb: CLI + MCP `dispatch_next`)

Read-only. `resolve_coord` → read `funnel.toml` at tip → find the active phase
(position exists, not `reaped`); if none, consult the existing readiness
authority (`compute_next_phases` over `plan_next_rows` — the seam
`dispatch_next_ready` already wraps; no duplication) for the next phase to
spawn; if none remain, terminal. **Parallel file-disjoint phases:** more than
one phase may be mid-funnel; the oracle stays single-prescription, chosen by
an **actionability ladder** (RV-304 F-4 — a bare lowest-id pick lets an
awaiting phase starve a runnable one):

1. any mid-funnel phase carries **red verify evidence** ⇒
   `triage-verify-failure` for the lowest such — the verify suite is
   coord-tree-global, so red evidence is a *global* condition; the judgment
   halt deliberately outranks further mechanical progress;
2. else the lowest-id phase whose `expected_next` is a **runnable verb**
   (import / verify / conclude / reap) ⇒ that command — an awaiting phase
   never suppresses runnable work;
3. else (every mid-funnel phase awaits its worker) ⇒ `await-worker`, naming
   all awaited phases;
4. none mid-funnel ⇒ spawn prescription via the readiness authority; none
   remain ⇒ `all-reaped` terminal.

The other in-flight phases are surfaced in `detail` at every rung.

```rust
// Resolved payload — ONE prescribed action, always total over the domain.
struct NextCore {
  kind: NextKind,          // spawn | await-worker | import | verify |
                           // triage-verify-failure | reverify-stale | conclude |
                           // reap | all-reaped
  phase: Option<String>,
  command: Option<String>, // runnable literal, e.g. "doctrine dispatch verify --slice 228 --phase PHASE-02"
  detail: String,          // position + evidence context
}
```

`kind` derives from `expected_next` (§2) — same function the refusals use.
`await-worker` and `triage-verify-failure` carry no `command` (the former waits,
the latter is the deliberate judgment beat). `all-reaped` says "sub-funnel
complete — consult `dispatch status`" (Altitude B stays out of oracle scope:
REQ-386 / RV-300 F-6; OQ-4 sibling revision owns candidate/close sourcing).

## §7 Safe-commit guard (REQ-389 write side)

### The verb — `dispatch commit`

`doctrine dispatch commit --slice N -m <msg> -- <path>…` (engine + thin CLI;
MCP mirror deferred with the confined-orchestrator altitude, REQ-335):

- pathspec **structurally mandatory** (clap: at least one path);
- refuses if the to-be-committed set ⊈ declared paths, or contains any deletion
  not explicitly named (`commit-undeclared-path` / `commit-undeclared-deletion`);
- commits path-limited in the coord tree; when *declared* deletions passed
  their own validation, the verb hands the hook exactly that **validated
  deletion path-set** via a scoped env (`DOCTRINE_ALLOWED_DELETIONS`,
  path-list) for its child `git commit` only — the hook's deletion arm skips
  **only those paths**; the **reversion arm always runs** (a declared deletion
  never licenses sibling modification-reversions in the same commit —
  RV-304 F-5). The verb never sets the blanket `DOCTRINE_ALLOW_DELETE`; that
  stays the *operator's* manual both-arms escape hatch (deliberate acts are
  not ISS-234). The two guards compose instead of deadlocking. The
  orchestrator's authored-`.doctrine/` writes route here; skills drop their
  raw `git commit` prose.

### The hook — coord-worktree pre-commit backstop

Installed by `dispatch setup` (and re-checked by `doctor`): enable
`extensions.worktreeConfig` (one-time, repo-local), set **per-worktree**
`core.hooksPath` to a doctrine-owned dir under the coord worktree's private
gitdir, write the shipped hook script there (embedded in the binary —
**flake.nix graft required**, see §13 R2).

Hook behaviour (runs on ANY `git commit` in the coord worktree, any harness —
parity by construction, the ADR-011 disk-over-env argument; VA-gated anyway):

1. Inspect the to-be-committed diff (the temporary index git builds for
   pathspec commits makes this exact) and refuse the **funnel-reversion
   signature**, two arms (RV-303 F-7):
   - **deletion arm** — any deletion ⇒ refuse (pure-git check,
     `diff --cached --diff-filter=D`); mass staged deletions are the classic
     ISS-234 shape (imports that add files);
   - **reversion arm** — any modification whose staged blob equals the path's
     **pre-advance** blob while HEAD carries a funnel-advanced blob: the
     modification-only reverse-diff (imports that only modify files stage
     reversions with **no** deletions — the deletion arm alone waves it
     through). Delegated to a thin binary check (`doctrine dispatch
     hook-check`; logic in Rust, unit-testable, bounded by the `FUNNEL_MARKER`
     provenance walk). Binary absent ⇒ **fail-closed**: refuse, naming the
     missing binary and the escape hatch — a coord worktree without `doctrine`
     on PATH is already a broken operating posture, and degrading to
     deletion-only would silently reopen exactly the modification-only hole
     this arm exists to close (F-7 contest); the escape hatch keeps a
     deliberate operator unblocked.
   Refusals name the paths, the escape hatch (`DOCTRINE_ALLOW_DELETE=1` —
   covers both arms), and `dispatch commit`.
2. On pass, **chain**: resolve the effective non-worktree `core.hooksPath`
   (query `--local`, `--global`, `--system` scopes in precedence order — the
   operator's **global** hook must keep firing; resolved at *runtime*, never
   baked at install) else fall back to the common-gitdir `hooks/pre-commit`;
   exec it with identical argv/stdin; propagate its exit.

Scope: coordination worktrees only — never the primary tree, never solo forks
(POL-002: the guard rides the topology doctrine owns, not the host repo).
`--no-verify` bypass is out of accident scope (deliberate acts are not ISS-234).

## §8 Move E — the read-verb surface (REQ-388)

Principle: the orchestrator never shells raw git for a funnel read — each read
is **absorbed** into a verb's payload or **exposed** as a read verb. The nine
existing `src/git.rs` primitives stay engine-internal where only verbs consume
them; orchestrator-facing reads get surfaced:

| Need (OQ-7) | Today | Target |
|---|---|---|
| tree state after funnel writes (incl. **untracked-aware clean** — gap 1) | shell `git status` (ISS-234 trigger) | **`dispatch tree-state --slice N`** (CLI + MCP): tracked/untracked dirt, reverse-diff detection (index vs tip), staged anomalies. `git.rs` gains `tree_clean_untracked` |
| three-dot content diff (gap 2) | shell `git diff A...B` | **`dispatch delta --from A --to B [--names\|--content]`**; `git.rs` gains `three_dot_diff` |
| current branch (gap 3) + isolation classification | shell `rev-parse` / rebuild risk — **but a branch-read seam already exists**: `current_branch` at `src/dispatch.rs:1761` (RV-304 F-10) | **`dispatch whereami`**: branch via the existing `current_branch` **relocated into `git.rs`** (all existing callers migrated — reuse/relocate, NOT a new build), `is_linked_worktree` (**wrapped from `src/worktree/shared.rs:54` — relocation/wrap, NOT reimplementation**, RV-300 F-7), primary/coord/fork classification |
| history (gap 4) | shell `git log` | **`dispatch history --slice N [--ref R] [-n]`**; `git.rs` gains bounded `log_oneline` |
| ignore check (gap 5) | shell `git check-ignore` | **`dispatch ignored <path>…`**; `git.rs` gains `check_ignore` |
| ref/ancestry/content/worktree reads (the nine: `read_path_at`, `resolve_ref`, `is_ancestor`, `git_cherry`, `changed_paths`, `trunk_commit`, `blob_oid_at`, `list_worktrees`, `tree_clean`) | `pub(crate)` primitives | unchanged — consumed by verbs; exposed only via the verbs above (no bare pass-through surface) |

Post-write `git status` disappears twice over: verify's forward-sync normalizes
the tree, and `tree-state` reports it when needed. The dispatch skills +
`dispatch-mechanics.md` are rewritten in-slice to use the verbs (the VA sweep in
§11 holds them to it).

## §9 `ReceiptStatus` integration (D2)

`derive_receipt_status(sheet, has_boundary)` →
`derive_receipt_status(sheet, has_boundary, position: Option<Position>)`:

- `None` (solo, or pre-funnel dispatch phase): **byte-identical** to today —
  the behaviour-preservation gate on shared machinery applies.
- `Some(p)` — the **full matrix** (total, table-driven-tested; RV-303 F-9).
  Position is senior; every authority-order contradiction is an explicit alarm;
  the sheet remains the worker-condition axis (the ladder grows no off-ladder
  nodes). Wire tokens unchanged; doc-comments updated. **Rows are ordered:
  first match wins, top to bottom** (F-9 contest — `Err` outranks everything;
  the boundary-ahead alarm outranks the sheet-ahead alarm):

  | position | sheet | boundary | receipt |
  |---|---|---|---|
  | `p ≥ Concluded` | any — **incl. read `Err`** | present | `Completed` (a lagging sheet is benign projection lag — the §4/F-5 reorder makes sheet-behind the *normal* crash residue, self-healing; a sheet read `Err` here surfaces in `detail` as integrity degradation but **never masks durable completion** — position⊕boundary are the committed truth and the disposable tier may not overrule them, RV-304 F-8) |
  | `p ≥ Concluded` | any | absent | `ConcludeIncomplete` — **integrity alarm**: structurally impossible via the funnel ⇒ record corruption or hand-landed commit |
  | `p < Concluded` | any | **present** | `ConcludeIncomplete` — **alarm**: boundary ahead of authority (hand-landed / legacy row) |
  | `p < Concluded` | read `Err` | absent | `Unknown` (fail-loud — pre-conclude the sheet is the worker-condition axis and is genuinely required to distinguish `InProgress`/`Blocked`) |
  | `p < Concluded` | `completed` | absent | `ConcludeIncomplete` — **alarm**: sheet ahead of authority (hand flip; post-reorder the funnel never writes sheet first) |
  | `p < Concluded` | `in_progress` / `blocked` | absent | `InProgress` / `Blocked` (sheet drives, as today) |
  | `p < Concluded` | absent / `planned` | absent | `InProgress` — position proves funnel activity started; `detail` notes "sheet not started" (benign lag, not an alarm) |

  (`position = None` keeps today's `Err ⇒ Unknown` first — byte-identical, R1.)

`dispatch_phase_receipt` payload gains `position: Option<String>`.
`run_status` phase table rendering: unchanged (legacy strings verbatim).
Altitude-B readiness (`plan_next_rows` / `compute_next_phases`): untouched.

## §10 Code impact summary (→ `design-target` selectors)

| Path | Change |
|---|---|
| `src/funnel_machine.rs` | **new** — pure leaf: `Position`, `Transition`, `attempt`, `expected_next`, refusal payload, `const` table + renderer for the D7 artifact |
| `src/dispatch.rs` | sole-writer home (command tier, RV-304 F-6): `FunnelRecord` parse/render/read-at-tip, `land_funnel_transition` sole writer, verify (sync + suite + land), next, `derive_receipt_status` third input, guard verb; `current_branch` **relocates out** to `git.rs` (F-10); **D9**: the command-tier landing-authority resolver (`fork → landing verdict`: unambiguous row ∧ `concluded` ∧ live OID == `import.fork_tip`) lives here, beside the record it reads |
| `src/mcp_server/dispatch.rs` | gates on import/conclude/reap; new `dispatch_verify` + `dispatch_next` tools; `dispatch_phase_receipt` position field; **D9/D10**: `dispatch_reap` derives the funnel-landed fact from the row it already read, consumes the report-returning gc entry point through a non-stdout sink, and maps every gc verdict to `Refused{reason, detail}` |
| `src/mcp_server/worker_commit.rs` | post-land `RecordWorkerCommit` record |
| `src/git.rs` | four **new** gap reads (`tree_clean_untracked`, `three_dot_diff`, `log_oneline`, `check_ignore`) + `current_branch` **relocated** from `src/dispatch.rs:1761` with callers migrated (RV-304 F-10) |
| ~~`src/worktree/shared.rs`~~ | ~~`is_linked_worktree` re-export/wrap into the read surface (no move of the fn itself)~~ — **not delivered**; the read surface landed without needing the wrap. Selector withdrawn at reconcile (RV-312 F-4) |
| `src/worktree/dispatch_record.rs` | `DispatchRecord` gains `slice` + `phase` (fork-creation snapshot — the durable fork binding; F-4); resolver/classifier gains the caller-declared **expected fork state** (`AtBase` \| `Advanced` — RV-304 F-7). `arm-spawn` / `worktree fork --worker` gain the phase argument |
| `src/verify.rs` + `src/commands/check.rs` | runner extraction: status-returning `run_suite` callable below command tier; `check.rs` thin exit-forwarding shell (F-10) |
| `.doctrine/adr/001/layering.toml` | `funnel_machine = "leaf"` row — the pure-leaf claim gated by the architecture-layering check, not prose (F-12) |
| `src/worktree/create.rs` + `src/worktree/fork.rs` | **branch-as-claim fork sequence** (RV-304 F-2 + contests): claim (atomic branch create) → bind (record write under the claim, no-clobber belt) → act (worktree last), all under the per-name claim lock (`flock`, runtime tier, crash-released) |
| `src/worktree/gc.rs` | branch-residue sweep, **lock-gated**: non-blocking acquire of the per-name claim lock before classify/delete — busy lock = active claimant, skip (RV-304 F-2). **D9**: gathered-facts struct gains the injected funnel-landed fact (classifier consults it first; shell skips the `git cherry` read when it holds); a report-returning entry point returns the gc verdict as data so the MCP surface can map it (D10), with `run_gc`/`run_gc_to` kept as byte-compatible bail-mapping shells (R1). The typed outcome carries the residue as a **set**, not a variant — critical and administrative leftovers co-occur in one `gc-incomplete` (§4.1, F-4 round 2). The entry point takes **neither `force` nor `dry_run`** and **writes no report** — `Busy` is returned, not printed, and no write error may fail a completed reap (F-4 round 3) |
| `src/worktree/inventory.rs` | **D9 (F-5 round 2)**: `landed_cell`'s `WorkerFork` arm consults the injected landing verdict before `landed_against`, so `worktree list` and `dispatch_reap` cannot disagree. One call site — the slice is already derived per row by `resolve_row`/`slice_of`, not taken from the `--slice` filter. `LandedCell` tokens unchanged (`landed_cell_tokens_are_stable` stays green **unmodified**) |
| `src/mcp_server/tools.rs` | `dispatch_reap` tool description restated to the D9/D10 contract (the old text documents the unlanded fork as a hard error) |
| `src/commands/` | CLI wiring: `dispatch verify\|next\|commit\|tree-state\|delta\|whereami\|history\|ignored\|hook-check`; `arm-spawn` spawn-row; `record-boundary` reroute |
| `src/dispatch_config.rs` | `[dispatch] verify_suite` key |
| `src/doctor_checks.rs` | hook-present + worktreeConfig checks for live coord worktrees |
| hook script (embedded asset) | **new** — shipped pre-commit script under `install/git-hooks/`. ~~**flake.nix `srcWithDist` graft** (§13 R2)~~ — **R2 dissolved at PHASE-02**: the script rides the existing `install/` RustEmbed root, so no graft was needed and `flake.nix` was never touched. Selector withdrawn at reconcile (RV-312 F-4) |
| `plugins/doctrine/skills/{dispatch,dispatch-agent,dispatch-subprocess}/**` + `install/dispatch-mechanics.md` | funnel beats rewritten to verbs; rescue-idiom prose deleted. **Was written `.agents/skills/`** — that is the *untracked projection* `doctrine install` writes, not source. Those three selectors were unsatisfiable and are withdrawn at reconcile (RV-312 F-3); the real source above is delivered and conformant |
| `.doctrine/spec/tech/021/` | D7 golden artifact (+ § prose at reconcile, ship-time REV per Non-Goals). **Not a `design-target` selector** — `classify_import` (`src/worktree/import.rs:146-153`) returns `doctrine-touch` before the selector leg, so no design-target can ever admit a `.doctrine/` path; the golden is orchestrator-authored at the phase base. Selector withdrawn at reconcile (RV-312 F-3) |

## §11 Verification alignment (per requirement)

- **REQ-384**: VT — Class-1 atomicity (import/verify/conclude commits carry
  delta⊕position in one tree; inspect the landed commit); Class-2 idempotent
  re-record + heal-forward (kill between act and record, re-drive, position
  correct); lost-ref-race retry lands exactly one transition. VT —
  **lost-response import retry** (RV-304 F-1): CAS landed, reply lost,
  re-drive `dispatch_import` ⇒ no-op replay on matching `fork_tip` despite the
  freshly derived `onto`; distinct from the pre-CAS lost-ref-race retry (acts
  onto the new tip). VT — **worker_commit retry signature** (RV-305 F-1):
  lost-response retry (row landed, reply lost) ⇒ recorded no-op replay naming
  the landed tip; kill between fork commit and coord record, re-drive ⇒ the
  verb lands its own lagging `RecordWorkerCommit` row; dirty advanced fork
  refuses (late re-commit); retry at `imported`+ refuses `already-<position>`
  — never a `stale-record` misdiagnosis. VT — **ungated one-commit fork**
  (F-1 contest): a commit created outside `worker_commit` wearing the retry
  signature with a belt-violating delta (scope breach / red gate) refuses
  naming the violated belt — never adopted, never recorded; the same shape
  with a belt-passing delta adopts and records (belts are the content
  authority, not authorship). VT — **one-commit heal** (F-1 second contest):
  kill `dispatch_import` mid-heal ⇒ either nothing landed (retry re-heals
  the full prefix) or position is `imported` — the record never durably
  rests at `worker-committed` from any import path. VT — **branch-as-claim** (RV-304 F-2): kill between the
  claim/bind steps and the worktree act ⇒ inert residue (branch⊕record,
  never a live fork; gc sweeps — the kernel released the crashed spawn's
  claim lock; retry refuses at the claim); a sweep attempted while a live
  spawn holds the claim lock skips that name (busy lock = active claimant —
  the spawn–gc race cell);
  any live fork resolves its binding — driven through the production resolver
  with the `Advanced` expectation (RV-304 F-7), not a test-local read;
  **two concurrent same-name spawns ⇒ exactly one wins the atomic branch
  claim**, the loser refuses, and the winner's worktree pairs with the
  binding written under its own claim — the original binding stays
  byte-identical under any collision (F-2 contests); a crash-residue retry
  refuses at the claim naming the gc sweep. VT — a
  **fresh verify pass concludes** (the evidence commit never self-stales —
  the stale gate is modulo-funnel-record, never bare oid inequality, RV-305
  F-2) and an unrelated post-verify commit refuses `conclude-verify-stale`
  (F-1). VT —
  conclude kill-window: kill between boundary⊕position CAS and sheet
  projection ⇒ receipt still `Completed`, next prescribes `Reap` (F-5). VT —
  §9 full matrix, table-driven over position × sheet × boundary (F-9),
  including sheet read `Err` × `p ≥ Concluded` × boundary present ⇒
  `Completed` (RV-304 F-8). VT — D7 golden artifact matches the leaf table.
- **REQ-385**: VT — table-driven exhaustive `(position × transition)` legality
  matrix (every illegal pair refuses with the right `expected`/`reason`);
  conclude refusals: unverified / verify-failed / verify-stale each distinct;
  `worker_commit` pre-gate refuses a late re-commit at `imported`+ AND a
  missing-binding fork (`unprovable-fork` — no ungated fallback; F-3);
  heal-at-`None` with no fork binding refuses `unprovable-fork` (F-4). VT —
  **preflight** (RV-304 F-3): an illegal `verify` (e.g. at `spawned`) refuses
  at the pure kind gate — neither sync nor suite runs. VT —
  verify: a green-but-mutating suite lands `VerifyFailed{suite-mutated-tree}`,
  never pass evidence (F-11), **including a green suite that creates and
  consumes an unignored untracked file** (RV-304 F-9); a reverse-set path
  carrying an operator edit refuses `verify-tree-dirty`, restore untouched
  (F-6).
- **REQ-386**: VT — oracle total over the domain (every reachable position ⇒
  exactly one prescription; property-style over the table); terminal at
  all-reaped; `next` agrees with `expected_next` by construction (same fn).
  VT — **actionability ladder over mixed-position parallel rows** (RV-304
  F-4): an awaiting lower phase never suppresses a runnable higher phase
  (import prescribed, not await); red evidence anywhere ⇒ triage prescribed;
  property — whenever any runnable action exists, the oracle prescribes one.
  VA — OQ-5 benchmark (below).
- **REQ-387**: VT — the CLI escape hatch (`record-boundary`) and the MCP verbs
  land through `land_funnel_transition` (no second writer: enforced by
  visibility — the splice/commit fns are private to the writer module); VT —
  the architecture-layering gate covers `funnel_machine = "leaf"` (F-12); VA —
  code-shape review confirms no position write bypasses `attempt`.
  **Reconciliation posture** (F-8): REQ-387 flips `active` only if the
  subprocess arm projects through the gated funnel in-slice; if that spins out
  (scope Non-Goals), REQ-387 stays `pending` at close and the fast-follow
  slice owns the flip — partial delivery is never presented as satisfaction.
- **REQ-388**: VT — four new gap reads + the relocated `current_branch` (unit,
  over fixture repos); **one owned branch-read seam** — the relocation
  migrates every existing `src/dispatch.rs` caller, no parallel
  implementation survives (RV-304 F-10). VA — skill sweep:
  no raw-git funnel reads remain in dispatch skills/mechanics docs.
- **REQ-389**: VT — hook refuses a deletion-carrying commit (the ISS-234 repro:
  ref-advance → pathless commit → refused) **and** a modification-only
  reverse-diff (import modifies existing files only → pathless commit → the
  reversion arm refuses; F-7), plus a mixed A/M/D case; binary-absent ⇒
  fail-closed refusal naming the escape hatch (no deletion-only degrade);
  escape hatch works; **a declared deletion mixed with a stale
  reverse-modification in one `dispatch commit` still refuses** — the
  reversion arm stays live under the scoped deletion exemption
  (RV-304 F-5); **chained global hook
  still fires** (the operator-gotcha case: global `core.hooksPath` set → both
  run); guard verb refuses undeclared paths/deletions, pathless is
  unrepresentable. VA — hook parity across supported arms (claude main-thread,
  codex/pi subprocess orchestrators): same refusal in each.
- **OQ-5** (terminal acceptance): memory-blind benchmark — fresh orchestrator,
  zero dispatch memories, standard run + top-5 quirk scenarios (RFC-011
  case-notes prioritisation), driven by `next`/refusal output alone; token cost
  + completion measured against the SL-224/225 baseline. Harness = plan-phase.
- **D9/D10** (no REQ of their own — they close ISS-245/ISS-246 ahead of the OQ-5
  benchmark, which would otherwise measure the defects instead of the design):
  VT — the three landed-oracle cells (funnel row at `concluded` reaps with no
  override; no funnel row falls back to `git cherry` **unchanged**; an import
  commit whose patch is a strict *superset* of the fork's is still recognised as
  landed — the ISS-245 shape). VT — every actionable `dispatch_reap` verdict
  surfaces as `Refused{reason, detail}` with an enumerated reason and the remedy
  in `detail`, never a bare `Err`. VT — the full funnel `spawn→reap` completes on
  a **populated** funnel record with the fork branch and worktree present, zero
  `--force`, zero CLI fallback (a mechanism exercised only in its degenerate
  row-absent case has not been exercised — the PHASE-06 lesson). VA — sibling
  sweep: every dispatch MCP tool audited for an `Err` path a caller is expected to
  act on, survivors reported. Behaviour preservation (R1): `tests/e2e_worktree_gc.rs`
  stays green **unmodified**.
- **D9 safety cells (RV-308)** — each is a fail-closed case, and each is a VT:
  a fork whose live OID has **advanced past** its row's `import.fork_tip` falls back
  to `git cherry` and refuses (F-1); a **recycled fork name** matching more than one
  row refuses as ambiguous rather than binding the first (F-1); a **busy claim lock**
  refuses and does **not** advance the position (F-4); `AdministrativeResidue`
  advances **and** reports the leftover, `CriticalResidue` does neither (F-4); a
  **mixed** residue set — a surviving branch *and* a failed dispatch-record delete in
  one `gc-incomplete` — refuses **and reports both members**, critical outranking
  administrative (F-4 round 2); a **lost `Reap`-row CAS** after a successful delete is
  reported distinctly, never as plain success (F-3); **`Busy` is observable to the MCP
  caller**, which passes `io::sink()` — i.e. returned as data, never merely printed
  (F-4 round 3). Two round-3 cells are discharged **by signature, not by test**: the
  typed entry point exposes no `force`/`dry_run` (so the CLI dry-run verdict collapse
  is unreachable from the funnel) and writes no report (so no write error can fail a
  completed reap).
- **D9 oracle agreement (RV-308 F-5 round 2)** — VT: the same funnel-landed fork that
  `dispatch_reap` reaps renders **`landed`**, not `not-landed`, in `worktree list`;
  and a fork with **no** funnel row still renders the `git cherry` verdict unchanged
  (the fall-through). The `LandedCell` tokens stay stable — `landed_cell_tokens_are_stable`
  (`src/worktree/inventory.rs:489`) must pass **unmodified**, so this is additive
  routing, not a rendering change. CLI `worktree gc` is out of this cell (NEW-OQ-C).
- **Governance mapping (RV-308 F-7)** — D9 changes the *mechanism* by which an
  active requirement is satisfied, so it is not requirement-free after all:
  **PRD-015 REQ-301** ("a reap … happens only when its result provably landed in
  durable git state — never on a disposable receipt that could survive a crash and
  lie") is **satisfied, not violated** — the funnel record is durable committed
  state, which is exactly the distinction the requirement draws; likewise SPEC-021's
  "the landed oracle is durable git state, never a runtime receipt". But
  **SPEC-012 §gc names `git cherry <coordination-HEAD> <fork-branch>` as the
  mechanism**, and the D7 golden artifact (`.doctrine/spec/tech/021/funnel-machine.md`)
  still renders the old `landed-oracle ok` gate. The artifact regenerates with the
  code (D7); the SPEC-012 sentence needs a REV, which rides the slice's already-deferred
  ship-time sibling REV — recorded here so reconcile cannot lose it.

## §12 Phasing sketch (plan-stage refines; ids minted at `/plan`)

1. **Move E reads** — five `git.rs` primitives + `tree-state`/`delta`/`whereami`/
   `history`/`ignored` verbs + wrap `is_linked_worktree`.
2. **Move E guard** — `dispatch commit` verb; hook script + setup install +
   doctor checks; skill prose updates for reads+writes. *(ISS-234 closes here.)*
3. **Machine + record** — `funnel_machine.rs` leaf + D7 artifact/golden;
   `FunnelRecord` + `land_funnel_transition`; `arm-spawn`/`worker_commit`
   Class-2 records.
4. **Gates + verify** — verb gates on import/conclude/reap (+ heal-forward);
   `dispatch verify`; `ReceiptStatus` third input + receipt payload.
5. **Oracle + skills** — `dispatch next` (CLI+MCP); dispatch skills rewritten
   to the `next`-loop; rescue-idiom prose deleted.
6. **Benchmark** — OQ-5 harness + measurement + OQ-6 memory retirement list.

Move A lands *behind* move E so verify's forward-sync and the guard exist before
gates start refusing (no window where a refusal prescribes a verb that doesn't
exist yet).

## §13 Risks & invariants

- **R1 — behaviour preservation on shared machinery**: `derive_receipt_status`
  and `run_status` output must stay byte-identical where `position = None`;
  existing suites are the proof and must stay green unchanged.
- **R2 — embedded-asset strip**: the hook script is a new embedded asset;
  without a flake.nix `srcWithDist` graft the nix binary ships hollow with no
  compile error. Gate: `just nix-build` at close (host-side).
- **R3 — forward-sync vs fault-safety**: verify's sync is the one place the
  funnel touches the working tree; it is a path-bounded, **per-path
  proof-gated** `restore` over the derived reverse-diff set only (§5 —
  `reset --keep` is a known-broken shape here): only bytes proven identical to
  the stale baseline are touched; refuses on outside-set dirt AND inside-set
  divergence (F-6); never discards uncommitted work (the no-stash/no-discard
  rule).
- **R4 — hook shadowing**: per-worktree hooksPath outranks local *and global*
  config; chaining is load-bearing (operator's global hook — VT-pinned).
- **R5 — two-altitude confusion** (extraction finding): `next` (sub-funnel) vs
  `select_guidance` (slice lifecycle) must be documented as distinct oracles;
  `next`'s terminal beat hands off explicitly. Skill prose keeps one loop:
  phase-funnel = `next`; post-phases = `status`.
- **R6 — a record certifying an irreversible delete**: D9 authorises `branch -D`
  on a proof that is not git archaeology. The argument is stated in §4.1 and rests
  on two properties that must not silently regress: the record is durable committed
  state (I1), and the `Import` row is atomic with the delta (D1). If either is ever
  relaxed, D9 must be re-derived, not inherited. Blast radius is bounded by scope:
  the fact is injected only by the funnel's own reap verb, so a fork with no funnel
  row can never be reaped on a funnel proof, and the tip-binding check (§4.1) means
  a fork carrying anything beyond what was imported falls back to `git cherry`.
- **I1**: no code path writes `funnel.toml` except `land_funnel_transition`.
- **I4**: a funnel-managed reap reaches `reaped` iff the transition gate passes, the
  gc outcome is advance-eligible per §4.1's typed table, **and the `Reap` row lands**
  (Class 2 — the CAS may lose after the fork is destroyed; that state is reported
  distinctly, never as plain success — RV-308 F-3). It is never contingent on gc
  finding the fork present: an absent fork completes.
- **I5**: the coordination ref is **append-only under CAS** — advanced, never
  rewritten (ADR-006/ADR-012 orchestrator-sole-writer; `dispatch sync` projects).
  D9's soundness rests on this (§4.1); if it is ever relaxed, D9 must be re-derived
  rather than inherited.
- **I2**: no funnel verb mutates the coord index/worktree except verify's
  gated ff-sync (and `dispatch commit`, which commits only named paths);
  verify's post-suite identity check refuses pass-evidence on suite-mutated
  bytes (F-11), so I2 survives arbitrary configured suites.
- **I3**: positions only advance (monotone); evidence may update in place.

## §14 Question ledger

All slice-carried questions settled: OQ-1 (REV-032: prescribe + refuse — both),
OQ-2 → D1/§3, OQ-7 → §8, NEW-OQ-A → D6, NEW-OQ-B → D7. Deferred by design
(unchanged from scope Non-Goals): subprocess-arm full gating (machine makes it
additive; REQ-387 stays `pending` at close if it defers — §11/F-8), MCP mirror
of `dispatch commit` (rides REQ-335 reconciliation),
OQ-3/OQ-4/move-D tail, ship-time sibling REV for the four active-REQ modifies,
OQ-6 retirement list (rides the benchmark phase).

**NEW-OQ-C (open; narrowed to CLI `worktree gc`)** — when does the last landing
consumer adopt the resolver? D9 wires `dispatch_reap` and `worktree::inventory`
(§4.1); CLI `worktree gc` alone keeps the patch-id oracle, so a funnel-landed fork
still refuses there while the funnel reaps it. The deferral has a basis this time:
gc takes no slice and derives none, so adoption needs a cross-slice scan of
`.doctrine/dispatch/*/funnel.toml` — a new mechanism, not the one-call-site
substitution inventory got. Bounded by `next` prescribing the MCP reap literal, so
no prescribed path routes an operator there; the refusal gains a signpost naming the
funnel verb. Revisit at the OQ-5 benchmark, or sooner if an operator is observed
reaching for CLI gc on a funnel-managed fork.

*Round-1 framing corrected:* this question previously covered two consumers and
argued both deferrals were "wiring once the resolver exists". That was true of
inventory and is the reason it no longer defers (RV-308 F-5 round 2); it was never
true of gc.
