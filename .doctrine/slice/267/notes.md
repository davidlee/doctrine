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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-26 · PHASE-02 · 599dfb5c8

### Produced
- design locked — SL-267 under run dr-01a0d8ff-b311 (rev 35); `design.md` 9 sections (commits 9750594e4, c17d9229a, 53add8888).
- plan authored — 6 phases, rule-first; runtime sheets materialised (commit e4fa00d7a).
- PHASE-01 done — the rule and its two published homes: ADR-024 (accepted; `related` ADR-005/ADR-019; DEC-127 carries the reverse `concerns` edge), `reference/shipped-corpus-authoring.md`, `reference/design-run-obligations.md`, and their two `publication/manifest.toml` rows (commit 187f05f91).
- PHASE-02 done — the `install/` sweep: 33 files, every repo-private id/path inlined, dropped or repointed; the `inquiring.toml` repoint to `reference/design-run-obligations.md`; the `doctrine.toml.example` per-knob whys inlined; the `routing-process.md` boot pointer + worked example. Disposition ledger + evidence in this file (commit 599dfb5c8).
- minted: ADR-024; DEC-311..DEC-316; CHR-081; RV-391; mem.pattern.shipped-corpus.delivery-copy-cannot-cite-its-owner.
- ledger corrections: IMP-484 (observation gap re-verified false, swept in the design, the item and slice-267.md); CHR-080 and ISS-309 carried as the accuracy and citation ledgers.
- gates: `doctrine check gate` exit 0 at PHASE-01 (187f05f91) and PHASE-02 (599dfb5c8); publication validate ok; e2e_claude_install 13/13 both times (PHASE-02's `store_allowlist` byte-unchanged).

### Learned
- mem.pattern.shipped-corpus.delivery-copy-cannot-cite-its-owner — a shipped delivery copy cannot cite its governance owner; the link runs governance → published address, one way.
- mem.pattern.design-run.review-disposition-route-and-vocab — a severe finding's disposition needs route *and* vocab; the tool accepts the route alone.
- observation 01a0d908-973f — `library tree` groups a template under a `reference/` heading but its address is `templates/<name>`.
- **ISS-309's id regex is incomplete** — it omits `CON`/`EVD`/`HYP`/`CPT`, `FR-`/`NF-` membership labels, and doc-local `D-*` design ids. The `install/` site count was under-stated; PHASE-02 swept five further live sites (ledger § "Additions"). Durable input for `ISS-309` part 2 / `QUE-227`.
- **The maintainer-note header has a settled fix** — name the doc's own published address, keep "published, not projected", drop the repo-private id and the `install/<file>.md` source path (five docs rewritten to the PHASE-01 doc shape).

### Open
- QUE-227 — drift-gate seam and the duplicate POL-002 rule (ISS-309 part 2); the only durable defence against re-drift.
- CHR-081 — consolidate the two local memories restating the grounding rule (out of scope; local-memory health is a non-goal corpus).
- ISS-215 — boot-index defect; CHR-036 — distilling project-local memories (both out of scope).
- **Audit flag (PHASE-02 D3)** — `install/review-ledger.md`'s `design-prompts/reviewing.md` pointer is a *published address* (conforming form 2) but sits inside the e2e `store_allowlist`; left `leave` because conforming it would need an out-of-selector `tests/**` edit. Weigh at audit.
