# `module_name_repetitions` is publicity-gated, so it lands with the export, not with the type

`clippy::module_name_repetitions` is `deny` here (`Cargo.toml` `[workspace.lints.clippy]`,
via `pedantic`). It fires on an item whose name starts or ends with its
containing module's name — but **only for items public outside the crate**.

That is why `dispatch_config::DispatchConfig`, `install_config::InstallConfig`
and `reserve::ReservationConfig` have sat in same-named modules for the whole
life of this repo without a single `#[expect]` for the lint anywhere in the
tree: they are all `pub(crate)`.

**SL-248 PHASE-02 was the first item to cross.** `pub mod interpretation;` in
`src/lib.rs` plus `pub struct InterpretationPolicy` fires it at once. The lint
covers type-like items only — `pub const INTERPRETATION_SCHEMA` in the same
module does **not** trip it.

**Consequence for anyone widening the root library's export set** (`EXPORTED` in
`tests/architecture_layering.rs`): promoting an existing `pub(crate)` type to
`pub` can red the lint leg on code that has been green for years, and the
diagnostic points at the type rather than at the export that changed its
visibility. Expect it; do not read it as a defect in the type.

**Remedy, in order of preference:** rename if the name is yours to choose;
otherwise item-level `#[expect(clippy::module_name_repetitions, reason = "…")]`
naming whatever fixes the name (a design or an exit criterion).
`#[allow]` is not available — `clippy::allow_attributes` is also `deny`
([[mem.pattern.lint.expect-not-allow]]). Never module-blanket it.
