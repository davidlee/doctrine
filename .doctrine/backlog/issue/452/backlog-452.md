# ISS-452: Opening a review pass emits no change row

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

`src/design_run/run.rs:494-501` assigns `next.review.pass` on entry to
`Stage::Reviewing` — a state change — and emits **no change row**.
`ChangeEvent` has no opening counterpart. It has `ReviewDisposed`, whose own
doc says the row exists *so the choice is legible in the log rather than only
inside the snapshot*; the identical argument applies to the opening and is not
made.

```rust
if next.run.stage == Stage::Reviewing
    && prior.run.stage != Stage::Reviewing
    && let Some(review) = resolved.review_pass.clone()
{
    next.review.pass = Some(ReviewPass::over(...));
}
```

The `let Some(review)` guard is a second instance of the same shape: a
resolution carrying no review ref silently no-ops the assignment rather than
refusing it.

## Why it is filed rather than fixed

`SL-259` (*Truthful apply*) made `DEC-248` the rule — *a material change is
emitted by the code that performs it* — and closed the two instances it had
witnesses for (`ISS-367`, `ISS-450`). This site was not one of its four legs and
the slice did not touch it.

Two partial mitigations, neither sufficient on `DEC-248`'s own terms:

1. The `StageMoved` row records the transition that causes the opening. That is
   the same coverage argument that was available for `ISS-450` (`NodeCreated`
   covers the `needs` edge it landed) and was rejected there.
2. A pass that opened is visible in the snapshot. `ReviewDisposed` exists
   because the snapshot alone was judged insufficient for the closing half.

## Provenance

Raised as `RV-366` `F-7` — the `SL-259` implementation audit — and disposed
`follow-up`. `SL-259`'s `design.md` `sec-9` closes by predicting exactly this:
*"the classes here are the ones visible from inside a single run's worth of use.
Others may not be."* This item is that prediction landing on a concrete site.
