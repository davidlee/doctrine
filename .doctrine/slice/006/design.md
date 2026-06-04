# Design SL-006: ADR support

## 1. Design Problem

Land doctrine's first governance entity ([slice-006.md](slice-006.md)): an
Architecture Decision Record with `new` / `list` / `status`. The non-trivial part
is *not* the engine — unlike SL-005 (memory's string identity forced an engine
generalisation), an ADR is numeric, reserved, top-level — **exactly the slice
shape**, so it rides `src/entity.rs` unchanged. The design problem is therefore one
of *discipline*, not mechanism:

1. **Do not fork the slice.** `src/adr.rs` will mirror `src/slice.rs` closely.
   The risk is parallel implementation (CLAUDE.md): copy-paste that drifts. The
   design must decide what is genuinely ADR-specific vs. what is already shared in
   `src/entity.rs`, and resist extracting a premature slice/ADR abstraction.
2. **The one new capability — authored-status mutation.** `adr status` mutates a
   **committed authored** toml in place. Every existing `toml_edit` mutation
   (`state::set_phase_status`) targets **runtime** state. This is the first verb
   that edit-mutates a committed file. The design must place it correctly w.r.t.
   the storage rule and decide whether it is shared with the (currently absent)
   slice status verb now or later. A key asymmetry (§ 7 D3, Q1/Q2):
   `set_phase_status` keeps an in-file progress log because runtime state is
   gitignored and git cannot see its transitions; an ADR toml is **committed**, so
   git history *is* its transition audit trail — an in-file log would duplicate git
   and violate the storage rule.

> This revision incorporates the adversarial design review (§ 10). The review's
> blocking finding — that mirroring slice's metadata-list helpers is real parallel
> implementation, not "two thin callers" — is **accepted**: D4 now extracts the
> already-identical substrate into a shared module. Its second blocking finding
> (governance transitions need a trail) is **resolved differently than proposed**:
> git history is the trail for a committed entity (above); v1 documents permit-any +
> convention and adds a no-op write guard, rather than an in-file log.
3. **Where v1 stops.** Status-in-toml not symlink dirs; forward-ref relationship
   fields authored-but-inert; no supersede, no boot listing. The design fixes
   these seams so the follow-ups are additive.

## 2. Current State

`src/entity.rs` is a kind-agnostic scaffolding engine, proven across slice /
design / plan / phases / memory (SL-003/004/005). Post-SL-005 its identity model
admits both numeric and named entities:

- `Kind { dir, prefix, scaffold }` — `prefix` renders `SL` → `SL-003` (the
  `{{ref}}` token); `scaffold: fn(&ScaffoldCtx) -> Fileset`.
- `MaterialiseRequest::{Fresh, InExisting{id}, Named{name}}` — placement at call
  time. ADR uses **`Fresh`**, the numeric-allocate path (`candidate_id` = `max+1`,
  claim in a bounded race-retry loop).
- `ScaffoldCtx { eid: EntityId, slug, title, date }`; `EntityId::Numbered{id,
  canonical}`; `ctx.numbered()` yields `(id, canonical)`.
- `Artifact::{File, Symlink}`; `write_fileset` transactional + sole path→fs joiner
  (H1); clobber-refused for sub-artefacts.
- `entity::materialise(&KIND, &LocalFs, &root, &request, &Inputs)` → `Materialised
  { eid, dir }`.

`src/slice.rs` is the reference numeric caller: `SLICE_KIND` (`Fresh`, 2-file +
symlink), `render_toml`/`render_md` (`asset_text` + `{{token}}` replace),
`Meta{id,slug,title,status}` + `read_meta`/`read_metas`/`sort_and_filter`/
`format_list`, `run_new`/`run_list`, `today()` clock-in-shell. `slice new`'s only
nuance over a plain Fresh is its slug-resolution *policy* (`--slug` override else
`derive_slug`).

