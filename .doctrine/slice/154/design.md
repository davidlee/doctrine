# Design SL-154: Reliable conformance-registry capture

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

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

The deeper issue surfaced during design: the existing capture model is **unsound for
dispatched and mixed solo↔dispatch slices** (§2). This slice closes both leaks and
makes recording robust across landing-path transitions — registry-population only;
the conformance consumer and its algebra are untouched.

## 2. Current State

Empirically root-caused from code + topology (the SL-147/SL-153 registries on disk
were since hand-bootstrapped, so the original failing state is gone — root-cause is
from code, not live forensics).

### Solo binding (`state.rs::capture_phase_boundary`, bound to `set_phase_status`)

- `in_progress` flip → stamps `code_start_oid = HEAD` into the **phase sheet**
  (runtime) once; `completed` flip → records `(stamp, HEAD)` via
  `record_source_delta` (F-6 guard: `is_ancestor` + non-merge `end`; upsert by phase).
- An **empty range** (`start == end`) records fine. So the scope's hypothesis (a)
  ("end read early → start==end → dropped row") is **refuted**. A row is dropped
  **only when the start-stamp is absent** (state.rs:524 swallowed-warning degrade):
  the phase never entered `in_progress` under the current binding (a stale PATH
  binary flipped it; or a bootstrap slice predating the binding — SL-147's own case),
  or the runtime tier was wiped.
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
  phase against the wrong tree, producing a **garbage range**. The branch-proxy guard
  is unsound under the real flip-from-session-root workflow.
- Net: for dispatched/mixed slices the solo binding can both *miss* real phases and
  *manufacture wrong* ones. Objective 3 (mixed-mode coherence) is therefore a
  first-class target, not a footnote.

### Funnel (`dispatch.rs::run_record_boundary`)

- Registry population today rides a per-arm hand-step at funnel Record beat (router
  step 8): claude `dispatch record-boundary` writes the dispatch **ledger** *and* the
  registry (`:606` + `:614`); codex/pi `slice record-delta` writes the **registry
  only** (no ledger). No machinery beat guarantees a landed phase deposits a row;
  SL-153 reached audit empty.

### Constraints discovered

- **Audit precedes integrate.** `slice conformance` runs at audit; stage-2
  `dispatch sync --integrate` is `/close`'s job *post-audit* (dispatch router
  Conclude). So the registry must be complete by **`prepare-review`** (the mandatory
  pre-audit conclude beat), not integrate.
- **ISS-039 (out of scope).** The ledger `boundaries.toml` is written to the live
  coord worktree but **never committed** to `dispatch/NNN`; `read_ledger`
  (dispatch.rs:1991) sources from the committed ref, so it reads empty (this is why
  `plan_phases` projects 0 phase-cuts on the claude arm). A derive **must not** read
  the committed ref — it reads the **live coord worktree working file** (located via
  `git::worktree_for_ref`), valid because `prepare-review` (stage-1) runs *before*
  the worktree is removed.
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
  — the *local* tree (state.rs:743). They coincide **only on the primary tree**.
- `git::worktree_for_ref(root, refname) -> Option<PathBuf>` (git.rs:1189) — locates a
  live worktree from any tree; the coord-worktree locator for both the derive source
  and the sound guard.

## 3. Forces & Constraints

- **ADR-001 (layering):** new logic stays pure where it can; git/disk in the shell.
- **POL-002 (platform independence):** recording keys on doctrine-owned signals
  (recorded SHAs, the live coord worktree, the `dispatch/NNN` ref) — never host commit
  conventions.
- **Behaviour-preservation gate:** existing `set_phase_status` and dispatch suites
  must stay green; the solo *stamp-present* path stays byte-identical.
- **Audit-before-integrate:** enforcement point is `prepare-review`.
- **Conformance folds all paths** (no `.doctrine/` strip): only an *exact* phase
  start is sound.
- **ISS-039 (out):** derive reads the working ledger, never the committed ref.
- **Arm asymmetry (IMP-171):** the dispatch ledger is claude-only; a symmetric
  codex/pi ledger couples to `phase/<N>` projection turning on (dispatch.rs:2049,
  unconditional) — deferred.
- `record-delta` stays the manual escape hatch.

## 4. Guiding Principles

- **Each phase is recorded by the writer that holds its exact range.** Funnel phases
  → the ledger (and the derive that reads it); solo phases → the binding at the
  `completed` flip, in a true solo context.
