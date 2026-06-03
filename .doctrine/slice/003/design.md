# Design SL-003: Entity-scaffold engine + design-doc sibling

Design doc for slice-003, structured to the `doctrine slice design` template (the
tool this slice builds). Pure prose (the doctrine design-doc shape: no
frontmatter, no fenced data blocks — entity-model.md). Hand-authored: the
scaffolder does not exist yet, so this first one is written by hand; later design
docs are scaffolded from the same skeleton.

## 1. Design Problem

Extract the kind-agnostic directory-entity machinery out of `src/slice.rs` into a
small engine, **without freezing its abstraction boundary against a single
caller** — the mistake slice-002 made. drift (third caller) and spec (fourth,
per-subtype filesets) need this machinery but are registry-gated (deferred), so
they cannot shape the boundary now. The problem is to find a *second real caller*
that exists today and is shaped *differently* enough to force the boundary right.

The roadmap supplies one: the slice's own **design-doc sibling** — a sub-artefact
(non-reserved, prose) under an existing slice dir, the opposite shape to the
top-level reserved 2-file-plus-symlink slice. Extracting against both at once is
the point: the generalisation is corrected *by use during this slice*, not
anticipated. It **supersedes slice-002**.

## 2. Current State

`src/slice.rs` holds everything inline: `candidate_id`, `derive_slug`,
`build_scaffold` (a slice-specific `Scaffold` struct with hardcoded
`toml_path`/`md_path`/`symlink`), `scan_ids`, `reserve_create` (the claim loop
with `fs::create_dir` + `ErrorKind::AlreadyExists` inlined), `write_scaffold`,
`read_metas`, and the CLI verbs (`new`, `list`). slice-002 proposed extracting
this against the single slice caller and froze the boundary wrong: a fixed
toml+md pair, when a spec subtype needs a per-subtype fileset of ~13 files.

The reservation primitive is real but *nominal*: the claim is an inlined
`fs::create_dir`, not an `acquire` seam, so the "swap the backend, not the
caller" promise (reservation-spec) holds only on paper until the seam exists in
code.

## 3. Forces & Constraints

- **Behaviour-preserving.** The slice-001 test suite passes unchanged throughout
  — the extraction is a refactor, not a rewrite. It is the gate at every step.
- **Pure/imperative discipline** (slices-spec § Architecture): candidate id, slug,
  render, fileset are pure over inputs; only the `acquire` syscall and IO sit in
  the shell, behind a seam, asserted without disk.
- **No premature trait.** Lift *only* the one `acquire` method, not the full
  `LeaseBackend`; parameterise *file mechanics*, not a shared `Meta` (Non-Goals).
- **Project gate.** `cargo test` + `cargo clippy` (deny-level) + `cargo fmt`.
- **Shared seams reused** (`crate::root`, `crate::install::asset_text`,
  `&mut dyn Write`), not duplicated.
- **Out of scope.** IP/phase siblings (relational block + runtime state), design
  *review* (`RVW-`) and any block-bearing sibling, the `git-ref` backend / full
  `LeaseBackend`, drift and spec entities. This slice only makes the engine *fit
  to host* them.

## 4. Guiding Principles

- **Extract against two callers of different shape, not one.** The boundary is
  corrected by real use this slice (slice + design-doc), not by anticipation.
- **Fileset is a function, not a fixed pair.** Each kind supplies its own set; the
  engine never hardcodes a file count (slice-002 M3).
- **Only the `acquire` seam now.** Lift the one method that makes the backend swap
  real; resist pulling the full trait or the `git-ref` backend (over-build risk).
- **Parameterise file mechanics, not metadata.** No shared `Meta` until a second
  metadata-bearing *reader* proves the shape.
- **Defer stays deferred.** Findings make the engine *fit to host* drift/spec;
  they are not a licence to build a deferred entity. Roadmap moves go via
  supersede, not scope creep.

## 5. Proposed Design

### 5.1 System Model

