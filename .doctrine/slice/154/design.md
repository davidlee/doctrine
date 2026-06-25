# Design SL-154: Reliable conformance-registry capture

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->
<!-- Body revised 2026-06-26 to the COMMITTED-REF model (ISS-039 absorbed). The
     prior working-file-read draft (codex pass-2 P2-3, SPEC-022 violation) is
     fully retracted; this body is the single source of truth. §10 is the review
     ledger (history of all passes + the decision trail). -->

## 1. Design Problem

RFC-004 v0.1 (SL-147) shipped `slice conformance`, which diffs a slice's declared
`design-target` selectors against its **actual** git delta. The actual-side input is
the arm-neutral **conformance registry** — `.doctrine/state/slice/NNN/boundaries.toml`
(runtime tier) — one `[[boundary]]` row per landed phase. The consumer fail-closes
when the registry is incomplete, so an unpopulated registry makes conformance
unavailable **at audit — exactly when it is wanted**.

Two landing paths feed the registry and both leak:

- **ISS-051 (solo path):** a phase (the final phase most often) can land no row.
- **ISS-052 (funnel path):** a dispatched slice can reach audit with the registry
  empty (SL-153).

Design surfaced three deeper issues, all now in scope:

1. The capture model is **unsound for dispatched and mixed solo↔dispatch slices**
   (§2 — the branch-proxy guard mis-fires when phase flips run from the session root).
2. The only spec-legal funnel source — the dispatch boundaries ledger read from the
   `dispatch/NNN` tip — is **empty in practice** because the claude arm never commits
   it (**ISS-039**; SPEC-022 violation). This slice **absorbs ISS-039**.
3. The reopen path leaves a **stale registry row the gate blesses** (P2-1), and the
   coord-worktree liveness probe is **unsound against prunable entries** (P2-2).

This slice closes both population leaks and makes recording robust across landing-path
transitions — registry-population plus the bounded ledger-commit fix. The conformance
consumer and its algebra are untouched.

## 2. Current State

Empirically root-caused from code + topology (the SL-147/SL-153 registries on disk
were since hand-bootstrapped, so the original failing state is gone — root-cause is
from code, not live forensics).

### Solo binding (`state.rs::capture_phase_boundary`, bound to `set_phase_status`)

- `in_progress` flip → stamps `code_start_oid = HEAD` into the **phase sheet**
  (runtime) once; `completed` flip → records `(stamp, HEAD)` via
  `record_source_delta` (F-6 guard: `is_ancestor` + non-merge `end`; upsert by phase).
- An **empty range** (`start == end`) records fine (a present row). So the scope's
  hypothesis (a) ("end read early → start==end → dropped row") is **refuted**. A row is
  dropped **only when the start-stamp is absent** (state.rs:524 swallowed-warning
  degrade): the phase never entered `in_progress` under the current binding (a stale
  PATH binary flipped it; or a bootstrap slice predating the binding — SL-147's own
  case), or the runtime tier was wiped.
- `init_phases` is **per-file-skip**, so re-running `slice phases` does not clobber
  the stamp; only a full `.doctrine/state` wipe does — which takes `boundaries.toml`
  with it (same tier). Relocating the stamp buys nothing.

### The unsound-capture finding (dispatched + mixed slices)

- **Phase status flips are authored writes run from the session root** (dispatch
  skill:20: "Step out to the session root only for authored writes (slice status…)").
  There, `HEAD` is `edge`/`main` — **not** a dispatched phase's code tip on
  `dispatch/NNN`.
- The arm-guard skips solo capture only when `current_branch(project_root) ==
  dispatch/NNN` (state.rs:481). Flipped from the session root, that branch is `edge`,
  so the guard **does not fire** — the solo binding would capture a *dispatched*
  phase against the wrong tree. With `start == end == edge HEAD` it manufactures an
  **empty-range row** that `registry_completeness` (presence-only) **blesses**. The
  branch-proxy guard is unsound under the real flip-from-session-root workflow.
- Net: for dispatched/mixed slices the solo binding can both *miss* real phases and
  *manufacture a passing-but-garbage row*. Objective 3 (mixed-mode coherence) is a
  first-class target, not a footnote.

### Funnel — the spec-mandated source is committed, but the claude arm never commits it (ISS-039)

