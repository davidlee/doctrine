# Design SL-136: Extend tagging to all entity types — generic cross-kind tag verb

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`doctrine backlog tag` and `doctrine memory tag` exist as separate, forked write
paths over different TOML storage locations. Most entity kinds cannot be tagged
at all (no write verb), and `list --tag` silently matches nothing for governance
because the filter reads no tags. Provide **one** cross-kind `doctrine tag` verb
and make `list --tag` work, without a parallel implementation per kind.

Motivation: SL-133 (IMP-118) will add a project-global per-tag **coefficient**
(default 1.0) feeding graph-traversal prioritisation — tags become a first-class
classification axis, not just backlog/memory decoration. This slice does not
build the coefficient; it must keep tag **normalisation stable** so coefficient
lookups key consistently (`tag::normalize_tag` already canonicalises).

## 2. Current State

- `tag::normalize_tag` — the shared WRITE chokepoint (leaf, imports nothing).
  Already extracted (SL-100). Trim→lowercase→`[a-z0-9_:-]` charset reject.
- `backlog::apply_tags` (`backlog.rs`) — root-level `tags`, set-merge, no-op
  guard, stamps `updated`, **bails (F-1) if `tags` key absent** — justified
  in-comment by a tail-insert-corruption fear (see §5.5 / D4: empirically false
  in toml_edit 0.22).
- `backlog::fold_filter_tag` — the lenient filter-side fold (trim+lowercase, no
  reject). Lives in backlog; needed by every `list --tag` path.
- `integrity::parse_canonical_ref(ref) -> (&KindRef, id)` + `kind_by_prefix` —
  the universal numbered-ref resolver. `entity::id_path(root, kind, id, Toml)`
  builds the `<stem>-NNN.toml` path (already used by `commands/relation.rs`).
- `meta::Meta` — the shared list reader (`id/slug/title/status`); **no tags**.
  `slice::key()` and `governance::key()` both hardcode `tags: Vec::new()`, so
  `slice/adr/policy/standard/rfc list --tag` match nothing.
- `listing::build` — resolves filter+format; `tags_admit` already implements OR
  tag matching. `--tag` clap arg already on the shared `ListArgs`. `build` does
  **not** fold `--tag` inputs; backlog folds pre-build.
- Tag storage today is heterogeneous: backlog/knowledge/spec seed root `tags=[]`;
  governance (ADR/POL/STD) + RFC carry `[relationships].tags`; slice has none;
  memory uses `[scope].tags`.

## 3. Forces & Constraints

- ADR-001 module layering (leaf ← engine ← command, no cycles): `tag.rs` stays a
  leaf; `commands/tag.rs` is command-tier; `listing` (engine) may call `tag`
  (leaf).
- "Kind is data, not a trait" (`mem.pattern.entity.kind-is-data-not-trait`): no
  trait abstraction over kinds — ride the existing `integrity::KINDS` data table.
- No parallel implementation (CLAUDE.md): reuse `parse_canonical_ref`/`id_path`,
  not a new resolver; backlog must **delegate**, not duplicate.
- Behaviour-preservation gate: backlog tag + governance/slice list suites are the
  proof for the shared-machinery change; they stay green (one backlog bail-test
  is rewritten — see D4).
- Pure/impure split: the write core takes an injected `today`; no clock/disk in
  the leaf.

## 4. Guiding Principles

- **Uniformity over special-casing.** One storage location (root `tags`), one
  write leaf, one filter fold. Heterogeneity is the accident to remove, not
  preserve.
- **Reuse the seam.** Resolution, path-building, set-merge, no-op guard already
  exist — generalise, don't re-roll.
- **Self-healing migration.** Insert-if-missing means existing un-keyed entities
  (every pre-136 slice) need no bulk migration — first tag seeds the key.

## 5. Proposed Design

### 5.1 System Model

Every taggable entity stores a **root-level `tags: Vec<String>`** (governance/RFC
migrate root-ward — governance-changing, see D6). Taggable = a **curated set
whose read surfaces render tags** (D2): slice, ADR/POL/STD/RFC, backlog
(ISS/IMP/CHR/RSK/IDE), knowledge (ASM/DEC/QUE/CON), spec (PRD/SPEC), **REQ**.
**Excluded** — concept-map, review (RV), REC, revision (REV): their
`show`/`--json`/`list` do **not** render tags, so writing would create
**write-only metadata** that silently vanishes (Codex MAJOR-1). Wiring their read
surfaces is deferred to **IMP-144**; until then the verb refuses them with a
pointer to IMP-144. A `TAGGABLE` prefix const in `tag.rs` is the gate; storage
stays uniform root, so there is no per-kind *location* dispatch — only a
membership check. Memory excludes itself (its `mem.*` ref fails
`parse_canonical_ref`).

