# Design SL-004: Implementation-plan and phase siblings

Design doc for slice-004, structured to the `doctrine slice design` template. Pure
prose (no frontmatter, no fenced data blocks — entity-model.md). Builds directly on
the slice-003 engine (`src/entity.rs`): `Kind`, `MaterialiseMode`, `Artifact`,
`materialise`, the `acquire` seam.

## 1. Design Problem

slice-003 left the engine *fit to host* richer siblings but proved only the two
simplest shapes (reserved 2-file slice; non-reserved single-prose design doc). The
next two change-side siblings break that simplicity along **two independent axes**:

- The **implementation plan** carries the first **relational authored facet**
  (`plan.overview` — ordered phases + spec/req links). It is design-data: authored,
  queried, fixed schema → TOML under the storage rule.
- **Phase tracking** carries the first **mutable runtime state** (`phase.tracking` —
  status, timestamps, a progress log). The entity model puts this in a *separate*
  `.doctrine/state/` tree, disposable and gitignored, never in the authored taxonomy.

The problem is to land both without (a) smearing mutable runtime state into the
authored entity tree, (b) forcing mutable state through the engine's write-once /
refuse-clobber scaffold contract, or (c) inheriting slice-003's writer debt
silently: the IP is the engine's first **multi-file sub-artefact**, and the
sub-artefact writer (`create_in_existing`) has *no* partial-write cleanup (slice-003
audit §2 `[M]`).

## 2. Current State

`src/entity.rs` materialises filesets through two modes:

- `AllocateFreshEntity` (`allocate_fresh`) — reserved top-level; **owns** the won dir,
  so on a write failure it `remove_dir_all`s it (H2, entity.rs:237).
- `CreateInExistingEntity` (`create_in_existing`, entity.rs:250) — sub-artefact; writes
  a fileset under a parent it does **not** own. Today the only caller (design doc) is a
  *single* file, so the gap is invisible. On a mid-fileset failure it cannot
  `remove_dir_all` the parent → leftover files survive. This is the debt the IP
  inherits the moment it writes more than one file.

`write_fileset` (entity.rs:296) writes artifacts in order, `create_dir_all`-ing each
parent, with no rollback. `refuse_clobber` (entity.rs:285) and `safe_join` (H1) already
guard the sub-artefact path. There is **no runtime-state surface at all**: nothing
under `.doctrine/state/`, no mutable writer, no `toml_edit` dependency.

`plan.overview@v1` / `phase.tracking@v1` shapes are in the schema bundle
(spec-driver-schemas). spec-driver duplicates phase metadata across `plan.overview`'s
phase list *and* per-phase `phase.overview` blocks — doctrine collapses that (D2).

## 3. Forces & Constraints

- **Behaviour-preserving for slice + design doc.** The slice-001 and slice-003 engine
  suites pass unchanged — every step is gated by them.
- **The authored ÷ runtime boundary is a hard line** (entity-model.md § Runtime state;
  spec-entity-spec § Design-data vs runtime-state). Mutable state lives under
  `.doctrine/state/`, never in `.doctrine/slice/<id>/`.
- **The engine scaffolds; it does not mutate.** It materialises write-once filesets and
  refuses clobber (entity.rs L1 / slice-003 audit). Mutable, idempotently-rewritten
  runtime state must **not** be forced through it.
- **Edit-preserving mutation** (`toml_edit`, not serde reserialize) for anything the
  tool rewrites — comments and unknown keys survive (entity-model § Rust model).
- **No spec/requirement registry yet.** `plan.overview` spec/req link fields are present
  but empty in v1, same posture as slice `[relationships]`.
- **Project gate.** `cargo test` + `cargo clippy` (deny-level) + `cargo fmt`.
- **Out of scope.** Spec-family table machinery + FK validation, multi-agent/resumable
  run orchestration, the broader `.doctrine/state/` surface (session/lease/review),
  `git-ref` backend. This slice builds the state *tree* and the *phase* facet only.

## 4. Guiding Principles

- **Authored stays pure; runtime lives apart.** `.doctrine/slice/<id>/` holds only
  authored toml+md; all mutable churn is in a mirrored, gitignored `.doctrine/state/`
  tree. The split is *two modules*, not one module with a flag.
- **Fewer entities, more facets.** The IP is a slice **facet** (`plan.{toml,md}`,
  1:1 with the slice), not a reserved `IP-` entity. Phase tracking is a slice-scoped
  runtime facet, not a `PHASE-` entity. No new reservation namespace.
