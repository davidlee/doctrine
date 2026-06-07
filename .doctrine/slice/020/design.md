# Design SL-020: Backlog entity v1: work-intake items (one kind + item_kind facet)

## 1. Design Problem

Make backlog items first-class doctrine entities (PRD-009) — the capture layer of
the change loop (ADR-003). Model the **whole** `backlog_item`: one entity
discriminated by an `item_kind` facet, the canon status lifecycle, the orthogonal
`resolution` close-reason, the per-kind descriptive facets (only risk has any), and
the outbound relation seam (ADR-004). Implement a **coherent v1 subset** —
capture / survey / inspect / transition — over the shared scaffold engine
(`src/entity.rs`, SL-003) without forking it, sequencing skill-wiring last. The
deferred layers (authored + derived priority, the promote bridge, `sync`, the
reverse-reference scan) must attach to a stable item without reshaping it.

Requirements in v1 reach: `REQ-049..053`, `REQ-057`, `REQ-058`, `REQ-059`.
Designed-but-deferred: `REQ-054` (priority), `REQ-055` (promote), `REQ-056`
(relation writing/derivation).

## 2. Current State

Backlog is *intent only*. `glossary.md` reserves the five 3-char prefixes
(`ISS/IMP/CHR/RSK/IDE`, all `folder = y`); `entity-model.md:74` fixes "one
`backlog_item` + `item_kind`, not six schemas; risk gets extra facet fields";
`entity-model.md:109` fixes the status vocab. Nothing is structural — items live
ad-hoc in the gitignored `backlog.local.md`.

The reuse seams already shipped:

- **`src/entity.rs`** — the kind-blind engine. `Kind { dir, prefix, scaffold }` is
  *data*, one dispatch site. `materialise(kind, fs, root, MaterialiseRequest::Fresh,
  inputs)` scans `kind.dir` for the monotonic next id (`candidate_id`), renders the
  `Fileset` from a pure `ScaffoldCtx`, and writes it atomically with race-retry.
  `Artifact::{File,Symlink}` paths are tree-root-relative (the engine is the sole
  joiner; rejects `..` escape).
- **`src/adr.rs`** — the *minimal* precedent: a single reserved `Kind`, a pure
  scaffold (sister TOML + prose MD + `NNN-slug` symlink), and a flat-enum status
  verb that mutates one authored TOML edit-preservingly via `toml_edit` (no
  reserialise; inert tables/comments/unknowns survive; F-1 refuses a malformed file
  rather than tail-inserting a key into a trailing subtable).
- **`src/spec.rs`** — the *closest* precedent: a subtype discriminator
  (`SpecSubtype::{Product,Tech}`) whose `kind() -> &'static Kind` selects one of two
  const `Kind`s with **own dir + own prefix + own scaffold** → independent id
  counters per subtype (`spec/product` `PRD`, `spec/tech` `SPEC`). Closed enums use
  `#[serde(rename_all = "kebab-case")]` + an `as_str()` render mirror; seeded-empty
  array files parse via `#[serde(default)]`. `run_list` iterates subtypes and
  concatenates per-subtype blocks. Payload-carrying edges (membership `label`+`order`,
  interaction notes) live in *sister* files (`members.toml`, `interactions.toml`)
  because a generic inline edge would be lossy; payload-free links do not.
- **`src/meta.rs`** — the shared list substrate: `Meta { id, slug, title, status }`,
  `read_metas`, `sort_and_filter`, `render_table`, `format_list`. Any entity whose
  TOML carries those four top-level keys round-trips into `Meta`.

## 3. Forces & Constraints

- **Governing canon.** PRD-009 (one entity; closed vocab; the membership test; the
  `status`/`resolution`/facet three-never-overlap invariant; outbound relations),
  `entity-model.md:74/:109`, `glossary.md` (ids + status), **ADR-004**
  (relations outbound-only, reciprocity derived), **ADR-003** (the loop; backlog is
  *capture*), **PRD-011** (derived graph priority *reads* this slice's outbound seam
  + authored-priority seam; entirely deferred here).
- **Storage rule.** Authored TOML, typed/enumerated; no untyped frontmatter bag.
  Prose in MD; never queried/derived data in prose.
