# research.md — RFC-033 (learning surface: corpus unification, format, publication)

## Header

**Producers**

| thread | runner | raw output |
|---|---|---|
| 1 — governance applicability | `./scripts/pi-research --think medium` (deepseek-flash) | `raw/thread-1-governance.md` |
| 2 — surface census and duplication | `./scripts/pi-scout` (deepseek-flash) | `raw/thread-2-census.md` |
| 3 — prior attempts and rejected options | `./scripts/pi-research --think medium` | `raw/thread-3-history.md` |
| 4 — external prior art | orchestrator, `web_search` + `fetch_content` | recorded inline in this file (§ Thread 4) |

**Baseline.** None. The `/research` staleness verb is slice-scoped
(`doctrine slice research <id>`); RFCs carry no `baseline.toml`. A consumer must
re-check by hand or refresh this artefact. Threads 1–3 are dated 2026-09-26 and
were run against `161c916b1`.

**Consumers.** The RFC's *Discussion*; a later PRD and tech spec if the change
proceeds; any Revision descending from ADR-005 / ADR-019 / ADR-024.

**Verification legend.** ✓ = independently verified by the orchestrator at
authoring time, by re-running the command or reading the cited site. Unmarked =
researcher claim: cited, not checked. Consumers may only lean on ✓ rows, or rows
they verify at point of use.

---

## Thread 1 — governance applicability

### Binding constraints

| artefact | what it constrains | anchor status |
|---|---|---|
| **ADR-005** | the tiering itself: push / pull-reference / skills. Invariants a change must not break: "single source per fact"; "exactly **one** workflow doc (`routing-process.md`)"; the ratified restate line — a skill may name a verb, must not reproduce flag syntax, option tables, or storage mechanics as prose. It constrains duplication; **it nominates no source.** | prose-only; its `§References` cites `src/skills.rs`, which **no longer exists** (the skills embed is `src/install.rs:20`) |
| **ADR-019** | owns the asset vocabulary — embedding, publication, projection — and states it is "the one canonical vocabulary — descendants cite it unchanged". A unification that coins a parallel vocabulary violates this. | prose-only; cites `src/install.rs` (**live**) |
| **SPEC-018** | the worked precedent for a single source: the relation vocabulary's source is the code-authoritative `RELATION_RULES` (`src/relation.rs:379` ✓), "pointed at, never transcribed". Binds the *shape* of any source nomination. | live ✓ |
| **SPEC-003** | the whole-system synthesis altitude: it "holds the principles that hold across all containers … never restating any single container's mechanism". Cross-surface concepts belong here, not on a new surface. | no `[[source]]` block, by design |
| **SPEC-026 / PRD-017** | defines "published": the publication manifest is the sole authority; the library is read-only; licence is a closed set **{MIT, GPL}**, fail-closed. A personal blog is none of these — it is a publish-*out* target. | SPEC-026 anchor `src/search.rs` live |
| **ADR-024** | shipped-corpus grounding — but §Scope fences itself to **shipped** assets only: it "does not govern slice prose, design docs, specs, reviews, this repository's own governance, or its source and tests". | prose-only |
| **SPEC-011 / PRD-007** | the boot snapshot's section and source taxonomy lives in `boot_sequence` (`src/boot.rs:104`) ✓ — a code-visible contract, not an authoring choice. | live ✓ |
| **SPEC-016 / SPEC-005 / SPEC-019 / SPEC-031** | per-kind render contracts (POL/STD, ADR, knowledge record, phase plan). A single authoring format must not silently drift these. | live |
| **STD-001 / STD-002 / STD-003** | shared vocabulary and naming; **no silent skip** — a degraded read is disclosed. STD-003 binds whatever enforcement mechanism is chosen. | standards |
| **POL-001** | tone. Binds copy on any public surface. | prose-only |
| **ADR-013 / ADR-014** | the change machinery: governance change lands through a Revision; an RFC asserts no canon. | accepted |

### Checked and not applicable

- **POL-002** — scoped to *shipped* behaviour; this repo's own blog toolchain is
  its own outward-facing practice. It would bind only if doctrine shipped an
  export verb.
