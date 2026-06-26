# Design SL-154: Reliable conformance-registry capture

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->
<!-- Body revised 2026-06-26 to the COMMITTED-REF model (ISS-039 absorbed). The
     prior working-file-read draft (codex pass-2 P2-3, SPEC-022 violation) is
     fully retracted; this body is the single source of truth. §10 is the review
     ledger (history of all passes + the decision trail). -->
<!-- Revision 4 (2026-06-26): pass-5 reshape. D11 reshaped from the empty-only
     projection-source guard (proven unsound — pass-5) to a PER-PHASE PROVENANCE
     set-check; `BoundaryRow` gains a `provenance` field (D11/§5.2/§5.3/§7).
     `git::code_delta_paths` is dropped. New §5.6 records the registry's nav/value
     role (efficiency lens). F4 explicitly NOT closed by provenance. -->

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

Every registry writer stamps the row's **`provenance`** (`Solo` | `Funnel` |
`Manual`; absent → `Unknown`). It is the discriminator D11 needs and nothing else
records (§5.6): the committed ledger holds *only* funnel phases, so "registry phase
absent from the committed ledger" is ambiguous (legit solo vs lost funnel row)
**without** provenance.

```
solo completed-flip ──(guard: no LIVE coord worktree)──> record_source_delta[Solo] ─┐
prepare-review:                                                                      │
  1. commit-boundaries: splice working ledger → dispatch/NNN tip (SPEC-022)         │
  2. PROJECTION-SOURCE GUARD (D11): every registry[Funnel|Unknown] phase ∈ committed ──│
  3. derive: read_ledger(committed) → record_source_delta[Funnel] per row (UPSERT) ─┤
funnel double-write (run_record_boundary) ─> record_source_delta[Funnel] ───────────┤
codex/pi + manual record-delta ─> record_source_delta[Manual: preserve existing] ───┤
                                                                                    v
                                                  .doctrine/state/slice/NNN/boundaries.toml
                                                                                    │
                              prepare-review GATE: registry_completeness(primary,│ primary,id)
                                                                                    v
                                                              Complete | HALT (named gap)
```

D11 (step 2) runs **before the ref projection** (step 6) — the load-bearing ordering,
so a halt creates no refs (clean `record-delta` → re-run, F1). The registry read is
taken pre-derive for clarity; the derive cannot mask the gap (it only upserts rows that
*are* in the committed ledger, never removes a `Funnel` registry row absent from it —
pass-6 MINOR). D11's expected-in-ledger set is provenance **∈ {Funnel, Unknown}**:
positively-`Solo` and `Manual` rows are excluded; `Unknown` (legacy, unclassifiable) is
included so a mid-upgrade active slice halts loudly rather than silently under-projecting.

Reopen (completed→non-completed) **evicts** the phase's registry row + clears its
stamp (P2-1), so a redo re-captures fresh or fails loud — never a stale blessed row.

- **ISS-039 commit (funnel prerequisite).** At `prepare-review`, before any read, splice
  the live coord worktree's working `boundaries.toml` onto the `dispatch/NNN` tip via a
  doctrine-mediated commit (mirrors `commit_journal`). Now `read_ledger`, `plan_phases`,
  and the derive read **one** committed, checkout-independent source (SPEC-022-legal).
- **Solo binding (solo phases).** Keep the stamp; record `(stamp, HEAD)` at `completed`
  with **`provenance = Solo`**. **Guard:** skip iff a *live* coord worktree exists for
  `dispatch/NNN` (the slice is under active dispatch → the funnel/derive owns recording).
  Stamp absent → no row + a surfaced warning; the gate / conformance fail-closed catches it.
- **Projection-source guard (D11, funnel phases).** Before the ref projection, assert
  every registry row whose `provenance ∈ {Funnel, Unknown}` has a committed-ledger row;
  halt naming the missing phases otherwise. Provenance is the discriminator — a registry
  phase absent from the committed ledger is a legit solo phase OR a lost funnel row, and
  only provenance tells them apart (§5.6). `Unknown` (legacy) is included so a mid-upgrade
  active slice halts loudly (pass-6); `Solo`/`Manual` excluded. Subsumes the old
  empty-only guard.
- **Derive-at-gate (funnel phases), authoritative + self-correcting.** At
  `prepare-review`, read the **committed** ledger and `record_source_delta` each row
  (upsert, **`provenance = Funnel`**) — overwrites any binding mis-capture, fills any
  missing funnel row.
- **Gate (both arms).** `registry_completeness` resolved against the **primary** tree
  for *both* the completed-set and the registry; `bail!` on any gap.
- **Funnel inline double-write retained.** `run_record_boundary` is **unchanged**
  (ledger working file + registry); the derive is a redundant-but-authoritative
  reconciler over it (no contract break — codex F5).

### 5.2 Interfaces & Contracts

**`prepare_review` (dispatch.rs:1497)** — the commit + derive + gate run **BEFORE the
ref projection** (the ordering is load-bearing — codex pass-3 F1; see below):