- **One reconciliation point makes the authoritative source win.** `prepare-review`
  derive-from-working-ledger **upserts**, so it both auto-heals missing funnel rows
  *and overwrites* any garbage a mis-firing binding wrote for a dispatched phase.
- **Auto-heal where it is sound; fail loud where the data is destroyed.** Funnel rows
  are soundly recoverable retroactively (the ledger persists). A pure-solo phase's
  range exists only at flip-time; if lost there it is physically unrecoverable —
  fail closed + `record-delta`. Never manufacture a wrong row (a wrong conformance
  verdict is worse than a flagged gap).
- **Sound signals, not proxies.** The guard keys on "a live coord worktree exists for
  this slice", not the ambient branch name.
- **P as Q's foundation** (IMP-171): every seam touched is where the symmetric-derive
  follow-up extends.

## 5. Proposed Design

### 5.1 System Model

```
solo completed-flip ──(guard: no live coord worktree)──> record_source_delta ─┐
prepare-review derive ──(live coord working ledger, UPSERT, authoritative)────┤
codex/pi record-delta ────────────────────────────────────────────────────────┤
manual record-delta (escape hatch) ───────────────────────────────────────────┤
                                                                               v
                                              .doctrine/state/slice/NNN/boundaries.toml
                                                                               │
                            prepare-review GATE: registry_completeness(primary,│primary,id)
                                                                               v
                                                              Complete | HALT (named gap)
```

- **Solo binding (solo phases).** Keep the stamp; record `(stamp, HEAD)` at
  `completed`. **Guard:** skip iff a live coord worktree exists for `dispatch/NNN`
  (the slice is under active dispatch → the funnel/derive owns recording). Stamp
  absent → no row + a surfaced warning; the gate / conformance fail-closed catches it.
- **Derive-at-gate (funnel phases), authoritative + self-correcting.** At
  `prepare-review`, read the live coord worktree working ledger and
  `record_source_delta` each row (upsert) — overwrites any binding mis-capture, fills
  any missing funnel row. ISS-039-independent (working file, stage-1 only).
- **Gate (both arms).** `registry_completeness` resolved against the **primary** tree
  for *both* the completed-set and the registry; `bail!` on any gap.
- **Funnel inline double-write retained.** `run_record_boundary` is **unchanged**
  (ledger + registry); the derive is a redundant-but-authoritative reconciler over it
  (no contract break — codex F5). For a row the inline write missed/errored, the
  derive recovers it; for a row it wrote, the derive upserts the identical value.

### 5.2 Interfaces & Contracts

**Solo binding** (`capture_phase_boundary`, state.rs): two changes only.
1. **Guard predicate** branch-proxy → coord-worktree presence:
   ```rust
   // was: current_branch(project_root) == format!("dispatch/{slice_id:03}")
   // now: a live coordination worktree for this slice owns recording.
   match crate::git::worktree_for_ref(project_root, &format!("refs/heads/dispatch/{slice_id:03}")) {
       Ok(Some(_)) => return None,       // under active dispatch — funnel/derive owns it
       Ok(None)    => {}                  // solo context — record
       Err(e)      => { warn_capture(phase_id, &format!("coord probe failed: {e}")); return None; }
   }
   ```
   Sound from any tree (works when the flip runs from the session root). No
   `resolve_phase_start` / chain helper — chaining is unsound (D1), so the absent-stamp
   branch records **nothing** (surfaced warning), unchanged in its non-blocking posture.
2. Stamp-present path **unchanged** (precise `(stamp, HEAD)`).

**Funnel** — `run_prepare_review` (dispatch.rs) gains, after phase planning:
```text
let primary = git::primary_worktree(root)?;          // F1 fix: one canonical tree
// derive (claude): locate live coord worktree, read its WORKING ledger, upsert each row
if let Some(coord) = git::worktree_for_ref(root, "refs/heads/dispatch/NNN")? {
    for row in ledger::read_boundaries_in_worktree(&coord, slice)? {   // NEW pub reader (OQ-4)
        state::record_source_delta(&primary, slice, row)?;             // upsert; authoritative
    }
}
// gate (both arms): primary-rooted completeness
match state::registry_completeness(&primary, &primary, slice)? {
    Complete            => {}
    Incomplete { gaps } => bail!("prepare-review: conformance registry incomplete: {gaps};\
                                  record-delta the missing phase(s) before audit"),
}
```

