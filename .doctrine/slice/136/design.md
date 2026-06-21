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

Every taggable entity stores a **root-level `tags: Vec<String>`**. Taggable =
*any resolvable canonical numbered ref* (`parse_canonical_ref` succeeds) — all 21
numbered kinds, no whitelist to maintain. Memory is excluded for free (its
`mem.*` ref fails `parse_canonical_ref`).

```
doctrine tag set <ID> <TAGS...> [-d/--remove <TAGS>...]   # additive-merge
doctrine tag clear <ID>                                    # remove all
```

Resolution (the whole dispatch — no per-kind location logic):

```rust
let (kref, id) = integrity::parse_canonical_ref(reference)?;   // SL-136 → (slice, 136)
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
- Template seeding (`tags = []` at root): add to **slice, requirement (REQ),
  concept-map (CM)**; backlog/knowledge/spec already seeded; gov/RFC seeded via
  the §5.4 migration. Low-traffic status-less kinds (review/RV, REC, REV) not
  seeded — self-heal on first tag.

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
- `slice::key()` + `governance::key()` set `tags: m.tags.clone()` (governance
  covers ADR/POL/STD and RFC, which routes through `governance::run_list`).
- **Centralise the filter-fold into `listing::build`** (fold each `--tag` input
  trim+lowercase via `tag::fold_filter_tag`); remove backlog's redundant
  pre-fold. Idempotent → every list kind gets case-insensitive `--tag` uniformly.

Governance/RFC migration (one commit):
- **Files (~29):** strip the `tags` line from `[relationships]`. Existing files
  need **no** root `tags=[]` seeded — `Meta`/`Doc` read root tags with
  `#[serde(default)]`, so absent = empty (A4, less churn). RFC-002's live tags
  (`program, consumption-surfaces, estimate, value, scoring`) are restored by one
  `doctrine tag set RFC-002 …` after the verb lands (re-seeds root).
- **Templates (4):** adr/policy/standard/rfc — remove `tags` from `[relationships]`,
  add root `tags = []`.
- **Struct surgery:** drop the typed `tags` from governance's `Relationships`;
  add root `tags` to its `Doc`; **repoint the `show` render** from
  `doc.relationships.tags` to `doc.tags` (A3, `governance.rs` ~L313-320).

### 5.5 Invariants, Assumptions & Edge Cases

- **Root insert-if-missing is safe in toml_edit 0.22** (D4). Probe: parse a
  slice-shaped doc, `as_table_mut().insert("tags", …)`, render — the key lands
  **above** the trailing `[relationships]` in every shape tested (no blank line /
  inline comments / multi-subtable / leading doc-comment). The corruption premise
  in `backlog::apply_tags`' F-1 comment and in
  `mem.pattern.entity.edit-preserving-status-transition` is stale for root
  inserts; both get corrected.
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
- OQ-2: REQ/CM templates seeded now; review/REC/REV left to self-heal. If those
  kinds turn out to want visible seeded fields, a one-line template add each — no
  code change (uniform).

## 7. Decisions, Rationale & Alternatives

- **D1 — Uniform root-level `tags` for all kinds.** Alt: keep per-kind locations
  (`[relationships].tags` for gov, root for others) with a `TagLoc` dispatch.
  Rejected: a location enum + per-kind table is exactly the special-casing this
  slice removes; uniform root collapses dispatch to nothing and makes the list
  filter-fix fall out of `Meta` gaining one field. Cost: migrate gov/RFC (§5.4).
- **D2 — No taggability whitelist.** Taggable = any `parse_canonical_ref` hit.
  Alt: curated whitelist. Rejected: uniform storage makes every numbered kind
  free to tag; a whitelist is maintenance with no payoff. Memory excludes itself
  by ref shape.
- **D3 — `set` mirrors backlog (additive `tags` + `--remove`); `clear` removes
  all.** Alt: `set` = full-replace. Rejected: divergence from backlog would block
  clean delegation and break the behaviour-preservation gate. Backlog becomes a
  pure delegate.
- **D4 — Insert-if-missing, not F-1 bail.** Empirically safe at root (§5.5).
  Backlog's only behaviour change: a malformed/hand-trimmed file self-heals
  instead of bailing. The one backlog test asserting the bail is rewritten to
  assert self-heal (gate stays meaningful). The stale corruption comment + memory
  are corrected.
- **D5 — Centralise the filter-fold in `listing::build`.** Alt: fold in each
  `run_list`. Rejected: three call sites + every future kind; one fold site is
  DRY and uniform. Idempotent, so backlog's removed pre-fold is behaviour-neutral.

## 8. Risks & Mitigations

- R1 — Governance struct surgery (drop `Relationships.tags`, add root `tags` to
  `Doc`) breaks a `show`/JSON golden. Mitigation: governance `show`/list suites
  are the gate; update goldens deliberately, assert RFC-002 round-trips.
- R2 — A backlog test asserts the F-1 bail. Mitigation: known (D4) — rewrite to
  assert self-heal; grep before coding.
- R3 — `meta::Meta` gaining `tags` perturbs an unrelated Meta consumer.
  Mitigation: `#[serde(default)]` keeps every existing file parsing; run the full
  list/show suites across kinds.
- R4 — Migration misses a gov file or drops RFC-002's tags. Mitigation: scripted
  grep for residual `[relationships].tags`; explicit RFC-002 restore + assertion.

## 9. Quality Engineering & Validation

Per §6 of the scope and §5.5 invariants:

- `tag.rs` units: insert-if-missing seeds; no-op guard on **unsorted** store;
  set algebra sorted union/diff; `updated` stamped-if-present / skipped-if-absent;
  clear-on-untagged = no-op; `fold_filter_tag` lenient; **regression: root insert
  lands above a trailing `[relationships]`** (locks the probe).
- `commands/tag.rs`: set/clear round-trip on real scaffolds across kinds
  (slice/adr/knowledge/spec/REQ/CM); overlap reject; memory-ref friendly error.
- `meta`: tags default absent→empty, present→read. **A2: update the 5 `Meta`
  struct-literal sites** (`adr.rs`, `policy.rs`, `governance.rs` ~L1150,
  `meta.rs` test helper, `backlog.rs` ~L2411) with `tags: vec![]` — literals
  need the field even with serde-default.
- list: slice + governance `list --tag` match case-insensitively.
- migration: no residual `[relationships].tags`; RFC-002 restored (asserted);
  governance `show` renders tags from root.
- Behaviour-preservation: backlog tag + gov/slice list suites green; the bail
  test rewritten.

Phasing:

1. **Shared leaf** — `apply_tags_set` + `fold_filter_tag` hoist into `tag.rs`;
   backlog `apply_tags`/`run_tag`/list-filter delegate. Behaviour-preserving.
2. **Generic verb** — `commands/tag.rs` (`set`/`clear`) + `Command::Tag` wiring.
3. **Templates + Meta + list fix** — seed slice/REQ/CM templates; `Meta.tags`;
   `slice::key()`/`governance::key()`; centralise fold in `listing::build`.
4. **Governance/RFC migration** — root-ward move in files + 4 templates + struct
   surgery; RFC-002 restore.

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

No finding overturned a §7 decision; all are execute-time mechanics.