```
doctrine tag set <ID> <TAGS...> [-d/--remove <TAGS>...]   # additive-merge
doctrine tag clear <ID>                                    # remove all
```

Resolution (no per-kind location logic; only a taggability gate):

```rust
let (kref, id) = integrity::parse_canonical_ref(reference)?;   // SL-136 → (slice, 136)
ensure!(tag::TAGGABLE.contains(&kref.kind.prefix), "{} not taggable yet (IMP-144)", kref.kind.prefix);
let path = entity::id_path(root, kref.kind, id, entity::Ext::Toml);
let changed = tag::apply_tags_set(&mut doc, &adds, &removes, &today)?;
```

### 5.2 Interfaces & Contracts

Shared leaf (`tag.rs`):

```rust
/// Root-level `tags` SET edit on a held &mut DocumentMut, edit-preserving.
/// Callers pass PRE-NORMALIZED sets. No disk/clock. true = wrote.
pub(crate) fn apply_tags_set(
    doc: &mut toml_edit::DocumentMut,
    adds: &BTreeSet<String>,
    removes: &BTreeSet<String>,
    today: &str,
) -> anyhow::Result<bool>;

/// The lenient FILTER-side fold (trim+lowercase, NO reject). Hoisted from backlog.
pub(crate) fn fold_filter_tag(raw: &str) -> String;
```

CLI (`cli.rs` + `commands/tag.rs`):

```rust
enum TagCommand {
    Set   { id: String, tags: Vec<String>, remove: Vec<String>, path: Option<PathBuf> },
    Clear { id: String, path: Option<PathBuf> },
}
// Command::Tag { command: TagCommand } -> commands::tag::dispatch
```

`backlog::BacklogCommand::Tag` is **kept** (back-compat) and delegates to
`tag::apply_tags_set`. `memory tag` is **out of scope** — untouched (it writes
`[scope].tags`, a separate location).

### 5.3 Data, State & Ownership

- `tag.rs` owns the entire tag vocabulary: `normalize_tag` (write), `fold_filter_tag`
  (filter), `apply_tags_set` (write core).
- `meta::Meta` gains `#[serde(default)] tags: Vec<String>` — absent parses as
  empty (no migration of existing files needed for the read path).
- Template seeding (`tags = []` at root): add to **slice, requirement (REQ)**;
  backlog/knowledge/spec already seeded; gov/RFC seeded via the §5.4 migration.
  Excluded kinds (CM/RV/REC/REV) get no seed — not taggable until IMP-144.
- **Full read-surface parity for every newly-included kind** (Codex 2nd-pass
  BLOCKER): a kind is in `TAGGABLE` only if **all three** read surfaces render
  tags — `list --tag` filter, `show` table, and `--json`. Partial wiring (filter
  but not show/json) is the same write-only smell D2 killed, only quieter. In
  scope per kind:
  - **slice** — `Meta.tags` + `slice::key().tags`; `show` table tag row; show-json field.
  - **spec (PRD/SPEC)** — `spec::key().tags` (was omitted from the worklist);
    spec `show` tag row; show-json field.
  - **REQ** — `req_key().tags` (already wired, `spec.rs:1665`); add `tags` to
    `ReqJsonRow` (`spec.rs:~1556`) and to the `show_json` member req object
    (`spec.rs:~1167`); REQ `show` tag row.
  - **gov/RFC** — `governance::key().tags`; the §5.4 migration repoints the `show`
    table render root-ward. **`--json` needs no builder change (RV-129 F-2,
    root-expose):** `governance.rs:360` show_json is serde-driven
    (`serde_json::to_value(doc)`), so once `tags` is stored at root it renders at
    root automatically — uniform with every other kind's JSON. The existing
    `supersedes`/`related` splice (reconstructing the SL-095 `[[relation]]` shape)
    is **not** extended to tags: tags is a pure path relocation (string array →
    same string array), nothing to reconstruct. The only `--json` work is updating
    the goldens (§5.4) to expect tags at root.
  - **backlog / knowledge** — already render tags on all three; unchanged.
  (The excluded kinds are excluded because none of their three surfaces is wired
  — IMP-144.)

### 5.4 Lifecycle, Operations & Dynamics

`apply_tags_set` semantics (generalised from `backlog::apply_tags`):

1. `current` = root `tags` array read verbatim, **absent → empty set**.
2. `new = (current ∪ adds) ∖ removes`, stored **sorted**.
3. **No-op guard:** `set(new) == set(current)` → `Ok(false)`, no write, **no
   seed, no stamp**. (`tag clear` on an untagged entity is a clean no-op.)
