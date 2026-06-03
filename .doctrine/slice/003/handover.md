# slice-003 handover — engine + design-doc sibling

Brief for a fresh agent picking up **review & refinement** of slice-003's design.
The design doc ([design.md](design.md)) is written and committed; no code yet.
Your job is to adversarially review the *design*, refine it, and (if it survives)
take it toward build-ready. This file is your priors and the audit-trail home —
append your review + dispositions here, the same way slice-002 recorded its
rounds.

## What slice-003 is

Extract the kind-agnostic directory-entity machinery out of `src/slice.rs` into an
engine, shaped by **two callers of different shape**: the slice (top-level,
reserved, toml+md+symlink) and a new **design-doc sibling** (sub-artefact,
non-reserved, single prose file under an existing slice dir). First step is
lifting the inlined `mkdir` claim to a one-method `acquire` seam. It **supersedes
slice-002** (`supersedes = [2]` in slice-003.toml `[relationships]`).

Status is `proposed` — *not* flipped to `ready`; that gate is the user's. Build
sequence is design.md § 8 (seam → engine → design-doc Kind → verb), each step
green against the slice-001 suite.

## Read first, in this order

1. [design.md](design.md) — the artefact under review. The engine: `acquire` seam,
   `Kind` descriptor (fileset-as-function + optional reservation), materialise loop.
   Decisions D1–D7, open Q1–Q3.
2. [slice-003.md](slice-003.md) — the slice contract (scope, non-goals, approach).
3. `src/slice.rs` — the only built code; the engine's starting point. Note
   `reserve_create` (the mkdir inline to lift), `build_scaffold`/`Scaffold` (the
   fixed-pair struct that becomes `Fileset`), and the slice-001 test suite (the
   behaviour-preservation gate).
4. `../002/handover.md` — **your priors.** Two dispositioned rounds on the broader
   entity model + a drift-schema realignment + the consolidation direction. The
   settled calls below come from there.
5. Notes the design leans on: [reservation-spec](../../../doc/reservation-spec.md)
   § Code seam (the `acquire` invariant), [slices-spec](../../../doc/slices-spec.md)
   (on-disk shape, pure/imperative split), [entity-model](../../../doc/entity-model.md)
   (the storage rule + where this engine sits for drift/spec).

## Already settled — do not re-open without new argument

These were reasoned through over two review rounds (002/handover.md). Hold the
line unless you bring genuinely new evidence; if you re-litigate, say so and
argue it.

- **Extract against two callers, not one** (slice-002 superseded). A standalone
  one-caller refactor froze the boundary wrong; the design-doc is the corrective
  second caller.
- **Fileset is a function, not a fixed toml+md pair** (slice-002 M3). The frozen
  2-file boundary was *the* bug; `Vec<Artifact>` is the fix.
- **Only the `acquire` seam now — not the full `LeaseBackend` trait, not `git-ref`.**
  Lifting more is the over-build risk the slice names.
- **No premature shared `Meta` trait.** Parameterise file mechanics, not metadata;
  `Meta`/list stays slice-side until a second metadata-bearing reader exists.
- **Spec family = one model, three subtypes, own folders, per-subtype filesets**
  (entity-model.md). Relevant because spec is the engine's eventual fourth caller.
- **Keep deferred deferred.** drift and spec stay registry-gated; this slice only
  makes the engine *fit to host* them. A finding is not a licence to start building
  a deferred entity. Roadmap changes go via supersede (002→003), not scope creep.

## High-value review targets (fresh surface — expect the real findings here)

The engine boundary is the live build decision; it is cheaper to fix on paper now
than after extraction. Push hardest on:

- **D1 — the `acquire` seam is path-based (`&Path`), not the abstract `key: &str`.**
  The design argues the git-ref evolution is additive (second impl + split the
  dir-creation out of `acquire`). Is that true, or does path-baking leak in a way
  that *will* force a caller rewrite — exactly the F1 failure slice-002's seam was
  meant to prevent? This is the load-bearing claim; verify it, don't take it.
