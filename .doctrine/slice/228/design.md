# SL-228 — Design: Zero-rescue dispatch funnel

> Status: **drafted, pending User approval** (internal pass + RV-303 external
> inquisition + RV-304 round-2 external inquisition integrated). Design input:
> [`extraction.md`](extraction.md) (as-built graph, commit `0603f11f`) — this
> document works the **delta** over that graph and does not restate it.
> Requirements: SPEC-021 FR-008..011 (REQ-384..387), SPEC-022 FR-010/011
> (REQ-388/389), all `pending`, minted by REV-032.

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
| **D7** | Drift resistance = **code-derived golden artifact**: the leaf's `const` transition table renders to a committed table+mermaid artifact under `.doctrine/spec/tech/021/`, pinned by a golden VT test; the SPEC-021 § embeds/references it. Structure pinned mechanically; semantics governed socially (REV). | NEW-OQ-B |
| **D8** | Class-2 transitions (act on a non-coord ref) are recorded **post-act** by the acting verb, with **heal-forward**: a later verb that can *prove* the missing Class-2 transition — from the durable fork binding plus git facts (RV-304 F-2: the binding is a crash-durable local file, not a git fact) — records it itself before its own transition. No rescue verbs, no read-side healing. | REQ-384 crash-safety; zero-rescue |

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

/// The FR-009 refusal payload — surfaced VERBATIM by verbs and rendered by
/// `next`: the refusal text IS the recovery procedure. `attempted` is the
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
| `Concluded` | `Reap` | landed-oracle ok (engine-side), or fork already absent (idempotent completion) | `Reaped` |
| `Reaped` | — (terminal) | — | — |

