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
fresh-as-of: 2026-10-20 · implementation complete, awaiting audit · 2841ac1d3

### Produced
- design.md (run dr-01a1514a…, locked rev 40); DEC-386, DEC-387, DEC-388, DEC-389, DEC-390; RV-423 (concluded).
- PHASE-01 `0fed5b55f` · PHASE-02 `c2b65323d` · PHASE-03 `67e49b2f1` · PHASE-04 `9814373f5` · PHASE-05 `6bdf72a25` + `2841ac1d3`; plan repair `9b368b541`. Five per-phase fresh-context reviews (pass or pass-with-nits; every finding disposed in-phase).
- REV-074 over SPEC-009/010/011, raised and NOT yet applied (reconcile's job).

### Learned
- Friction observations recorded: research agents unusable outside the jail; the lock gate needs routes on verified major findings (reviewer should pass `--route` at raise).
- Local memory `mem.fact.doctrine.agents-skill-mirror-is-published-source` is to be retired at close (design sec-3).
- `write_atomic` now preserves an existing target's mode — a rewrite of a private `0600` settings file must not republish it at the umask default (surfaced by PHASE-04 review).
- A linked `.pi/settings.json` takes the disclosed manual fallback: the atomic rename would replace the user's link and leave the target stale. The same hazard still exists on the `.claude`/codex settings path (pre-existing, not introduced here) — candidate follow-up.
- `plan.toml` with a commented-out `[[phase]]` header silently absorbs that phase's keys into `[requirements]` and yields a shorter phase list: the plan looked complete (0/4) while PHASE-01 was invisible. Repaired at phase start.

### Open
- A1 (pi/codex follow `.agents/skills` symlinks): VH at close (design sec-7).
- One-off checkout cleanup at close (design sec-7): npx copies in `.agents/skills` + `.pi/skills`, `skills-lock.json` (delete in a commit), old `.pi/extensions/doctrine/`, then `doctrine install`.
- Follow-up candidate: `install_append_system` has no production caller (design sec-4).
- Follow-up candidate: symlink-safe settings writes on the Claude/codex arms.
- REV-074 must be approved and applied at reconcile, and `doctrine spec validate` run on all three specs afterwards.