4. Else: insert-or-replace root `tags` with the sorted array (insert-if-missing,
   §5.5 safe); stamp `updated = today` **only if the `updated` key exists**;
   `Ok(true)`.

`clear` = read current tags, pass them as `removes` with empty `adds` → step 3
no-ops when already empty, step 4 empties otherwise. No dedicated leaf path.

Verb shell (`commands/tag.rs`): guard the ref exists (`integrity::ensure_ref_resolves`
— `parse_canonical_ref` only parses, never probes disk, so an unknown id must
fail cleanly here, not as a raw read error), `set` requires ≥1 add-or-remove
(A5, mirror backlog), normalise adds/removes via `normalize_tag`, reject
add∩remove overlap, resolve path, call leaf, `write_atomic` if changed, print
`Tagged <ID>: a, b` / `(none)`.

List `--tag` fix:
- `slice::key()` + `governance::key()` + **`spec::key()`** set `tags:
  m.tags.clone()` (governance covers ADR/POL/STD and RFC via `governance::run_list`;
  `spec::key()` covers PRD/SPEC and was missing from the first-pass worklist —
  `src/spec.rs:~1349` still hardcodes `Vec::new()`). REQ's `req_key()` already
  wires `tags` (`spec.rs:1665`).
- **Show + JSON render** for each newly-included kind (full-parity, §5.3): slice
  and spec gain a `show` tag row + show-json `tags` field; REQ gains `tags` on
  `ReqJsonRow` + the `show_json` member object (RV-129 F-5: **additive** — both REQ
  JSON sites drop tags today, so the parity test asserts tag **presence**, not
  "unchanged"); governance's `--json` tags fall out of the serde-driven
  `to_value(doc)` for free once storage moves root-ward (RV-129 F-2, no builder
  change), while its `show` **table** render is repointed root-ward by the §5.4
  migration.
- **Centralise the filter-fold into `listing::build`** (fold each `--tag` input
  trim+lowercase via `tag::fold_filter_tag`); remove backlog's redundant
  pre-fold. Idempotent → every list kind gets case-insensitive `--tag` uniformly.

Governance/RFC migration — **governance-changing** (D6). The storage move
contradicts two tech specs that pin governance tags as typed; the amendment rides
a **Revision authored at reconciliation** (ADR-013), the in-slice code/test/corpus
changes land now. Blast radius (measured, Codex BLOCKER-1/MAJOR-2):

- **Canon (the REV, at closure):** **three** specs pin governance tags as typed
  and are amended root-level by one Revision — SPEC-005 §Decisions D2 ("`tags`
  remain in the typed `[relationships]` table"), SPEC-018 §relations ("`tags` …
  stays typed"), and **SPEC-016** (responsibility text describes the governance
  `[relationships]` seam as carrying `tags` — `spec-016.toml:17`; Codex 2nd-pass).
- **Struct surgery (`governance.rs`):** drop typed `tags` from `Relationships`;
  add root `tags` to `Doc`; **repoint the `show` render** `doc.relationships.tags`
  → `doc.tags` (~L313-320); fix the two `Meta` literals (A2).
- **Corpus (~28 files):** strip the `[relationships].tags` line from ADR/POL/RFC
  tomls. No root `tags=[]` seed needed — `Doc`/`Meta` read root with
  `#[serde(default)]` (A4). **RFC-002's live tags**
  (`program, consumption-surfaces, estimate, value, scoring`) restored by one
  `doctrine tag set RFC-002 …` after the verb lands.
- **Templates (4):** adr/policy/standard/rfc — move `tags` from `[relationships]`
  to root.
- **Goldens to rewrite (mechanical):** `tests/e2e_adr_cli_golden.rs` (fixture,
  `show` render, JSON envelope nesting, status-edit-preserve, refusal fixture),
  `tests/e2e_standard_cli_golden.rs` (mirror), `tests/e2e_catalog_cli.rs`
  (fixture), `src/adr.rs:322` (typed-tags assert), and the two render-unit tests
  `src/policy.rs:295` + `src/standard.rs:302`
  (`render_*_toml_relationships_are_preserved…` loop over `["superseded_by",
  "tags"]` — drop `tags` from the typed-axis loop; Codex 3rd-pass). **Invert**
  `tests/e2e_relation_migration_storage.rs` — its
  `governance_corpus_..._tags_stay_typed` + `assert_governance_shape` guards now
  assert tags are root, not typed; **extend `governance_files()` (`:86`) to
  include `rfc`** — it scans only `adr/policy/standard`, so RFC migration is
  currently unverified (Codex 2nd-pass).