- **SPEC-022 § "Run-ledger object-db sourcing" (spec-022.md:180) is unambiguous:** the
  run ledger — `journal.toml`, **`boundaries.toml`**, `orthogonal.toml` under
  `.doctrine/dispatch/NNN/` — is **tree-read from the `dispatch/NNN` branch tip**
  (`read_path_at` against the object db), *never the working filesystem*, **identically
  in stage-1 and stage-2**. That checkout-independence is what lets audit run from the
  root while the coordination tree is elsewhere.
- `read_ledger` (dispatch.rs:1991) implements exactly that read. But the claude arm
  **never commits `boundaries.toml`** onto `dispatch/NNN` — only `journal.toml` is
  spliced (`commit_journal`, dispatch.rs:2094). So `read_ledger::<Boundaries>` returns
  `Boundaries::default()` (empty), `plan_phases` projects **0 phase-cuts** (the visible
  ISS-039 symptom), and **there is no spec-legal per-phase source at `prepare-review`**.
  ISS-052's clean fix is therefore *blocked on* ISS-039: the implementation must be
  brought into SPEC-022 conformance by committing the ledger.
- Registry population today rides a per-arm hand-step at the funnel Record beat (router
  step 8): claude `dispatch record-boundary` writes the dispatch **ledger working file**
  *and* the registry; codex/pi `slice record-delta` writes the registry only. No
  machinery beat guarantees a landed phase deposits a row; SL-153 reached audit empty.

### Reopen + liveness footguns (codex pass-2)

- **P2-1 — reopen leaves a stale row.** The reopen path (`set_phase_status`,
  state.rs:386–401) clears `completed`/`started` but **not** `code_start_oid`; capture
  keeps the original stamp on re-entry (state.rs:503); `registry_completeness` checks
  *presence*, not *range freshness*. A phase reopened after a transition keeps its old
  registry row; the guard stands down; the derive has no fresh ledger row to overwrite
  → silent garbage conformance.
- **P2-2 — `worktree_for_ref` is not a liveness probe.** `parse_worktree_for_ref`
  (git.rs:1163) ignores `prunable` and never stats the path, so a deleted /
  failed-cleanup coord entry reads as "live" and suppresses solo capture **forever**
  (POL-002 footgun).

### Constraints discovered

- **Audit precedes integrate.** `slice conformance` runs at audit; stage-2
  `dispatch sync --integrate` is `/close`'s job *post-audit*. So the registry must be
  complete by **`prepare-review`** (the mandatory pre-audit conclude beat), not integrate.
- **Conformance does not strip `.doctrine/`.** `conformance_outcome` builds `actual`
  from `git diff --name-status start..end`, folding **every** path (slice.rs:1919–1928).
  So any start that is not the phase's *exact* code start mis-attributes intervening
  knowledge/notes commits → false `undeclared` edits. This kills naive chaining
  (§7 D1).

### Pre-built machinery to reuse

- `state::check_completeness` / `registry_completeness` (state.rs:654, :765) — the
  pure F-2 cross-check `slice conformance` already uses to fail closed. Note
  `registry_completeness(cwd, project_root, id)`: `recorded` normalizes to the
  primary tree (`primary_worktree`), but `completed` reads `phases_dir(project_root)`
  — the *local* tree (state.rs:743). They coincide **only on the primary tree** → D4.
- `dispatch.rs::read_ledger` (:1991) — the committed-ref reader; the **single** funnel
  source now (derive + `plan_phases` both go through it).
- `dispatch.rs::commit_journal` (:2094) — splices `journal.toml` into the tip tree and
  advances `dispatch/NNN` under CAS (no checkout); `git::tree_with_file` +
  `commit_tree` + `update_ref_cas`. The exact pattern the boundaries-ledger commit
  mirrors (§5, D7).
- `state::record_source_delta` (:613) — the single upsert writer (F-6 guard).
- `git::worktree_for_ref` (:1189) / `primary_worktree` (:554) — worktree locators; the
  former wrapped for liveness (P2-2, D9).

## 3. Forces & Constraints

