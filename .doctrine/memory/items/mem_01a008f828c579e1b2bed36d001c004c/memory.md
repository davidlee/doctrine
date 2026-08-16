# Locate a helper before routing a layering repair through it

A repair that removes a back edge can reinstate it three sections later, through a
helper whose module nobody checked. The tell is a call written with a *plausible*
module path rather than a read one.

**SL-238, `RV-358`.** `F-1` ruled that `backlog → commands` closes a command-tier
cycle, and the accepted repair was fn-pointer injection: `cli.rs` supplies
`commands::dep_seq`'s operations, `backlog` gains no import. Two findings later,
`F-5`'s repair gave `backlog needs` the admissible-target gate and spelled the call

    anyhow::ensure!(kinds::is_admissible_dep_target(tkref.kind), …)

`kinds::is_admissible_dep_target` does not exist. The function is
`commands::dep_seq::is_admissible_dep_target` (`dep_seq.rs:34`), built from that
module's `is_work_like`, and its refusal message interpolates
`record_kind_list_slash()`, which reads `knowledge::RecordKind::ALL`. So the repair
for `F-5` was the exact edge `F-1` removed, one axis over — and it read as correct
because `ADMISSIBLE_DEP_TARGETS`, the *constant*, really is in `kinds`. The
predicate over it is not.

## Why it survives a careful read

The wrong module is the *right* module for the adjacent thing. A constant in `kinds`
makes `kinds::` feel settled for the predicate too; a type in `entity` makes
`entity::` feel settled for its helpers. Nothing in the sentence looks unverified,
so the layering argument built on top of it inherits an unchecked premise, and the
argument is exactly where the review's attention is.

## The move

When a repair's correctness rests on a module boundary, resolve every named callee
to a file and line before writing the prose around it — not after. Two cheap
falsifiers, both seconds of work:

- Grep for the definition, not the call. `fn <name>` locates it; a call site only
  proves someone else could reach it from where *they* sit.
- Re-run the finding's own rule over the repair. If the finding said "X must not
  reach Y", ask of each name the repair introduces: whose module is this?

The same pass caught the sibling failure — the design said "`cli.rs`'s
`Command::Backlog` arm fills the struct", but that arm calls
`backlog::dispatch(command, color)` (`cli.rs:1705`) and reaches no `run_*` function
at all, so the injection had no route. **A plausible caller is not the caller**;
read the dispatch chain to the function that actually takes the argument.

Related: [[mem.pattern.review.bind-scope-bar-and-never-self-rule]] — this is the
defect class that makes self-ruling on your own remediation a bad bet.
Sibling: [[mem.pattern.lint.back-edge-tangle-inject-fnptr]] — the repair idiom
whose second application this was.
