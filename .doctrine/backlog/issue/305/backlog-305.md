# ISS-305: candidate create --help contradicts the conflicted lifecycle it always takes

## What

The clap help on `dispatch candidate create` (`src/dispatch.rs:420-422`) states,
unconditionally:

> A content conflict aborts cleanly, writing no row/ref/worktree.

Since SL-212 that is false on the arm most users reach. `create_conflict_worktree`
(`src/dispatch.rs:2708+`) CAS-creates the branch, writes a `Conflicted` row with an
empty `merge_oid`, provisions a worktree on the branch, materialises the conflict
stages — and returns `Ok(())`. It writes all three of the things the help says it
writes none of, and it exits **zero**.

The reachability makes it worse than a stale sentence: `--worktree` is **mandatory**
for a `review_surface` candidate (the verb refuses without it), so for that role the
"aborts cleanly" branch is not merely uncommon — it is **unreachable**. The help
describes only the path that role can never take.

The neighbouring internal doc comment at `src/dispatch.rs:2404` is correct and
correctly scoped ("Happy path only — … the conflicted + `--worktree` lifecycle is
PHASE-03"). Only the user-facing string is wrong.

## Why it matters

Two costs, and the second is the real one:

1. A caller who trusts the help writes `set -e` around `candidate create` and
   believes a conflict will stop the script. It will not — the conflict is a
   recorded lifecycle state, not a command failure.
2. The refusal to auto-resolve is carried by the **ledger**
   (`candidates.toml`: `status = "conflicted"`, `merge_oid = ""`), never by the
   exit status. Nothing in the help says to read it, so the one surface that
   does carry the verdict is the one a reader is not sent to.

## How found

SL-241 PHASE-05 T5, the H10/H16 conflict sub-probe
(`scripts/spike-capsule/lib/conflict.sh`). The probe was written against the help
text and asserted a non-zero refusal; the assertion reddened against correct
software. Recorded there as **F-P05-40**, and the probe now asserts the real
contract — exit zero, refusal in the ledger — with the asymmetry named.

The same sub-probe shows `sync --integrate --trunk` refusing the *staleness* case
**non-zero**. So within one layer the conflict path and the staleness path disagree
about how a refusal is signalled. That asymmetry is a QUE-202 input, and it is the
reason this is filed as an issue rather than folded into IMP-135's general help-text
consistency pass: the defect is factual wrongness about behaviour, not tone or shape.

## Fix sketch

Amend the clap doc comment to describe both arms — the clean merge, and the
conflicted lifecycle that parks the branch at base, records a `Conflicted` row and
materialises a worktree for `dispatch candidate ingest`. State that a conflict is
reported by the recorded row and **not** by the exit status.

Not fixed in SL-241: that phase is a shell spike and admits no Rust changes (S4).
