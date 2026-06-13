# Design SL-058: Finish the relation surface: fix stale scaffold templates, migrate their entity fallout, add agent guidance

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

SL-048 ("the cut") migrated tier-1 relations to the uniform `[[relation]]` idiom
and shipped read (`read_block`/`tier1_edges`, `inspect`, `slice show`) and write
(`link`/`unlink`) end to end. It did **not** update the scaffold templates. Every
entity scaffolded since is born with the migrated tier-1 axes in the legacy typed
`[relationships]` table — malformed against the very invariant SL-048 asserts.

The harm is already realised: ADR-011 (from the stale `adr.toml`) tripped
`e2e_relation_migration_storage` and blocked a binary update (repaired in
`138038c`); 10 backlog items carry stale `slices`/`specs`/`drift` keys (latent —
the test's parser misses their inline-comment header). The fix must (a) stop new
malformation at the source (templates), (b) clean the existing fallout
(entities), (c) close the detection gap that let the fallout hide (test), and
(d) give agents the guidance that would have prevented hand-authoring drift
(IMP-049).

This is **conformance cleanup, not redesign**. The relation model, vocabulary,
storage shape, and machinery are fixed (ADR-004, ADR-010, SPEC-018) and correct.

## 2. Current State

- **Templates** (`install/templates/`, embedded via RustEmbed, read through
  `crate::install::asset_text`):
  - `slice.toml` — emits a `[relationships]` table with a reserved comment +
    illustrative `specs`/`requirements`/`supersedes`. For slices the *whole*
    typed table migrated away (test: "slice: the whole table is gone").
  - `adr.toml` + `policy.toml` + `standard.toml` — all three governance
    templates emit `supersedes`/`superseded_by`/`related`/`tags` (policy/standard:
    `related` confirmed at line 16). Only `related` migrated (→ `[[relation]]`);
    the supersession pair + `tags` stay typed (OD-3). The governance migration
    test (`e2e_relation_migration_storage`) scans **adr + policy + standard**, so
    all three templates are equally stale even though only ADR-011 has so far
    been born malformed.
  - `backlog.toml` + `backlog-risk.toml` — emit
    `slices`/`specs`/`drift`/`needs`/`after`/`triggers` under `[relationships]`.
    `slices`/`specs`/`drift` migrated (→ `[[relation]]`); `needs`/`after`/
    `triggers` (dep/seq/trigger axes) stay typed.
- **Entity fallout** — `ADR-011` (fixed). Backlog: `ISS-009`, `ISS-010`,
  `IMP-045..051`, `IDE-005` (10 files) carry typed migrated keys. Of these only
  `IMP-045` has a *populated* migrated key (`slices = ["SL-056"]`); the rest are
  empty. `SL-056` (slice kind, F-E) additionally carries a stale comment-only
  `[relationships]` table — benign (no edges) but non-conformant with D2; in scope.
- **Detection gap** — `tests/e2e_relation_migration_storage.rs::view()`
  attributes `[relationships]` keys only when `line == "[relationships]"`
  (exact). A header with a trailing inline comment (`[relationships]  # …`, the
  backlog template's shape) is never entered, so its migrated keys are invisible
  to `assert_no_migrated_key_left`. That is why the 10 backlog items are latent
  rather than red.
- **Guidance** — none. No skill, memory, or doc tells an agent how/when to relate
  structurally, the legal vocabulary, or the `link` verb.

## 3. Forces & Constraints

- **ADR-004 / ADR-010 / SPEC-018** — outbound-only; `RELATION_RULES` is the single
  source of truth; tier-1 = `[[relation]]`, kept-typed axes stay typed. Templates
  and migrated entities must match the table, never transcribe it.
- **Behaviour-preservation gate** — the SL-048 / SL-046 / relation / cordage
  suites are the proof the machinery is unchanged; they stay green untouched.
- **RustEmbed footgun** (`mem.pattern.embed.rustembed-recompile-and-symlinks`,
  `mem.pattern.build.rust-embed-no-rerun`) — a lone template edit is invisible
  until the embedding crate recompiles. The verification must defeat a false
  green.
- **Storage rule** — templates and entities are authored tier; edit-preserving on
  populated TOML (`mem.pattern.entity.edit-preserving-status-transition`).
- **Dogfood** — the shipped `link` verb is the sanctioned writer; prefer it over
  hand-authoring `[[relation]]` rows where an edge must be created.
- **Skills source-of-truth** — `plugins/`, not the gitignored installed copy
  (`mem.pattern.distribution.skills-source-vs-installed`); skill content refresh
  needs reinstall + re-embed (`mem.pattern.distribution.skill-refresh-command`).

## 4. Guiding Principles

- Fix the source (templates) before the symptoms (entities); add the guard before
  declaring done.
- Point at `RELATION_RULES`, never restate the vocabulary.
- Edge-preserving: a populated migrated key is data — convert it, never drop it.
- The regression guard lives at the truest root reachable: a template-level
  assertion catches the next stale template before any entity is born from it.

## 5. Proposed Design

### 5.1 System Model

Four work streams, ordered by dependency:

```
templates ──(re-embed)──► scaffold-output guard (test)
    │
    └─► entity migration (link IMP-045; strip 9 empty backlog + SL-056) ──► corpus invariant green
                                                              ▲
parser hardening (view inline-comment) ──────────────────────┘
guidance (memory + using-doctrine.md + authoring skills) ── independent
```

### 5.2 Interfaces & Contracts

No new code interfaces. Surfaces touched:

- **Templates** (data): post-cut shapes — six files:
  - `slice.toml`: remove the `[relationships]` table entirely; replace with a
    comment documenting the `[[relation]]` idiom + `doctrine link` (no typed
    axes remain for slices).
  - `adr.toml` + `policy.toml` + `standard.toml`: drop the `related` line; keep
    `supersedes`/`superseded_by`/`tags` typed; add a `[[relation]]` guidance
    comment. (Identical edit, three files — governance shares the `related`
    migration.)
  - `backlog.toml` + `backlog-risk.toml`: drop `slices`/`specs`/`drift`; keep
    `needs`/`after`/`triggers` typed; add a `[[relation]]` guidance comment.
- **`link`/`unlink`** (existing verb, dogfooded): `doctrine link IMP-045 slices
  SL-056` authors IMP-045's row before its typed `slices` key is stripped.
- **`view()` parser** (test helper): enter `In::Relationships` when the trimmed
  line, with any trailing `#…` comment stripped, equals `[relationships]` —
  i.e. `line.split('#').next().trim() == "[relationships]"`, NOT a loose
  `starts_with` (which would false-match a sub-table `[relationships.x]`). Same
  treatment for `[[relation]]`. Preserves the textual-position F1 semantics (the
  header index is still the line number). Key extraction additionally strips
  surrounding quotes (`"slices"` → `slices`) so a legal quoted migrated key cannot
  evade `assert_no_migrated_key_left` (F-H).
- **Guidance**: a doctrine memory (pattern), a `using-doctrine.md` section, and
  targeted insertions in the authoring skills.

### 5.3 Data, State & Ownership

- **Templates** own the born shape of every future entity — the single write
  point. Owned by `install/templates/*`, embedded at compile.
- **Entity files** own their authored relations. Migration rule: for each of the
  10 backlog files, (i) if a migrated key is populated, author the equivalent
  `[[relation]]` row via `link`; (ii) remove the three migrated keys
  (`slices`/`specs`/`drift`) from the typed `[relationships]` table, leaving
  `needs`/`after`/`triggers`. Empty migrated keys carry no data — pure removal.
  IMP-045 is the only (i) case. **SL-056** (slice kind, F-E) carries a stale
  comment-only `[relationships]` table from the old slice template — strip the
  whole table (no edge, pure removal), matching D2's table-absent slice shape. The
  execution re-scan covers the slice corpus too, not only backlog.
- **`needs`/`after`/`triggers`** stay under `[relationships]` typed — NOT
  migrated, not touched.

### 5.4 Lifecycle, Operations & Dynamics

Execution order (one phase boundary at re-embed):

1. Edit the six templates → **touch the embedding crate** → rebuild → assert
   `slice new`/`adr new`/`policy new`/`standard new`/`backlog new` (+ risk-backlog)
   emit the new shape (defeats RustEmbed false green).
2. Harden `view()`; add the template-level guard test — a TEXT scan (reusing the
   hardened `view()`) over each embedded template asset, **kind-specific** (F-D):
   for the slice kind, assert NO `[relationships]` header at all (the bare-key scan
   alone passes slice.toml, whose migrated axes are only commented); for
   governance/backlog, assert migrated tier-1 keys absent AND the kept-typed keys
   present AND the `[[relation]]` guidance comment present. NB raw templates carry
   `{{slug}}`/`{{id}}` placeholders and are NOT valid TOML, so the guard must
   text-scan, never `toml::from_str` (the black-box scaffold test in §9 covers the
   rendered shape).
3. Migrate the 10 entities (link IMP-045; strip all 10).
4. Author guidance: memory + `using-doctrine.md` + authoring-skill insertions.

Step 2's hardened test, run against the corpus after step 3, is the closure
proof: previously-latent items are now both well-formed AND covered.

### 5.5 Invariants, Assumptions & Edge Cases

- INV: no embedded template emits a migrated tier-1 key in `[relationships]`
  (new guard); the **slice** template emits no `[relationships]` header at all
  (kind-specific guard, F-D).
- INV: corpus carries no migrated tier-1 key in a typed `[relationships]` slot
  (existing test, now with a parser that actually sees inline-comment headers);
  the slice corpus invariant additionally asserts no `[relationships]` header for
  the slice kind, and the `name == "056"` hardcode skip is removed (F-E).
- INV: `RELATION_RULES` unchanged; machinery suites green unchanged.
- Edge: a populated migrated key (IMP-045) must round-trip to a `[[relation]]`
  row before strip — verify the edge renders in `inspect`/`backlog show` after.
- Edge: `slice.toml` losing its whole `[relationships]` table — confirm
  `slice show` / `slice new` parse a slice with no typed relationships table
  (read-tolerant; SL-058 itself already proves this).
- Edge: concurrent authoring keeps minting backlog items from the stale template
  until the fix lands — re-scan for new fallout at execution start (the list grew
  045→051 mid-design).

## 6. Open Questions & Unknowns

- OQ-1: should the template-guard test live in
  `e2e_relation_migration_storage.rs` (corpus-invariant peer) or a new
  `templates`-focused test module? (Lean: same file — it is the migration
  invariant, one source axis up.)
- OQ-2: exact insertion points in the authoring skills — resolved at execution by
  reading `/slice`, `/design`, `/plan` and peers; the design commits to *which*
  skills (§7 D4), not the prose.

## 7. Decisions, Rationale & Alternatives

- **D1 — Entity migration via `link` + strip, not a one-shot migrator.** Only one
  edge exists (IMP-045); a bespoke migrator (as SL-048 used for the bulk cut) is
  overkill for 11 files and 1 edge. Dogfooding `link` also exercises the shipped
  writer on real data. Alt: pure hand-edit — rejected for the populated key (loses
  the dogfood signal and risks a malformed hand row). **Cutover rule (F-G):**
  link+strip holds while populated migrated keys ≤ 1 AND every other fallout file
  is an empty-key removal AND the total malformed set ≤ ~12. If the execution
  re-scan finds >1 populated migrated key, OR any populated *non-backlog* fallout,
  OR a materially larger malformed set → switch to a scan-driven migrator (or
  rescope) before continuing.
- **D2 — Slice template drops the whole `[relationships]` table.** Slices have no
  kept-typed axis; an empty stub would re-tempt typed authoring and re-introduce
  the F1 trailing-table hazard. A comment documents the `[[relation]]` idiom. Alt:
  empty stub — rejected.
- **D3 — Harden the parser AND add a template-level guard.** (User decision.) The
  parser fix makes the corpus invariant honest; the template guard catches the
  root (a stale template) before any entity is born. Defence in depth at both the
  source and the corpus. Alt: parser-only — rejected (leaves the root unguarded);
  separate item — rejected (the gap is in scope and cheap).
- **D4 — Guidance lands in memory + `using-doctrine.md` + the authoring skills.**
  (User decision: option 1 + evaluate `/slice`, `/design`, `/plan`, peer
  authoring skills.) Memory is the durable agent-recall surface; `using-doctrine.md`
  is the shipped reference; the authoring skills are where relate-intent actually
  arises (scoping a slice's `specs`/`governed_by`, an ADR's `related`). No new
  skill — the intent is a step within existing authoring flows, not a standalone
  verb. Alt: new `/relate` skill — rejected (over-weights a one-line action;
  would split the slice).

## 8. Risks & Mitigations

- **R1 — RustEmbed false green** (edit invisible until recompile). Mitigation:
  touch the embedding crate; assert scaffold *output*, not template file contents.
- **R2 — Dropping a populated key loses an edge.** Mitigation: D3 edge-preserving
  rule; re-scan all 10 for populated keys at execution (only IMP-045 today);
  verify the migrated edge renders post-strip.
- **R3 — Concurrent authoring adds new fallout mid-flight.** Mitigation: re-scan
  the corpus for stale typed keys immediately before the entity-migration phase;
  the template fix stops the inflow.
- **R4 — Skill edits drift from installed copies.** Mitigation: edit `plugins/`
  source, reinstall + re-embed per `mem.pattern.distribution.skill-refresh-command`.
- **R5 — Over-reach into machinery.** Mitigation: behaviour-preservation gate —
  no edits to `read_block`/`tier1_edges`/`link`/`format_show`.

## 9. Quality Engineering & Validation

- **Scaffold-output black-box test** (`mem.pattern.testing.black-box-cli-golden`):
  `slice new`/`adr new`/`policy new`/`standard new`/`backlog new` + the risk-backlog
  scaffold path (all six fixed templates, F-F) produce no migrated typed key and
  carry the `[[relation]]` guidance comment; `slice new` produces no
  `[relationships]` header at all.
- **Template-guard unit test**: a TEXT scan (hardened `view()`) over each embedded
  template asset, kind-specific (F-D) — slice: no `[relationships]` header;
  governance/backlog: migrated tier-1 keys absent, kept-typed keys present,
  guidance comment present. The root guard, independent of the CLI. Text-scan, not
  `toml::from_str` (placeholders are invalid TOML).
- **Hardened corpus invariant**: `e2e_relation_migration_storage` passes over the
  full corpus including the now-migrated backlog items AND a stripped SL-056, with
  the inline-comment header visible to `view()` and the `name == "056"` hardcode
  removed (F-E); the slice corpus check asserts no `[relationships]` header for
  slices.
- **Round-trip fixture**: scaffold an entity, `link` a relation, assert it renders
  in `slice show`/`inspect`; `unlink` removes it; `validate` clean.
- **Behaviour-preservation**: SL-048 / SL-046 / relation / cordage suites green
  unchanged. `just gate` clean; clippy zero warnings.

## 10. Review Notes

Internal adversarial pass (integrated):

- **F-A — scope hole (governance breadth).** Original draft scoped three
  templates; `policy.toml` + `standard.toml` also emit stale `related = []` and
  are scanned by the same governance migration test. Corrected to **six**
  templates (slice, adr, policy, standard, backlog, backlog-risk); adr/policy/
  standard take the identical `related`-drop edit. No current policy/standard
  entity is malformed (grep clean) — template fix is preventive there.
- **F-B — guard mechanism unbuildable as drafted.** "Parse each template" fails:
  raw templates carry `{{slug}}`/`{{id}}` placeholders (invalid TOML). Re-specified
  the template guard as a text scan over embedded assets via the hardened `view()`.
- **F-C — parser-hardening precision.** Loose `starts_with("[relationships]")`
  would false-match `[relationships.x]`; specified comment-stripped exact match.

External adversarial pass (codex / GPT-5.5, integrated). Each finding
re-derived against ground truth (`RELATION_RULES` @ `src/relation.rs:252`,
`tests/e2e_relation_migration_storage.rs`, the templates) before disposition.

- **F-D — template guard is toothless for the slice kind. (MAJOR; ACCEPT.)**
  `slice.toml` emits its migrated axes only as *commented* examples under a
  `[relationships]` header (lines 8-14) — no bare keys. The §5.2/§9 guard ("zero
  migrated tier-1 keys" via the bare-key `view()` scan) therefore PASSES the
  current stale slice template, because `view()` skips `#` lines. The guard does
  not enforce D2's whole-table removal. Fix: the template guard is **kind-specific**
  — for the slice kind it asserts NO `[relationships]` header at all; for
  governance/backlog it asserts migrated keys absent AND the kept-typed keys
  present AND the `[[relation]]` guidance comment present. (adr/backlog templates
  emit real bare keys, so the bare-key scan already catches *those* kinds — F-D is
  slice-only.) §5.2, §5.5, §9 updated.
- **F-E — slice-kind conformance is asserted nowhere; SL-056 + a test hardcode.
  (MAJOR; ACCEPT.)** The slice corpus invariant (`e2e_…::slice_corpus_…`, line
  146) checks only bare migrated keys + F1 ordering — it does NOT assert
  table-absence, so a slice born with a comment-only `[relationships]` table passes
  vacuously. SL-056 carries exactly that stale table and is *additionally*
  hardcoded-skipped (`if name == "056" continue`, line 154). The entity is benign
  (no edges, render-identical), but (a) D2 makes the correct slice shape
  table-absent, and (b) a hardcoded entity skip contradicts SL-058's "close the
  detection gap" mandate. Fix: the slice corpus invariant asserts NO
  `[relationships]` header for the slice kind (matching the F-D guard); strip
  SL-056's comment-only table as fallout (pure removal — no edge); remove the
  `name == "056"` hardcode; re-scan all slices at execution for other comment-only
  stale tables. Entity scope +1 (now: ADR-011 done + 10 backlog + SL-056). §5.3,
  §5.5 updated.
- **F-F — §9 black-box coverage omits policy/standard/backlog-risk. (MAJOR;
  ACCEPT.)** Six templates are fixed but §9 named only `slice new`/`adr new`/
  `backlog new`. `doctrine policy new` / `standard new` exist; the risk backlog has
  its own template (`backlog-risk.toml`). With the RustEmbed footgun, an unasserted
  scaffold path is an unverified fix. Fix: §9 black-box set adds `policy new`,
  `standard new`, and the risk-backlog scaffold path.
- **F-G — D1 has no bounded cutover rule. (MAJOR; ACCEPT.)** "Switch to a
  migrator if inflation is material" deferred the threshold to the executor. Fix:
  D1 pins it — link+strip holds while populated migrated keys ≤ 1 (today: IMP-045
  only) AND every other fallout file is an empty-key removal AND total malformed
  files ≤ ~12. If the execution re-scan finds >1 populated migrated key, OR any
  populated non-backlog fallout, OR a materially larger malformed set → switch to a
  scan-driven migrator or rescope before continuing.
- **F-H — `view()` quoted-key false negative. (MINOR; ACCEPT, low-likelihood.)**
  `view()` extracts a key with `line.split('=').next().trim()`, keeping the quotes
  on a legal `"slices" = []`, so a quoted migrated key would evade
  `assert_no_migrated_key_left`. The corpus is template-born (bare keys only), so
  this cannot arise today, but the guard is meant to be defensive. Fix: strip
  surrounding quotes when extracting the key in `view()`. (A full `toml`-parsed
  entity check is deferred — YAGNI until a quoted-key entity is ever observed.)
- **F-I — internal count contradiction. (MINOR; ACCEPT.)** §5.4 step 1 said
  "the four templates"; §5.2/§10 say six. `slice-058.md` §Context still framed
  three. Fix: §5.4 → six; `slice-058.md` reconciled.

Scope-completeness re-derivation (charge item 1, independent): every `Tier::One`
rule in `RELATION_RULES` was cross-checked against every entity template.
`GovernedBy` (SLICE/PRD/SPEC) and `Consumes` (PRD) are tier-1, but
`spec-product.toml` / `spec-tech.toml` emit NO `[relationships]` table and never
template `governed_by`/`consumes` — confirmed clean. The six-template set is
**complete**; no kind beyond {slice, adr, policy, standard, backlog,
backlog-risk} is born malformed.

Governance-alignment re-derivation (charge item 4): the kept-typed partition is
not an assumption — `e2e_relation_migration_storage` lines 184-223 (governance)
and 226-249 (backlog) assert it against the corpus. adr/policy/standard keep
`supersedes`/`superseded_by`/`tags` typed (OD-3: gov `supersedes` is
`LifecycleOnly`, storage-excluded) and migrate only `related`; backlog keeps
`needs`/`after`/`triggers` typed and migrates `slices`/`specs`/`drift`. Design
§5.3 matches `RELATION_RULES` exactly.