```text
let coord_ref = "refs/heads/dispatch/NNN";
let tip0 = resolve_commit(root, coord_ref)?;                 // existing

// (1) ISS-039 commit — land the working boundaries ledger onto the tip (D7).
//     CONTENT-IDEMPOTENT: parse+validate working bytes, re-serialize canonical,
//     and commit ONLY when it differs from the committed blob — a re-run with
//     identical content does NOT advance the ref (F1). No-op when no live coord
//     worktree / no working file (re-run after removal, early removal).
let tip = match git::live_worktree_for_ref(root, coord_ref)? {        // D9 liveness wrapper
    Some(coord) => commit_boundaries(root, &tip0, coord_ref, &coord, slice)?,  // tip0 or new tip
    None => tip0,
};
let tip_tree = tree_of(root, &tip)?;
let trunk_base = merge_base(root, &tip, &trunk_tip)?;        // recomputed off the (maybe new) tip

// (2) read the committed ledger (now populated) — the SINGLE source (INV-4).
let boundaries = read_ledger::<Boundaries>(root, coord_ref, slice3, "boundaries.toml")?;

// (3) PROJECTION-SOURCE GUARD (D11, reshaped pass-5; refined pass-6). The committed
//     ledger holds ONLY funnel phases. A funnel phase whose committed-ledger row was
//     LOST (coord worktree removed before prepare-review; partial working ledger)
//     under-projects silently: plan_phases emits no cut for it, yet the funnel double-
//     write (run_record_boundary:614) already wrote the REGISTRY row, so
//     registry_completeness passes. PROVENANCE discriminates: every registry row that is
//     NOT positively solo/manual (Funnel, or Unknown legacy we cannot clear) must have a
//     committed-ledger row. Read pre-derive (the derive can't mask it — pass-6 MINOR);
//     the load-bearing ordering is BEFORE projection (step 6).
let primary = git::primary_worktree(root)?;
let registry = state::read_source_deltas(&primary, slice)?;
let committed: HashSet<&str> =
    boundaries.rows.iter().map(|r| r.phase.as_str()).collect();
let missing: Vec<&str> = registry.rows.iter()
    .filter(|r| matches!(r.provenance, Provenance::Funnel | Provenance::Unknown))
    .map(|r| r.phase.as_str())
    .filter(|p| !committed.contains(p))
    .collect();
if !missing.is_empty() {
    bail!("prepare-review: committed boundaries ledger is missing phase(s) {missing:?} on \
           dispatch/{slice3} that the registry records as funnel-owned (or legacy/unclassified). \
           The registry has them but the dispatch ref does not — the coordination worktree was \
           likely removed before prepare-review, or these are pre-provenance rows. Re-run with \
           the coord worktree present (it persists until integrate), or record-delta + COMMIT \
           the ledger for the named phase(s).");
}

// (4) DERIVE the registry + (5) GATE — BEFORE projection, so a halt leaves NO
//     refs to collide with on the operator's record-delta → re-run (F1).
for row in &boundaries.rows { state::record_source_delta(&primary, slice, row.clone())?; }
if let Incomplete { gaps } = state::registry_completeness(&primary, &primary, slice)? {
    bail!("prepare-review: conformance registry incomplete: {gaps}; \
           record-delta the missing phase(s) before audit");   // halt: no refs created yet
}

// (6) existing projection — plan_review + plan_phases(boundaries) +
//     with_journaled_projection (journal.toml splice, :2182). Unchanged.
```

The guard reuses `state::read_source_deltas` (state.rs:588) and `Provenance` on the
boundary row — **no code-delta diff, no exclusion set**. The pass-5 reshape *deletes*
the old `git::code_delta_paths` helper and its `trunk_base..tip` filter: the invariant is
now "**every registry phase that is not positively solo/manual has a committed-ledger
row**", a pure phase-id set comparison that never touches the working tree or duplicates
`plan_review`'s filter (the three pass-5 defects of the empty-only guard — too-weak
predicate, false-halt on solo/empty-code, wrong-exclusion-set — all dissolve).

Provenance write/expectation rules (pass-6):
- `Funnel` — funnel double-write (`run_record_boundary`) + the derive. **Expected** in
  the committed ledger.
- `Solo` — the binding. **Excluded** (solo phases legitimately have no committed-ledger row).
- `Manual` — `record-delta`, but **only on a fresh row with no prior landing capture**;
  `record-delta` **preserves** an existing `Funnel`/`Solo` provenance (it corrects the
  *range*, not the landing path). **Excluded** — a genuine `Manual` row is the irreducible
  hand-bootstrap floor (D1), with no funnel assertion that a ledger row should exist. The
  preserve rule is what stops `record-delta` from blinding D11 (pass-6 BLOCKER): a lost
  funnel row stays `Funnel`, so D11 keeps halting until the operator commits the ledger.
- `Unknown` — absent field (legacy / pre-provenance). **Expected** (included), so a
  mid-upgrade active slice with an unclassified funnel row halts loudly instead of
  silently under-projecting (pass-6 MAJOR). Closed slices never re-enter prepare-review,
  so their `Unknown` rows are inert.

New `commit_boundaries(root, parent, coord_ref, coord_worktree, slice) -> Result<String>`
— the boundaries twin of `commit_journal`, with two hardenings over a naive splice:
1. **Validate before commit (F3).** Read the working file via
   `ledger::read_boundaries_file` (below), `toml::from_str::<Boundaries>` it. A parse
   error is a clean `Err` (the working ledger is malformed) — never commit garbage onto
   the tip, unlike a verbatim-byte splice.
2. **Content-idempotent via TREE-oid compare (F1; pass-4 MINOR).** Re-serialize the
   parsed `Boundaries` to canonical TOML and splice: `tree_with_file(tip_tree, path,
   canonical)` → candidate tree oid. If it **equals the current `tip_tree`** → return
   `parent` unchanged (no commit, no ref advance). This sidesteps any
   committed-blob-canonicalization assumption (git dedups identical content to the same
   blob, hence the same tree) — a raw-blob compare would falsely diff on formatting.
   Differ → `commit_tree(parent, "ledger: boundaries")` → `update_ref_cas(coord_ref,
   commit, parent)` (a moved ref bails like `commit_journal`, R6). Returns the new tip.

(DRY — OQ-6: factor a shared `splice_ledger_file(...)` that `commit_journal` and
`commit_boundaries` both call; decide at /plan if it reads cleanly.)

New `ledger::read_boundaries_file(worktree_root, slice) -> Result<Option<String>>`: the
raw working-file reader over the worktree-relative `dispatch_dir` path (`dispatch_dir`
is private to ledger.rs, :375 — expose this reader rather than rebuild the string in
dispatch.rs; OQ-4). `None` when the file is absent (→ `commit_boundaries` no-ops).

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
2. Stamp-present path: precise `(stamp, HEAD)`, now with **`provenance = Solo`** on the
   recorded `BoundaryRow` (the only delta to this path — D11 reads it to exclude solo
   phases from the funnel-expected set).

