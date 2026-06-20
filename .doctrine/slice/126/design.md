# SL-126 Design — structural close-gate: dispatched code unintegrated

> Status: drafted (pre-lock). Resolves IMP-102. Backstop sibling to SL-121.

## 1. Problem & framing

Closing a **dispatched** slice depends on a human running `doctrine dispatch
sync --integrate` (close step-3a, `plugins/doctrine/skills/close/SKILL.md:62`) to
project the slice's journaled refs onto trunk. Nothing structural stops `slice
status <id> done` from succeeding when that step was skipped or failed — the
journaled code sits unintegrated while the slice is marked terminal. The teeth
today are skill prose, not the binary.

**The gate is a structural cousin of close-verify step-3a(b), not its
equivalent.** Close already prescribes, in prose, a tree-equality check at
integrate time:

```bash
planned=$(doctrine dispatch sync --slice N --show-journal-trunk-oid --trunk refs/heads/main)
git diff --quiet "$planned" refs/heads/main   # planned tip == trunk tip, right now
```

IMP-102 puts a *related* check in the binary, but with a **deliberately different
and weaker semantic**, because it fires at a different time. By `slice status …
done`, trunk may have advanced (other slices landed), so tree-equality would
false-refuse. The gate therefore asks the weaker question **"did the projected
commit get integrated into trunk's history?"** — `is_ancestor(planned_oid,
trunk_tip)`. Two honest consequences of that choice (called out so no one mistakes
ancestry for tree-on-trunk):

- it **tolerates a moved-forward trunk** (the reason for choosing it); and
- it does **not** detect a *post-integration revert* — if the projected commit
  landed and trunk later reverted it, ancestry still holds and the gate passes.
  That is **out of scope by design**: integration *did* occur; a later deliberate
  revert is a separate concern, not the "forgot to run integrate" omission this
  gate exists to catch.

The gate is **not** a substitute for close step-3a(b)'s tree-true check at
integrate time — it is a coarser backstop for the omission case.

SL-121 fixes the *other* face of this hub (`git-ref-vs-worktree-placement`): it
makes `sync --integrate` itself leave clean state. SL-126 is the structural
backstop — even with a clean integrate, a human who forgets to run it must not be
able to mark the slice `done`. Belt to SL-121's suspenders.

## 2. Current vs target behaviour

`slice::run_status` (`src/slice.rs:354`) already carries two reverse close-gates,
both one-way shell→query couplings (the queried module never imports `slice`):

- **blocker scan** (`src/slice.rs:374`) — `review::unresolved_blockers_for`,
  fires on both closure-seam crossings.
- **drift gate** (`src/slice.rs:394`) — `undischarged_drift`, fires on
  `reconcile → done` only.

**Target:** a **third** gate of the same shape on the `reconcile → done`
crossing, refusing when a dispatched slice's journaled trunk tip is not yet an
ancestor of trunk. Composes with the existing two — any one refuses
independently.

## 3. Mechanism

### 3.1 The query (new, in `ledger`)

`ledger.rs` owns the `Journal`/`JournalRow` types and imports nothing in-crate
(tier `leaf`, `out=0`). It is the cycle-free home: `dispatch` already imports
`crate::slice::read_plan` (`src/dispatch.rs:93`), so a query in `dispatch` called
from `slice` would form the forbidden `slice ↔ dispatch` cycle (ADR-001). The
query reads only leaves (`git`) + local types, so it stays `leaf`. It is kept
**mechanical** — it identifies the trunk row by an exact ref the *caller* supplies
and returns a neutral verdict; the trunk ref's value and the user-facing refusal
copy live in the `slice` shell (cohesion: dispatch/close *policy* does not leak
into a leaf — RV-codex F6).

```rust
/// Integration status of a slice's dispatched code vs a given trunk ref.
pub(crate) enum TrunkIntegration {
    /// No dispatch ref, or a dispatch ref with an empty journal — nothing was
    /// ever projected. (A bare `worktree coordinate` creates the dispatch branch
    /// eagerly with an empty journal; that alone is NOT "dispatched" — RV-codex F2.)
    NotDispatched,
    /// The journal trunk row's `planned_new_oid` is an ancestor of `trunk_ref`'s tip.
    Integrated,
    /// Dispatched (journal has rows) but integration is unproven; String = reason.
    Blocked(String),
}