- **One authored source for the phase plan.** `plan.toml` declares the phases; tracking
  never re-authors phase metadata — it only folds runtime progress against the plan
  (collapses spec-driver's duplication).
- **Discharge inherited debt at the point of first use.** The IP is the first multi-file
  sub-artefact, so this slice makes the sub-artefact writer transactional — here, not
  "later".
- **Symlink is convenience; id is identity.** Phase state is reached by computing the
  path from the slice id, never by following a symlink (same rule as the slug symlink).
- **A checkbox is not a reason for TOML — a *query* is.** The storage rule keys off
  "does the *tooling* read it," not "does it have a status." Phase content sorts on two
  axes: *durability* (authored→durable→`plan.toml`; runtime execution progress→
  disposable→state tree — this is what the gitignore boundary forces) × *structure*
  (tool-queried→TOML; narrative→`.md`). **Durability keys on whether the data is authored
  vs accrued-at-runtime, not on the word "status":** an entity's *lifecycle* status is an
  authored, queried, durable field (entity-model § State vocabulary); a phase's
  execution progress (status transitions, timestamps, the progress log) is **disposable —
  loss-accepted, gitignored**, not "regenerable" (a timestamped log cannot be reconstructed
  from the plan; only the empty *scaffold* re-seeds). Most phase sections are
  `{id, status, one-liner}` — the one-liner is
  a TOML *string field*, not a prose file; the prose *file* is only the multi-paragraph
  tail. Structure is earned by a consumer, not minted for every `[x]`.

## 5. Proposed Design

### 5.1 System Model

Three surfaces, one boundary, plus two prose file conventions:

1. **The IP facet** — `plan.toml` (relational `plan.overview` rows) + `plan.md` (prose),
   scaffolded as a **multi-file `CreateInExistingEntity` fileset** by the existing
   engine, into `.doctrine/slice/<id>/`. Authored: written once, then hand-edited; the
   tool reads/queries it but does not mutate it in v1. This is the **durable, structured**
   plan.

2. **The phase runtime facet** — per declared phase, a `phase-NN.toml` (tracking) **and a
   `phase-NN.md` (the detailed, disposable phase sheet)**, both under
   `.doctrine/state/slice/<id>/phases/`, owned by a **new `src/state.rs`** (not the
   engine). The `.toml` is mutable structured tracking (`toml_edit`); the `.md` is
   prose that *expands* the plan's high-level phase entry into concrete tasks /
   assumptions / verification at execution time. It deliberately **duplicates and
   elaborates** the plan's phase row — that prose overlap is accepted (the spec-driver
   `plan-phases` skill does the same). The sheet is **disposable runtime scratch** —
   gitignored, `rm -rf`-able. It is *not* fully regenerable: the plan can re-seed the
   scaffold (objective echo, reading-list, task checklist), but execution-time
   risks/decisions/findings authored into the sheet are **not** in the plan. Those are
   working notes by default; the **durable persistence path is the future slice
   close-out audit** (Follow-Ups), which harvests findings/decisions into the tracked
   audit artefact at slice end. Until then, anything an author needs to survive
   `rm -rf` is theirs to lift into `notes.md`. The sheet is never the authoritative copy.

