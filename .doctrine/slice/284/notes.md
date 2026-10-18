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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
