# Module split under a layering.toml umbrella requires mixed-umbrella sub-classification entries or just gate fails

Splitting a module into a submodule folder (or adding a sub-file) under a unit
classified in `.doctrine/adr/001/layering.toml` makes it a **mixed umbrella** and
**requires** new `"module::file"` sub-classification entries — or `just gate`'s
`MixedUmbrella` assertion (`tests/architecture_layering.rs`) goes RED.

- **Recurring design omission — caught three times:** SL-132 (RV-121), SL-133
  (RV-130 F-1), SL-116 (RV-131 F-3). Designers keep treating module splits as
  "pure mechanical." They are not — **the binding tier map is a first-class
  deliverable of any module-split slice.**
- **Tier = HIGHEST altitude of any non-test file** in the unit (most-knowing-wins).
  `leaf` = std+leaf only; `engine` = engine+leaf, NEVER a command module;
  `command` = reaches a command module. A file importing a command module (e.g.
  `crate::slice::run_phases`) is **command**, not engine — don't label by cohesion.
- The umbrella top entry (`mod.rs`'s tier) stays; only files that differ get a
  `"module::file" = "tier"` row. Precedent: `"catalog::scan" = "command"`,
  `"catalog::hydrate" = "engine"`, … in `layering.toml`.
- **Regenerate authoritatively, don't hand-guess:**
  `cargo test --test architecture_layering dump_real_graph -- --nocapture --ignored`
  (the same extractor that built the map, SL-112).
- Add `just gate` (`MixedUmbrella`) green as an explicit slice **exit criterion**.

See [[mem_019ee0545f3a7382ad14eb11a2f3887a]] (the SL-112 layering gate) and
[[mem_019ee7d35d0d7872a6218ffcfb0b9f47]] (crate-root type edges are not module
edges — a gate pre-filter subtlety).
