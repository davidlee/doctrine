A capability token taken **by value** so a caller cannot use it twice:

```rust
fn roll_back(token: CreationToken) {
    let _removed = std::fs::remove_dir_all(&token.path);  // borrows
}
```

`clippy::needless_pass_by_value` (pedantic, denied here) fires: the argument is
passed by value but never consumed, so clippy suggests `&CreationToken` — which
would destroy the whole point, since a shared reference can be used any number of
times.

Destructure instead. The move is real, the lint is satisfied, and the signature
keeps its meaning:

```rust
fn roll_back(token: CreationToken) {
    let CreationToken { path } = token;
    let _removed = std::fs::remove_dir_all(path);
}
```

Same shape for any linear-type / RAII-adjacent token: if the by-value parameter
is the API contract, move a field out of it rather than borrowing through it.
