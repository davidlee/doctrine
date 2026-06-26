# IMP-173: Dispatch-run ownership signal for solo stand-down

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**F4 hardening follow-up to SL-154** (design §7 D9-limitation / D12, §8 R2).

SL-154's solo capture binding stands down when a **live coordination worktree**
exists for `dispatch/NNN`. Liveness ≠ ownership: a coord worktree left un-pruned
through the pre-integrate audit window **false-stands-down a post-drive solo
phase** (the binding records nothing → the gate/conformance halts loudly →
`record-delta`). Loud and recoverable, but a real rough edge.

The `provenance` field added in SL-154 (D12) does **not** fix this — it marks
ownership *post-record*, whereas this is the binding's *pre-record* stand-down
decision. The precise fix is a **dispatch-run ownership signal** (run-state: "is
THIS phase owned by an active run?") rather than worktree presence — so the
binding records a genuinely-solo post-drive phase instead of standing down on a
stale worktree.

Not in SL-154 scope (accepted as R2/F4 there). Source: SL-154 design §7 (D9
limitation), §8 R2.
