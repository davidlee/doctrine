# ASM-007: Interpretation classes exhaustive

**Assumption carried.** The five trigger classes in [[interpretation-surface]]
(explicit execution · build-system evaluation · toolchain auto-load · git-level
auto-load · path-shaped data · resource shape) are exhaustive — every way
untrusted content acquires agency on the trusted side falls into one of them.

## Why we are carrying it rather than proving it

The taxonomy was derived from one language ecosystem (Rust/nix) plus git, then
cross-checked against a second (TypeScript/npm) on paper. Two ecosystems is
weak evidence for exhaustiveness, and a missed class is a silent hole: the rig
audit only refuses what the taxonomy can name.

## What would falsify it

SL-241's TypeScript light fixture ([[two-spike-fixtures]]) instantiating a
hostile trigger that none of the five classes describes. That is a deliberate
test, not a hope — the fixture exists partly to run it.

Falsification is cheap here and expensive later: a class discovered during the
spike amends a knowledge record; a class discovered after the post-spike REV
amends shipped enforcement.

## Related

- [[interpretation-surface]] — the taxonomy under test.
- [[interpretation-surface-ownership]] — what rests on it.
- [[two-spike-fixtures]] — the falsification vehicle.
