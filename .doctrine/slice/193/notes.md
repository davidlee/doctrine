# SL-193 — implementation notes

## Outcome

Exposed hymn slots no longer double-emit. The projector (`src/install.rs`
`project_starters`) now emits a self-`replaces` sidecar (`.toml`) beside each
exposed starter (`.md`), wired as an install forward-step. The sidecar's
self-`replaces` suppresses the framework twin at resolve → single emit. Engine
(`src/hymns.rs`) is untouched in production (test-only expose/seal symmetry
goldens); REV-019's projector-only thesis holds.

## Durable facts (captured elsewhere — not re-stated here)

- **`prompt explain` vs `prompt resolve`.** `explain` prints the *raw ranked
  active set* (both twins, provenance-ordered) — a **pre-suppression
  diagnostic**; only `resolve` applies the `replaces` suppression graph.
  `prompt check` verifies legality. Captured in doctrine memory
  (`mem.fact.hymns.explain-vs-resolve-suppression`). This was the F-1 finding:
  the design PROSE originally named `explain` as the demonstrator; corrected at
  reconcile.
- **F5 idempotence.** Forward-step projection must run **early** (before the
  step-3 agent-render loop consumes the disk corpus), else a single `install`
  pass doubles (ISS-206 regression). Captured in design.md §Adversarial review F5.

## Standing gaps (consciously accepted, REV-019)

- Hand-authored-no-sidecar exposed snippet still doubles (explicit non-goal).
- Sidecar repair: deleting the `replaces` line from a projected `.toml` yields a
  permanently doubling slot the write-if-absent projector won't repair; `prompt
  check` surfaces it. Accepted flip side of rejecting always-clobber (D2/iii).

## Closure

- RV-239 reconciliation: 3 findings, all terminal. F-1 (verb correction) applied
  to design.md; F-2 (conformance undeclared) resolved by landing the bundle's
  selector; F-3 gate confirmations, no remediation. No REV surface.
- Landed on edge as the genuine delta of impl bundle `fadaef5e` (`review/193`)
  via cherry-pick — the bundle was base-drifted, so only its own 10-file patch
  was taken, not a whole-tree integration.