`git::live_worktree_for_ref` is **not** a wrapper over `worktree_for_ref` — the existing
`parse_worktree_for_ref` (git.rs:1163) returns the path on the matching `branch` line and
**discards the block's `prunable` marker**, so liveness is unreachable from the outside
(F2). D9 therefore **extends the parser** to surface the full block
(`{ path, branch, prunable }`) and the liveness helper applies `!prunable &&
path.exists()`. `worktree_for_ref` keeps its current signature for its existing callers
(behaviour-preservation); the new helper is the liveness-aware sibling.

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

**Funnel** — `run_record_boundary` (dispatch.rs:587): double-write retained; the only
change is it stamps **`provenance = Funnel`** on the `BoundaryRow` it writes to *both*
the committed working ledger and the registry.

**Provenance merge lives in the state-layer writer, not the caller (pass-7 MAJOR fix).**
`state::record_source_delta` (state.rs:613) already does a single read-modify-write upsert;
the provenance policy rides *inside* it, keyed on the **incoming** row's provenance — so
the merge is atomic with the write (no caller pre-read, no race a concurrent funnel/derive
write could lose):

```text
// inside record_source_delta's RMW, when a row for `phase` already exists:
match incoming.provenance {
    Solo | Funnel => existing.provenance = incoming.provenance,  // landing writers are authoritative
    Manual        => { /* keep existing.provenance — a correction never reclassifies */ }
}
// no existing row: store incoming.provenance as-is (Manual for a fresh record-delta row).
```
(Code never *writes* `Unknown`; it only *reads* it from a legacy file. So a `Manual`
incoming over an existing `Unknown` row **keeps `Unknown`**.)

**`record-delta`** (`run_record_delta`, slice.rs:1970) therefore just constructs the row
with `provenance = Manual` and calls `record_source_delta`; the seam preserves any existing
`Solo`/`Funnel`/`Unknown`. Consequences: a lost funnel row stays `Funnel` → D11 keeps
halting until the ledger is committed (bare `record-delta` **cannot** clear a funnel gap);
a legacy `Unknown` row stays `Unknown` → still halts (bare `record-delta` cannot clear it
either — see §5.5 for the reclassification path). Only a phase with **no prior row** lands
`Manual`. `slice.rs` (sets `Manual`) + `state.rs` (the merge) are both design-target.

### 5.3 Data, State & Ownership

- **`BoundaryRow` (`boundary.rs:16`) gains one field — `provenance`** (the pass-5
  reshape; D11):
  ```rust
  #[derive(…, Default)]
  #[serde(rename_all = "snake_case")]
  pub(crate) enum Provenance {
      Solo,             // solo binding at the completed flip — D11-EXCLUDED
      Funnel,           // run_record_boundary double-write + the prepare-review derive — D11-EXPECTED
      Manual,           // record-delta incoming; merge preserves any existing (incl Unknown) — D11-EXCLUDED
      #[default] Unknown,   // pre-provenance row (absent in TOML) — D11-EXPECTED (loud on active slices)
  }
  // on BoundaryRow:
  #[serde(default)] pub provenance: Provenance,
  ```
  `#[serde(default)]` is the **entire** back-compat story (User decision): legacy
  on-disk rows (147/153, any mid-flight committed ledger) parse as `Unknown`. Provenance
  has exactly one consumer — D11 at prepare-review — which **closed slices never
  re-enter**, so their `Unknown` rows are inert. An **active** slice crossing the upgrade
  is the only exposed case: D11 treats `Unknown` as **expected** (§5.2), so it halts
  loudly rather than silently under-projecting; the operator hand-fixes once (re-record
  boundaries → `Funnel`, or `record-delta`). **No backfill, no migration code** — the
  guard makes the need loud, the fix is manual (User decision: a one-time hand-fix beats
  ongoing legacy machinery). The committed-ledger rows carry `Funnel` by construction
  (only the funnel writes them); D11 reads provenance from the *registry*, not the
  committed ledger.
- **Registry** `.doctrine/state/slice/NNN/boundaries.toml` (runtime, primary-resolved).
  Writers stamp provenance: solo binding → `Solo`; `prepare-review` derive (new) +
  funnel inline write → `Funnel`; codex/pi + manual `record-delta` → `Manual` *as the
  incoming value*, but the **state-seam merge** (§5.2) preserves any existing
  `Solo`/`Funnel`/`Unknown`, so a correction only lands `Manual` on a brand-new row.
  Eviction: reopen (new). All mutations via `record_source_delta` (upsert) /
  `forget_source_delta` (remove) against the primary tree → idempotent; the derive is the
  authoritative last writer for funnel phases. Provenance is **sticky** (pass-6/7): only a
  landing writer (binding/funnel/derive, incoming `Solo`/`Funnel`) sets/overwrites it; a
  correction (incoming `Manual`) never reclassifies an existing row. The merge is atomic
  inside `record_source_delta`'s read-modify-write — not a caller pre-read (race-free).
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
- **Re-run semantics (F1, sharpened).** `prepare-review` is **not** freely re-runnable
  for *projection* — once `review/*`/`phase/*` refs exist, the existing zero-oid CAS
  reports them stale and bails (dispatch.rs:1564/1592), and the recovery `commit_journal`
  records Failed rows (e2e_dispatch_sync.rs:435). This slice must not **worsen** that:
  `commit_boundaries` is content-idempotent (identical ledger → no commit, no ref
  advance), so a re-run does not poison the journal via a fresh boundaries commit. And
  because the **derive + gate run before projection**, the operator's intended re-run —
  *gate halts → `record-delta` the gap → re-run* — creates no projection refs on the
  halted first pass, so the re-run reaches projection cleanly with the registry now
  complete. (Re-running a *successful* prepare-review still hits the pre-existing
  stale-ref bail — unchanged, out of scope.)
