# Notes SL-281: Install artefact ownership

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-16 · design locked (stage plan) · f7cba8e36 + design commit

### Produced
- design.md (locked, run dr-01a1422f…, rev 50); RV-418 (13/13 verified; pass waived for missing routes)
- DEC-372, DEC-373, DEC-374, DEC-375, DEC-376, DEC-377
- backlog: IMP-513, IMP-514, IMP-515, IMP-516, IMP-517, ISS-520; IMP-406 annotated + after SL-281
- design-target selectors (9)

### Learned
- ISS-216's refresh half already fixed; real fault is stale-layout links kept as foreign
- `.doctrine/agents/AGENTS.md` is a repo-local relic, not install output
- pi skills duplication = `npx skills --agent pi --agent universal` from detect_agents
- boot.rs detector misses `.pi` (RV-418 F-11)
- friction observations: 4 recorded (apply flag, runbook text, ISS-356 recurrence, missing routes → IMP-517)

### Open
- REV for SPEC-009 + REQ-170 + REQ-176 — create/draft at /plan (DEC-376)
- research baseline stale vs amended scope — restamp at /plan

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