`src/state.rs::set_phase_status` is the lone `toml_edit` mutator: parse
`DocumentMut`, `table.insert("status", …)` + timestamps + append a progress row,
write back — but against **runtime** `phase-NN.toml`, with started/completed/
progress bookkeeping ADR does not have.

Templates are embedded via `rust_embed` over `install/` (`src/install.rs:16`);
`asset_text("templates/foo")` reads them. `install/templates/` holds slice/design/
plan/phase/notes; ADR adds two. The slice `[relationships]` table is authored-but-
inert — the precedent for ADR's reserved relationship fields.

`time` is in the enabled feature set (used by `slice::today`). No engine change,
no new dependency.

## 3. Forces & Constraints

- **No parallel implementation** (CLAUDE.md; reservation-spec § Code seam): ADR
  rides the engine; `src/adr.rs` reuses every engine primitive. The
  metadata-list substrate slice already has (`Meta`, `sort_and_filter`,
  `format_list`, `read_metas`, `today`) is **status/path-parametric, not slice-
  specific** — four of the five carry zero slice knowledge and `today()` is
  identical. So it is **extracted to a shared module now**, not mirrored (§ 7 D4);
  the only genuinely per-kind code is the scaffold/render fns and `set_adr_status`.
- **Storage rule** (CLAUDE.md): structured data in TOML, prose in MD, never queried
  data in prose. ADR status is queried (`list --status`) → lives in toml. The
  spec-driver frontmatter-in-md shape is rejected for this reason (§ 7 D1).
- **Behaviour-preservation gate**: the engine is untouched; entity/slice/state
  suites must stay green unchanged. ADR adds only new code paths.
- **Pure/imperative split** (slices-spec § Architecture): no clock in the pure
  layer — the date is an *input*, as in slice. `today()` is in the shell.
- **Generalise only as far as forced** — applied twice, cutting both ways: the
  *list substrate* is already-identical code (slice wrote it, ADR needs the same),
  so lifting it is forced *now* (not speculative — § 7 D4). The *status-mutation
  primitive* has one consumer (ADR; slice's status verb does not exist yet), so it
  stays local *now* (§ 7 D3); the follow-up slice verb is the second consumer that
  earns its extraction. Lift identical code; do not abstract single-consumer code.
- **Append-only ledger semantics** (the *point* of ADRs): decisions are retired by
  supersession, not edited away. v1 honours this by keeping the `[relationships]`
  forward-ref fields present so F1 supersede is additive (§ 5.3) — but does not yet
  enforce immutability of accepted records (Non-Goal; § 6 Q2).
- **Provisional status vocabulary**: `AdrStatus` membership may change pre-harden;
  the model must not depend on exact members.

## 4. Guiding Principles

- **A new kind is data, not a subsystem.** The engine's whole thesis is that a new
  numeric entity is a `Kind` const + a scaffold fn + templates. ADR proves it: the
  *only* genuinely new logic is `run_status`'s authored mutation; everything else
  is a parameterisation of paths already walked by slice.
- **Lift identical code, mirror only what's per-kind.** The status/path-parametric
  list substrate moves to a shared module (single-sourced `Meta` + list helpers,
  § 7 D4); what stays in `src/adr.rs` (scaffold, render, `set_adr_status`) reads as
  a sibling of `src/slice.rs` — same section order — because it is genuinely
  per-kind, not because we're tolerating duplication.
- **Status is authored truth; everything else derives from it.** The toml `status`
  is the single source. `list --status` filters it; the F2 symlink index would
  *rebuild* from it; the F1 reverse-link would *derive* from forward refs. No
  second store of the same fact (spec-driver ADR-002).

## 5. Proposed Design

### 5.1 System Model

ADR is a top-level `Fresh` numeric kind. `src/entity.rs` owns materialisation
(unchanged); a **shared metadata-list module** owns the `Meta` reader + filter +
format (lifted from `slice.rs`, now used by both); `src/adr.rs` owns only the
ADR-specific scaffold/render/`set_adr_status`. Three CLI verbs, two templates,
three `main.rs` arms. No engine change, no state-tree, no gitignore change.

```
.doctrine/adr/
  001/
    adr-001.toml      # authored metadata (queried)
    adr-001.md        # authored prose (Context/Decision/Consequences/…)
  001-<slug> -> 001   # human alias symlink
  002/ …              # monotonic id, max+1
```

All three files are authored/committed. No `.doctrine/state/adr` (status is
authored, not runtime). No symlink-by-status dirs in v1 (F2).

### 5.2 Interfaces & Contracts

CLI (clap), mirroring `SliceCommand`:

```
doctrine adr new [TITLE] [--slug S] [-p ROOT]
doctrine adr list [--status S] [-p ROOT]
doctrine adr status <ID> --status <proposed|accepted|rejected|superseded|deprecated> [-p ROOT]
```

- **`adr new`** → `entity::materialise(&ADR_KIND, &LocalFs, &root,
  &MaterialiseRequest::Fresh, &Inputs{slug,title,date})`. Title resolved via the
  shared `resolve_title` policy (arg else stdin prompt; non-empty). Slug: `--slug`
  override else `entity::derive_slug(title)`. Prints `Created ADR NNN: <dir>`.
- **`adr list`** → shared `read_metas(adr_root, "adr")` → `sort_and_filter(rows,
  status)` → `format_list` → stdout. Row = `NNN status slug title`. All three are
  the shared module's functions (the toml stem `"adr"` vs `"slice"` is the only
  call-site difference; D4).
