When expanding phase criteria (`/plan` stage 2), a design's `Verification
alignment` block is the obvious source for `VT` test titles — and it is grouped
by **design section**, which is not the same cut as the **phase**.

A phase's provenance row hands it a block. Some titles in that block exercise
code another phase writes. Take them anyway and the mandate names a `test_file`
the phase never touches: `verify-vt` FAILs it for the whole life of the phase,
and when it finally passes it passes because a *different* phase landed the
file. That is an inert gate wearing a green light.

**Rule: assign each `VT` to the phase whose `test_file` it names.** Block
membership is positional; ownership follows the code. When the two disagree,
move the title and record the move in a `# NOTE for the integration pass`
comment beside the phase that gave it up, so the stage-3 pass sees a decision
rather than a gap.

Worked instances, `SL-248` (five in one session):

- `resolver_whose_basename_is_forbidden_by_the_policy_refuses` sat in the
  backend section's closure group, but the design states the check as a
  *provisioning step*. Moved to the phase owning `provision.rs`.
- Three titles in the bubblewrap block guard rules that live in
  `CapsulePlacement::try_new` — a different file and a different phase. Moved
  to it, reinforced by that design's own rule that a refusal and its positive
  control must land together.
- Two capsule-root titles in the configuration section exercise the placement
  validator, which the configuration phase does not write. Moved forward.

The inverse also happens and is the cheaper error: a title whose code *is*
yours but which sits in a block your provenance row does not name. Claim it and
say why — a phase silently declining a title it owns is how a suite ends up
with an unproven rule.

Related: [[mem.pattern.doctrine.size-phases-on-evidence-not-lines]],
[[mem.pattern.doctrine.en-criteria-name-the-honest-dependency]].
