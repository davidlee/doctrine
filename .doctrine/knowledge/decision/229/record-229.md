# DEC-229: Enum pin is an exhaustiveness barrier, not a generated type

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What `DEC-221` got right, and the one thing it could not have known

`DEC-221` reasoned correctly about *oracles*. A struct's field set is reachable
from one serialised value and an enum's variant set is not, so the two halves
need different instruments; tier-2 equality has no oracle for a variant set at
all, which is why `SL-244` generates rather than asserts. None of that is
disturbed here.

What `DEC-221` did not distinguish is **ownership** from **the guarantee**.
`condition_vocabulary!` (`gate.rs:438`) owns the `Condition` type: from one
edge-keyed row list it emits the enum definition, `ALL`, `as_str`, the per-variant
serde renames, `CONTRACTS` and `boundary_conditions`. `SL-244` could take that
route because `SL-244` introduced `Condition`.

`DEC-227`'s closure is not like that. `Dispose`, `DelegationAct`, `AgentAct`,
`ActKind`, `Stage`, `Posture`, `Authority` and the rest already exist, most carry
data on their variants, and all of them are shared machinery under the
behaviour-preservation gate. Regenerating their definitions would be the largest
and riskiest change in a slice scoped as a documentation surface, and it would
put `R3`'s clippy blindness over real logic rather than over a table.

## The separation

Read `condition_vocabulary!`'s own doc comment for what it is actually buying
(`gate.rs:450-452`): grouping rows by edge *"closes the last hole — an edge left
unguarded makes the generated match non-exhaustive, a build failure."*

The guarantee is the **non-exhaustive match**, not the generated type. And a
non-exhaustive match can be generated over a type the macro does not own.

```rust
payload_variants! {
    Dispose {
        Create      = "create",
        Adopt       = "adopt",
        Unresolved  = "unresolved",
        NonDurable  = "non-durable",
    }
}
```

emits `VARIANTS: &[&str]` for the contract table, and:

```rust
const fn _pin(value: &Dispose) {
    match value {
        Dispose::Create     { .. } => {}
        Dispose::Adopt      { .. } => {}
        Dispose::Unresolved { .. } => {}
        Dispose::NonDurable { .. } => {}
    }
}
```

The braced pattern `Variant { .. }` is uniform across unit, tuple and struct
variants. That was verified against the compiler rather than assumed, and it is
what lets one macro shape cover every enum in the closure regardless of what its
variants carry.

## Why this is strictly better here

Both directions fail at compile time, which is what `DEC-221` wanted:

- a variant added to the enum but absent from the row list leaves the generated
  match non-exhaustive;
- a row naming a variant the enum does not have is an unresolved path.

And the costs `DEC-221` weighed mostly evaporate. The generated region is a token
array and a match with empty arms, so there is nothing inside it that could hold
a bug for clippy to have missed — `R3` is satisfied rather than accepted. No type
definition moves, so the behaviour-preservation gate on shared machinery is met
by construction rather than by argument.

## The residue

The barrier pins the **variant set**. It does not pin that each token string
equals serde's actual rename: a `#[serde(rename_all = …)]` change, or a
per-variant `rename`, could drift a literal while the match stays exhaustive.

That closes by a per-variant round trip — serialise a value of each variant,
assert the emitted token matches the row. Worth noting *why* a round-trip test is
sufficient here when an assertion was insufficient in `DEC-221`'s analysis: the
exhaustiveness barrier guarantees the test has a case for every variant. An
assertion without the barrier could pass while silently testing a subset. The two
mechanisms compose; neither would do on its own.

## Scope of the supersession

Enums only. `DEC-221`'s struct half — serde-key-set equality against an
exhaustive no-`..` literal, on `SL-249`'s `I9` pattern — is untouched and still
carries the thirteen struct types in `DEC-227`'s closure.

`DEC-221` was recorded as held loosely at the user's explicit direction, with
revision expected rather than exceptional. This is that revision, and it arrived
from the direction `DEC-221` named: drafting against the real types found the
mechanism costlier and less necessary than it looked from the inquiry.