- **Behaviour-preservation gate.** Backlog must ride `src/entity.rs` **unchanged**;
  the existing slice/ADR/spec/memory suites stay green unchanged.
- **Pure/imperative split.** Clock, disk, git are injected — the scaffold/render/
  validate layer is pure over `ScaffoldCtx` + embedded templates; verbs are the thin
  shell (the `date`/`uid` pattern).
- **Repo lints (as-you-go).** No `as` casts; `BTreeMap/BTreeSet` not `Hash*`;
  suppress with `expect(reason=…)` never bare `allow`; no `print_stdout` (use
  `writeln!`); string assembly per the repo deny-set. `cargo clippy` (bins/lib) zero
  warnings; `just check` clean.
- **rust-embed re-embed footgun** (`mem.pattern.embed.rustembed-recompile-and-symlinks`):
  a lone template edit is invisible until the embedding crate recompiles.
- **Authored-entity wiring trap** (`mem.pattern.install.authored-entity-wiring`):
  a new authored tree needs `install/manifest.toml` `[dirs].create` + the gitignore
  negation, or it is silently uncommittable.

## 4. Guiding Principles

- **Ride the engine; do not fork.** Backlog is the engine's next caller after ADR
  and spec — exactly the `spec` subtype seam applied to five kinds. Extract only
  genuinely-shared substrate; add nothing to `entity.rs`.
- **One entity; `item_kind` discriminates.** Kind variation is a facet on one
  entity, never parallel schemas. Only risk adds fields.
- **Type everything; open only provenance.** Closed enums for closed vocab
  (`item_kind`, `status`, `resolution`, risk levels); an open `Option<String>` only
  for risk `origin` (the `spec.category` precedent for descriptive provenance).
- **Inline payload-free relations.** The canon-correct location under the
  `entity-model` payload rule; sister edge files are reserved for payload-carrying
  edges, of which backlog has none (ADR-004). (A follow-up ADR will codify this
  inline-authored / registry-derived split corpus-wide — § Follow-Ups.)
- **Design the whole; ship a subset.** The deferred layers' product semantics are
  resolved (OQ-002/003/004) and honoured by the model even though their verbs wait.

## 5. Proposed Design

### 5.1 System Model

A new module **`src/backlog.rs`**, structured like `src/adr.rs`/`src/spec.rs`,
owning only the backlog-specific parts; all kind-blind machinery stays in
`entity.rs`/`meta.rs`.

```text
CLI (clap subcommands)
        │  run_new / run_list / run_show / run_edit          (thin shell: clock, disk)
        ▼
src/backlog.rs
   ItemKind ──kind()──► &'static Kind {dir, prefix, scaffold}     (5 consts)
   pure: backlog_scaffold(item_kind, ctx) · render_toml/md · validate · filter · format
        │
        ▼
src/entity.rs  materialise(kind, LocalFs, root, Fresh, inputs)    (UNCHANGED)
        ▼
   .doctrine/backlog/<kind>/<NNN>/…
```

The discriminator:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum ItemKind { Issue, Improvement, Chore, Risk, Idea }

impl ItemKind {
    const fn kind(self) -> &'static Kind { /* → ISSUE_KIND … IDEA_KIND */ }
    fn prefix(self) -> &'static str { /* ISS|IMP|CHR|RSK|IDE */ }
    fn as_str(self) -> &'static str { /* issue|improvement|chore|risk|idea */ }
    fn from_prefix(p: &str) -> Option<Self>;   // show <ID> auto-detect
    fn has_facet(self) -> bool { matches!(self, Self::Risk) }
}

const ISSUE_KIND: Kind = Kind { dir: ".doctrine/backlog/issue",
    prefix: "ISS", scaffold: |c| backlog_scaffold(ItemKind::Issue, c) };
// … IMPROVEMENT_KIND/CHORE_KIND/RISK_KIND/IDEA_KIND likewise
```

Boundary precedence `risk > issue > improvement > chore > idea` (PRD-009 §4) is
recorded as a comment/const for the future multi-kind resolver; v1's verbs never
exercise it (kind is an explicit `new` argument).

Three-layer model (the `entity-model` Rust convention, mirroring spec):

```text
RawBacklogToml   tolerant parse / toml_edit DocumentMut on mutate — preserves unknowns
BacklogItem      validated: typed id, ItemKind, Status, Option<Resolution>,
                 tags, Option<RiskFacet>, Relationships