- **POL-003** — reaches harness coupling only (hooks, codecs, matchers, adapters).
- **ADR-002** — governs the orientation memory *mechanism* (the `repo=""`
  class), not corpus content. Corroborates the census's "no spec anchors
  `memory/`": `grep -rln 'identifier = "memory' .doctrine/spec` → nothing.
- **ADR-011, ADR-001** — spawn interface; module layering. No bearing on content.
- **SPEC-023 / PRD-007** — governs composable *guidance* (the hymn cascade),
  not the corpus's format authority.
- **SPEC-025 / PRD-016 / SPEC-027** — govern the *local* reading UI
  (loopback-only, presentation-neutral projections). Silent on a public target.
- **SPEC-007 / PRD-004** — owns the memory *engine*, not corpus content. It does
  own one live mechanism the change must respect: the boot `## Onboarding`
  section is onboarding-tagged memory body text (`src/boot.rs:253-256`;
  `memory::onboarding_bodies`, `src/memory.rs:3013`).
- **SPEC-010 / PRD-003** — owns the skill *tree and delivery*, not skill content.
- **PRD-006 / SPEC-009** — projection mechanics. Applies to root reorganisation
  only.
- **RFC-017** — tier 5, asserts no canon. Evidence, not constraint.

### Revision candidates

- **ADR-005** — needs *amendment or a superseding ADR*, not a Revision alone: the
  gap is a missing axis (derivation), not a stale delta. Blocking clauses:
  §Decision "exactly **one** workflow doc" and the restate line. Also stale:
  `§References` → `src/skills.rs`.
- **SPEC-026 + PRD-017** — need a Revision or a descending spec to permit a
  public target: SPEC-026 FR-001 (manifest is sole authority), D3 (library is
  read-only), D6 (licence closed set). Either extend the model with an outbound
  export contract, or fence the blog explicitly out of scope.
- **ADR-024** — a Revision *only if* blog material must obey the grounding rule;
  the §Scope fence currently excludes it. A blog reader has the same
  silent-substitution failure ADR-024 exists to prevent.
- **SPEC-011 / PRD-007** — a Revision if the boot's source taxonomy changes
  (e.g. demoting the inlined onboarding memory body now that it is visible as
  duplication).
- **DEC-010** — `proposed` and acknowledged stale; it asserts "published set =
  full projection complement". Settle or supersede.
- **SL-144 / CHR-023** — blocking open work, not governance: `ready`, 0/5, scope
  reconciled 2026-07-03, i.e. *before* ADR-019. Re-scope, re-sequence, or
  supersede before a unification programme routes.

### Open questions this thread cannot settle

1. **Which "single source" is meant** — *authoring* source (one file others
   mechanically derive) versus *governance owner* (one artefact owns the rule,
   the rest cite it). ADR-005 chose access-pattern tiering; the two readings
   imply different revisions.
2. **Is the personal blog in doctrine's product scope?** In scope implies a new
   publication contract (export path, licence, citation grounding, format); out
   of scope implies a fence.
3. **The licence of the shipped memory corpus.** The engine is
   `GPL-3.0-only` ✓ (`Cargo.toml:6`); `install/` is uniformly MIT; the memory
   corpus carries no per-asset declaration and is absent from the publication
   manifest.
4. **What "unify the format" ranges over.** The surfaces are heterogeneous:
   entities in TOML+MD, `SKILL.md`, `install/*.md` prose, `README.md`, a
   TypeScript frontend. One format, or one information architecture *over*
   several formats?
5. **Whether a public reading surface reuses the loopback explorer** (SPEC-025 is
   loopback-only) — design, downstream of the product decision.
6. **The enforcement home.** STD-003 binds disclosure, but no currency check
   exists: SL-242 §Objective 4 proposes one that `src/doctor_checks.rs` does not
   implement.

---

## Thread 2 — code map (surface census)

### Surface inventory ✓

