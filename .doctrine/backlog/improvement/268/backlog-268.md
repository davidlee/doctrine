# IMP-268: Guard arm-spawn default-base to dispatch branch

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

SL-199 chose option (b) for the confined arm's base sourcing: `dispatch
arm-spawn` defaults `--base` to the coord-root `HEAD` when omitted, removing the
last per-spawn LLM responsibility for base selection (F13 determinism). The
accepted worst case of (b) is a **late** miss: a wrong base is only reachable
off-arm (main thread run from a cwd resolving to a different doctrine root) and
fails loud downstream at `worker_commit`'s `C^==B` precondition — a halted phase,
no security regression, but a *later* catch than option (a)'s exit-2-on-omission.

## The improvement (b+)

Have `run_arm_spawn` assert the resolved root is on a `dispatch/<NNN>` branch
before defaulting `--base` to its `HEAD` — refuse the default otherwise. Restores
loud-**early** failure for the wrong-root case at ~zero cost. Explicit `--base`
is unaffected (an explicit value always wins).

## Trigger

Do this **if token waste is observed** as a result of the late miss — i.e. if a
wrong-root arm actually reaches `worker_commit` and burns a halt/retry cycle in
practice. Until then the downstream belt is sufficient and this is not worth the
engine churn.

Origin: SL-199 PHASE-05 design delta (base-sourcing decision).
