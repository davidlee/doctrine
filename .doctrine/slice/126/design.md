# SL-126 Design — structural close-gate: dispatched code unintegrated

> Status: drafted (pre-lock). Resolves IMP-102. Backstop sibling to SL-121.

## 1. Problem & framing

Closing a **dispatched** slice depends on a human running `doctrine dispatch
sync --integrate` (close step-3a, `plugins/doctrine/skills/close/SKILL.md:62`) to
project the slice's journaled refs onto trunk. Nothing structural stops `slice
status <id> done` from succeeding when that step was skipped or failed — the
journaled code sits unintegrated while the slice is marked terminal. The teeth
today are skill prose, not the binary.

**The gate is the binary form of close-verify step-3a(b).** Close already
prescribes, in prose, exactly this check at integrate time:

```bash
planned=$(doctrine dispatch sync --slice N --show-journal-trunk-oid --trunk refs/heads/main)
git diff --quiet "$planned" refs/heads/main
```

IMP-102 moves those teeth into `slice status done`. **Divergence (deliberate):**
the gate fires *later* than step-3a — by the time `slice status … done` runs,
trunk may have advanced (other slices landed). So the gate tests **ancestry**
(`is_ancestor(planned_oid, trunk_tip)`), not the skill's tree-equality
(`git diff --quiet`), which would false-refuse a trunk that moved forward.

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
query reads only leaves (`git`) + local types, so it stays `leaf`.

```rust
/// Three-state integration status of a slice's dispatched code vs trunk.
pub(crate) enum TrunkIntegration {
    /// No `refs/heads/dispatch/<slice>` ref — slice was never dispatched.
    NotDispatched,
    /// The journal trunk row's `planned_new_oid` is an ancestor of trunk.
    Integrated,
    /// Dispatched, but integration is unproven; the String is the refusal reason.
    Blocked(String),
}

pub(crate) fn trunk_integration(root: &Path, slice: u32) -> anyhow::Result<TrunkIntegration>
```

Resolution order:

1. `git rev-parse --verify --quiet refs/heads/dispatch/<slice:03>` absent ⇒
   `NotDispatched`. (`read_path_at` returns `None` for both absent-ref and
   absent-path, so the ref-existence probe is **separate** and explicit.)
2. Tree-read `journal.toml` at the dispatch ref via `git::read_path_at` (the
   coordination worktree is GC'd at `/dispatch` conclude, so the on-disk copy is
   unreliable — the `sync-tree-reads-ledger-not-worktree` invariant; **not**
   `ledger::read_journal`, which reads the filesystem). Unreadable / TOML-parse
   failure ⇒ `Blocked("journal unreadable")`.
3. **Trunk row = self-describing** (OQ-1, option (b)): the single `refs/heads/*`
   row whose `target_ref` is **outside** the dispatch-internal namespaces
   `{refs/heads/dispatch/, refs/heads/review/, refs/heads/candidate/,
   refs/heads/phase/}`. `0` such rows ⇒ `Blocked("no trunk row in journal")`
   (the exact shape of a never-integrated slice); `>1` ⇒
   `Blocked("ambiguous trunk row")`.
4. Empty `planned_new_oid` ⇒ `Blocked("trunk row has no planned oid")` (guards
   `is_ancestor("")` from erroring; fail-closed).
5. Resolve that row's `target_ref` tip (`rev-parse`); unresolved ⇒
   `Blocked("trunk ref unresolved")`.
6. `git::is_ancestor(planned_new_oid, tip)` → `true` ⇒ `Integrated`; `false` ⇒
   `Blocked("planned tip not on trunk")`.

**Fail-closed (OQ-2):** only `NotDispatched` and `Integrated` pass; every anomaly
on a dispatched slice is `Blocked(reason)`. No bypass flag in v1 — the recovery
is to actually integrate (or abandon).

### 3.2 The gate site (`slice::run_status`, after the drift gate ~3.2:405)

```rust
if from == "reconcile" && to == "done" {
    match crate::ledger::trunk_integration(&root, id)? {
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
as `slice → review` (blocker gate).

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
- **VT-2** dispatched, trunk row `planned_new_oid` is ancestor of trunk →
  succeeds.
- **VT-3** dispatched, `planned_new_oid` not on trunk → refused, named token.
- **VT-4** dispatched, journal has no trunk row → refused (fail-closed).
- **VT-5** gate fires **only** on `reconcile → done` — an unintegrated dispatched
  slice on `audit → reconcile` is not gated here.
- **VT-6** composition — an unintegrated slice that *also* has an unresolved
  blocker is refused (either gate suffices).
- **VT-7** (unit) `trunk_integration` truth table from a git fixture (dispatch ref
  + committed `journal.toml`): each variant incl. unreadable + ambiguous.

Evidence lands as Rust tests beside `ledger`/`slice`, using the existing
git-repo fixture pattern (cf. dispatch journal tests).

## 6. Decisions & non-goals

- **D1 (OQ-1):** trunk ref is **self-describing from the journal trunk row**
  (namespace elimination), not a hardcoded literal or new config. The
  `[dispatch] deliver_to` config that would become the single source of truth is
  deferred to **IMP-124** (fulfils the close-skill TODO; `after: SL-126`).
- **D2 (OQ-2):** **fail-closed** — any dispatched slice not provably integrated
  refuses; no `--force` bypass in v1.
- **D3:** ancestry (`is_ancestor`), not tree-equality, so a moved-forward trunk
  still passes (§1).
- **Non-goals:** no trunk mutation, no auto-integrate (ADR-006 sole-writer), no
  `deliver_to` config, no bypass flag, `reconcile → done` only.
