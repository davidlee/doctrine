# SL-228 — Design: Zero-rescue dispatch funnel

> Status: **drafted, pending adversarial review** (internal pass integrated;
> external `/inquisition` with codex parked next). Design input:
> [`extraction.md`](extraction.md) (as-built graph, commit `0603f11f`) — this
> document works the **delta** over that graph and does not restate it.
> Requirements: SPEC-021 FR-008..011 (REQ-384..387), SPEC-022 FR-010/011
> (REQ-388/389), all `pending`, minted by REV-032.

## §0 Summary

Two moves, E before A, benchmark terminal:

- **Move E** — every funnel git read becomes a first-class read verb (five gaps
  built, the rest reused/absorbed); every coord-tree write is bounded by a
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
| **D3** | `dispatch verify` is **self-executing**: conditional forward-sync of the coord worktree (clean → ff to tip; dirty → typed refusal naming paths), run the configured suite (`[dispatch] verify_suite`, default `gate`, via the existing `check` pipeline), land evidence `{status, verified_oid, suite, at}` in one CAS commit. Ladder is **monotone**: fail leaves position at `imported` with red evidence; pass advances. Conclude gate is **strict**: `position == verified ∧ verify.verified_oid == coord tip`. | REQ-384/385; ISS-234 fix-direction (b) delivered as side effect |
| **D4** | `dispatch next` covers the **full ladder** (spawn prescription via the existing readiness authority → await-worker → import → verify → conclude → reap), terminal at all-reaped ("consult `dispatch status`"). `next` is read-only and never heals; drift is caught by verb gates whose refusals name the fixing verb. | REQ-386; OQ-5 forcing function |
| **D5** | Safe-commit guard = **verb + hook**: a pathspec-mandatory commit verb for the orchestrator's authored writes, plus a `dispatch setup`-installed per-worktree pre-commit hook (worktreeConfig) that refuses deletions in the commit set. Hook **chains to the effective non-worktree hooksPath** (local → global → system — the operator's global hook must keep firing) with fallback to common-gitdir hooks. Coord worktrees only (POL-002). | REQ-389; ISS-234 (6 repros); ADR-011 mechanism-over-prose |
| **D6** | Machine = **pure leaf** `src/funnel_machine.rs` (ADR-001); persistence in the engine tier as sole writer; every transport reaches the same engine writer, which consults the same leaf — projection additive for the deferred subprocess arm. Spec home: **inline SPEC-021 §** at reconcile; SPEC-022 FRs cross-reference (no new spec; REQ custody stays where REV-032 minted it). | REQ-387; NEW-OQ-A |
| **D7** | Drift resistance = **code-derived golden artifact**: the leaf's `const` transition table renders to a committed table+mermaid artifact under `.doctrine/spec/tech/021/`, pinned by a golden VT test; the SPEC-021 § embeds/references it. Structure pinned mechanically; semantics governed socially (REV). | NEW-OQ-B |
| **D8** | Class-2 transitions (act on a non-coord ref) are recorded **post-act** by the acting verb, with **heal-forward**: a later verb that can *prove* the missing Class-2 transition from git facts records it itself before its own transition. No rescue verbs, no read-side healing. | REQ-384 crash-safety; zero-rescue |

## §2 The machine (`src/funnel_machine.rs`, new — pure leaf)

No git, disk, clock, or engine imports (ADR-001 leaf; pure/imperative split).
Single-sources every state/transition token (STD-001).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Position { Spawned, WorkerCommitted, Imported, Verified, Concluded, Reaped }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Transition { Spawn, RecordWorkerCommit, Import, Verify, Conclude, Reap }

/// Non-positional gate inputs, gathered by the engine shell. Pure data.
pub(crate) struct TransitionFacts<'a> {
  /// Last verify evidence on the row (None = never verified).
  pub verify: Option<&'a VerifyEvidence>,
  /// The live coord tip at gate time (staleness check input).
  pub coord_tip: &'a str,
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

