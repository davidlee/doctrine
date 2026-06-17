# SL-082 Design: Dispose of `doc/*`

## Frame

SL-021 lifted durable architectural content from 9 `doc/*.md` files into 19 tech
specs under `.doctrine/spec/tech/` (SPEC-003..019, all `draft`). Two
authoritative homes now co-exist — the exact "untrusted prose" problem PRD-012
§1 exists to kill. SL-021's design §9 F4 established an interim authority rule
(tech specs are authoritative once authored; `doc/*` demoted to historical
seed), but physically the directory still lives and agents still read it.
SL-082 removes it and cleans every dangling reference.

Governing inputs: ADR-003 (canonical change loop), ADR-009 (slice lifecycle),
SL-021 design §2 taxonomy (doc→spec mapping) + §9 F4 (interim authority rule),
SL-084 (CLAUDE.md separation — no longer a symlink; adds dispatch skills to the
skills tree).

## 1. Current vs target behavior

**Current:** 9 `doc/*.md` files committed at repo root. Referenced from 6 source
files, 8 skill files, 2 install templates, and 6 memory records. SL-021's 19
tech specs carry the same architectural content in proper entity form.

**Target:** `doc/` removed from the working tree. Every reference repointed to
an existing entity (tech spec, ADR) or removed if the prose convention is
heretical. No `.gitignore` guard — removal is the only gate. `rg 'doc/'` across
the repo returns zero hits for legacy `doc/` references.

## 2. Post-SL-021 drift check

One commit touched `doc/` after SL-021 created the tech specs (commit
`8305613`, Jun 11): `8f0e49f` (SL-066, Jun 14). It updated
`doc/entity-model.md` and `doc/spec-entity-spec.md` to reflect ADR-013
(Revision as standalone change-axis kind). The changes were:

- `entity-model.md`: resolved the open REV- home question; replaced "Push back"
  with a "Resolved" decision block citing ADR-013.
- `spec-entity-spec.md`: removed REV- from the spec family at three sites +
  Open-Q #4; replaced "deferred (no subtype in v1)" with "not a spec subtype
  — ADR-013."

These are editorial resolutions of existing open questions, not new
architecture. All content is also captured in ADR-013. The drift confirms the
duplicate-authority problem — `doc/` was still used as a sync surface for
architectural decisions — but introduces no net-new material that needs
rehoming before disposal. No mapping changes required.

## 3. Reference map — per-citation replacement

### 3.1 Source code (6 files)

| # | File:line | Current | Replacement |
|---|---|---|---|
| S1 | `src/boot.rs:1689,1709,1737,1847` | Test fixtures: `"point at doc/spec.md"` | `"point at .doctrine/spec/tech/"` |
| S2 | `src/corpus.rs:820` | `paths = ["doc/"]` (corpus config) | `paths = [".doctrine/spec/tech/"]` |
| S3 | `src/corpus.rs:895` | Test: string-replace `paths = ["doc/"]` → `tags` | Replace target: `paths = [".doctrine/spec/tech/"]` |
| S4 | `src/coverage.rs:369` | Doc comment: `source = "file:doc/*.md"` | `source = "file:.doctrine/spec/tech/*/*.md"` |
| S5 | `src/coverage.rs:1173` | Test: `MatchSource::File("doc/../../x")` | `MatchSource::File(".doctrine/spec/tech/../../x")` |
| S6 | `src/coverage.rs:1200,1218-1219` | Test: `MatchSource::File("doc/*.md")` | `MatchSource::File(".doctrine/spec/tech/*/*.md")` |
| S7 | `src/coverage.rs:1247` | Test: `MatchSource::File("doc/spec.md")` | `MatchSource::File(".doctrine/spec/tech/003/spec-003.md")` |
| S8 | `src/install.rs:394` | Comment: `unembedded under doc/` | `unembedded under the legacy doc/ directory` (historical note, keep) |
| S9 | `src/memory.rs:3017` | Test: `strings(&["doc/"])` | `strings(&[".doctrine/spec/tech/"])` |
| S10 | `src/spec.rs:120` | Doc comment: `` legacy canon `doc/spec-entity-spec.md` `` | `` legacy canon `doc/spec-entity-spec.md`; superseded by SPEC-004 `` |

### 3.2 Skills (8 files, 11 refs)

