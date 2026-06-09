# Design SL-030: Policy entity kind (POL)

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`policy` (`POL-NNN`) is a planned governance kind in `glossary.md` — grouped with
`standard` (`STD`) and ADR — with zero representation: no CLI verb, no entity
tree, no boot-snapshot surface. STD is imminent. Introduce POL such that POL and
the coming STD do **not** each become a near-verbatim copy of `src/adr.rs`, while
honouring ADR's earned separateness (it records *decisions*; POL/STD record
*standing rules*).

## 2. Current State

ADR is the only shipped governance kind (`src/adr.rs`, SL-006/SL-025). It rides
`entity::Kind` as a top-level reserved kind over the kind-blind engine
(`src/entity.rs`): a numeric dir under `.doctrine/adr/` holding `adr-NNN.toml`
(queried metadata: `status`, `[relationships]`) + scaffolded `adr-NNN.md` +
`NNN-slug` symlink. Shared substrate already exists in `crate::listing`
(`validate_statuses`, `canonical_id`, `build`, `retain`, `render_table`,
`json_envelope`) and `crate::meta` (`Meta`, `read_metas`).

What `adr.rs` still owns and would be duplicated per new governance kind:
`list_rows` + `key` + `render_table` + `json_rows`/`AdrRow`; `read_adr` +
`AdrDoc` + `Relationships`; `format_show`; `show_json`; `parse_ref`;
`set_adr_status`. ~200 LOC of mechanical, near-identical-per-kind plumbing.

`src/boot.rs` projects accepted ADRs into the governance snapshot via a
`SourceKind` + a `("Accepted ADRs", …)` section row (`boot_sequence`), filtered to
`accepted` and ordered before the build-volatile section.

## 3. Forces & Constraints

- **DRY / no parallel implementation** (CLAUDE.md). With STD imminent, copying
  `adr.rs` ships a known 3× duplication.
- **As simple as possible, but no simpler / YAGNI.** A speculative governance
  facet unifying a deferred kind is over-reach.
- **`entity.rs` already chose kind-as-data.** The engine is kind-blind; `Kind`
  is a struct passed in. The right extension shares by *parameterization*, not by
  a new runtime discriminator.
- **Behaviour-preservation gate** (CLAUDE.md / project): changing shared
  machinery means the existing ADR suites are the proof — they must stay green
  unchanged.
- **Storage rule:** queried data in `*.toml`, prose in `*.md`; never queried data
  in prose.
- **clap constraint:** a `--status` value enum must be a concrete type per
  command — the CLI-arg type cannot be erased to `&str`.
- **Pure/imperative split:** clock/disk live in the thin shell; the status
  transition takes `today` as an input.

## 4. Guiding Principles

Share the *mechanism*, keep the *identity*. Each governance kind stays a distinct
entity (own prefix, tree, status vocab) — a thin module of **data**; the
identical list/show/status logic lives once in a shared spine, parameterized by
that data. This is the consistent extension of the existing kind-blind engine,
not a second discrimination axis.

## 5. Proposed Design

### 5.1 System Model

```
                 entity.rs  (kind-blind engine: materialise, claim, scan)
                 meta.rs    (Meta, read_metas)        listing.rs (filter/format spine)
                     ▲                                      ▲
                     │ NEW: governance.rs  ── shared CLI/status spine ──┘
                     │   list_rows · key · render_table · json_rows(GovRow)
                     │   read_doc · Doc · Relationships · format_show · show_json
                     │   parse_ref · set_status        (all take &GovKind)
        ┌────────────┼────────────┐
   adr.rs (thin)  policy.rs (thin)  [standard.rs later, thin]
   ADR_KIND        POLICY_KIND
   AdrStatus       PolicyStatus     ← per-kind clap enums (concrete, clap-required)
   ADR_STATUSES    POLICY_STATUSES  ← known-sets (data)
   is_hidden       is_hidden        ← hide-set (data)
   scaffold        scaffold         ← prose/toml templates (data)
   run_* wrappers  run_* wrappers   ← bind concrete enum, delegate to governance::*
```