- **SPEC-022 (run-ledger object-db sourcing):** the dispatch ledger — including
  `boundaries.toml` — is read from the `dispatch/NNN` tip, never the working FS,
  identically stage-1/stage-2. **Binding** — the funnel source must be the committed
  ref. Absorbing ISS-039 brings the impl into conformance; **no REV** (the spec already
  mandates the committed ledger — D10).
- **ADR-001 (layering):** new logic stays pure where it can; git/disk in the shell.
- **POL-002 (platform independence):** recording keys on doctrine-owned signals
  (recorded SHAs, the committed `dispatch/NNN` ledger, a *liveness-verified* coord
  worktree) — never host commit conventions.
- **Behaviour-preservation gate:** existing `set_phase_status` and dispatch suites
  must stay green; the solo *stamp-present* path stays byte-identical; shared
  `git::worktree_for_ref` callers are untouched (new liveness wrapper, D9).
- **R-5 belt:** PHASE commits strip `.doctrine/`. The boundaries-ledger commit is a
  **separate doctrine-mediated commit** (like the journal), never a phase commit.
- **Audit-before-integrate:** enforcement point is `prepare-review`.
- **Conformance folds all paths** (no `.doctrine/` strip): only an *exact* phase
  start is sound (kills chaining — D1).
- **Arm bound (IMP-171):** the dispatch boundaries ledger is claude-only; a symmetric
  codex/pi ledger couples to `phase/<N>` projection turning on unconditionally
  (dispatch.rs:2049) — deferred. The ISS-039 commit here is claude-arm-bounded.
- `record-delta` stays the manual escape hatch.

## 4. Guiding Principles

- **Each phase is recorded by the writer that holds its exact range.** Funnel phases
  → the committed ledger (and the derive that reads it); solo phases → the binding at
  the `completed` flip, in a true solo context.
- **One reconciliation point makes the authoritative source win.** The `prepare-review`
  derive **upserts** from the committed ledger, so it both auto-heals missing funnel
  rows *and overwrites* any garbage a mis-firing binding wrote for a dispatched phase.
- **Auto-heal where it is sound; fail loud where the data is destroyed.** Funnel rows
  are soundly recoverable retroactively (the committed ledger persists on the ref). A
  pure-solo phase's range exists only at flip-time; if lost there it is physically
  unrecoverable — fail closed + `record-delta`. **Never manufacture a passing row that
  is wrong** (a wrong conformance verdict is worse than a flagged gap) — this is why
  the guard stays (§5, R2).
- **Sound signals, not proxies.** The guard keys on "a *live* coord worktree exists for
  this slice" (liveness-verified), not the ambient branch name.
- **Conform to the spec, don't paper over it.** The funnel source is the committed ref
  the spec already mandates; ISS-039 is fixed at its root, not bypassed.

## 5. Proposed Design

### 5.1 System Model

```
solo completed-flip ──(guard: no LIVE coord worktree)──> record_source_delta ─┐
prepare-review:                                                               │
  1. commit-boundaries: splice working ledger → dispatch/NNN tip (SPEC-022)   │
  2. derive: read_ledger(committed) → record_source_delta per row (UPSERT) ───┤
codex/pi record-delta ────────────────────────────────────────────────────────┤
manual record-delta (escape hatch) ────────────────────────────────────────────┤
                                                                               v
                                              .doctrine/state/slice/NNN/boundaries.toml
                                                                               │
                            prepare-review GATE: registry_completeness(primary,│primary,id)
                                                                               v
                                                              Complete | HALT (named gap)
```

Reopen (completed→non-completed) **evicts** the phase's registry row + clears its
stamp (P2-1), so a redo re-captures fresh or fails loud — never a stale blessed row.

- **ISS-039 commit (funnel prerequisite).** At `prepare-review`, before any read, splice
  the live coord worktree's working `boundaries.toml` onto the `dispatch/NNN` tip via a
  doctrine-mediated commit (mirrors `commit_journal`). Now `read_ledger`, `plan_phases`,
  and the derive read **one** committed, checkout-independent source (SPEC-022-legal).
- **Solo binding (solo phases).** Keep the stamp; record `(stamp, HEAD)` at `completed`.
  **Guard:** skip iff a *live* coord worktree exists for `dispatch/NNN` (the slice is
  under active dispatch → the funnel/derive owns recording). Stamp absent → no row + a
  surfaced warning; the gate / conformance fail-closed catches it.