- **D2 — `Kind` as a data struct with a `fn` pointer vs a trait.** Does the `fn`
  signature carry everything (template reads return `Result`; sub-artefact ctx)?
  Where does a struct-of-fns crack first?
- **The `Artifact` model (File | Symlink).** Does it cover everything the slice
  does today (symlink `AlreadyExists` tolerance) and everything a spec subtype
  needs, or is it the next boundary frozen too low?
- **The non-reserved branch.** Is "resolve existing dir + refuse clobber" the
  right shape for *all* sub-artefacts, or design-doc-specific?
- **slice.rs / entity.rs split.** Is the kind-blind / slice-specific line drawn in
  the right place — especially `Meta`/`read_metas`/`format_list` staying behind?

## The bar (same discipline as 002/handover.md)

- **Verify before accepting.** A claim about the schema, a code path, or a file is
  a hypothesis — check it. Verification source: `spec-driver-schemas.local.md` at
  repo root (**gitignored — do not commit**), and the live corpus
  `~/dev/spec-driver/.spec-driver/`.
- **Re-judge severity; push back on over-claims; cheapest fix wins.** A sentence, a
  field, a renamed boundary — not a rewrite.
- **If a finding reshapes the roadmap**, do it the established way (supersede +
  banner + `supersedes`), not by silently widening this slice.

## If you touch code

Project gate: `cargo test` + `cargo clippy` (deny-level; `--all-targets` is *not*
the gate) + `cargo fmt`. Red/green/refactor. Behaviour-preserving extraction —
the slice-001 suite must stay green at every step. Update slice-003.toml
`status`/`Done` per convention.

## Audit trail

Append your review verbatim + a disposition table (`finding → call → landed-in`)
below, as 002/handover.md does. Keep the design's reasoning reconstructable.

---

# Round 1 (slice-003) — adversarial design review + disposition

A fresh agent reviewed [design.md](design.md) against `src/slice.rs`, the
reservation-spec § Code seam, the slice-001 suite, and the now-committed
`install/templates/design.md`. Verdict below; nothing here is fatal or major —
the design was already shaped by two prior rounds (002/handover.md) and survives.
This is a confirm/tighten pass. No code changed.

## Review verbatim

**Verdict: green, with reconciliations.** The load-bearing D1 claim holds; the
engine boundary is drawn in the right place; the `Fileset`/`Kind` split answers
slice-002 M3 correctly. Findings are notes-reconciliation and pseudocode
completeness, not redesign.

### D1 — the `acquire` seam (the claim to verify, not take)

The design's `acquire(&self, claim: &Path)` **diverges** from reservation-spec
§ Code seam, which writes the seam as `acquire(&self, key: &str)`. It does **not**
reintroduce the F1 hazard slice-002's seam was meant to prevent:

- F1's essence = the claim must sit behind a one-method `Won | AlreadyHeld` seam,
  not an inlined `fs::create_dir` + `ErrorKind::AlreadyExists`. The design
  satisfies this exactly (`Reservation::acquire` returning `Acquired`).
- The **Kind callers** (slice Kind, design-doc Kind) call
  `materialise(kind, reservation, ctx)`; they never touch `acquire`. A future
  seam signature change (`&Path` → abstract `&str` key) plus splitting
  dir-creation out of the claim touch only **engine internals** (the
  `materialise` loop + the backend impls), never the Kinds. The "caller rewrite"
  F1 named is therefore avoided — the seam achieves its purpose.
- The design is honest about the engine-internal cost: D1 already states the
  seam "generalises to an abstract key and the dir-creation splits out of
  `acquire` into the materialise step." That is engine churn, not caller churn.

Further, `&Path`-now is arguably *better* than the spec's `&str`-now: an
abstract-key seam today forces `LocalFs` to either (a) map a `<kind>/id/<n>` key
back to a path — i.e. know the slice layout, the coupling the spec explicitly
forbids ("the primitive is blind to meaning") — or (b) claim a separate lock key
distinct from the entity dir, diverging from today's "the dir *is* the claim"
and adding an on-disk lock tree. The design's `&Path` keeps the dir-is-claim
invariant, the slice-001 retry test unchanged, and zero new on-disk artifacts.

