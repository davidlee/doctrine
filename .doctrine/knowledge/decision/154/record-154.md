# DEC-154: Unconfirmable worktree topology keeps the deny

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The shape of the mistake this avoids

`DEC-152` says an ordinary subagent may pass through. The arm it says that
about is answering a different question than it appears to: not *is this an
ordinary subagent* but *did the worktree test come back true*. Those coincide
only while every false is a confident one.

Three ways to get a false that isn't:

| what fails | why it reads as "not a worktree" |
|---|---|
| `is_linked_worktree` errors | `matches!(…, Ok(true))` folds `Err` into `false` |
| `CLAUDE_PROJECT_DIR` absent | no anchor ⇒ early `return false` |
| either `common_git_dir` errors | the match's `_ => false` arm |

The third is ordinary bad luck. The second is not: an absent anchor fails for
the **entire** worker population simultaneously, so the failure mode is not
"one worker occasionally slips" but "no worker is confined and nothing says
so".

Today all three deny, and a mis-seated worker is loudly dead rather than
quietly unconfined. Blanket pass-through would trade a visible failure for an
invisible one, which is the wrong direction for a wall whose entire remaining
job is preventing accidents.

## Why not the cheaper split

An earlier reading of this question split the arm on whether the payload `cwd`
resolves at all. That is nearly free — the shell already encodes the
unresolvable case as an empty `PathBuf` — and it is the wrong axis. Cheapness
is only a virtue where the thing bought is the thing wanted, and what was
wanted here is insurance against a worker escaping its worktree. That accident
is a topology failure, not a cwd failure.

## The residual, and where it gets fixed

An ordinary subagent whose cwd is outside any git repo reads `Unknown` too, and
gets denied — the very defect `SL-247` exists to fix, recurring rarely.

The temptation, if that ever bites, will be to widen the deny into a pass. The
better instrument is a different seam: **assert at spawn rather than infer at
deny.** A `doctrine` verb that checks the spawner is correctly seated before it
launches establishes the fact positively, in front of an orchestrator or a
human who can read the error — where `PreToolUse` can only guess from evidence
that may be missing, in front of nobody.

## What this does not touch

`Jail(wt)` itself. The Bash leg (nested bwrap, rw cwd only, shared `.git` RO)
and the `Edit`/`Write` `realpath ⊆ cwd` check are unchanged and stay proven by
the existing suites. `inq-5` asked whether the worktree arm needed
strengthening; it does not. What needed changing was the arm beside it.

See [[mem.fact.dispatch.confined-subagent-cwd-resets-breaks-positional-arming]]
for why a jailed subagent cannot `cd` its way into the pass-through arm: its
`Bash` cwd resets to the worktree root on every call.