3. **The runtime-state boundary, in code** — the engine keeps its write-once/
   refuse-clobber contract for authored entities; `state.rs` owns mutable IO under
   `.doctrine/state/` with ensure-parent-on-demand semantics (a state parent is *not* a
   reserved entity, so the engine's "parent must exist" guard does not fit). The two
   never write into each other's tree.

**Two slice-folder file conventions** (low-ceremony, parallel to the durable/disposable
split above):

- **`notes.md` — durable, tracked, on-demand.** A per-slice scratchpad in the authored
  slice folder, scaffolded from a *very simple* template by `doctrine slice notes <id>`
  (the exact single-file `design.md` pattern). On-demand, not auto-created, so `slice new`
  output is unchanged and empty notes files never accrue.
- **`handover.md` — disposable, gitignored, no tooling.** Agents drop an ad-hoc handover
  in the slice folder when context must survive a session boundary. **No template, no
  verb** — it is pure convention plus a gitignore entry (the inverse of `notes.md`: the
  contrast is the point). It sits *physically* in the slice folder for zero-ceremony
  ergonomics (right next to the slice it concerns), but because it is gitignored it never
  enters the **git-tracked** authored surface — the purity invariant is about the tracked
  tree, not physical adjacency (§5.5). The future slice close-out audit GCs it once the
  slice closes (Follow-Ups). **On entity-model coherence (finding 7):** entity-model
  § Runtime state lists "handoff" among the *structured, machine-managed* runtime caches
  (session state, review-index, heartbeat/lease) that live under `.doctrine/state/`.
  `handover.md` is deliberately **not** that: it is an unstructured, toolless, human/agent
  prose note with no schema and no reader — the antithesis of a managed cache. It is an
  explicit, reasoned carve-out (a zero-ceremony convenience file), not the structured
  handoff-state surface, which remains deferred and, when built, lands under
  `.doctrine/state/` as the entity model directs. The boundary the engine enforces is
  *tracked-tree purity*; physical location of a gitignored toolless note is a UX choice.

drift/spec later reuse the IP path (relational authored facet + transactional
multi-file writer); session/lease/review later reuse `state.rs` (the `.doctrine/state/`
writer). Both seams are proven here against one real caller each.

### 5.2 Interfaces & Contracts

**The IP facet (engine `Kind`, no new engine types).** A slice-side `const Kind`:

```rust
// dir = ".doctrine/slice"; CreateInExistingEntity; prefix "SL" (inherits parent ref).
// scaffold returns a 2-Artifact fileset: plan.toml + plan.md, both under "<id>/".
const PLAN_KIND: Kind = Kind {
    dir: ".doctrine/slice",
    prefix: "SL",
    mode: MaterialiseMode::CreateInExistingEntity,
    scaffold: plan_scaffold,   // ctx -> [File "<id>/plan.toml", File "<id>/plan.md"]
};
```

`plan.toml` carries the `plan.overview` shape, doctrine-flattened (no fenced block —
it *is* the file), and is the **single durable source of every phase's authored
contract** — objective, criteria *definitions*, verification *expectations*, links.
Their *status* (done/pass) is runtime and lives in the phase tracking (§5.3), joined by
the `id`s declared here:

```toml
schema  = "doctrine.plan.overview"
version = 1
slice   = "SL-004"            # the parent ref (was plan/delta in spec-driver)

[specs]                        # empty in v1 (no registry)
primary       = []
collaborators = []

[requirements]
targets      = []
dependencies = []

[[phase]]                      # ordered; the single authored source of the plan
id        = "PHASE-01"
name      = "…"
objective = "…"
# Criteria/verification carry ids so runtime status joins back by ref (§5.3).
entrance_criteria = [{ id = "EN-1", text = "…" }]
exit_criteria     = [{ id = "EX-1", text = "…" }]
verification      = [{ id = "VT-1", expects = "…" }]
specs             = []         # per-phase narrowing of the plan's specs (optional)
requirements      = []
```

`plan.md` is prose narrative (`# Implementation Plan SL-004: {{title}}`), template-only
substitution. Authored data is owned (TOML); prose is a default (entity-model
§ Templates are defaults).

**The phase-tracking writer (`src/state.rs`).** A small mutable-runtime module — *not*
a `Kind`. It does **not** fork the engine's filesystem-safety logic: `safe_join`, the
atomic create-new write, the real-directory check, and the symlink set-or-replace are
lifted into a shared `src/fsutil.rs` (extracted from `entity.rs` as the first step of
this slice) and consumed by both the engine and `state.rs`. The module boundary (D3)
separates *contracts* (scaffold-once vs mutate-in-place), not IO primitives.

**Id ↔ filename mapping (pinned, finding 4).** The canonical phase id is `PHASE-NN`
(`NN` = zero-padded ordinal, declared in `plan.toml`). The on-disk filename is
`phase-NN.{toml,md}` — a deterministic lowercase of the `PHASE-` prefix, `NN` verbatim.
The runtime join key (a tracking `ref`, the `phase` field) is the canonical `PHASE-NN`,
never the filename. `phase_stem` enforces the `PHASE-<digits>` form, which both
makes the filename derivation total and rejects filesystem-unsafe input.

```rust
/// Canonical state path for a slice's phase tracking. Computed from the id; the
/// tool never follows the convenience symlink (id is identity).
fn phases_dir(project_root: &Path, slice_id: u32) -> PathBuf
    // -> .doctrine/state/slice/<id>/phases

/// Validate a phase id and derive its filename stem. Enforces the canonical
/// `PHASE-<digits>` form (-> stem `phase-NN`); rejects empty, separators, `..`,
/// leading dot — a phase id reaches the filesystem, so it is untrusted input.
fn phase_stem(phase_id: &str) -> anyhow::Result<String>   // "PHASE-01" -> "phase-01"

/// Materialise per declared phase a phase-NN.toml (tracking) + phase-NN.md (the
/// disposable detailed sheet, from a simple template) under the state tree.
/// Validates the plan's phase ids are well-formed AND unique up front (a duplicate
/// id would alias two phases onto one file — finding 6; this check is v1, not
/// deferred, because phases are consumed immediately). Ensures the (gitignored,
/// possibly-absent) parent, then writes any **missing file of each phase's pair**
/// (per-file skip, not per-phase — so a phase left half-written by an earlier crash
/// is completed on re-run, finding round-2). Diffs existing tracking against the
/// plan: returns a report of new / orphan (tracking exists, plan phase gone) / stale
/// (id renamed) phases. Orphan removal is never silent — reported, repaired only under
/// explicit `--prune` (destructive). Refreshes the symlink (verifies target; replaces
/// a wrong link; errors on a real file/dir squatting the path). Idempotent on the
/// no-drift, no-partial path.
fn init_phases(project_root: &Path, slice_id: u32, plan: &Plan) -> anyhow::Result<InitReport>

/// Edit-preserving status transition on one phase (toml_edit): sets `status`,
/// stamps `last_updated`/`started`/`completed`, appends a `[[progress]]` row.
/// Preserves comments and unknown keys (entity-model § Rust model). Derives the
/// filename via `phase_stem` (validates `phase_id`) before resolving the path.
fn set_phase_status(
    project_root: &Path, slice_id: u32, phase_id: &str, status: PhaseStatus, note: Option<&str>,
) -> anyhow::Result<()>
```

`phase-NN.toml` — **minimal in v1**: phase-level status + an append-only progress log,
the one thing a rollup (`slice show`) and the future done-gate must read. Richer
structured tracking (per-criterion done-flags, verification status, task rows) **does
not start here** — it begins as a markdown checklist in `phase-NN.md` and *graduates* to
TOML rows when a consumer actually lands (D5/Q2). Graduation is free: the graduating
content is *status* (disposable runtime), so no durable data migrates.

```toml
schema  = "doctrine.phase.tracking"
version = 1
phase   = "PHASE-01"
status  = "planned"            # planned | in_progress | completed | blocked
started      = ""
completed    = ""
last_updated = ""

[[progress]]                    # append-only runtime log (toml_edit)
timestamp = "…"
status    = "in_progress"
note      = "…"
# Graduates here when a tool consumes them (D5/Q2/Q5):
#   [[criterion]] ref = "EX-1"  done = true       ← when `slice validate` ships
#   [[verification]] ref = "VT-1"  status = "verified"
#   [[task]] id = "2.1"  status = "done"  parallel = false
```

**CLI verbs (slice-side, thin):**

| Verb | Effect |
|---|---|
| `doctrine slice plan <id>` | Scaffold `plan.{toml,md}` into the slice dir (engine, `PLAN_KIND`). Refuses clobber. |
| `doctrine slice phases <id>` | Read `plan.toml`; `init_phases` → `phase-NN.{toml,md}` per declared phase under the state tree. |
| `doctrine slice phase <id> <phase-id> --status <s> [--note …]` | `set_phase_status` (the mutable runtime path; proves `toml_edit`). |
| `doctrine slice notes <id>` | Scaffold a durable `notes.md` into the slice dir (engine, single-file `CreateInExistingEntity` — the `design.md` pattern). Refuses clobber. |

`handover.md` has **no verb** — agents author it ad hoc (disposable, gitignored).

### 5.3 Data, State & Ownership

| Path | Owner | Nature |
|---|---|---|
| `.doctrine/slice/<id>/plan.toml` | `plan_scaffold` (engine) + human | authored, relational, **read** by tool, hand-edited |
| `.doctrine/slice/<id>/plan.md` | template | authored prose, write-once |
| `.doctrine/slice/<id>/notes.md` | template (on-demand) | **durable** authored scratchpad, tracked |
| `.doctrine/slice/<id>/handover.md` | agent (ad hoc) | **disposable**, gitignored, no template/verb |
| `.doctrine/state/slice/<id>/phases/phase-NN.toml` | `state.rs` | **mutable runtime** tracking, tool-written |
| `.doctrine/state/slice/<id>/phases/phase-NN.md` | template (via `init_phases`) | **disposable runtime** phase sheet (scratch; durable bits harvested by close-out audit) |
| `.doctrine/slice/<id>/phases` → `../../state/slice/<id>/phases` | `state.rs` (on write) | gitignored convenience symlink; **not** authority |
| `install/manifest.toml` `[gitignore]` | this slice | `+ .doctrine/state/`, `+ .doctrine/slice/*/phases`, `+ .doctrine/slice/*/handover.md` |
| `install/templates/` | this slice | `+ plan.toml`, `+ plan.md`, `+ phase.md`, `+ notes.md` |
| `src/state.rs` (new) | this slice | the `.doctrine/state/` runtime writer |
| `src/slice.rs` | this slice | `PLAN_KIND`, `NOTES_KIND`, plan read (`Plan` model), the four verbs |
| `src/entity.rs` | this slice | transactional multi-file sub-artefact writer (D4) |

**Where each phase-sheet section lands** (mapping the spec-driver 11-section phase sheet
onto the two axes — durability × structure):

| Section (spec-driver) | Durable plan.toml | Runtime phase-NN.toml | Runtime phase-NN.md |
|---|---|---|---|
| 1 Objective | ✓ (string) | | |
| 2 Links & References | ✓ typed FKs (specs/req/decisions) | | reading-list (file:line) |
| 3/4 Entrance/Exit criteria | ✓ definitions (id+text) | done-flags *(graduate)* | |
| 5 Verification | ✓ expectations (id+expects) | status *(graduate)* | |
| 6 Assumptions & STOP | | | ✓ |
| 7 Tasks & Progress | | task rows *(graduate)*; progress log, phase status | |
| 7 Task Details | | | ✓ |
| 8 Risks · 9 Decisions · 10 Findings | | | ✓ |
| 11 Wrap-up checklist | | flags *(graduate)* | |

*(graduate)* = ships as a markdown checklist in `phase-NN.md` in v1, becomes TOML rows
when a consumer lands (D5/Q2). Definitions are authored once in the plan; only *status*
is runtime. §2's typed edges are largely **inherited from the plan**, not re-authored.

`.doctrine/state/` is **not** added to manifest `[dirs]` (authored dirs only) — the
state writer creates it lazily, so a clone with the dir gitignored-away just recreates
it on first write. The plan `Plan` model is the first relational *read* model; it stays
slice-side (no premature shared `Meta`, per slice-003 Non-Goal).

### 5.4 Lifecycle, Operations & Dynamics

**Authoring flow.** `slice plan <id>` scaffolds the plan skeleton → human authors phases
in `plan.toml` → `slice phases <id>` reads the plan and materialises `phase-NN.{toml,md}`
per declared phase → as work runs, `slice phase <id> PHASE-NN --status …` folds progress
into the runtime tree. The plan is authored once; tracking churns.

**Transactional multi-file write (D4 — discharges slice-003 `[M]`).** The sub-artefact
writer cannot delete a parent it does not own, so it tracks exactly what *this call*
creates and undoes that on failure. Corrections over a naive design (audit findings 1–2,
round-2): clobber refusal must be **atomic with the write** (a separate `exists()` pass is
a TOCTOU window); `create_dir_all` cannot report which components it created (so dir
creation is **component-wise**); a created path is tracked **before** its content write (a
mid-write failure must still unlink the just-created file); and the writer handles **every**
artifact arm (`File` *and* `Symlink`) since it replaces the shared `write_fileset`:

```text
create_in_existing(kind, tree_root, inputs):
  resolve parent dir (err if absent)            # unchanged
  fileset = kind.scaffold(ctx)
  refuse_clobber(tree_root, fileset)            # fast-fail courtesy only; NOT the safety boundary
  write_fileset_transactional(tree_root, fileset):
    created_paths = []; created_dirs = []       # created_paths: files AND symlinks, in order
    for art in fileset:
      # dirs: walk the rel path; for each missing component, create_dir it.
      for component in ancestors(art.rel):
        match create_dir(component):
          Ok            -> created_dirs.push(component)
          AlreadyExists -> if not is_real_dir(component): goto rollback   # a FILE squats the path
                           else skip (pre-existing dir; not ours)
          err           -> goto rollback
      match art:
        File:    # atomic create-new IS the clobber refusal (one syscall, no race).
          match OpenOptions::create_new(true).write(true).open(art.abs):
            Ok(f)  -> created_paths.push(art.abs)   # track BEFORE write, so a partial
                      write(f, body) or goto rollback   #   write is still unwound
            err    -> goto rollback
        Symlink: # symlink(2) is atomic; AlreadyExists is a clobber → fail (not tolerate)
          match symlink(art.target, art.abs):
            Ok     -> created_paths.push(art.abs)
            err    -> goto rollback
    rollback (on any error):
      remove created_paths                       (reverse order; unlink file or symlink)
      remove_dir created_dirs                    (reverse order)
        ignore NotFound and DirectoryNotEmpty    # another artefact/process populated it
      propagate the original error
```

Only paths/dirs *this call* created are removed — pre-existing parents and any dir a
concurrent writer populated are untouched (the `DirectoryNotEmpty`-tolerant `remove_dir`,
not `remove_dir_all`). `create_new(true)` makes the file create atomically fail on an
existing target, collapsing refuse-clobber and write into one race-free step (the threat
is low on a local single-user CLI, but the atomic form is strictly better and near-free).
Tracking the path *before* the content write means a write that fails after `create_new`
still unwinds the empty/partial file. The `Symlink` arm is preserved (the IP/notes
filesets are file-only today, but this writer is the shared `write_fileset`, so dropping
the arm would silently break any future symlink-bearing sub-artefact).
This is weaker than `allocate_fresh`'s `remove_dir_all` (which can nuke the whole won
dir) by necessity, and that asymmetry is the point: a sub-artefact is a guest. `init_phases`
in the state tree is separately idempotent (skip-if-exists **per file**, so a phase left
half-written by a crash completes on re-run) rather than transactional — runtime state is
disposable, so a partial init is re-runnable, not
rolled back.

