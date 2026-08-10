# ISS-330: doctrine config set panics in any debug build — clap required/required_unless conflict

`doctrine config set` panics before it parses anything, in any build with
`debug_assertions` on (i.e. every `cargo build` / `just check` binary):

```
$ ./target/debug/doctrine config set --help
thread 'main' panicked at clap_builder-4.6.0/src/builder/debug_asserts.rs:202:13:
Argument key: `required` conflicts with `required_unless*`
```

## Cause

`src/commands/config.rs:34` (and the same shape at `:52`, `:75`):

```rust
/// Coefficient key …
#[arg(required_unless_present = "tag")]
pub(crate) key: String,
```

`key` is a **non-`Option` positional**, so clap-derive marks it `required`.
`required_unless_present` is a *conditional* requirement, and clap 4.6's
`debug_asserts` refuses an argument that is both unconditionally required and
conditionally required. The intent — "required unless `--tag` is given" — needs
`key: Option<String>`, exactly as `value` beside it would also need.

## How it surfaced

SL-249 PHASE-04's `VT-4` introspects the built command tree
(`<Cli as CommandFactory>::command()` then `Command::build()`) to assert each
`knowledge edit <kind>` subverb's flags equal that kind's `facet_fields` row.
`build()` recurses the whole tree, so the `config` subtree's debug assert fires
and the test panics on an unrelated command. The test works around it by
cloning and building only the `knowledge` subtree; that workaround should be
removed once this is fixed.

## Why it was not caught

Nothing in the suite builds the *whole* command tree — `write_class` tests and
`render_boot_map` walk declared subcommands without `build()`, and clap builds
lazily along the parse path, so only an actual `config set` invocation reaches
it. No test invokes `config set`.

## Fix sketch

Make the three conditionally-required positionals `Option<T>` and unwrap them
in the run shell (or drop `required_unless_present` and refuse in the shell,
which is where the `--tag` branch already lives). Add one argv-driven test per
`config` verb so the debug assert is exercised.
