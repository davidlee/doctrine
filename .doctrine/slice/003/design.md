# SL-003 — Entity-scaffold engine + design-doc sibling

Design doc for slice-003. Pure prose (the Heresiarch design-doc shape: no
frontmatter, no fenced blocks — entity-model.md). Hand-authored: the tool that
would scaffold it (`heresy slice design`) is what this slice builds, so this
first one is written by hand; later design docs are scaffolded.

## 1. Executive Summary

Extract the kind-agnostic directory-entity machinery out of `src/slice.rs` into a
small engine, shaped by **two callers of different shape** — the existing slice
(top-level, reserved, 2-file + symlink) and a new **design-doc sibling**
(sub-artefact, non-reserved, single prose file under an existing slice dir). The
load-bearing first step is lifting the inlined `mkdir` claim to a one-method
`acquire` seam (reservation-spec § Code seam). End state: slice and design-doc are
two `Kind` descriptors over one engine; drift (third caller) and spec (fourth,
per-subtype filesets) later drop in as descriptors, not forks.

This is the slice-002 charge ("don't freeze an abstraction against one caller")
answered structurally: the boundary is corrected *by a second real caller during
this slice*, not anticipated.

## 2. Problem & Constraints

**Current state.** `src/slice.rs` holds everything inline: `candidate_id`,
`derive_slug`, `build_scaffold` (a slice-specific `Scaffold` struct with hardcoded
`toml_path`/`md_path`/`symlink`), `scan_ids`, `reserve_create` (the claim loop
with `fs::create_dir` + `ErrorKind::AlreadyExists` inlined), `write_scaffold`,
`read_metas`, and the CLI verbs. drift and spec need this machinery but are
registry-gated (deferred); slice-002 proposed extracting it against the single
slice caller and froze the boundary wrong (a fixed toml+md pair — a spec needs a
per-subtype fileset of ~13).

**Constraints / guardrails.**
- **Behaviour-preserving.** The slice-001 test suite passes unchanged throughout
  — the extraction is a refactor, not a rewrite.
- **Pure/imperative discipline** (slices-spec § Architecture): candidate id, slug,
  render, fileset are pure over inputs; only the `acquire` syscall and IO sit in
  the shell, behind a seam, asserted without disk.
- **No premature trait.** Lift *only* the one `acquire` method, not the full
  `LeaseBackend`; parameterise *file mechanics*, not a shared `Meta` (slice-003
  Non-Goals).
- **Project gate.** `cargo test` + `cargo clippy` (deny-level) + `cargo fmt`.
- **Shared seams reused** (`crate::root`, `crate::install::asset_text`,
  `&mut dyn Write`), not duplicated.

**Out of scope.** IP/phase siblings (relational block + runtime state), design
*review* (`RVW-`) and any block-bearing sibling, the `git-ref` backend / full
`LeaseBackend`, drift and spec entities. This slice only makes the engine *fit to
host* them.

## 3. Architecture Intent

Three pieces: the `acquire` seam, the `Kind` descriptor (fileset-as-function +
optional reservation), and the engine loop that drives them.

### The `acquire` seam

The claim becomes a one-method trait so the `git-ref` backend drops in later
without a caller rewrite:

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
backend, lifted verbatim out of `reserve_create`. The behaviour is identical, so
the slice-001 retry test stays green.

### The `Kind` descriptor

A descriptor is *data* — what differs between entity kinds — not a trait. One
dispatch site, so a struct of fields + one function pointer is simplest:

```rust
pub struct Kind {
    /// Entity tree relative to the project root, e.g. ".doctrine/slice".
    pub dir: &'static str,
    /// Canonical-id prefix, e.g. "SL" → "SL-003" (the `{{ref}}` token).
    pub prefix: &'static str,
    /// Top-level kinds reserve a fresh id; sub-artefacts reuse a parent's id.
    pub reserve: bool,
    /// Fileset as a function — NOT a fixed toml+md pair. A slice yields toml +
    /// md + symlink; a design doc yields one prose file; a spec yields its
    /// subtype's set. This is the slice-002 fix (M3).
    pub scaffold: fn(&ScaffoldCtx) -> anyhow::Result<Fileset>,
}

pub struct ScaffoldCtx<'a> {
    pub dir: &'a Path,        // the entity dir (new numeric dir, or existing parent)
    pub id: u32,
    pub canonical_id: &'a str, // "SL-003" — the {{ref}} token (Q2)
    pub slug: &'a str,
    pub title: &'a str,
    pub date: &'a str,
}

pub enum Artifact {
    File { path: PathBuf, body: String },
    Symlink { path: PathBuf, target: String },
}
pub type Fileset = Vec<Artifact>;
```

`Fileset` generalises today's `Scaffold` struct: the slice's hardcoded
toml/md/symlink fields become three `Artifact`s; a design doc returns one
`File`. The engine writes `Artifact`s uniformly (the symlink keeps today's
`AlreadyExists`-tolerant create).

