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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-26 · design (locked) · a51c614bb

### Produced
- design locked — SL-267 under run dr-01a0d8ff-b311 (rev 35); `design.md` 9 sections (commits 9750594e4, c17d9229a, 53add8888).
- minted: DEC-311..DEC-316 (the six design decisions, accepted at lock); CHR-081 (local-memory consolidation follow-up); RV-391 (design review; 12 findings, 11 verified, F-7 deferred to plan).
- ledger corrections: IMP-484 (observation gap re-verified false, swept in the design, the item and slice-267.md); CHR-080 and ISS-309 carried as the accuracy and citation ledgers.

### Learned
- mem.pattern.design-run.review-disposition-route-and-vocab — a severe finding's disposition needs route *and* vocab; the tool accepts the route alone.
- observation 01a0d908-973f — `library tree` groups a template under a `reference/` heading but its address is `templates/<name>`.

### Open
- QUE-227 — drift-gate seam and the duplicate POL-002 rule (deferred; ISS-309 part 2). The gate is the only durable defence against re-drift.
- RV-391 F-7 — the scratch-repo control criterion is owed to /plan: plant one repo-private id per channel; the read must flag each.
- CHR-081 — consolidate the two local memories restating the grounding rule (out of scope; local-memory health is a non-goal corpus).
- ISS-215 — boot-index defect; CHR-036 — distilling project-local memories (both out of scope).