| # | File:line | Current | Replacement |
|---|---|---|---|
| K1 | `canon/SKILL.md:18` | `` `doc/*` — evergreen, authoritative specs.`` | `` `.doctrine/spec/tech/` + `.doctrine/adr/` — authoritative specs and decisions.`` |
| K2 | `slice/SKILL.md:10` | `evergreen spec material (that lives under \`doc/*\`).` | `evergreen spec material (\`.doctrine/spec/\`).` |
| K3 | `slice/SKILL.md:15` | `` `/canon` for ADRs, `doc/*`, and`` | `` `/canon` for ADRs and `.doctrine/spec/tech/`, and`` |
| K4 | `slice/SKILL.md:43` | `author under \`doc/*\`.` | `author a tech spec (\`doctrine spec new tech\`) or product spec (\`doctrine spec new product\`).` |
| K5 | `route/SKILL.md:36` | `Authoring evergreen specs under \`doc/*\` →` | `Authoring tech specs → \`/spec-tech\`; product specs → \`/spec-product\`` |
| K6 | `audit/SKILL.md:27` | `` `design.md`, ADRs, and `doc/*`, not a spec engine.`` | `` `design.md`, ADRs, and `.doctrine/spec/tech/`, not a spec engine.`` |
| K7 | `audit/SKILL.md:41` | `` relevant ADRs and `doc/*` specs (see `/canon`)`` | `relevant ADRs and tech specs (see \`/canon\`)` |
| K8 | `inquisition/SKILL.md:58` | `` `.doctrine/adr/` (ADRs — `doctrine adr list`), `doc/*` (evergreen specs)`` | `` `.doctrine/adr/` (ADRs), `.doctrine/spec/tech/` (tech specs), `.doctrine/spec/product/` (product specs)`` |
| K9 | `preflight/SKILL.md:90` | `related specs under \`doc/*\`` | `related tech specs under \`.doctrine/spec/tech/\`` |
| K10 | `record-memory/SKILL.md:82` | `or author under \`doc/*\` instead.` | `or author a spec or ADR instead.` |
| K11 | `design/SKILL.compact.md:22` | `` related `doc/*` specs `` | `` related `.doctrine/spec/tech/` specs `` |

**Not changed:**
- `retrieve-memory/SKILL.md:8` — `file/doc/ADR` is a generic
  enumeration ("file, doc, or ADR"), not a `doc/` directory path. False positive.
- `CLAUDE.md`, `AGENTS.md` — confirmed zero `doc/` references (SL-084
  cross-informing round). No changes needed.

### 3.3 Install templates (2 files)

| # | File:line | Current | Replacement |
|---|---|---|---|
| I1 | `install/glossary.md:100` | `### \`doc/*\`` (section heading + body) | Remove section; glossary entry superseded by `.doctrine/spec/tech/` + `.doctrine/adr/` |
| I2 | `install/governance.md:9` | `doc/* → evergreen authoritative specs` | `.doctrine/spec/tech/ → authoritative tech specs; .doctrine/adr/ → authoritative decisions` |
| I3 | `install/governance.md:20` | `<!-- - Architecture spec: doc/architecture.md -->` | Remove comment (stale — references a file that never existed) |

### 3.4 Memory records (6 records)

| # | Record key | Paths field | Replacement |
|---|---|---|---|
| M1 | `mem.concept.backlog.work-intake-membership` | `["doc/entity-model.md", ".doctrine/spec/product/009"]` | `[".doctrine/spec/tech/004", ".doctrine/spec/product/009"]` (SPEC-004 entity engine) |
| M2 | `mem.fact.backend.forgettable-event-store` | `["doc/memory-spec.md"]` | `[".doctrine/spec/tech/007"]` (SPEC-007 memory engine) |
| M3 | `mem.concept.workflow.canonical-change-loop` | `["doc/slices-spec.md"]` | `[".doctrine/spec/tech/014"]` (SPEC-014 slice surface) |

Prose-only refs (no `paths` field change; update markdown body):

| # | Record key | Prose ref | Replacement |
|---|---|---|---|
| M4 | `mem.concept.spec-composition-requirement-peers` | `` `doc/spec-entity-spec.md` `` | `SPEC-004` (entity engine) |
| M5 | `mem.concept.backlog.work-intake-membership` (md) | `` `doc/slices-spec.md` `` | `SPEC-014` |
| M6 | `mem.idea.governance-surface-customizable` | `` `doc/*` `` ×2 | `` `.doctrine/spec/tech/` `` + `` `.doctrine/adr/` `` |

## 4. Design decisions

### D1 — Per-citation mapping, not batch sed

