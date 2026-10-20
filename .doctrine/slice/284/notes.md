# Notes SL-284: Embedded skills for non-Claude harnesses

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design triage (2026-10-19, exploring)

Evidence: `research/research.md` (threads 1-3).

**Shaping decisions (user-settled, 2026-10-19):**
- S1 — codex / pi / universal take `skills = ".agents/skills"`; doctrine never
  links into `.pi/skills` (pi warns on the duplicate).
- S2 — no product migration code. Existing npx copies fall to the existing
  `Shadows` notice. The user's own checkouts get a one-off cleanup at close
  (rm npx copies in `.agents/skills` + `.pi/skills`, rm `skills-lock.json`,
  re-install). No `.pi/skills` / lockfile disclosure ships.
- S3 — remove the npx fallback outright; an unknown `--agent` is refused.
- S4 — ISS-479: a first-class pi boot adapter; `[harness.pi] boot = "pi"`. Pi
  arm = pi extensions + `AGENTS.md` import. Codex arm = hooks + MCP. Pi does
  not read codex MCP config.
- S5 — spec changes via one REV over SPEC-010 / SPEC-009 / SPEC-011.

**Open questions:** see the design run's inquiry graph.

**Risks:** R3 SL-283 textual collision on `[harness.codex]` (sequence).
`universal --global` lands at `~/.agents/skills`, not npx's
`~/.config/agents/skills` (accepted).

**Assumptions:** A1 — pi and codex follow symlinked skill dirs under
`.agents/skills` (VH at close).

**Constraining governance:** SPEC-010 D2/D3/REQ-175, SPEC-009 D5/REQ-172,
SPEC-011, ADR-019, POL-002, POL-003, STD-001, STD-003.

## Design review passes

- **RV-423** (codex gpt-6-sol, high; 2026-10-19): 8 findings (F-1..F-5 major,
  F-6..F-8 minor), all fix-now in design rev 35, verified by the raiser, concluded.
- **Further pass: none needed at design.** The remaining risk is in
  implementation detail: the `RefreshReport` outcome plumbing, the rewritten e2e
  suites, and the `--only-memory` seam. The audit's RV covers that against the
  real diff. A further design pass would only re-probe the pi activation wording
  and the REV clause list, both now explicit.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-20 · reconcile · bb7d66e46

### Produced
- design.md + DEC-386..DEC-390 + RV-423 — the design arc (run dr-01a1514a…).
- PHASE-01..05 + plan repair — `0fed5b55f`, `c2b65323d`, `67e49b2f1`,
  `9814373f5`, `6bdf72a25`, `2841ac1d3`, `9b368b541`.
- `3b7090881` — the audit's four fixes: a partial pi write is disclosed, the
  settings fallback names the requirement instead of an impossible edit, the
  activation notice stops promising a `/reload` that cannot load, two stale
  `Leg::Skills` comments and an empty-route offer render.
- `8557a881f` — IMP-520, IMP-521, ISS-527 (the audit's three follow-ups).
- `043acddf9`, `bb7d66e46` — RV-424 (12 findings, brief) and this harvest.
- gate — `doctrine check gate` green at the range tip and again after
  `3b7090881`; `doctrine check commit` green at `bb7d66e46`. `.doctrine` work went
  out in its own commits, separate from code where that mattered.

### Learned
- mem.pattern.governance.time-precedence-is-not-recorded — a later accepted
  decision does not retire an earlier one, because time-precedence is a rule an
  agent applies, not a fact the corpus records.
- mem.pattern.reporting.app-propagation-discards-partial-outcomes — `?` per field
  in a multi-field outcome struct discards the partial success already gathered.
- mem_019ed002b43d7630b2d69797489e51aa — enriched: a commented-out `[[phase]]`
  header absorbs that phase's keys and yields a *different valid plan*, silently.

### Open
- ASM-013 — pi and codex follow `.agents/skills` symlinks; human check at close,
  after the checkout cleanup.
- REV-074 — approval and apply at reconcile, then `doctrine spec validate` on
  SPEC-009, SPEC-010, SPEC-011.
- DEC-377 — successor decision plus supersession at reconcile: the
  knowledge-record half of RV-424 F-1, which a Revision cannot carry.
- selector registry corrections — the six `doctrine slice selector` edits named by
  RV-424 F-7; conformance stays red until they land.
- IMP-520 — harden `write_atomic` (temp created before the mode is applied;
  special bits copied).
- IMP-521 — symlink-safe settings writes on the Claude and codex arms.
- ISS-527 — SL-119's pi `APPEND_SYSTEM` delivery has no production caller.
- VH-1 — one-off checkout cleanup: 40 npx copies in `.agents/skills`, 33 in
  `.pi/skills`, the tracked `skills-lock.json`, old `.pi/extensions/doctrine/`,
  then `doctrine install`.
