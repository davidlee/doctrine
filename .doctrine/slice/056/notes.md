# SL-056 — implementation notes

Durable cross-phase facts harvested as phases land. Runtime progress lives in the
gitignored phase sheets (`phases/phase-NN.{toml,md}`); this file holds what must
survive a handover.

## PHASE-01 — G1+G3 governance gate (done)

**G1 — ADR-008 accepted.** Revised → `accepted` (was `proposed`). Three folds:
- §5.1 false-green/false-red evidence cluster folded into Context (validates D-B1
  empirically): shared-target false-green (touch+rerun), worktree-removal false-red
  (stale `CARGO_MANIFEST_DIR`), debug-vs-release timing.
- D-B2 sharpened to a **standing structural fact** — flake ro-binds
  `~/.cargo/bin/doctrine` ⇒ in-jail `cargo install` is structurally impossible ⇒
  **no install race because no install** (cannot regress).
- D-B3 re-scoped around the **nested-userns feasibility question (OQ-2)**, not cost:
  the gate is whether unprivileged userns creation works in-jail (`bwrap
  --unshare-user --ro-bind / / true`), since the outer bwrap may seccomp-block
  `clone(CLONE_NEWUSER)`. codex/pi-only; ro-binds the marker only, never
  `settings.local.json`. Also reframed: build isolation is a perf/false-green
  concern, **not a trust signal** — trust rides the marker + import belt.

**G3 — the spawn-interface ADR is `ADR-011`** (slug `spawn-interface-agnostic-contract`).
**Cite ADR-011 by this id in PHASE-03 (G2) and PHASE-04 (G4).** It states the
harness-agnostic **contract** (D1 marker identity, D2 orchestrator owns
fork-or-mark+provision+per-wt-env-emission) + the **per-harness altitude table**
(D3) + codex/pi enhancement layering (D4) + the charge-2 base-pinning residual (D5)
+ the **φ two-valued claude fail-closed cell** (D6, marked `proposed`) + the **σ
blast-radius confession** (D7). It **references ADR-006, does not amend it** (that
is G2/PHASE-03, post-spike).

**Status discipline:** ADR-011 stays `proposed` — its φ cell (D6) is O3-spike-
contingent. PHASE-03 (G2) firms the φ cell to the PHASE-02 spike outcome and is the
phase that promotes ADR-011 toward `accepted`.

**VH-1 gate:** PHASE-01 exit requires the **design owner to accept** the ADR-008
transition and the ADR-011 draft before PHASE-02 (the O3 spike) begins.
