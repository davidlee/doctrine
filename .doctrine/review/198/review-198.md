# Review RV-198 — reconciliation of SL-179

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reviewed surface: the **main tree** (SL-179 was solo-executed, not dispatched —
no candidate interaction branch). Facet `reconciliation`, self-audit (`--as`).

Lines of attack:
1. **Conformance algebra** — does the code touched match `design.md`'s
   design-targets? Run `slice conformance`; treat undeclared/undelivered as leads.
2. **Governance landed first (canon: no code ahead of governance).** Did the D8
   REV (REV-017) actually narrow SPEC-002 D8 + REQ-113 to the four narrowings, and
   are they internally consistent with design D3/D4? (PHASE-01 VA-1.)
3. **The two leaks the slice exists to close** are demonstrably closed *on the
   candidate binary*, not just in unit tests — the deferred VA dogfoods:
   - forget refuses a live Failed/Blocked cell, no mutation (PHASE-04 VA-1);
   - `reconcile→done` hard-refuses a live Failed cell on a gate-set req and is
     **not** accept-dischargeable (PHASE-03 VA-1, codex M8 non-vacuous dogfood).
4. **NF-001 verdict-independence wall** — `reconcile::select_status` still takes
   no `Verdict`/`Composite` after the reason split (PHASE-03 VA-2).
5. Gate green (`check gate`), behaviour-preservation (lag-discharge tests
   unchanged — design adversarial F2).

## Synthesis

**Outcome: clean audit, hand to /reconcile. No blockers, no spec/governance
remediation outstanding.** Both leaks are closed and demonstrated end-to-end on
the candidate binary; governance was reconciled ahead of the code in PHASE-01.

### Conformance
`slice conformance SL-179`: 4 conformant src paths (coverage.rs, coverage_store.rs,
reconcile.rs, slice.rs — all design-targets matched), 6 undeclared, 1 undelivered.
Both deltas dispositioned **aligned** (F-1, F-2):

- **F-1 (undelivered `coverage_view.rs`).** Design §4.3/PHASE-02 declared a render
  edit; none landed. The reason-split is absorbed by existing delegation —
  `coverage_view.rs:393` `Verdict::Divergent(r) => Some(r.label())` auto-renders the
  new `observed-failure`/`observed-blocked` labels, and `observed_state()` keeps the
  combined `any_failed_or_blocked` health summary. §4.3's own conditional ("keep
  combined if coarse; sharpen the verdict cell") is satisfied **unedited**. The
  stale design declaration is the only residue → optional one-line design tidy.
- **F-2 (6 undeclared paths).** All PHASE-01 governance: REV-017 (×3), SPEC-002 D8
  prose, REQ-113, and the `slice-179.toml [gate].extra_reqs` seed (codex M8).
  Undeclared only because conformance tracks code path-selectors, not authored
  governance. No scope creep.

### Governance (PHASE-01 VA-1 — MET)
SPEC-002 D8 (`spec-002.md` D8 + FR-006/REQ-113) carries all four narrowings —
Failed un-acceptable, VH-Blocked bar (fresh VH `Verified` + REC citing both keys),
withdrawal-as-recorded-act (D4), and `done`-gated-not-`abandoned` (codex M6) —
internally consistent with design D3/D4. No residual "all residual drift uniformly
acceptable" wording remains. REV-017 is `done`/`approved`.

### The deferred VA dogfoods, executed on the candidate `./target/debug/doctrine`
Method: seeded a VT cell on REQ-113 with a failing check (`--command false`,
unmatched matcher), `coverage verify` derived it to **Failed** (`Divergent:
observed-failure` — the label-split renders live). Then:

- **PHASE-04 VA-1 — MET.** `coverage forget` of the Failed cell refused (exit 1),
  named the 4-tuple + status + four recorded remedies (verify / record / withdraw
  REC / hand-edit), and the cell **remained** (no remove, no save — the atomic
  refuse-before-mutate path).
- **PHASE-03 VA-1 — MET.** `slice status SL-179 done` (from `reconcile`) refused,
  citing `REQ-113 (authored: active)` under the per-reason header *"Failed coverage
  cell — a check ran and contradicted (not accept-dischargeable)"* with the
  mode-aware remedy. Non-vacuous dogfood (codex M8): the seeded cell is a declared
  gate-set req and the candidate binary's own close edge bit. Refused ⇒ no
  transition; SL-179 held at `reconcile`. Scratch cell then removed (the untracked
  `coverage.toml` I created), REQ-113 returned to its real state — tree clean.
- **PHASE-03 VA-2 — MET** (confirmed at PHASE-03): `reconcile::select_status(to,
  prior)` takes no `Verdict`/`Composite` — the NF-001 verdict-independence wall
  stands after the reason split; gate coverage reads are refuse-only.

### Standing risks / accepted tradeoffs
- The positive half of the close dogfood ("close passes once the cell clears") is
  **not** flipped here — actually transitioning SL-179→`done` is /close's job after
  /reconcile. Cleared-state was confirmed by the Failed cell's removal (gate no
  longer cites REQ-113); the real `done` happens at /close. Conscious, not a gap.
- `check gate` green (lint+test+fmt+build); all four phases' VT suites pass; the
  lag-discharge anchors (vt4/vt5/vt6 + close-integration) stayed green **unchanged**
  — the hard-refuse *added* tests rather than flipping them (design adversarial F2,
  behaviour-preservation proof).

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §4.3 (`coverage_view.rs`) — optional, minor.** The section declares a
  render edit that never landed because the `Verdict::Divergent(r) => r.label()`
  delegation absorbed the reason-split (F-1). Tidy the prose to record that
  `coverage_view.rs` needed no change — the combined health summary +
  reason-label delegation already render the split. Purely a design-accuracy note;
  no behaviour implication.

### Governance/spec (REV)
- **None.** Governance was reconciled ahead of the code in PHASE-01: REV-017
  (`done`/`approved`) already narrowed SPEC-002 D8 + REQ-113, and the audit (F-2,
  PHASE-01 VA-1) confirms they carry the four narrowings consistently. No further
  REV is owed by this audit.
