# Review RV-152 — reconciliation of SL-148

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of the SL-148 cumulative impl bundle (`review/148` @ `0c11259c`
— PHASE-01..05 squashed to one src-only commit, 18 files, +1860/−63) against the
locked design (`design.md` §5.2–5.5, D1/D5/D8/D9, F-V1..F-V7) and plan
(`plan.toml` PHASE-02..05 EX/VT).

Lines of attack:
- **Behaviour gate (I3)** — does the full pre-existing suite stay green, with
  byte-identical create-command stdout (LocalFs parity, the seam enrichment must
  not move observable output)?
- **Load-bearing distributed CAS (R1/R2)** — is the zero-oid create-lease
  correct, and is the porcelain classification machine-stable (only the explicit
  lease/create rejection retries; everything else hard-errors)?
- **Structural conformance (§5.2)** — do the PHASE-03 module/seam shapes match
  design, or are deviations justified and behaviour-preserving?
- **Scope discipline** — is the PHASE-05 delta the literal default-flip, or did
  latent gaps leak in?
- **Gates** — VH-1 human acceptance; the inherited memory-sync failure's
  provenance; clean integration onto current `main`.

Review surface: the impl bundle (`review/148^..review/148`), audited at
integration by cherry-picking onto current `main` (clean, zero conflicts) and
running the full doctrine-package suite + clippy + fmt. Per-phase `phase/148-NN`
review refs were not materialised (0 cuts; boundaries.toml gitignored) — phase
granularity is carried by the 4 phase commit messages and on-disk boundaries
oids; the cumulative delta is the complete reviewable surface.

## Synthesis

**Closure story.** SL-148 ships doctrine's first remote git mutation — a `GitRef`
`Claim` backend that linearizes fresh-id allocation at a shared remote via a
zero-oid create-CAS (`push --force-with-lease=<ref>:0`), behind a reach config
(`local | shared | auto`), plus a held-claims survey (`doctrine reservation
list`). The implementation is faithful to the locked design and the adversarial
record (F-1..F-13, F-V1..F-V7): one claim loop, one enriched seam
(`ClaimCtx{dir,id}`), the named/memory path split off the trait (D9), the 11
Fresh sites routed through `reserve::backend`, and `resolve_backend` as the sole
backend selector / reachability probe / degradation decider (D8 fail-closed).

**Evidence.** Integration gate (bundle cherry-picked onto current `main`, clean):
full doctrine suite **2874 passed / 0 failed**, `cargo clippy` zero warnings,
`cargo fmt --check` clean. The load-bearing risks are independently closed: an
adversarial pass over the CAS/classification path found no defects, and
`vt1_two_clones_racing_the_same_id_do_not_collide` empirically proves
collision-freedom (I1). The behaviour gate holds — the golden create-command
e2e suite is green, so LocalFs parity / byte-identical stdout is preserved; the
degradation signal is stderr-only and one-time (observed in suite output).

**Findings (6, all terminal).** Three `aligned` (F-2 resolve_remote in-scope;
F-4 inherited failure is SL-143/base-staleness, green at integration; F-5
CAS/classification sound), one `design-wrong` (F-1: §5.2 prose stale vs three
layering-driven, behaviour-preserving as-built refinements — design amend at
reconcile), one `tolerated` (F-6: cosmetic layering.toml comment drift), one
`blocker` cleared by human acceptance (F-3: VH-1 reach=auto default accepted by
the User 2026-06-24).

**Standing risks / consciously accepted tradeoffs.**
- **Mixed-reach unsoundness (E5/R6)** — a team mixing `local` and `shared`
  clones can still collide on a branch-only-authored id; defended by a
  documented team-wide uniform-reach assumption + `validate`/`reseat` backstop,
  not in code. Accepted for v1.
- **Permanent ref accumulation (OQ-2)** — no GC; reserved-but-unauthored ids are
  harmless permanent gaps. Accepted.
- **Network e2e** — the jail forbids a network remote; correctness is proven
  against a local bare-repo substrate (R5). Network is a manual dev affordance.
- **`acquired` is client-declared** (F-12) — forgeable `GIT_COMMITTER_DATE`;
  surfaced as best-effort metadata in the survey, not a server clock.