- **`relation_graph.rs` fixtures (Codex 2nd-pass):** `seed_adr`'s comment
  (`:1300`, "supersedes/superseded_by/tags stay typed") and the inline ADR
  fixtures (`:1662-1668`, `[relationships]…tags = []`) bake typed governance
  tags; repoint to root `tags` so the relation-graph suite stays green.

### 5.5 Invariants, Assumptions & Edge Cases

- **Root insert-if-missing is safe in toml_edit 0.22** (D4). **Proven** by the
  committed spike **CHR-019** (`tests/spike_chr019_root_tag_insert.rs`, toml_edit
  `0.22.27`, 7 tests over the **live** corpus — RV-129 F-1, premise settled FALSE):
  `as_table_mut().insert("tags", …)` lands the key **above** every trailing
  `[relationships]` / `[[relation]]` / named subtable, and the rendered doc
  **re-parses with `root.tags` set** (semantic, not textual). The original
  evidence was a throwaway `/tmp/tomlprobe` on synthetic shapes; the spike instead
  exercises the real worst-case shapes that probe never touched — SL-118
  (`[[relation]]` → named `[estimate]` → trailing comment), RFC-002 (16×
  `[[relation]]` + the only live tag set, relocated and preserved), SL-048
  (comment after the last relation), ADR-014/POL-001 (same-file root `status` +
  `[relationships].tags` overlap), spec-016 (root tags already present + `[[source]]`
  AoT). **Why it is safe:** toml_edit's encoder emits header-less root leaf
  key/values **above** all child table headers regardless of insertion order
  (structural to TOML — a key after a `[header]` would belong to that table), so a
  root insert via this API cannot tail-land inside a trailing subtable. The
  corruption premise in `backlog::apply_tags`' F-1 comment (and the identical one
  in `dep_seq::apply_status`) is therefore **stale for root inserts**. Scope of the
  change is the **tag write path only** — the status-refusal goldens
  (`set_adr_status`/requirement, which lock byte-unchanged refusal on a *missing
  status* key) are **not** touched; that paranoia is now known over-conservative
  but is harmless to keep. Insert-if-missing is thus a tag-path decision, not a
  repo-wide reversal. (RV-129 F-1's "disjoint seams" objection is conceded:
  ADR/POL carry both root `status` and `[relationships].tags` in one file — but the
  spike proves that overlap is benign.)
- No-op guard MUST compare as **sets** (not ordered vecs) so an idempotent re-add
  against an unsorted hand-authored store does not spuriously write + stamp.
- `updated` stamp is **conditional on key presence** — status-less kinds without
  `updated` are tagged without a stamp; backlog (always seeds `updated`) stamps
  exactly as before.
- Memory / non-numbered refs fail `parse_canonical_ref` → mapped to a friendly
  "use `doctrine memory tag`" / "not a numbered entity" error, never a panic.

## 6. Open Questions & Unknowns

- OQ-1: Hoisting backlog's tag **presentation** (the `Tagged …:` line) fully into
  the shared module is deferred — the verb shell prints it directly for now. If a
  second presentation surface appears, extract `tag::format_tag_line`. Tracked as
  a possible follow-up (user note), not done here.
- OQ-2: Excluded kinds (CM/RV/REC/REV) gain tags once their read surfaces are
  wired — **IMP-144**. With uniform root storage that is read-surface work per
  kind plus one `TAGGABLE` const entry; no write-path code.

## 7. Decisions, Rationale & Alternatives

- **D1 — Uniform root-level `tags` for all taggable kinds.** Alt A: keep per-kind
  locations (`[relationships].tags` for gov, root for others) with a `TagLoc`
  dispatch — canonical today, zero migration, but a permanent location split.
  Chosen uniform-root instead: collapses dispatch to a membership check and makes
  the list filter-fix fall out of `Meta` gaining one field. Cost: migrate gov/RFC
  + a governance Revision (D6). Weighed and accepted: blast radius is bounded and
  mechanical (§5.4), buys permanent uniformity.
- **D2 — Curated taggable set, read-surface-gated (NOT "tag anything").** Alt:
  taggable = any `parse_canonical_ref` hit. **Rejected (Codex MAJOR-1):** kinds
  whose `show`/`--json`/`list` do not render tags (CM/RV/REC/REV) would accept
  **write-only metadata** that silently vanishes — that is "accept hidden data",
  not "no whitelist". The gate is a `TAGGABLE` const = kinds wired to *read* tags:
  slice, gov/RFC, backlog, knowledge, spec, REQ. Excluded kinds wait on IMP-144
  (their read-surface wiring). Memory excludes itself by ref shape.