| surface | extent | lines | home |
|---|---|---|---|
| boot snapshot (generated runtime) | 31 `##` sections ✓ | 627 ✓ | `.doctrine/state/boot.md` |
| shipped memory corpus | 35 memory entities (31 id-named dirs + 31 slug symlinks = 66 glob matches; 4 slug-named real dirs), types: 21 signpost / 8 concept / 4 pattern / 2 fact | 1 260 ✓ (distinct bodies; the naive `memory/*/memory.md` glob returns 2 401 by counting symlink targets twice) | `memory/` |
| published reference docs | 17 `install/*.md` ✓; 15 get a `reference/` address | 2 182 ✓ | `install/` → `publication/manifest.toml` |
| skill masters | 39 masters, 41 `.md` ✓ | 5 315 ✓ | `.agents/skills/` (`.pi/skills/` is 39 symlinks) |
| README | 1 | 343 ✓ | `README.md` |
| web map frontend | 22 `.ts` + `index.html`, built to `dist/` | — | `web/map/` |
| vendored, not doctrine's own | 18 files | — | `docs/claude/` |

Two of the 17 `install/*.md` do **not** become references:
`project-orientation.md` (a base backing) and, for a second address, a duplicate
`claude-activation.md` manifest row.

### Duplication evidence

The shared facts, and where each appears:

- **`mem.signpost.doctrine.overview`** (82 lines, the densest node) — the source
  `memory/mem_019e9a11b3797af3a8833c67acfa69bf/memory.md`, inlined verbatim into
  boot `§Onboarding` (`.doctrine/state/boot.md:390-480`), and therefore into
  every session's cached prefix. It restates pillars, storage tiers, the "CLI is
  source of truth" rule, and the core loop.
- **"The CLI is the source of truth / don't guess the command shape"** — the
  rule is restated across **24 files** ✓ (`rg -li 'source of truth' memory
  install .agents README.md`), including 12+ per-verb memories, 4 skills, 3
  reference docs, and boot.
- **Storage rule / storage tiers / "read via `show`"** — 8 files
  (`mem.fact.doctrine.storage-tiers`, `mem.concept.doctrine.storage-model`,
  `mem.signpost.doctrine.overview`, `mem.signpost.doctrine.file-map`,
  `install/using-doctrine.md:177,188`, `install/glossary.md:109`,
  `.agents/skills/canon/SKILL.md:32`), all of which boot then re-inlines.
- **Core loop** — 7 surfaces, with **three different orderings**:
  `lifecycle-start` (`route → slice → design → plan → phase → audit → reconcile
  → close`), `audit` (`… phase-plan → execute → audit → close`), and
  `install/routing-process.md:49-56`.
- **Route / routing gate** — ~12 surfaces including the boot opener
  (`install/routing-process.md:1-3`) and `mem.concept.doctrine.routing-gate`.
- **`mem.signpost.doctrine.reference-docs`** — names 4 docs; the publication
  manifest carries 18 reference entries. The index has drifted from the register.
- **`mem.signpost.doctrine.skill-map`** — deliberately does not enumerate skills
  (it defers to the boot routing table), so the skill set is restated instead in
  `install/routing-process.md` (86 lines) and boot's ~40-row routing table.
- **Boot `§Onboarding`** inlines **two** memory bodies: the shipped overview and
  the local `mem.signpost.project.orientation`.

### Projection and materialisation mechanics

- **Five RustEmbed roots** — `src/install.rs:20` (`plugins/`), `src/map_server/assets.rs:14`
  (`web/map/dist/`), `src/corpus.rs:44` (`memory/`), `src/asset_source.rs:21`
  (`install/`), `src/asset_source.rs:29` (`publication/`).
- **`flake.nix:345-357`** re-grafts each root after crane's `cleanCargoSource`
  strips non-Rust assets (`rm -rf` + `cp -R` for `plugins install memory
  web/map/dist templates publication`). Six grafted names against five embed
  roots: a new root added in Rust but not grafted here ships asset-incomplete.
- **`memory sync`** materialises the embedded corpus into the gitignored
  `.doctrine/memory/shipped/` (`src/corpus.rs:398-420`), with a bounded prune that
  discloses rather than silently drops non-parsing children.
- **Write-if-absent vs published-on-demand** — `install/manifest.toml:11`
  eagerly projects three base backings (`[.gitignore, doctrine.toml,
  project-orientation.md]`); everything else embedded is reachable only through
  the publication manifest. `[memory] seed_items = []` retires the eager
  orientation memory seed.
- **The publication register is derived, not curated**: the completeness
  invariant is `{embedded filenames} − {base backings} ⊆ published backings`
  (`publication/manifest.toml:10-12`).

