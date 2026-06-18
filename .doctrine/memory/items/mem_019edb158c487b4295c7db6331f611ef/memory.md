# dtoml::parse is the shared config reader — never eagerly validate optional sub-configs there

`dtoml::parse()` (src/dtoml.rs) is the SINGLE `doctrine.toml` reader. Its callers
span the config surface: `conduct::parse` (src/conduct.rs:138),
`verify::load_config` (src/verify.rs:146), and `coverage_store::load_config`
(src/coverage_store.rs:202) all go through it via `.conduct` / `.verification`.

**The trap:** adding an eager validation call for an OPTIONAL sub-config inside
`parse()` — e.g. `resolve_confidence(&doc.estimation)?` — propagates that
sub-config's validity into EVERY config read across unrelated commands. A malformed
`[estimation]` table would then fail `doctrine conduct`/`verify`/coverage loads,
coupling unrelated reads to a sub-config they don't even consume.

**Rule:** `parse()` should only deserialize. Validation of an optional sub-config
belongs to the consumer that actually needs it (call `resolve_*` at the use site,
not in the shared reader). "Purely informational until consumed" fields must stay
that way — do not give them a runtime effect at the parse boundary.

**Dead-code corollary:** optional sub-configs that are parsed now but consumed in
a later slice will be unused in non-test builds. Mark them with the existing
`#[cfg_attr(not(test), expect(dead_code, reason = "consumed by SL-NNN …"))]`
convention (same as `estimate::parse_optional`), not by inventing a fake consumer.

**Provenance:** SL-101 RV-085 F-1/F-2. The eager `resolve_confidence()?` in
`dtoml::parse()` coupled all config reads to estimation-config validity,
violating design §3.3 "no runtime effect in this slice". Fixed by removing the
eager call + marking the v1-unused facet API; regression test
`dtoml::tests::malformed_estimation_confidence_does_not_block_config_read` pins it.
