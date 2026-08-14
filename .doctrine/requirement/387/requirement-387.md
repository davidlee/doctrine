# REQ-387: One funnel state machine owns the shared transition semantics; each dispatch transport (main-thread, subprocess, confined-orchestrator) projects into that single authority, recording the same transitions whether committing directly or through mediation; per-transport altitude reconciles with REQ-291 and REQ-335.

## Statement

> **AMENDED — NARROWED (SL-254, 2026-08-14).** The single-authority requirement
> stands and `src/funnel_machine.rs` is RETAINED as that authority. What narrows
> is the transport enumeration: of the three transports the title names, TWO are
> gone. `main-thread` was the in-session claude arm, collapsed onto the one
> confined subprocess arm; `confined-orchestrator` was Mode B, retired entirely
> (`DEC-217`, REQ-335 as amended). Only `subprocess` remains, and it is now
> universal — every harness, claude included, spawns through
> `scripts/spawn-confined.sh <harness>`. The title is retained unchanged; the
> elaboration below states what must hold.

One funnel state machine owns the shared transition semantics, and every path
that lands a position does so through `funnel_machine::attempt` — no other code
path may land one. With a single transport this is now a degenerate case of the
original property rather than a contested one: there is nothing left to diverge
from the authority.

The property is retained rather than dissolved because its VALUE was never the
plurality of transports — it was that the machine, not its callers, owns the
legal-transition table. A future transport (a new harness posture, a remote
executor) inherits the same authority by construction.

The title's closing clause, "per-transport altitude reconciles with REQ-291 and
REQ-335", is superseded: REQ-291 as amended states a single kernel-level altitude
uniform across all harnesses (uniform across *platforms* only in intent and
write-fencing — see REQ-291 clause 5), and REQ-335's tier is retired. There is no
per-transport altitude left to reconcile.

## Rationale

Collapsing to one transport removes the divergence risk this requirement was
written against, but not the reason to keep the machine central: the transition
table is the only thing standing between an interrupted funnel and an
unrecoverable position, and it must stay single-sourced whether it serves one
caller or four.
