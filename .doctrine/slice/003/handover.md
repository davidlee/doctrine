# slice-003 handover — implementation brief

Brief for a fresh agent **building** slice-003. The design is settled: two
internal review rounds + two external passes, all dispositioned, every fix
landed. Status is now `ready` (the user flipped the gate, 2026-06-04). No code
yet. Your job is to implement [design.md](design.md) faithfully — it is the build
spec, not a sketch. The audit trail below carries the *why* of every decision;
read it, don't relitigate it.

## What you are building

Extract the kind-agnostic directory-entity machinery out of `src/slice.rs` into a
new `src/entity.rs` engine, driven by a `Kind` descriptor, with **two callers of
different shape**: the existing slice (top-level, reserved, toml+md+symlink) and a
new **design-doc sibling** (sub-artefact, non-reserved, single prose file under an
existing slice dir). Adds `heresy slice design <id>`. Supersedes slice-002.

## Read first, in this order

1. [design.md](design.md) — **the build spec.** § 5.2 is the target interface
   (`Reservation`/`Acquired`/`LocalFs`, `Kind`/`MaterialiseMode`/`ScaffoldCtx`/
   `Artifact{rel_path}`/`Fileset`); § 5.4 is the `materialise` loop; § 8 is the
   build sequence; § 5.5 is the invariant checklist you must satisfy.
2. `src/slice.rs` — the starting code. `reserve_create` (the mkdir inline you
   lift), `build_scaffold`/`Scaffold` (the fixed-pair struct that becomes
   `Fileset`), `Meta`/`read_metas`/`format_list` (stay slice-side), and the
   slice-001 test suite (the behaviour-preservation gate).
3. **The audit trail below** — four rounds of review. The *why* of D1–D7, the
   `{{ref}}` token, the deferred facet, and the round-2 writer-contract hardening
   (H1/H2/M1/M4). Settled; do not re-open without genuinely new evidence.
4. Specs the design leans on: [reservation-spec](../../../doc/reservation-spec.md)
   § Code seam, [slices-spec](../../../doc/slices-spec.md) (on-disk shape,
   pure/imperative split, the WHAT/HOW edge), [entity-model](../../../doc/entity-model.md)
   (storage rule; templates-are-defaults).

## Build sequence (design.md § 8 — each step green against slice-001)

1. **`acquire` seam, in place.** Add `Reservation`/`Acquired`/`LocalFs`; rewrite
   `reserve_create`'s claim against it. Pure refactor; suite green.
2. **Engine module `src/entity.rs`.** Move the kind-blind pure fns + the
   `materialise` loop + the `Artifact` writer behind `Kind`/`ScaffoldCtx`. The
   slice `Kind` reproduces today's exact toml+md+symlink output. Suite still green.
3. **Design-doc `Kind` + `design.md` template** (already in `install/templates/`).
   The `CreateInExistingEntity` mode. Unit test the non-reserved path.
4. **Wire `heresy slice design <id>`.** Thin CLI verb over the engine
   (`SliceCommand::Design` in `src/main.rs`, reading the parent title via `Meta`).

## Must-honour invariants (the round-2 hardening — don't drop these)

- **H1 path containment.** `Artifact` paths are `rel_path`, relative to the
  entity-tree root (`Kind.dir`, **not** `ctx.dir` — the slug symlink sits at the
  root beside the numeric dir). The engine is the sole joiner; reject absolute
  paths and any `..` escaping the tree *before* writing. Test it.
- **H2 no ghost entities.** `AllocateFreshEntity`: a `Won` claim means the dir is
  ours, so on any write failure `remove_dir_all` it and propagate the error. Test:
  `reserved materialise write failure cleans up the won directory`.
- **M1 closed mode enum.** `MaterialiseMode { AllocateFreshEntity,
  CreateInExistingEntity }` — never a `reserve: bool`.
- **M4 scaffold purity.** `Kind.scaffold` is pure over `ScaffoldCtx` + embedded
  template text (`asset_text` is rust-embed, not disk IO). No clock/disk/git/root
  inside scaffold — the clock is `today()` → `ctx.date` in the shell.

## Do NOT build (deferred — stay in scope)

- `git-ref` backend / full `LeaseBackend` / the `Kind` reservation-namespace field
  (M2 — lands with git-ref, not now; an unused field trips the dead-code gate).
- The design-doc TOML facet, approval-as-state, `RVW-` review entity (D5).
- `heresy slice validate` (M5 — deferred note only).
- drift / spec entities. The engine only becomes *fit to host* them.

A finding mid-build is not a licence to widen scope; roadmap changes go via
supersede, not creep.

## The gate