**The real gap:** the design silently contradicts its cited authority.
reservation-spec § Code seam currently *mandates* `key: &str` as the thing to
lift now. Either the spec is reconciled (record that the local seam lands
path-based and generalises to a key with the git-ref backend) or D1 must justify
rejecting `&str`-now. The cheapest fix is one sentence each side.

### Minor findings

- **M-loop — engine pseudocode drops the parent-dir create.** Today's
  `reserve_create` does `fs::create_dir_all(slice_root)` *before* the claim loop
  (`src/slice.rs:192`); the § 3 `materialise` pseudocode omits it. `mkdir` of
  `kind.dir/001` is non-recursive and errors `NotFound` (not `AlreadyExists` →
  no retry) if `kind.dir` is absent — the first-ever-slice case. The slice-001
  `reserve_create_writes_well_formed_slice` test catches a regression, but the
  pseudocode should show the `create_dir_all(kind.dir)` step.

- **M-nonreserved — the non-reserved branch over-claims.** "resolve existing dir
  + refuse clobber" is the right shape for *file-creating* sub-artefacts
  (design-doc, a future phase file), **not** for *row-appending* sub-artefacts
  (requirement rows → `toml_edit` mutation of an existing table, round-2 NB).
  Those are a distinct mutate-existing verb this engine does not (and should not
  yet) model. Tighten "all sub-artefacts" → "file-creating sub-artefacts." Also:
  `resolve_existing` must error if the parent slice dir is absent — name that
  not-found error, not just the clobber refusal.