`run_record_boundary`: **unchanged** (double-write retained). `record-delta` verb:
**unchanged** (escape hatch). New: `ledger::read_boundaries_in_worktree(worktree_root,
slice)` (a `pub(crate)` reader over the worktree-relative `dispatch_dir` path — OQ-4,
DRY over rebuilding the string in `dispatch.rs`).

### 5.3 Data, State & Ownership

- **Registry** `.doctrine/state/slice/NNN/boundaries.toml` (runtime, primary-resolved):
  shape unchanged. Writers: solo binding, `prepare-review` derive (new), funnel inline
  write, codex/pi + manual `record-delta`. All via `record_source_delta` (upsert by
  phase → idempotent; the derive is the authoritative last writer for funnel phases).
- **Ledger** `.doctrine/dispatch/NNN/boundaries.toml` (claude-only): unchanged writer
  (`run_record_boundary`); now *also* the derive's read source — from the **live coord
  worktree working file**, never the committed ref (empty under ISS-039).
- **Phase sheet** `.../NNN/phases/phase-NN.toml`: still carries the stamp (precision
  input). No relocation.

### 5.4 Lifecycle, Operations & Dynamics — landing-path transitions

| Slice shape | Solo phases | Funnel phases | Reconciliation |
|---|---|---|---|
| Pure solo | binding records (no coord worktree → guard never fires) | — | conformance `check_completeness` (final net) |
| Pure dispatch | binding stands down (coord worktree present) | inline write + derive (authoritative) | prepare-review gate |
| **Solo→dispatch (SL-153)** | binding records **before** the drive (solo context) | inline + derive from ledger | gate checks the **union** |
| Dispatch→solo | inline + derive | binding records **after** conclude (coord gone) | conformance final net (post-gate solo work) |
| Interleaved | binding per solo phase; derive **upserts/corrects** any funnel phase the binding mis-grabbed | inline + derive | gate + conformance |

Load-bearing mechanism: **derive-at-gate with upsert** is both the funnel auto-heal
and the corrector for cross-context mis-captures — a transition cannot leave a wrong
or missing *funnel* row. A solo phase completed *during* an active drive (coord
worktree present) is the one crack: the binding stands down and the funnel does not
own it → no row → the gate halts loudly → `record-delta`. Rare, loud, recoverable.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1:** by audit, every completed phase has exactly one registry row (binding for
  solo; inline+derive for funnel; gate enforces; conformance is the pure-solo net).
- **INV-2:** `record_source_delta` upsert ⇒ no duplicates across writers; the derive is
  the authoritative last write for funnel phases.
- **INV-3:** the stamp, when present, is authoritative for solo start (precision).
- **Irreducible manual case:** a **pure-solo** phase whose flip-time capture was lost
  (stale binary / wiped runtime tier) and that has **no ledger row** — its range is
  physically destroyed; no sound retroactive reconstruction exists (chaining folds in
  inter-phase commits — D1). Fails loud at the gate / conformance + `record-delta`.
  **Accepted floor.**
- **Empty-code phase:** `start == end` records (unchanged); satisfies the gate.
- **Derive idempotency:** re-running `prepare-review` re-derives (upsert) — safe.
- **Coord worktree absent at prepare-review** (removed early, against convention):
  derive no-ops; the gate halts on any resulting gap.

## 6. Open Questions & Unknowns

- **OQ-1 (resolved → D1):** chain-fallback for a lost solo stamp? **No** — unsound
  (conformance folds all paths). Drop; fail loud.
- **OQ-2 (resolved → D5):** drop `run_record_boundary`'s registry half? **No, keep** —
  the derive makes it redundant-but-harmless and the drop is a contract break (codex
  F5). Keeping it is also more robust (two independent attempts).
- **OQ-3:** any consumer that reads the registry *mid-drive*, before `prepare-review`?
  (Audit at impl: conformance runs at audit, post-`prepare-review`; no mid-drive
  reader found.)
- **OQ-4:** expose `ledger::read_boundaries_in_worktree` (DRY) vs rebuild the path in
  `dispatch.rs`. Lean: the reader. `dispatch_dir` is private to `ledger.rs` (:375).
- **OQ-5:** does the guard's coord-worktree probe add meaningful cost to every status
  flip? It is one `git worktree list`. Acceptable; only the dispatch case returns Some.

## 7. Decisions, Rationale & Alternatives