`cargo test` + `cargo clippy` (deny-level; `--all-targets` is *not* the gate) +
`cargo fmt`. Red/green/**refactor**. Behaviour-preserving extraction — the
slice-001 suite stays green at **every** step (H2 cleanup is the one deliberate
addition; slice-001 has no opposing assertion). Update slice-003.toml `updated`/
`Done` per convention as you land steps. Append build notes below the audit trail.

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

---

# Round 2 (slice-003) — external review + disposition

A second external pass after the WHAT/HOW edge and the templates-are-defaults
rule landed. Verdict: **buildable, not quite build-ready** — the `Kind`
abstraction is sound; the risk is *filesystem correctness* once arbitrary
filesets and partial failures enter the engine (the writer/materialisation
contract). All findings paper-stage. No code touched.

## Review verbatim (condensed faithfully)

- **H1 — `Artifact.path: PathBuf` too permissive.** A descriptor can return
  `../../foo`, an absolute path, or a symlink outside the tree; the engine
  "writes uniformly". Make paths relative + validate (reject absolute / `..`);
  the engine is the sole joiner.
- **H2 — failed materialisation leaves a ghost entity.** Local `mkdir` claims +
  creates the dir; a later file-write failure leaves `003/` with missing
  toml/md/symlink — a malformed entity, not the spec's harmless reserved gap.
  Clean up the won dir on write failure (+ test); document the git-ref distinction.
- **M1 — `reserve: bool` carries too much** (id-alloc, fresh-dir, missing-parent,
  clobber). Replace with an explicit placement/mode enum before `false` accretes
  "existing parent / row append / nested / overwrite / no id".
- **M2 — future git-ref reservation needs a namespace field** (`slice/id/<n>`),
  not derivable from `dir`. Add/reserve `reservation_namespace` on `Kind` so
  git-ref doesn't "discover" it missing.
- **M3 — "spec drops in as descriptor, not fork" overstated.** Engine does spec
  *initial scaffold*; mutation/validation/FK/registry are separate. Tighten.
- **M4 — scaffold purity contract underspecified.** `fn(&ScaffoldCtx)->Fileset`
  must not read disk/clock/git/root; template loading is the danger. State it; say
  whether `asset_text` is compile-time embedded.
- **M5 — design-doc presence is workflow-significant but unobservable.** No gate
  this slice; add a deferred `heresy slice validate` note (non-trivial slice has
  design.md or a trivial marker; don't parse prose; facet carries queryable meta).
- **L1 — "sub-artefacts don't reserve" → "file-creating sub-artefacts".** Engine
  materialises filesets; it doesn't append rows / mutate tables / allocate row ids.
- **L2 — status stays `proposed`.** Agree. **L3 — template/exemplar divergence is
  a note, not a blocker** (`{{ref}}`+`{{title}}` resolved).

## Disposition

| Finding | Severity | Call | Landed in |
|---|---|---|---|
| H1 path escape | High | **Accept** — paths relative to entity-tree root (`Kind.dir`), engine sole joiner, reject absolute/`..`. *Corrected base:* tree root, **not** `ctx.dir` — the slug symlink legitimately sits at root level beside the numeric dir. | §5.2 `Artifact{rel_path}`; §5.5 Path containment; §9 test |
| H2 ghost entity | High | **Accept** — `Won` ⟹ dir is ours ⟹ `remove_dir_all` on write failure + propagate; document local-collapses vs git-ref-gap. One deliberate improvement over pure behaviour-preservation (slice-001 has no opposing assertion). | §5.4 loop; §5.5 No-ghost; §9 test |
| M1 `reserve` overloaded | Med | **Accept (cheaper form)** — `reserve: bool` → closed `MaterialiseMode` enum (no field-folding into variants). Names the branches; third mode = compiler-forced variant. | §5.2 enum; §5.4 match; D4 |
| M2 namespace field | Med | **Reject-defer (with reason)** — `slice/id/<n>` already in reservation-spec § Key table; the field is additive when git-ref lands, and a set-but-unread field now trips the deny-level dead-code gate. Forward-note instead. | D1 (M2 note) |
| M3 spec overstated | Med | **Accept** — "initial scaffold" vs lifecycle; "supports spec" ≠ "spec done". | §5.1 |
| M4 purity contract | Med | **Accept** — state purity invariant; `asset_text` is rust-embed (compile-time, not disk IO), so scaffold calling it stays pure-ish (only fallibility: template presence/format). | §5.5 Scaffold purity |
| M5 unobservable invariant | Med | **Accept (deferred note, no gate)** — future `heresy slice validate`; trivial marker is a TOML field (queryable ∈ TOML); never parse prose. | §6 Q5; slices-spec § Division of labour |
| L1 rows vs filesets | Low | **Accept** — explicit "engine materialises filesets, not rows". | §5.4 |
| L2 status proposed | Low | **Keep** — unchanged. | — |
| L3 template divergence | Low | **Keep (note only)** — resolved. | — |

Net: 7 accepts (2 High, 4 Med, 1 Low) + 1 reject-defer (M2) + 2 keeps. The High
pair (path containment, ghost cleanup) hardens the writer contract — the genuinely
new risk surface once arbitrary filesets and partial failure enter the engine.
Status stays `proposed`. No code touched.