**State-tree writes.** `state.rs` ensures `.doctrine/state/slice/<id>/phases/` on demand
(the parent is not a reserved entity — the engine's existence guard does not apply),
writes the `phase-NN.{toml,md}` skeletons (skip-if-exists), and **diffs existing tracking
against the plan** to report new / orphan / stale phases (§5.2 `init_phases`) — a phase
renamed or removed in `plan.toml` leaves a reported orphan, never a silent stale file;
removal is explicit (`--prune`). The symlink refresh **verifies the link target**:
a correct link is left, a wrong link is replaced, and a real file/dir squatting the path
is an error (not the old blanket `AlreadyExists`-tolerant create, which masked both).
Status updates use `toml_edit`
to append `[[progress]]` and set fields on `phase-NN.toml` without disturbing
comments/unknown keys; `phase-NN.md` is never tool-mutated after scaffold (prose, agent-
/human-owned).

**`notes.md` / `handover.md`.** `slice notes <id>` runs the engine's single-file
`CreateInExistingEntity` path (the `design.md` flow, a `NOTES_KIND`) — durable, refuses
clobber. `handover.md` has no code path at all: it is an agent convention plus the
installer gitignore entry, so a dropped handover never pollutes git history.

### 5.5 Invariants, Assumptions & Edge Cases

- **The git-tracked authored tree stays pure.** Nothing *tracked* under
  `.doctrine/slice/<id>/` is tool-mutated after scaffold except via the human; all runtime
  churn is under `.doctrine/state/`. The invariant is about the **tracked** surface, not
  physical location: `handover.md` sits in the slice folder for ergonomics but is gitignored,
  so it never enters the tracked tree (finding 7). (Test: `slice phase …` writes only under
  `.doctrine/state/`; `handover.md` is matched by a gitignore entry.)
- **Symlink-blind resolution.** Every state read/write computes the path from the id;
  removing or breaking the symlink changes nothing the tool does. (Test: delete symlink,
  `slice phase` still resolves.) The symlink *refresh* nonetheless verifies and corrects a
  wrong link, and errors on a real file/dir at that path — convenience kept honest without
  ever being trusted as authority. (Test: wrong-target link is replaced; squatting dir errors.)
- **Transactional cleanup is exact.** A mid-fileset failure leaves the parent dir exactly
  as it was pre-call — no leftover plan.toml when plan.md fails; only components this call
  created are removed, and a dir a concurrent writer populated is left intact (D4, findings
  1–2). (Test: second-file failure leaves parent byte-identical; pre-existing + concurrently
  populated dirs survive rollback.)
- **Phase/criterion ids are immutable and never reused.** Once authored in `plan.toml`, a
  phase id and its criterion/verification ids (`EN-n`/`EX-n`/`VT-n`) never renumber or
  recycle — edits append, never reassign. This is the precondition for the runtime status
  join (finding 6): graduated tracking rows join by `ref`, so a reused id would alias old
  status onto a new criterion. Validation (dup/dangling-ref check) lands with the first
  consumer that reads the rows (graduation, D5/Q2); the *rule* holds from v1. (Test deferred
  to the consumer; documented as an authoring contract now.)
- **Phase init reports drift, never masks it.** Re-running `slice phases <id>` after the
  plan grows adds new phases and never clobbers existing tracking; after a phase is renamed
  or removed it **reports** the orphan/stale file rather than silently leaving it live.
  Removal is explicit (`--prune`), never automatic (finding 5). (Test: rename a plan phase,
  re-init, assert the orphan is reported and not silently consumed by a rollup.)
- **Edit-preserving status update.** A hand-added comment / unknown key in
  `phase-NN.toml` survives a `set_phase_status` (`toml_edit`). (Test.)
- **Engine contract unchanged for existing kinds.** slice + design-doc byte output and the
  slice-001/003 suites stay green (the transactional writer is behaviour-identical on the
  success path).
- **Path containment (H1) still holds** — plan/phase artifacts are tree-relative; `safe_join`
  rejects escapes unchanged.
- **gitignore is additive.** Installer only appends absent entries (`read_gitignore_lines`);
  re-install is a no-op.

### 5.6 Build Sequence

Ordered, each step green against the slice-001/003 suites before the next (the slice-003
§5.4 discipline; audit finding 11). Each `fsutil` primitive is **extracted in the step
that first calls it** — never ahead of use — so no step lands a dead fn that trips
`deny(unused)` (finding 11, round-2). Steps 1–2 are pure engine/IO behind the existing
suites; nothing user-visible lands until the writer is correct.

1. **Transactional writer (D4) + `fsutil` seed** — rewrite the shared `write_fileset` to
   component-wise `create_dir` tracking + `create_new(true)` atomic write + symlink arm +
   reverse rollback. In the *same* step, lift `safe_join` and the new atomic-write/real-dir
   helpers into `src/fsutil.rs` — each consumed immediately by this writer (no unused fn).
   Tests: mid-write file failure unwinds the partial file; second-artifact failure leaves
   parent byte-identical; pre-existing + concurrently-populated dirs survive; file-squats-dir
   errors; success path byte-identical to today. *Discharges slice-003 `[M]`.*
2. **`plan.{toml,md}` templates + `PLAN_KIND` + `slice plan` verb** — the first multi-file
   `CreateInExistingEntity`, now on the transactional writer. Refuses clobber; errors if the
   slice is absent.
3. **`Plan` read model** — the first relational read; parse `plan.toml` → phases (validates
   phase-id well-formedness + uniqueness). Slice-side, no shared `Meta`.
4. **`src/state.rs` + `slice phases`** — `phases_dir`, `phase_stem`, `init_phases`
   (ensure-parent, per-file skip, plan-drift report, verified symlink refresh — the
   symlink set-or-replace helper extracts into `fsutil` here, its first state-side use).
   Add `toml_edit` to `Cargo.toml` here (`unused_crate_dependencies` is paused, so adding
   it one step before its step-5 first use is clean).
5. **`set_phase_status` + `slice phase`** — the `toml_edit` mutate path (status set,
   `[[progress]]` append, comment/unknown-key preservation). Symlink-blind resolution.
6. **`slice notes` + `NOTES_KIND`** — single-file durable scaffold (the `design.md` path,
   second caller).
7. **Installer gitignore entries** — `.doctrine/state/`, `.doctrine/slice/*/phases`,
   `.doctrine/slice/*/handover.md`. Additive; re-install a no-op.

Template contracts (drafted before step 2/4): `plan.toml` carries the `plan.overview` rows
(§5.2); `plan.md`/`notes.md`/`phase.md` are **prose-only** token-substitution scaffolds —
no queried data in a `.md` (the storage rule; entity-model § Templates are defaults), so a
re-headed or trimmed sheet never breaks tooling.

## 6. Open Questions & Unknowns

- **Q1 — One tracking file per phase vs one per slice.** Chosen: per phase
  (`phase-NN.toml`) — independent updates, matches the plan's phase list. Revisit only if
  cross-phase atomic updates are ever needed (unlikely; gitignored runtime).
- **Q2 — The graduation trigger for criteria/verification status (resolved to a path,
  not deferred).** v1 carries phase status + progress only; per-criterion done-flags and
  verification status start as a `phase-NN.md` checklist. They **graduate to TOML rows**
  (`[[criterion]] ref="EX-1" done=true`, keyed to the plan, not copied text) the moment a
  tool reads them — concretely, when `slice validate`/the done-gate (M5) needs exit-criteria
  to gate `done`. Open only in *timing* (which consumer lands first), not in *shape*.
- **Q3 — Read-locality reassembler.** `doctrine slice show <id>` joining plan + tracking +
  (later) coverage at read time is the mitigation for the split tree (cf. the spec
  read-locality CLI). Deferred to a follow-up; the data is queryable without it.
- **Q4 — Does the plan ever need its own `IP-` id?** Only if a slice carries multiple
  plans. v1 is 1:1 (a facet). Additivity is precise (finding 9): if multi-plan ever
  emerges, `plan.toml` **persists as the permanent 1:1 facet** — the new capability is a
  *new* reserved-entity surface for the *additional* plans, not a migration of the existing
  facet. Old plans are never rewritten or re-pathed; the escape hatch adds a surface beside
  the facet, it does not convert it.
- **Q5 — Task-row granularity.** v1 keeps tasks as a `phase-NN.md` checklist (agents author
  light markdown). Task *rows* (`[[task]] id status parallel`) graduate to `phase-NN.toml`
  when a consumer needs queryable per-task status — a progress rollup, or multi-agent
  execution dividing work by the `parallel` flag (out of scope now). Same graduation
  mechanism as Q2; no counts field in the interim (a rollup recomputes from the checklist
  or the rows when they exist).

## 7. Decisions, Rationale & Alternatives

- **D1 — The IP is a slice facet (`plan.{toml,md}`), not a reserved `IP-` entity.** 1:1
  with the slice, which already namespaces it; fewer-entities-more-facets (entity-model).
  Engine-wise it is a 2-`Artifact` `CreateInExistingEntity` fileset — no new mode, no new
  reservation. *Alternative rejected:* a reserved `IP-NNN` entity — buys a second number
  space and a reservation namespace for a strictly 1:1 artefact (Q4 holds the escape hatch).
- **D2 — Phase content sorts on two axes (durability × structure), not one.** Driven by
  a section-by-section pass over a real spec-driver phase sheet (§5.3 table):
  - **Definitions are authored planning → durable → `plan.toml`** (per phase, id-bearing):
    objective, entrance/exit criteria *text*, verification *expectations*, typed links.
    `plan.overview` is the single authored source; doctrine does *not* re-encode it as
    per-phase `phase.overview` *blocks* (spec-driver's drift-prone duplication).
  - **Execution *progress* is accrued-at-runtime → disposable → phase tracking**, joined
    to the plan by id (criterion/verification/task *status*). The discriminator is
    authored-vs-runtime, not the word "status": an entity's authored *lifecycle* status
    stays a durable, queried field (entity-model § State vocabulary); the phase's progress
    (status, timestamps, log) is **disposable — loss-accepted, gitignored** (not
    regenerable: the plan re-seeds only the empty scaffold). Ships minimal (phase status +
    progress) and *graduates* to TOML rows on demand (D5/Q2/Q5).
  - **Narrative is prose → `phase-NN.md`** (disposable scratch): assumptions/STOP, risks,
    decisions, findings, Task-Details, the §2 reading-list. Prose duplication of the
    plan is fine — the sheet is never authoritative. Note it is disposable, **not** fully
    regenerable: the plan re-seeds the scaffold, but execution-time risks/decisions/findings
    are not in the plan. Their durable home is the close-out audit's harvest (§5.1); until
    then they are working notes (finding 3).

  The discriminator is **"does the tool read it,"** not "does it have a checkbox" (§4) —
  a `{id, status, one-liner}` row is one TOML row (text as a string field), never a
  prose file. *Alternative rejected:* per-phase authored *structured* blocks (spec-driver)
  — reintroduces the metadata duplication the entity model exists to remove.
- **D3 — Phase tracking is runtime state in `.doctrine/state/`, owned by `src/state.rs`,
  NOT the engine.** The engine's contract is write-once + refuse-clobber + immutable
  scaffold; phase tracking is mutable and idempotently rewritten. Forcing it through
  `CreateInExistingEntity` would mean either clobbering accrued state or special-casing the
  engine. A separate module keeps each contract clean and is the seam session/lease/review
  reuse later. *Alternative rejected:* a `MutateInPlace` engine mode — pollutes the
  scaffold engine with mutation semantics it was deliberately kept free of (slice-003 L1).
  **The split is of *contracts*, not IO primitives** (finding 4): `safe_join`, atomic
  create-new, real-dir check, and symlink set-or-replace are lifted into a shared
  `src/fsutil.rs` and consumed by both `entity.rs` and `state.rs`, so the new writer does
  not re-implement path-containment or symlink safety. `phase_id`, which becomes a filename,
  is validated (filename-safe) before it reaches the filesystem.
- **D4 — Make the sub-artefact writer transactional now.** The IP is the first multi-file
  `CreateInExistingEntity`; the writer it inherits leaves leftovers on partial failure
  (slice-003 audit `[M]`, explicitly deferred *to this slice*). Track-and-unlink (not
  remove-parent — the parent is a guest's host). Two correctness requirements the naive
  form misses (audit findings 1–2): (i) clobber refusal is the **atomic** `create_new(true)`
  file open, not a separate pre-write `exists()` pass (which is a TOCTOU window); (ii) dir
  creation is **component-wise `create_dir`** so the writer knows exactly which components it
  made — `create_dir_all` reports nothing and would make the "parent exactly as pre-call"
  invariant unprovable. Rollback unlinks created files and `remove_dir`s created components
  in reverse, tolerating `DirectoryNotEmpty`/`NotFound` (never `remove_dir_all`, never the
  parent). *Alternatives rejected:* stage-and-rename a temp dir — heavier, and the slice dir
  is not ours to swap; a separate `exists()` clobber gate — racy and redundant once
  `create_new` does it atomically.
- **D5 — `toml_edit` enters for the *state* writer, not the authored plan; structured
  tracking graduates on demand.** `plan.toml` is scaffolded once then hand-edited (no tool
  mutation in v1) → no edit-preserving writer yet. `phase-NN.toml` *is* tool-mutated
  (progress appends, status sets) → it gets `toml_edit`. v1 tracks only phase status +
  progress; per-criterion / per-verification / per-task status starts as a markdown
  checklist in `phase-NN.md` and **graduates to TOML rows when a consumer lands** (first:
  exit-criteria, when `slice validate`/the done-gate ships — M5). Graduation is free
  because the graduating content is *status* — disposable runtime, not durable data — so
  nothing migrates (the prose sheet's risks/decisions are a separate concern, harvested by
  the close-out audit, §5.1 / Follow-Ups), and we avoid
  forcing agents to hand-author rigid TOML for status nothing queries yet ("as simple as
  possible, but no simpler"). *Alternative rejected:* full `phase.tracking@v1` (criteria
  arrays + tasks[] + counts) up front — speculative structure ahead of any reader.
- **D6 — `.doctrine/state/` and `.doctrine/slice/*/phases` are gitignored via the installer;
  the state dir is not a managed `[dir]`.** Runtime state is disposable and must not be
  committed; the symlink must be ignored too or it dangles into the ignored tree on a fresh
  clone. The writer recreates both on demand. *Alternative rejected:* tracking `.doctrine/state/`
  — couples runtime churn to the repo history the entity model separates it from.
- **D7 — Verbs are `slice plan`, `slice phases`, `slice phase`, `slice notes`.** Plain-English,
  parallel to `slice design`. `phases` (plural) materialises the set; `phase` (singular)
  transitions one.
- **D8 — `notes.md` is durable + tool-scaffolded (on-demand); `handover.md` is disposable +
  toolless.** The two are a deliberate matched pair on the durable/disposable axis. `notes.md`
  is a tracked scratchpad scaffolded from a *very simple* template via `slice notes` — on-demand
  (not part of `slice new`) so the slice-001 fileset stays byte-stable and empty notes never
  accrue. `handover.md` is the opposite: no template, no verb, agent-authored when a session
  boundary looms, and gitignored so it never enters history. *Why on-demand, not auto:* a
  durable file auto-created empty in every slice is clutter; a verb costs one line and creates
  it only when wanted (the `design.md`/`NOTES_KIND` path already exists). *Why no tooling for
  handover:* its whole value is zero-ceremony ad-hoc capture; a schema would defeat it.

## 8. Risks & Mitigations

- **Runtime/authored leakage** — a tool path writing state into the authored tree, or a
  read treating state as authoritative. *Mitigated* by the two-module split (engine never
  writes under `.doctrine/state/`; `state.rs` never writes under `.doctrine/slice/<id>/`
  except the gitignored symlink), the gitignore entries, and the symlink-blind invariant.
- **Transactional-writer correctness** — under-cleanup (leftovers) or over-cleanup
  (removing a pre-existing file). *Mitigated* by tracking *only* what this call created and
  a mid-fileset-failure test asserting the parent is byte-identical pre/post.
- **Scope creep into spec tables / multi-agent orchestration.** *Held off* (§3 Out of scope;
  Q2/Q5 defer the rich tracking surface; plan spec/req fields stay empty).
- **`toml_edit` mis-round-tripping** comments/unknown keys on the progress append.
  *Mitigated* by an explicit preserve-comment test (the entity-model promise).

## 9. Quality Engineering & Validation

- **Existing suites unchanged** — slice-001 + slice-003 engine tests green throughout
  (behaviour-preserving on the scaffold success path).
- **Transactional writer (D4):** a 2-file sub-artefact whose second file fails leaves the
  parent dir exactly as before (first file unlinked, parent untouched); a fully-successful
  write is byte-identical to today. Plus (findings 1–2): a pre-existing dir component
  survives rollback; a dir populated by another writer mid-call is not removed
  (`DirectoryNotEmpty`-tolerant); `create_new(true)` fails atomically on an existing target.
- **IP facet scaffold:** `slice plan <id>` writes `plan.{toml,md}` under an existing slice,
  refuses to clobber an existing plan, errors if the slice is absent.
- **`state.rs` runtime writer:** `init_phases` creates the state tree on demand (parent
  absent), writes `phase-NN.{toml,md}` per declared phase, is idempotent on the no-drift
  path (re-run adds new phases, never clobbers existing tracking or sheet edits); writes
  land *only* under `.doctrine/state/`. Drift (finding 5): a renamed/removed plan phase is
  **reported** as orphan/stale, not silently left live; `--prune` removes explicitly.
  Partial-init recovery (round-2): a phase with only its `.toml` (crash before `.md`) is
  completed on re-run — skip keys per file, not per phase. Duplicate plan phase ids error.
- **Input validation (findings 4, 6):** `phase_stem` enforces `PHASE-<digits>` (rejects
  empty / separator / `..` / leading-dot) before a phase id reaches the filesystem, and
  `init_phases` rejects duplicate phase ids in the plan (a dup would alias two phases onto
  one tracking file). Both are v1 — phase ids are consumed immediately, not graduated.
- **Symlink refresh (finding 10):** a wrong-target `phases` link is replaced; a real file/dir
  squatting the path errors (not silently tolerated).
- **`notes.md` scaffold:** `slice notes <id>` writes a durable `notes.md` under an existing
  slice, refuses to clobber an existing one, errors if the slice is absent (the `design.md`
  path, asserted for the second caller).
- **Edit-preserving status update:** `set_phase_status` sets fields + appends `[[progress]]`
  while preserving a hand-added comment and an unknown key (`toml_edit`).
- **Symlink-blind resolution:** with the convenience symlink deleted, `slice phase` still
  resolves and updates via the id-derived path.
- **Installer gitignore:** a fresh install appends `.doctrine/state/` + `.doctrine/slice/*/phases`;
  re-install is a no-op.
- Lint clean (deny-level), formatted.

## 10. Review Notes

Adversarial design review **complete** (gpt-5.5 hostile pass, adjudicated) — dispositions
in [audit.md](audit.md). Eleven findings; six gated revisions are folded into this doc:
transactional-writer rewrite (atomic `create_new` + component-wise `create_dir`, D4/§5.4,
findings 1–2), phase-sheet durability honesty + close-out-audit harvest (§5.1/§5.3/D5,
finding 3), shared `fsutil` + `phase_id` validation (D3/§5.2, finding 4), plan-drift report
in `init_phases` (§5.2/§5.4/§5.5, finding 5), id-immutability invariant (§5.5/Q-series,
finding 6), and the git-tracked-purity reword for `handover.md` (§5.1/§5.5, finding 7).
Clarifications: WHAT/HOW scope (slice-004.md, finding 8), Q4 additivity (finding 9), symlink
refresh (finding 10), build sequence §5.6 (finding 11). Status stays `proposed` — the
`ready` gate is the user's.

## References

- Slice contract: [slice-004.md](slice-004.md).
- Engine being extended: `src/entity.rs` (`Kind`/`MaterialiseMode`/`Artifact`/`materialise`,
  the `create_in_existing` writer); slice-003 design [003/design.md](../003/design.md);
  the inherited writer debt: [003/audit.md](../003/audit.md) §2 `[M]`.
- Runtime-state boundary: [entity-model.md](../../../doc/entity-model.md) § Runtime state,
  § The storage rule; [spec-entity-spec](../../../doc/spec-entity-spec.md)
  § Design-data vs runtime-state.
- Schema shapes (adapted, flattened, frontmatter/blocks dropped): schema bundle
  `plan.overview@v1`, `phase.tracking@v1`.
- Installer dirs + gitignore mechanism: `install/manifest.toml`, `src/install.rs`.
