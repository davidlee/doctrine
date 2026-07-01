# Phase code_start_oid binds to HEAD at the in_progress flip — never rewrite that commit

`doctrine slice phase --status in_progress <SL> <PHASE>` stamps
`code_start_oid = <current HEAD>` into the phase tracking TOML. At `completed`, it
records the source delta `code_start_oid → code_end_oid (=HEAD)` and REQUIRES
`code_start` to be an ancestor of `code_end` (a forward delta). Consequences:

- **Whatever HEAD is at the flip becomes the phase's immutable anchor** — including
  an empty/checkpoint commit made just before. Do NOT squash, amend, or rebase it
  away afterward: doing so orphans `code_start` off the mainline and the completion
  warns `record_source_delta: code_start <X> is not an ancestor of code_end <Y>
  (not a forward delta)`, skipping the binding.
- **If history is restructured between start and end** (auto-commits, parallel-slice
  landings, rebases), the same orphaning happens even without a deliberate squash.
  Seen on SL-183 PHASE-01: `code_start` (a `mem(...)` commit) landed on a lineage
  discarded by later "doctrine" auto-commits; PHASE-01 completed with NO
  `code_end_oid`. Accepted for a probe phase (ships no Rust ⇒ evidence-conformance,
  not delta-conformance) — but a CODE phase needs the delta, so protect the anchor.

**How to apply:**
- Before flipping a phase `in_progress`, make sure HEAD is where you want the phase
  to START (commit real prep first; don't leave a throwaway commit as the anchor —
  or if you do, keep it).
- Never `git rebase -i` / squash / amend across a phase's `code_start_oid`. Check
  `grep code_start_oid .doctrine/state/slice/<n>/phases/phase-NN.toml` before any
  history edit on the slice's commits.
- A probe/doc phase with no source delta is acceptable; a code phase with a skipped
  binding is a conformance gap — re-stamp start to a HEAD-reachable oid BEFORE
  completing, not after.

Related: [[mem_019f031a315c7803900fcf398092e674]] (boundary start-oid pollution),
[[mem_019f0d369fe97231a788b89d56629d43]] (undeclared conformance sweeps foreign
commits), [[mem_019e9a11e8337613bdf8e96f75a9e6b2]] (lifecycle start→…→close).