Write policy lives in the *writer/branch*, not in `Artifact` data: the
reserved branch writes freely (the dir was just won, fresh — nothing to
clobber); the non-reserved branch refuses a pre-existing target (D7). Noted
coupling: the no-clobber policy currently rides on `reserve`, not a per-write
flag — fine while reserve ⟺ fresh-dir, revisit if a kind ever wants the other
pairing. The File-writer does `create_dir_all(path.parent())` so a future
nested fileset (a spec subtype's ~13 files) needs no engine change.

### The engine loop

```text
materialise(kind, reservation, ctx_inputs):
  create_dir_all(kind.dir)                     # parent tree; non-recursive mkdir
                                               # below needs it (today's reserve_create)
  if kind.reserve:
    loop (bounded):
      id   = candidate_id(scan(kind.dir))
      dir  = kind.dir / format!("{id:03}")
      ref  = format!("{}-{id:03}", kind.prefix)  # canonical id, e.g. "SL-003"
      match reservation.acquire(dir):        # local: mkdir == dir creation
        Won        => write(kind.scaffold(ctx(dir, id, ref, ...)?)); return dir
        AlreadyHeld => continue                # lost race; recompute
  else:                                        # sub-artefact: no claim
    dir = resolve_existing(kind.dir, id)       # the parent dir; err if absent
    ref = caller-supplied parent ref           # sub-artefact has no own id/ref
    refuse if a target file already exists     # no silent clobber (file-creating
                                               # sub-artefacts only; row-append is
                                               # a separate mutate verb, not here)
    write(kind.scaffold(ctx(dir, id, ref, ...))?)
```

This is `reserve_create` generalised: the claim is now `reservation.acquire`, the
output is now `kind.scaffold`, and the non-reserved branch is new (the design-doc
path that proves the engine spans both shapes).

## 4. Code Impact Summary

| Path | Current | Target |
|---|---|---|
| `src/slice.rs` | all machinery + slice CLI inline; `Scaffold` struct; `reserve_create` inlines mkdir | the **slice `Kind`** (dir, reserve=true, scaffold fn building toml+md+symlink artifacts), the slice `Meta`/list, and thin CLI wiring; calls the engine |
| `src/entity.rs` (new) | — | the engine: `candidate_id`, `scan_ids`, `materialise` loop, `write` (artifacts), `Kind`/`ScaffoldCtx`/`Artifact`/`Fileset`, `Reservation`/`Acquired`/`LocalFs`, `derive_slug` |
| `install/templates/` | `slice.toml`, `slice.md` | + `design.md` template (prose, `{{ref}}` + `{{title}}`) |
| slice CLI | `new`, `list` | + `design <id>` |

`derive_slug`, `candidate_id`, `scan_ids`, `today`-injection and the symlink
write are kind-blind → they move to the engine. The pure `derive_slug` *helper*
moves; the slug *resolution policy* (use `--slug` else derive, bail if empty)
stays CLI-side — it is slice-Kind-specific (a design doc has no slug of its own).
`Meta`/`read_metas`/`format_list` are slice-specific *reading* (not scaffolding)
→ they stay slice-side until a second metadata-bearing caller proves a shared
shape (Non-Goal: no premature `Meta` trait). The design-doc verb *reads* the
parent slice `Meta` for its title — confirming the read stays slice-side.

## 5. Verification Alignment

- **slice-001 suite passes unchanged** — the faithful-extraction proof. The
  `reserve_create_retries_on_collision` test now exercises the `acquire` seam
  (LocalFs) without modification.
- **Engine unit tests (kind-blind), driven by a test `Kind`:** candidate-id incl.
  the `AlreadyHeld`→retry path through the seam; scan; slug; fileset write (files
  + symlink); the bounded-retry exhaustion error.
- **`acquire` seam:** `LocalFs::acquire` returns `Won` then `AlreadyHeld` on a
  re-claim of the same path.
- **Design-doc `Kind`:** produces `design.md` under an *existing* slice dir, with
  **no** id reservation and **no** symlink — exercises the non-reserved,
  single-file, single-`Artifact` path; refuses to clobber an existing `design.md`.
- Lint clean (deny-level), formatted.

## 6. Design Decisions & Trade-offs

- **D1 — `acquire` seam is path-based (`&Path`), not the abstract `key: &str`
  yet.** For local, the claim path *is* the entity dir — the `mkdir` both claims
  and creates. Faithful to today's behaviour, minimal. *Trade-off:* `git-ref`
  claims a ref, not a dir, so when it lands the seam generalises to an abstract
  key and the dir-creation splits out of `acquire` into the materialise step. That
  evolution is additive (a second impl + splitting one call); it does not rewrite
  callers, which is the whole point of having the seam. Recorded so the local
  flavour is deliberate, not accidental. **Reconciliation:** reservation-spec
  § Code seam writes the seam as `acquire(&self, key: &str)`; this slice lands it
  path-based on purpose (an abstract key now would force `LocalFs` either to know
  the entity layout — the coupling the primitive forbids — or to claim a separate
  lock tree, diverging from today's "the dir *is* the claim"). "Callers" here
  means the **Kind** callers (slice, design-doc): they invoke `materialise`, never
  `acquire`, so the later `&Path`→key generalisation + dir-creation split is
  *engine-internal* (the loop + the impls), and the Kinds are untouched — the F1
  hazard (slice-002) is avoided. The spec is reconciled to record this.
- **D2 — `Kind` is a data struct with a `fn` pointer, not a trait.** One dispatch
  site, no per-kind state; a struct is the least machinery. A trait buys nothing
  until a kind needs behaviour the function signature can't carry.
- **D3 — Fileset is `Vec<Artifact>` (File | Symlink), kind-supplied.** Directly
  answers slice-002 M3 (the fixed toml+md pair was the frozen-too-high boundary).
  A spec subtype later returns its own set; the engine never hardcodes a count.
- **D4 — Reservation is optional per kind (`reserve: bool`).** Top-level kinds
  (slice, later spec) claim an id; sub-artefacts (design doc, later requirements
  rows) live under a parent and don't. The non-reserved branch is the second shape
  that makes the generalisation real rather than nominal.
- **D5 — In *this* slice the design doc is a single `design.md`, prose-only, no
  reservation, no sister TOML.** The slice dir already namespaces it; one design
  doc per slice in v1. Revisions are git history, not `DR-001`/`DR-002` files.
  This prose-only shape is a deliberate scoping choice: it keeps the second
  caller minimal so the engine extraction is what gets proven here, and it holds
  off the premature-`Meta`-trait Non-Goal (a *read* facet would be the second
  metadata-bearing reader).
  - **Known follow-up — design-doc TOML facet (deferred, sequence A).** A design
    doc *does* carry structured data, and entity-model.md's storage rule says it
    belongs in a sister facet, not prose (the canonical `design_revision`
    frontmatter, dropped here, is exactly this). Deferred to a follow-up slice via
    supersede, **not** built now — recorded so it is not relitigated:
    - **Facet fields:** `date`, key files / globs, governance-doc relationships
      (a `[relationships]` table, the shape slice.toml already reserves).
    - **Design-doc *approval* is NOT a facet field — it is slice state.** It gates
      planning, so it lives in the slice lifecycle (the `status` transition,
      later `.doctrine/state/`), per entity-model.md *approval-separate*. Keep it
      out of the design data.
    - **Structured adversarial review (workflow / findings / disposition) is a
      future `RVW-` entity** (this slice's Non-Goal). This handover's
      hand-written review + disposition table is its precursor.
    - **Engine impact: none.** A toml+md design doc is just a 2-`Artifact`
      non-reserved fileset; `Fileset = Vec<Artifact>` already admits it. The facet
      slice adds the fields + the reader, not engine surface.
- **D6 — Verb is `heresy slice design <id>`.** Scaffolds `design.md` into the
  resolved slice dir. (Name over `dr`/`design-doc` for plain English; revisit if a
  `RVW-` review verb later wants a shared noun.)
- **D7 — No silent clobber.** The non-reserved path refuses if the target file
  exists; `--force` is a later affordance, not v1.

## 7. Open Questions

- **Q1 — `Kind` registry.** Two kinds can be two `const Kind`s referenced
  directly. A lookup table (`&str -> &Kind`) is only needed when a generic
  `heresy <kind> new` dispatch exists; not this slice. Leave direct.
- **Q2 — Title/id for a sub-artefact (resolved).** A design doc has no slug or id
  of its own. The template uses two tokens: `{{ref}}` (the parent's canonical id,
  e.g. `SL-003`) and `{{title}}` (the parent title). `ScaffoldCtx` carries
  `canonical_id` + `title`; the verb supplies them from the parent slice — `id`
  from the CLI arg, `title` read from the parent slice `Meta`. (Confirms the Meta
  read stays slice-side, § 4.)
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

## 8. Rollout (build sequence — each step green)

1. **`acquire` seam, in place.** Add `Reservation`/`Acquired`/`LocalFs`; rewrite
   `reserve_create`'s claim against it. Pure refactor; slice-001 suite green. Cheap
   and valuable even if the slice stopped here (reservation-spec § Code seam).
2. **Engine module `src/entity.rs`.** Move the kind-blind pure fns + the
   `materialise` loop + `Artifact` write behind `Kind`/`ScaffoldCtx`. The slice
   `Kind` reproduces today's exact two-file + symlink output. Suite still green.
3. **Design-doc `Kind` + `design.md` template.** The non-reserved, single-file
   kind. Engine unit test for the non-reserved path.
4. **Wire `heresy slice design <id>`.** Thin CLI verb over the engine.

## 9. References

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