- **Derive-at-gate (funnel phases), authoritative + self-correcting.** At
  `prepare-review`, read the **committed** ledger and `record_source_delta` each row
  (upsert) — overwrites any binding mis-capture, fills any missing funnel row.
- **Gate (both arms).** `registry_completeness` resolved against the **primary** tree
  for *both* the completed-set and the registry; `bail!` on any gap.
- **Funnel inline double-write retained.** `run_record_boundary` is **unchanged**
  (ledger working file + registry); the derive is a redundant-but-authoritative
  reconciler over it (no contract break — codex F5).

### 5.2 Interfaces & Contracts

**`prepare_review` (dispatch.rs:1497)** — new steps bracket the existing projection:

```text
let coord_ref = "refs/heads/dispatch/NNN";
let tip0 = resolve_commit(root, coord_ref)?;                 // existing

// (1) ISS-039 commit — land the working boundaries ledger onto the tip (D7).
//     No-op when no live coord worktree / no working file (re-run, early removal).
let tip = match git::live_worktree_for_ref(root, coord_ref)? {        // D9 liveness wrapper
    Some(coord) => match ledger::read_boundaries_file(&coord, slice)? {  // NEW reader (OQ-4)
        Some(body) => commit_boundaries(root, &tree_of(root,&tip0)?, &tip0,
                                        coord_ref, &body)?,  // splice; CAS; returns new tip
        None => tip0,
    },
    None => tip0,
};
let tip_tree = tree_of(root, &tip)?;
let trunk_base = merge_base(root, &tip, &trunk_tip)?;        // recomputed off the new tip

// existing: read committed ledgers (now boundaries is populated), project refs
let boundaries = read_ledger::<Boundaries>(root, coord_ref, slice3, "boundaries.toml")?;
// ... plan_review + plan_phases + with_journaled_projection (journal.toml splice) ...

// (2) derive registry from the SAME committed boundaries + (3) gate (D2/D4).
let primary = git::primary_worktree(root)?;
for row in &boundaries.rows { state::record_source_delta(&primary, slice, row.clone())?; }
match state::registry_completeness(&primary, &primary, slice)? {
    Complete            => {}
    Incomplete { gaps } => bail!("prepare-review: conformance registry incomplete: {gaps}; \
                                  record-delta the missing phase(s) before audit"),
}
```

New `commit_boundaries(root, base_tree, parent, coord_ref, body) -> Result<String>`:
the boundaries twin of `commit_journal` — `tree_with_file(base_tree,
".doctrine/dispatch/NNN/boundaries.toml", body)` → `commit_tree(parent,
"ledger: boundaries")` → `update_ref_cas(coord_ref, commit, parent)`. (DRY note — OQ-6:
factor a shared `splice_ledger_file(...)` that both `commit_journal` and
`commit_boundaries` call; deferred to /plan if it reads cleanly.) The working ledger is
spliced **verbatim** (the bytes the worker wrote), not re-serialized — fidelity, and
`read_ledger` parses it back unchanged.

New `ledger::read_boundaries_file(worktree_root, slice) -> Result<Option<String>>`: the
raw working-file reader over the worktree-relative `dispatch_dir` path (`dispatch_dir`
is private to ledger.rs, :375 — expose this reader rather than rebuild the string in
dispatch.rs; OQ-4).

**Solo binding** (`capture_phase_boundary`, state.rs): two changes only.
1. **Guard predicate** branch-proxy → *live* coord-worktree presence (D3 + D9):
   ```rust
   // was: current_branch(project_root) == format!("dispatch/{slice_id:03}")
   // now: a LIVE coordination worktree for this slice owns recording.
   match crate::git::live_worktree_for_ref(project_root, &format!("refs/heads/dispatch/{slice_id:03}")) {
       Ok(Some(_)) => return None,       // under active dispatch — funnel/derive owns it
       Ok(None)    => {}                  // solo context (or stale entry) — record
       Err(e)      => { warn_capture(phase_id, &format!("coord probe failed: {e}")); return None; }
   }
   ```
   Sound from any tree (works when the flip runs from the session root) and immune to a
   pruned/stale coord entry (D9). No chaining — the absent-stamp branch records
   **nothing** (surfaced warning), unchanged in its non-blocking posture.
