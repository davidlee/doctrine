# Review RV-288 — reconciliation of SL-223

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** SL-223 — Publication seam (RFC-021 C1 / Contract B; PRD-017,
SPEC-026, ADR-019 root). In-tree self-audit from the `edge` parent tree
(non-dispatched slice; no candidate branch). Facet `reconciliation`.

**Reviewed surface.** The four completed phases as landed on `edge`, shipped in
the v0.25.3 release. Evidence: `slice conformance SL-223`, `check gate` (green),
and direct read of the delivered command-tier surface against the locked design.

**Lines of attack (invariants held):**
1. **Design-delta honesty** — the D-A reversal (RV-287 F-4) must be the *minimal*
   resolution: `publication validate` ships as the production consumer forced by
   `deny(unused)`, stays a *publication*-tier verb, and does not smuggle in the
   deferred `library list|tree|show` UX.
2. **Dead-field discipline** — `deny(unused)` flags never-read struct fields; every
   parsed `PublicationEntry` field (esp. `kind`, `backing`) must be read on a
   non-test path.
3. **Layering** — every new module edge downward, no cycle, no tangle-baseline.
4. **Conformance** — declared design-targets vs. what git actually touched.
5. **Coverage honesty** — §9's met/partial/pending classification must be recorded,
   not overclaimed.

## Synthesis

The slice is **clean**. `check gate` is green; the design-delta claims all verify
against the tree:

- **D-A reversal is minimal and honest.** `run_publication_validate`
  (`src/commands/publication.rs`) is the sole production consumer that makes the
  `publication` engine's `load`→`resolve`→`emit` API and every error variant
  reachable under `deny(unused)`. It is a `PublicationCommand::Validate`
  publication-tier verb emitting to `io::sink()` for the licence/resolvability
  gate — it surfaces no asset bytes; `library list|tree|show` stay deferred
  (REQ-375 pending). This is the compile-forced minimum, not scope creep.
- **Dead-field discipline holds.** `report_line()` reads every parsed field —
  address, backing, **kind**, title, licence, provenance, customization — on the
  non-test validate path. `kind` and `backing` (the fields the plan flagged) are
  both covered.
- **Layering is downward.** ADR-001 layering.toml registers `asset_source`=leaf
  (out=0), `publication`=engine→asset_source(leaf), and `commands`→publication
  inherits command tier. No new upward edge; no tangle-baseline needed. The
  pre-existing `install::asset_text` upward wart is now a *delegating shim*
  (layering.toml:180 documents the retirement path).

**Standing risks / consciously-accepted tradeoffs:**
- **Coverage is partial by design.** Only the mechanism requirements
  (REQ-374/376/379/380) are *met*; the PRD user-facing invariants
  (REQ-359/363/367/369/373/381) are foundation/partial pending the deferred
  `library` verbs and the runtime-loaded adapter (OQ-4). Recorded honestly at
  reconcile — F-3.
- **Selector registry lag.** The design's §6 selector set predates the D-A
  reversal and never grew to cover the command tier that reversal shipped, so
  conformance flags 5 real deliverables undeclared. A registry/mirror edit, not a
  code defect — F-1.
- **Release cut inside the phase window.** David cut v0.25.3 before the PHASE-04
  completed-flip, so the version-bump commit rode into the phase conformance delta
  (F-2, aligned). The released binary *is* the audited binary — VA-1 (host nix
  build + artifact probe) was satisfied by that full release. Cosmetic
  delta-widening, no correctness impact.

No blocker, no major. The two `verified` findings are reconcile actions (registry +
coverage); the one `aligned` finding needs no change. Ledger `done · await=none`.

## Reconciliation Brief

### Per-slice (direct edit)
- **F-1 — selector registry + design §6 mirror.** Load-bearing change is the
  selector registry (`slice-223.toml`), which is what `slice conformance` reads —
  a prose-only §6 edit would leave conformance red. Run
  `doctrine slice selector add SL-223 <path> --intent design-target` for each of
  `src/commands/publication.rs`, `src/commands/cli.rs`, `src/commands/mod.rs`,
  `src/commands/guard.rs`, `tests/e2e_publication.rs`. Then mirror the additions in
  design.md §6 (the human-readable design-target list), citing the D-A reversal as
  the reason the command tier joined the surface. Re-run `slice conformance SL-223`
  to confirm undeclared drops to the F-2 noise only.

### Governance/spec (coverage record)
- **F-3 — requirement coverage.** Design §9 already carries the honest RV-287 F-5
  classification; record it. `doctrine coverage record` per requirement at its §9
  degree:
  - **met** — REQ-374, REQ-376, REQ-379, REQ-380.
  - **partial / foundation** — REQ-359, REQ-363, REQ-367, REQ-369, REQ-373
    (embedded covered, runtime-loaded pending OQ-4), REQ-381 (seam foundation +
    no-write proven).
  - **pending (later slices)** — REQ-375, REQ-377, REQ-378, REQ-382, REQ-383.
  Do not mark any partial requirement met — the user-facing outcomes pend the
  deferred `library` verbs.

_No plan-criterion (`EN-/EX-/VT-`, immutable-append) edits in this brief; no
governance ADR/REV changes — the design's decisions (D-A…D-F) already integrated
RV-286/RV-287 and stand._

## Reconciliation Outcome

### Direct edits applied (per-slice)
- **F-1 — selector registry (load-bearing).** `slice selector add SL-223` at
  intent=design-target for `src/commands/publication.rs`, `src/commands/cli.rs`,
  `src/commands/mod.rs`, `src/commands/guard.rs`, `tests/e2e_publication.rs` (5
  upserted). `slice conformance SL-223` re-run: conformant 9→14, all five moved
  out of undeclared; the remaining 8 undeclared are exactly the F-2 release/workflow
  noise (v0.25.3 bump JSONs + Cargo.lock, RFC-011 case-notes, the slice's own
  metadata). Registry clean.
- **F-1 mirror — design.md §5.1.** The `publication validate` command-handler bullet
  now names the concrete design-target paths (handler / dispatch+variant /
  registration / guard allow / e2e), mirroring the registry.

### Coverage recorded (F-3)
Per design §9 (RV-287 F-5 honest classification), VA attestations under SL-223,
change SL-223:
- **verified** (met — full acceptance exercised): REQ-374, REQ-376, REQ-379, REQ-380.
- **in-progress** (foundation/partial — mechanism landed, user-facing outcome pends
  the deferred `library` verbs / runtime adapter OQ-4): REQ-359, REQ-363, REQ-367,
  REQ-369, REQ-373, REQ-381.
- **not recorded** (SL-223 contributes nothing): REQ-375, REQ-377, REQ-378,
  REQ-382, REQ-383 — later slices.

_Coverage note:_ `coverage show` reports the met cells as `observed=stale /
Indeterminate` against authored SPEC-026 status `pending`. This is by design, not a
defect — observed coverage is decoupled from `ReqStatus` (NF-001 / ADR-009 §3); the
substrate is correctly surfacing that the authored requirement statuses have not
been moved. Moving them is a governance call deliberately out of the audit brief
(the PRD invariants stay pending until the library slice closes their user-facing
outcome). VA attestations drift `stale` as the tree advances — re-attest or wire VT
recipes when the library slice lands.

### Withdrawn / tolerated
- **F-2 (aligned)** — no write. The remaining undeclared paths are correctly
  unclaimed release/workflow churn.

Reconcile pass complete — every brief item resolved, no REV needed (both surfaces
were slice-side: selector registry + coverage substrate). Handoff to /close.
