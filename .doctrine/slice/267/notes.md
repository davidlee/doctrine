# Notes SL-267: Shipped-corpus conformance

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design triage (exploring, 2026-09-26)

Evidence: `research/research.md` (runtime, gitignored) — its thread labels `T1`
(governance), `T2` (code map), `CT` (cross-thread) are cited below. Durable
ledgers: ISS-309 (citations), CHR-080 (accuracy), IMP-484 (sufficiency).

### Constraining governance

- **POL-002** — binds by interpretation; its prohibitions are *mechanism*-worded
  ("enforces, computes, or depends on"), so a shipped *prose citation* is outside
  its text. Textual gap = revision candidate. (T1)
- **DEC-127** — accepted, user-decided: *"no shipped asset may cite a
  repo-private artefact."* Already reaches `install/`, and points at ISS-309 as
  the sweep. (T1)
- **ADR-005** — shipped knowledge tiered by access pattern; the test is
  *reachability, not presence*; the restate line. (T1)
- **ADR-019** — embedding / publication / projection independent → **three
  channels, three reach tests**. (T1/T2)
- **ADR-023** — the grounding rule is tier-3 framework content, delivered through
  boot/reference/skills, not a slice note. (T1)
- **PRD-017 / SPEC-026** — the *stable logical address* per published asset;
  authority for `reference/<name>.md` as the replacement vocabulary. (T1)
- **PRD-003 / PRD-006 / SPEC-009** — skills and install mechanisms; SPEC-009 is
  forward-intent. (T1)
- Not engaged: STD-001 (src-only), STD-002 (entity naming), STD-003 (src-only),
  PRD-004/SPEC-007, PRD-002/SPEC-006. (T1)

### Shaping decisions (proposed)

- **Scope is `install/` + `memory/` + `plugins/`**, the `plugins/` extension
  confirmed by the user 2026-09-26 (it was previously an agent-proposed addition).
- **One change, three axes** — citation conformance (ISS-309), accuracy
  (CHR-080), sufficiency (IMP-484) — resting on one grounding rule.
- **Per-site disposition taxonomy:** `inline | drop | repoint`; default `repoint`
  → a published `reference/<name>.md`; `inline` for one-clause rationale; `drop`
  last resort. (ISS-309 § Shape of the fix)
- **Verification is per-channel and "replacement resolves"** — never "the id is
  gone". No existing gate scans the three corpora, so the slice carries its own
  evidence. (CT1, CT6)
- **The drift gate and the duplicate POL-002 rule stay deferred** (QUE-227,
  ISS-309 part 2). Do not mint a third rule. (CT5)

### Open questions

- `OQ-1` **(pivot, blocking)** — QUE-226: what may a shipped claim be grounded
  on, and at which tier? Options: published address only / prose without a
  reference / a widened vocabulary. (T1)
- `OQ-2` (blocking, needs OQ-1) — does the grounding rule escalate to an ADR
  descending ADR-005/ADR-019, or stay slice-local? (T1, SL-267 Follow-Ups)
- `OQ-3` (blocking, needs OQ-1) — where does the hard cases' rationale live:
  `sketches/thin-adapter.md`'s two load-bearing runbook steps,
  `install/doctrine.toml.example`'s per-knob ids, `inquiring.toml`'s steps?
  (ISS-309, T1)
- `OQ-4` — the verification method's concrete form (what the per-channel
  client-read test records as evidence).
- `OQ-5` — sufficiency: which CLI surfaces get shipped orientation, at which
  tier (IMP-484); the boot index's signposts-only reach.
- `OQ-6` — the accuracy pass's shape: whole-corpus claim-vs-CLI method (CHR-080).

### Assumptions

- `ASM-1` — a published `reference/<name>.md` address is resolvable in every
  client repo by construction (ADR-019/PRD-017). Verify at point of use.
- `ASM-2` — `memory/` masters reach clients via RustEmbed + `memory sync`; a
  master edit is hand-edit + `cargo build` + `sync` + `install` (no write verb).
- `ASM-3` — the sweep is prose-only; no semantic `src/**` change (scope fence).

### Risks

- `R1` — sweeping the illustration classes corrupts the id-vocabulary docs and
  projected templates (`glossary.md`, `routing-process.md:65-66`,
  `templates/*.md`). (ISS-309 § Not violations)
- `R2` — verifying by absence: a zero-hit grep proves the deletion ran, not that
  the reader is served (`mem.pattern.doctrine.reseat-renumbers-does-not-retarget`).
- `R3` — with the drift gate deferred, the corpus re-drifts: the live `IMP-483`
  recurrence is the evidence (now landed via SL-264 `166f69a9f`). Residual, out
  of scope here.
- `R4` — a shipped doc naming the `design-prompts` store trips
  `tests/e2e_claude_install.rs`'s allowlist
  (`mem.fact.design-run.design-prompts-name-is-allowlisted`) — and this slice
  edits files *in* `install/design-prompts/`.
- `R5` — `plugins/` edits need `touch src/install.rs` + rebuild to re-embed
  (`mem.pattern.distribution.skill-refresh-command`); three embeds, three
  rebuild/golden paths. Not a semantic src change, but a build step.
- `R6` — shared worktree, other agents active (SL-264); path-limit add+commit.

## Design review (reviewing, 2026-09-26)

- **Pass:** `RV-391` (design review of SL-267; policy human-only). Four findings
  raised, all integrated `fix-now` and verified: `F-1` (verification had no
  evidence artefact or bound → sec-8 revised), `F-2` (sufficiency destinations
  underspecified → sec-6 revised), `F-3` (`POL-002` textual gap → sec-5
  dispositioned), `F-4` (new design-run doc vs existing homes → sec-5 justified).
  Sections 5, 6 and 8 were re-declared and re-materialised (rev 24→25).
- **What a further review pass would probe (review.passes):** the two new
  published docs' concrete filenames and their `publication/manifest.toml`
  entries, and whether the `install/routing-process.md` pointer to the authoring
  doc reads as client noise; and the accuracy pass's sample coverage across the
  corpus. No further pass is needed before locking: the contested choices were
  settled in inquiry (DEC-311..316), the review findings are integrated, and the
  remaining uncertainty is implementation-level, exercised by the plan's
  verification and the audit.