- **Coord worktree absent / stale / partial at prepare-review (D11):** commit-boundaries
  no-ops or commits a partial ledger. The funnel double-write (run_record_boundary:614)
  has independently filled the **registry** (`provenance = Funnel`) for every recorded
  phase, so `registry_completeness` would FALSELY pass while `plan_phases` under-projects.
  The **projection-source guard (D11)** catches it per-phase: any registry `Funnel` row
  with no committed-ledger row ⇒ halt, naming the phase(s). This covers **total** loss
  (committed ledger empty, all funnel rows missing) *and* **partial** loss (some funnel
  rows missing) — the pass-5 hole the old empty-only predicate left open.
- **Manual escape-hatch — D11 enforces "commit the ledger" (pass-6 BLOCKER fix):**
  `record-delta` writes the registry but **not** the committed ledger. The earlier draft
  let it stamp `Manual` unconditionally, so an operator could clear a D11 halt with
  `record-delta` while projection stayed empty — a silent under-projection (pass-6).
  **Closed:** `record-delta` is **provenance-preserving** — a phase that already has a
  `Funnel` row keeps `Funnel`, so D11 **re-halts** on the next run until the operator
  actually commits the ledger (re-run with the coord worktree present). A `Manual` row
  therefore only arises for a phase with **no prior landing capture** — the irreducible
  hand-bootstrap floor (D1, a pure-solo phase whose flip-time range was destroyed) — for
  which there is no funnel assertion that a committed-ledger row should exist; D11
  excluding it is correct, not a hole. Note this means a funnel phase whose
  `run_record_boundary` **never fired at all** (no registry row → D4 halts → operator
  `record-delta`s a fresh `Manual` row) gets its *conformance* repaired but not a
  *review-cut* — the missing cut is a funnel-drive defect (the funnel never recorded the
  phase), outside D11's remit and the accepted floor, not a regression this guard introduces.
- **Mixed solo→dispatch (the pass-5 false-halt):** solo phases land code on the dispatch
  branch with **no** committed-ledger row by design; their registry rows are `Solo` and
  D11 excludes them. An all-doc / empty-code *dispatch* phase is recorded by the funnel
  (`start == end`, a present row) into both ledgers → present in the committed set → no
  halt. D11 never inspects code paths, so neither shape false-halts (the two pass-5
  false-halt cases).
- **Legacy / mid-upgrade active slice (pass-6 MAJOR; pass-7 BLOCKER):** rows written before
  the `provenance` field read `Unknown`. For a **closed** slice this is inert (no re-entry
  to prepare-review). For an **active** slice crossing the upgrade, an `Unknown` row absent
  from the committed ledger is unclassifiable — possibly a lost funnel row — so D11
  **includes** `Unknown` in the expected set and **halts loudly**, naming the phase.
  **Clearing the halt requires reclassification, not a bare `record-delta`** (which
  preserves `Unknown` — §5.2, so it cannot clear it and *cannot* silently downgrade it to
  an excluded `Manual`): the operator either **re-records through the landing path** (the
  funnel `record-boundary` → `Funnel` + commits the ledger) or **hand-edits the runtime
  registry row's `provenance`** to the truth (`Solo` if it was solo, `Funnel` + commit the
  ledger if funnel). A legacy `Unknown` that *is* solo false-halts until reclassified —
  the accepted, bounded, **loud** cost of never silently under-projecting. Hand-editing a
  runtime/disposable file is exactly the User's "hand-fix > legacy machinery" decision; no
  new reclassify verb is built.

### 5.6 Value: registry as the authoritative, self-describing navigation surface

The correctness work has an ergonomic dividend worth claiming as SL-154 value (User-
requested efficiency/workflow lens):

- **Nav-coherence.** Pre-slice, the registry and the dispatch ledger could diverge (the
  registry hand-bootstrapped, the ledger uncommitted/empty — the ISS-039/ISS-052 split).
  The committed-ref derive-upsert makes the registry the **one authoritative, populated**
  per-phase record — a single place an agent or `slice conformance` reads, instead of
  reconciling two ledgers by hand.
- **Self-describing provenance.** The `provenance` field added for D11 also tags each
  phase with *where it landed* — `Funnel` phases route an agent to `dispatch/NNN` /
  `review/NNN-NN`; `Solo` to `edge`. The field earns its keep twice: soundness gate +
  navigation signal.
- **Storage rule holds — defer the derived view.** The registry stores OIDs (the source),
  not a path list; a nav consumer re-diffs. Fine for the one current consumer. A *derived*
  per-phase file-set view (`slice show --phase-files`, or a gitignored cache) is a
  **follow-up feature gated on SL-154's reliable population** — backlogged, not scoped
  here (no premature derived tier).
- **Halt-message ergonomics.** The D11 (and gate) bail name the bounded missing **phase
  ids**, not a count — the operator/agent goes straight to the fix. This falls out of the
  per-phase set check for free.

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
  **Two hardenings (codex pass-3): (a)** parse+validate the working ledger before the
  commit — never land malformed TOML on the tip (F3); **(b)** content-idempotent — commit
  only when the canonical re-serialization differs from the committed blob, so a re-run
  doesn't advance the ref and poison the journal (F1). **Ordering:** derive + gate run
  **before** the ref projection, so a gate-halt creates no refs (clean record-delta →
  re-run; F1).
- **D8 — Reopen evicts the row + clears the stamp (P2-1).** A completed→non-completed
  transition removes the phase's registry row (`forget_source_delta`) and clears
  `code_start_oid`, so a redo re-captures fresh or fails loud — never a stale blessed row.
