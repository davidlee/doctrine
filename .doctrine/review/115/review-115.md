# Review RV-115 — reconciliation of SL-126

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation audit of a dispatched slice).

**Review surface (F-2).** Reviewed the **candidate interaction branch**
`cand-126-review-001` (`candidate/126/review-001` @ `cd2535b3`) — `review/126`
merged onto current `main` (`a0cfe490`). `main` advanced past the coordination
base (`c3aed867`) while the slice was dispatched; the candidate cleanly merges
(main IS an ancestor — no conflict), and the integrated surface verifies green
(clippy workspace-clean, 2094 unit + 17 layering + 26 e2e tests pass). Evidence
refs `dispatch/126` / `review/126` are immutable (R2).

**Lines of attack — invariants held against the locked design:**
- **§3.1 resolution order** — does `ledger::trunk_integration` implement the
  REVISED order exactly (explicit ref probe; absent-ref/empty-journal ⇒
  NotDispatched, F2; exact `target_ref` match with `>1` fail-closed, F1/F4/F7;
  empty-oid guard; ancestry), and stay **mechanical/ref-agnostic** (no
  `refs/heads/main` literal, no refusal policy in the leaf, D4/F6)?
- **§3.2 gate** — does the third gate fire on `reconcile → done` **only**,
  **compose** independently with the blocker + drift gates, and own `TRUNK_REF`
  + the refusal copy in the **shell**?
- **§3.3 DRY** — does `run_show_journal_trunk_oid` consume the shared
  `read_journal_at_ref`, with the bespoke trunk-oid `read_ledger::<Journal>` path
  gone and behaviour preserved?
- **ADR-001 layering** — only new edges `slice → ledger` and `ledger → git`
  (both downward); no `slice ↔ dispatch` cycle; leaf `tangle_baseline` = 0, no new
  `accepted_violation`.
- **VT completeness** — is every VT-1..VT-8 criterion backed by a real test, and
  is the unit truth-table backstopped by the **real-journal** e2e (VT-7/VT-8)?

## Synthesis

**Verdict: conformant. Reconcile-ready, no blockers.** SL-126 implements the
locked design with no behavioural divergence; the two findings are a nit and a
design-scoped non-issue, both terminal.

**What was verified against the candidate (integrated) surface:**

- **§3.1 query (`ledger::trunk_integration`, `src/ledger.rs:463`).** Implements the
  REVISED six-step order exactly: explicit dispatch-ref probe (step 1);
  absent-journal / zero-rows ⇒ `NotDispatched`, unreadable ⇒ `Blocked` (step 2,
  F2); exact `target_ref == trunk_ref` match **counted**, `0` and `>1` both
  fail-closed (step 3, F1/F4/F7); empty-oid guard (step 4); trunk-tip resolve (step
  5); `is_ancestor` (step 6). Every `Blocked` reason token matches design §3.1
  verbatim. The leaf stays **mechanical and ref-agnostic** — no `refs/heads/main`
  literal, no refusal policy (D4/F6). `read_journal_at_ref` tree-reads at the ref
  (not disk), correctly distinct from `read_journal` (filesystem).
- **§3.2 gate (`slice::run_status`, `src/slice.rs:414`).** A third `if from ==
  "reconcile" && to == "done"` block, sited after the drift gate, before
  `set_slice_status`. **Composes** — a separate `if`, so blocker / drift /
  integration each refuse independently. `NotDispatched | Integrated` pass;
  `Blocked(reason)` bails with the named token + recovery copy (matches §3.2
  verbatim). `TRUNK_REF` + the refusal copy are owned in the shell (`:447`).
- **§3.3 DRY.** `run_show_journal_trunk_oid` consumes `read_journal_at_ref(…)
  .unwrap_or_default()`; the bespoke trunk-oid `read_ledger::<Journal>` path is
  gone; the `with_context` "no journal row" refusal is preserved (behaviour
  unchanged).
- **ADR-001 layering.** `architecture_layering` gate green (17/17): `slice →
  ledger` and `ledger → git` both downward, no `slice ↔ dispatch` cycle, leaf
  `tangle_baseline` = 0, no new `accepted_violation`.
- **VT completeness.** Unit truth-table complete (`ledger`: 11 tests incl.
  ambiguous-row, unreadable, empty-oid, edge-vs-trunk exact match; `read_journal_at_ref`
  None/None/Some) and **backstopped by the real-journal e2e** (`tests/e2e_dispatch_sync.rs`:
  `vt7_close_integration_succeeds_after_real_trunk_integrate` /
  `…_refused_without_trunk_integrate`). Gate behavioural tests VT-1..VT-6 (+vt1c,
  +vt6 blocker-independence) present.

**Evidence.** Integrated candidate surface (`review/126` ⊕ `main@a0cfe490`):
clippy workspace-clean, **2094 unit + 17 layering + 26 e2e tests pass**. `main`
advanced past the coordination base during dispatch; the candidate merges with no
conflict (`main` is an ancestor), so the slice integrates cleanly with current
trunk.

**Standing risks / accepted tradeoffs.** None material. Design D3's conscious
weakness is intact: the gate proves *integration occurred* (ancestry), not
tree-survival — it tolerates a moved-forward trunk and does **not** flag a
post-integration revert (out of scope by design). F-1 (comment §-ref) tolerated;
F-2 (two journal readers) is design-scoped intent, not drift.

## Reconciliation Brief

### Per-slice (direct edit)
- **None.** `design.md` matches the implementation — the locked §3.1/§3.2/§3.3
  prose is accurate; no design edits required.

### Governance/spec (REV)
- **None.** No ADR/REQ/spec drift surfaced. The slice adds only downward edges
  permitted by ADR-001 (verified by the fitness gate); no governance artifact
  needs to change.

### Non-reconcile notes (no write surface)
- F-1 (nit, `tolerated`): `src/slice.rs:406` comment cites "design §3.1"; should be
  "§3.2". Cosmetic; ride any future `slice.rs` touch. Not a reconcile item (code
  comment, not design/governance).
- F-2 (minor, `aligned`): `read_journal_at_ref` and `read_ledger::<Journal>`
  coexist by design §3.3's narrow scope. No action.

**Handoff:** the reconciliation brief is empty of write-surface work — design and
governance already tell the truth. `/reconcile` confirms no-op truth and hands to
`/close`. Close step-3a (`dispatch sync --integrate --trunk refs/heads/main`) must
run before `reconcile → done`, or this slice's own gate refuses the transition.