- **D3 — `set` mirrors backlog (additive `tags` + `--remove`); `clear` removes
  all.** Alt: `set` = full-replace. Rejected: divergence from backlog would block
  clean delegation and break the behaviour-preservation gate. Backlog becomes a
  pure delegate.
- **D4 — Insert-if-missing on the tag write path, not F-1 bail.** **Proven** safe
  at root incl. `[[relation]]` shapes by the committed spike CHR-019 (§5.5; RV-129
  F-1 settled FALSE). Scoped to tagging only — the
  status-refusal goldens stay. Backlog's only behaviour change: a
  malformed/hand-trimmed file self-heals instead of bailing; the one backlog test
  asserting that bail is rewritten to assert self-heal. The stale corruption
  comment on `backlog::apply_tags` is corrected.
- **D5 — Centralise the filter-fold in `listing::build`.** Alt: fold in each
  `run_list`. Rejected: three call sites + every future kind; one fold site is
  DRY and uniform. Idempotent, so backlog's removed pre-fold is behaviour-neutral.
- **D6 — SL-136 is governance-changing; the spec amendment rides a Revision at
  reconciliation.** D1 contradicts SPEC-005 D2, SPEC-018 §relations, and SPEC-016
  (Codex 2nd-pass), which pin governance tags as typed in `[relationships]` (Codex
  BLOCKER, verified in source). A slice design cannot overrule a tech spec. Per
  ADR-013 a governance dependency routes through a **Revision (REV)**; the
  reconciliation **timing** is canon under ADR-003 (tech specs are reconciled from
  observed implementation at `/reconcile`, after audit, before `/close`) — not the
  project governance note. Chosen: **revision-at-reconciliation** — the
  code/test/corpus changes land in-slice, one REV amending all **three** specs is
  authored at `/reconcile` before `/close`. Alt: revision-first (author the REV,
  then build) — heavier sequencing for no correctness gain given the bounded,
  reversible blast radius. The REV obligation is recorded here and in the scope so
  closure cannot silently skip it.
- **D7 — VA-1 stays a soft (agent-checked) gate, not a hard PHASE-04→reconcile
  coupling.** RV-129 F-6 flags that PHASE-04 lands a corpus deliberately
  contradicting SPEC-005/016/018, ratified by the REV only later at `/reconcile`,
  with VA-1 (the weakest verification mode) as the only intervening check — so a
  slice that stalls after PHASE-04 sits canon-violating. **Decision: keep soft.**
  The temporary contradiction is the *expected* intermediate state of a
  governance-changing slice (D6); the canonical change loop (ADR-003) routes
  audit → reconcile → close, and `/close` already verifies spec-coherence, so the
  window only matters under mid-flight abandonment — which the process closes
  anyway. A hard gate is ceremony against an abandonment case the lifecycle
  already handles. Alt (hard VH/VT gate coupling PHASE-04 exit to the REV):
  rejected as disproportionate; reconsider only if stalled-slice canon drift is
  observed in practice. The R5 mitigation (obligation recorded in D6 + scope +
  carried to `/reconcile`) remains the primary guard.

## 8. Risks & Mitigations

- R1 — Governance struct surgery breaks `show`/JSON/migration goldens. Mitigation:
  the inventory in §5.4 is the work-list (adr/standard/catalog goldens +
  migration-guard inversion + `adr.rs:322`); update deliberately, assert RFC-002
  round-trips root-side.
- R2 — A backlog test asserts the F-1 bail. Mitigation: known (D4) — rewrite to
  assert self-heal; grep before coding.
- R3 — `meta::Meta` gaining `tags` perturbs an unrelated Meta consumer.
  Mitigation: `#[serde(default)]` keeps every existing file parsing; fix the named
  literal sites (A2); run the full list/show suites across kinds.
- R4 — Migration misses a gov file or drops RFC-002's tags. Mitigation: scripted
  grep for residual `[relationships].tags` **scoped to `install/templates` + the
  committed corpus** (not the gitignored derived `.doctrine/templates/`, RV-129
  F-3), **paired** with the golden/compile checks (grep is blind to serde/loop-var
  consumers); explicit RFC-002 restore + assertion; `governance_files()` extended
  to scan `rfc`.
- R5 — Closure forgets the REV (D6), leaving the corpus non-canonical against
  SPEC-005/018. Mitigation: obligation recorded in design D6 + scope Phase 4 +
  carried to `/reconcile`; `/close` verifies spec-coherence.
- R6 — REQ read-surface wiring is larger than assumed. **Resolved (verified):** REQ
  rides a bespoke spec-mediated path, not `Meta`, but its `req_key()` already wires
  `tags` (`spec.rs:1665`) and the `Requirement` struct already carries a
  `#[serde(default)] tags` field (`requirement.rs:168`). Remaining work — `tags` on
  `ReqJsonRow` + `show_json` member object + REQ show row — is mechanical and stays
  in scope; REQ stays in `TAGGABLE`.
