# ISS-455: run_bounded join can outlive its deadline when a descendant holds the pipes

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`coverage_verify::run_argv` (and the `subprocess::run_bounded` extraction
proposed in IMP-452) bounds only the **direct** child. After `try_wait` reports the
child finished, or after kill-and-wait on timeout, it joins the stdout/stderr
drain threads. A child that forks a descendant inheriting those pipes and then
exits leaves the drains blocked until the descendant closes them, so the call can
outlive its wall-clock bound arbitrarily.

Reproduction of the pipe half: `time sh -c 'sleep 30 & exit 0' | cat` — the
parent exits at once, the pipe stays open for 30 s.

## Reach

- `coverage_verify` runs arbitrary configured verification commands (e.g. test
  runners that spawn helpers), so it is exposed today.
- SL-245's `graphviz` render path is not: it has no deadline (IMP-452), and
  `dot` does not fork.

## Direction (not decided)

Own the process tree — spawn in a new process group and kill the group on
timeout — and/or bound the drain joins so a held pipe cannot extend the deadline.
Needs a test helper that forks a descendant holding all three pipe ends.

Raised by RV-368 F-5 (codex design review of SL-245).