- **D1 — Drop chaining; no `resolve_phase_start`.** Conformance folds every path with
  no `.doctrine/` strip (slice.rs:1919), so `start = prev.end` mis-attributes
  inter-phase commits → false `undeclared`. A wrong row is worse than a gap. The stamp
  is the only sound exact start; absent it, record nothing and let the gate/conformance
  fail closed.
- **D2 — Derive-at-gate, authoritative + self-correcting (upsert).** The funnel
  auto-heal *and* the corrector for the unsound-capture finding (§2). Reads the working
  ledger (ISS-039-independent), stage-1 only.
- **D3 — Sound guard: live coord worktree, not branch-proxy.** The branch-proxy fails
  under flip-from-session-root (§2). "A live coord worktree exists for `dispatch/NNN`"
  is a doctrine-owned, tree-independent signal for "the funnel owns recording".
- **D4 — Primary-rooted gate.** `registry_completeness(primary, primary, id)` so the
  completed-set and the registry are read from the same canonical tree (codex F1).
- **D5 — Keep the funnel inline double-write** (reverses the earlier drop). Redundant
  under the derive, but no contract break (codex F5) and more robust.
- **D6 — Defer codex/pi symmetric derive (IMP-171) and ISS-039 (RFC-005 H3).** The
  reproduction is claude-arm; codex/pi ledger couples to untested phase-ref projection.
- **Alternative rejected — chain-fallback (B):** unsound (D1). **Read-time fallback in
  conformance:** scope-rejected; papers over the write gap; ledger is claude-only.

## 8. Risks & Mitigations

- **R1 — derive reads a stale/absent working ledger.** Coord worktree gone early, or a
  funnel phase the inline write never recorded to the ledger → that row can't be
  derived → the **gate** halts loudly with the named gap. Fail-closed, never silent.
- **R2 — guard false-stand-down.** A genuinely-solo phase completed during an active
  drive (coord worktree present) is skipped by the binding and not owned by the funnel
  → the gate catches it (§5.4 crack). Loud + `record-delta`. The inverse (binding
  mis-captures a funnel phase) is corrected by the derive upsert.
- **R3 — gate false-halt.** `registry_completeness` keys on completed `PHASE-NN`; a
  blocked/not-completed phase is excluded. Mitigation: same completed-set source as
  conformance (parity); message names exact gaps + remedy.
- **R4 — behaviour regression in the binding.** Only the guard predicate + the
  absent-stamp branch change; stamp-present path byte-identical. Mitigation: existing
  binding suite green; add the unsound-capture regression tests (below).
- **R5 — guard probe cost / failure.** `worktree_for_ref` per flip; a probe *error*
  stands the binding down with a surfaced warning (conservative — the gate still nets).

## 9. Quality Engineering & Validation

- **Pure unit:** `check_completeness` (reuse). (No `resolve_phase_start` to test — D1.)
- **VT — guard soundness (the unsound-capture fix):** a dispatched phase flipped from a
  session-root context with a live coord worktree → binding **stands down** (no garbage
  row); without a coord worktree → binding records.
- **VT — derive authoritative/self-correcting:** a binding-written garbage row for a
  funnel phase is **overwritten** by the derive's ledger row (upsert).
- **VT — derive fills missing:** ledger with N boundaries → `prepare-review` populates
  N registry rows.
- **VT — gate primary-rooted:** run `prepare-review` from a coord-tree cwd → the gate
  reads the **primary** completed-set + registry (not the empty coord tree); a real gap
  halts (the ISS-052 regression), a complete registry passes.
- **VT — mixed-mode union:** solo rows (pre-drive) + derived funnel rows → gate passes;
  a solo-during-drive phase → gate halts.
- **VT — solo stamp-present unchanged:** behaviour-preservation.
- **VT — irreducible case:** pure-solo lost stamp, no ledger → conformance `Incomplete`,
  named, `record-delta` remedy.
- **Behaviour-preservation:** `e2e_dispatch_sync` (incl. the :1132 double-write pin) +
  `set_phase_status` suites green unchanged.
- `just check` green, clippy plain (no `--all-targets`), per commit.

## 10. Review Notes

### Internal adversarial pass (2026-06-26)

- **F-1 (fixed in-draft):** the first draft derived from the committed coord ref, which
  ISS-039 leaves empty → would have halted every claude dispatch. Fixed: derive from the
  live coord worktree working file.

### External pass — codex (GPT-5.5), 2026-06-26 — findings + dispositions