- **M-clobber — clobber-policy is conflated with the `reserve` flag.** The
  symlink's `AlreadyExists`-tolerance and D7's no-clobber both live in the
  writer/branch, not in `Artifact`. Reserved writes freely (fresh dir);
  non-reserved refuses. Fine for v1, but the policy rides on `reserve: bool`
  rather than being expressed per-Artifact/per-write — flag the latent coupling
  before a kind wants the other combination. Optional forward-fit: the engine's
  File-writer doing `create_dir_all(path.parent())` future-proofs nested
  filesets (a spec subtype's ~13 files) at trivial cost.

- **M-slug — name the slug split.** `derive_slug` (pure) moving to the engine is
  correct (future top-level kinds reuse it). But slug *resolution policy*
  (use `--slug` else derive, bail on empty — `src/slice.rs:276-282`) is
  slice-Kind-specific (a design-doc has no slug) and stays CLI-side. The design
  says "derive_slug ... moves to the engine"; it should distinguish the pure
  helper (engine) from the resolution policy (slice CLI).

- **Q2 resolved.** The committed `install/templates/design.md` uses **only**
  `{{title}}` (heading `# Design: {{title}}` — no `{{id}}`/`{{date}}`/`{{slug}}`).
  Q2's "likely yes" is confirmed. The design-doc verb therefore needs exactly one
  piece of parent context — the title — read via slice-side `Meta`/`read_metas`.
  This **validates** the § 4 decision to keep `Meta` slice-side: the design-doc
  Kind consumes parent metadata that the slice already knows how to read.

- **m-template — exemplar/template divergence (note only).** The committed
  template is a richer 10-section structure (Design Problem / Current State /
  Forces / Principles / Proposed Design 5.1–5.5 / Open Questions / Decisions /
  Risks / Quality Engineering / Review Notes) — distinct from both `slice.md`
  (7-section) and the hand-authored `slice/003/design.md` exemplar (Executive
  Summary / Problem / Architecture Intent / ...). Future `heresy slice design`
  output will not resemble this design doc. Not a blocker — the exemplar predates
  the tool and is grandfathered — but recorded so the divergence is deliberate.

### Strengths (recorded, no action)

- `materialise(kind, reservation, ctx)` taking the backend as an injected
  parameter is exactly right: it gives the engine unit tests a mock seam (no
  disk) and makes the git-ref swap a CLI-side backend selection, additive to
  both engine and Kinds.
- D3 (`Vec<Artifact>`) covers slice (File+File+Symlink), drift (slice-shaped),
  and spec (~13 File) without a new variant. The boundary is no longer frozen
  too low (slice-002 M3 answered).

## Disposition

| Finding | Call | Landed in |
|---|---|---|
| D1 seam `&Path` vs spec `&str` | **Accept (keep `&Path`), reconcile spec** — F1 hazard avoided (Kinds never touch `acquire`); `&str`-now would couple the backend to layout or add a lock tree. | design.md D1 (one sentence: divergence is deliberate, generalises with git-ref); reservation-spec § Code seam (record local lands path-based) |
| M-loop parent create | **Accept** — show `create_dir_all(kind.dir)` in the § 3 loop | design.md § 3 engine loop |
| M-nonreserved over-claim | **Accept** — scope to *file-creating* sub-artefacts; name the parent-not-found error | design.md § 3 non-reserved branch; D4 |
| M-clobber coupling | **Accept (flag), defer fix** — note policy rides on `reserve`; optional `create_dir_all(parent)` forward-fit | design.md D7 / Artifact note |
| M-slug split | **Accept** — distinguish pure helper (engine) from resolution policy (CLI) | design.md § 4 |
| Q2 template tokens | **Resolved** — only `{{title}}`; validates Meta staying slice-side | design.md Q2 (close); § 4 strengthened |
| m-template divergence | **Accept (note only)** — exemplar grandfathered | this audit trail |

All findings are paper-stage edits (a sentence, a pseudocode line, a scoped
word). None blocks the build sequence (§ 8). Status stays `proposed`; the
`ready` gate remains the user's. No code touched.

## Round 1 dispositions applied (+ two author corrections)

The user accepted the round-1 dispositions and added two inputs that correct
them. Applied across design.md, slice-003.md, reservation-spec, and the template:

- **Q2 re-resolved (corrects round-1).** The design template carries the **id**,
  not title alone. Two tokens: `{{ref}}` (parent canonical id, e.g. `SL-003`) +
  `{{title}}`. `Kind` gains `prefix` (`"SL"`); `ScaffoldCtx` gains `canonical_id`;
  reserved kinds derive `ref = "{prefix}-{id:03}"`, sub-artefacts inherit the
  parent's. Template heading → `# Design {{ref}}: {{title}}`. (The padded `{{ref}}`
  token is the entity-model canonical-id form, shared across the family.)
- **Design-doc gains a TOML facet — deferred, sequence A.** The user confirmed a
  design doc carries structured data (`date`, key files / globs, governance-doc
  relationships) that entity-model.md's storage rule puts in a sister facet, not
  prose. Held to a **follow-up slice via supersede**, not built now: it keeps
  slice-003 the engine slice and holds off the second metadata-bearing reader
  ("no premature `Meta`"). Captured in design.md D5 + slice-003.md Follow-Ups so
  it is not relitigated. **Approval is slice state, not a facet field** (it gates
  planning); **structured adversarial review is a future `RVW-` entity**. Engine
  surface unaffected — a toml+md design doc is a 2-`Artifact` non-reserved fileset.
- **D1 / reservation-spec reconciled.** Kept the path-based `&Path` seam (the
  `&str`-now alternative would couple the local backend to the layout or add a
  lock tree); reservation-spec § Code seam now records that local lands
  path-based and the abstract key generalises engine-internally with `git-ref`,
  leaving the Kind callers untouched (F1 avoided).
- **Minor findings landed:** § 3 loop shows `create_dir_all(kind.dir)`;
  non-reserved branch scoped to *file-creating* sub-artefacts + errs on absent
  parent; clobber-policy coupling + `create_dir_all(parent)` forward-fit noted on
  the `Artifact` writer; slug pure-helper/resolution-policy split named in § 4.

Status stays `proposed`. No code touched (paper + template asset only).