- **`adr status`** → `set_adr_status(root, id, status, today())`: read
  `adr-NNN.toml`, and **only if status differs**, `toml_edit` set `status` +
  `updated`, write back. A no-op transition produces no write and no diff (truly
  idempotent). Errors if the ADR dir is absent. No in-file transition log — git
  history of the committed toml is the audit trail (§ 1.2, Q1/Q2).

Engine contract used: `Fresh` allocation, transactional writer, alias symlink.
No new engine API.

### 5.3 Data, State & Ownership

`adr-NNN.toml` (authored; `Meta` reads a subset, unknown keys preserved on disk):

```toml
schema  = "doctrine.adr"
version = 1
id      = {{id}}
slug    = "{{slug}}"
title   = "{{title}}"
status  = "proposed"
created = "{{date}}"
updated = "{{date}}"

[relationships]   # reserved; authored-but-inert in v1 (slice precedent)
supersedes    = []   # forward ref → ADR(s) this retires (F1 writes this)
superseded_by = []   # reverse; F1 decides store-vs-derive (prefer derive, ADR-002)
related       = []
tags          = []
```

`Meta { id, slug, title, status }` — the reader struct, identical fields to
`slice::Meta`; serde ignores the `[relationships]` table and `schema`/`version`.

`adr-NNN.md` (authored prose), tokens `{{ref}}` (= `ADR-NNN`) + `{{title}}`:
sections ported from spec-driver `supekku/templates/ADR.md` — `# {{ref}}:
{{title}}`, Context, Decision, Consequences (Positive/Negative/Neutral),
Verification, References. **No frontmatter** (metadata lives in the toml).

Ownership: `status` and `updated` are **tool-owned** — mutated via `adr status`
(and F1 supersede). Hand-editing `status` is supported (it is plain authored toml)
but discouraged: prefer the verb so `updated` stays coherent. `id`/`slug`/`created`
are write-once at scaffold (no `adr rename` in v1 — § 6 Q5). The `.md` body is
hand-authored after scaffold; the tool never rewrites it.

