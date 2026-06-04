# Design SL-005: Memory entity v1

## 1. Design Problem

Land the first memory build step ([slice-005.md](slice-005.md)): a native
current-state memory entity with `record` / `show` / `list` and the install-manifest
split. The non-trivial part is not the CRUD — it is that memory is the **entity
engine's first string-identity, reservation-free caller**, and the engine
(`src/entity.rs`) is numeric-identity + reservation-centric throughout. The design
must generalise the engine to materialise a **caller-named** entity with **no id
allocation**, driven by memory as the proving caller, **without churning the
behaviour** of the four numeric callers (slice, design, plan, phases) that ride it.

Secondary: the schema/parse shape (the three-layer model minus the registry), the
uid-minting seam (impurity kept in the shell), and where this slice stops
(`anchor_kind = none`, no retrieval, no ledger).

> This revision incorporates the adversarial external review (§ 10). The three
> structural decisions taken there — rename the claim seam (D7), replace the
> mode/inputs Option-bag with a runtime `MaterialiseRequest` enum (D8), lock the
> `Materialised` return to an owned identity enum (D9) — widen the engine churn
> beyond the original minimal sketch, but each removes an invalid-state surface
> that Rust can then enforce. Numeric callers stay **behaviour**-preserving; their
> signatures change mechanically.

## 2. Current State

`src/entity.rs` is a kind-agnostic scaffolding engine, proven across slice / design
/ plan / phases (slice-003/004). Its identity model is **uniformly numeric**:

- `Kind { dir, prefix, mode, scaffold }`; `prefix` renders `SL` → `SL-003`. `mode`
  is a `const` `MaterialiseMode` discriminant carried *on the Kind* (not per call).
- `MaterialiseMode::{AllocateFreshEntity, CreateInExistingEntity}` — both numeric.
- `Inputs { existing_id: Option<u32>, slug, title, date }` — an **Option-bag**: the
  payload for `CreateInExistingEntity` rides an `Option` read at runtime
  (`.context("requires a parent id")`), even though placement is fixed per Kind.
- `candidate_id(existing: &[u32]) -> u32` (`max + 1`), `scan_ids` (numeric dirs).
- `allocate_fresh`: scan → candidate → `format!("{id:03}")` → claim in a
  **race-retry loop** (an `AlreadyHeld` means *lost the race* → recompute & retry).
- The claim seam: `trait Reservation { fn acquire(&self, claim: &Path) -> Acquired }`,
  backend `LocalFs` (the `mkdir` *is* the claim). `Acquired::{Won, AlreadyHeld}`.
  The doc comment already frames `acquire` as the generic atomic claim a future
  `git-ref` backend slots into — but the *trait* is named `Reservation`.
- `ScaffoldCtx { id: u32, canonical_id: &str, slug, title, date }` — the render
  context; scaffolds token-substitute `{{ref}}` = `canonical_id`.
- `Materialised { id: u32, dir }` — `id` is non-optional.

Scaffolds render embedded templates (`install/templates/*`, `asset_text` +
`{{token}}` replace) into `Fileset` = `Vec<Artifact::{File,Symlink}>`; the engine's
`write_fileset` is transactional (tracks created paths/dirs, unwinds on failure)
and is the sole path→fs joiner (H1). The writer uses `std::os::unix::fs::symlink` —
the engine is already **unix-only**. `uuid` (v4/v5) and `time` are in the workspace,
**commented out** in doctrine's `Cargo.toml`; v7 is **not** in the enabled feature
set.

## 3. Forces & Constraints

- **Interop constraint 3** (memory-spec § Identity): `memory_uid` is a client-minted
  UUID **minted once per logical memory and stored, never regenerated** (:268-270) —
  *not* content-derived; the deterministic-`uuid5` "content-addressed" property is
  `event_id`'s, at the deferred ledger seam (§ 5.6, D3). A stored v7 uid satisfies it.
- **No reservation needed.** A UUID is collision-*resistant* enough across clones
  that v1 treats any collision as an exceptional duplicate, not a race to arbitrate
  (reservation-spec exists to arbitrate *numeric* `max+1` races; a uid has no race).
  So the `candidate_id`/retry machinery is not just unused — it is *wrong* for
  memory (an `AlreadyHeld` on a uid is a duplicate, not a lost race).