Each `doc/` reference has a specific semantic — a test fixture expecting a
particular path shape, a comment citing a specific legacy doc, a skill prose
convention directing agent behaviour, or a memory `paths` scope anchor.
Mechanical `s|doc/|.doctrine/spec/tech/|g` would produce wrong results in
multiple cases (e.g., `"doc/spec.md"` → `".doctrine/spec/tech/spec.md"` which
doesn't exist). Each citation mapped individually per §3.

### D2 — No `.gitignore` guard

Removal is the only gate. If `doc/` is resurrected, CI or the next `rg` sweep
catches it.

### D3 — Delete the `doc/*` mental model from skills

The `doc/*` convention is heretical, not just a stale path. Three patterns
deleted and replaced:

- **"evergreen specs under `doc/*`"** → entity surface (`.doctrine/spec/tech/`
  + `.doctrine/adr/`)
- **"author under `doc/*`"** → `doctrine spec new` / `doctrine adr new` verbs
- **"see `doc/*`"** → see the tech spec corpus + ADRs

The replacement consistently points agents at the first-class entity surface
rather than a flat unstructured directory.

### D4 — Memory `paths` updates are scope-only

The memory engine's `paths` field is a scope anchor (what the memory is
*about*), not a live dependency. Updating from `doc/*` paths to
`.doctrine/spec/tech/NNN/` directories does not change the memory's meaning or
verification status. No `verified` flag reset needed. Prose refs in
`memory.md` bodies updated for consistency.

### D5 — Stale comment removal

`install/governance.md:20` contains `<!-- - Architecture spec:
doc/architecture.md -->` — an HTML comment referencing a file that never
existed. Removed.

## 5. Phase shape (provisional — for `/plan`)

Five phases. PHASE-01 through PHASE-04 are file-disjoint across four surfaces
(source, skills, install, memory) and could run concurrently. PHASE-05 must
be last (removes `doc/` after all references are clean).

| Phase | Surface | Files |
|---|---|---|
| PHASE-01 | Source code | `src/boot.rs`, `src/corpus.rs`, `src/coverage.rs`, `src/install.rs`, `src/memory.rs`, `src/spec.rs` |
| PHASE-02 | Skills | 8 SKILL.md files under `plugins/doctrine/skills/` |
| PHASE-03 | Install templates | `install/glossary.md`, `install/governance.md` |
| PHASE-04 | Memory records | 6 memory records under `.doctrine/memory/items/` |
| PHASE-05 | Remove `doc/` + verify | `git rm -r doc/`, final `rg` sweep, `just check`, `doctrine claude install` |

## 6. Verification

| Id | Mode | Criterion |
|---|---|---|
| VA-1 | Agent | Every reference in §3 is repointed per its row or removed |
| VT-1 | Test | `rg 'doc/' src/` returns zero hits for legacy `doc/` paths (allow `"legacy doc/"` in historical comments) |
| VT-2 | Test | `rg 'doc/\*' plugins/doctrine/skills/` returns zero hits |
| VT-3 | Test | `rg 'doc/' install/` returns zero hits |
| VT-4 | Test | Memory record `paths` fields reference existing `.doctrine/spec/tech/NNN/` directories |
| VT-5 | Test | `just check` green |
| VT-6 | Test | `doctrine claude install` succeeds; installed skills match source |
| VT-7 | Test | `doc/` does not exist in the working tree |

## 7. Risks

- **Source test breakage (low).** `src/coverage.rs` tests parse `MatchSource::File`
  values that currently use `doc/` paths. Changing those paths may break test
  assertions if they check path string equality rather than parsed semantics.
  Mitigation: read the test assertions before editing; adjust expected values
  inline.

- **Corpus config behavioural change (low).** `src/corpus.rs:820` has
  `paths = ["doc/"]` — changing this to `.doctrine/spec/tech/` may alter which
  corpus paths the memory engine scans. The memory engine's path scanning is
  documented in `memory-spec.md`; verify the change is semantically equivalent
  (a directory of specs vs a directory of markdown — the engine may filter by
  file type).

- **Skill install propagation (procedural).** After editing skill source files
  under `plugins/`, `doctrine claude install` must be re-run so the derived
  tree (`.doctrine/skills/`) matches. Both PHASE-02 and PHASE-05 include this
  step.

- **Memory record concurrent mutation (procedural).** Memory records under
  `.doctrine/memory/items/` may be updated by other agents during this slice.
  Low risk given the narrow scope of changes (paths + prose), but verify no
  conflicts before committing PHASE-04.