Audit trail: every `status`/`updated` change lands in a reviewed git commit — the
committed toml's history *is* the transition ledger. No in-file log (that would
duplicate git and re-store derived data in an authored file). This is the
deliberate asymmetry with `set_phase_status`, whose log exists only because runtime
state is gitignored (§ 1.2). Caveat: `updated` is a mild merge-conflict magnet
across branches — acceptable for a low-frequency governance artifact.

### 5.4 Lifecycle, Operations & Dynamics

```
adr new   → status=proposed   (scaffold; id=max+1; clobber-safe via Fresh claim)
adr status → proposed → accepted          (decision ratified)
                      → rejected           (decision declined)
            accepted  → superseded         (F1 supersede sets this + the link)
                      → deprecated          (obsolete, not replaced)
```

v1 permits any status→any status transition (no state-machine enforcement — § 6
Q1). `updated` bumps on every *effective* transition; a no-op (same status) writes
nothing. `created` never changes. The `.md` prose and the toml `status` are edited
independently — the tool owns status, the human owns prose.

### 5.5 Invariants, Assumptions & Edge Cases

- **I1** `id` monotonic `max+1`, gaps not backfilled (engine `candidate_id`).
- **I2** the alias symlink `NNN-slug` is convenience, not authority — the tool
  resolves ADRs by numeric id, never by slug (slice precedent).
- **I3** `adr status` on a missing id is a hard error (no implicit create).
- **I4** authored `[relationships]` rows are inert in v1 — no FK validation, no
  resolution (slice `[relationships]` posture).
- **I5** `adr status` is a true no-op when the target equals the current status:
  no write, no `updated` bump, no diff.
- **I6** F2 forward-compat: F2's future `<status>/` symlink dirs sit *beside* the
  numeric dirs under `ADR_KIND.dir` and are gitignored; `entity::scan_ids` already
  skips non-numeric dir names, so they cannot be miscounted as ADR ids. v1's flat
  `.doctrine/adr/<NNN>/` layout does not foreclose F2 (no restructure needed).
- **Edge:** `adr new` with an empty/symbol-only title → slug derivation fails →
  error asking for `--slug` (slice behaviour, reused `resolve_title`/`derive_slug`).
- **Edge:** concurrent `adr new` → engine race-retry loop arbitrates (same as
  slice; engine-tested).
- **Edge:** `adr status --status` with an out-of-enum value → clap rejects at parse
  (ValueEnum), no file touched.

## 6. Open Questions & Unknowns

- **Q1 — transition validation. RESOLVED: permit any transition in v1.** The
  review flagged "any→any" as a footgun for a governance entity. Resolution: permit
  any, because (a) the toml is committed, so every transition is a reviewed,
  attributable, revertible git commit — a fat-finger is visible and recoverable,
  unlike a silent runtime mutation; (b) a state-machine is governance *policy*, not
  entity mechanism, and belongs with F1 (where `superseded` gains a real link to
  protect). v1 ships the no-op guard (I5) so accidental re-runs leave no trace.
  Minimal terminal guards (e.g. refuse transitions *out of* `superseded` once F1
  links exist) land with F1.
- **Q2 — accepted-record immutability. RESOLVED: convention in v1.** ADRs are
  append-only-by-convention (edit → supersede). v1 does not lock accepted records;
  git history + review are the enforcement surface. Hard enforcement is a
  governance feature layered later, not an entity invariant.
- **Q3 — `new --status`.** Allow seeding a non-`proposed` status at creation (e.g.
  importing an already-accepted decision)? *Lean: defer*; v1 always scaffolds
  `proposed`, transition with `adr status`. Cheap to add later.
- **Q4 — id display width.** `{:03}` like slices (`ADR-001`)? *Lean: yes*, for
  cross-entity visual consistency.
- **Q5 — slug refinement.** Titles/slugs get refined while `proposed`, but slug is
  write-once (no `adr rename`); the alias symlink can go stale. *Lean: out of v1*
  (slice has the identical posture). Recorded as a deferral, not a gap; an
  `adr rename` (retitle + reslug + relink) is a future verb for both ADR and slice.