## PHASE-02 disposition ledger — install/ sweep

Evidence for PHASE-02 `EX-1`/`EX-2` (`design.md` sec-8: one row per candidate,
`file:line | class | disposition | resolution`). Built **before any edit** (`EN-2`).

**Locators** (a grep locates; it does not conclude — `DEC-314`):

- id grep: `grep -rnE '\b(SL|ADR|RV|REV|PRD|SPEC|IMP|ISS|CHR|DEC|RFC|REQ|POL|STD|IDE|QUE|RSK|ASM|REC|CM)-[0-9]{3}\b' install/` → **115 lines / 40 files**.
- path grep: `grep -rnE '(install/[A-Za-z0-9_./-]+\.(md|toml|rs)|src/[A-Za-z0-9_./-]+\.rs|\.doctrine/(spec|adr|slice|memory|requirement)/)' install/` → the second dimension the id regex misses (`A2`).
- **Positive controls** (so the empty results are trusted): the same id regex finds ids in `.doctrine/adr/019/adr-019.md`; the path regex finds `src/relation.rs` in `.doctrine/slice/267/design.md`. `grep -rn 'design-prompts' install/` finds the SL-260 allowlisted `install/review-ledger.md:182` (D3).

**Do-not-sweep classes (EX-2), transcribed first — all `leave`:**

- `install/glossary.md` kind↔abbreviation table (:8-36) — reference-form illustration.
- `install/routing-process.md:65-66` (`SL-023`/`ADR-005`/`REQ-059`) — reference-form list.
- `install/templates/{spec-product,spec-tech,design,plan}.md:3-6` reference-form headers — illustration.
- commented payload examples in `install/templates/{spec-product,spec-tech,members,interactions}.toml` — illustration.
- client-structure references: `.doctrine/spec/`, `.doctrine/adr/`, `.doctrine/slice/`, `.doctrine/memory/`, `adr-nnn.md`, `phase-01.md`, `handover.md`, `research.md` (`install/glossary.md:96,116,120`, `install/project-orientation.md:49`, `install/manifest.toml:33-44`, `install/mod.just:38,47`).
- fill-in-the-blank scaffolding: `install/harvest.md:55,62-63`.

### Rows

| file:line | class | disposition | resolution |
|---|---|---|---|
| claude-activation.md:1-2 | maintainer-note header | inline | published-address header shape; drop `ADR-005`/`ADR-019`/`install/claude-activation.md` |
| claude-activation.md:5 | one-clause rationale | inline | "the plugin/marketplace install path retired" |
| claude-activation.md:10 | one-clause rationale | inline | "`doctrine install` activates Claude Code by writing hooks" |
| claude-activation.md:56 | one-clause rationale | inline | "never a host absolute path in a tracked file" |
| claude-activation.md:60 | one-clause rationale | inline | "the same override `.mcp.json` has taken" |
| dispatch-mechanics.md:1-2 | maintainer-note header | inline | published-address header shape |
| dispatch-mechanics.md:6 | author note | drop | authoring provenance has no client meaning |
| dispatch-mechanics.md:138 | one-clause rationale | inline | "the mass-delete signature" |
| dispatch-mechanics.md:312 | one-clause rationale | inline | "the sanctioned replacement for a hand-edited trunk row" |
| dispatch-mechanics.md:369-370 | durable referent | inline | keep the four posture *descriptions*, drop the ids |
| harvest.md:1-2 | maintainer-note header | inline | published-address header shape |
| harvest.md:55,62-63 | fill-in-the-blank | leave | EX-2 |
| harvest.md:91 | rule-name | inline | "**Shipped-corpus conformance.**" |
| review-ledger.md:1-2 | maintainer-note header | inline | published-address header shape |
| review-ledger.md:9 | one-clause rationale | inline | "on the **RV kind** (`RV-NNN`)" |
| review-ledger.md:100 | one-clause rationale | inline | "selectors, seeded at `/slice` and `/design`" |
| review-ledger.md:163 | one-clause rationale | inline | "assertion, not a security boundary" |
| review-ledger.md:168 | provisional trial | inline | "under a provisional trial" |
| review-ledger.md:219 | one-clause rationale | inline | "a reading of the §6 ledger alone" |
| review-ledger.md:220 | live-open-issue reference | drop | names a defect in doctrine's own code a client does not have |
| review-ledger.md:182 | published address | leave | `design-prompts/reviewing.md` is a published address (D3) |
| routing-process.md:65-66 | reference-form list | leave | EX-2 |
| routing-process.md:70 | worked example | inline | obviously-synthetic example; clause no longer commits the error |
| routing-process.md:77 | one-clause rationale | inline | drop `ADR-019`; keep "published, not projected" |
| using-doctrine.md:1-2 | maintainer-note header | inline | published-address header shape |
| using-doctrine.md:52 | one-clause rationale | inline | "**knowledge_record = epistemic / governance records**" |
| using-doctrine.md:179 | one-clause rationale | inline | "reciprocity is derived" |
| using-doctrine.md:182 | private path + id | inline | "the relation module is the single source of truth" |
| glossary.md:8-36 | reference-form illustration | leave | EX-2 |
| glossary.md:44 | grey illustration | leave | the doc that defines reference forms; recorded, not reworded |
| glossary.md:96,116,120 | client-structure | leave | EX-2 |
| project-orientation.md:49 | client-structure | leave | EX-2 |
| manifest.toml:5 | one-clause rationale | inline | "the MINIMAL projection base" |
| manifest.toml:35 | one-clause rationale | inline | drop `SL-018` |
| manifest.toml:47 | one-clause rationale | inline | "Observations: authoritative records" |
| manifest.toml:56 | private path | inline | name the published address `using-doctrine.md` |
| manifest.toml:62 | durable referent | inline | keep the description, drop ids |
| manifest.toml:33-44 | client-structure | leave | projection patterns over the client's tree |
| doctrine.toml:3 | one-clause rationale | inline | "the single canonical home for project-local config" |
| doctrine.toml:11 | one-clause rationale | inline | "the conduct axis" |
| doctrine.toml.example:5 | one-clause rationale | inline | "the single canonical home" |
| doctrine.toml.example:9 | one-clause rationale | inline | "the conduct axis" |
| doctrine.toml.example:36 | one-clause rationale | inline | "the closure gate" |
| doctrine.toml.example:48 | one-clause rationale | inline | "how fresh entity ids are allocated" |
| doctrine.toml.example:83 | one-clause rationale | inline | "pi RPC mode" |
| doctrine.toml.example:89 | one-clause rationale | inline | "the buffered-trunk posture" |
| doctrine.toml.example:100 | one-clause rationale | inline | "the declared hard scope tier" |
| doctrine.toml.example:113 | one-clause rationale | inline | "the belt that read the rest of this list" |
| git-hooks/pre-commit:2 | header note | inline | "the coordination-worktree pre-commit backstop" |
| git-hooks/pre-commit:6 | one-clause rationale | inline | "the funnel-reversion signature" |
| git-hooks/pre-commit:15 | one-clause rationale | inline | "deliberate acts are not that signature" |
| git-hooks/pre-commit:24 | one-clause rationale | inline | "reopen the reversion hole" |
| agents/claude/dispatch-worker.md:36 | private path | inline | name the agent type, not `src/worktree/mod.rs` |
| design-prompts/exploring.toml:1 | header note | inline | "the runbook guarding the edge out of `exploring`" |
| design-prompts/exploring.toml:17 | one-clause rationale | inline | "once the override seam lands" |
| design-prompts/inquiring.toml:1 | header note | inline | "the runbook guarding the edge out of `inquiring`" |
| design-prompts/inquiring.toml:8 | one-clause rationale | inline | keep the lens/obligation prose, drop the ids |
| design-prompts/inquiring.toml:21 | durable referent | repoint | `reference/design-run-obligations.md` (VT-1) |
| design-prompts/reviewing.toml:1 | header note | inline | "the runbook guarding the edge out of `reviewing`" |
| design-prompts/reviewing.toml:15 | one-clause rationale | inline | "not bookkeeping error" |
| design-prompts/reviewing.toml:22 | one-clause rationale | inline | "deferred because its RENDER is unsketched" |
| design-prompts/drafting.toml:1 | header note | inline | "the runbook guarding the edge out of `drafting`" |
| design-prompts/drafting.toml:6 | one-clause rationale | inline | keep the lens/obligation prose, drop the ids |
| design-prompts/reviewing.md:100,105 | provisional trial | inline | drop the proposal number; keep "provisional" |
| design-prompts/delegation.md:19 | already fixed | leave | landed fixed by SL-264 `166f69a9f` |
| templates/knowledge-*.toml:22-30 (7 files) | one-clause rationale | inline | "(verb-written)" |
| templates/backlog.toml:14,16 | one-clause rationale | inline | "outbound-only"; "uniform `[[relation]]` rows" |
| templates/backlog-risk.toml:20,22 | one-clause rationale | inline | as backlog.toml |
| templates/slice.toml:14,16 | one-clause rationale | inline | "uniform `[[relation]]` rows"; "the dep/seq axes" |
| templates/slice.toml:18,19 | command example | inline | obviously-synthetic placeholders (`SPEC-NNN`, `ADR-NNN`) |
| templates/review.toml:4,12 | one-clause rationale | inline | "status is DERIVED"; "the outbound `reviews` edge" |
| templates/rec.toml:4,12 | one-clause rationale | inline | "one reconciliation act"; drop `(Slice B)` |
| templates/revision.toml:9 | one-clause rationale | inline | "a first-class change-axis entity" |
| templates/revision.toml:20 | commented payload example | leave | illustration (D2) |
| templates/rec.md:3 | one-clause rationale | inline | "Reconciliation record" |
| templates/revision.md:3 | one-clause rationale | inline | "Revision — a pending revise-intent" |
| templates/review.md:3 | one-clause rationale | inline | "Adversarial-review ledger" |
| templates/{spec-product,spec-tech,members,interactions}.toml | commented payload example | leave | EX-2 |
| templates/{spec-product,spec-tech,design,plan}.md | reference-form header | leave | EX-2 |