/// The FR-009 refusal payload — surfaced VERBATIM by verbs and rendered by
/// `next`: the refusal text IS the recovery procedure.
pub(crate) struct IllegalTransition {
  pub current: Option<Position>,
  pub attempted: Transition,
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
| `Verified` | `Conclude` | `verify.status == Pass ∧ verify.verified_oid == coord_tip` | `Concluded` |
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

Attempting a transition whose target equals `current` **with matching key facts**
(same fork tip for `RecordWorkerCommit`, same `verified_oid` for `Verify`, etc.)
returns `Ok` as a **no-op replay** (reported, not refused) — crash-retry safe on
every arm. Mismatched facts refuse (`already-<position>` + detail).

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
  import_oid = "…"               # the import commit itself
  at = "…"

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

Engine (in `src/dispatch.rs`): `FunnelRecord::parse/render`, `read_funnel_at(root,
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
  ConcludeIncomplete gap structurally impossible in funnel flows).
- **Class 2 — post-act record** (the act lives on another ref/medium):
  `Spawn` (worktree+branch creation; recorded by `arm-spawn`),
  `RecordWorkerCommit` (fork ref advance; recorded server-side by
  `worker_commit` immediately after it lands the fork commit),
  `Reap` (worktree/branch deletion; recorded by `dispatch_reap` after `run_gc`).
  Rule (D8): record lands strictly **after** the act. Crash window ⇒ position
  lags reality; recovery is **heal-forward over the provable prefix**: a verb
  that can prove missing earlier transitions from git facts records them before
  its own. `dispatch_import` at `position == spawned` with `fork tip ≠ base`
  lands `RecordWorkerCommit` then `Import`; at `position == None` with a live
  fork it heals the full prefix (`Spawn` + `RecordWorkerCommit`) — which is also
  what lets the **deferred subprocess arm** enter the gated funnel from import
  onward before its own spawn verbs record (Non-Goals stays honest);
  `dispatch_reap` re-run at `concluded` with the fork already absent records
  `Reaped` (idempotent completion). Re-running the original recorder is a no-op
  replay (§2). `worker_commit` resolves fork → (slice, phase) via the spawn
  row's `fork` field; no spawn row ⇒ skip the record (import heals it).

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
| `worker_commit` | none positional (worker-side; base guard `HEAD==B` stays) — **adds** post-land `RecordWorkerCommit` record on coord | delta/scope/base/gate belts |
| `dispatch_import` | `attempt(Import)` (heal-forward per D8) | scope belt, merge compose, CAS |
| `dispatch verify` *(new)* | `attempt(Verify)` | — (see §5) |
| `dispatch_conclude_phase` | `attempt(Conclude)` — refuses `conclude-unverified` / `conclude-verify-failed` / `conclude-verify-stale` | sheet flip + mirror, boundary CAS |
| `dispatch_reap` | `attempt(Reap)` | landed-oracle via `run_gc` |
| `dispatch record-boundary` (CLI escape hatch) | routes through `land_funnel_transition(Conclude)` — same authority, no bypass | — |

`arm-spawn` additionally lands the `Spawn` row (Class 2) after arming.

## §5 `dispatch verify` (new verb: CLI + MCP `dispatch_verify`)

`dispatch verify --slice N --phase PHASE-NN`:

1. `resolve_coord`; `attempt(Verify)` legality (must be `imported`/`verified`).
2. **Conditional forward-sync** of the coord worktree to the coord tip. NOT a
   `reset` — the ref advanced *under* the checkout, so HEAD already IS the tip
   and the tree/index lag it; `reset --keep` cannot resync this shape (known
   trap, memory-pinned). Mechanism: compute the **reverse-diff set** (paths
   changed by funnel-provenance commits the tree hasn't caught up to — the
   funnel record + `FUNNEL_MARKER` history make it derivable), then
   `git restore --source=HEAD --staged --worktree -- <that set>` (the ISS-234
   known-good idiom, verb-owned now). Any index/worktree delta **outside** the
   reverse-diff set ⇒ typed refusal `verify-tree-dirty` naming the paths (never
   discard uncommitted work). This collapses the ISS-234 reverse-diff window
   for the rest of the phase funnel as a side effect.
3. Run the configured suite: `[dispatch] verify_suite` (default `"gate"`)
   through the existing `check` pipeline (project-configured recipes — POL-002:
   the bar is project posture, not framework doctrine).
4. Land evidence in one CAS commit: pass ⇒ position `verified`; fail ⇒ position
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
one phase may be mid-funnel; the oracle stays single-prescription —
deterministic pick (lowest phase id mid-funnel), the other in-flight phases
surfaced in `detail`.

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
- commits path-limited in the coord tree; when a *declared* deletion passed its
  own validation, the verb sets the hook's escape env for its child `git commit`
  only — the two guards compose instead of deadlocking. The orchestrator's
  authored-`.doctrine/` writes route here; skills drop their raw `git commit`
  prose.

### The hook — coord-worktree pre-commit backstop

Installed by `dispatch setup` (and re-checked by `doctor`): enable
`extensions.worktreeConfig` (one-time, repo-local), set **per-worktree**
`core.hooksPath` to a doctrine-owned dir under the coord worktree's private
gitdir, write the shipped hook script there (embedded in the binary —
**flake.nix graft required**, see §13 R2).

Hook behaviour (runs on ANY `git commit` in the coord worktree, any harness —
parity by construction, the ADR-011 disk-over-env argument; VA-gated anyway):

1. Inspect the to-be-committed diff (the temporary index git builds for
   pathspec commits makes this exact): **any deletion ⇒ refuse**, naming the
   paths, the escape hatch (`DOCTRINE_ALLOW_DELETE=1`), and `dispatch commit`.
   This is the reverse-diff's precise signature — mass staged deletions the
   author never asked for (ISS-234).
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
| current branch (gap 3) + isolation classification | shell `rev-parse` / rebuild risk | **`dispatch whereami`**: branch, `is_linked_worktree` (**wrapped from `src/worktree/shared.rs:54` — relocation/wrap, NOT reimplementation**, RV-300 F-7), primary/coord/fork classification |
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
- `Some(p)`:
  - `p ≥ Concluded` ⇒ `Completed` (boundary-backed by construction; if
    `!has_boundary` ⇒ `ConcludeIncomplete` — now an **integrity alarm**:
    structurally impossible via the funnel, so it means record corruption or a
    hand-landed commit; fail-loud, doc-comment updated, wire token unchanged).
  - `p < Concluded` ∧ sheet says `completed` ⇒ `ConcludeIncomplete` (alarm:
    sheet claims ahead of authority — degraded-mirror detection).
  - otherwise sheet drives `InProgress`/`Blocked` as today (worker-condition
    axis stays sheet-tier; the ladder does not grow off-ladder nodes).

`dispatch_phase_receipt` payload gains `position: Option<String>`.
`run_status` phase table rendering: unchanged (legacy strings verbatim).
Altitude-B readiness (`plan_next_rows` / `compute_next_phases`): untouched.

## §10 Code impact summary (→ `design-target` selectors)

| Path | Change |
|---|---|
| `src/funnel_machine.rs` | **new** — pure leaf: `Position`, `Transition`, `attempt`, `expected_next`, refusal payload, `const` table + renderer for the D7 artifact |
| `src/dispatch.rs` | engine: `FunnelRecord` parse/render/read-at-tip, `land_funnel_transition` sole writer, verify engine (sync + suite + land), next engine, `derive_receipt_status` third input, guard-verb engine |
| `src/mcp_server/dispatch.rs` | gates on import/conclude/reap; new `dispatch_verify` + `dispatch_next` tools; `dispatch_phase_receipt` position field |
| `src/mcp_server/worker_commit.rs` | post-land `RecordWorkerCommit` record |
| `src/git.rs` | five gap reads: `tree_clean_untracked`, `three_dot_diff`, `current_branch`, `log_oneline`, `check_ignore` |
| `src/worktree/shared.rs` | `is_linked_worktree` re-export/wrap into the read surface (no move of the fn itself) |
| `src/commands/` | CLI wiring: `dispatch verify\|next\|commit\|tree-state\|delta\|whereami\|history\|ignored`; `arm-spawn` spawn-row; `record-boundary` reroute |
| `src/dispatch_config.rs` | `[dispatch] verify_suite` key |
| `src/doctor_checks.rs` | hook-present + worktreeConfig checks for live coord worktrees |
| hook script (embedded asset) | **new** — shipped pre-commit script; **flake.nix `srcWithDist` graft** (§13 R2) |
| `.agents/skills/` + `install/dispatch-mechanics.md` | funnel beats rewritten to verbs; rescue-idiom prose deleted |
| `.doctrine/spec/tech/021/` | D7 golden artifact (+ § prose at reconcile, ship-time REV per Non-Goals) |

## §11 Verification alignment (per requirement)

- **REQ-384**: VT — Class-1 atomicity (import/verify/conclude commits carry
  delta⊕position in one tree; inspect the landed commit); Class-2 idempotent
  re-record + heal-forward (kill between act and record, re-drive, position
  correct); lost-ref-race retry lands exactly one transition. VT — D7 golden
  artifact matches the leaf table.
- **REQ-385**: VT — table-driven exhaustive `(position × transition)` legality
  matrix (every illegal pair refuses with the right `expected`/`reason`);
  conclude refusals: unverified / verify-failed / verify-stale each distinct.
- **REQ-386**: VT — oracle total over the domain (every reachable position ⇒
  exactly one prescription; property-style over the table); terminal at
  all-reaped; `next` agrees with `expected_next` by construction (same fn).
  VA — OQ-5 benchmark (below).
- **REQ-387**: VT — the CLI escape hatch (`record-boundary`) and the MCP verbs
  land through `land_funnel_transition` (no second writer: enforced by
  visibility — the splice/commit fns are private to the writer module); VA —
  code-shape review confirms no position write bypasses `attempt`.
- **REQ-388**: VT — five gap reads (unit, over fixture repos); VA — skill sweep:
  no raw-git funnel reads remain in dispatch skills/mechanics docs.
- **REQ-389**: VT — hook refuses a deletion-carrying commit (the ISS-234 repro:
  ref-advance → pathless commit → refused); escape hatch works; **chained
  global hook still fires** (the operator-gotcha case: global `core.hooksPath`
  set → both run); guard verb refuses undeclared paths/deletions, pathless is
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
  funnel touches the working tree; it is a path-bounded `restore` over the
  derived reverse-diff set only (§5 — `reset --keep` is a known-broken shape
  here), refuses on any delta outside that set, and never discards uncommitted
  work (the no-stash/no-discard rule).
- **R4 — hook shadowing**: per-worktree hooksPath outranks local *and global*
  config; chaining is load-bearing (operator's global hook — VT-pinned).
- **R5 — two-altitude confusion** (extraction finding): `next` (sub-funnel) vs
  `select_guidance` (slice lifecycle) must be documented as distinct oracles;
  `next`'s terminal beat hands off explicitly. Skill prose keeps one loop:
  phase-funnel = `next`; post-phases = `status`.
- **I1**: no code path writes `funnel.toml` except `land_funnel_transition`.
- **I2**: no funnel verb mutates the coord index/worktree except verify's
  gated ff-sync (and `dispatch commit`, which commits only named paths).
- **I3**: positions only advance (monotone); evidence may update in place.

## §14 Question ledger

All slice-carried questions settled: OQ-1 (REV-032: prescribe + refuse — both),
OQ-2 → D1/§3, OQ-7 → §8, NEW-OQ-A → D6, NEW-OQ-B → D7. Deferred by design
(unchanged from scope Non-Goals): subprocess-arm full gating (machine makes it
additive), MCP mirror of `dispatch commit` (rides REQ-335 reconciliation),
OQ-3/OQ-4/move-D tail, ship-time sibling REV for the four active-REQ modifies,
OQ-6 retirement list (rides the benchmark phase).
