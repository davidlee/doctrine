Two things bit SL-249 PHASE-04 inside twenty minutes, both about putting a clap
command tree under a test's microscope.

## 1. `Command::build()` is whole-tree, so it fires *other* commands' asserts

To assert a subverb's declared flags (`get_arguments()` → `get_long()`), the
command must be **built** — clap's auto-generated `--help` arg does not exist
until `_build_self` runs. The obvious spelling is:

```rust
let mut cli = <Cli as CommandFactory>::command();
cli.build();                              // ← recurses EVERY sibling subtree
```

`Command::build` calls `_build_recursive`, which builds every descendant and
runs each one's `debug_asserts`. So an unrelated command's latent declaration
bug panics *your* test, with a message that names neither your command nor
theirs:

```
Argument key: `required` conflicts with `required_unless*`
```

That was `doctrine config set` (ISS-330) — a non-`Option` positional carrying
`required_unless_present`, which also panics on any real invocation in a
debug build, but had no test to notice.

**Narrow the build to the subtree you are introspecting:**

```rust
let mut sub = <Cli as CommandFactory>::command()
    .find_subcommand("knowledge").expect("…").clone();
sub.build();                              // only this subtree's asserts run
```

The corollary is the useful half: `cli.build()` in *one* test is a free
whole-tree declaration audit. Worth having deliberately, once, rather than
discovering it sideways.

## 2. A wide nested `Subcommand` enum trips `clippy::large_enum_variant`

Nesting `#[command(subcommand)] facet: Option<KnowledgeFacetEdit>` inside a
parent variant inlines the widest facet variant (8 × `Option<String>` plus a
flattened target) into the parent enum: 384 bytes against a 152-byte
second-largest. `warnings = "deny"` makes that a build failure.

`Option<Box<KnowledgeFacetEdit>>` fixes it and costs nothing at the derive —
clap implements `Subcommand for Box<T>` (and `Args for Box<T>`),
`clap_builder/src/derive.rs:366,375`. Auto-deref means call sites are
unchanged.

## See also

[[mem.fact.clap.subcommand-vs-optional-positional]] — why the subverbs may sit
beside the kind-blind positional at all.
