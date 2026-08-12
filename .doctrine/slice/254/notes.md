# Notes SL-254: Collapse dispatch onto one subprocess arm

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-13 · proposed (scoped + researched, design not started) · eca536780

### Produced
- SL-247 abandoned at design; scope `## Summary` carries the dissolution (e788e1520)
- minted: DEC-202 — retire the claude in-session arm for a confined `claude -p` subprocess; EVD-023 — `claude -p` is subscription-billed, falsifying ADR-011's premise (e788e1520)
- SL-254 scoped with selectors + relations, carrying a reconcile/close survey (61c54752f)
- minted: ISS-347 — REQ-288 and SPEC-021 responsibility 2 misstate the env-marker as `.claude/` presence; it is `CLAUDECODE=1` (95a00592c)
- marker-identity scope drift corrected against DEC-202 (eca536780)
- research round complete — 5 threads, `research/research.md` assembled. **Runtime tier, gitignored**: it does not survive a state wipe, so anything load-bearing must be lifted before then
- 2 friction observations recorded; the second (`019ff631…`) is **uncommitted**
- verification gate: `doctrine check gate` NOT run this session — no code was modified, `doctrine check commit` green at each commit

### Learned
- mem_019ff650d94a7960a638913a40416165 — collide "what calls this" research findings against "what should exist" decisions at synthesis
- The reuse finding: `crates/doctrine-control` already implements clone-inside-bwrap (`provision.rs:949`, `backend/bubblewrap.rs:1110`) — see QUE-215

### Open
- QUE-214 — Pole A or Pole B: how far the demolition goes; decides slice size and phase count
- QUE-215 — ride `doctrine-control`'s provisioning/bwrap backend, or keep the shell script
- SL-254 `OQ-2` — five backlog items plausibly dissolved rather than fixed (IMP-269, IMP-342, IMP-334, IMP-337, IMP-407), plus IMP-401 and IDE-024; confirm at reconcile
- SL-254 `OQ-3` — one arm or a degraded second rung if an environment cannot run `claude -p`
- Nominate/denominate hooks are orphaned by the collapse and the scope is silent — disposition explicitly at design
- `worker_commit`'s fate: its enforcement path is wholly worktree-topology-coupled and its ro-git premise dies with the clone; settle before solving headless MCP auth
