# String assembly under repo clippy denies

Build Strings via Vec<String>+concat — push_str(&format!) and write!().expect() both denied

## What

Building a multi-line `String` in this repo is hemmed in by `Cargo.toml [lints]`:

- `clippy::format_push_string` (deny) — kills `out.push_str(&format!(…))` and
  `out += &format!(…)`.
- `clippy::expect_used` + `clippy::unwrap_used` (deny, **non-test code too** — not
  only under `--all-targets`) — kills `write!(out, …).expect(…)` / `.unwrap()`.
- `clippy::let_underscore_must_use` (deny) — kills `let _ = write!(out, …)`.

So all the usual infallible-`fmt::Write`-to-`String` idioms are closed off at once.

## How to apply

Use the house style (see `src/retrieve.rs::format_find`, `src/spec.rs::render`):
build a `Vec<String>` of pre-formatted pieces and join/concat:

```rust
let mut parts: Vec<String> = Vec::new();
parts.push(format!("line {x}\n"));        // Vec::push is NOT the lint
if cond { parts.push("literal\n".to_string()); }
parts.concat()                            // or parts.join("")
```

`Vec::push(format!(…))` is fine — the lint only fires on `String::push_str`/`+=`.
Pure compose fns return the `String` directly; the impure shell does the single
`write!(io::stdout(), "{}", doc)?` (and `?`-propagates `fmt::Error` via `anyhow`).

Related: [[mem.pattern.lint.clippy-denies]], [[mem.pattern.lint.expect-not-allow]].
