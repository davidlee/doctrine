# Solo /execute cannot land into a tree another agent is editing

`doctrine worktree land --fork <branch>` runs `classify_land` with the tree's
cleanliness as its **first** precondition — before fork existence, worktree
liveness or the marker check (`src/worktree/mod.rs` `classify_land`; the
refusal-token test pins the precedence as *tree-unclean wins over every later
fault*). It is a hard refusal, `land-refused: tree-unclean`, and it fires
*before* any merge, so it is safe to attempt — it just cannot succeed.

That makes solo `/execute` **unlandable in this repo's normal posture.** AGENTS
says *"Assume multiple agents are working in the same repository"*; a tree with
another agent's uncommitted file in it is dirty by design, and the remedy the
refusal names (a clean tree) is not the executing agent's to perform — cleaning
would discard another agent's work, which AGENTS forbids outright (no stash, no
`checkout --`). The reflex to "just commit their file" is worse: it lands
somebody else's half-finished work under your message.

## What actually happens to the phase

The in_progress flip already stamped `code_start_oid` (HEAD at flip time), and
completion would stamp `code_end_oid = HEAD` in the primary tree. So a phase
that cannot land **cannot be completed either**: completing it would record a
boundary whose range does not contain the code. Leave it `in_progress` and
escalate — do not flip a phase to completed on the strength of a green fork.

Two adjacent traps this is *not* (both survive landing, and are handled
separately):

- [[mem.pattern.audit.fork-land-unbound-source-delta]] — a landed fork whose
  boundary was never written to the primary registry (under-reports).
- the boundary-pollution cell — a range that spans trunk movement between
  `code_start` and `code_end` (over-reports). With `land` being a `--no-ff`
  merge onto a moved trunk, `dc3622ff3..merge` includes every commit that
  landed in between; `record-delta --start <fork-base> --end <fork-tip>` is the
  correction, because the phase's own code is exactly the fork's range.

## How to apply

1. Before forking for a solo `/execute`, check the primary tree
   (`git status --porcelain`). If it holds another agent's work, say so up
   front — the phase can be executed but not **landed**, and that changes the
   plan.
2. Run the fork and the TDD loop normally; `just check` green on the fork is
   real evidence and worth having regardless.
3. Attempt `land` (it is safe: it refuses before merging). On `tree-unclean`,
   stop, leave the phase `in_progress`, and hand the human the fork branch and
   the blocking paths. Serialising the landing is a coordination act only the
   human can make.
4. Do not work around the guard — `worktree gc` will not help either (its
   oracle requires the fork to have landed).

Measured on SL-266 PHASE-01 (2026-09-25): two commits green on
`slice/SL-266-inquiry-map-tree-view`, blocked by `.doctrine/slice/264/slice-264.toml`,
`.doctrine/slice/267/slice-267.toml` (modified) and `review/390` (untracked)
from a concurrently-working agent. Friction records:
`.doctrine/observations/records/af/`, `.../58/`.
