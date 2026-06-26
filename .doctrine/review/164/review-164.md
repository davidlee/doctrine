# Review RV-164 — reconciliation of SL-156

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject surface.** Solo-fork slice (not dispatched). Reviewed on `edge` after
landing the fork `slice/SL-156-cargo-isolation` (tip a8c55580 + a key-alias fix)
via `git merge --no-ff` → merge commit `79b1ee21`. Review verbs refuse inside a
worktree fork (IMP-024), so audit runs from the primary tree; the fork's runtime
phase sheets were copied into the primary runtime state so conformance reads the
completed phases.

**What this audit probes** (design.md is canonical; ADR-008 + REV-011 + POL-002
govern):

1. **Mechanism conformance (B1).** Did the slice retire the shared
   `CARGO_TARGET_DIR` and make the platform build-tool-agnostic, per §5.1–5.2?
   Code surfaces (`flake.nix`, `src/worktree/{fork,coordinate,gc,mod,provision}.rs`,
   the two SKILLs, AGENTS.md, justfile, the env-contract test blocks) all show
   **conformant** in `slice conformance` — verify they match §5.2 intent.
2. **Path-conformance algebra.** `slice conformance 156`: 16 undeclared / 2
   undelivered / 11 conformant. Run down every undeclared/undelivered cell — is
   it scope creep, a stale selector, or a missed design update?
3. **POL-002 (VA, EAP-3 scope).** No `CARGO_TARGET_DIR`/cargo literal load-bears
   in the touched platform surfaces.
4. **Behaviour-preservation (EAP-1).** Creation/marking/coordination assertions
   stay green; only env-contract assertion blocks excised.
5. **Deferred verification (R5).** VT-1/VT-2/VH-1 are launch-time — the flake
   change is inert pre-relaunch (jail has no nix); in-session proof is the
   `.env_remove` simulation + the fork's recorded green gate. The final
   `<wt>/target` semantics + REV-011 application are reconcile/close work.
6. **EX-3 memory triage + EX-1/2 ritual retirement** delivered per plan.

**Invariants pinned:** isolation-by-construction (no shared export ⇒ no shared
target); platform exits the build-env business (no env contract on any stdout);
the close gate is REV-011 (ADR-008 D-B1/D-B5 amendment) applied at reconcile +
VH-1 discharged post-relaunch — routed through the reconciliation brief, NOT an
RV blocker (a blocker would refuse the audit→reconcile transition the brief needs).

