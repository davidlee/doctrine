## The rule

This workspace denies `clippy::use_debug` (`Cargo.toml`, "Debug leftovers"
block, alongside `dbg_macro` / `print_stdout` / `print_stderr`). The lint's
subject is **debugging remnants reaching an output handle**, so it fires on the
`write!` / `writeln!` / `print!` / `println!` family and **not** on `format!`.

So this is a hard gate error:

```rust
let _written = write!(buffer, "row {id:?}={row:?}");
```

and this, three functions away in the same file, has passed every gate since it
landed:

```rust
let mut rendered = format!("provision refused: {refusal:?}");
```

## Why it bites

The two shapes are interchangeable to a reader and the file will usually already
contain the permitted one, so the natural move — extend the existing
buffer-and-`write!` renderer with a `{:?}` — reads as consistent with its
neighbours and fails the gate. `cargo check` does not catch it; only clippy
does, which in this repo means a full `doctrine check gate` cycle.

## What to do instead

Build a `Vec<String>` of lines with `format!` and `join("\n")` them. It is
better code than the buffer it replaces: no `let _written =` bindings to
swallow, no `use std::fmt::Write as _`, and the result reads the way the test's
expected value is written.

```rust
let mut lines = vec![format!("outcome={}", render_outcome(v))];
lines.extend(v.rows.iter().map(|(id, row)| format!("row {id:?}={row:?}")));
lines.join("\n")
```

An `#[expect(clippy::use_debug, reason = …)]` is the wrong reach here: the lint
is right that a `Debug` rendering should not go straight to a handle, and
`allow_attributes` is itself denied.

## Related

Rendering derived `Debug` rather than a hand-written name table is often the
*correct* choice — see SL-248 `EX-15` / `sec-9` `R9`, where a match arm per enum
variant would have been a second unchecked projection of a normative table. The
lint does not object to that; it objects to where the string goes.