2. Stamp-present path **unchanged** (precise `(stamp, HEAD)`).

**Reopen eviction** (`set_phase_status`, state.rs:386–401) — P2-1, D8. On a
completed→non-completed transition (detected from the pre-write status):
```text
if old_status == Completed && new_status != Completed {
    table.insert("code_start_oid", value(""));         // clear stamp → redo re-stamps fresh
    state::forget_source_delta(&primary, slice, phase)?;  // evict the registry row (NEW)
}
```
New `state::forget_source_delta(cwd, slice, phase) -> Result<bool>`: read-modify-write
removal of the phase's row from `boundaries.toml` (the inverse of `record_source_delta`;
absent → `false`, no error). Resolved against the **primary** tree (same as the writer).

**Funnel** — `run_record_boundary`: **unchanged** (double-write retained). `record-delta`
verb: **unchanged** (escape hatch).

### 5.3 Data, State & Ownership

- **Registry** `.doctrine/state/slice/NNN/boundaries.toml` (runtime, primary-resolved):
  shape unchanged. Writers: solo binding, `prepare-review` derive (new), funnel inline
  write, codex/pi + manual `record-delta`. Eviction: reopen (new). All mutations via
  `record_source_delta` (upsert) / `forget_source_delta` (remove) against the primary
  tree → idempotent; the derive is the authoritative last writer for funnel phases.
- **Dispatch ledger** `.doctrine/dispatch/NNN/boundaries.toml` (claude-only): written as
  a working file by `run_record_boundary` during the drive; **now committed** onto
  `dispatch/NNN` at `prepare-review` (new, ISS-039). Read via `read_ledger` (committed
  ref) by both the derive and `plan_phases` — one source, SPEC-022-legal.
- **Phase sheet** `.../NNN/phases/phase-NN.toml`: carries the stamp (precision input);
  the stamp is **cleared on reopen** (P2-1). No relocation.

### 5.4 Lifecycle, Operations & Dynamics — landing-path transitions

| Slice shape | Solo phases | Funnel phases | Reconciliation |
|---|---|---|---|
| Pure solo | binding records (no live coord worktree → guard never fires) | — | conformance `check_completeness` (final net) |
| Pure dispatch | binding stands down (live coord worktree) | commit + derive (authoritative) | prepare-review gate |
| **Solo→dispatch (SL-153)** | binding records **before** the drive (solo context) | commit + derive from committed ledger | gate checks the **union** |
| Dispatch→solo | commit + derive | binding records **after** conclude (coord gone) | conformance final net (post-gate solo work) |
| Interleaved | binding per solo phase; derive **upserts/corrects** any funnel phase the binding mis-grabbed | commit + derive | gate + conformance |

Load-bearing mechanism: **commit-then-derive-at-gate with upsert** is both the funnel
auto-heal and the corrector for cross-context mis-captures — a transition cannot leave a
wrong or missing *funnel* row. A solo phase completed *during* an active drive (live
coord worktree present) is the one crack: the binding stands down and the funnel does
not own it → no row → the gate halts loudly → `record-delta`. Rare, loud, recoverable —
the accepted cost of never blessing a garbage row (R2).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1:** by audit, every completed phase has exactly one registry row (binding for
  solo; commit+derive for funnel; gate enforces; conformance is the pure-solo net).
- **INV-2:** `record_source_delta` upsert ⇒ no duplicates across writers; the derive is
  the authoritative last write for funnel phases.
- **INV-3:** the stamp, when present, is authoritative for solo start (precision); a
  reopen clears it so a redo never reuses a stale start (P2-1).
- **INV-4:** the derive and `plan_phases` read the **same** committed boundaries ledger
  — no source divergence (codex F4 closed by construction).
- **Irreducible manual case:** a **pure-solo** phase whose flip-time capture was lost
  (stale binary / wiped runtime tier) and that has **no committed ledger row** — its
  range is physically destroyed; no sound retroactive reconstruction (chaining folds in
  inter-phase commits — D1). Fails loud at the gate / conformance + `record-delta`.
  **Accepted floor.**
- **Empty-code phase:** `start == end` records a present row (unchanged); satisfies the
  gate; `plan_phases` emits no ref for it (dispatch.rs:2050).
