# IMP-269: Fork subagents rejected from writing outside worktree jail

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observation

A `/fork` subagent cannot write at all (`Edit`/`Write`/`Bash` all denied). Root
cause verified: the confinement wall (`src/worktree/jail.rs::resolve_target`,
~:344) rejects any subagent whose cwd is **not a linked worktree** —
`agent_id` present + cwd = the main checkout (`IsMainCheckout`, jail.rs:1856) ⇒
`Target::Reject(REASON_NOT_WORKTREE)` ⇒ `Deny`. A fork inherits the parent's cwd
(the primary/coord tree), so it is walled.

## Why this is NOT an SL-199 concern

This is the wall's **fail-closed default** doing its job — the RFC-005 "no
un-jailing" invariant SL-199 explicitly rests on. Letting fork subagents write to
the primary tree would punch a hole in the confinement wall, contradicting the
threat model. The confined-dispatch path does **not** rely on `/fork`: the
orchestrator is spawned into a linked coord tree (`Jail(coord)`, can write) and
workers are forked into isolated worktrees (`Jail(worktree)`, can write).

## The open question (broader than dispatch)

Is there a legitimate carve-out for **non-dispatch** subagent writes? Candidates:
- a **read-only fork** posture that is explicitly write-denied by design (accept
  the limitation, document it as intended);
- **auto-provision** a fork its own linked worktree on spawn (so it becomes
  `Jail(wt)` and may write safely);
- a narrow, opt-in main-tree write grant gated by something forge-proof.

Any change here touches the confinement wall and likely warrants an RFC/ADR, not
a quiet policy tweak. Kept out of SL-199 scope by design.

Origin: SL-199 PHASE-05 design delta (user-raised design input, 2026-07-05).

## Resolution — obsolete (2026-08-14, `SL-254` reconcile)

The wall this item names no longer exists. `SL-254` deleted
`src/worktree/pretooluse.rs` and `src/worktree/subagent.rs` **whole**;
`REASON_NOT_WORKTREE` and `PRIVILEGED_AGENT_TYPES` are at zero references in
`src/`, and `jail.rs`'s module doc now records that the `PreToolUse` per-tool-call
decision layer is gone and *"what remains is the WRAP-PREFIX core"* — the bwrap
argv the confined subprocess arm is spawned under.

So a `/fork` subagent is no longer walled: doctrine does not mediate subagent
tool calls at all.

This is the delivery of a decision already taken, not a side effect. `DEC-152`
(*Non-worktree subagents pass through unconfined*, accepted 2026-08-05) ruled the
wall's remit is the worktree population only, and named this item in its
consequences — *"IMP-269 and IMP-342 plausibly discharge with it; confirm at
reconcile, do not assume."* `SL-247` was abandoned before it could confirm;
`SL-254` performed the retirement and this is the confirmation.

Closed `obsolete` rather than `fixed`: the mechanism was removed, not repaired.

Provenance: `SL-254` Follow-Ups `OQ-2` · `DEC-152` · `SL-247` `OQ-3`.
