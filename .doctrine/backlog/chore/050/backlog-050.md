# CHR-050: Audit runtime-state scope resolution for dispatch/ and review/ subtrees

## Problem

The runtime tier has an unrecognised second axis. "Authored / runtime / derived"
says *how durable* a file is; it says nothing about **whose fact it is**. Inside
the runtime tier three scopes are tangled in one flat namespace under
`.doctrine/state/`:

| Scope | Meaning | Examples |
|---|---|---|
| **repo** | true of the *slice*, whichever tree you stand in | `slice/NNN/boundaries.toml`, `slice/NNN/phases/` |
| **tree** | true of *this checkout* | `boot.md` |
| **session** | true of *this agent run* | `mem-surface-seen-<uuid>.txt`, `handover.md`, `mem-surface.log` |

Scope is currently decided **ad hoc, per call site**, with nothing in the naming
or types to signal which you get:

- `boundaries_path` (`src/state.rs:716`) resolves through
  `crate::git::primary_worktree(cwd)` — documented *"so every worktree shares one
  file"*. Repo-scoped, correct.
- `phases_dir` (`src/state.rs:135`) is a bare path join on whatever root it is
  handed. Same directory, one level apart, same kind of fact — resolved
  differently.

That divergence is the root cause behind ISS-212, IMP-272, IDE-028's mirror,
ISS-269, and the SL-228 / RV-312 appended-phase refusal: five patches on one
missing rule.

## Scope of this chore

ISS-269's slice fixes the phase-sheet instance and records the scope taxonomy as
its design rationale. This chore is the **audit of the remaining subtrees**, kept
separate so that slice does not triple its blast radius on unverified suspicion.

Suspects — **resolution not yet checked**, treat as leads not findings:

- `.doctrine/state/dispatch/` — journal, candidate worktrees. A drive is one
  thing across trees; smells repo-scoped. Verify how its paths resolve.
- `.doctrine/state/review/` — 301 entries. RV work happens in linked worktrees at
  audit time, which is exactly the ISS-269 trigger shape. Prime suspect for the
  identical defect.

For each: determine the intended scope, check whether the path constructor
matches it, and file an issue where it does not.

## Related

- **ISS-269** — the phase-sheet instance; establishes the rule this audit applies.
- **IDE-028**, **IMP-272**, **ISS-212** — prior patches on the same root cause.
- **IMP-338** — the session tier has no defined home at all; the flat-drawer
  accumulation below is its symptom.

## Aside: the flat drawer

`.doctrine/state/` top level holds 150+ `mem-surface-seen-<session-uuid>.txt`
accumulated since 2026-07-10, alongside `boot.md`, ad-hoc `.md` scratch, and the
`slice/` tree. Any survey of the tier is unaffordable — an `ls` cost ~8k tokens
during ISS-269 preflight (friction observation recorded 2026-07-28). Session-scoped
runtime wants its own subdirectory and a reaper. Adjacent to IMP-338; not this
chore's job to fix, but the same taxonomy motivates it.
