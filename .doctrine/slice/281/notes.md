# Notes SL-281: Install artefact ownership

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-16 · plan approved, status ready · design re-locked at plan

### Produced
- design.md (locked, run dr-01a1422f…, rev 50); RV-418 (13/13 verified; pass waived for missing routes)
- DEC-372, DEC-373, DEC-374, DEC-375, DEC-376, DEC-377
- backlog: IMP-513, IMP-514, IMP-515, IMP-516, IMP-517, ISS-520; IMP-406 annotated + after SL-281
- design-target selectors (9)
- plan.toml/plan.md (PHASE-01..06); REV-071 drafted + approved (SPEC-009, SPEC-010, REQ-167/170/172/176/180)
- RV-419 (plan-time amendment pass, codex; F-1 verified)

### Learned
- ISS-216's refresh half already fixed; real fault is stale-layout links kept as foreign
- `.doctrine/agents/AGENTS.md` is a repo-local relic, not install output
- pi skills duplication = `npx skills --agent pi --agent universal` from detect_agents
- boot.rs detector misses `.pi` (RV-418 F-11)
- plan-time: ensure_gitignored has no prod caller beyond Step::Gitignore → deleted (design amended); REV widened past design table (same class as RV-418 F-7)
- friction observations: 5 recorded (apply flag, runbook text, ISS-356 recurrence, missing routes → IMP-517)

### Open
- REV-071 applies at /reconcile
- next: /phase-plan PHASE-01

## Design review passes

- **RV-418** (adversarial, codex `gpt-6-sol`, 4 rounds, 2026-10-16): 13
  findings, all verified. The hard part was F-1, prune and staging under
  symlinked ancestors. It took three rounds to settle on: removals gated by an
  ancestor `lstat`; writes and relinks limited to member paths; staging always
  exclusive-create and never pruned.

**Further pass: none needed before lock.** The design surfaces that remain
unprobed by RV-418 are implementation-level, and `/audit` covers them better
than another design pass:

- that `discover()` and the registry seams compose as the Code impact table
  claims;
- the exact report-line wording;
- the REV's before/after excerpts, which are drafted at `/plan` and approved
  separately.

If anything, a later pass should probe the staging rule's interaction with
`materialise_canonical`'s remove-then-rename window once code exists.