- **memory-spec § Identity (:128): "No reservation namespace, no `acquire` call."**
  The engine seam is nonetheless reused — so the *names* must stop implying memory
  takes a reservation. Resolved by D7 (rename the seam to a neutral claim).
- **No parallel implementation** (CLAUDE.md; reservation-spec § Code seam): memory
  must ride the engine, not fork a second materialiser.
- **Behaviour-preserving for numeric callers**: their suites are the gate.
  Signatures change (D8/D9); observable behaviour does not.
- **Pure/imperative split** (slices-spec § Architecture): no clock, rng, git, or
  disk in the pure layer — the uid and date are *inputs*, like the dir listing.
- **Generalise only as far as forced** (reservation-spec § "apparatus arrives with
  the caller"): the second identity shape justifies exactly the generalisation that
  removes an invalid state — no speculative identity-strategy framework.
- **Provisional vocab** (memory-spec § Locked decisions): type/status membership may
  change pre-harden; the model must not depend on exact members.
- **Stored memory is hostile input** (memory-spec § Security, :87/:362): rendered as
  data, never instruction. `show` honours this even at the CLI (§ 5.2).

## 4. Guiding Principles

- The engine's placement is *"a closed enum, not a bool, so a third placement is a
  compiler-forced new variant"* (entity.rs) — the design **keeps that invitation and
  sharpens it**: the placement enum becomes a *runtime* `MaterialiseRequest` that
  carries its own payload, so an invalid placement/payload pair cannot be expressed
  (D8), not merely a third const variant beside an Option-bag.
- Reuse the claim seam's mechanism unchanged — `mkdir` is still the atomic claim;
  only the *interpretation* of an existing claim differs (duplicate vs retry) and
  the seam's *name* loses its reservation connotation (D7).
- Read path is plain serde (the `Meta` pattern); the edit-preserving `toml_edit`
  path is owed only by a mutation verb, which v1 does not ship.
- Stop at the entity. Retrieval, git, links, ledger are later callers.

## 5. Proposed Design

### 5.1 System Model — string identity in the engine

Three coupled changes: an identity widening for rendering (`EntityId`), a runtime
placement enum that carries its payload (`MaterialiseRequest`), and an owned
identity in the return (`Materialised`/`OwnedEntityId`).

**`EntityId` enum** replaces the bare `id: u32` + `canonical_id` in the render
context, expressing both identity shapes the engine now serves:

```rust
pub(crate) enum EntityId<'a> {
    Numbered { id: u32, canonical: &'a str },  // SL-003 — numeric
    Named { name: &'a str },                    // <memory_uid> — caller-supplied
}
```

`ScaffoldCtx` carries `eid: EntityId<'a>` in place of `id`/`canonical_id`; `slug`,
`title`, `date` unchanged. Numeric scaffolds destructure `Numbered { id, canonical }`
(mechanical, behaviour-preserving); the memory scaffold reads `Named { name }`.

**`MaterialiseRequest` — runtime placement carrying its payload (D8).** The const
`Kind.mode` field is **removed**; placement and its identity payload move into a
runtime enum passed to `materialise`:

```rust
pub(crate) enum MaterialiseRequest<'a> {
    Fresh,                       // allocate next numeric id (slice, spec)
    InExisting { id: u32 },      // create under an existing parent (design, phases)
    Named { name: &'a str },     // caller-supplied identity (memory uid)
}
```

`Inputs` keeps only the *common* render fields (`slug`, `title`, `date`) — the
`existing_id: Option<u32>` bag is gone, its payload now in `InExisting { id }`. This
is the change that closes review #3: there is no `name`-without-named, no
`id`-without-existing, no placement/payload mismatch — each variant *is* its
payload. (Placement was a const Kind property only incidentally; it is genuinely a
per-*call* choice made at one kind-specific call site each, so moving it to the call
loses nothing and removes the second discriminant that could disagree with a
payload.)

`materialise` dispatches on the request, not on a Kind field:

```rust
fn materialise(kind, claim, project_root, req: &MaterialiseRequest, inputs)
    -> anyhow::Result<Materialised>
{
    let tree_root = project_root.join(kind.dir);   // H1 base, unchanged
    fs::create_dir_all(&tree_root)?;
    match req {
        Fresh              => allocate_fresh(kind, claim, &tree_root, inputs, …),
        InExisting { id }  => create_in_existing(kind, &tree_root, *id, inputs),
        Named { name }     => allocate_named(kind, claim, &tree_root, name, inputs),
    }
}
```

**`allocate_named`** — the new sibling. Path correctness (review #2): memory's
`Kind.dir = ".doctrine/memory/items"`, so `tree_root` already *is* the `items/`
root and `tree_root.join(name)` lands `…/items/<uid>/` — no missing segment, no new
parameter. The invariant: **`allocate_named` writes at `tree_root.join(name)`, and a
named Kind sets `dir` to the directory that must directly parent the named entities.**

```rust
fn allocate_named(kind, claim, tree_root, name, inputs) -> Materialised {
    let dir = tree_root.join(name);          // no scan, no candidate, no {:03}
    match claim.claim(&dir)? {               // same mkdir seam (renamed, D7)
        Won => { scaffold_and_write(kind, tree_root, &ctx_named(name, inputs))?;
                 Ok(Materialised { eid: Named { name: name.into() }, dir }) }
        AlreadyHeld => bail!("memory entity {name} already exists"),  // duplicate, not a race
    }
}
```

No retry loop. Reuses `scaffold_and_write` + `write_fileset` (incl. the H2 "Won ⟹
ours ⟹ clean up a partial scaffold" guarantee) verbatim.

**`Materialised` → owned identity (D9).** `id: u32` cannot represent a named
entity, so the return is locked to an owned enum (not the rejected
`name: Option<String>`, which reproduces the very invalid-state surface D8 removes):

```rust
pub(crate) enum OwnedEntityId {
    Numbered { id: u32, canonical: String },
    Named { name: String },
}
pub(crate) struct Materialised { pub eid: OwnedEntityId, pub dir: PathBuf }

impl OwnedEntityId {
    pub fn numeric_id(&self) -> Option<u32>;   // numeric callers read this
    pub fn canonical_ref(&self) -> Option<&str>;
}
```

Numeric callers migrate from `.id` to `.eid.numeric_id().expect(..)` /
`.canonical_ref()` — mechanical, gated by their suites. `run_record` reads
`OwnedEntityId::Named { name }` to print the uid.

This is the whole generalisation: one render enum, one runtime request enum (mode
field removed), one owned-return enum, one `allocate_named` sibling, one seam
rename. The claim mechanism and the transactional writer are untouched in behaviour.

### 5.2 Interfaces & Contracts (CLI)

```
doctrine memory record <title> --type <t> [--key <k>] [--status <s>]
                                [--summary <text>] [--tag <t>]...   [--path <root>]
doctrine memory show <uid|key>                                      [--path <root>]
doctrine memory list [--type <t>] [--status <s>] [--tag <t>]        [--path <root>]
```

- `record`: `--type` required (provisional 6-value enum); `--status` default
  `active`; `--key` optional `mem.<type>.<domain>.<subject>` (shorthand normalized,
  segment-validated). Mints uid (v7, § 5.6), scaffolds `items/<uid>/`, and — iff
  `--key` — creates a `<key> -> <uid>` symlink **as part of the fileset** so its
  creation is transactional (§ 5.5). Prints the uid (and key) + path. `--tag`
  values write `scope.tags` (not a top-level field — review #9).
- `show <uid|key>` (**symlink-only resolution — review #6**): the argument is first
  parsed into a validated `MemoryRef::{Uid, Key}` — `Uid` matches `^mem_[0-9a-f]{32}$`,
  `Key` matches the `mem.<seg>…` grammar (per-segment `[a-z0-9]+(-[a-z0-9]+)*`,
  2–7 segments, memory-spec § Identity) — **rejecting any separator / absolute /
  `..` before touching disk**. The path is then built through
  `fsutil::safe_join(items_root, name)` (**codex-MAJOR-3**: the read path must reuse
  the H1 chokepoint — `safe_join` is currently write-only, `entity.rs:289,328`; a raw
  `items/<arg>/…` join is a traversal hole for the user-supplied key). A uid hits the
  real dir, a key hits the slug symlink (fs resolves it). **No `memory_key` scan
  fallback** in v1 — a scan would make stale hand-edited keys semi-authoritative and
  add O(n) to a direct-lookup command; the registry/index that could re-key safely
  arrives in SL-009. (slice-005.md is updated to drop its "/ a `memory_key` scan"
  clause.) **Security render (review #14 + codex-MAJOR-4):** `show` prints the full
  hostile-input metadata header the spec mandates (memory-spec § Security :365-367) —
  **`memory_uid` / `memory_key`, `trust_level`, `verification_state`, `scope`, and
  `anchor`** — then the body **labelled as memory content, never emitted as
  instruction**. (The original header dropped `scope` and `anchor`; the spec lists
  them explicitly, so both are restored. In v1 `anchor` renders as `none`.)
- `list`: scans real dirs under `items/` (symlinks skipped by `file_type().is_dir()`,
  as `scan_ids` already does), parses each `memory.toml`, AND-filters on
  type/status/tag, formats rows (uid-short / type / status / key / title).
  **Order (review #13, contract not note): `created` descending, then `uid`
  ascending** for a deterministic, human-useful default.

`--path` mirrors every other subcommand (explicit project root, else walk-up).

`memory.md` body (review #7): v1 scaffolds a **template containing title + summary
only** — no editor, no stdin, no `--body`. Richer body capture is a later mutation
verb. `show` therefore renders bounded, tool-authored prose.

### 5.3 Data, State & Ownership — schema & parse

Two-layer parse (registry layer deferred to SL-009):

```rust
struct RawMemoryToml {            // tolerant; serde(default) on nested blocks
    memory_uid: String, memory_key: Option<String>, schema_version: u32,
    memory_type: String, status: String, title: String, summary: String,
    created: String, updated: String,
    #[serde(default)] scope: RawScope, #[serde(default)] git: RawGit,
    #[serde(default)] review: RawReview, #[serde(default)] trust: RawTrust,
    #[serde(default)] ranking: RawRanking,
    #[serde(default, rename = "relation")] relations: Vec<RawRelation>,
    #[serde(default, rename = "source")]   sources:   Vec<RawSource>,
    #[serde(flatten)] extra: BTreeMap<String, toml::Value>,
}
struct Memory { uid, key: Option<_>, memory_type: MemoryType, status: Status, .. }
enum MemoryType { Concept, Fact, Pattern, Signpost, System, Thread }   // provisional
enum Status { Active, Draft, Superseded, Retracted, Archived, Quarantined }
```

**Nested-default mechanics (review #8 — make it precise, don't hand-wave):**

- every nested raw struct (`RawScope`/`RawGit`/`RawReview`/`RawTrust`/`RawRanking`)
  derives `Default`; `RawMemoryToml` marks each with `#[serde(default)]`, so a
  deleted `[block]` fills defaults rather than failing to parse.
- fields that are *schema-required after validation* (e.g. a non-empty `memory_uid`,
  a known `memory_type`) are checked in `TryFrom<RawMemoryToml> for Memory`, not by
  serde presence.
- **`extra` lives only at the top level** (the single `#[serde(flatten)]`). The
  "unknown keys preserved" promise (slices-spec) is therefore scoped to top-level
  keys; unknown keys *inside* a nested block are **not** preserved in v1 (no nested
  `extra`). Stated so the promise is not overbroad.

Closed enums (`#[serde(rename_all="kebab-case")]`) — parse error on an unknown
member, the explicit-vocab posture (not drift-spec's soft `Other` arm; memory's
vocab is doctrine-owned, not externally produced). Read is plain serde; no
`toml_edit` (no mutation in v1).

**`schema_version` is validated, not ignored (review #10 — reverses original Q5).**
For an owned, about-to-be-durable schema, emit-and-ignore risks a future
incompatible file silently parsing as v1. v1 **emits `1`, accepts only `1`, and
errors on missing or unsupported version.**

The `memory.toml` template substitutes values on hand at scaffold
(`{{uid}}`/`{{key}}`/`{{type}}`/`{{status}}`/`{{title}}`/`{{summary}}`/`{{date}}`/
`{{schema_version}}`/`{{workspace}}`); `[scope]`/`[git]`/`[review]`/`[trust]`/
`[ranking]` scaffold with defaults (`anchor_kind = none`, empty
`paths`/`globs`/`commands`/`tags`, `verification_state = unverified`,
`trust_level = medium`, `severity = none`, `weight = 0`).

**`scope.workspace` is carried unconditionally (codex-BLOCKING-2 — interop
constraint 6, memory-spec :84-86/:154/:294).** `workspace` is *not* part of the
deferred git/anchoring work (that is `repo` + the frame, SL-008); it is a coordinate
**carried on every memory from the first record**, even single-tenant. v1 scaffolds
`scope.workspace = "default"` always, the model carries it (non-empty after
validation), and `list`/`show` read it (it is a hard-filter key in the SL-007
deterministic sort, :314 — so it must exist now, not be back-filled later). The
original scaffold-defaults list silently omitted it; restored.

**Tag validation (review #9):** `--tag` values are free lowercase strings
(memory-spec calls tags "stable categorization", not scope segments) — trimmed,
non-empty, deduplicated; no `mem.*` segment grammar imposed.

Ownership: `items/<uid>/` is git-tracked authored state; `index/`/`embeddings/`/
`state/` are gitignored (manifest split, § 5.4). No ledger written.

### 5.4 Lifecycle, Operations & Dynamics

`record` is the only writer. Status defaults `active`; transitions are hand-edits in
v1 (no mutation verb — same staging as slice/drift). uid minting lives in the shell
(`uuid::Uuid::now_v7()` → `mem_<32 hex>`, § 5.6), passed into the pure scaffold as an
input beside the date — no rng/clock in pure code. The `--key` symlink is emitted as
an `Artifact::Symlink` **in the fileset**, so `write_fileset`'s transaction covers it
(§ 5.5). Manifest change (`install/manifest.toml`):

```toml
[dirs]   create  = [ …, ".doctrine/memory/items" ]
[gitignore] entries = [ ".doctrine/memory/index/*", ".doctrine/memory/embeddings/*",
                        ".doctrine/memory/state/*" ]   # replaces ".doctrine/memory/*"
```

### 5.5 Invariants, Assumptions & Edge Cases

- **uid uniqueness** holds with overwhelming probability by construction; an
  `AlreadyHeld` claim is nonetheless a hard error (defence in depth — a collision or
  a re-run with a fixed uid surfaces, it never silently merges).
- **`record` is not idempotent** in v1: each call mints a fresh uid → a re-run makes
  a second memory. This is a CLI-UX matter, **not** a § Identity violation — the spec
  requires the uid be minted *once per logical memory and stored* (it is), and reserves
  *append*-idempotency for the `event_id`/`uuid5` ledger seam (§ 5.6, D3). Two
  `record` calls are two logical memories. Accepted; dedup-on-replay lands with the
  ledger.
- **`active` ≠ retrieval-eligible (review #12).** Invariant to document:
  `status = active` is *lifecycle*-active; with `anchor_kind = none`, empty scope,
  and `verification_state = unverified`, the record is **not** retrieval-eligible —
  retrieval suppression and scope-matching (SL-007/security) gate that separately.
  `active + unverified + unscoped` is consistent, not contradictory.
- **key symlink — transactional, pre-existing is a hard error (review #5).** Created
  only with `--key`, **inside the fileset**: `symlink(2)` errors on any existing path
  (stale alias, alias to a missing uid, real file/dir squatting the name, or a
  normalized key colliding with a uid-shaped name) → `write_fileset` rolls back the
  whole record (the uid dir included). No silent overwrite, no partial record. A
  later re-key is a hand-edit (deliberately deferred). Hand-editing `memory_key`
  desyncs the symlink — the slices stale-symlink risk, accepted.
- **list skips symlinks** (real-dir filter), so key aliases never double-count.
- **non-git project**: irrelevant this slice — no frame is built, `anchor_kind = none`.
- **unix symlinks assumed (review #15).** The engine already calls
  `std::os::unix::fs::symlink`; `show <key>` inherits that. Non-unix support, if ever
  needed, degrades key aliasing to the SL-009 registry rather than a symlink.

### 5.6 Identity format (resolves Q1 / review #11 / ed2)

- **Generator: UUID v7** (time-ordered). Matches the spec's example bytes
  (`mem_018f3a…`, `mem_018e…` — `018…` is a v7 timestamp prefix) and gives a useful
  monotonic default order. Requires enabling the `uuid` crate's **`v7`** feature in
  doctrine's `Cargo.toml` (workspace dep; currently v4/v5 only).
- **Grammar:** `mem_` + 32 lowercase hex (UUID *simple* form, no hyphens):
  `^mem_[0-9a-f]{32}$`. Minted lowercase; reject uppercase/hyphenated rather than
  normalize.
- **v7 satisfies § Identity — the "content-addressed" adjective is `event_id`'s, not
  the uid's (corrects ed2 / codex-BLOCKING-1).** A closer read of the *operative*
  per-field spec resolves the apparent tension: memory-spec § Identity (:268-270)
  defines `memory_uid` as a **"client-minted UUID, minted once per logical memory …
  stored, never regenerated"** — explicitly *not* content-derived. It is `event_id`
  that is the **deterministic `uuid5` over a fixed namespace** (:274), which is what
  makes *append* idempotent (interop constraint 3, :73-75). The umbrella
  "content-addressed" adjective is realised by `event_id` at the ledger seam, not by
  the uid. A v7 uid minted once and stored is therefore **fully compliant** with
  § Identity. `event_id`/uuid5 and append-idempotency belong to the deferred ledger
  seam (slice-005 non-goals); v1 ships no events, so the property has nothing to
  violate. (The earlier draft wrongly framed the uid as *owing* content-addressing —
  it does not.)

## 6. Open Questions & Unknowns — all resolved

1. **uid format** — **RESOLVED: `mem_` + UUID v7 simple form** (§ 5.6). Enable the
   `v7` feature; default list order is `created desc, uid asc`.
2. **`record` default status** — **RESOLVED: `active`** (spec-driver parity), with
   the "active ≠ retrieval-eligible" invariant documented (§ 5.5).
3. **`--key` ergonomics** — **RESOLVED: optional** (no auto-derivation;
   `domain`/`subject` aren't recoverable from a title).
4. **`Materialised` return shape** — **RESOLVED: owned `OwnedEntityId` enum +
   accessors** (D9, § 5.1) — not `name: Option<String>`.
5. **`schema_version`** — **RESOLVED: emit `1`, validate `== 1`** (review #10),
   reversing the original emit-and-ignore lean.

## 7. Decisions, Rationale & Alternatives

- **D1 — string identity via `EntityId` render enum.** The doctrine-consistent
  identity widening. *Rejected:* a memory-local materialiser bypassing the engine (a
  parallel implementation, CLAUDE.md); a full identity-strategy trait object
  (over-generalises past the second shape — YAGNI).
- **D2 — no reservation; reuse the claim mechanism as a create-with-duplicate-guard.**
  The `mkdir` seam is untouched; only the interpretation of an existing claim differs
  (duplicate vs retry). Memory takes **no** reservation namespace.
- **D3 — uid minted in the imperative shell, v7**, an input to the pure layer (date
  precedent). Keeps pure code clock/rng-free. v7 over v4 for time-ordering and
  spec-example parity (`018f…`). Per § Identity the uid is the minted-once,
  stored idempotency anchor — a stored v7 uid is compliant; the deterministic
  `event_id`/`uuid5` that makes *append* idempotent is the deferred ledger seam's
  concern, not the uid's (corrects the earlier content-addressing mis-framing —
  codex-BLOCKING-1).
- **D4 — `anchor_kind = none`, no git this slice.** Defers all `git_context` work to
  SL-008; unanchored unscoped memory is permitted (memory-spec § Scope & anchoring).
- **D5 — read path plain serde, no `toml_edit`.** Mutation (and its edit-preserving
  writer) is owed by the ledger seam, not v1.
- **D6 — `--key` optional; resolution by dir/symlink, no scan.** Cheapest correct
  `show`; `list` reads the real dirs.
- **D7 — rename the claim seam (review #1).** memory-spec:128 says "no `acquire`
  call"; the engine's seam is nonetheless reused. Rename `trait Reservation` →
  **`Claim`**, `fn acquire` → **`claim`**, keeping `Acquired::{Won, AlreadyHeld}`.
  Reservation becomes *one caller's interpretation* of the generic atomic claim;
  memory's interpretation is create-with-duplicate-guard. *Alternative rejected:*
  keep the name and reconcile only in prose — leaves the spec/name collision live.
  Blast radius: `entity.rs` + the reservation-spec wording + numeric call sites
  (mechanical rename).
- **D8 — `MaterialiseRequest` runtime enum; drop `Kind.mode` and `Inputs.existing_id`
  (review #3).** The const `mode` + Option-bag admits invalid pairs
  (named-without-name, numeric-with-name, both-set). Folding placement *and its
  payload* into one runtime enum makes each invalid pair unrepresentable. *Alternative
  rejected:* a third const `MaterialiseMode` variant beside `name: Option<&str>` in
  `Inputs` — the original sketch; cheaper churn but keeps the cross-product. The
  const-vs-runtime tension (mode was a Kind property) is resolved by recognising
  placement is genuinely a per-call choice made at one kind-specific site each.
- **D9 — `Materialised` carries an owned `OwnedEntityId` enum (review #4).** `id: u32`
  can't represent a named entity. *Alternative rejected:* `name: Option<String>`
  beside `id` — reproduces the invalid-state surface D8 removes. Accessor helpers
  (`numeric_id`, `canonical_ref`) bound numeric-caller churn.
- **D10 — `schema_version` validated `== 1` (review #10).** Cheap now; prevents a
  future incompatible file silently reading as v1.

## 8. Risks & Mitigations

- **Engine churn is wider than the original sketch (D7/D8/D9).** The seam rename, the
  `Kind.mode` removal, and the `Materialised`/`Inputs` reshape touch all four numeric
  callers' *signatures*. Mitigation: the changes are mechanical (rename, destructure,
  `.eid.numeric_id()`), behaviour-preserving, and the full slice/design/plan/phase
  suites gate every step. No observable behaviour changes.
- **`EntityId` widening churns the four numeric scaffolds.** Mechanical destructure
  (`Numbered { id, canonical }`); suites gate.
- **`Kind.mode` removal could let a call site pass the wrong placement.** Each kind
  has exactly one materialise call site; the request is built there. Lower risk than
  the removed cross-product, and a misplacement fails loudly (wrong tree / duplicate).
- **Provisional vocab churn.** Members may change pre-harden; cost is a code edit with
  no corpus to migrate yet — accepted, the reason v1 hardens nothing.
- **Double-record (non-idempotent).** A re-run makes a duplicate; mitigation is
  delete, and the content-addressed idempotency anchor lands with the ledger seam.
- **uuid feature creep.** Enabling `uuid`'s `v7` feature in doctrine's `Cargo.toml`
  (already a workspace dep) — v7 only.
- **`extra`-scope promise.** Narrowed to top-level keys (§ 5.3); a future need for
  nested-block preservation is a per-block `extra` addition, flagged not built.

## 9. Quality Engineering & Validation

Pure-layer unit tests (the doctrine pattern — inputs in, no disk/clock):

- **Engine, kind-blind**: a named test `Kind` drives a `Named` request — `Won` writes
  the named dir + fileset; a pre-existing name → `AlreadyHeld` → duplicate error (no
  retry, distinct from `Fresh`'s retry test); a scaffold failure after `Won` cleans
  up the named dir (H2, mirrored).
- **`MaterialiseRequest` dispatch**: each variant routes to the right path; the
  numeric `Fresh`/`InExisting` tests survive the enum migration unchanged in
  behaviour (their `Inputs`/return reads are mechanically updated).
- **Identity regression**: existing `allocate_fresh` / `create_in_existing` tests stay
  green through the `EntityId`/`OwnedEntityId` widening; numeric scaffolds render
  `{{ref}}` unchanged; `OwnedEntityId::numeric_id`/`canonical_ref` accessors covered.
- **Schema**: `RawMemoryToml` ↔ `memory.toml` round-trip; top-level unknown keys
  preserved in `extra`; a **deleted nested block fills defaults**; a nested unknown
  key is **not** preserved (scope assertion); an unknown `memory_type`/`status`
  member is a parse error; **`schema_version != 1` is a hard error** (review #10).
- **Render**: `memory.toml`/`memory.md` template token substitution from a ctx.
- **Key**: shorthand normalization (`pattern.cli.skinny` → `mem.pattern.cli.skinny`),
  segment validation, symlink target; **tag** normalization/validation.
- **Duplicate key + rollback (review #5)**: a pre-existing key alias (incl. stale /
  to-missing-uid) → hard error and the **uid dir is rolled back** (alias-in-fileset
  transactionality).
- **List**: row formatting; type/status/tag AND-filter; **`created desc, uid asc`
  ordering** (review #13); symlink aliases excluded.
- **show resolution (review #6)**: uid hits the dir, key hits the symlink; **no scan
  fallback** (a stale `memory_key` with no symlink does not resolve).
- **show arg validation (codex-MAJOR-3)**: a `<uid|key>` carrying `..` / a separator /
  an absolute path is **rejected at parse, before any fs access**; the resolved read
  path goes through `safe_join` (a malicious arg cannot escape `items/`).
- **show security (review #14 + codex-MAJOR-4)**: the rendered header carries the full
  spec set — uid/key, trust_level, verification_state, **scope, and anchor** — and the
  body is labelled data, never bare instruction-shaped output.
- **workspace carried (codex-BLOCKING-2)**: a freshly recorded `memory.toml` has
  `scope.workspace = "default"`; the model rejects an empty workspace; `list`/`show`
  surface it.

**Integration (review ed3) — one tempdir test** for `record`→`show`→`list` end to
end, exercising the real symlink create/resolve and path cleanup that the
without-disk seam asserts cannot cover. Pure tests stay dominant.

**Install (review ed4)**: the install-plan test asserts the blanket
`.doctrine/memory/*` ignore is **replaced** by the three narrower entries (not left
duplicated alongside them) and `items/` is created.

`cargo test` + `cargo clippy` (zero warnings) gate.

## 10. Review Notes

External adversarial review dispositioned (slice-002/003/004 rhythm). Verdict
accepted: *architecture sound, implementation contract was too loose.* Summary of
calls — 13 accepts, 1 reversal, 1 architectural rebuttal, 3 escalated decisions:

- **Accepted & folded in**: #2 path (`Kind.dir = items`, no new param), #4 owned
  `Materialised` (D9), #5 transactional key alias + hard duplicate, #6 symlink-only
  `show` (and slice-005.md updated), #7 thin `memory.md` body, #8 nested-default
  mechanics + narrowed `extra` scope, #9 `scope.tags` + tag validation, #12
  active≠retrieval-eligible invariant, #13 list-order contract, #14 `show` security
  render, #15 unix-symlink assumption, ed1 softened collision wording, ed3 tempdir
  integration test, ed4 manifest-migration test.
- **Reversal**: #10 `schema_version` now validated `== 1` (D10), not emit-and-ignore.
- **Rebuttal (review #3)**: the suggested "carry `name` in the `MaterialiseMode`
  variant" is **infeasible** — `mode` is a `const Kind` field, the uid is runtime. The
  concern (invalid states) is valid and addressed instead by a *runtime*
  `MaterialiseRequest` enum that drops both `Kind.mode` and `Inputs.existing_id` (D8).
- **Escalated → user decisions**: #1 seam name → **rename** `Reservation`→`Claim`
  (D7); #3 shape → **`MaterialiseRequest` runtime enum** (D8); uid generator →
  **v7** (D3/§ 5.6), with the content-addressed/v4 tension (ed2) surfaced and
  consciously deferred to the ledger seam.

**Second adversarial pass (codex mcp, independent).** Tasked with finding what the
first review + author missed. Corroborated D8 (only `entity.rs:198-205` reads
`kind.mode`; the four call sites hard-code one placement each — removal is mechanical),
alias-rollback transactionality, `schema_version == 1`, and the `OwnedEntityId`
migration surface — no change to those. New findings dispositioned:

- **codex-BLOCKING-2 — `scope.workspace` silently dropped → ACCEPTED.** Interop
  constraint 6 (:84-86/:154/:294) carries `workspace` on every memory from the first
  record; it is a hard-filter key (:314). v1 now scaffolds `workspace = "default"`
  unconditionally (§ 5.3). Distinct from the deferred `repo`/git frame (SL-008).
- **codex-MAJOR-3 — `show` read-path traversal → ACCEPTED.** `safe_join` was
  write-only (`entity.rs:289,328`); the `show` key arg is user input. v1 parses a
  validated `MemoryRef` and reuses `safe_join` on the read path (§ 5.2).
- **codex-MAJOR-4 — `show` render dropped `scope`/`anchor` → ACCEPTED.** Spec § Security
  (:365-367) lists both; header restored to the full set (§ 5.2).
- **codex-BLOCKING-1 — "non-idempotent uid violates content-addressing" → REJECTED as
  blocking, reclassified doc-fix.** § Identity (:268-270) defines `memory_uid` as a
  *minted-once, stored* UUID — not content-derived; `event_id`/`uuid5` (:274) is what
  carries the content-addressed/append-idempotent property, at the deferred ledger
  seam. A stored v7 uid is compliant. The finding correctly exposed *mis-framing* in
  the earlier §5.6/D3 wording, now corrected (§ 5.6, D3, § 5.5). v7 stands, on firmer
  ground.

Remaining for the plan step: confirm the seam-rename blast radius against
reservation-spec wording, and sequence the numeric-caller signature migration so each
suite stays green per commit.