## 7. Decisions, Rationale & Alternatives

- **D1 — toml+md split, not frontmatter-in-md.** *Decision:* metadata in
  `adr-NNN.toml`, prose in `adr-NNN.md`. *Rationale:* storage rule forbids queried
  data in prose; `status` is queried. *Alternative (spec-driver):* one
  `ADR-NNN-slug.md` with YAML frontmatter — rejected: needs a frontmatter reader
  the engine lacks, and violates the storage rule. *Cost:* diverges from the
  universal ADR-as-single-file convention; accepted for doctrine-internal
  consistency.
- **D2 — top-level `Fresh` kind, no engine change.** *Decision:* `ADR_KIND` is a
  `Fresh` numeric kind beside `SLICE_KIND`. *Rationale:* ADR identity is numeric +
  reserved + top-level — the slice shape exactly; the engine already serves it.
  *Alternative:* nest ADRs under slices — rejected: ADRs are project-global
  governance, not slice-scoped.
- **D3 — `set_adr_status` local to `adr.rs`, not shared with `state` yet.**
  *Decision:* implement the authored `toml_edit` mutation in `src/adr.rs`.
  *Rationale:* one consumer; ADR's mutation differs from `set_phase_status` (no
  progress log, no started/completed, authored not runtime target). *Alternative:*
  extract a shared "set authored toml status+timestamp" primitive now — rejected:
  premature (generalise-only-as-forced). The follow-up slice-status verb is the
  second consumer that earns it.
- **D4 — extract the shared metadata-list substrate now (revised post-review).**
  *Decision:* lift `Meta`, `sort_and_filter`, `format_list`, `read_metas`, and
  `today` out of `slice.rs` into a shared module; slice and ADR both call it.
  `read_metas(tree_root, stem)` is parameterised by the toml stem (`"slice"` /
  `"adr"`) — note the stem (`"adr"`) is distinct from `Kind.prefix` (`"ADR"`), so it
  is a separate argument, not reused. *Rationale:* these functions are
  status/path-parametric — four of five carry **zero** slice knowledge and `today()`
  is byte-identical; mirroring them is the parallel-implementation CLAUDE.md
  forbids, and would let `format_list`/`sort_and_filter` drift (e.g. a future
  `--format=tsv` landing in one). Lifting already-identical code is *forced* DRY,
  not speculative abstraction. *What is NOT extracted:* a `numeric_entity`
  trait/generic — rejected as the over-abstraction the review agrees to avoid; only
  the concrete functions move. *What stays per-kind in `adr.rs`:* `adr_scaffold`,
  the two render fns, `set_adr_status`. *Behaviour-preservation:* slice's suites
  gate the lift (mechanical, observable behaviour unchanged). *Module siting:* a
  small dedicated module (e.g. `src/meta.rs` / `src/listing.rs`) rather than
  `entity.rs` — `format_list` is CLI presentation, not engine; keep the engine
  free of presentation (final name decided in plan).
- **D5 — status authored, not symlink-indexed.** *Decision:* `list --status`
  filters the toml field. *Rationale:* slice idiom; symlink-by-status is a derived
  index (F2), not the source of truth. *Alternative:* spec-driver's
  `rebuild_status_symlinks` now — deferred (F2); v1 needs no derived tier.

### Follow-ups locked as seams (not built)

- **F1 `adr supersede <new> <old>`** — two-file authored `toml_edit`: `supersedes`
  on new, `superseded_by` + `status=superseded` on old; reverse link derived not
  stored (ADR-002). Reuses D3's mutation; the second consumer that extracts it.
- **F2 symlink-by-status index** — `adr reindex` rebuilding `.doctrine/adr/<status>/`
  from toml; gitignore the derived subtree; `adr list --format=tsv`.
- **Governance boot listing** — when doctrine grows a boot generator, inject
  accepted ADRs via the `--format=tsv` seam (port of `admin preboot`).