- R7 — full read-surface parity (D2/§5.3) touches more render sites than the
  first-pass worklist (slice/spec show+json, gov show-json). Mitigation: the §5.3
  per-kind list + §9 parity tests are the work-list; one show+json test per kind
  asserts no surface is write-only.

## 9. Quality Engineering & Validation

Per §6 of the scope and §5.5 invariants:

- `tag.rs` units: insert-if-missing seeds; no-op guard on **unsorted** store;
  set algebra sorted union/diff; `updated` stamped-if-present / skipped-if-absent;
  clear-on-untagged = no-op; `fold_filter_tag` lenient; **regression: root insert
  lands above a trailing `[relationships]`** (the CHR-019 spike,
  `tests/spike_chr019_root_tag_insert.rs`, is the committed seed — kept as VT-2).
- `commands/tag.rs`: set/clear round-trip on real scaffolds across taggable kinds
  (slice/adr/knowledge/spec/REQ); overlap reject; **excluded kind (e.g. `CM-1`)
  refused with the IMP-144 pointer**; memory-ref friendly error.
- `meta`: tags default absent→empty, present→read. **A2: update the `Meta`
  struct-literal sites** (`adr.rs`, `policy.rs`, `governance.rs` ~L1150,
  `meta.rs` test helper, `backlog.rs` ~L2411) with `tags: vec![]` — literals
  need the field even with serde-default.
- list: slice + spec (PRD/SPEC) + governance + REQ `list --tag` match
  case-insensitively (spec::key was the gap).
- **Full read-surface parity (per kind):** for slice, spec, REQ, gov/RFC a written
  tag is visible on **all three** surfaces — `list --tag` filter, `show` table row,
  and `--json` field. Specifically: `ReqJsonRow` + `show_json` member object carry
  `tags`; slice/spec/gov show-json carry `tags`. No surface left write-only.
- migration (storage residue only — **not** the read-parity proof, RV-129 F-2/F-3):
  the residual-`[relationships].tags` grep is **scoped to `install/templates` + the
  committed corpus** (`.doctrine/{adr,policy,standard,rfc}/**`), **never the
  gitignored, derived `.doctrine/templates/`** — that tree is regenerated from the
  `install/` RustEmbed (`src/install.rs:17`), so greppng it both over-fires (flags
  derived copies) and proves nothing. Grep alone **cannot** gate this migration
  (F-3): it is blind to serde-driven (`to_value(doc)`) and loop-var consumers, so
  the gate pairs the grep with the **golden/compile-level checks** — the rewritten
  adr/standard JSON goldens (now expecting root tags), `governance_files()`
  extended to scan `rfc`, and the CHR-019 re-parse spike. RFC-002 restored
  (asserted); governance `show` renders tags from root; the migration-guard test
  inverted; `relation_graph.rs` ADR fixtures (`:1300`,`:1662-1668`) repointed
  root-ward and that suite green.
- read-parity (RV-129 F-2 — proven by **semantic per-kind enumeration**, §5.3, not
  by grep): for slice/spec/REQ/gov each of the three surfaces — `list --tag`,
  `show`, `--json` — is wired and **asserted by a dedicated test**; the dangerous
  consumers are the grep-invisible ones (governance `to_value(doc)` JSON; the
  REQ-JSON drop sites), which the §5.3 enumeration names explicitly rather than
  trusting a textual scan to surface.
- Behaviour-preservation: backlog tag + gov/slice list suites green; the bail
  test rewritten.

Phasing:

1. **Shared leaf** — `apply_tags_set` + `fold_filter_tag` hoist into `tag.rs`;
   `TAGGABLE` const; backlog `apply_tags`/`run_tag`/list-filter delegate.
   Behaviour-preserving.
2. **Generic verb** — `commands/tag.rs` (`set`/`clear`) + `Command::Tag` wiring;
   taggability gate (excluded kinds → IMP-144 error).
3. **Templates + Meta + full read surface** — seed slice/REQ templates; `Meta.tags`;
   `slice::key()`/`governance::key()`/`spec::key()` + REQ list wiring; **show + json
   tag render for slice/spec/REQ/gov** (full parity, §5.3); centralise fold in
   `listing::build`.
4. **Governance/RFC migration** — root-ward move in files + 4 templates + struct
   surgery + golden/migration-guard updates (incl. `governance_files()` += `rfc`,
   `relation_graph.rs` fixtures); RFC-002 restore. **REV obligation (D6) — amends
   SPEC-005/016/018 — flagged for `/reconcile`.**