Everything not in the table refuses with `IllegalTransition{expected, reason}`.
Distinct reason tokens (STD-001 consts): `not-spawned`, `worker-not-committed`,
`not-imported`, `conclude-unverified` (no evidence), `conclude-verify-failed`
(red evidence), `conclude-verify-stale` (`verified_oid != tip`), `not-concluded`,
`already-<position>` (see idempotent replay), `terminal`.

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
for `Verify`. Mismatched replay-identity facts refuse (`already-<position>` +
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
  that can prove missing earlier transitions records them before its own.
  `dispatch_import` at `position == spawned` with `fork tip ≠ base` lands
  `RecordWorkerCommit` then `Import`; at `position == None` it heals the
  full prefix (`Spawn` + `RecordWorkerCommit`) **only from the durable fork
  binding**: `DispatchRecord` gains `slice` + `phase` fields, snapshotted by
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

  Crash residues (branch-only, or branch⊕record without a worktree) are
  inert — never a live fork, `worker_commit`/import refuse on their own
  gates; a same-name retry refuses at the claim and the refusal prescribes
  the gc sweep (gc already owns record deletion — "no record survives
  without a live worktree" — and gains the branch-residue sweep). A residue
  can never acquire a worktree concurrently with its sweep, because
  acquiring one would require winning a claim the residue still holds
  (zero-rescue: refusal + prescription, never inline replace). Honest
  tier claim: the binding is a crash-durable *local file* in the coord's
  runtime-state tier, **not a git fact** — destroying that tier under a live
  fork ⇒ `unprovable-fork`, the deliberate triage beat (zero-rescue never
  guesses ownership). Heal reads the binding through the **single existing
  resolver** (`resolve_agent`/`classify_resolve`), which gains a
  caller-declared **expected fork state** — `AtBase` (worker_commit pre-act:
  `HEAD == base`) vs `Advanced` (import/heal: exactly one commit,
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
  contests).

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
| `worker_commit` | `attempt(RecordWorkerCommit)` **pre-act, always** — the fork binding is the same `DispatchRecord` the verb already reads for its base guard (claude arm = its only arm), so there is no unbound-but-legitimate case: refuses at `imported`+ (no late re-commit) and refuses `unprovable-fork` on a missing binding (RV-303 F-3 contest — no ungated fallback); the pre-gate is advisory against races, the post-act CAS record stays authoritative — **adds** post-land `RecordWorkerCommit` record on coord | delta/scope/base/gate belts (base guard `HEAD==B` stays) |
| `dispatch_import` | `attempt(Import)` (heal-forward per D8) | scope belt, merge compose, CAS |
| `dispatch verify` *(new)* | `preflight(Verify)` at entry (pure kind gate — evidence exists only post-suite, RV-304 F-3); the evidence-bearing `attempt(Verify)` runs at landing | — (see §5) |
| `dispatch_conclude_phase` | `attempt(Conclude)` — refuses `conclude-unverified` / `conclude-verify-failed` / `conclude-verify-stale` | **reordered** (RV-303 F-5): boundary⊕position CAS lands **first**; sheet flip + mirror becomes a trailing projection (crash after CAS ⇒ sheet lags position — benign, position is senior per §9) |
| `dispatch_reap` | `attempt(Reap)` | landed-oracle via `run_gc` |
| `dispatch record-boundary` (CLI escape hatch) | routes through `land_funnel_transition(Conclude)` — same authority, no bypass | — |

`arm-spawn` additionally lands the `Spawn` row (Class 2) after arming.

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
| `src/dispatch.rs` | sole-writer home (command tier, RV-304 F-6): `FunnelRecord` parse/render/read-at-tip, `land_funnel_transition` sole writer, verify (sync + suite + land), next, `derive_receipt_status` third input, guard verb; `current_branch` **relocates out** to `git.rs` (F-10) |
| `src/mcp_server/dispatch.rs` | gates on import/conclude/reap; new `dispatch_verify` + `dispatch_next` tools; `dispatch_phase_receipt` position field |
| `src/mcp_server/worker_commit.rs` | post-land `RecordWorkerCommit` record |
| `src/git.rs` | four **new** gap reads (`tree_clean_untracked`, `three_dot_diff`, `log_oneline`, `check_ignore`) + `current_branch` **relocated** from `src/dispatch.rs:1761` with callers migrated (RV-304 F-10) |
| `src/worktree/shared.rs` | `is_linked_worktree` re-export/wrap into the read surface (no move of the fn itself) |
| `src/worktree/dispatch_record.rs` | `DispatchRecord` gains `slice` + `phase` (fork-creation snapshot — the durable fork binding; F-4); resolver/classifier gains the caller-declared **expected fork state** (`AtBase` \| `Advanced` — RV-304 F-7). `arm-spawn` / `worktree fork --worker` gain the phase argument |
| `src/verify.rs` + `src/commands/check.rs` | runner extraction: status-returning `run_suite` callable below command tier; `check.rs` thin exit-forwarding shell (F-10) |
| `.doctrine/adr/001/layering.toml` | `funnel_machine = "leaf"` row — the pure-leaf claim gated by the architecture-layering check, not prose (F-12) |
| `src/worktree/create.rs` + `src/worktree/fork.rs` | **branch-as-claim fork sequence** (RV-304 F-2 + contests): claim (atomic branch create) → bind (record write under the claim, no-clobber belt) → act (worktree last); gc gains the branch-residue sweep |
| `src/commands/` | CLI wiring: `dispatch verify\|next\|commit\|tree-state\|delta\|whereami\|history\|ignored\|hook-check`; `arm-spawn` spawn-row; `record-boundary` reroute |
| `src/dispatch_config.rs` | `[dispatch] verify_suite` key |
| `src/doctor_checks.rs` | hook-present + worktreeConfig checks for live coord worktrees |
| hook script (embedded asset) | **new** — shipped pre-commit script; **flake.nix `srcWithDist` graft** (§13 R2) |
| `.agents/skills/` + `install/dispatch-mechanics.md` | funnel beats rewritten to verbs; rescue-idiom prose deleted |
| `.doctrine/spec/tech/021/` | D7 golden artifact (+ § prose at reconcile, ship-time REV per Non-Goals) |

## §11 Verification alignment (per requirement)

- **REQ-384**: VT — Class-1 atomicity (import/verify/conclude commits carry
  delta⊕position in one tree; inspect the landed commit); Class-2 idempotent
  re-record + heal-forward (kill between act and record, re-drive, position
  correct); lost-ref-race retry lands exactly one transition. VT —
  **lost-response import retry** (RV-304 F-1): CAS landed, reply lost,
  re-drive `dispatch_import` ⇒ no-op replay on matching `fork_tip` despite the
  freshly derived `onto`; distinct from the pre-CAS lost-ref-race retry (acts
  onto the new tip). VT — **branch-as-claim** (RV-304 F-2): kill between the
  claim/bind steps and the worktree act ⇒ inert residue (branch⊕record,
  never a live fork; gc sweeps; retry refuses at the claim);
  any live fork resolves its binding — driven through the production resolver
  with the `Advanced` expectation (RV-304 F-7), not a test-local read;
  **two concurrent same-name spawns ⇒ exactly one wins the atomic branch
  claim**, the loser refuses, and the winner's worktree pairs with the
  binding written under its own claim — the original binding stays
  byte-identical under any collision (F-2 contests); a crash-residue retry
  refuses at the claim naming the gc sweep. VT — a
  **fresh verify pass concludes** (the evidence commit never self-stales) and
  an unrelated post-verify commit refuses `conclude-verify-stale` (F-1). VT —
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
- **I1**: no code path writes `funnel.toml` except `land_funnel_transition`.
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