(registry)       reverse-edge index + derived priority — DEFERRED (PRD-011)
```

### 5.2 Interfaces & Contracts

CLI surface (4 verbs):

```
doctrine backlog new <kind> "Title" [--slug <s>]
doctrine backlog list [--kind <k>] [--status <s>] [--tag <t>] [--all] [<substr>]
doctrine backlog show <ID>                       # ID = ISS-007 etc.
doctrine backlog edit <ID> --status <s> [--resolution <r>]
```

Types:

```rust
#[derive(…, clap::ValueEnum)] #[serde(rename_all="kebab-case")]
enum Status { Open, Triaged, Started, Resolved, Closed }     // closed canon set

#[derive(…, clap::ValueEnum)] #[serde(rename_all="kebab-case")]
enum Resolution { Fixed, Done, Mitigated, Accepted, Expired,  // one generic set,
                  Duplicate, WontDo, Obsolete, Promoted }     // kind-agnostic

#[derive(…, clap::ValueEnum)] #[serde(rename_all="kebab-case")]
enum RiskLevel { Low, Medium, High, Critical }

struct RiskFacet { likelihood: Option<RiskLevel>, impact: Option<RiskLevel>,
                   origin: Option<String>, controls: Vec<String> }

struct Relationships { slices: Vec<String>, specs: Vec<String>, drift: Vec<String> } // outbound-only

struct BacklogItem {
    id: u32, slug: String, title: String, kind: ItemKind,
    status: Status, resolution: Option<Resolution>,
    created: String, updated: String, tags: Vec<String>,
    facet: Option<RiskFacet>, relationships: Relationships,
}
```

Each enum carries an `as_str()` render mirror (the spec/adr convention) and
`Status::is_terminal()` (`Resolved | Closed`). Pure functions: `backlog_scaffold`,
`render_backlog_toml`, `render_backlog_md`, `validate_transition`, `visible`,
`format_rows`. Shell verbs read the clock and pass `today` in (adr precedent).

### 5.3 Data, State & Ownership

On-disk (per-kind tree → independent counters, the spec subtype seam ×5):

```
.doctrine/backlog/<kind>/<NNN>/
    backlog-<NNN>.toml      structured, queried
    backlog-<NNN>.md        prose body (no frontmatter)
.doctrine/backlog/<kind>/<NNN>-<slug> -> <NNN>     symlink alias
```

`backlog-<NNN>.toml` (risk shown; the four plain kinds omit `[facet]`):

```toml
id = 3
slug = "token-expiry-off-by-one"
title = "Token expiry off-by-one"
kind = "risk"
status = "open"
resolution = ""                 # seeded empty top-level key (see mutation note)
created = "2026-06-08"
updated = "2026-06-08"
tags = []

[facet]                         # risk only
likelihood = ""                 # low|medium|high|critical
impact = ""
origin = ""
controls = []

