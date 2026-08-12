# DEC-211: The REV contents and the anchor sweep

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->


## Canon verification (design, 2026-08-13)

Read directly rather than through the research round's quotations, because this
decision commits to amending both.

**ADR-011 — five sites, not four.** Context line 28 (`For claude the only viable
backend is the in-session Agent tool`); D1 at 43-45 (worker identity is an
orchestrator-stamped disk marker); D3's table at 89 and 93 (`disk marker only —
no env channel` and `OS confinement | none — Agent is not a subprocess to wrap`);
D4 at 98-100 (three enhancements `codex/pi-only until a free claude env backend
lands`); and **line 263**, which restates D4's codex/pi-only claim in the
consequences register. The fifth site was not in this decision's original list.

**ADR-006 D2b — thread 1's claim holds, and the note pattern is real.** D2b's
main body (line 120) reads: raw-tree confinement `is not CLI-stoppable` and `the
harness does not confine workers to their worktree (observed: Claude Code creates
the worktree but lets the agent write to main)`. The SL-181 note at line 143
exists as described and already declares the gap `enforcement-closed on confined
arms`, citing SL-182/183/185. So SL-254's note extends an existing pattern to the
arm that was unconfined, exactly as thread 1 recommended.

**What the research did not surface.** The SL-181 note's self-limiting degenerate
cases include `standalone clone → contained to a disposable object store`. Under
SL-254 the clone is the NORMAL provisioning shape, not a degenerate one, so the
new note must re-cut that framing rather than merely append to it. This is the
same shape as DEC-207's describe_mode truth-table re-cut — in both places a case
ADR-006 and marker.rs treat as the anomaly becomes the expected case. Amending
one without the other would leave the governance and the code disagreeing about
which situation is normal.

**Second ADR-006 site.** Line 308 restates the gap in the consequences register:
`The worker-sole-writer invariant has no harness enforcement (D2b)`. It moves with
D2b.

**A fence composition change.** D2b's defence-in-depth fence is enumerated as the
R-5 import belt, the IMP-052 post-spawn check, the env-worker-on-main catch and
bwrap-no-push. DEC-204 re-homes the belt and DEC-207 deletes the marker leg, so
the fence's composition changes and the note must describe the fence that
actually exists afterwards.