Module layering (ADR-001, leaf ← engine ← command). `governance.rs` is a
**command-tier** shared module — it legitimately uses `root::find` and
`clock::today` (shell concerns), so it does **not** sit at the engine tier beside
the pure leaf `listing.rs`. It depends downward on `entity`/`meta`/`listing`
(engine/leaf) and sideways on the `root`/`clock`/`install` shell utilities. The
per-kind command modules (`adr`/`policy`) depend on `governance` (command →
command, acyclic); `boot.rs` already depends on `adr` and will likewise call
`policy::list_rows`. No engine/leaf module depends on `governance.rs`, so no
cycle is introduced. Within `governance.rs`, two faces: **io/compute** helpers
that take a resolved `root`/`path` as input (`list_rows`, `set_status`,
`read_doc`, `parse_ref`, `format_show`, `show_json` — boot calls `list_rows`
directly) and the thin **shell** wrappers (`run_*`) that do `root::find` +
`clock::today` + stdout. (Codex BLOCKER-1.)

### 5.2 Interfaces & Contracts

The descriptor the spine binds:

```rust
// governance.rs
pub(crate) struct GovKind {
    pub kind: entity::Kind,                  // dir, prefix, scaffold (existing struct)
    pub stem: &'static str,                  // file stem AND JSON envelope/object key: "adr" / "policy"
    pub statuses: &'static [&'static str],   // known-set (validate_statuses authority)
    pub hidden: fn(&str) -> bool,            // default-list hide-set
}
```

`json_label` is **dropped** (Codex MINOR-7): in `adr.rs` the JSON `"kind"` value
and the dynamic object key are both `"adr"` — identical to the file stem. A
separate field only admits incoherent states (`stem="policy", json_label="adr"`)
no kind wants. `stem` serves both file naming and JSON labelling — 4 fields, all
exercised by ADR **and** POL from day one (tightens R3).

Spine entry points (each takes `&GovKind`; the clock/`today` is injected):

```rust
pub(crate) fn list_rows(g: &GovKind, root: &Path, args: ListArgs) -> Result<String>;
pub(crate) fn run_show(g: &GovKind, path: Option<PathBuf>, reference: &str, format: Format) -> Result<()>;
pub(crate) fn set_status(g: &GovKind, gov_root: &Path, id: u32, status: &str, today: &str) -> Result<()>;
pub(crate) fn run_new(g: &GovKind, path: Option<PathBuf>, title: Option<String>, slug: Option<String>) -> Result<()>;
```

Per-kind wrapper (policy.rs), binding the concrete clap enum at the boundary:

```rust
pub(crate) fn run_status(path: Option<PathBuf>, id: u32, status: PolicyStatus) -> Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let gov_root = root.join(POLICY_KIND.kind.dir);
    governance::set_status(&POLICY_KIND, &gov_root, id, status.as_str(), &crate::clock::today())?;
    writeln!(io::stdout(), "POL {id:03}: {}", status.as_str())?;
    Ok(())
}
```

`GovRow` replaces the per-kind `AdrRow` (identical fields: `id/status/slug/title`,
all `String`). `Doc`/`Relationships` replace `AdrDoc`/ADR's `Relationships`
(identical shape). `show_json`/`json_envelope` use `g.stem` instead of the
literal `"adr"` (the dynamic object key forces a hand-built `serde_json::Map`, not
the `json!` macro — see §10 R2). `parse_ref` must **preserve ADR's exact
semantics** (Codex MAJOR-3): strip `"{PREFIX}-"` or its lowercase `"{prefix}-"`
(the two literal cases ADR accepts today, parameterized on `g.kind.prefix`) or a
bare id — **not** case-insensitive (a case-insensitive strip would newly accept
`AdR-7`, an observable ADR behaviour change). Error text interpolates the prefix.

**Tag-filter parity (Codex BLOCKER-2, downgraded — pre-existing).** `adr::key`
returns `tags: Vec::new()` ([src/adr.rs:219]) and `meta::read_metas` reads only
`id/slug/title/status` — so ADR's `--tag` axis already matches nothing. The
shared spine **inherits this limitation**; POL's `--tag` is likewise inert. This
is parity, not a regression — but the design does **not** claim governance kinds
support tag filtering. A real tag reader (extend `Meta` or a sibling read) is a
**follow-up** (§6), not in scope here.