## 10. Review Notes

Internal adversarial pass (integrated above):

- **A1 — existence guard.** `parse_canonical_ref` parses without a disk probe, so
  `tag set SL-999` would surface a raw read error. Verb shell guards with
  `integrity::ensure_ref_resolves`. → §5.4.
- **A2 — `Meta` literal sites.** Adding `tags` to `meta::Meta` breaks 5
  struct-literal constructions (serde-default covers parsing, not literals).
  Enumerated → §9.
- **A3 — governance `show`.** Renders `doc.relationships.tags` today; migration
  repoints to root `doc.tags` and drops the Relationships render. → §5.4.
- **A4 — migration churn.** Existing gov/RFC files need only the stale
  `[relationships].tags` stripped; root tags read as default-empty. No bulk
  root-seed. → §5.4.
- **A5 — empty `set` guard.** `set` with neither add nor remove errors (mirrors
  backlog). → §5.4.
- **A6 — prefix uniqueness.** All 21 `integrity::KINDS` prefixes are unique
  (`kind_by_prefix` first-match is unambiguous); only the `TK` test-kind dupes,
  outside KINDS. No action.

No internal finding overturned a §7 decision; all are execute-time mechanics.

External adversarial pass — Codex (GPT-5.5), all claims verified in source:

- **BLOCKER-1 — D1 breaks live contracts.** Governance `show`/`--json` read
  `relationships.tags`; goldens pin it (`e2e_adr_cli_golden`,
  `e2e_standard_cli_golden`), and `e2e_relation_migration_storage:428` guards it.
  **Accepted** — full inventory + rewrites folded into §5.4/§8 R1; the storage
  move is now explicitly governance-changing (D6).
- **BLOCKER-2 — D4 unproven / contradicts repo paranoia.** **Partially upheld.**
  The safety claim is now proven (standalone probe incl. `[[relation]]` + semantic
  re-parse, §5.5) and scoped to the tag path only — the status-refusal goldens are
  untouched, so no repo-wide reversal. Documented, not hand-waved.
- **MAJOR-1 — D2 creates write-only metadata** on CM/RV/REC/REV. **Accepted —
  decision reversed.** D2 is now a curated, read-surface-gated `TAGGABLE` set;
  excluded kinds deferred to IMP-144.
- **MAJOR-2 — design overrules canon** (SPEC-005 + SPEC-018 pin typed tags).
  **Accepted.** Resolved via D6: revision-at-reconciliation (ADR-013), REV
  amending both specs, obligation recorded (R5).

Net: Codex reversed D2 outright and forced D1 to declare itself governance-changing
(D6) with a measured blast radius. D3/D4/D5 stand (D4 with proof + scoping).

Second external pass — Codex (GPT-5.5), all claims verified in source:

- **BLOCKER — partial read surface = quiet write-only.** The curated set claims
  "read surfaces render tags," but the first-pass worklist only wired the
  list-filter (and not for spec), leaving slice/spec/REQ tags unrendered in
  `show`/`--json`. **Accepted — full-parity rule adopted (D2/§5.3):** a kind is
  taggable only with all three surfaces (list-filter + show + json) wired; §5.3
  enumerates the per-kind render work, §9 adds parity tests.
- **MAJOR — `spec::key()` omitted from list-fix.** `src/spec.rs:~1349` hardcodes
  `tags: Vec::new()`, so `spec list --tag` stayed dead. **Accepted** — `spec::key()`
  added to the §5.4 list-fix worklist.
- **MAJOR — REQ JSON write-only.** `req_key` wires list tags but `ReqJsonRow`
  (`spec.rs:~1556`) and `show_json`'s member object (`spec.rs:~1167`) carry none.
  **Accepted** — both folded into §5.3/§9.
- **MAJOR — migration inventory incomplete (3 misses).** (a) `SPEC-016` also pins
  typed gov tags (`spec-016.toml:17`) → D6 REV now amends **three** specs; (b)
  `governance_files()` (`e2e_relation_migration_storage.rs:86`) scans only
  adr/policy/standard, omitting RFC → extended to `rfc`; (c) `relation_graph.rs`
  fixtures (`:1300`,`:1662-1668`) bake typed gov tags → repoint root-ward. All
  folded into §5.4.
- **Sequencing authority correction.** D6's revision-at-reconciliation **timing**
  is canon under **ADR-003** (specs reconciled from implementation at `/reconcile`),
  not the project governance note; ADR-013 still governs the REV routing. D6 fixed.
- **Confirmed closed:** D4 scoping holds (tag-write `backlog.rs:1936` and
  status-write `dep_seq.rs:282` are disjoint seams); exclusions correct (CM/RV/REC/
  REV render no tags on any surface).

