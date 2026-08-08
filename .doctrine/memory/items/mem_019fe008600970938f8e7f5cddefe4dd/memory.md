# Exporting a `pub(crate)` item from a package that has both a bin and a lib target

Adding `src/lib.rs` to a package whose `main.rs` keeps its own module tree means
each exported item is authored `pub` inside a module both targets declare. Under
this repo's lint config the *lib* target is green and the *bin* target is not,
and **every obvious remedy is separately denied**. Measured on the real tree
(SL-248 PHASE-01):

| arrangement | verdict |
|---|---|
| bare `pub` in a private `mod` | `unreachable_pub` (rustc, deny) in the bin |
| `#[expect(unreachable_pub, reason=…)]` | `unfulfilled_lint_expectations` in the **lib** — the lint never fires there |
| `#[allow(unreachable_pub, reason=…)]` | `clippy::allow_attributes` (deny) |
| `pub use` shim at the bin crate root | `clippy::pub_use` (deny, `Cargo.toml`) |
| per-item `#[expect(clippy::pub_use)]` on the `use` | `clippy::useless_attribute` — clippy permits lint attributes on `use` items only for a hardcoded allowlist, and `pub_use` is not on it |
| **`pub mod clock;` at the bin crate root** | **passes `build` and `clippy`, zero suppressions** |

**Use `pub mod`.** Declaring the module public at the crate root gives the item
the path out of the crate that `unreachable_pub` asks for. It is inert in a
binary (nothing can link one) and costs no attribute.

The lib side still needs `#![expect(clippy::pub_use, reason=…)]` as an **inner**
attribute at the top of `lib.rs` — a curated re-export set has no other spelling,
and an outer attribute on the `use` item hits `useless_attribute`.

**Second trap, same change.** The lib's module set must be transitively closed
over `crate::` paths *including* `#[cfg(test)]` code, and the closure is usually
deeper than it looks: a directory module drags whatever its sub-files import
(`kinds` → `kinds/resolve.rs` → `crate::fsutil`). Once closed, `unused = "deny"`
flags everything outside the export set as dead — live in the bin, dead in the
lib by construction — so `lib.rs` also needs
`#![expect(dead_code, unused_imports, reason=…)]`.

**`cargo build` will not find either problem.** `E0432` on the missing module
surfaces only when the library's own tests first compile, so gate the step with
`cargo test`, never `cargo build`.

See [[mem.pattern.lint.new-workspace-member-cargo-metadata]] for the sibling
trap when the second crate is added, and
[[mem.pattern.lint.clippy-denies]] for the deny list itself.