- **D9 — Liveness-verified coord probe (P2-2).** New `git::live_worktree_for_ref`. **Not a
  wrapper** — `parse_worktree_for_ref` (git.rs:1163) discards the block's `prunable`
  marker, so liveness is unreachable from outside (F2). D9 **extends the parser** to
  surface `{ path, branch, prunable }`; the helper applies `!prunable && path.exists()`.
  Used by the guard **and** the commit-boundaries locator. `worktree_for_ref` keeps its
  signature for existing callers (behaviour-preservation). Chosen over a new
  dispatch-active runtime marker (`worktree/marker.rs` is per-worker, not a coord-root
  signal — more moving parts). **Limitation (F4):** a live coord worktree means
  "liveness", not strictly "the funnel still owns these phases" — a coord worktree left
  un-pruned through the pre-integrate audit window false-stands-down a post-drive solo
  phase. Caught loudly by the gate/conformance + `record-delta`; a precise dispatch-run
  ownership signal is a hardening follow-up (BL-ISS, §8 R2), not in this slice.
- **D10 — No REV for SPEC-022.** SPEC-022:180 *already* mandates the committed
  boundaries ledger; ISS-039 is the impl in violation. Committing it is conformance, not
  a spec change — verified the spec text names `boundaries.toml` in the committed
  run-ledger set. (If any spec text had described it as uncommitted, a REV would route —
  it does not.)
- **D11 — Projection-source guard, per-phase provenance set-check (pass-5 reshape).**
  The registry gate (D4) proves the conformance *input* is complete, but **not** that the
  projection *source* (the committed boundaries ledger feeding `plan_phases`) is complete
  — and the funnel double-write (D5, run_record_boundary:614) fills the registry
  independently of the committed ledger. **Pass-4's empty-only guard was proven unsound
  (pass-5):** (i) it fired only on a *fully* empty ledger, missing **partial** loss;
  (ii) "landed code ⇒ ledger row" **false-halted** mixed solo→dispatch and empty-code
  dispatch phases; (iii) its `code_delta_paths` copied `plan_review`'s exclusion set,
  guarding the wrong projection. **Reshaped:** at prepare-review, **before the ref
  projection**, assert **every registry row whose `provenance ∈ {Funnel, Unknown}` has a
  committed-ledger row**; halt naming the gaps. Pure phase-id set comparison — no
  working-tree diff, no exclusion set, no `code_delta_paths` (the helper is **deleted**).
  Provenance is the *only* signal distinguishing "legit solo phase, no committed row" from
  "lost funnel row" (D12); catches total **and** partial loss; never false-halts `Solo`
  or empty-code phases; names phases (§5.6). **`Unknown` is included** so a mid-upgrade
  active slice halts loudly, not silently (pass-6 MAJOR); closed-slice `Unknown` rows are
  inert (no prepare-review re-entry). Ordering load-bearing **before projection** (a halt
  creates no refs); the pre-derive read is clarity only — the derive cannot mask the gap
  (pass-6 MINOR).