### 5.3 Data, State & Ownership

- **`.doctrine/policy/NNN/`** — authored tree: `policy-NNN.toml` (queried:
  `id/slug/title/status/created/updated` + inert `[relationships]`), `policy-NNN.md`
  (prose body), `NNN-slug` symlink. Mirrors the ADR layout exactly.
- **`install/templates/policy.{toml,md}`** — rust-embedded scaffold templates;
  `policy.toml` mirrors `adr.toml` (metadata + `[relationships]`), seeds
  `status = "draft"`. `policy.md` body reuses the tuned prior art from
  `../spec-driver/supekku/templates/policy-template.md` (attributed) — sections
  **Statement / Rationale / Scope / Verification / References** — but **drops its
  YAML frontmatter**: per the storage rule and ADR-D1, metadata lives in the
  sister TOML, never in the prose body.
- **Status vocab (data, owned by policy.rs):** `draft → required → deprecated /
  retired`. `required` is the load-bearing in-force state (a policy is *required*
  to follow — term from the supekku prior art; the boot section shows only
  these). `deprecated` = sunsetting but extant; `retired` = terminal off.
  Supersession is a **relationship** (`relationships.supersedes`), not a status —
  cleaner than ADR, which carries both. Hide-set: `deprecated`, `retired`.
  Relationship axes: same four as ADR (`supersedes`, `superseded_by`, `related`,
  `tags`).

### 5.4 Lifecycle, Operations & Dynamics

`doctrine policy new|list|show|status` mirror the ADR verbs:
- **new** → `governance::run_new(&POLICY_KIND, …)` → `entity::materialise` (Fresh:
  monotonic id, race-retry inherited). Output `Created POL NNN: <dir>`.
- **list** → `governance::list_rows`: `validate_statuses` against
  `POLICY_STATUSES`, shared filter/format, hide-set applied, sorted by id,
  prefixed `POL-NNN` ids + header.
- **show <POL-NNN|NNN>** → read one policy's toml (as data) + md (prose),
  render readable whole or faithful JSON. Read-only, single-entity.
- **status** → edit-preserving `set_status`: set `status`, stamp `updated`,
  I5 no-op guard (unchanged status writes nothing), malformed-refuse guard
  (missing `status`/`updated` → bail, never tail-insert into `[relationships]`).

### 5.5 Invariants, Assumptions & Edge Cases

- **Behaviour preservation:** ADR's CLI-observable output (new/list/show/status)
  is byte-identical before and after the extraction. The existing adr suite is
  the proof; it stays green unchanged at the CLI surface.
- **Known-set ↔ enum lockstep:** `POLICY_STATUSES` mirrors `PolicyStatus`
  variants, pinned by a drift canary test (mirrors `adr_known_set_matches_variants`).
- **Hide-set ⊆ known-set:** `is_hidden` only names statuses in the vocab.
- **boot in-force filter:** only `required` policies project. **Caveat (Codex
  MAJOR-4):** `boot::section_or_marker` collapses *both* a producer `Err` *and* a
  genuinely-empty listing into the same `not yet populated` marker
  ([src/boot.rs:171]) — so a malformed policy corpus renders as "no policies",
  hiding corruption. This is **pre-existing, shared behaviour** (ADR's section
  behaves identically); changing it is a boot-wide concern, **out of scope** for
  SL-030. Documented as inherited; `boot --check`'s disk-sentry remains the
  backstop. Tests assert the empty→marker case; the error→marker case is
  acknowledged as inherited, not introduced.