[relationships]                 # outbound-only (ADR-004); seeded empty
slices = []
specs = []
drift = []
```

`id/slug/title/status` are top-level so the file round-trips into `meta::Meta`
(verified: `Meta` does not `deny_unknown_fields`, so the extra keys are ignored;
`status` is a `String` there). `kind` is stored *and* implied by the dir — stored
so a single read yields the validated entity without path inspection; the dir
remains the reservation namespace.

**Empty-string ↔ `Option` seam.** `resolution`, `likelihood`, `impact` are seeded
`""` (not absent) so the `edit` verb can `toml_edit`-set them *in place* (a tail
`insert` of a fresh key would land after the `[facet]`/`[relationships]` headers —
adr F-1 corruption). `""` is not an enum variant, so the `RawBacklogToml →
BacklogItem` validation layer maps `"" → None` and parses any non-empty value to
its enum (erroring on an unknown token). `list`/`show` read the validated entity,
never the raw string.

**Templates** (rust-embed assets): two — `templates/backlog.toml` (the four plain
kinds) and `templates/backlog-risk.toml` (adds `[facet]`) — plus
`templates/backlog.md`. Two templates, not one-with-a-conditional, follows spec's
template-per-variant precedent and keeps each template literal. (Re-embed footgun:
a template-only edit needs a crate recompile to take.)

**Ownership.** `status`/`resolution` are authored, hand-settable, **ungated**
(slices/ADRs/specs ship this way). `relationships` are outbound-only; the reverse
view is derived (deferred). Facets are descriptive; no close-reason ever lands in a
facet (PRD-009 invariant).

**Install wiring.** `install/manifest.toml` `[dirs].create` gains `.doctrine/backlog`;
`.gitignore` gains the `!.doctrine/backlog/` negation (recursive — covers the five
kind subdirs). Without both, a created item is uncommittable.

### 5.4 Lifecycle, Operations & Dynamics

- **`new`** — resolve title/slug (`input::resolve_*`), read `clock::today()`,
  `materialise(item_kind.kind(), &LocalFs, &root, &MaterialiseRequest::Fresh,
  &Inputs{slug,title,date})`; print `Created ISS-007: <dir>`. Monotonic per-kind id,
  race-retry inherited. Counters are independent across kinds (separate `dir`s).
- **`list`** — for each `ItemKind`, read its tree into `Vec<BacklogItem>`; merge;
  filter; sort **kind then ascending id**; render via `meta::render_table`. **A
  kind's reservation dir need not exist** — the engine creates `.doctrine/backlog/
  <kind>` lazily on that kind's first `new`, and an empty dir is not git-committable
  (so it never survives a clone). The reader therefore treats a **missing
  `<kind>` dir as the empty set**, never an error; `list` on a virgin repo prints
  an empty table (the manifest seeds only the `.doctrine/backlog` parent — D4). Filters
  AND together. **Visibility:** with no `--status`/`--all`, show only active
  (`open|triaged|started`); `--all` shows all; an explicit `--status resolved`/
  `closed` shows that terminal state. Promoted items are terminal, so the
  hide-terminal rule already excludes them (PRD-011 `REQ-075`) — no special branch.
  Ordering is a stable grouping, **not** a priority claim (priority is PRD-011).
- **`show <ID>`** — split prefix → `ItemKind::from_prefix` → read
  `<kind>/<NNN>/backlog-<NNN>.toml` → render identity + (risk) facet + timestamps +
  **outbound** relationships. Pure stdout reassembly from the item's own file
  ("cannot go stale"); inbound refs are the future registry surface's (ADR-004).
- **`edit <ID> --status [--resolution]`** — `toml_edit` in place: set `status`,
  optionally `resolution`, bump `updated`. Edit-preserving; no-op guard (unchanged
  values write nothing); F-1 refuse if the seeded keys are absent (malformed →
  regenerate). **Coupling validation** (`validate_transition`): a terminal status
  requires a `resolution`; a non-terminal status forbids one. Setting
  `--resolution promoted` by hand is legal (the bridge automation is deferred).
  **Re-open (D9):** moving to a non-terminal status **auto-clears** `resolution`
  (back to `""`), so re-opening is one command and the `resolution ⟺ terminal`
  invariant always holds post-write. A `resolution=promoted` item is re-openable by
  hand — v1 is ungated, exactly the OQ-003 escape hatch ("the operator clears the
  resolution and abandons the slice by hand"); no special promoted guard.
  **Origin-edge interaction (the OQ-003 boundary).** Backlog-side re-open touches
  the *item* only; the slice→item promotion-origin edge is **slice-authored**
  (ADR-004 §1) and is *not* torn down by `backlog edit`. A bare backlog-side
  re-open of a `promoted` item is therefore the **improper** half of the escape
  hatch: it leaves an active item still claimed as a slice's consumed origin. The
  sanctioned correction is **slice-side** — abandoning the slice tears down the
  origin edge (OQ-003), with re-opening the item the *second* step, not a
  standalone undo. v1 stays ungated (it cannot reach across to the slice), so the
  residual dangling edge is a **derived-surface** concern: the registry
  reverse-scan (deferred, PRD-011) is the reconciler that surfaces an origin edge
  pointing at a non-terminal item — never authored-tier truth to repair here.

Deferred ops attach without reshaping: an authored `priority` field
(`REQ-054`/PRD-011 OQ-001), the `--from-backlog` promote bridge (sets terminal +
`resolution=promoted`; the slice authors the origin edge, ADR-004 §1), a `link`
verb, `sync`, and the registry reverse scan + derived priority (PRD-011).

### 5.5 Invariants, Assumptions & Edge Cases

Invariants (PRD-009 §4): `item_kind` fixed at capture; identity = prefix+number,
permanent, slug non-authoritative; `resolution ⟺ terminal status`; relationships
outbound-only; every facet typed (no bag); a terminal item stays addressable
("hidden" is a view); the relation seam is always present even when empty; a
`promoted` item's slice-side origin edge is reconciled **slice-side** (OQ-003 /
ADR-004 §1), never repaired by a backlog-side re-open (§5.4 origin-edge interaction).

Edge cases: empty backlog → first id per kind; **a kind with no reservation dir
yet** → that kind contributes the empty set to `list` (never an error), and
yields the first id on its first `new`; **id parse** splits on the last
`-`, upper-cases the prefix, parses the numeric tail as `u32` (tolerates `ISS-7`
and `ISS-007`, rejects an unknown prefix or non-numeric tail) — note the five
counters are independent, so `ISS-001` and `RSK-001` coexist and the prefix is
load-bearing for disambiguation; `show` on an unknown prefix → hard error; `edit`
on a missing id → hard error (never implicit create); malformed TOML (missing
seeded keys) → refuse, not corrupt; `edit --status started --resolution X` →
rejected by coupling (a non-terminal status takes no resolution); re-opening a
terminal item auto-clears its resolution (D9).

Assumptions (carried from scope, verified here): `entity.rs` admits this caller
with a per-kind fileset descriptor and **no engine change**; the `mkdir` reservation
primitive scales to five backlog namespaces (same primitive slices/ADRs/specs use).

## 6. Open Questions & Unknowns

All slice §Q1–Q6 are resolved (§7). Residual, all out of v1 scope:

- The exact authored-priority field shape (`rank`/`band`/`pin`/file) — PRD-011
  OQ-001; deferred with the whole priority layer.
- Whether `edit` should later grow `--title`/`--tag`/facet editing — v1 is
  `status`/`resolution` only; facets/tags are hand-edited (slug/title already
  non-authoritative). Candidate follow-up, not a v1 gap.

## 7. Decisions, Rationale & Alternatives

- **D1 — inline `[relationships]`** (not a sister `edges.toml`). Backlog edges are
  payload-free outbound (ADR-004); the `entity-model` payload rule puts those inline
  (slice/ADR precedent), reserving sister files for payload-carrying edges (spec
  membership). *Alt rejected:* sister edge file — imports spec's pattern without its
  payload reason. → **Follow-up ADR** to codify the inline-authored vs
  registry-derived edge split corpus-wide.
- **D2 — one generic `Resolution` enum, kind-agnostic**, with `resolution ⟺ terminal`
  coupling. Matches PRD-009's "single generic close-reason for every kind"; typed
  per the storage rule. *Alts:* per-kind sets (contradicts "single generic", adds a
  validity matrix); open string (untyped bag — forbidden).
- **D3 — `RiskLevel {low,medium,high,critical}`**; `likelihood`/`impact`
  `Option<RiskLevel>`; `origin` open `Option<String>` (provenance, `spec.category`
  precedent); `controls` `Vec<String>`. `critical` included — cheap, avoids a
  spartan scale.
- **D4 — per-kind dirs + prefixes** (independent counters). Forced by the engine
  (`Fresh` scans `Kind.dir`) and by glossary's independent `ISS-/IMP-/…` counters;
  the spec `product|tech` seam ×5. *Alt rejected:* one shared `backlog/id` dir —
  loses per-kind counters, needs a prefix re-derivation layer.
- **D5 — `list` hides terminal by default**, `--all`/explicit `--status` reveal;
  promoted falls out of the terminal rule. Deterministic kind-then-id order; not a
  priority claim.
- **D6 — no `link` verb in v1.** The relationship seam is seeded-empty and
  hand-edited, exactly as slice/ADR ship; storage lands, an edge-writer does not.
- **D7 — two templates** (`backlog.toml` + `backlog-risk.toml`). Spec's
  template-per-variant precedent; each template stays literal. *Alt:* one template +
  conditional `[facet]` injection — more scaffold branching for no real gain.
- **D8 — `edit` scope = `status`/`resolution` only in v1** (facets/tags/title
  hand-edited). Keeps the first mutating verb minimal and edit-preserving.
- **D9 — re-open auto-clears `resolution`.** Moving to a non-terminal status sets
  `resolution=""`; the verb never leaves a non-terminal item carrying a close-reason.
  *Alt rejected:* reject-unless-explicit-clear — two commands for a re-open, no
  safety gain (the post-state invariant is the same).

Mutation note (carried into D6/D8): `resolution` is seeded as a top-level `""` key
so `toml_edit` sets it **in place**; a tail-insert would land it after the
`[facet]`/`[relationships]` headers — inside a subtable (adr F-1 corruption). The
verb refuses a file missing the seeded `status`/`resolution`/`updated` keys.

## 8. Risks & Mitigations

- **Modelling drift back to four schemas** → one enum + one dispatch + one module;
  a test asserts a single `backlog.rs` caller over the unchanged engine.
- **Behaviour-preservation** → `entity.rs` untouched; existing suites must stay
  green unchanged (the gate). Any shared extraction is additive.
- **Status-vocab divergence vs the corpus** (`started`/`closed` vs
  `in-progress`/none) → not silently remapped; flagged for the deferred importer.
- **Storage rule vs catch-all** → facets enumerated; `origin` is the sole open
  string, justified as provenance.
- **rust-embed re-embed** → note the recompile dependency in the plan's template
  phase; assert rendered output in tests, not template bytes.
- **Authored-entity wiring trap** → manifest + gitignore negation is an explicit
  task with a "created item is `git add`-able" check.

## 9. Quality Engineering & Validation

TDD red/green/refactor throughout. Test classes (→ PRD-009 acceptance gates /
`REQ-049..053,057,058,059`):

- **Render round-trip** — `backlog-NNN.toml` parses into `meta::Meta` (4 keys) *and*
  the full `BacklogItem`; no `{{token}}` survives; risk seeds `[facet]`, plain kinds
  do not. **All five kinds** seed the mutable `status`/`resolution`/`updated` keys
  (the `edit`-in-place seam, §5.3) — asserted per kind, not risk alone, or `edit`
  would refuse a non-risk item as malformed.
- **Scaffold shape** — per-kind fileset (toml + md + symlink); risk includes the
  facet block; paths tree-relative.
- **`new`** — monotonic per-kind allocation and **counter isolation** across kinds
  (an `ISS` create does not advance `RSK`); race-retry inherited.
- **`list`** — the visibility/filter/order matrix: default hides terminal; `--all`
  and explicit `--status` reveal; `--kind`/`--tag`/substring AND; promoted hidden by
  the terminal rule; kind-then-id order. **Empty/missing-dir:** `list` on a virgin
  repo (no kind dirs created) prints an empty table, not an error; a kind with no
  reservation dir contributes the empty set (§5.4).
- **`show`** — prefix auto-detect; identity + facet + outbound relations render;
  unknown prefix errors.
- **`edit`** — coupling (terminal⟺resolution, both directions); no-op guard;
  malformed refuse; missing-id hard error; `updated` bumps, the rest survives.
- **Behaviour-preservation** — slice/ADR/spec/memory suites green unchanged.

`cargo clippy` (bins/lib) zero warnings; `just check` clean.

Suggested phase sequence (plan owns the final cut): (1) kind/enums/types +
templates + scaffold [render/round-trip]; (2) `new` + install wiring; (3) `list`;
(4) `show`; (5) `edit` + coupling; (6) skill-wiring loop-map (behaviour-preservation
on the shared skill/boot surface — last, after the verbs land).

## 10. Review Notes

Internal adversarial pass (findings + disposition):

- **R1 — re-open mechanism was underspecified.** "Clearing a resolution requires
  moving off terminal" named no mechanism. *Fixed:* D9 — non-terminal transition
  auto-clears `resolution`; invariant holds post-write.
- **R2 — empty-string ↔ `Option` parse seam was implicit.** `resolution`/levels are
  seeded `""` for in-place `toml_edit`, but `""` is no enum variant. *Fixed:* §5.3
  states the `"" → None` mapping in the validation layer.
- **R3 — id parse tolerance unspecified.** *Fixed:* §5.5 — split last `-`, upper
  prefix, `u32` tail (`ISS-7`/`ISS-007`); unknown prefix / bad tail error.
- **R4 — `Status::is_terminal` is *not* `slice::is_terminal_status`.** Different
  vocab (slice `…|audit|done` vs backlog `…|resolved|closed`). A backlog-local
  predicate is correct, not duplication — flagged so a reviewer does not "DRY" them
  into one wrong set.
- **R5 — gitignore recursion is a wiring risk, not a design change.** Re-including
  `.doctrine/backlog/` under an ignored `.doctrine/` needs the intermediate
  un-ignore. *Disposition:* the wiring phase proves it with a real `git add`
  assertion (§9), per `mem.pattern.install.authored-entity-wiring`.
- **R6 — claim "engine unchanged" is load-bearing.** Re-checked: `new` needs only
  `Fresh`; `list`/`show`/`edit` are backlog-local + `meta`/`toml_edit`. No
  `entity.rs` change. The behaviour-preservation suite is the proof.
- **R7 — `list` cross-kind order is arbitrary.** Declaration order (Issue…Idea) is
  a deterministic placeholder, *explicitly not* a priority claim; real ordering is
  PRD-011. Accepted as-is for v1.

External inquisition pass (`inquisition.md`, 2026-06-08) — dispositions:

- **C1 — false citation `glossary:109`.** The status vocab lives at
  `entity-model.md:109`; `glossary.md` (40 lines) has none. *Fixed:* `slice-020.md`
  divergence row corrected to `(entity-model:109)`.
- **C2 — missing kind-dir on `list`.** Per-kind dirs are engine-created lazily and
  empty dirs are not git-committable, so the reader must tolerate absence. *Fixed:*
  §5.4 `list` + §5.5 edge case + §9 test now state missing-dir → empty set; manifest
  seeds only the `.doctrine/backlog` parent (D4).
- **C3 — promoted re-open vs slice-side origin edge.** *Fixed:* §5.4 origin-edge
  interaction + §5.5 invariant — bare backlog-side re-open of a `promoted` item is
  the improper half; correction is slice-side (OQ-003/ADR-004 §1); the residual
  dangling edge is the deferred registry reverse-scan's to surface (PRD-011).
- **C4 — RECANTED (false charge).** The Inquisitor read the empty `requirement-0NN.md`
  prose tier and cried "hollow"; the requirement `description` + `acceptance_criteria`
  live in `requirement-0NN.toml`, fully populated (verified; `spec show PRD-009`
  synthesizes all eleven). The storage rule working as written — structured → TOML,
  prose → MD. §9 traceability has real acceptance criteria to conform to. No blocker.
- **C5/C6 — minor.** *Fixed:* stripped redundant `FR-006 /` (kept durable `REQ-054`)
  at `slice-020.md` ×3; §9 round-trip now asserts seeded keys for all five kinds.

Doctrinal alignment: re-confirmed against ADR-003 (capture step), ADR-004
(outbound-only; promote origin edge is slice-side), PRD-009 invariants
(status/resolution/facet three-never-overlap; ungated lifecycle), PRD-011 (priority
deferred; terminal/promoted hidden), the storage rule, and the reference-form
convention. No governance conflict surfaced; the one new corpus-wide question
(inline vs derived edges) is deferred to a follow-up ADR, not normalized around.

## Follow-Ups

- **ADR — inline-authored vs registry-derived edges.** Codify corpus-wide that
  payload-free relations are authored inline (`[relationships]`) while the generic
  `[[edge]] from/rel/to` form is the registry's *derived* representation; sister
  edge files are for payload-carrying edges only. Harmonise slice/ADR/backlog
  `[relationships]` array naming. (First `backlog new` candidate once the CLI ships.)
- Authored + derived **priority** (`REQ-054` / PRD-011), the **promote bridge**
  (`REQ-055`), the registry **reverse scan** + derived inbound view (`REQ-056` /
  PRD-011), `sync`, a `link` verb, the Spec-Driver **importer**, the `problem` kind.