### Additions beyond ISS-309's regex (locator gap — a finding)

**Finding.** `ISS-309`'s id regex (`SL|ADR|RV|REV|PRD|SPEC|IMP|ISS|CHR|DEC|RFC|REQ|POL|STD|IDE|QUE|RSK|ASM|REC|CM`, 3-digit) **omits** the knowledge-kind prefixes (`CON`, `EVD`, `HYP`, `CPT`), membership labels (`FR-`, `NF-`), and doc-local design ids (`D-C8`, `D-Q3`, …). A second full-prefix grep (`\b[A-Z]{2,5}-[0-9]{2,3}\b`) found five further live sites. The ledger premise ("classification plus edit, not discovery") held, but the count was under-stated. Durable input for `ISS-309` part 2 / `QUE-227`.

| file:line | class | disposition | resolution |
|---|---|---|---|
| review-ledger.md:184 | repo-private constraint id | inline | "Nothing validates it." (was `CON-006`) |
| design-prompts/reviewing.md:182 | repo-private constraint id | inline | "Each unenforced clause is enumerated…" (was `CON-006`) |
| review-ledger.md:124,210-220,223 | doc-local design ids | inline | drop `D-C9b`/`D-C9a`/`D-C8`; the rule's substance is stated in place |
| templates/review.toml:4,16 | doc-local design ids | inline | drop `(ADR-007 D-C8)` / `(D-C8)` |
| templates/rec.toml:4-5 | doc-local design id | inline | drop `(SPEC-002 D-Q3)` |
| templates/rec.toml:12 | internal plan term | inline | drop `(Slice B)` |
| manifest.toml:5,14,20,29,34 | repo-private ids + member labels | inline | drop `SL-227`/`FR-007`/`FR-008`/`D8`/`slice-004`/`slice-005`/`PHASE-06` |
| manifest.toml:8 | private source path | inline | "the engine's `BASE_BACKINGS`" (was `publication.rs`) |
| manifest.toml:15 | private source path | inline | "the entity engine's `materialise*`" (was `entity.rs`) |
| doctrine.toml.example:56 | doc-local design id | inline | "the y/N prompt" (was `D8`) |
| git-hooks/pre-commit:2 | header + private design ref | inline | drop `SL-228 PHASE-02`, `design §7` |
| templates/plan.toml:38 | commented payload example | leave | `src/foo.rs` is obviously synthetic |
| harvest.md:54,58,61,62 | fill-in-the-blank | leave | EX-2 (line numbers shifted by the header edit) |
| glossary.md:12,28-30 | reference-form illustration | leave | kind/label table |