Three pieces over one engine: the **`acquire` seam** (the one impure-critical
claim, behind a trait), the **`Kind` descriptor** (data — fileset-as-function +
optional reservation — what differs between kinds), and the **`materialise`
loop** that drives them. A slice and a design doc are two `Kind` values; the
engine is kind-blind. drift and spec later drop in as further `Kind`s for their
**initial scaffold** — spec's row/prose split, FK validation, derived
relationships, and registry integration remain separate callers/features, not
part of this engine (M3). "Engine supports spec" ≠ "spec is mostly done."

### 5.2 Interfaces & Contracts

The claim becomes a one-method trait so the `git-ref` backend drops in later
without a Kind-caller rewrite:

```rust
pub enum Acquired { Won, AlreadyHeld }

pub trait Reservation {
    /// Atomic, exclusive claim. `Won` if this caller created it; `AlreadyHeld`
    /// if another agent won the race. Only this op arbitrates the race.
    fn acquire(&self, claim: &Path) -> anyhow::Result<Acquired>;
}

pub struct LocalFs;
impl Reservation for LocalFs {
    fn acquire(&self, claim: &Path) -> anyhow::Result<Acquired> {
        match fs::create_dir(claim) {
            Ok(()) => Ok(Acquired::Won),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(Acquired::AlreadyHeld),
            Err(e) => Err(e).with_context(|| format!("Failed to claim {}", claim.display())),
        }
    }
}
```

`LocalFs` is the only impl in this slice — the `mkdir` of slices-spec § local
backend, lifted verbatim out of `reserve_create`. Behaviour is identical, so the
slice-001 retry test stays green.

A `Kind` is *data*, not a trait — one dispatch site, no per-kind state:

```rust
pub struct Kind {
    /// Entity-tree root, relative to the project root, e.g. ".doctrine/slice".
    pub dir: &'static str,
    /// Canonical-id prefix, e.g. "SL" → "SL-003" (the `{{ref}}` token).
    pub prefix: &'static str,
    /// How the entity is placed — a closed enum, not a `bool`, so a third mode
    /// is a compiler-forced new variant, never an overloaded `false` (M1).
    pub mode: MaterialiseMode,
    /// Fileset as a function — NOT a fixed toml+md pair. A slice yields toml +
    /// md + symlink; a design doc yields one prose file; a spec yields its
    /// subtype's set. This is the slice-002 fix (M3-prior).
    pub scaffold: fn(&ScaffoldCtx) -> anyhow::Result<Fileset>,
}

pub enum MaterialiseMode {
    /// Allocate a fresh reserved id under `dir` (slice, later spec).
    AllocateFreshEntity,
    /// Create file(s) in an existing parent entity (design doc, later phases).
    CreateInExistingEntity,
}

pub struct ScaffoldCtx<'a> {
    pub dir: &'a Path,         // the entity dir (new numeric dir, or existing parent)
    pub id: u32,
    pub canonical_id: &'a str, // "SL-003" — the {{ref}} token (D-Q2)
    pub slug: &'a str,
    pub title: &'a str,
    pub date: &'a str,
}

// Artifact paths are RELATIVE to the entity-tree root (`Kind.dir`). The engine
// is the sole joiner: it rejects absolute paths and any `..` that escapes the
// tree before any write (H1). The slug symlink sits at the tree root *beside*
// the numeric dir (`003-slug`), which is why the base is the tree root, not the
// entity dir — a descriptor still cannot reach outside `Kind.dir`.
pub enum Artifact {
    File { rel_path: PathBuf, body: String },
    Symlink { rel_path: PathBuf, target: String },
}
pub type Fileset = Vec<Artifact>;
```

