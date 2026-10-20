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
fresh-as-of: 2026-10-20 · audited (RV-424), handed to reconcile · 8557a881f

### Produced
- design.md (run dr-01a1514a…, locked rev 40); DEC-386, DEC-387, DEC-388, DEC-389, DEC-390; RV-423 (concluded).
- PHASE-01 `0fed5b55f` · PHASE-02 `c2b65323d` · PHASE-03 `67e49b2f1` · PHASE-04 `9814373f5` · PHASE-05 `6bdf72a25` + `2841ac1d3`; plan repair `9b368b541`. Five per-phase fresh-context reviews (pass or pass-with-nits; every finding disposed in-phase).
- REV-074 over SPEC-009/010/011, raised and NOT yet applied (reconcile's job).
- **RV-424** (audit): 12 findings, all terminal — 4 fix-now, 3 design-wrong (brief), 3 follow-up, 2 tolerated.
- `3b7090881` — audit fixes: partial pi write disclosed; settings fallback names the requirement, not an impossible edit; activation notice no longer promises a `/reload` that cannot load; two stale `Leg::Skills` comments; empty-route offer render.
- `8557a881f` — IMP-520, IMP-521, ISS-527 (the audit's three follow-ups).

### Learned
- Friction observations recorded: research agents unusable outside the jail; the lock gate needs routes on verified major findings (reviewer should pass `--route` at raise).
- Local memory `mem.fact.doctrine.agents-skill-mirror-is-published-source` is to be retired at close (design sec-3).
- `write_atomic` now preserves an existing target's mode — a rewrite of a private `0600` settings file must not republish it at the umask default (surfaced by PHASE-04 review).
- A linked `.pi/settings.json` takes the disclosed manual fallback: the atomic rename would replace the user's link and leave the target stale. The same hazard still exists on the `.claude`/codex settings path (pre-existing, not introduced here) — candidate follow-up.
- `plan.toml` with a commented-out `[[phase]]` header silently absorbs that phase's keys into `[requirements]` and yields a shorter phase list: the plan looked complete (0/4) while PHASE-01 was invisible. Repaired at phase start.
- **Per-phase reviews cannot see inter-phase residue.** All four audit defects that were fixable were in seams between phases (a stale leg comment outside every phase's edited region, a partial-write error path, a fallback message crossing the planner/shell boundary). The per-phase reviews were each correct within their diff; only the whole-range read caught them.
- **A later accepted decision does not retire an earlier one.** DEC-388/DEC-389 clearly govern over DEC-377, but nothing in the corpus says so — no `supersede` edge, no REV row. Time-precedence is a rule agents apply, not a fact the corpus records; the earlier decision keeps reading as a live instruction.
- **Adding a failure point to a struct-literal arm discards earlier outcomes.** The pi arm built one `RefreshReport` with `?` per field; the new settings leg, running last, took the three extension outcomes down with it on error. A multi-field report built by `?` cannot report partial success.

### Open
- A1 (pi/codex follow `.agents/skills` symlinks): VH at close (design sec-7).
- One-off checkout cleanup at close (design sec-7): npx copies in `.agents/skills` (40 dirs) + `.pi/skills` (33 dirs), `skills-lock.json` (delete in a commit), old `.pi/extensions/doctrine/`, then `doctrine install`.
- IMP-520 — harden `write_atomic` (temp created before the mode is applied; special bits copied).
- IMP-521 — symlink-safe settings writes on the Claude and codex arms.
- ISS-527 — SL-119's pi `APPEND_SYSTEM` delivery has no production caller.
- REV-074 must be approved and applied at reconcile (now including the DEC-377 row RV-424 F-1 requires), and `doctrine spec validate` run on all three specs afterwards.
