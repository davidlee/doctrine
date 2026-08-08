Read out of `clap_builder-4.6.0`'s own source during SL-249 PHASE-04 planning,
not recalled.

## What the parser actually does

`src/parser/parser.rs:113-128` — for every token, while parse state is not
mid-option, clap tests the token against the subcommand table *before* it is
offered to positional filling. `:584-592` `possible_subcommand` returns a match
on an exact subcommand name (or an inferred prefix, if inference is on), and
suppresses that lookup only when `args_conflicts_with_subcommands` is set AND a
valid arg was already found.

So `cmd <subname> …` dispatches to the subcommand and `cmd <other> …` falls
through to the positional. **Coexistence parses.**

## What actually breaks

Requirement validation. `src/parser/validator.rs:55` skips the required-args
check only when `subcommand_negates_reqs` is set and a subcommand matched. A
**required** positional sharing the slot with a subcommand name is therefore the
joint that fights — which is what `src/commands/compare.rs:18-26` recorded for
SL-210 (`<A>`/`<B>` were required, and `compare list` demanded them).

The note in `compare.rs` is correct about what it hit, and easy to over-read as
"clap cannot do this at all". It can, when the positional is `Option<T>`.

## The shape that works

```rust
Edit {
    id: Option<String>,                       // NOT String
    #[command(subcommand)] sub: Option<SubEnum>,
    // … the parent's own flags …
}
```

with **neither** `args_conflicts_with_subcommands` nor
`subcommand_negates_reqs` set — with nothing required, there is nothing to
negate. A missing `id` becomes the command's own worded refusal instead of
clap's.

Precondition worth checking before you rely on it: the positional's value space
must not collide with any subcommand name. (In doctrine's case ids are
`XXX-NNN`-shaped, so they cannot.)

Note also `debug_asserts.rs:679-689` — the only build-time panic in this area is
about a **`last(true)`** required positional plus subcommands, not about
required positionals generally.