### Verification (EX-1, EX-2, EX-7, VA-1, VA-2)

- **Locators, with positive controls** (`DEC-314`). Post-sweep `grep -rnE '\b(SL|ADR|RV|…)-[0-9]{3}\b' install/` returns only do-not-sweep sites; the same regex finds ids in `.doctrine/adr/019/adr-019.md`. Full-prefix grep `\b[A-Z]{2,5}-[0-9]{2,3}\b` likewise. Path grep `(src/*.rs|install/*.md|slice-[0-9]{3})` returns only `templates/plan.toml:38` (synthetic). `grep -rn 'design-prompts' install/` returns only the pre-existing allowlisted `review-ledger.md:181`.
- **Repoint resolves** (`VA-1`): `doctrine library show reference/design-run-obligations.md` streams the doc (evidence the `inquiring.toml` repoint target is real, `DEC-314`).
- **Illustrations provably untouched** (`VA-2`): `git diff` shows no hunk touching `routing-process.md:65-66`, `harvest.md`'s fill-in-the-blank block, or `revision.toml:20`; `glossary.md` is unchanged entirely.
- **Gates** (`EX-7`): `cargo build` clean; `doctrine publication validate` ok; `cargo test --test e2e_claude_install` **13/13** (incl. `design_prompts_have_no_consumer_outside_the_design_run` with `store_allowlist` byte-unchanged, `no_shipped_guidance_advertises_slice_design_as_canonical`, and both command-acceptance tests).
- **No golden moved incidentally** (`EX-6`): manifest entry count still 98 (PHASE-01's two rows); no test pins the install asset set beyond the frozen e2e suite, which is green.

## PHASE-03 disposition ledger — shipped memory and skills

Evidence for PHASE-03 `EX-1` (`design.md` sec-8: one row per candidate,
`file:line | class | disposition | resolution`). Built **before the bulk edit**
(`EN-1`, `EN-2`).

### Locators, with positive controls (`DEC-314`)

- id grep (full prefix set — `mem.pattern.install.shipped-corpus-citation-grep-prefix-set`):
  `grep -rnE '\b[A-Z]{2,5}-[0-9]{2,3}\b' memory/ plugins/` → **memory/ 16 lines / 9 files** (+1 path-only, +2 doc-path-only), **plugins/ 69 lines / 15 files** (+ `elicit/SKILL.md` = 16 files).
- ISS-309's narrower regex over the same two trees → 81 lines: it misses `FR-001`
  (`spec-product/SKILL.md:129`), `SL-42`/`QUE-7` (`knowledge/SKILL.md:60`),
  `D-C8` (`close/SKILL.md:48`), `CON-006` (`plan/SKILL.md:52`).
- **path** grep `(src/[A-Za-z0-9_./-]+\.rs|install/[A-Za-z0-9_./-]+\.md|doc/[A-Za-z0-9_./-]+\.md|slice-[0-9]{3})`
  → `memory/` 6 lines / 5 files (3 `src/`+`install/`, 3 `doc/`), `plugins/` 0.
- **doc-local designs (`D-NN`, `D-CNb`, `F-N`, `S-N`, `INV-N`, `C-V`, `OQ-N`,
  `§N.M`)** — a fourth dimension the id regex cannot see at all: 5 lines in
  `memory/`, ~30 sites across 8 skill files. Found by a widened pass
  (`\bF[0-9]+\b|\bC-V\b|\bINV-[0-9]+\b|\bD[0-9]+\b|\bS[0-9]\b`) **after** the
  first bulk pass — see "Additions", and `A2`'s premise is confirmed a third time.
- **Positive controls**: the id regex finds ids in `.doctrine/adr/019/adr-019.md`;
  the path regex finds `src/relation.rs` in `.doctrine/slice/267/design.md`; the
  dangling-key search finds an injected fake key and not a present one (T7).
- `grep -rn 'design-prompts' memory/ plugins/` → **zero** (R4 binds as *do not
  introduce*). Control: the same grep finds `install/review-ledger.md:181`.
- Frontmatter `description:` fields and the two sibling skill files
  (`worktree/NOTICE.md`, `rigour/reference.md`) carry **no** candidate.

### EN-2 — the two rebuild paths, exercised on one file each (R5, A4)

Both demonstrated **before** the bulk edit, each on a real site (no wasted edit).

**(a) memory path** — `memory/mem_019ec92b301478e0ab67f4e4eb534fb9/memory.md:3`
(`Revisions (REV kind, ADR-013)` → `Revisions (the **REV kind**)`), then
`cargo build` → `doctrine memory sync -y` (dry-run first: `1 changed`) →
`doctrine install`. Result: `diff -r` of the pre-phase copy against
`.doctrine/memory/shipped/` shows **exactly one file, one hunk, three lines** — the
edit. `doctrine install` is *not* what materialises the corpus; `memory sync` is.

**(b) plugins path** — `plugins/doctrine/skills/elicit/SKILL.md:8` (`(RFC-019)`
removed), then `cargo build` **without** touching `src/install.rs` →
`doctrine install -a claude -s elicit -y` → `diff` of
`.doctrine/skills/elicit/SKILL.md` shows **exactly the edit**.

**Finding (corrects two memories at once).** The `plugins/` root **does** re-embed
on a plain `cargo build` — no `touch src/install.rs` needed. So
`mem.pattern.build.rust-embed-no-rerun`'s "not re-probed: `plugins/`" caveat is
now resolved by probe (it covers the `install/templates` root, and explicitly
left `memory/`+`plugins/` open), **and** `mem_019eae55811f7412b11559068fe8a279`'s
"Skill content refresh = … + touch src/install.rs" over-states the touch as a
precondition rather than an escape hatch. The touch remains harmless and is kept
as belt-and-suspenders in the bulk rebuilds.

**Collateral check.** `doctrine install` wrote `.claude/settings.json`,
`.claude/agents/dispatch-worker.md`, `.claude/skills/elicit` — all under
`.claude/*`, which `.gitignore:4` covers. `git status --porcelain` shows **only**
the two corpus files dirty. No tracked path moved.

### Do-not-sweep classes re-checked against these two sub-corpora (EN-1)

**ISS-309's claim is wrong and is corrected here.** It asserts "all genuine, none
illustrations" under `plugins/` and "none" in the 11 memory files. Both are
wrong: five illustration/fill-in-the-blank sites exist under `plugins/` (**and
the memory inventory is 13 masters, not 11** — see "Additions"). The five
under `plugins/`:

- `spec-product/SKILL.md:129` — `- FR-001 — …` is the *reference-form
  illustration* (it teaches what a requirements heading looks like).
- `spec-product/SKILL.md:220` — `OQ-001` worked example; doc-local enumerations
  are bare by convention, and the shape is obviously synthetic.
- `knowledge/SKILL.md:60` — `doctrine needs SL-42 QUE-7`: non-zero-padded ⇒
  obviously synthetic (STD-002 forbids the real corpus from minting it).
- `plan/SKILL.md:71` — `test_file = "src/plan.rs"`: synthetic path in a TOML
  example.
- `slice/SKILL.md:39` — `src/foo/**`, `src/bar.rs`: synthetic selector example.

Client-structure references under `plugins/` (the *client's own* tree — leave):
`slice/SKILL.md:10,15`, `preflight/SKILL.md:84`, `research/SKILL.md:13`,
`record-memory/SKILL.md:33`, `inquisition/SKILL.md:58` — all `.doctrine/spec/`,
`.doctrine/adr/`, `.doctrine/slice/`, `.doctrine/memory/items/`.

Client-structure references under `memory/` (leave): the `.doctrine/slice/`,
`.doctrine/adr/`, `.doctrine/spec/{product,tech}/`, `.doctrine/memory/items/`,
`.doctrine/revision/` directory conventions in `mem_019e9a11cda27db…`,
`mem_019e9a1234ff7e6…`, `mem_019e9a12c5a97d0…`, `mem_019ec92b300d7530…`,
`mem_019ec92b300f7d43…`, `mem_019ec92b10067842…`, `mem_019e9a11e8337613…`,
`mem_019e9a1244d37f7…` (`.doctrine/slice/nnn/`), and the `memory.toml`
`paths`/`globs` scope matchers (D3).

### Rows — `memory/`

| file:line | class | disposition | resolution |
|---|---|---|---|
| mem_019ec92b301478e0ab67f4e4eb534fb9/memory.md:3 | one-clause rationale | inline | **DONE (EN-2 probe)** — "Revisions (the **REV kind**)" |
| mem_88193c2859d72f043ef83a97a5952a96/memory.md:72-73 | private spec as point of truth | inline | **EX-2** — the cascade rule stands in the body; drop the `spec-023.md` path, `SPEC-023` and the `install/hymns/README.md` path |
| mem_019f2b93f5e178009191711f607caff6/memory.md:35 | dangling wikilink | drop | **EX-3** — `[[mem.signpost.doctrine.dispatch-claude-arm-wrong-base]]` names a key not in the corpus; the clause restates the trap, so delete the pointer |
| mem_019f2b93f5e178009191711f607caff6/memory.md:26 | one-clause rationale | inline | "the global class"; "tiering" — drop `ADR-002`/`ADR-005` |
| mem_019f2b93f5e178009191711f607caff6/memory.md:28 | author note | drop | `CHR-036` is the chore that produced this — no client meaning |
| mem_019f2b93f5e178009191711f607caff6/memory.md:31-32 | durable referent | inline | keep the four posture *descriptions*, drop `ADR-006`/`008`/`011`/`012` |
| mem_019e9a11cda27db19c0c75bafa453d5d/memory.md:19 | one-clause rationale | inline | "revision change-axis records" — drop `REV kind, ADR-013` |
| mem_019e9a11cda27db19c0c75bafa453d5d/memory.md:37 | one-clause rationale | inline | "docs (the pull tier)" |
| mem_019ec92b10037850817507044f0f99ef/memory.md:25 | one-clause rationale | inline | "shipped reference docs (the pull tier)" |
| mem_019ed279444273b093816ccbb8e5da64/memory.md:4 | one-clause rationale | inline | "review (the **RV kind**)" |
| mem_019ec92b30127db0aac7eb7badb1cbf2/memory.md:13 | one-clause rationale | inline | "**review ledger** (the RV kind)" |
| mem_019ec92b30127db0aac7eb7badb1cbf2/memory.md:28 | doc-local design id | inline | drop `(D-C9b)` — "The close gate refuses …" |
| mem_019e9a11e8337613bdf8e96f75a9e6b2/memory.md:24 | one-clause rationale | inline | "(the closure seam)" |
| mem_019e9a12789f7ac39c0841f4d976503b/memory.md:21 | one-clause rationale | inline | "(the RV kind) … a reconciliation gate" |
| mem_019e9a1244d37f72a9b7246d2c976ef7/memory.md:12 | private source path | inline | drop `(\`src/git.rs\`)` — "behind a thin seam" |
| mem_019e9a1244d37f72a9b7246d2c976ef7/memory.md:26-28 | private doc paths as point of truth | repoint | `doc/entity-model.md` + `doc/relation-index.md` (neither exists) → `reference/using-doctrine.md`; drop "The code lives under `src/`" |
| mem_019e9a1234ff7e619d865592b0042cb9/memory.md:25-27 | private doc path as point of truth | repoint | **file not in ISS-309** — `doc/entity-model.md` → `reference/using-doctrine.md` ("Storage tiers — what goes where") |
| mem_019e9a12560d7972b29124e09f4de704/memory.md:26-27 | private doc path as point of truth | drop | **file not in ISS-309** — `doc/memory-spec.md` dropped; the `doctrine memory --help` surface kept |
| mem_019f176f71537d12b1b09826a003a602/memory.md:10 | private path + fn name | inline | "the predicate demands ALL THREE" |
| mem_019f176f71537d12b1b09826a003a602/memory.md:36-39 | historical slice/revision ids | inline | "a slice modified a requirement (via a revision) and attested another"; drop `SL-165`/`REQ-316`/`REV-014`/`REQ-317`/`REC-093`/`REC-094`/`SL-064` |

### Rows — `plugins/`

| file:line | class | disposition | resolution |
|---|---|---|---|
| spec-product/SKILL.md:150-151,208,229 | misinstruction | inline | **EX-4** — restate so the **client's own** PRD is the canonical shape; drop `PRD-001` |
| spec-tech/SKILL.md:56-61 | misinstruction | inline | **EX-4** — the three C4 levels as level *descriptions*; drop `SPEC-003`/`004`/`005` |
| spec-tech/SKILL.md:40,48 | one-clause rationale | inline | drop `PRD-012`; "both home-grown and forward-intent" |
| spec-product/SKILL.md:129 | reference-form illustration | leave | A3 — teaches the requirements-heading shape |
| spec-product/SKILL.md:220 | doc-local example | leave | A3 — bare local enumeration by convention |
| reconcile/SKILL.md:9,37,61-62,175-179,199,224,227-228,231,244 | worked-example narrative | inline | the `(SL-080)` narrative's pedagogy survives without private ids → synthetic shapes; `ADR-003 §7/§11`, `ADR-009 §1`, `ADR-006 §D5`, `REQ-077`, `REV-011`, `RV-042`, `SL-080`, `SL-056` |
| close/SKILL.md:40,46,49,94,108,111,132,135-136 | private refs | inline | `ADR-009 §1`→"the back-edge"; `ADR-007`→"the review kind"; `ISS-314` (a **live open issue here**)→drop; `ISS-030`, `SL-121`, `SPEC-022`, `SL-190`, `SL-211`, `IMP-236`→descriptions |
| dispatch/SKILL.md:16,19,23,125,130,157,176 | private refs | inline | drop `ADR-008`/`ADR-012` labels, `ISS-234`, `IMP-101`, `D6`/`IMP-171`, `SL-170 S3/S6`, `ISS-052` — keep the mechanism each names |
| dispatch/SKILL.md:15 | client-structure + placeholder | leave | `.dispatch/SL-<n>` — the client's own dir and id |
| worktree/SKILL.md:45,51,62,75,123,240 | private refs | inline | `SL-031 §5.2`, `SL-064`, `ADR-012`, `SL-056`, `SL-156`, `SL-008` — keep "a THIRD path", "the sole write target", the traits |
| audit/SKILL.md:11,63,84,135-136 | private refs | inline | `ADR-007`; `RFC-004`/`SL-147`→"the old seeding was retired"; `ADR-006 §D5`/`REQ-077`→the generic dispositions |
| plan/SKILL.md:43,52,66 | private refs + doc-local | inline | `RFC-026 P10 trial`→"provisional"; `CON-006`→"an unenforced clause"; `IMP-209`→the rule |
| plan/SKILL.md:85 | rule name | inline | `POL-002` names the shipping rule; keep in prose if it reads as prose, else drop the label |
| plan/SKILL.md:71 | synthetic | leave | A3 |
| execute/SKILL.md:43,82-83 | example ids | inline | `feat(SL-009)` → `feat(SL-NNN)`; `slice/SL-029-…`/`.worktrees/SL-029` → synthetic shapes |
| record-memory/SKILL.md:23,106 | private refs | inline | `SL-008 D6` → the behaviour it describes |
| record-memory/SKILL.md:33 | client-structure | leave | `.doctrine/memory/items/` |
| inquisition/SKILL.md:21,71 | private refs | inline | `ADR-007`; `SL-147` |
| inquisition/SKILL.md:58 | client-structure | leave | `.doctrine/adr/`, `.doctrine/spec/*` |
| code-review/SKILL.md:92,136 | private refs | inline | `ADR-007`; `SL-147` |
| handover/SKILL.md:115 | private ref | inline | drop `SL-170 S6` / `S1` |
| elicit/SKILL.md:8 | one-clause rationale | inline | **DONE (EN-2 probe)** — "the queue/curator split" |
| elicit/SKILL.md:14 | doc-local design id | inline | drop `(D18)` |
| worktree/SKILL.md:25,45,62,75,82,100,116-117,123,154,181,186,192,203,240 | doc-local + private refs | inline | see the additional class below — `OQ-1`, `D1`/`D5`/`D9`/`F1`, `F2`, `F7`, `C-V`, `SL-031 §5.2`, `SL-064`, `SL-056`, `SL-156`, `SL-008`; **anchors updated** (`#provisioning`) |
| reconcile/SKILL.md:13,40,51,199,224,227-228,231,244 | doc-local design ids | inline | `D2`/`D9`/`D3` dropped; `F2`→`finding F-2`, `RV-042`→`RV-NNN`, `REV-011`→`REV-NNN`, `SL-056`→description, `ADR-009`→the back-edge |
| audit/SKILL.md:27,105,125,144,167 | doc-local design ids | inline | `F-2`/`D3`/`D-C9b` dropped |
| close/SKILL.md:47-48 | doc-local design ids | inline | `D-C9b`/`D7`/`D-C8` dropped, the semantics kept |
| dispatch/SKILL.md:50,169 | doc-local invariant ids | inline | `INV-1`/`INV-6` labels dropped |
| execute/SKILL.md:32,59,106 | doc-local design refs | inline | `design D5`/`F-6`/`§8.1` dropped |
| plan/SKILL.md:64,91 | doc-local stage ids | inline | `S3`/`S6` dropped — "the VT gate" |
| code-review/SKILL.md:126 | private design section | inline | drop `design §5.5`; keep the `review-ledger.md §1` (a published address) |
| record-memory/SKILL.md:23,106 | private refs | inline | drop `SL-008 D6`; keep the behaviour it describes |
| knowledge/SKILL.md:60 | synthetic | leave | A3 |
| slice/SKILL.md:10,15 | client-structure | leave | `.doctrine/spec/` |
| slice/SKILL.md:39 | synthetic | leave | A3 |
| preflight/SKILL.md:84 | client-structure | leave | `.doctrine/spec/tech/` |
| research/SKILL.md:13 | client-structure | leave | `.doctrine/slice/NNN/research/` |

### Additions / corrections beyond ISS-309 (EN-1 finding)

**Finding 1 — the `plugins/` inventory is stale by growth.** "14 skill files, 74
sites" was swept 2026-08-03; the shipped skill tree now holds **35 `SKILL.md`**
and the live full-prefix grep finds **69 id lines across 15 files** (+ the
`elicit` probe). Its "none illustrations under `plugins/`" claim is **wrong**
(five sites, above). Durable input for `ISS-309` part 2 / `QUE-227`, alongside
PHASE-02's prefix-set finding.

**Finding 2 — the memory inventory is 13 masters, not 11.** ISS-309's "11 of 35"
counts only the id-grep class. A fourth class — a **private `doc/` path cited as
a point of truth** — puts two more masters in scope (`storage-model`,
`memory-model`; `doc/` does not exist in this repo at all, so these are dangling
*and* authoritative-sounding, the same failure shape as `hymn-cascade`'s SPEC-023
line). Its "none [illustrations] in the 11 memory files" claim happens to hold
for the three do-not-sweep classes, but the denominator was wrong. **This is the
first time in the slice that a claim of mine (`EN-1` confirmed the 11 holds) was
falsified by a later pass** — recorded as the phase's own lesson, not hidden.

**Finding 3 — a fourth locator dimension: doc-local design ids.** `D-NN`,
`D-CNb`, `F-N`, `S-N`, `INV-N`, `C-V`, `§N.M` are invisible to *both* the
ISS-309 regex and the full-prefix `\b[A-Z]{2,5}-[0-9]{2,3}\b` set — they have no
matching prefix shape. ~35 sites across 8 skill files and 1 memory (`D-C9b`).
They are the references PHASE-02 already swept under the same heading
(`D-C8`/`D-Q3`/`Slice B`), so the class is established; what is new is that the
*skill* corpus carries ~7× as many as `install/` did. Found only after a second,
widened pass — **the locator came third, which is why `DEC-314` is a discipline
and not a formality.**

**A near-miss worth naming.** `worktree/SKILL.md` used `D-NN` as *section labels*
(`## Provisioning (D9)`) and a **cross-reference anchor** (`#provisioning-d9`).
Dropping the label silently broke the anchor; it was caught by grepping the
whole file for `#provisioning` after the edit, not by the ledger. Recorded as a
pattern rather than a one-off: **a doc-local id may be load-bearing as an
anchor**, so a sweep must re-resolve every intra-file link it touches.

### Verification (EX-1..EX-6, VA-1..VA-3)

- **`EX-1` — one row per candidate.** Both tables above; each `repoint`'s
  resolution is a published address (`doctrine library show reference/model-band.md`,
  `…/reference/using-doctrine.md` both stream), each `inline` names the post-edit
  line, each `leave` names its class.
- **`EX-2` — hymn-cascade no longer names a private spec as truth.**
  `grep -n 'spec-023\|SPEC-023\|install/hymns' memory/mem_88193c2859d72f043ef83a97a5952a96/memory.md`
  → **zero**; the cascade rule stands in the body, and the pointer is
  `reference/model-band.md`.
- **`EX-3` — zero dangling wikilinks.** `grep -rhoE '\[\[mem\.[a-zA-Z0-9._-]+\]\]' memory/ plugins/`
  → keys, `comm -23` against `ls memory/` → **empty**. Controls: a present key
  (`mem.signpost.doctrine.file-map`) is *not* reported; an injected fake key **is**.
  The `dispatch` signpost's `…dispatch-claude-arm-wrong-base` pointer is gone.
- **`EX-4` — the two misinstruction sites.** `spec-product/SKILL.md` now says
  "Mirror the canonical shape of **your own repo's** product spec";
  `spec-tech/SKILL.md` teaches `context`/`container`/`component` as level
  descriptions and names no `SPEC-NNN`. Both re-read **in the installed tree**
  (`.doctrine/skills/…`) — see `VA-3`.
- **`EX-5` — re-embedded and materialised, with the diff as evidence.**
  `diff -rq` of the pre-phase snapshots against the live trees returns **exactly
  13 `memory/shipped/*.md`** and **exactly 14 `.doctrine/skills/*/SKILL.md`** — no
  additions, no removals, no file outside the edited set. Both trees are
  gitignored, so the file lists + the mechanism (T1) are the re-derivable
  evidence; the pre-phase snapshots are at `/tmp/sl267-pre/` for this session.
- **`EX-6` — green.** `doctrine check gate` exit **0** (124 `test result: ok`,
  zero failures); `doctrine doctor` **51** findings (byte-identical to PHASE-02's
  pre-existing count); `doctrine publication validate` **98** `ok` entries
  (unchanged); `cargo test --test e2e_claude_install` **13/13** — including
  `design_prompts_have_no_consumer_outside_the_design_run` with `store_allowlist`
  **byte-unchanged** (`git diff --stat tests/` empty) and both `VT-1` keywords
  (`install_wires_skills_agent_and_hooks_directly`,
  `sealed_design_hymn_and_four_fragments_ship_installed`).
- **`VA-1` — replacement resolving, whole-file reads.** Every repoint proven with
  `doctrine library show`; the changed regions were re-read from the
  *materialised* copies (memory) and the *installed* tree (skills), not the
  sources — hymn-cascade, dispatch signpost, reconcile narrative + Outcome,
  worktree headings/anchors, spec-product, spec-tech.
- **`VA-2` — post-sweep corpus clean, illustrations untouched.** `memory/` id
  grep → **zero**; `plugins/` id grep → only the 3 do-not-sweep sites; doc-local
  grep → only `OQ-001` (a doc-local enumeration, admissible); path grep → only
  synthetic placeholders and the two `.toml` scope records (D3);
  `grep -rn design-prompts memory/ plugins/` → **zero**.
- **`VA-3` — read in the installed tree.** `.doctrine/skills/spec-product/SKILL.md`
  and `.doctrine/skills/spec-tech/SKILL.md` read correctly for a client with no
  doctrine corpus: no `PRD-001`, no `SPEC-003/004/005`, the referent in both is
  the client's own record.
- **No golden moved incidentally.** `publication validate` still 98 entries;
  `tests/**` untouched; `doctrine doctor` unchanged.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-26 · PHASE-03 · bfd0d466b

### Produced
- design locked — SL-267 under run dr-01a0d8ff-b311 (rev 35); `design.md` 9 sections (commits 9750594e4, c17d9229a, 53add8888).
- plan authored — 6 phases, rule-first; runtime sheets materialised (commit e4fa00d7a).
- PHASE-01 done — the rule and its two published homes: ADR-024 (accepted; `related` ADR-005/ADR-019; DEC-127 carries the reverse `concerns` edge), `reference/shipped-corpus-authoring.md`, `reference/design-run-obligations.md`, and their two `publication/manifest.toml` rows (commit 187f05f91).
- PHASE-02 done — the `install/` sweep: 33 files, every repo-private id/path inlined, dropped or repointed; the `inquiring.toml` repoint to `reference/design-run-obligations.md`; the `doctrine.toml.example` per-knob whys inlined; the `routing-process.md` boot pointer + worked example. Disposition ledger + evidence in this file (commit 599dfb5c8).
- PHASE-03 done — the `memory/` + `plugins/` sweep: **13 masters** (not ISS-309's 11), **14 skill files**; every repo-private entity id, source/spec path and doc-local design id inlined, dropped or repointed to a published address; the two misinstruction sites restated (`spec-product` ↔ `PRD-001`, `spec-tech` ↔ `SPEC-003/004/005`); the dangling `dispatch` wikilink removed. Disposition ledger + materialisation diffs in this file (commits 13e4fcbf1, b8d622a4c).
- minted: mem.pattern.build.plugins-embed-auto-rebuild; three new sections appended to mem.pattern.install.shipped-corpus-citation-grep-prefix-set.
- minted: ADR-024; DEC-311..DEC-316; CHR-081; RV-391; mem.pattern.shipped-corpus.delivery-copy-cannot-cite-its-owner.
- ledger corrections: IMP-484 (observation gap re-verified false, swept in the design, the item and slice-267.md); CHR-080 and ISS-309 carried as the accuracy and citation ledgers.
- gates: `doctrine check gate` exit 0 at PHASE-01 (187f05f91) and PHASE-02 (599dfb5c8); publication validate ok; e2e_claude_install 13/13 both times (PHASE-02's `store_allowlist` byte-unchanged).

### Learned
- mem.pattern.shipped-corpus.delivery-copy-cannot-cite-its-owner — a shipped delivery copy cannot cite its governance owner; the link runs governance → published address, one way.
- mem.pattern.design-run.review-disposition-route-and-vocab — a severe finding's disposition needs route *and* vocab; the tool accepts the route alone.
- observation 01a0d908-973f — `library tree` groups a template under a `reference/` heading but its address is `templates/<name>`.
- **ISS-309's id regex is incomplete** — it omits `CON`/`EVD`/`HYP`/`CPT`, `FR-`/`NF-` membership labels, and doc-local `D-*` design ids. The `install/` site count was under-stated; PHASE-02 swept five further live sites (ledger § "Additions"). Durable input for `ISS-309` part 2 / `QUE-227`.
- **The maintainer-note header has a settled fix** — name the doc's own published address, keep "published, not projected", drop the repo-private id and the `install/<file>.md` source path (five docs rewritten to the PHASE-01 doc shape).
- **mem.pattern.build.plugins-embed-auto-rebuild** — the `plugins/` RustEmbed root re-embeds on a **plain `cargo build`**; `touch src/install.rs` is an escape hatch, not a precondition (`mem.pattern.build.rust-embed-no-rerun`'s "not re-probed: plugins/" caveat is now closed by probe). What *is* mandatory is the final `install -s <id>`: the re-embed moves the **binary**, only `install` moves the **installed tree**. Also probed: `memory/` re-materialises through `doctrine memory sync`, a different path from the cargo embed.
- **ISS-309 is a floor on three axes, not one.** PHASE-02 found the prefix-set gap; PHASE-03 found the skill corpus has *grown* past the item (14→16 files, 74→~104 sites, 35 `SKILL.md` on disk) **and** that the item omits an entire fourth locator dimension — no-hyphen doc-local design ids (`D1`/`F2`/`S3`/`INV-6`/`C-V`/`§8.1`, ~35 sites across 8 skill files) — **and** that its memory count is 11 where the live grep finds 13 (a private `doc/` path cited as a point of truth is a citation too). All three are appended to `mem.pattern.install.shipped-corpus-citation-grep-prefix-set`.
- **A doc-local id can be load-bearing as an anchor.** `worktree/SKILL.md` labelled its sections `(D9)` and cross-referenced `[Provisioning](#provisioning-d9)`; dropping the label silently broke the link. A sweep must re-resolve every intra-file `#anchor` it touches — caught by grep, not by the ledger.
- **A `.toml` scope field is data, not prose.** `paths`/`globs` in a shipped `memory.toml` are matchers against the *client's* tree, so `.doctrine/**` entries are correct and a private entry (`memory/`, `src/`, `doc/*.md`) is a dead matcher — recorded, not swept (D3).

### Open
- QUE-227 — drift-gate seam and the duplicate POL-002 rule (ISS-309 part 2); the only durable defence against re-drift.
- CHR-081 — consolidate the two local memories restating the grounding rule (out of scope; local-memory health is a non-goal corpus).
- ISS-215 — boot-index defect; CHR-036 — distilling project-local memories (both out of scope).
- **Audit flag (PHASE-02 D3)** — `install/review-ledger.md`'s `design-prompts/reviewing.md` pointer is a *published address* (conforming form 2) but sits inside the e2e `store_allowlist`; left `leave` because conforming it would need an out-of-selector `tests/**` edit. Weigh at audit.
- **Audit flag (PHASE-03 D3)** — five shipped `memory.toml` scope entries name doctrine-private matchers: `memory/` + `doc/memory-spec.md` (`mem_019e9a12560d7972b29124e09f4de704`), `src/` + `doc/entity-model.md` + `doc/relation-index.md` (`mem_019e9a1244d37f72a9b7246d2c976ef7`), `install/hymns/` (`mem_88193c2859d72f043ef83a97a5952a96`). Left `leave`: a scope field is a retrieval matcher, not a citation, and editing one changes behaviour outside axis A. Weigh at audit — this is the one class the sweep *deferred by design*.
- **Audit flag (PHASE-03 A2)** — the plan's PHASE-03 objective states "11 of the 35 shipped memory masters, and 14 skill files carrying 74 sites". All three numbers were low. Not a scope breach (the selectors cover the whole sub-corpus and `EX-1` says *per candidate*), but the plan's terrain description is now known-inaccurate for the audit's conformance read.