- **F1 (BLOCKER) root-mismatch gate → ACCEPTED.** Gate now primary-rooted (D4, §5.2).
- **F2 (BLOCKER) chain-fallback pollutes the range → ACCEPTED.** Chaining dropped; no
  `resolve_phase_start`; solo lost-stamp fails loud (D1). Verified conformance folds all
  paths (slice.rs:1919).
- **F3 (MAJOR) `read_source_deltas` order-unstable → MOOT under D1** (no prev lookup);
  the gate is set-based, order-independent.
- **F4 (MAJOR) derive vs `plan_phases` source divergence → ACCEPTED as justified.**
  Derive is stage-1-only (working ledger sound); `plan_phases` spans stage-1+2 (needs
  the committed ref). Proper unification = fix ISS-039 (out of scope); documented.
- **F5 (MAJOR) dropping the registry half is a contract break → ACCEPTED.** Double-write
  retained (D5); derive is the authoritative reconciler over it.

### Design-conversation finding (beyond codex) — the unsound-capture model

The arm-guard's branch-proxy is unsound under flip-from-session-root (§2): it can miss
real phases and manufacture wrong ones for dispatched/mixed slices. Addressed by the
sound coord-worktree guard (D3) + derive-upsert self-correction (D2). This is the
objective-3 core, surfaced by the user's solo↔dispatch transition concern.

### External pass 2 — codex (GPT-5.5), 2026-06-26 — findings + dispositions

Pass-1 status (codex's own check): F1 ✓, F2 ✓, F3 moot, F5 ✓; **F4 only partial —
graduated into the spec conflict (P2-3 below).**

- **P2-1 (BLOCKER) reopen leaves a stale row the gate blesses → ACCEPTED, in scope.**
  The reopen path (state.rs:386–400) clears `completed`/`started` but **not**
  `code_start_oid`; capture keeps the original stamp on re-entry (state.rs:503);
  `registry_completeness` checks *presence*, not *range freshness*. A phase reopened
  after a transition keeps its old row; guard stands down; derive has no ledger row to
  overwrite → silent garbage conformance. **Fix:** on reopen (completed→non-completed),
  **evict that phase's registry row + clear its stamp** so the redo re-captures fresh
  (or fails loud). Solo-side; independent of the funnel decision.
- **P2-2 (MAJOR) `worktree_for_ref` is not a liveness probe → ACCEPTED, in scope.**
  `parse_worktree_for_ref` (git.rs:1163) ignores `prunable` and never stats the path,
  so a deleted/failed-cleanup coord entry suppresses solo capture **forever** (POL-002
  footgun). **Fix:** a liveness-verified probe (reject `prunable`, stat the path) **or**
  a doctrine-owned "dispatch-active" runtime marker. Prefer the marker if one already
  exists; else the verified probe.
- **P2-3 (BLOCKER, governance) the working-ledger read violates SPEC-022 → ACCEPTED;
  RESHAPES the funnel half.** SPEC-022 (spec-022.md:180; a named responsibility in
  spec-022.toml) requires the run ledger — *including* `boundaries.toml` — be tree-read
  from the `dispatch/<N>` tip via `read_path_at`, **never the working filesystem**,
  identically stage-1 and stage-2. The D2/§5.2 working-file read is a direct violation
  and is **RETRACTED**. There is no spec-legal per-phase source at prepare-review unless
  the boundaries ledger is **committed** to `dispatch/NNN` (the journal is; boundaries
  aren't — that *is* ISS-039). So ISS-052's clean fix is blocked on ISS-039.

### DECISION (User, 2026-06-26): absorb ISS-039 into scope (fork option 1)

Commit `boundaries.toml` to `dispatch/NNN` alongside `journal.toml` (ISS-039's
documented fix direction). Then **both** the derive *and* `plan_phases` read the
**committed ref** — SPEC-022-legal, F4 divergence eliminated, and claude per-phase
review cuts (currently 0 from this bug) are restored as a bonus. Bounded to the claude
arm; does NOT touch the codex/pi phase-ref coupling (IMP-171). The §2/§5 design body
below still describes the RETRACTED working-file read and **must be revised** by the
next agent to the committed-ref model — see `handover.md` and `notes.md`.

<!-- §2/§5 body PENDING REVISION to the committed-ref (ISS-039-absorbed) model.
     §10 supersedes the body where they conflict until then. -->

