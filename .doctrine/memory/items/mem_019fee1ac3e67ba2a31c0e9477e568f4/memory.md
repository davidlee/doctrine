## The pattern

When a review identifies a single high-level root cause, the temptation is to
sweep every downstream component into it: *"X is wrong, therefore A, B and C all
exist only because of X, therefore delete them."* That inference is only valid
where the component's **requirement** is genuinely entailed by the root cause —
not merely where the component's **current mechanism** was shaped by it.

Test each downstream conclusion separately: *does this component serve a
requirement that survives the root cause being fixed?*

- If no → the requirement dissolves; delete requirement and code together.
- If yes → rescue the requirement, redesign the mechanism. The mechanism may
  still be disproportionate, but that is an **independent** criticism and must
  be argued on its own evidence.

## The worked example

`RV-353` charged the capsule programme (`F-5`) with welding separable concerns
together, treating one authority-boundary decision as though it settled every
dependent concern. In `F-7` the same review then argued that because environments
should be *selected* rather than *discovered* (`F-2`), the `[interpretation]`
policy and its 1,624 lines "cease to exist as problems" and deletion was "the
correct opening move."

Wrong, and wrong in exactly the charged way. A realised closure or base image
tells you *what software exists*. It does not answer whether trusted
orchestration may execute `cargo`, whether worker-controlled `build.rs` is
admissible, or whether a verification command crosses the authority boundary.
Those are questions about **who interprets worker-controlled content**, which is
orthogonal to how the environment was obtained. Any successor retaining a
trusted/untrusted split inherits the requirement whatever its provisioning
substrate. Caught by external review; recorded as `F-12`.

Note the near-miss: verifying that `src/interpretation.rs` had exactly one
consumer confirmed it was *mechanically* removable, which is a true fact that
answers a different question than the one being asked. Confirming removability
is not confirming that removal is right.

## Why it recurs

A root-cause finding is satisfying and compresses well, and compression is
precisely the failure mode — the conclusion inherits the root cause's
confidence without inheriting its evidence. The stronger a review's central
diagnosis, the more carefully its downstream deletions need separate warrant.

Related: [[mem.pattern.doctrine.conventions]].
