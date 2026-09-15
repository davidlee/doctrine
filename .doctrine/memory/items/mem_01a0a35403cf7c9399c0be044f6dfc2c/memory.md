## The shape

```rust
for plan in batch {
    execute(plan)?;   // journals, claims an id, writes authored bytes
}
```

where `execute` opens with a *check* — a guard whose inputs are the item and
some state read before the loop. Per item it looks impeccable: "refuse before any
effect of this item". Across the batch it is not. Item 1 completes its effect,
item 2's guard refuses, and the caller reports a failure over a tree that moved.

Single-item batches hide it completely, which is why it survives: every fixture
declares one.

## The test that finds it

Two items where the FIRST is fresh and the SECOND trips the guard, then assert on
the artefact count, not the error:

```rust
let minted = || std::fs::read_dir(&records).map_or(0, Iterator::count);
let before = minted();
let error = apply(..).unwrap_err();
assert_eq!(minted(), before, "no record for the plan ORDERED AHEAD of the one that refused");
```

Asserting the error alone passes on the broken tree. (Count entries, not records —
a mint here writes a directory *and* a slug symlink, so the delta is 2 per record.)

## The fix

Hoist the predicate to a pre-pass over the whole batch, and **move** it rather
than copying it — leave the loop reading whatever state it needs to route itself,
but not re-asking the question. Probe it: make the hoisted guard never refuse and
confirm the *pre-existing* single-item pin goes red too. If it stays green, a
second copy was left behind.

## Why this is not automatically a spec violation

`SPEC-029` here says the guarantee is *the run does not advance*, not that
nothing was written — journalled effects deliberately remain and stay
recoverable, because promising "nothing was written" invites a cleanup path that
deletes authored knowledge. So the hoist is worth doing on its own terms (an
avoidable authored write on a doomed submission) rather than as a repair. Check
what the governing spec actually promises before writing "violation" anywhere.

Found: SL-259 PHASE-06, `commands::design::execute_mint`'s step-1 retry guard,
now `refuse_unresumable_mints`. Related:
[[mem.pattern.testing.replay-cannot-prove-idempotence-behind-a-state-guard]].