pub(crate) fn trunk_integration(
    root: &Path,
    slice: u32,
    trunk_ref: &str,   // the caller-owned delivery ref (slice passes TRUNK_REF)
) -> anyhow::Result<TrunkIntegration>
```

Resolution order:

1. `git rev-parse --verify --quiet refs/heads/dispatch/<slice:03>` absent ⇒
   `NotDispatched`. (`read_path_at` returns `None` for both absent-ref and
   absent-path, so the ref-existence probe is **separate** and explicit.)
2. Tree-read `journal.toml` at the dispatch ref via `git::read_path_at` (the
   coordination worktree is GC'd at `/dispatch` conclude, so the on-disk copy is
   unreliable — the `sync-tree-reads-ledger-not-worktree` invariant; **not**
   `ledger::read_journal`, which reads the filesystem). TOML-parse failure ⇒
   `Blocked("journal unreadable")`. **Absent file OR zero rows ⇒ `NotDispatched`**
   — a coordinated-but-never-projected slice has nothing to integrate, so it must
   not be gated (RV-codex F2).
3. **Trunk row = exact match** `target_ref == trunk_ref` (RV-codex F1/F4). This is
   the *same* selector `dispatch::run_show_journal_trunk_oid` already uses
   (`src/dispatch.rs:147`) and the same ref close step-3a passes
   (`--trunk refs/heads/main`). Uniqueness is a **guarantee from the integrate
   writer** — `integrate`'s `fresh` filter dedups rows by `target_ref`
   (`src/dispatch.rs:34`), so at most one row can match — *not* an inferred
   heuristic. The `review/`, `phase/`, and any `--edge` row (e.g.
   `refs/heads/edge`, `tests/e2e_dispatch_sync.rs:695`) simply never equal
   `trunk_ref`, so they are inert here. No trunk row ⇒
   `Blocked("dispatched but no trunk row — integrate --trunk never completed")`.
4. Empty `planned_new_oid` ⇒ `Blocked("trunk row has no planned oid")` (guards
   `is_ancestor("")` from erroring; fail-closed).
5. Resolve `trunk_ref`'s tip (`rev-parse`); unresolved ⇒
   `Blocked("trunk ref {trunk_ref} unresolved")`.
6. `git::is_ancestor(planned_new_oid, tip)` → `true` ⇒ `Integrated`; `false` ⇒
   `Blocked("planned tip not on trunk")`.

**Fail-closed (OQ-2):** `NotDispatched` (steps 1–2) and `Integrated` pass; every
anomaly on a slice that *did* project (journal has rows) is `Blocked(reason)`. No
bypass flag in v1 — the recovery is to actually integrate (or abandon).

### 3.2 The gate site (`slice::run_status`, after the drift gate ~line 405)

```rust
/// The trunk delivery ref. Mirrors close step-3a's `--trunk refs/heads/main`
/// and `run_show_journal_trunk_oid`'s selector. Generalised to a configured
/// `[dispatch] deliver_to` by IMP-124; the config read would land HERE, in the
/// shell, keeping `ledger` ref-agnostic.
const TRUNK_REF: &str = "refs/heads/main";

