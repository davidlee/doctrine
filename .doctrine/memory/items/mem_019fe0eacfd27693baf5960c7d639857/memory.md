# Adding a CLI subcommand variant requires classifying it in `guard.rs`

`src/commands/guard.rs` classifies every command as `Read` or `Write(<name>)`
for the worker-mode write guard, and it does so with an **exhaustive** match
per command enum. So adding a variant to `KnowledgeCommand`, `BacklogCommand`,
`SpecCommand`, … is a **compile error** there until you classify it:

```
error[E0004]: non-exhaustive patterns: `&KnowledgeCommand::Edit { .. }` not covered
   --> src/commands/guard.rs:212:49
```

That is the design working: a new write verb cannot silently escape the guard.
The fix is one line, e.g.

```rust
KnowledgeCommand::Edit { .. } => Write("knowledge edit"),
```

The label string is what the refusal message names, so it should read as the
CLI invocation (`"knowledge edit"`), not as the function.

Worth knowing before you plan a phase that adds a verb: `guard.rs` is a seam
the verb's own module never mentions, so it is easy to leave out of a reading
list and then meet as a surprise at first build. (SL-249 PHASE-08 met exactly
this.)
