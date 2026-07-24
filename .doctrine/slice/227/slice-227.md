# Library read surface and minimal projection

## Context

RFC-021 Claim **C1** (semantic ownership + minimal projection) is governed and
rooted on **ADR-019** (embedding ≠ publication ≠ projection, seven independent
asset properties). C1 lands as two complementary contracts:

- **Contract A — minimal projection.** Install provisions a justified minimal
  base and leaves the rest framework-owned. Its *spec prose* landed under
  **REV-028** (SPEC-009 / PRD-006), which explicitly deferred the **mechanism**
  to a later implementation slice: SPEC-009 is currently partly forward-intent.
  The shipped installer still projects ~80 files.
- **Contract B — publication / library / search.** Governed by **PRD-017 /
  SPEC-026**. **SL-223 (done)** peeled off *publication alone*: the manifest, the
  logical-address resolver (SPEC-026 FR-002, `active`), and the neutral byte-level
  asset-read seam (FR-004, `active`), shipping a **templates-only provisional**
  publication declaration. SL-223 named its two successors as later slices — the
  **library read surface** and **federated search**.

This slice delivers the **library read surface** (Contract B) *and* the
**minimal-projection install flip** (Contract A mechanism) as one coherent change.
They are separate contracts but a deliberate **pair**: minimal projection removes
~77 files from the repo; the library read surface is what restores access to them
(RFC-021 design tension — "minimal projection can reduce user control … pair it
with a stable published library"). Cutting the footprint *before* the read path
exists is the exact user-control regression the RFC forbids. Bundling them lets
the pairing be enforced **intra-slice, by phase ordering**: the read surface lands
first, the projection cut second — the read path provably exists before any file
stops landing.

**Federated search (SPEC-026 FR-005/006, NF-003/004) is explicitly out of scope**
— it is the *third* Contract B surface and its own slice (see IDE-041 sequencing).

## Scope & Objectives

**In scope — phased library-first:**

1. **Library read surface** (SPEC-026, Contract B):
   - `doctrine library list [path]` / `tree [path]` — read manifest metadata at a
     logical path (**FR-003 / REQ-375**).
   - `doctrine library show <logical-path>` — resolve an address and stream exact
     bytes to stdout; distinct reported error classes for missing source, malformed
     policy, unsupported source type, metadata-without-bytes; traversal-like
     addresses rejected; unavailable-source entries stay visible in list/tree
     (marked) and only fail on `show` (**FR-003 acceptance**).
   - Manifest as **sole authority** for the public set — embedded-but-undeclared
     asset is invisible; duplicate logical address rejected at admission
     (**FR-001 / REQ-373**).
   - No reachable write path over any protected root — read-only is structural
     (**NF-002 / REQ-381**).
   - **Settle QUE-172** (is the shipped memory corpus published-for-copy) to fix
     the manifest's real initial collection set, widening SL-223's templates-only
     stub. Licence classification (FR-007, shipped all-MIT for templates) reaches
     further as non-template collections are exposed; ASM-003's provisional
     `{customizable, fixed}` customization-status stays as-is (C2 widens it).

2. **Minimal-projection install flip** (SPEC-009, Contract A mechanism):
   - Project the **three-file base** (`.gitignore`, `doctrine.toml`,
     `boot-project.md`) from the projection policy (**FR-007 / REQ-353**).
   - Entity roots materialize **on first use** of the kind (**FR-008 / REQ-354**).
   - Hymns materialize on **first customization** (**FR-009 / REQ-355**;
     SL-193's `project_starters` seam is the anchor).
   - Standing governance materializes on **explicit user definition**
     (**FR-010 / REQ-356**).
   - **No auxiliary framework asset projected by default** — the agent / workflow /
     skill / reference / memory legs are **re-gated, not deleted**; a selected
     harness still installs its own required adapter (**NF-004 / REQ-357**).
   - Governance surface stays **distinct** from the orientation surface
     (**NF-005 / REQ-358**).

**Objective:** minimal base install with no loss of reachability — every asset the
flip stops projecting is reachable through `doctrine library` before the flip lands.

## Non-Goals

- **Federated search** (SPEC-026 FR-005/006, NF-003/004) — separate slice.
- **Existing-install migration / cleanup** — out of scope per REV-028 and
  RFC-021 non-goals (no meaningful external user base).
- **Contract C2/C3** (behaviour registry, workflow-derived activation) — gated
  behind C1; not started.
- **C2 customization-status vocabulary** — ASM-003 keeps the provisional
  `{customizable, fixed}` enum; C2 widens it later.
- The library `serve`/HTTP surface, write/materialize verbs, or copy-into-repo —
  library is strictly read-only here.

## Affected Surface

- `src/install.rs` (~4067 lines) — the projection flip: `build_plan`,
  `run_forward_steps` legs (agents / workflows / hymns / skills / reference /
  memory), the base projection set. The dominant, riskiest half.
- `src/asset_source.rs` — the neutral byte-read seam SL-223 landed; the library
  reads through it (FR-004, already `active`).
- Publication manifest + logical-address resolver (SL-223) — the library veneer's
  substrate; extend the manifest's collection set per QUE-172.
- New `doctrine library` command module (`list | tree | show`) + CLI wiring.
- `install/` embedded assets + `manifest.toml` — projection-policy declaration.
- Tests: existing install tests assert today's ~80-file projection — they change
  with the flip (behaviour-change gate, not a silent break).

## Risks / Assumptions / Open Questions

- **R1 — install refactor regression.** The flip rewrites projection behaviour in
  a large module; every existing install test encodes the current set. Re-gate the
  auxiliary legs (harness adapter must survive), don't delete. Highest-risk phase.
- **R2 — FR-008 first-use-root seam is cross-cutting.** Today roots are
  `create_dir_all`'d ad hoc at each write; there is no unified "first use of kind
  materializes root" contract. Most writes already guard the parent, so the seam
  may be largely satisfiable by a single `ensure_root` chokepoint — verify at
  design. Candidate for its own phase.
- **R3 — RustEmbed re-embed lag** (`mem_019e9a21…`, `mem_019e98a7…`): a lone
  manifest/template edit is invisible until the embedding crate recompiles; a new
  authored entity root needs a manifest dir + gitignore negation
  (`mem_019e9796…`). Carry into design/execute.
- **R4 — QUE-172 must settle** before the manifest's real collection set is fixed;
  it blocks the library's initial public set and how far licence classification
  reaches. Settle within phase 1.
- **A1 — pairing by phase order is sufficient** to honour the RFC's "read path
  before the cut" invariant; no cross-slice ordering needed.
- **OQ-1 — IDE-041 (search sequencing)** stays open and out of scope; this slice
  registers the library as a future search provider but does not build federation.

## Verification / Closure Intent

- **Library:** `library list|tree|show` behaviour per FR-003 acceptance —
  round-trip a published asset, byte-exact `show`, the four distinct error
  classes, traversal rejection, unavailable-entry-visible-but-show-fails. Manifest
  sole-authority (undeclared asset invisible; duplicate-address rejected). NF-002
  no-write-path proven structural over every protected root.
- **Install flip:** a fresh install lands exactly the three-file base (FR-007);
  no auxiliary asset projected by default (NF-004); entity roots appear on first
  use (FR-008); harness adapter still installs; governance distinct from
  orientation (NF-005). **No-silent-unreachable gate:** every asset the flip stops
  projecting resolves through `doctrine library show` first.
- Closure gates on SPEC-009 (FR-007–010, NF-004/005) and SPEC-026 (FR-001, FR-003,
  NF-002) coverage reconciled; QUE-172 settled.

## Follow-Ups

- Federated search slice (SPEC-026 FR-005/006, NF-003/004) — resolves IDE-041.