if from == "reconcile" && to == "done" {
    match crate::ledger::trunk_integration(&root, id, TRUNK_REF)? {
        crate::ledger::TrunkIntegration::NotDispatched
        | crate::ledger::TrunkIntegration::Integrated => {}
        crate::ledger::TrunkIntegration::Blocked(reason) => anyhow::bail!(
            "slice {} → done: refused — dispatched code not integrated to trunk: \
             {reason} (run close step-3a `dispatch sync --integrate`, verify, retry)",
            canonical_id(id)
        ),
    }
}
```

`slice` (command) → `ledger` (leaf): downward, ADR-001 rule 1. Same one-way shape
as `slice → review` (blocker gate). The trunk ref is owned here (the impure
config-reading seam), not in the leaf.

### 3.3 DRY (no parallel journal-read)

Extract the tree-read into `ledger::read_journal_at_ref(root, slice) ->
anyhow::Result<Option<Journal>>` and refactor `dispatch::run_show_journal_trunk_oid`
(`src/dispatch.rs:135`) to consume it, retiring the bespoke
`read_ledger::<Journal>` path for the trunk-oid read.

## 4. Layering (ADR-001) — verified

| Edge | Tiers | Verdict |
|---|---|---|
| `slice → ledger` (gate) | command → leaf | downward ✅ |
| `ledger → git` (tree-read) | leaf → leaf | leaves may depend on leaves ✅; `git ∌ ledger` ⇒ no cycle, `tangle_baseline` leaf=0 holds |
| `dispatch → ledger` (DRY) | command → leaf | pre-existing, downward ✅ |

The `slice ↔ dispatch` cycle is **avoided** by siting the query in `ledger`. No
new `[[accepted_violation]]`, no tangle growth — `just gate`'s `syn` fitness
check passes untouched.

## 5. Verification

- **VT-1** not-dispatched (no `dispatch/<slice>` ref) → `reconcile → done`
  succeeds.
- **VT-1b** dispatch ref present but journal **empty/zero rows** (bare
  `coordinate`, never projected) → `NotDispatched` → `reconcile → done` succeeds
  (RV-codex F2 regression).
- **VT-2** dispatched, trunk row `planned_new_oid` is ancestor of trunk →
  succeeds.
- **VT-2b** journal carries **both** a trunk row (`refs/heads/main`) **and** an
  edge row (`refs/heads/edge`); the gate selects the trunk row by exact match and
  resolves on it — no ambiguity, no false refuse (RV-codex F1 regression).
- **VT-3** dispatched, `planned_new_oid` not on trunk → refused, named token.
- **VT-4** dispatched (journal has rows) but **no** `refs/heads/main` row →
  refused (fail-closed).
- **VT-5** gate fires **only** on `reconcile → done` — an unintegrated dispatched
  slice on `audit → reconcile` is not gated here.
- **VT-6** composition — an unintegrated slice that *also* has an unresolved
  blocker is refused (either gate suffices).
- **VT-7** (unit) `trunk_integration` truth table from a git fixture (dispatch ref
  + committed `journal.toml`): every variant incl. journal-unreadable and
  empty-oid.

Evidence lands as Rust tests beside `ledger`/`slice`, using the existing
git-repo fixture pattern (cf. dispatch journal tests).

## 6. Decisions & non-goals

- **D1 (OQ-1) — REVISED after RV-codex F1/F4.** The trunk row is identified by
  **exact `target_ref == TRUNK_REF`** (`refs/heads/main`), *not* by
  namespace-elimination (the original option (b), which false-refused a valid
  `--trunk main --edge refs/heads/edge` journal — two non-excluded rows). Exact
  match mirrors `run_show_journal_trunk_oid` (`src/dispatch.rs:147`) and close
  step-3a's `--trunk`; uniqueness is **guaranteed by the integrate writer's
  `fresh` dedup** (`src/dispatch.rs:34`), not inferred. `TRUNK_REF` is owned by
  the `slice` shell (the config-read seam); **IMP-124** generalises it to
  `[dispatch] deliver_to` (after SL-126). Same `refs/heads/main` assumption the
  existing read surface already bakes in — no new limitation.
- **D2 (OQ-2):** **fail-closed** — a slice that *projected* (journal has rows) but
  is not provably integrated refuses; no `--force` bypass in v1. A
  coordinated-but-never-projected slice (empty journal) is *not* gated (F2).
- **D3 — ancestry, with eyes open (RV-codex F3).** `is_ancestor`, not
  tree-equality, so a moved-forward trunk still passes (§1). Consciously weaker
  than close step-3a(b): it proves *integration occurred*, not that the projected
  tree survives at trunk tip, and so does **not** flag a post-integration revert
  (out of scope — a deliberate act, not the forgotten-integrate omission). The
  gate is a backstop for the omission case, **not** a replacement for 3a(b).
- **D4 (RV-codex F6):** `trunk_integration` stays mechanical in leaf `ledger`
  (find row by caller-supplied ref, ancestry, neutral verdict); the trunk ref and
  refusal copy live in the `slice` shell. No dispatch/close policy in the leaf.
- **Non-goals:** no trunk mutation, no auto-integrate (ADR-006 sole-writer), no
  `deliver_to` config (→ IMP-124), no bypass flag, no post-integration-revert
  detection, `reconcile → done` only.