**Pre-integrate note.** review/148's base (`2148f662`) predates current `main`'s
SL-143 fixes; the diff vs `main` shows `.doctrine/*` noise that is purely
stale-base artefact (the bundle itself is pure src). The edge→main promotion is
already landed; re-check `dispatch status --slice 148` for fresh drift and finish
the promotion before `dispatch sync --integrate --trunk refs/heads/main`.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §5.2 / EX-2 (F-1)** — amend the prose to match the as-built (code
  is correct; the deviations are layering-driven and behaviour-preserving):
  (a) reservation config + parse/load live in the `reserve` module — there is no
  separate `reservation_config` module (R9: one already-classified engine
  module, one layering.toml entry);
  (b) the D8 fallback prompt is injected as a `PromptFn` fn-pointer from the
  command tier (caller passes `install::prompt_confirm`) — `reserve` never
  imports `install`, preserving the ADR-001 engine↛command boundary;
  (c) `[reservation]` is projected inside `reserve` (its own
  `parse_reservation_config`/`ReservationDoc`) over the shared
  `dtoml::read_doctrine_toml_text` file seam, kept OFF `dtoml::DoctrineToml`
  (engine types on a leaf struct would force a leaf→engine import) — consistent
  with §5.2's own "never eagerly validated in dtoml::parse".

### Governance/spec (REV)
- **SPEC-008 (R7)** — add a prose note: the remote reservation ref class
  (`refs/doctrine/reservation/<prefix>/<NNN>`) and the new remote git ops
  (`fetch_refspec`, `push_ref_cas`, `for_each_ref`) in `git.rs`.
- **SPEC-022 (R7/F-4)** — cross-reference note: git's ref taxonomy now includes
  the remote reservation ref class + the remote push/fetch ops (previously scoped
  to local coordination/evidence refs). No conflict — PRD-005/SPEC-008 ratify it.
- **PRD-005 §6 (R7/D8 addendum)** — note the tightening: `auto` fail-closes on a
  *configured-remote* transient failure (stricter than the literal "fall back +
  signal"); the operator opts into local fallback explicitly (y/N prompt /
  `DOCTRINE_RESERVATION_FALLBACK=1` / `allow-local-fallback`). The literal §6
  fallback governs only the structurally single-tree (no-remote) case.
- **`mem.system.engine.identity-claim-seam` §2 (R8/D9)** — update: the claim seam
  is now **Fresh-numeric-only**; SL-005 D7's named+numeric unification is
  superseded (the enrichment made `ClaimCtx` numeric-shaped). Memory's directory
  claim is now an inline `fs::create_dir`-or-bail, not a `Claim` backend.
- **`.doctrine/adr/001/layering.toml` comment (F-6, optional/cosmetic)** — the
  `reserve = "engine"` inline comment "→ entity only (out=1)" undercounts the
  out-edges; reserve imports `entity`/`git`/`dtoml` (out=3). Comment-only fix.

## Reconciliation Outcome

### Direct edits applied
- **design.md §5.2** (RV-152 F-1): amended the "Reach config" paragraph
  (`reservation_config module` → `reserve` module) and added an **As-built (SL-148
  reconcile)** note recording all three layering-driven refinements — config home,
  config projection off `DoctrineToml`, and the `PromptFn` injected prompt seam.
- **mem.system.engine.identity-claim-seam §2** (RV-152 R8/D9): appended the
  supersession note — the claim seam is now Fresh-numeric-only; the named (memory)
  path inlines `fs::create_dir`-or-bail, no longer a `Claim` backend.

### REVs completed
- **REV-010** (`reconcile-sl-148`): **done**, approved. Four `modify` rows, all
  landed by hand under the authored-truth honour model. Rationale + before/after
  excerpts in revision-010.md:
  - **SPEC-008** §"Trunk-aware fork safety" (R7) — "Shipped (SL-148)" note: remote
    reservation ref class + the three `git.rs` remote ops + config-selected reach.
  - **SPEC-022** §"ref taxonomy" (R7/F-4) — added the permanent remote reservation
    ref class as a third class; widened the local-only coordination-ref framing.
  - **PRD-005** §6 "Reach selection" (F-3/D8) — recorded the fail-closed tightening:
    auto hard-errors on a configured-remote failure; literal fall-back governs only
    the no-remote single-tree case; explicit operator opt-in.
  - **ADR-001** `layering.toml` (F-6) — corrected the `reserve` out-edge comment to
    `entity/git/dtoml (out=3)`.

### Withdrawn / tolerated (no write)
- **F-2, F-4, F-5** — `aligned`; no change needed (resolve_remote in-scope;
  inherited failure not SL-148; CAS/classification sound). Rationale in dispositions.
- **F-3** — `blocker` cleared by human acceptance (User, 2026-06-24); no write.

Reconcile pass complete — every brief item resolved, no half-applied REV. Handoff
to /close.
