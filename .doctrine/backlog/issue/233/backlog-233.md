# ISS-233: codex/pi dispatch arm: verify per-phase completed flip reaches primary tree for prepare-review gate

## Context

Follow-up split off from IMP-272. That fix cured the **claude arm**: it mirrors
the per-phase completed flip into the primary tree from `dispatch_conclude_phase`
(`src/mcp_server/dispatch.rs`), because `prepare-review`'s completeness gate reads
the completed-phase set from the PRIMARY tree
(`registry_completeness(&primary, &primary, …)`, `src/dispatch.rs`).

The **codex/pi arm** was verified out of scope for that fix but left unresolved:

- Its registry write is `slice record-delta <SL> PHASE-NN --commit <S>`
  (`run_record_delta`, `src/slice.rs`), which resolves the registry against the
  primary tree but **does NOT flip phase status**.
- The per-phase `completed` flip on that arm is a **separate** step whose locus is
  undocumented in `/dispatch-subprocess` and `/dispatch` — it is not
  `dispatch_conclude_phase` (that's the claude-arm MCP tool only).

## Question

Where does the codex/pi orchestrator flip each phase to `completed`, and does that
flip land in the PRIMARY tree the gate reads? If it flips from the coordination
worktree (as the claude arm did), it shares the IMP-272 gap — the gate will refuse
landed rows as "not a completed phase".

## Outcome

Verify against a real codex/pi drive (or trace the router). If the gap exists,
apply the same primary-mirror cure at that arm's flip locus. If it already writes
primary, close as `wont-do` / `obsolete` with the evidence noted.

Origin: IMP-272 fix (fork D-D, subprocess-arm scope deferral).