- **Commit-boundaries idempotency:** re-running `prepare-review` re-splices the working
  ledger (identical bytes → identical tree → CAS no-op or a same-content commit) and
  re-derives (upsert) — safe.
- **Coord worktree absent / stale at prepare-review:** commit-boundaries no-ops (the
  ledger committed by a prior run stays on the tip); the derive reads whatever is on the
  committed ref; the gate halts on any resulting gap.

## 6. Open Questions & Unknowns

- **OQ-1 (resolved → D1):** chain-fallback for a lost solo stamp? **No** — unsound.
- **OQ-2 (resolved → D5):** drop `run_record_boundary`'s registry half? **No, keep** —
  redundant-but-harmless; dropping is a contract break (codex F5).
- **OQ-3 (resolved):** any consumer that reads the registry *mid-drive*, before
  `prepare-review`? None found — conformance runs at audit, post-`prepare-review`.
- **OQ-4 (resolved → D7):** expose `ledger::read_boundaries_file` (DRY) vs rebuild the
  path in `dispatch.rs`. The reader — `dispatch_dir` is private to ledger.rs (:375).
- **OQ-5 (resolved → D9):** does the guard's probe cost every status flip? One `git
  worktree list`, liveness-filtered. Acceptable; only the active-dispatch case stands down.
- **OQ-6 (open, /plan):** factor a shared `splice_ledger_file` for `commit_journal` +
  `commit_boundaries`? Decide at implementation if it reads cleanly (DRY vs premature
  abstraction over two callers).
- **OQ-7 (open, verify):** does committing the boundaries ledger re-enable claude
  per-phase projection (`plan_phases` now sees rows) cleanly? `e2e_dispatch_lifecycle`
  expects `phase/064-01` — confirm phase-cuts firing does not break it (§9).

## 7. Decisions, Rationale & Alternatives

- **D1 — Drop chaining; no `resolve_phase_start`.** Conformance folds every path with
  no `.doctrine/` strip (slice.rs:1919), so `start = prev.end` mis-attributes
  inter-phase commits → false `undeclared`. A wrong row is worse than a gap. The stamp
  is the only sound exact start; absent it, record nothing and fail closed.
- **D2 — Commit-then-derive-at-gate, authoritative + self-correcting (upsert).** The
  funnel auto-heal *and* the corrector for the unsound-capture finding (§2). Reads the
  **committed** ledger (SPEC-022-legal), at `prepare-review`.
- **D3 — Sound guard: live coord worktree, not branch-proxy.** The branch-proxy fails
  under flip-from-session-root (§2). "A live coord worktree exists for `dispatch/NNN`"
  is a doctrine-owned, tree-independent signal for "the funnel owns recording".
  **Kept (not removed) despite the authoritative derive:** without it, a dispatched
  phase flipped from the session root writes an **empty-range row** the presence-only
  gate blesses; if the funnel also missed that phase the derive has no row to overwrite
  → the gate passes with garbage. The guard makes that case **halt loudly** instead.
- **D4 — Primary-rooted gate.** `registry_completeness(primary, primary, id)` so the
  completed-set and the registry are read from the same canonical tree (codex F1).
- **D5 — Keep the funnel inline double-write** (reverses an earlier drop). Redundant
  under the derive, but no contract break (codex F5) and more robust.
- **D6 — Defer codex/pi symmetric derive (IMP-171).** The reproduction is claude-arm;
  a codex/pi ledger couples to untested phase-ref projection. ISS-039 absorbed here is
  claude-arm-bounded.
- **D7 — Commit the boundaries ledger via a `prepare-review` splice (absorbs ISS-039).**
  Mirror `commit_journal`: one doctrine-mediated commit at the top of `prepare_review`,
  before any read, landing the working ledger onto `dispatch/NNN`. Chosen over a
  per-phase commit during the drive (N commits, a new pattern journal.toml doesn't use,
  earlier availability not needed since `prepare-review` is the read point). Separate
  from phase commits (R-5 belt strips `.doctrine/` — this is the journal's pattern).
- **D8 — Reopen evicts the row + clears the stamp (P2-1).** A completed→non-completed
  transition removes the phase's registry row (`forget_source_delta`) and clears
  `code_start_oid`, so a redo re-captures fresh or fails loud — never a stale blessed row.
- **D9 — Liveness-verified coord probe (P2-2).** New `git::live_worktree_for_ref`
  wrapper: reject `prunable`, stat the path. Used by the guard **and** the
  commit-boundaries locator. Shared `worktree_for_ref` callers untouched
  (behaviour-preservation). Chosen over a new dispatch-active runtime marker
  (`worktree/marker.rs` is per-worker, not a coord-root signal — more moving parts).
- **D10 — No REV for SPEC-022.** SPEC-022:180 *already* mandates the committed
  boundaries ledger; ISS-039 is the impl in violation. Committing it is conformance, not
  a spec change — verified the spec text names `boundaries.toml` in the committed
  run-ledger set. (If any spec text had described it as uncommitted, a REV would route —
  it does not.)
- **Alternatives rejected** — chain-fallback (B): unsound (D1). Working-file derive
  (the retracted pass-2 draft): SPEC-022 violation (P2-3). Read-time fallback in
  conformance: scope-rejected; papers over the write gap.

## 8. Risks & Mitigations

- **R1 — derive reads a stale/incomplete committed ledger.** A funnel phase the inline
  write never recorded to the working ledger never gets committed/derived → that row is
  missing → the **gate** halts loudly with the named gap. Fail-closed, never silent.
- **R2 — guard false-stand-down (the §5.4 crack).** A genuinely-solo phase completed
  during an active drive (live coord worktree present) is skipped by the binding and not
  owned by the funnel → the gate catches it. Loud + `record-delta`. Accepted as the cost
  of never blessing a garbage row (the inverse — binding mis-captures a funnel phase — is
  corrected by the derive upsert; an *unrecorded* dispatched phase is the case the guard
  must protect, hence D3-kept).
- **R3 — gate false-halt.** `registry_completeness` keys on completed `PHASE-NN`; a
  blocked/not-completed phase is excluded. Mitigation: same completed-set source as
  conformance (parity); message names exact gaps + remedy.
- **R4 — ISS-039 commit perturbs projection / sync.** Committing the boundaries ledger
  re-enables claude per-phase projection (`plan_phases` now sees rows; 0 today). Risk: a
  suite pinned to 0 phase-cuts, or the extra commit shifting a tip a test asserts.
  Mitigation: §9 — run `e2e_dispatch_lifecycle` (expects `phase/064-01`) +
  `e2e_dispatch_sync`; the ledger commit excludes from `review/<slice>` already
  (`plan_review` drops `.doctrine/dispatch/NNN`, dispatch.rs:2015).
- **R5 — behaviour regression in the binding / shared locators.** Only the guard
  predicate, the absent-stamp branch, and the reopen branch change in state.rs; the
  stamp-present path is byte-identical; `worktree_for_ref` is wrapped, not modified.
  Mitigation: existing binding + dispatch suites green; add the regression tests (§9).
- **R6 — commit-boundaries CAS race.** Another writer advances `dispatch/NNN` between
  the tip read and the CAS. Mirrors `commit_journal`'s `RefCas::Moved` bail — report,
  never clobber.

## 9. Quality Engineering & Validation

- **Pure unit:** `check_completeness` (reuse). `forget_source_delta` round-trip
  (record → forget → absent). (No `resolve_phase_start` — D1.)
- **VT — ISS-039 commit:** a claude drive with N working-ledger boundaries →
  `prepare-review` commits `boundaries.toml` onto `dispatch/NNN` (`git ls-tree` shows it
  alongside `journal.toml`) and `read_ledger` reads N rows from the committed tip.
- **VT — guard soundness (the unsound-capture fix):** a dispatched phase flipped from a
  session-root context with a live coord worktree → binding **stands down** (no garbage
  row); without a coord worktree → binding records.
- **VT — liveness probe (P2-2):** a `prunable`/deleted coord worktree entry → the guard
  treats it as absent → solo capture **records** (not suppressed forever).
- **VT — derive authoritative/self-correcting:** a binding-written garbage row for a
  funnel phase is **overwritten** by the derive's committed-ledger row (upsert).
- **VT — derive fills missing:** committed ledger with N boundaries → `prepare-review`
  populates N registry rows.
- **VT — gate primary-rooted:** run `prepare-review` from a coord-tree cwd → the gate
  reads the **primary** completed-set + registry (not the empty coord tree); a real gap
  halts (the ISS-052 regression), a complete registry passes.
- **VT — reopen eviction (P2-1):** complete a phase (row present) → reopen → row evicted
  + stamp cleared; re-complete with a fresh range → exactly one fresh row (no stale).
- **VT — mixed-mode union:** solo rows (pre-drive) + derived funnel rows → gate passes;
  a solo-during-drive phase → gate halts.
- **VT — solo stamp-present unchanged:** behaviour-preservation.
- **VT — irreducible case:** pure-solo lost stamp, no ledger → conformance `Incomplete`,
  named, `record-delta` remedy.
- **Behaviour-preservation:** `e2e_dispatch_lifecycle` (`phase/064-01`, R4/OQ-7) +
  `e2e_dispatch_sync` (incl. the :1132 double-write pin) + `set_phase_status` suites
  green.
- `just check` green, clippy plain (no `--all-targets`), per commit.

## 10. Review Notes

### Internal adversarial pass (2026-06-26)

- **F-1 (fixed in-draft):** the first draft derived from the committed coord ref, which
  ISS-039 leaves empty → would have halted every claude dispatch. (Superseded — ISS-039
  is now absorbed and the ledger is committed, so the committed-ref read is correct.)

### External pass — codex (GPT-5.5), 2026-06-26 — findings + dispositions

- **F1 (BLOCKER) root-mismatch gate → ACCEPTED.** Gate primary-rooted (D4).
- **F2 (BLOCKER) chain-fallback pollutes the range → ACCEPTED.** Chaining dropped (D1).
- **F3 (MAJOR) `read_source_deltas` order-unstable → MOOT under D1;** the gate is set-based.
- **F4 (MAJOR) derive vs `plan_phases` source divergence → CLOSED by the committed-ref
  model.** Both now read the same committed ledger via `read_ledger` (INV-4). The
  pass-1 "justified divergence" disposition is obsolete — the divergence no longer exists.
- **F5 (MAJOR) dropping the registry half is a contract break → ACCEPTED.** Double-write
  retained (D5).

### Design-conversation finding — the unsound-capture model

The arm-guard's branch-proxy is unsound under flip-from-session-root (§2): it can miss
real phases and manufacture a passing garbage row for dispatched/mixed slices. Addressed
by the sound, liveness-verified coord-worktree guard (D3/D9) + derive-upsert
self-correction (D2). This is the objective-3 core.

### External pass 2 — codex (GPT-5.5), 2026-06-26 — findings + dispositions

- **P2-1 (BLOCKER) reopen leaves a stale row → ACCEPTED, in scope (D8, §5.2).**
- **P2-2 (MAJOR) `worktree_for_ref` is not a liveness probe → ACCEPTED, in scope
  (D9, §5.2).** Liveness-verified wrapper.
- **P2-3 (BLOCKER, governance) the working-ledger read violates SPEC-022 → ACCEPTED;
  RESHAPED the funnel half.** The working-file derive is **retracted**. The funnel source
  is now the **committed** ledger; absorbing ISS-039 (committing it) is the spec-legal
  fix (D2/D7/D10).

### DECISION (User, 2026-06-26): absorb ISS-039 into scope (fork option 1)

Commit `boundaries.toml` to `dispatch/NNN` alongside `journal.toml`. Then **both** the
derive *and* `plan_phases` read the committed ref — SPEC-022-legal, F4 divergence
eliminated, claude per-phase review cuts restored. Claude-arm-bounded (codex/pi stays
IMP-171).

### Revision 3 — committed-ref model integrated (2026-06-26)

The §1–§9 body is **revised to the committed-ref model** (this body is now the single
source of truth — the working-file-read draft is fully retracted). Integrated: ISS-039
absorption (D7 splice-commit), the committed-ref derive (D2), P2-1 reopen eviction (D8),
P2-2 liveness probe (D9), SPEC-022-conformance-no-REV (D10). Seam decisions confirmed
with the User (prepare-review splice; liveness-verified probe). **Pending:** a 3rd codex
pass on this revision (the ISS-039 commit seam + P2-1/P2-2 + the derive's committed-ref
re-read), then `/inquisition` or `/plan`.