- **D12 — `provenance` field on `BoundaryRow` (the discriminator; D11's foundation).**
  D11 needs to know, per completed phase, whether the funnel owns it. **No existing
  committed state records that** (verified: `journal.toml` is the CAS projection ledger
  *derived from* boundaries — circular; `candidates.toml` is review/close refs;
  `BoundaryRow` carried only OIDs). So the notes' "(B) derive from which writer recorded,
  cheaper" option is **illusory** — deriving from the writer requires persisting the
  writer's identity, i.e. a field. Chosen: add `provenance: Solo | Funnel | Manual`
  (absent → `Unknown`) to `BoundaryRow`. Lowest-touch (every registry writer already
  writes a `BoundaryRow`); also self-describes the registry for navigation (§5.6).
  **Provenance is sticky, merged atomically in the state seam (pass-6 BLOCKER + pass-7
  fixes):** the policy rides *inside* `record_source_delta`'s read-modify-write (state.rs:613),
  keyed on the **incoming** provenance — `Solo`/`Funnel` (landing writers) overwrite
  authoritatively; `Manual` (a `record-delta` correction) **preserves** any existing
  provenance, including `Solo`, `Funnel`, **and** `Unknown`, landing `Manual` only on a
  brand-new row. This is *not* a caller-side pre-read (which would race a concurrent
  funnel/derive write — pass-7 MAJOR). Without stickiness, `record-delta` would downgrade a
  lost funnel (or legacy `Unknown`) row to an excluded `Manual`, blind D11, and reopen the
  silent-under-projection hole. Back-compat is `#[serde(default)]` only — **no migration
  machinery** (User decision): D11 makes a mid-upgrade active slice's `Unknown` rows halt
  *loudly*, and clearing requires a one-time **reclassification** — re-record through the
  landing path, or hand-edit the runtime/disposable registry row's `provenance` to the
  truth; a bare `record-delta` deliberately neither clears nor downgrades it (§5.5). Closed
  slices never re-enter. `boundary.rs` + `slice.rs` + `state.rs` carry the change
  (`design-target`). **Does NOT close F4** — provenance marks ownership *post-record*; the
  solo binding's *stand-down decision* (pre-record, F4/R2) still keys on live-coord-worktree
  presence, and its precise fix (run-state ownership) remains the hardening follow-up
  (IMP-173). The efficiency note's "provenance → F4" claim is tempered here.
- **Alternatives rejected** — chain-fallback: unsound (D1). Working-file derive
  (the retracted pass-2 draft): SPEC-022 violation (P2-3). Read-time fallback in
  conformance: scope-rejected; papers over the write gap. **Dropping the funnel
  double-write to close the D11 gap:** rejected — it is a contract break (D5/F5); the D11
  guard closes the hole without removing the double-write. **Empty-only D11 guard +
  `code_delta_paths`** (pass-4 draft): rejected pass-5 — too weak (misses partial loss),
  false-halts solo/empty-code, guards the wrong exclusion set (D11/D12). **Provenance
  derived from a committed run-state** (notes option B): rejected — no such state exists
  (D12).

## 8. Risks & Mitigations

- **R1 — derive reads a stale/incomplete committed ledger.** A funnel phase the inline
  write never recorded to the working ledger never gets committed/derived → that row is
  missing → the **gate** halts loudly with the named gap. Fail-closed, never silent.
- **R2 — guard false-stand-down (F4-sharpened).** "A live coord worktree exists" is a
  *liveness* signal, not strictly "the funnel still owns these phases". A solo phase
  completed while a coord worktree is live — **not only during the active drive but
  through the pre-integrate audit window** (the coord tree is removed at integrate, after
  prepare-review) — stands the binding down though the funnel doesn't own it. Caught by
  the gate (at prepare-review) / conformance (at audit) → loud + `record-delta`. Accepted
  as the cost of never blessing a garbage row (the inverse — binding mis-captures a funnel
  phase — is corrected by the derive upsert; an *unrecorded* dispatched phase is the case
  the guard must protect, hence D3-kept). **Hardening follow-up:** a precise dispatch-run
  ownership signal (run-state, not worktree presence) would close the window — logged as a
  follow-up, not this slice's scope.
- **R3 — gate false-halt.** `registry_completeness` keys on completed `PHASE-NN`; a
  blocked/not-completed phase is excluded. Mitigation: same completed-set source as
  conformance (parity); message names exact gaps + remedy.
- **R4 — ISS-039 commit perturbs projection / sync (F5-corrected).** Committing the
  boundaries ledger re-enables claude per-phase projection. The "0 phase-cuts" symptom is
  **production-only** (real claude drives leave the ledger uncommitted — ISS-039); the
  existing e2e fixtures **manually `git commit` the ledger** (e2e_dispatch_lifecycle.rs:174,
  e2e_dispatch_sync.rs:111) and already assert `phase/064-01`, so they exercise projection
  but **not** the new splice-from-uncommitted-working-file path. Risk: a suite pinned to
  the old shape, or the extra commit shifting an asserted tip. Mitigation: §9 — keep
  `e2e_dispatch_lifecycle` / `e2e_dispatch_sync` green **and add a fixture that does NOT
  pre-commit the ledger** (the actual new path); the ledger commit is excluded from
  `review/<slice>` already (`plan_review` drops `.doctrine/dispatch/NNN`, dispatch.rs:2015).
- **R5 — behaviour regression in the binding / shared locators.** Only the guard
  predicate, the absent-stamp branch, and the reopen branch change in state.rs; the
  stamp-present path is byte-identical; `worktree_for_ref` keeps its signature (the
  parser gains a field; the new liveness helper is additive).
  Mitigation: existing binding + dispatch suites green; add the regression tests (§9).
- **R6 — commit-boundaries CAS race.** Another writer advances `dispatch/NNN` between
  the tip read and the CAS. Mirrors `commit_journal`'s `RefCas::Moved` bail — report,
  never clobber.
- **R7 — prepare-review re-run journal poisoning (F1).** A boundaries commit that advanced
  the ref every run would, on the next run, make the existing `review/*`/`phase/*` refs
  read as stale → the recovery `commit_journal` persists Failed rows over the verified
  journal (e2e_dispatch_sync.rs:435) → a later integrate refuses. Mitigation:
  `commit_boundaries` is content-idempotent (D7b — no commit on identical content) and the
  gate runs before projection (D7 ordering — a halt creates no refs). The pre-existing
  "re-running a *successful* prepare-review bails stale" is unchanged and out of scope.

## 9. Quality Engineering & Validation

- **Pure unit:** `check_completeness` (reuse). `forget_source_delta` round-trip
  (record → forget → absent). (No `resolve_phase_start` — D1.)
- **VT — ISS-039 commit (the NEW splice path, F5):** `record-boundary` writes the working
  ledger with N boundaries and **no manual `git add/commit`** (unlike the existing
  fixtures) → `prepare-review` itself commits `boundaries.toml` onto `dispatch/NNN`
  (`git ls-tree` shows it alongside `journal.toml`), `read_ledger` reads N rows, and
  `phase/<slice>-NN` refs project. This is the case the current e2e do not cover.
- **VT — commit-boundaries idempotent / no journal poison (F1):** run `prepare-review`
  twice on an unchanged working ledger → the second run does **not** advance
  `dispatch/NNN` (same tip oid) and does not rewrite verified journal rows as Failed.
- **VT — malformed ledger rejected (F3):** a working `boundaries.toml` that is invalid
  TOML → `commit_boundaries` returns `Err` and the dispatch tip is **unchanged** (no
  garbage committed).
- **VT — gate-before-projection (F1):** an incomplete registry → `prepare-review` halts
  with **no `review/*` / `phase/*` refs created**; after `record-delta` fills the gap a
  re-run projects cleanly.
- **VT — D11 total loss:** a dispatched slice whose `run_record_boundary` filled the
  **registry** (`Funnel` rows) but whose coord worktree was removed **before**
  prepare-review (committed ledger empty) → prepare-review **halts naming the phases**,
  does NOT pass on the pre-filled registry, creates no refs.
- **VT — D11 partial loss (the pass-5 hole):** committed ledger has *some* funnel rows,
  missing one a `Funnel` registry row records → halt naming **only the missing phase**;
  a complete committed ledger → no halt. (The empty-only guard missed this.)
- **VT — D11 no false-halt (the pass-5 false-halts):** (a) mixed solo→dispatch — solo
  phases (`Solo`) land code on the branch with no committed-ledger row → **no halt**;
  (b) empty-code dispatch phase (`start == end`, `Funnel`) recorded in both ledgers →
  **no halt**. D11 never reads code paths.
- **VT — D11 excludes `Solo`/`Manual`, includes `Unknown` (pass-6):** a `Solo` row absent
  from the committed ledger → no halt; a fresh `Manual` row (pure-solo hand-bootstrap)
  absent → no halt; an **`Unknown`** row absent from the committed ledger → **halt**
  naming it ("legacy/unclassified", the mid-upgrade active-slice case).
- **VT — provenance merge in `record_source_delta` (pass-6 BLOCKER / pass-7):** incoming
  `Manual` over an existing `Funnel` row → row stays `Funnel`; over existing `Solo` → stays
  `Solo`; over existing `Unknown` → stays `Unknown`; over **no** existing row → `Manual`.
  Incoming `Funnel`/`Solo` (landing writers) → overwrites. (Unit test at the state seam —
  proves the merge is atomic in the writer, not a caller pre-read.)
- **VT — record-delta cannot clear a funnel/legacy halt:** a `Funnel` (or `Unknown`) row
  whose committed-ledger row is missing → `record-delta` keeps its provenance → D11 **still
  halts**; only committing the ledger (Funnel) or reclassifying to `Solo` (legacy that was
  solo) clears it.
- **VT — provenance write-sites:** the solo binding stamps `Solo`, `run_record_boundary`
  stamps `Funnel` on both the working ledger and registry rows, the derive stamps
  `Funnel`; a legacy TOML row without the field deserializes as `Unknown`.
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
with the User (prepare-review splice; liveness-verified probe).

### External pass 3 — codex (GPT-5.5), 2026-06-26 — committed-ref revision — dispositions

All 5 verified against source; all ACCEPTED. None broke the committed-ref approach —
refinements to the commit seam, the liveness probe, and the test plan.

- **F1 (BLOCKER) re-run journal poisoning.** A boundaries commit that advances the ref
  every run makes the next run read existing projection refs as stale → recovery
  `commit_journal` persists Failed rows over the verified journal (e2e_dispatch_sync.rs:435)
  → integrate refuses. **Fixed:** `commit_boundaries` is **content-idempotent** (no commit
  on identical content) **and** the derive + gate run **before** projection, so a
  gate-halt creates no refs (clean `record-delta` → re-run). D7b + D7 ordering, §5.2,
  §5.5 re-run bullet, R7.
- **F2 (MAJOR) prunable unreachable by wrapping.** `parse_worktree_for_ref` (git.rs:1163)
  discards the `prunable` marker. **Fixed:** D9 **extends the parser** to surface
  `{ path, branch, prunable }`; not a wrapper. §5.2, D9.
- **F3 (MAJOR) raw bytes committed before validation.** **Fixed:** `commit_boundaries`
  parses+validates the working ledger before the commit; malformed → `Err`, tip unchanged.
  D7a, §5.2, R-VT.
- **F4 (MAJOR) liveness ≠ ownership.** A coord worktree un-pruned through the
  pre-integrate audit window false-stands-down a post-drive solo phase (wider than the
  admitted during-drive crack). **Accepted with mitigation** (gate/conformance loud +
  `record-delta`); a precise dispatch-run ownership signal is a **hardening follow-up**,
  not this slice. R2-sharpened, D9-limitation.
- **F5 (MINOR) R4 test rationale stale.** Existing e2e **pre-commit** the ledger
  (lifecycle:174, sync:111), so they don't exercise the new splice; "0 phase-cuts" is
  production-only. **Fixed:** R4 corrected; §9 adds a no-pre-commit fixture VT.

### External pass 4 — codex (GPT-5.5), 2026-06-26 — confirmatory — dispositions

Confirmed F1–F3, F5 integrations sound. Found one residual BLOCKER + one MINOR.

- **Pass-4 BLOCKER — registry gate masks an absent committed ledger.** Verified against
  source: `run_record_boundary` double-writes the **registry** (dispatch.rs:614) but only
  the **working** ledger file (:606) — never the committed dispatch ref. So a coord
  worktree removed before the first prepare-review ⇒ `commit_boundaries` no-ops ⇒ committed
  ledger empty ⇒ `plan_phases` projects 0, yet `registry_completeness` passes on the
  pre-filled registry ⇒ green gate, silent broken projection. **Fixed: D11 projection-
  source guard** (empty committed ledger + landed code ⇒ halt), §5.2/§5.5/§9.
- **Pass-4 MINOR — idempotency compared canonical-working vs raw-committed-blob.** Not
  canonical-to-canonical unless the committed blob is known canonical. **Fixed:** D7b now
  compares **tree oids** (`tree_with_file` result vs current `tip_tree`) — git dedups
  identical content, so this is robust to formatting. §5.2.
- **F4 re-confirmed insufficient-without-the-blocker → now covered:** the D11 guard closes
  the path where the gate was satisfied by the double-write while projection was empty.

### External pass 5 — codex (GPT-5.5), 2026-06-26 — D11 unsound — dispositions

Pass-5 reopened **D11** (the pass-4 empty-only projection-source guard). All three
defects ACCEPTED; the User-requested efficiency lens supplied the converging fix.

- **P5-1 (BLOCKER) empty-only predicate too weak → ACCEPTED.** D11 fired only on a fully
  empty committed ledger; a **partial** ledger slips past it and the registry gate
  (pre-filled by the double-write) → silent under-projection. **Fixed:** per-phase
  set-check (D11 reshaped).
- **P5-2 (BLOCKER) false-halts mixed solo→dispatch / empty-code → ACCEPTED.** "landed
  code ⇒ committed ledger row" is too broad: solo phases land code with no committed row
  by design; empty-code dispatch phases leave an empty `code_delta`. **Fixed:** provenance
  excludes `Solo`; the guard never inspects code paths (D11 reshaped).
- **P5-3 (MAJOR) exclusion-set mismatch → ACCEPTED.** `code_delta_paths` copied
  `plan_review`'s filter, not `plan_phases`' — guarding the wrong projection. **Fixed:**
  `code_delta_paths` **deleted**; the invariant is a pure phase-id set comparison (D11).
- **Efficiency lens (value/ergonomics, not correctness):** the missing per-phase landing
  path is exactly D11's discriminator. Disposition: **D12** adds `provenance` to
  `BoundaryRow` (the only sound option — B is illusory, D12); §5.6 claims nav-coherence as
  value and **backlogs** the derived nav view; the F4 ownership-signal hardening stays a
  backlog follow-up (provenance does **not** close F4 — D12). Halt-message phase-naming
  absorbed into D11.

### Revision 4 — pass-5 reshape integrated (2026-06-26)

D11 reshaped from the empty-only guard to the **per-phase provenance set-check**; **D12**
adds the `provenance` field; `git::code_delta_paths` **removed** from scope; **§5.6**
records the registry's nav/value role. §1–§9 above are revised accordingly (this body is
the single source of truth; `notes.md` "⚠ OPEN FINDINGS" is now **closed** — superseded by
this revision). Committed-ref core (D2/D7/D8/D9/D10) unchanged.