`Fileset` generalises today's `Scaffold` struct: the slice's hardcoded
toml/md/symlink fields become three `Artifact`s (the `003/slice-003.toml`,
`003/slice-003.md`, and the `003-slug` symlink, all tree-root-relative); a design
doc returns one `File`. The engine writes `Artifact`s uniformly (the symlink
keeps today's `AlreadyExists`-tolerant create).

The CLI contract gains one verb: `doctrine slice design <id>` (D6), scaffolding
`design.md` into the resolved slice dir.

### 5.3 Data, State & Ownership

| Path | Current | Target |
|---|---|---|
| `src/slice.rs` | all machinery + slice CLI inline; `Scaffold` struct; `reserve_create` inlines mkdir | the **slice `Kind`** (dir, mode=`AllocateFreshEntity`, scaffold fn building toml+md+symlink artifacts), the slice `Meta`/list, and thin CLI wiring; calls the engine |
| `src/entity.rs` (new) | — | the engine: `candidate_id`, `scan_ids`, `materialise` loop, `write` (artifacts), `Kind`/`ScaffoldCtx`/`Artifact`/`Fileset`, `Reservation`/`Acquired`/`LocalFs`, `derive_slug` |
| `install/templates/` | `slice.toml`, `slice.md` | + `design.md` template (prose, `{{ref}}` + `{{title}}`) |
| slice CLI | `new`, `list` | + `design <id>` |

`derive_slug`, `candidate_id`, `scan_ids`, `today`-injection and the symlink
write are kind-blind → they move to the engine. The pure `derive_slug` *helper*
moves; the slug *resolution policy* (use `--slug` else derive, bail if empty)
stays CLI-side — it is slice-Kind-specific (a design doc has no slug of its own).
`Meta`/`read_metas`/`format_list` are slice-specific *reading* (not scaffolding) →
they stay slice-side until a second metadata-bearing caller proves a shared shape
(Non-Goal: no premature `Meta` trait). The design-doc verb *reads* the parent
slice `Meta` for its title — confirming the read stays slice-side.

**Deferred ownership (not built here).** The design doc carries structured data
(`date`, key files / globs, governance-doc relationships) that entity-model.md's
storage rule puts in a sister TOML facet, not prose. Held to a follow-up slice
via supersede (D5). Design-doc *approval* is **slice state**, not a facet field —
it gates planning, so it lives in the slice lifecycle (`status`, later
`.doctrine/state/`). Structured adversarial review is a future `RVW-` entity.

### 5.4 Lifecycle, Operations & Dynamics

The engine loop is `reserve_create` generalised: the claim is now
`reservation.acquire`, the output is now `kind.scaffold`, and the non-reserved
branch is new (the design-doc path that proves the engine spans both shapes).

This engine **materialises filesets**. It does not append rows, mutate TOML
tables, or allocate row-local ids — those (e.g. a spec's `FR-001` requirement
rows) are a different caller with its own load-time duplicate detection (L1).

```text
materialise(kind, reservation, ctx_inputs):
  create_dir_all(kind.dir)                     # entity-tree root; the non-recursive
                                               # claim mkdir below needs it to exist
  match kind.mode:
    AllocateFreshEntity:                       # reserved top-level (slice, spec)
      loop (bounded):
        id   = candidate_id(scan(kind.dir))
        dir  = kind.dir / format!("{id:03}")
        ref  = format!("{}-{id:03}", kind.prefix)   # canonical id, e.g. "SL-003"
        match reservation.acquire(dir):        # local: mkdir == dir creation
          Won =>
            # Won ⟹ we just created `dir` ⟹ it is ours; on any write failure
            # remove it so a partial scaffold never leaves a ghost entity (H2).
            write_all(kind.scaffold(ctx(dir, id, ref, ...))?)
              or on error: remove_dir_all(dir); propagate the original error
            return dir
          AlreadyHeld => continue              # lost race; recompute
    CreateInExistingEntity:                    # sub-artefact: no claim, no id alloc
      dir = resolve_existing(kind.dir, id)     # the parent dir; err if absent
      ref = caller-supplied parent ref         # sub-artefact has no own id/ref
      refuse if a target file already exists   # no silent clobber, file-creating
                                               # sub-artefacts only (D7)
      write_all(kind.scaffold(ctx(dir, id, ref, ...))?)
```

For the future `git-ref` backend this H2 cleanup does *not* apply: there the
claim is a ref, the entity dir is created separately, and an abandoned claim is a
harmless reserved gap (reservation-spec) — a missing entity is normal. The
local "dir-is-claim" compromise (D1) collapses that separation, so for *local*
v1 a failed write must clean up or it becomes a malformed entity, not a gap.

**Build sequence (each step green against the slice-001 suite):**

1. **`acquire` seam, in place.** Add `Reservation`/`Acquired`/`LocalFs`; rewrite
   `reserve_create`'s claim against it. Pure refactor; suite green. Cheap and
   valuable even if the slice stopped here (reservation-spec § Code seam).
2. **Engine module `src/entity.rs`.** Move the kind-blind pure fns + the
   `materialise` loop + `Artifact` write behind `Kind`/`ScaffoldCtx`. The slice
   `Kind` reproduces today's exact two-file + symlink output. Suite still green.
3. **Design-doc `Kind` + `design.md` template.** The non-reserved, single-file
   kind. Engine unit test for the non-reserved path.
4. **Wire `doctrine slice design <id>`.** Thin CLI verb over the engine.

### 5.5 Invariants, Assumptions & Edge Cases

- **Behaviour preservation.** The slice `Kind` reproduces today's exact byte
  output; the slice-001 suite is the executable invariant.
- **Race arbitration.** Only `acquire` arbitrates; the retry loop is bounded
  (`MAX_CLAIM_RETRIES`), exhaustion is a loud error.
- **Symlink tolerance.** The slug symlink keeps today's `AlreadyExists`-tolerant
  create.
- **Path containment (H1).** Artifact paths are relative to the entity-tree root
  (`Kind.dir`); the engine rejects absolute paths and any `..` that escapes the
  tree *before* writing, and is the only code that joins a descriptor path to the
  filesystem. A bad descriptor cannot write outside its tree.
- **Scaffold purity (M4).** `Kind.scaffold` is pure over `ScaffoldCtx` plus
  compile-time-embedded template text (`crate::install::asset_text` is rust-embed,
  not disk IO). It must not read disk, the clock, git, or resolve the project
  root; all such IO sits in the shell before/after scaffold (the clock via
  `today()` → `ctx.date`). Its only fallibility is template presence/format.
- **No ghost entities (H2).** In `AllocateFreshEntity`, a `Won` claim means the
  dir is ours, so a write failure removes it and propagates the error — no
  half-written entity survives to break `slice list` (which `scan_ids` would count
  but `read_metas` then fail to parse).
- **Write policy lives in the writer/branch, not in `Artifact` data.** The
  reserved branch writes freely (the dir was just won, fresh — nothing to
  clobber); the existing-parent branch refuses a pre-existing target (D7). Noted
  coupling: no-clobber currently rides on `mode`, not a per-write flag — fine
  while `AllocateFreshEntity` ⟺ fresh-dir, revisit if a kind ever wants the other
  pairing.
- **Parent creation.** `materialise` ensures `kind.dir` exists (the first-ever
  entity case) before the non-recursive claim `mkdir`. The File-writer does
  `create_dir_all(path.parent())` so a future nested fileset needs no engine
  change.

## 6. Open Questions & Unknowns

- **Q1 — `Kind` registry.** Two kinds can be two `const Kind`s referenced
  directly. A lookup table (`&str -> &Kind`) is only needed when a generic
  `doctrine <kind> new` dispatch exists; not this slice. Leave direct.
- **Q3 — Does `list` ever go kind-generic?** `slice list` reads slice `Meta`.
  drift/spec will want their own `list`. Deferred until the second reader; the
  engine owns scaffolding only for now (no premature `Meta`).
- **Q4 — Abstract rollup families vs `Kind.prefix`.** One `prefix` per `Kind`
  fits a *concrete* kind. An abstract **rollup** — spec (product/tech/revision),
  or backlog (idea/issue/risk/chore) — is N concrete subtype Kinds, each with its
  own `dir` / `prefix` / numbering / reservation namespace, plus a rollup *view*;
  it is **not** one polymorphic Kind. So `prefix` stays per-concrete-kind and the
  rollup is a list/query concern (Q3), not a scaffold Kind. This is the settled
  spec-family decomposition (entity-model.md: one model, three subtypes, own
  folders) recurring for backlog. Recorded so per-kind `prefix` is not mistaken
  for a per-family one.
- **Q5 — Design-doc presence is workflow-significant but not yet observable
  (M5).** The slices-spec rule makes a design doc default/mandatory (except
  trivial + explicit approval), but no command sees whether a slice has one —
  a slice can be `ready` with no `design.md` and nothing surfaces it. **No gate
  this slice** (`slice list`/resolution stay unaffected). Deferred to a future
  `doctrine slice validate`: a non-trivial slice has a `design.md` or an explicit
  trivial/no-design marker (a TOML field — *queryable lives in TOML, not prose*,
  entity-model.md); `design.md` is never parsed for headings (templates are
  defaults, not contracts); the design-doc facet, when it lands, carries the
  queryable metadata. Recorded so the invariant is not lost between slices.

## 7. Decisions, Rationale & Alternatives

- **D1 — `acquire` seam is path-based (`&Path`), not the abstract `key: &str`
  yet.** For local, the claim path *is* the entity dir — the `mkdir` both claims
  and creates. Faithful to today's behaviour, minimal. *Trade-off:* `git-ref`
  claims a ref, not a dir, so when it lands the seam generalises to an abstract
  key and the dir-creation splits out of `acquire` into the materialise step. That
  evolution is additive (a second impl + splitting one call); it does not rewrite
  callers, which is the whole point of having the seam. **Reconciliation:**
  reservation-spec § Code seam writes the seam as `acquire(&self, key: &str)`;
  this slice lands it path-based on purpose (an abstract key now would force
  `LocalFs` either to know the entity layout — the coupling the primitive forbids
  — or to claim a separate lock tree, diverging from today's "the dir *is* the
  claim"). "Callers" here means the **Kind** callers (slice, design-doc): they
  invoke `materialise`, never `acquire`, so the later `&Path`→key generalisation +
  dir-creation split is *engine-internal* (the loop + the impls), and the Kinds
  are untouched — the F1 hazard (slice-002) is avoided. The spec is reconciled.
  *Reservation namespace (M2 — deferred, not now):* `git-ref` will key claims by a
  namespaced string (`slice/id/<n>`, reservation-spec § Key table), which is *not*
  cleanly derivable from `dir` (`.doctrine/slice` → `slice/id` needs the facet).
  That namespace lands as a `Kind` field **when `git-ref` lands**, not now: an
  unused field today buys nothing the spec's Key table doesn't already record, and
  a set-but-never-read field would trip the deny-level dead-code gate. Recorded so
  it is added deliberately, not "discovered" missing.
- **D2 — `Kind` is a data struct with a `fn` pointer, not a trait.** One dispatch
  site, no per-kind state; a struct is the least machinery. A trait buys nothing
  until a kind needs behaviour the function signature can't carry.
- **D3 — Fileset is `Vec<Artifact>` (File | Symlink), kind-supplied.** Directly
  answers slice-002 M3 (the fixed toml+md pair was the frozen-too-high boundary).
  A spec subtype later returns its own set; the engine never hardcodes a count.
- **D4 — Placement is a closed `MaterialiseMode` enum, not a `reserve: bool`.**
  `AllocateFreshEntity` (slice, later spec) claims a fresh id; `CreateInExistingEntity`
  (design doc, later phases) lives under a parent and doesn't. The second mode is
  what makes the generalisation real rather than nominal. *An enum over a bool
  (M1):* `reserve: false` would otherwise silently accrete meanings — "existing
  parent", "no id", "clobber-allowed", "nested child" — and a third placement
  would have nowhere loud to land. The closed enum forces each new mode to be an
  explicit variant the compiler checks.
- **D5 — In *this* slice the design doc is a single `design.md`, prose-only, no
  reservation, no sister TOML.** The slice dir already namespaces it; one design
  doc per slice in v1. Revisions are git history, not `DR-001`/`DR-002` files.
  This is a deliberate scoping choice: it keeps the second caller minimal so the
  engine extraction is what gets proven here, and it holds off the
  premature-`Meta`-trait Non-Goal (a *read* facet would be the second
  metadata-bearing reader). *Known follow-up (deferred, sequence A — via
  supersede, not built now):* the design-doc TOML facet (`date`, key files /
  globs, governance relationships); approval as slice state; structured review as
  a future `RVW-` entity. Engine-neutral — a toml+md design doc is a 2-`Artifact`
  non-reserved fileset (see § 5.3).
- **D6 — Verb is `doctrine slice design <id>`.** Scaffolds `design.md` into the
  resolved slice dir. (Name over `dr`/`design-doc` for plain English; revisit if a
  `RVW-` review verb later wants a shared noun.)
- **D7 — No silent clobber.** The non-reserved path refuses if the target file
  exists; `--force` is a later affordance, not v1.
- **D-Q2 (resolved) — sub-artefact tokens are `{{ref}}` + `{{title}}`.** A design
  doc has no slug or id of its own. The template uses `{{ref}}` (the parent's
  canonical id, e.g. `SL-003`) and `{{title}}` (parent title). `ScaffoldCtx`
  carries `canonical_id` + `title`; the verb supplies them from the parent slice
  — `id` from the CLI arg, `title` read from the parent slice `Meta`.
  *Alternative rejected:* unpadded `{{id}}` (→ `SL-3`) breaks the canonical-id
  form; `{{ref}}` is the entity-model canonical id, shared across the family.

## 8. Risks & Mitigations

- **Over-abstraction (the slice-002 charge).** Mitigated structurally: extracted
  against **two callers of different shape**, not one — the boundary is corrected
  by use, and the fileset-as-function shape is *forced* by the design doc not
  fitting the two-file mould.
- **Scope creep into IP/phases or the facet.** Held off explicitly (§ 3 Out of
  scope; D5 defers the facet via supersede). The design doc is prose-only here on
  purpose — the simplest sibling.
- **`acquire` seam churn.** Small, behaviour-preserving, guarded by the slice-001
  suite. The real risk is doing *too much* (pulling the full trait or `git-ref`);
  resisted — only the one method.

## 9. Quality Engineering & Validation

- **slice-001 suite passes unchanged** — the faithful-extraction proof. The
  `reserve_create_retries_on_collision` test now exercises the `acquire` seam
  (LocalFs) without modification.
- **Engine unit tests (kind-blind), driven by a test `Kind`:** candidate-id incl.
  the `AlreadyHeld`→retry path through the seam; scan; slug; fileset write (files
  + symlink); the bounded-retry exhaustion error.
- **Path containment (H1):** a descriptor returning an absolute path or one with a
  `..` escaping the tree is rejected before any write.
- **Ghost cleanup (H2):** `reserved materialise write failure cleans up the won
  directory` — inject a failing writer; assert the won dir is gone and the error
  propagates.
- **`acquire` seam:** `LocalFs::acquire` returns `Won` then `AlreadyHeld` on a
  re-claim of the same path.
- **Design-doc `Kind`:** produces `design.md` under an *existing* slice dir, with
  **no** id reservation and **no** symlink — exercises the non-reserved,
  single-file, single-`Artifact` path; refuses to clobber an existing `design.md`.
- Lint clean (deny-level), formatted.

## 10. Review Notes

Adversarial design review + dispositions live in the audit trail
([audit.md](audit.md) appendix, "Round 1 (slice-003)" onward), the same way
slice-002 recorded its rounds. Round 1 verified D1 (the load-bearing seam claim)
survives, reconciled it with reservation-spec, and landed the `{{ref}}` token,
the deferred-facet decision (sequence A), and the minor pseudocode/scope fixes
folded into this doc. Status stays `proposed` — the `ready` gate is the user's.

## References

- Slice contract: [slice-003.md](slice-003.md); superseded predecessor: slice-002.
- Code seam + backend evolution: [reservation-spec](../../../doc/reservation-spec.md)
  § Code seam, § The unification.
- On-disk shape, pure/imperative split: [slices-spec](../../../doc/slices-spec.md).
- Why the fileset is a function (M3) and the engine is a *third/fourth* caller for
  drift/spec: [drift-spec](../../../doc/drift-spec.md) § Follow-ups,
  [spec-entity-spec](../../../doc/spec-entity-spec.md) § Follow-ups,
  [entity-model](../../../doc/entity-model.md) § storage rule.
- Canonical design-doc body (adapted, frontmatter dropped): schema bundle
  `design_revision` template.
- Starting code: `src/slice.rs`.