## 8. Risks & Mitigations

- **R1 — copy-paste drift between `slice.rs` and `adr.rs`. RETIRED by D4.** The
  list substrate is single-sourced in the shared module, so there is no second copy
  to drift. The only remaining per-kind parallelism (scaffold/render fns) is
  genuinely kind-specific and small; review covers it.
- **R2 — authored toml mutation corrupts hand-edits.** *Mitigation:* `toml_edit`
  (edit-preserving: comments/unknown keys/formatting intact), proven in
  `set_phase_status`; round-trip test asserts preservation.
- **R3 — status vocabulary churn.** *Mitigation:* `AdrStatus` is a closed
  `ValueEnum` in one place; membership change is a one-line edit; model logic does
  not branch on specific members.
- **R4 — scope creep into supersession/governance.** *Mitigation:* Non-Goals
  fixed in the scope doc; F1/F2 seams pre-shaped so deferral is cheap.

## 9. Quality Engineering & Validation

- **Shared-module tests (moved with the code):** `Meta` round-trip,
  `sort_and_filter` status filter, `format_list` alignment, `read_metas` over a stem
  — these tests **move from `slice.rs`** into the shared module's test block (one
  home, both callers exercised). Slice's remaining tests stay green unchanged
  (behaviour-preservation).
- **Unit (pure), ADR-specific:** `adr_scaffold` layout (toml + md + symlink, 3
  artifacts); token substitution in both templates; `derive_slug` reuse.
- **`set_adr_status` round-trip:** scaffold → set accepted → re-read asserts
  `status=accepted` + `updated` bumped + `[relationships]`/comments preserved;
  set to the **same** status asserts no write / no diff (I5 no-op guard);
  missing-id errors.
- **Engine reuse (no new engine tests):** `Fresh` allocation, transactional
  rollback, race-retry are already covered in `entity.rs` — ADR inherits them.
- **Behaviour-preservation:** full `cargo test` green, **existing suites
  unchanged**.
- **End-to-end (real binary, scratch repo):** `adr new` ×2 (assert 001/002,
  monotonic) → `adr list` → `adr status 1 --status accepted` → `adr list --status
  accepted` (assert only 001).
- **Gate:** `cargo clippy` zero warnings; `just check` before commit.

## 10. Review Notes

**Round 1 — adversarial design review (second agent), verdict amber → resolved.**
Two blocking findings, both addressed:

- **D4 (parallel implementation).** Review: mirroring `Meta`/`sort_and_filter`/
  `format_list`/`today`/`read_metas` is real duplication (4 of 5 status-agnostic,
  `today()` identical), not "two thin callers." **Accepted** — D4 reversed to
  extract the substrate into a shared module now; only scaffold/render/`set_adr_status`
  stay per-kind. Retires R1. (Review's caution against a `numeric_entity` *trait*
  is honoured — only concrete functions move.)
- **Q1/Q2 (governance transitions need a trail).** Review proposed an in-file
  transition log mirroring `set_phase_status`. **Resolved differently:** the ADR
  toml is committed, so git history is the trail; an in-file log would duplicate git
  and re-store derived data in an authored file (storage rule). `set_phase_status`
  logs in-file only because runtime state is gitignored. v1 = permit-any +
  convention + no-op write guard (I5); state-machine/immutability deferred to F1.

Non-blocking, all folded in: no-op idempotency guard (I5), tool-owned status/
`updated` + merge-churn caveat (§ 5.3), `adr rename` deferral (Q5), F2 layout
forward-compat (I6). Confirmed sound as written: D1 (toml+md split), D2 (Fresh
kind), D3 (`set_adr_status` local), D5 (status authored not symlink-indexed).

_Optional round 2: codex MCP pass before plan (separate billing — ask first).
Otherwise decisions are locked; ready for `slice plan`._