### External pass 6 — codex (GPT-5.5), 2026-06-26 — Rev-4 reshape — dispositions

Hostile pass on the provenance reshape. Two real defects + two doc fixes; all ACCEPTED
and integrated (Rev 5). The reshape's core (provenance as the discriminator, set-check,
`code_delta_paths` dropped) survived — the defects were in the *edges* of provenance use.

- **P6-1 (BLOCKER) `record-delta` reintroduces silent under-projection → ACCEPTED, FIXED.**
  `record-delta` writes only the registry (`Manual`), not the committed ledger
  (slice.rs:1970); `plan_phases` projects only from committed rows (dispatch.rs:2041). The
  Rev-4 draft let it stamp `Manual` unconditionally → an operator could clear a D11 halt
  while projection stayed empty. **Fix:** `record-delta` is **provenance-preserving** —
  keeps an existing `Solo`/`Funnel`, stamps `Manual` only on a fresh row (D12, §5.2/§5.3/
  §5.5). A lost funnel row stays `Funnel`, so D11 re-halts until the ledger is committed —
  the "commit the ledger" contract is now **enforced**, not documented.
- **P6-2 (MAJOR) `Unknown` unsound for mid-upgrade active slices → ACCEPTED, FIXED.**
  Excluding `Unknown` let a legacy funnel row satisfy D4 while invisible to D11. **Fix:**
  D11 **includes** `Unknown` in the expected set (provenance ∈ {Funnel, Unknown}) → a
  mid-upgrade active slice halts loudly; a one-time hand-record clears it (User's
  hand-fix-over-machinery decision). Closed slices never re-enter, so their `Unknown` rows
  stay inert. §5.2/§5.3/§5.5/D11.
