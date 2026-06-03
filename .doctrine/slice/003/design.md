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
    /// Top-level kinds reserve a fresh id; sub-artefacts reuse a parent's id.
    pub reserve: bool,
    /// Fileset as a function — NOT a fixed toml+md pair. A slice yields toml +
    /// md + symlink; a design doc yields one prose file; a spec yields its
    /// subtype's set. This is the slice-002 fix (M3).
    pub scaffold: fn(&ScaffoldCtx) -> anyhow::Result<Fileset>,
}

pub struct ScaffoldCtx<'a> {
    pub dir: &'a Path,    // the entity dir (new numeric dir, or existing parent)
    pub id: u32,
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

### The engine loop

```text
materialise(kind, reservation, ctx_inputs):
  if kind.reserve:
    loop (bounded):
      id   = candidate_id(scan(kind.dir))
      dir  = kind.dir / format!("{id:03}")
      match reservation.acquire(dir):        # local: mkdir == dir creation
        Won        => write(kind.scaffold(ctx(dir, id, ...)?)); return dir
        AlreadyHeld => continue                # lost race; recompute
  else:                                        # sub-artefact: no claim
    dir = resolve_existing(kind.dir, id)       # the parent slice dir
    refuse if the artefact already exists      # no silent clobber
    write(kind.scaffold(ctx(dir, id, ...))?)
```

This is `reserve_create` generalised: the claim is now `reservation.acquire`, the
output is now `kind.scaffold`, and the non-reserved branch is new (the design-doc
path that proves the engine spans both shapes).

## 4. Code Impact Summary

| Path | Current | Target |
|---|---|---|
| `src/slice.rs` | all machinery + slice CLI inline; `Scaffold` struct; `reserve_create` inlines mkdir | the **slice `Kind`** (dir, reserve=true, scaffold fn building toml+md+symlink artifacts), the slice `Meta`/list, and thin CLI wiring; calls the engine |
| `src/entity.rs` (new) | — | the engine: `candidate_id`, `scan_ids`, `materialise` loop, `write` (artifacts), `Kind`/`ScaffoldCtx`/`Artifact`/`Fileset`, `Reservation`/`Acquired`/`LocalFs`, `derive_slug` |
| `install/templates/` | `slice.toml`, `slice.md` | + `design.md` template (prose, `{{title}}`) |
| slice CLI | `new`, `list` | + `design <id>` |

`derive_slug`, `candidate_id`, `scan_ids`, `today`-injection and the symlink
write are kind-blind → they move to the engine. `Meta`/`read_metas`/`format_list`
are slice-specific *reading* (not scaffolding) → they stay slice-side until a
second metadata-bearing caller proves a shared shape (Non-Goal: no premature
`Meta` trait).

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
  flavour is deliberate, not accidental.
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
- **D5 — Design doc is a single `design.md`, prose-only, no reservation, no
  sister TOML.** The slice dir already namespaces it; one design doc per slice in
  v1. Revisions are git history, not `DR-001`/`DR-002` files. If it ever grows
  facets it becomes a metadata-bearing kind — but that is exactly the
  premature-`Meta`-trait Non-Goal, held off.
- **D6 — Verb is `heresy slice design <id>`.** Scaffolds `design.md` into the
  resolved slice dir. (Name over `dr`/`design-doc` for plain English; revisit if a
  `RVW-` review verb later wants a shared noun.)
- **D7 — No silent clobber.** The non-reserved path refuses if the target file
  exists; `--force` is a later affordance, not v1.

## 7. Open Questions

- **Q1 — `Kind` registry.** Two kinds can be two `const Kind`s referenced
  directly. A lookup table (`&str -> &Kind`) is only needed when a generic
  `heresy <kind> new` dispatch exists; not this slice. Leave direct.
- **Q2 — Title/slug for a sub-artefact.** A design doc has no slug of its own;
  `ScaffoldCtx` carries the *parent* slice's slug/title. Confirm the design
  template needs only `{{title}}` (parent title) — likely yes.
- **Q3 — Does `list` ever go kind-generic?** `slice list` reads slice `Meta`.
  drift/spec will want their own `list`. Deferred until the second reader; the
  engine owns scaffolding only for now (no premature `Meta`).

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