### Hotspots

- `src/boot.rs` — the single projector; any surface change lands in
  `boot_sequence` (`src/boot.rs:104`) ✓ plus golden tests. Highest blast radius.
- `install/routing-process.md` — feeds boot *and* is itself published.
- `publication/manifest.toml` — every embedded asset needs a row or reachability
  breaks; a duplicate row already exists.
- `memory/mem_019e9a11b3797af3a8833c67acfa69bf/` (the overview) — the densest
  duplication node.
- `install/using-doctrine.md` — restates storage tiers, core process, and
  read-via-show; its installed copy `.doctrine/using-doctrine.md` (114 lines) has
  diverged from the 299-line source.
- `flake.nix:345-357` — must track `src/`'s embed roots.

---

## Thread 3 — prior attempts, contradictions, rejected options

### Prior attempts

- **ADR-002** (accepted) — defined the global-orientation memory class.
- **ADR-005** (accepted) — the tier rule. Left unbuilt: the PULL-tier "which verb
  for what" doc, restate-line enforcement, the PUSH-tier reference-forms block,
  and the `install/*.md` information architecture → routed to SL-144/CHR-023.
- **ADR-019** (accepted) — separated embedding, publication and projection;
  mandated minimal projection. Deferred the trust-acceptance gate and built no
  client refresh path (→ IDE-030).
- **ADR-024** (accepted, delivered in SL-267) — grounding rule; the drift gate was
  deferred (ISS-309 part 2).
- **SL-069** (done) — 13 memories added to make the corpus an onboarding anchor;
  boot Memory section trimmed. Left the self-updating mechanism open.
- **SL-143** (done) — swept 29 memories into a curated path (overview → concepts
  → workflow → reference). Left the self-correction gate unbuilt (→ IMP-163) and
  skipped knowledge/backlog signposts.
- **SL-144 / CHR-023** (`ready`, 0/5) — ADR-005 compliance: `install/*.md` IA,
  user hooks, restate-line audit, doc currency. Scope predates ADR-019;
  still treats `boot-footer.md` as live though SL-187 retired it.
- **SL-200 / SL-202** (done) — canonicalised body wikilinks and made them
  first-class graph edges.
- **SL-187** (done) — inlined onboarding-tagged memory bodies into the boot
  sector and retired the `boot-footer.md` round-trip.
- **SL-201** (done) — added `doctrine onboard`.
- **SL-227 / REV-031** (done) — projected `project-orientation.md` as the base
  orientation surface, replacing IDE-020's seeded orientation memory.
- **SL-267** (done) — corpus conformance sweep; authored ADR-024.
- **SL-242** (proposed) — retire the projected reference-doc model; settle
  DEC-010; add a doctor citation check.
- **DEC-010** (`proposed`) — published set = full projection complement;
  acknowledged out of date with SL-227.
- **IMP-315, IDE-030, IMP-163, ISS-313, CHR-079, CHR-078, IMP-154** (open) —
  stale projected docs; no client refresh verb; no self-correction gate; stale
  published glossary; no doc-to-help consistency check; the unified `show`
  router unnamed in guidance; search-wide indexing.
- **RFC-011, RFC-016** (open) — token/zero-rescue work; RFC-017's tests borrow
  from both.
