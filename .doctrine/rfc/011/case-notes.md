
[close; SL-228-close-vh1]
`dispatch sync --prepare-review` halted twice on the conformance-completeness
gate, for two causes the handover had recorded as benign:

1. PHASE-08/09 read as "recorded row for a non-completed phase". Cause:
   `registry_completeness` derives the completed set from `completed_phase_ids`,
   which reads the PRIMARY tree's gitignored phase sheets — and the mid-drive
   appended phases were never mirrored there (edge's plan.toml has no such
   phases, so `slice phases` cannot materialise them). The handover called the
   mirror warning "benign, but misreads as a defect". It is not benign: it
   blocks prepare-review at close. Cost: ~6 tool calls reading
   `state.rs`/`dispatch.rs` to establish that the gate roots on primary runtime
   state rather than on plan.toml or the committed ledger.

2. PHASE-07 (evidence-only, deliberately non-funnel) is `completed` but carries
   no source-delta row. The gate has NO exemption for a phase whose delta is
   authored `.doctrine/` artefacts rather than source, so it can only be
   satisfied by a synthetic `Manual` row. The handover asserted the opposite
   ("nothing to record-delta for it") — an untested assumption written before
   prepare-review was ever run for this slice.

Root cause common to both: the completeness gate's inputs (primary runtime
sheets + primary registry + committed ledger on the dispatch ref) span three
tiers in two trees, and its refusal names only the symptom phase, not which of
the three disagreed. Per-phase, the refusal cannot distinguish "you forgot
record-delta" from "this phase's sheet never reached the primary tree".
Same family as ISS-241 and the D10 counter-example set.

## [dispatch; sl230-p05-drive]

**Trunk-drift is invisible at the verb the router sends you to.** `/dispatch`
step 3 says run `dispatch plan-next --slice N`. Its output is phases + `next:`
and carries no base-freshness signal — the `trunk: moved (25 commit(s) ahead of
fork-point)` line lives only in `dispatch status`. The router *does* carry a
"Base freshness (mid-drive)" section saying to watch `dispatch status`, but the
hot-path step it prescribes is `plan-next`, so an orchestrator working the numbered
loop reaches the spawn without ever having run the verb that would tell it. Cost
here: caught only because the handover said "check `dispatch status` before
assuming trunk is stable" — i.e. by a slice-local packet note, not by the skill.
Cheap fix: have `plan-next` echo the same drift line, or fold the freshness check
into the router's step 3.

**Stale `file:line` citations in an authored plan invite a verification round.**
SL-230 PHASE-05's EX-3 pinned "the assertions at `tools.rs:1488` and `:1870` do not
move". By execute time they were at `:1535`/`:1917` — moved by four intervening
phases of the same slice. The criterion's *intent* (tool count stays 25) was
untouched, but the citation forced a check to distinguish "the numbers drifted"
from "a finding". Authored criteria should pin the invariant and cite a grep
anchor, not a line — the same rule the handover already applies to reading lists.

**ISS-253 (arm marker invisible from the coord worktree) confirmed again.** The
`/dispatch` router routes on `.claude/` presence; from the coord tree `ls -d
.claude` is a miss (it is untracked in the primary tree, so the fork does not
carry it). Cost one extra round trip to re-check against the project root.