Net second pass: no decision overturned, but the worklist was materially
incomplete — full read-surface parity (D2/§5.3), `spec::key()` + REQ-JSON wiring,
and three migration-inventory additions.

Third (focused) pass — Codex (GPT-5.5), confirmed in source:

- **Parity worklist complete.** No included kind has an unwired read surface
  outside the §5.3/§5.4 worklist; backlog/knowledge already render on all three
  (`backlog.rs:1041`, `knowledge.rs:971/1099/1258`).
- **MINOR ×2 — two more typed-tag unit tests** to migrate: `policy.rs:295` +
  `standard.rs:302` (`["superseded_by","tags"]` axis loop). Folded into §5.4.
- **Reviewed, not migration sites:** `relation.rs:1559` (synthetic
  trailing-typed-table refusal trap — `tags` incidental, not governance storage)
  and `relation_graph.rs:852` (asserts tags are not relation *edges* — stays true
  post-move). No change.

Net third pass: two mechanical test-site additions, no new decisions, parity
confirmed complete.

Fourth pass — RV-129 (adversarial, post-lock reopen), all 6 findings disposed in
source; the contradiction in F-1 was settled by a committed spike, not assertion:

- **F-1 (blocker) — D4 root-insert premise unproven; status/tags contradiction.**
  **Settled — premise FALSE.** Spike **CHR-019** (`tests/spike_chr019_root_tag_insert.rs`,
  toml_edit `0.22.27`, 7 tests over the live corpus) proves root insert lands above
  every trailing subtable/`[[relation]]` and re-parses at root, across the real
  worst-case shapes the `/tmp` probe never tested (SL-118, RFC-002, SL-048,
  ADR-014/POL-001, spec-016). toml_edit emits header-less root leaf key/values
  above all child headers regardless of insert order, so the both-seam refusal
  (`apply_tags` + `apply_status`) is stale for root inserts. D4 stands, now proven;
  the status-path bail is over-conservative but harmless (tag-path-only scope). The
  "disjoint seams" objection is conceded but proven benign (ADR/POL same-file
  overlap round-trips). → §5.5, D4.
- **F-2 (major) — grep-as-parity-proof falsified; serde + loop-var consumers.**
  **Accepted.** Read-parity is proven by the **semantic per-kind enumeration**
  (§5.3), not grep; grep gates storage residue only. The serde-driven
  `governance.rs:360` `to_value(doc)` JSON renders tags at root for free post-move
  (**decision: root-expose** — uniform with all kinds, goldens updated; the
  supersedes/related splice is NOT extended to tags). → §5.3/§5.4/§9.
- **F-3 (major) — VT-1 grep gate under- and over-fires.** **Accepted.** The
  migration grep is scoped to `install/templates` + committed corpus (never the
  gitignored derived `.doctrine/templates/`) **and paired with golden/compile
  checks** — grep alone cannot gate (blind to serde/loop-var reads). → §5.4/§8 R4/§9.
- **F-4 (major) — positive shape-pin tests assert OLD shape; RFC unguarded.**
  **Accepted — already integrated by the 2nd/3rd passes, confirmed complete:**
  adr/standard JSON goldens (root nesting), `policy.rs:295`/`standard.rs:302` axis
  loops, `assert_governance_shape` inversion, `governance_files()` += `rfc`,
  RFC-002 live-tag restore + assertion (§5.4). `governance.rs:1063` show-row
  (`"tags: lang"`) is a presentation positive-coverage test that stays green once
  `show` reads root — reviewed, no change.
- **F-5 (major) — REQ-JSON is an additive parity gap.** **Accepted; decision:
  in scope.** Both REQ JSON sites (`spec.rs:~1167` member object, `spec.rs:~1556`
  `ReqJsonRow`) drop tags today — nothing to preserve, a gap to fill. The slice
  *adds* tags to both, and VT-2(P3) asserts tag **presence** (not "unchanged").
  → §5.3/§5.4/§9.
- **F-6 (minor) — VA-1 soft gate over a canon-violation window.** **Accepted;
  decision: keep soft** (D7). The contradiction is the expected intermediate state
  of a governance-changing slice; ADR-003's audit→reconcile→close loop and
  `/close`'s spec-coherence check already close the window outside mid-flight
  abandonment. A hard gate is disproportionate. → D7.

Net fourth pass: no decision overturned; F-1 promoted from "claimed safe" to
"proven safe" by a committed spike (CHR-019, kept as VT-2 seed), two genuinely new
holes closed (F-3 grep scoping, F-6 gate posture), and the parity-proof method
sharpened from grep to semantic enumeration (F-2). Design coherent and converged —
ready for `/plan`.