- **RFC-017** (open, 2026-07-05) — human onboarding docs: positioning, tour,
  quickstart, mental-model page, FAQ. Its premise ("there is no human-facing
  on-ramp") is partly stale: `doctrine onboard`, clickable memory wikilinks, and
  the projected orientation file have all landed since.

No EVD, ASM or HYP record exists about docs, the memory corpus, or publication;
the knowledge corpus on this surface is entirely DEC and QUE.

### Contradictions and live tensions

1. **`project-orientation.md` vs SL-242** — *not* a direct conflict. SL-242 is
   scoped to the *projected reference-doc copies* under `.doctrine/`, explicitly
   keeps ADR-019 standing, and its residue inventory omits
   `project-orientation.md`. The genuine tension is one level out: **SL-242 vs
   SL-144**, which audit the same `install/manifest.toml` surface but were
   scoped 15 days apart across the ADR-019 flip. That is SL-242's own OQ-2,
   unanswered.
2. **RFC-017's premise is partly stale** (see above), while its three-part
   thesis — why / mental model / first success — is not.
3. **QUE-172 is `answered` with an empty answer facet**; the substantive answer
   lives only in DEC-010, which is `proposed` and stale.
4. **`boot-footer.md` is dead but undeleted** — described as retired in three
   places, removed by none; `install/boot-footer.md` is still tracked.
5. **IDE-020 (`done`) describes an architecture the product no longer has** — a
   seeded orientation memory, replaced by a projected file.
6. **RFC-017's proposed in-repo `docs/` home is occupied** by vendored Claude
   Code reference material.
7. **Two overlapping "no repo-private citation" rules** — DEC-127 (for `install/`)
   and ADR-024/DEC-311 (corpus-wide), with ADR-024 calling DEC-127 "related, not
   superseded".

### Already-rejected options

An RFC on this surface must not silently re-propose these:

- Exposing the memory corpus harder / the web UI as the primary docs (RFC-017 —
  agent-voiced reference assumes the mental model it should teach).
- A comprehensive manual (RFC-017 — redundant under agent mediation, competes
  with CLI-as-source-of-truth).
- A screencast or video tour (RFC-017 — deferred; rots faster than an annotated
  artefact chain).
- Naming the orientation surface `project-onboarding.md` (REV-031 — "onboarding"
  narrows standing orientation to first-run).
- A second TOML link store for memories (SL-200 — chose body wikilinks as the
  single store).
- A bounded published set (DEC-010, reversed after RV-299 X-F1 refuted its
  completeness).
- Restoring projection for the reference tier (IMP-315 option 2 — re-opens what
  ADR-019 narrowed).
- Rewriting every bare `<name>.md` citation to carry `library show` (SL-242 —
  parallel implementation).
- A migration primitive for already-installed clients (SL-242 — filed as
  IMP-378).
- A signpost per verb (DEC-315 — "the CLI is a product surface, not a table of
  contents, and reproducing `--help` in prose rots").
- Spec-sibling, duplicated, or hand-authored diagrams (DEC-127 — chose
  published + generated).
- "Status quo + better memories", one mega-verb, deleting reference classes
  (RFC-016).

### What remains genuinely undecided

RFC-017's OQ-1…OQ-7 (tour subject, tour format, quickstart's honest floor,
stranger-test protocol, mental-model page ownership, what a curated start-here
trail needs, README restructure); whether the shipped memory corpus is
published-for-copy; SL-242's OQ-1…3; how the corpus stays current (no working
currency check exists); the client-repo currency gap; RFC-017's relationship to
`project-orientation.md`; SL-144's disposition; IMP-232's disposition (its
target seed was retired).

---

## Thread 4 — external prior art

Consulted at authoring time; no raw file.

