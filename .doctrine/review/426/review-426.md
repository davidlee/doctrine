# Review RV-426 — reconciliation of SL-285

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Surface: edge after `worktree land --fork sl-285` (98b7907cd); solo fork, not a
dispatch candidate. Conformance mode.

Lines of attack:
- Code vs design.md (rev 31) sec-1..6, DEC-393/396/397/399/401/402, and notes
  § Reconcile carry (binding deltas decided in-flight, incl. QUE-229 re-tier).
- Every decision consequence has an exercising test (fatal legs, refusals,
  drift, disclosure lines).
- `slice conformance` cells: undeclared paths are scope creep or missing selectors.
- PHASE-01..03 EX/VT/VA/VH met; `doctrine check gate` green on edge.

Evidence: pi `review` (max) conformance pass over the fork (session 6babff1f);
its notes.md view predated the edge-side carry, so claims resting on notes were
re-checked against edge.

## Synthesis

SL-285 makes provisioning a fork two steps: copy the allowlist, then
regenerate the derived tier (skills, agent defs, harness links and config,
shipped memory) from the source through `install::regenerate`. Every
creation path (fork, create, coordinate, the clean dispatch candidate) and
repair (`worktree provision`) goes through one `ProvisionMode` seam. The
conflicted candidate is copy-only and says so.

Code conforms to design sec-1..6 and DEC-393/396/397/399/401/402 as amended
by notes § Reconcile carry. No blocker. Gate green on edge after landing
(98b7907cd). PHASE-01..03 EX/VT met; `slice verify-vt` passes; VH-1
confirmed by the user. Every new e2e was negative-controlled (regeneration
disabled → all red; `kept` row suppressed → red).

Findings: three untested decision consequences (F-1..F-3), fixed with
negative-controlled tests on sl-285. The design text lags the code
(F-4..F-6), which goes to reconcile. One pre-existing silent skip became
load-bearing (F-7), captured as ISS-532.

Risks accepted:
- A refused leg (malformed managed block, printed-fallback hook, foreign pi
  extension) fails provisioning and rolls back creation, where `install`
  only degrades. Decided in-flight (carry); the cost is a harder unhappy
  path in exchange for never handing over a half-wired fork.
- Repair over a dirty tracked file that a leg would rewrite refuses (byte
  drift guard). Deliberate: protects uncommitted edits.
- Ignored harness config bakes the minting binary's absolute path (DEC-402,
  F-7 observation): it resolves in the Linux jail because the jail binds `/`
  read-only. Rebuilding or deleting the minting tree changes the fork's
  codex hooks. The macOS arm was not probed.
- The shipped managed `.gitignore` block lacks `.codex/` and
  `.doctrine/slice/*/research/` for new projects (ISS-531, tagged next).
  The clean-fork guarantee holds only where the project ignores `/.codex`.

## Reconciliation Brief

### Per-slice (direct edit)
- F-4 (load-bearing): `doctrine slice selector add` design-target selectors
  for src/boot.rs, src/git.rs, src/worktree/create.rs,
  src/worktree/coordinate.rs, tests/e2e_dispatch_candidate.rs,
  tests/e2e_worktree_coordinate.rs, tests/e2e_worktree_create_fork.rs;
  re-run `slice conformance 285` to undeclared = 0. Mirror them in design
  sec-4's file/test list, and add boot.rs, git.rs, create.rs and
  coordinate.rs to slice-285.md § Affected surface.
- F-5: integrate notes § Reconcile carry into design.md:
  - sec-1 not-regenerated list, sec-3, sec-5: uninstalled source → copy
    only, print `derived tier not regenerated — doctrine not installed in
    <source>`, exit 0 (creation and repair).
  - sec-3 downgrade bullet: an absent destination stamp is not a downgrade.
  - sec-3 disclosure: regeneration progress (including boot wiring via
    `boot::apply_wire_into`) goes to stderr for provisioning callers;
    machine-read stdout of fork / coordinate / candidate create unchanged.
  - sec-3 refusals: four refusals before any write (unreadable source
    stamp, unreadable destination stamp, downgrade, stale wiring), not two.
  - sec-3 tracked-drift bullet: compares bytes; rewriting or reverting a
    pre-dirty tracked file is drift.
  - sec-3 fatal legs: a `Refused` leg halts like `Failed`.
  - sec-4 / sec-6: worktree::provision and worktree::fork re-tiered
    engine → command in ADR-001 layering.toml, same shape as
    worktree::coordinate; tangle baseline stays command = 76 (QUE-229
    answered: keep). Replaces sec-4's "no tier edit" / predicted +1.
  - sec-5: the clean-fork e2e fixture ignores `/.codex` itself; the
    shipped-block gap is ISS-531.
- F-6: sec-4, plan_wire is a third `WirePass::DryRunPlan` pass that
  collects touches; apply_wire and the pinned dry-run test are unchanged.

### Governance/spec (REV)
- None. The layering re-tier is a data edit inside ADR-001's mechanism
  (layering.toml), not a change to the ADR's rule.

## Reconciliation Outcome

### Direct edits applied
- Selector registry (F-4): 7 design-target selectors added (boot.rs, git.rs,
  worktree/create.rs, worktree/coordinate.rs, e2e_dispatch_candidate.rs,
  e2e_worktree_coordinate.rs, e2e_worktree_create_fork.rs);
  `slice conformance 285` undeclared = 0. Mirrored in design sec-4 and
  slice-285.md § Affected surface.
- design.md (F-5): notes § Reconcile carry integrated — uninstalled source
  (sec-1/3/5), absent destination stamp not a downgrade, four pre-write
  refusals, byte-level drift, `Refused` halts, stderr progress, layering
  re-tier with baseline 76 (sec-4/6), `/.codex` fixture and ISS-531 (sec-5).
- design.md (F-6): sec-4 `plan_wire` as the third `WirePass::DryRunPlan` pass.
- design.md sec-6: baked-binary-path risk updated with the PHASE-03 F-7
  observation (resolves in the Linux jail; residual stated).
- DEC-399, DEC-401: choice amended per the carry (user agreed 2026-10-23).
- The locked design run (rev 31) diverges from design.md by design; not
  re-adopted (reconcile direct-edit surface).

### REVs completed
- REV-075 (`reconcile-sl-285`): done — REQ-189 title and SPEC-012 § Fork
  provisioning amended to copy-then-regenerate. Not in the brief; named by
  design sec-6; user chose to land it (2026-10-23).

### Withdrawn / tolerated
- None beyond the ledger dispositions.
