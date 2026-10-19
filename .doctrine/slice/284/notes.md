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
fresh-as-of: 2026-10-19 · design locked (plan next) · 12f755b1a

### Produced
- design.md (run dr-01a1514a…, locked rev 40); DEC-386, DEC-387, DEC-388, DEC-389, DEC-390; RV-423 (concluded).

### Learned
- Friction observations recorded: research agents unusable outside the jail; the lock gate needs routes on verified major findings (reviewer should pass `--route` at raise).
- Local memory `mem.fact.doctrine.agents-skill-mirror-is-published-source` is to be retired at close (design sec-3).

### Open
- A1 (pi/codex follow `.agents/skills` symlinks): VH at close (design sec-7).
- REV over SPEC-009/010/011: to raise during execution (design sec-6).
- One-off checkout cleanup at close (design sec-7).
- Follow-up candidate: `install_append_system` has no production caller (design sec-4).
