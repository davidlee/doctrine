# anyhow blanket From impl covers custom error types

When you add a domain error enum to a Rust project that already uses `anyhow`:

- `anyhow` has a blanket `impl<E: StdError + Send + Sync + 'static> From<E> for anyhow::Error`.
- If your enum already derives `Debug` and you write manual `Display` + `Error`
  impls, and the type is `Send + Sync + 'static` (true for almost all enums
  carrying `String`/simple fields), you get `From` and `.into()` for free.
- **Do NOT write a manual `impl From<YourError> for anyhow::Error`** — it will
  conflict with the blanket impl at compile time (E0119).
- `downcast_ref::<YourError>()` works on the resulting `anyhow::Error` because
  the blanket impl preserves the concrete type (unlike `anyhow::Error::msg()`
  which stringifies).

## Example (from SL-109 PHASE-01)

```rust
#[derive(Debug)]
enum ReviewError {
    NotFound { reference: String },
    // ...
}

impl fmt::Display for ReviewError { /* ... */ }
impl std::error::Error for ReviewError { /* ... */ }

// NO manual From impl needed — the blanket covers it.

// Conversion works:
let err = ReviewError::NotFound { reference: "RV-001".into() };
let anyhow_err: anyhow::Error = err.into();
let downcast: &ReviewError = anyhow_err.downcast_ref::<ReviewError>().unwrap();
```

## Source

Discovered during SL-109 PHASE-01 implementation (2026-06-19). The initial
manual `impl From<ReviewError> for anyhow::Error` conflicted with anyhow's
blanket; removing it and relying on the blanket impl made both `.into()` and
`downcast_ref` work correctly.