- **P6-3 (MINOR) "before derive" rationale wrong → ACCEPTED, FIXED.** The derive only
  upserts *committed* rows; it can't mask a missing-ledger registry row. Load-bearing
  ordering is **before projection**, not before derive. Text corrected (§5.1/§5.2/D11);
  the pre-derive read kept for clarity only.
- **P6-4 (MINOR) `notes.md` self-contradiction → ACCEPTED, FIXED.** A stale
  `code_delta_paths "to be added"` survived in the code map (notes.md:171). Reconciled.
  (Codex's "source has no `provenance` field" is a non-finding — design stage, not yet
  implemented.)
- **Residual risk (codex):** provenance is a *post-record* ownership mark; the binding's
  *pre-record* stand-down still rides liveness (F4) — already accepted + backlogged
  (IMP-173). Unchanged.

### Revision 5 — pass-6 edges integrated (2026-06-26)

`record-delta` provenance-preserving (P6-1); D11 expected-set = {Funnel, Unknown} (P6-2);
ordering text corrected (P6-3); `slice.rs` joins design-target. Committed-ref core +
provenance discriminator (D2/D7/D8/D9/D10/D11/D12) intact.

### External pass 7 — codex (GPT-5.5), 2026-06-26 — confirmatory on Rev 5 — dispositions

Confirmed the Rev-4 reshape core holds and the non-legacy `Unknown` containment is sound.
Found two defects in the *pass-6 fixes themselves*; both ACCEPTED + integrated (Rev 6).

- **P7-1 (BLOCKER) `Unknown` remediation self-contradictory → ACCEPTED, FIXED.** Rev 5 said
  both "`record-delta` preserves provenance" *and* "a legacy `Unknown` row is cleared by
  `record-delta`" — incompatible (downgrade reopens the silent gap; preserve makes the
  documented fix false). **Fix:** `record-delta` preserves **all** existing provenance incl.
  `Unknown`; a legacy halt clears only by **reclassification** (re-record through the landing
  path, or hand-edit the runtime registry row) — never a bare `record-delta`. §5.2/§5.5/D12.
- **P7-2 (MAJOR) provenance-preserve at the wrong seam, racy → ACCEPTED, FIXED.** A
  caller-side pre-read in `run_record_delta` races a concurrent funnel/derive write. **Fix:**
  the merge moves *into* `record_source_delta`'s read-modify-write (state.rs:613), keyed on
  the incoming provenance (`Solo`/`Funnel` overwrite; `Manual` preserves) — atomic, no
  caller pre-read. `state.rs` carries the merge. §5.2/§5.3/D12.
- **P7-3 (MINOR) doc drift → FIXED.** §5.1 diagram and the `notes.md` gate still said
  `Funnel`-only / `Manual`-always; reconciled to {Funnel, Unknown} + the seam merge.
- **Confirmed closed:** non-legacy `Unknown` is legacy-only (every post-impl writer stamps
  provenance explicitly), so it is not a steady-state output — the false-halt is bounded to
  mid-upgrade slices.

### Revision 6 — pass-7 fixes integrated (2026-06-26)

`record-delta` preserves `Unknown` too; legacy halt clears by reclassification, not bare
`record-delta` (P7-1); the provenance merge is atomic in `record_source_delta` keyed on
incoming provenance (P7-2); §5.1 + `notes.md` reconciled (P7-3); `state.rs` confirmed in
design-target. The committed-ref core, the provenance discriminator, and the projection-
source guard are now internally consistent across §5.1–§5.6/§7/§9. **Re-pass codex on Rev
6; if clean, `/plan`.**