- **Diátaxis** (<https://diataxis.fr/>) — the dominant published framework for
  documentation content and architecture. It "identifies four distinct needs, and
  four corresponding forms of documentation — *tutorials*, *how-to guides*,
  *technical reference* and *explanation*" — and "solves problems related to
  documentation *content* (what to write), *style* (how to write it) and
  *architecture* (how to organise it)". Relevant because doctrine's corpus is
  almost entirely the *reference* and *explanation* modes, with no tutorial and
  no human-voiced orientation — which is RFC-017's diagnosis, arrived at
  independently.
- **Dual-audience practice** —
  <https://designsystemdocspec.org/humans-and-agents> argues one document can
  serve both readers, with a small agent-only addition;
  <https://stackademic.com/blog/docs-for-humans-vs-docs-for-ai-agents> argues
  both audiences "share a source of truth — the actual system — but they are
  different representations of that truth". The second is the more relevant
  position for doctrine, whose corpora differ in *voice*, not only in detail.
  <https://styleguide.ritza.co/ritza%27s-writing-rules/thinking-of-the-agents/>
  covers agent-legible conventions (`llms.txt` and similar).
- **Docs-as-code single-source pipelines** — the common published pattern is
  generated marker blocks rebuilt from one source with a CI drift check; see
  <https://www.noboil.dev/docs/single-source-of-truth> ("53 generators … every
  block is rebuilt from source … CI fails on drift") and
  <https://sourcegraph.com/blog/documentation-as-code>. Doctrine already has
  this shape for boot (`boot_sequence` + goldens) and for the publication
  register (a derived completeness invariant).
- **Publishing markdown to a static site** — several mature generators exist
  (<https://docmd.io/>, <https://github.com/jmagly/pagenary>,
  <https://github.com/so1ve/inkcairn>), all of the form "point the tool at a
  folder of markdown, get navigation, search, feeds, static output". A blog
  target is therefore cheap *mechanically*; the cost is editorial and legal, not
  tooling.

---

## Cross-thread findings

1. **"Single source" is a new axis, not a broken rule.** ADR-005 tiers knowledge
   by *access pattern* and deliberately names no source. So the change is not a
   repair; it adds a derivation axis ADR-005 neither grants nor forbids. Any
   resolution lands as a Revision descending from ADR-005, and probably amends
   it too.
2. **The vocabulary already exists, and it is ADR-019's.** Embedding,
   publication, projection. A unification that coins "source / derived /
   published" language of its own breaks ADR-019's stated canon. The natural home
   for cross-surface concepts is **SPEC-003** — the synthesis altitude that
   "never restat[es] any single container's mechanism" — not a fifth surface.
3. **The boot snapshot is already a projection of four sources.** It inlines two
   `install/*.md` assets, one shipped memory body, one local memory body, two
   command/governance tables, and a per-model hymn. "Many renders from one
   corpus" is therefore not new machinery; it is the existing pattern, extended
   and made principled.
4. **The publication register is the shape a corpus register needs.** It is
   already a sole authority, licence-declared per entry, read-only, with a
   derived completeness invariant. Reusing it (or the same vocabulary) costs
   little; inventing a parallel register costs a great deal and contradicts
   finding 2.
5. **Licence is the hard blocker for public publication.** The engine is
   `GPL-3.0-only` ✓; `install/` is MIT; the shipped memory corpus carries **no
   per-asset licence** and is not in the publication register. SPEC-026's licence
   set is fail-closed. Nothing public can be published from the memory corpus
   until this is settled — an editorial milestone that is really a legal one.
6. **ADR-024's scope fence leaves public material ungoverned.** It governs shipped
   assets; this repo's ADRs, RFCs and specs are excluded. A blog reader has the
   same silent-substitution failure (their `ADR-005` is not ours). Either extend
   the rule to public-facing material or record the exclusion as deliberate.
7. **Two open slices sit directly on this ground.** SL-144 (`ready`, 0/5,
   scoped pre-ADR-019) and SL-242 (`proposed`) both edit
   `install/manifest.toml`. A unification programme must re-scope or supersede
   them, not duplicate them — and settle SL-242's OQ-2 sequencing first.
8. **Duplication is concentrated, not diffuse.** One memory (the 82-line
   overview), one rule ("CLI is source of truth", 24 files), one document
   (`install/routing-process.md`, which boots *and* publishes), and the boot
   projector. That concentration makes a first pass cheap: fix the overview and
   the routing document, and the largest share of the duplication goes.

## Design-input deltas

What this research changes about the intended design:

- **The first decision is not "which format".** It is *which kind of sameness* —
  authoring source (one artefact others mechanically derive) versus governance
  owner (one artefact owns the rule; the rest cite it). The two imply different
  Revisions, and may both be wanted for different material.
- **Enforcement must be sited before content work.** STD-003 forbids silent
  skip, and no currency check exists. Without one, any unification decays the way
  ADR-005's did.
- **The public target needs its own product decision** — in scope, with an
  export contract, licence, and citation grounding; or fenced out explicitly.
  It cannot be settled as a detail of the format work.
- **RFC-017 must be absorbed, not duplicated.** Its five artefacts become the
  human render target of whichever option lands; its OQ-1…7 map onto the new
  questions, and its premise must be refreshed against SL-201/SL-227.
- **ADR-005 is the pivot artefact** and RFC-033 is governance-neutral. The
  resolution route is: RFC-033 discussion → a Revision descending from ADR-005
  (amending ADR-005, ADR-019 and/or ADR-024 as decided) → a new PRD at product
  altitude if the public target is in scope → a tech spec at container altitude
  → slices.
