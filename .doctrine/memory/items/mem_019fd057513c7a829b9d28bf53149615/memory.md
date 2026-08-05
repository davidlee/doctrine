A task breakdown runs forward. A shared helper extracted at task N is applied to
task N's sites and to every site after it — and *never* to the sites before it,
because no later task has a reason to re-read them and the tree stays green
either way. The duplication is invisible per-task and only visible in the
phase's whole diff.

**Where this bit.** SL-244 PHASE-05: `tests/e2e_design_delegation.rs`'s
`WRITER_ACTS` built two act payloads out of the wire types at `T4`;
`tests/design_act/` arrived at `T8` producing exactly those two values. Four
tasks and ten commits later nothing had re-pointed the earlier site. Found by the
close-out's parallel-implementation sweep, which is run over the phase's whole
diff rather than per task — that scope is what makes it findable.

**The cheap confirmation.** Delete the duplicate body and let the compiler
measure how total the duplication was: three imports lost their only reader and
went with it. A duplicate whose removal orphans imports was never partial.

**Practice.** When you extract a shared helper mid-stream, grep for its output
shape across the whole unit immediately — not at the next task, which will not
look. Failing that, make the whole-diff sweep a close-out beat and expect it to
find something; see [[mem.pattern.doctrine.close-out-sweeps-the-whole-diff]].
