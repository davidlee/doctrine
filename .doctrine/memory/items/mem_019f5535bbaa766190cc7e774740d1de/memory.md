# unused=deny forbids staging dead code across phases — wire computed-but-unrendered values into a live path

`Cargo.toml` sets `unused = { level = "deny" }` (and `warnings = "deny"`), so a
`pub(crate)` fn/type used only by `#[cfg(test)]` code is a hard build failure
under plain `cargo clippy` / `cargo build` (the gate lints the bin, NOT test
targets — no `--all-targets`). This bites phased work: a phase that builds a seam
(detection, grading, a computed value) which the *next* phase will render cannot
leave it uncalled.

**Don't reach for `#[allow(dead_code)]`** (the project treats staged dead code as
drift — see the SL-101 unintegrated-facets lesson). Instead **wire the computed
value into a live non-test path now, just don't render it**: attach it to an
existing result struct as a field the current renderer ignores (no serde? then
zero output change; `#[serde(skip)]` otherwise), and have the later phase read
that field. The chain becomes reachable from a real command, the lint passes, and
goldens stay byte-identical because nothing new is emitted.

Concrete: SL-218 PHASE-02 computed graded tensions but render was PHASE-03. The
fix was `explain()` populating `Explanation.tensions` (unrendered) — that made
`detect`/`grade`/`graded_tensions` all live while `explain_human`/`explain_json`
output stayed identical. PHASE-03 renders the field.

Corollary: this is also a design signal — if a value has no live consumer at all
this phase, question whether the phase boundary is drawn right, or accept the
unrendered-field bridge as the explicit seam.