- **Supersession ⇏ status (Codex MAJOR-5 — invariant + gap).** Because
  supersession is a *relationship* and boot filters on *status* only, a policy
  listed in another's `supersedes` can still read `required` and so appear in
  "Active Policies" alongside its replacement — exactly as ADR's status-only
  accepted-filter behaves today ([src/boot.rs:123]). **Invariant (authored
  discipline):** a policy named in any `supersedes` MUST be moved off `required`
  (to `retired`). No `policy supersede` verb exists to enforce this mechanically
  in v1 (parity with ADR's unbuilt `adr supersede`/F1) — it is a **follow-up**
  (§6). boot is, by design, a status projection, not a supersession-resolver.
- **Edit-preserving round-trip:** `[relationships]`, comments, unknown keys
  survive `status` (toml_edit in place; never reserialised).

## 6. Open Questions & Unknowns

- `OQ-1`: glossary POL row is already present; governance kinds show a blank
  `folder` column (ADR does too) — **no glossary change** intended. Confirmed in
  scoping.
- `OQ-2`: `parse_ref` error text parameterized on `g.kind.prefix` (e.g.
  `expected POL-007 or 7`) — resolved, folded into §5.2.
- `OQ-3` **(RESOLVED):** status vocab = `draft/required/deprecated/retired`
  (hybrid — `required` from supekku prior art, terminal `retired` added). See
  D2 / §10 R5.

**Deferred follow-ups (surfaced by the Codex pass, out of SL-030 scope):**
- **Governance tag reader** — extend `Meta`/a sibling read so `--tag` actually
  filters governance kinds (today inert for ADR too). Benefits ADR + POL + STD.
- **`policy supersede` verb** — mechanically flip a superseded policy off
  `required` (enforces the §5.5 invariant); parity with ADR's unbuilt F1
  `adr supersede`.
- **boot error vs empty disambiguation** — distinct failure marker / fail
  `boot --check` on producer errors; boot-wide, benefits every section.

## 7. Decisions, Rationale & Alternatives

- **D1 — Distinct kinds + shared spine (Option B), ADR migrated now.** POL/STD/ADR
  each a thin per-kind module over a shared, parameterized `governance.rs`.
  - *Rejected — copy `adr.rs` (Option A):* with STD imminent, knowingly ships 3×
    duplication; "extract later" leaves the half-extracted worst state.
  - *Rejected — one `governance` entity + `doc_kind` field (Option C):* introduces
    a second "what type is this?" axis alongside `entity::Kind` (incoherent); POL
    and STD diverge on status vocab + template, so it trades duplicated *modules*
    for duplicated *branches* — worse cohesion plus a dispatch layer. Its DRY claim
    is partly fake; clap still forces per-doc_kind status enums.
  - *Migrate ADR now, not later:* a half-extracted spine (POL on shared fns, ADR
    still fat) is the worst state; the behaviour-preservation gate makes the
    extraction safe (suites prove it), and with STD imminent it pays off now.
- **D2 — Status vocab `draft/required/deprecated/retired`; supersession is a
  relationship, not a status.** Policy is a standing rule, not a decision —
  ADR's `proposed/accepted/rejected` reads wrong for a policy. `required`
  (in-force) is the supekku prior-art term, sharper than `active` for a rule;
  `retired` is the terminal off-state the prior art lacks (hybrid resolution,
  OQ-3). Keeping supersession out of the status enum (unlike ADR) avoids the
  dual-source ambiguity ADR carries.
- **D3 — Spine at the engine tier (`governance.rs`), beside `meta`/`listing`.**
  Preserves ADR-001 layering; per-kind modules depend on it, it depends only
  downward.
- **D4 — clap enums stay per-kind, erased to `&str` at the spine boundary.** clap
  requires a concrete `ValueEnum`; the runtime logic shares on `&str` +
  `&[&str]`. This is also why Option C cannot fully unify.

## 8. Risks & Mitigations

- **R1 — Extraction regresses ADR behaviour.** *Mitigation:* behaviour-preservation
  gate — run the full adr suite green, unchanged, after each extraction step; the
  extraction is a refactor, not a behaviour change.
- **R2 — Install surface silently broken** (the known trap,
  `mem.pattern.install.authored-entity-wiring`). *Mitigation:* wire all three
  surfaces (manifest dir, `.gitignore !.doctrine/policy/`, parity) and assert a
  POL is committable + a fresh install scaffolds the tree.
- **R3 — Over-abstraction:** the `GovKind` descriptor grows fields only one kind
  uses. *Mitigation:* every field must be exercised by ≥2 kinds at introduction;
  ADR + POL both bind all five fields from day one.
- **R4 — boot drift:** policy section stale vs current governance.
  *Mitigation:* the existing `boot --check` disk sentry already covers the whole
  snapshot; the new section rides the same recompute.

## 9. Quality Engineering & Validation

TDD red/green/refactor. Test surfaces:
- **governance.rs (new, shared):** unit tests for `list_rows` hide-set + sort +
  prefix, `set_status` edit-preserving + no-op + malformed-refuse, `parse_ref`
  prefix/case/pad, `format_show`/`show_json` shape. Driven through both ADR and
  POL descriptors to prove parameterization.
- **policy.rs:** render round-trip (toml → `Meta`), hostile-title escape
  (`toml_string`), scaffold lays out two files + symlink, `run_new` monotonic id,
  status known-set ↔ enum drift canary.
- **adr.rs (migration):** existing suite stays green unchanged at the CLI surface
  — the behaviour-preservation proof. Test-internal `AdrDoc`→`governance::Doc`
  retypes are not behaviour changes.
- **boot.rs:** extend `regenerate_projects_*` — a `required` policy appears under
  "Active Policies"; `draft`/`retired` are hidden; empty → marker.
- **install:** fresh install scaffolds `.doctrine/policy`; `git add` of a
  `policy-NNN.toml` succeeds (not gitignored).

Gate before commit: `just check` (plain `cargo clippy`, zero warnings), fmt.

## 10. Review Notes

Internal adversarial pass (assumptions verified against `src/`):

- **R1 (sharpened) — "ADR suite stays green unchanged" overclaimed.** True at the
  *CLI-observable* surface (new/list/show/status output). But adr.rs unit tests
  poke internals that this design *moves* (`read_adr`, `set_adr_status`,
  `format_show`, `show_json`, `parse_ref`, `AdrDoc`). Those tests **relocate** to
  `governance.rs` and are driven by both descriptors. The behaviour-preservation
  gate is satisfied by the integration/CLI-level assertions, not the relocated
  unit tests. §9 amended accordingly.
- **R2 — `show_json` dynamic key.** ADR builds `json!({"kind":"adr","adr":doc,…})`
  — the *second* key is also dynamic under a `json_label`. The `json!` macro
  cannot take a runtime key; the spine builds a `serde_json::Map` manually
  (`insert("kind", label)`, `insert(label, doc)`, `insert("body", body)`). Impl
  note, not a design change.
- **R3 — `stem` must be explicit, not derived.** ADR coincidentally has
  `stem == prefix.to_lowercase()` (`"adr"`/`"ADR"`); **policy breaks the
  coincidence** (`stem = "policy"`, `prefix = "POL"`). Validates the explicit
  `GovKind.stem` field — never derive stem from prefix. (`meta::read_metas` takes
  the stem; `listing::canonical_id` takes the prefix — confirmed distinct args.)
- **R4 — `GovKind` as a `const` holding fn pointers** (`hidden: fn(&str)->bool`,
  embedded `kind.scaffold`). Verified const-compatible — `ADR_KIND` already holds
  a `scaffold` fn pointer in a `const`. ✓
- **R5 — Status vocab vs prior art (→ OQ-3, RESOLVED).** supekku
  `policy-template.md` used `draft/required/deprecated`; D2 originally chose
  `draft/active/deprecated/retired`. Reconciled to the hybrid
  `draft/required/deprecated/retired` (`required` from prior art + terminal
  `retired`). The spine treats the vocab as data, so the resolution is a one-line
  known-set + enum change.
- **Verified shared seams:** `entity::Kind` fields are `pub` (embeddable in
  `GovKind`); `listing::{canonical_id,build,retain,validate_statuses,render_table,
  json_envelope}` and `meta::read_metas(root, stem)` all parameterize cleanly —
  no new shared plumbing needed beyond `governance.rs`.
- **Prior-art reuse:** `policy.md` body = supekku
  `Statement/Rationale/Scope/Verification/References`, frontmatter dropped
  (storage rule). Attributed in §5.3.
- **ADR template checked vs supekku `ADR.md` (no change).** doctrine's `adr.md`
  body already matches supekku structurally (Context/Decision/Consequences{+/−/0}/
  Verification/References) and correctly keeps metadata in `adr.toml` (not YAML
  frontmatter). The only divergences — hint style (HTML-comment vs prose) and a
  richer status vocab (`draft`/`revision-required`) — were considered and
  **declined**: ADR stays untouched beyond the spine extraction, keeping SL-030
  scoped to policy.