**Where bodies were likely buried (confirmed):** the design-target selectors cite
`.agents/skills/…` but the repo uses `plugins/doctrine/skills/…` (SL-152 plugin
migration) — the sole cause of the 2 undelivered + 2 undeclared SKILL cells; a
PHASE-04 memory-triage deliverable (the successor's key-alias symlink) was
committed without its alias; the `src/ledger.rs` rustfmt drift and historical
footgun references (IMP-004 et al.) are flagged-for-reconcile carry-overs.

## Synthesis

**Verdict: conformant, reconcile-ready. No blocker.** SL-156 delivers B1 as
designed — the platform exits the build-env business and per-worktree isolation
becomes correct-by-construction from the *absence* of a shared `CARGO_TARGET_DIR`.
The 11 conformant cells (`flake.nix`, `src/worktree/{fork,coordinate,gc,mod,
provision}.rs`, both SKILLs, AGENTS.md, justfile, the three env-contract test
suites) match design §5.2 surface-for-surface; PHASE-03 notes show the
behaviour-preservation gate held at assertion granularity (EAP-1) — creation/
marking/coordination assertions green, only env-contract blocks excised — and the
gc target-base scaffold removed wholesale.

**The close story.** Every one of the 7 findings is verified-terminal. None is a
code defect in the shipped mechanism; they cluster into three buckets:

1. **Selector/prose hygiene (F-1, F-2)** — the design-target selectors lag the
   SL-152 skill relocation (`.agents/skills/` → `plugins/doctrine/skills/`) and
   omit the memory-triage path. The *work* is correct and design-anticipated; the
   *conformance algebra noise* (4 SKILL cells + 11 memory cells) is entirely
   explained by stale selectors. Reconcile fixes the selectors; no code moves.
2. **In-audit fixes (F-3, F-5)** — a missing key-alias symlink (would have broken
   `memory show/retrieve` by key) committed during the land; a notes-completeness
   line for the `worktree/SKILL.md` edit. Both remediated.
3. **Reconcile-delegated substance (F-4, F-7)** — the real gate: REV-011 (ADR-008
   D-B1/D-B5 mechanism amendment) approve+apply, VH-1 final-semantics discharge
   post jail-relaunch, and AP-5 relation triage (SL-156 ↔ IMP-004).

**Standing risks / consciously accepted tradeoffs.**

- **R5 deferral (the load-bearing one).** The flake `set-env` removal is
  launch-time; this session still inherits the shared jail `CARGO_TARGET_DIR`
  (confirmed `=/home/david/.cargo/doctrine-target-jail`), and nix cannot eval in
  the jail. So VT-1/VT-2/VH-1 are *honestly* unproven in-session — the proof is
  the `.env_remove` simulation (env-absent fallback path) + the fork's recorded
  green gate. The final `<wt>/target` shape is VH-1, owed at reconcile/close after
  a relaunch + one-time cold rebuild (old `doctrine-target-jail` abandoned).
- **Cold-fork builds (R1/OQ-1).** Accepted; sccache (D-B4) is the deferred lever
  if it bites. Not a correctness risk.
- **`src/ledger.rs` drift (F-6).** Pre-existing SL-154 noise, tolerated, out of
  scope.
- **Process note (not a slice defect).** Audit could not run inside the fork
  (IMP-024 — review baton lives in the primary tree's gitignored state). Resolved
  by landing the fork to edge (`--no-ff` → 79b1ee21) and copying the fork's
  runtime phase sheets into the primary runtime so conformance reads completed
  phases. The deferred-to-close landing posture and the parent-tree-only review
  constraint are in tension for solo forks; landing at audit (work complete) is
  the clean resolution. edge→main promotion still happens at close.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §5.2 + design-target selectors (F-1):** rewrite the two SKILL paths
  `.agents/skills/{dispatch-subprocess,worktree}/SKILL.md` →
  `plugins/doctrine/skills/{dispatch-subprocess,worktree}/SKILL.md` so
  `slice conformance 156` reads the SKILL edits as conformant (clears 2 undelivered
  + 2 undeclared).
- **design-target selectors (F-2):** extend selectors to cover
  `.doctrine/memory/items/` (the EX-3 triage surface), or consciously accept it as
  authored-progress noise. Low-value housekeeping — reconcile's call.

### Governance/spec (REV)
- **ADR-008 D-B1 + D-B5 via REV-011 (F-4):** approve + apply REV-011 — D-B1
  mechanism changes from "platform sets per-worktree `CARGO_TARGET_DIR` at spawn"
  to "no shared export; cargo defaults to in-tree `<worktree>/target`" (Amendment
  1); D-B5 — the flake loses the export, the removal *is* the mechanism (Amendment
  2); POL-002 is the forcing function. D-B2/D-B3 unchanged; D-B4 (sccache) gains
  relevance as the warm-fork-cache lever. The ADR edit is real only once the code
  lands (now landed on edge).
- **VH-1 discharge (F-4):** after jail relaunch, confirm two worktrees build
  distinct `<wt>/target` binaries their own e2e tests spawn (VT-1) and both arms
  verify honestly (VT-2). One-time: remove the abandoned
  `~/.cargo/doctrine-target-jail`.

### Relations (design AP-5)
- **SL-156 ↔ IMP-004 (F-7):** relate SL-156 to IMP-004 (the open shared-target
  artifact backlog item) and assess whether in-tree-per-worktree target *resolves*
  IMP-004 (status move) or narrows it. Historical footgun references
  (slice-152/104/080/073/127, backlog-004, rfc-005, review-158) stay intact —
  they record the world as it was.

### Out of scope (flag onward)
- **F-6:** `src/ledger.rs` rustfmt drift → SL-154.

## Reconciliation Outcome

### Direct edits applied
- **Selectors (F-1):** removed the three stale `.agents/skills/{dispatch-subprocess,
  worktree,dispatch-agent}/SKILL.md` selectors; added `plugins/doctrine/skills/…`
  equivalents (design-target for the two §5.2 SKILLs + EAP-4 note restored;
  scope-relevant for dispatch-agent).
- **Selectors (F-2):** added `.doctrine/memory/items/**` (design-target) covering
  the EX-3 triage surface. Result: `slice conformance 156` now 26 conformant, 0
  undelivered, 1 undeclared (`notes.md`, expected authored-progress noise).
- **design.md §5.2 (F-1):** the two `.agents/skills/…` prose path references →
  `plugins/doctrine/skills/…`.

### REVs completed
- **REV-011 (`adr-008-d-b1d-b5-platform-exits-build-env`): done** — covers F-4.
  started → approved → applied (1 `modify ADR-008` row surfaced for manual
  landing) → landed by hand → done. ADR-008 edits: D-B1 mechanism revised
  (platform-injects-`wt/<branch>`-at-spawn → retire shared export, in-tree
  `<worktree>/target`, correct on both arms — intent preserved, Amendment 1); the
  §5.1 "claude shares the jail-wide target / mitigation rituals stand" caveat
  marked **resolved**; D-B5 "keep flake minimal" → "flake exits the build-env
  business; justfile drops stale-target rituals" (Amendment 2). Rationale in
  revision-011.md. D-B2/D-B3 unchanged; D-B4 (sccache) remains the deferred
  warm-fork-cache lever.

### Relations (F-7)
- Linked **SL-156 `related` IMP-004** (slice-156.toml; relation block contiguous).
  **Assessment:** IMP-004 is *narrowed, not resolved* — SL-156 delivers IMP-004's
  D-B1 leg (via the revised in-tree mechanism); IMP-004's D-B3 (per-worker bwrap
  confinement spike) and the D2b OS-floor work remain open. IMP-004 stays `open`.
  Historical footgun references left intact (record the world as it was).

### Deferred to /close (not reconcile writes)
- **VH-1 (F-4):** final `<wt>/target` semantics on both arms — discharges only
  after a **jail relaunch** ("apply & reset"; flake set-env is launch-time, nix
  cannot eval in-jail, R5). In-session evidence: `.env_remove` simulation + fork's
  recorded green gate. One-time post-relaunch: remove the abandoned
  `~/.cargo/doctrine-target-jail`. This is the slice's remaining close-gate.

### Out of scope (flagged onward)
- **F-6:** `src/ledger.rs` rustfmt drift → CHR-027 (SL-154 domain), tolerated.

Reconcile pass complete — every brief item resolved (REV-011 done, direct edits
applied, relation linked). Handoff to /close; the sole open close-gate is VH-1,
which requires the user's jail relaunch.
