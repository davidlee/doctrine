# Dispatch worker fork omits gitignored build artifacts → spurious gate/test failures

A `doctrine worktree fork --worker` provisions only a small allowlist (the env
contract + a couple of tracked files — "N copied"); it does **not** copy gitignored
**build artifacts**. `web/map/dist/` is the live example: a gitignored, externally-built
(npm) directory that `src/map_server/assets.rs` embeds via `#[derive(RustEmbed)]
#[folder = "web/map/dist/"]`.

**Symptom.** In a worker fork, `web/map/dist/` is empty, so RustEmbed embeds nothing.
The crate still **compiles** (empty embed folder is legal) but three runtime tests fail:
- `map_server::assets::tests::assets_get_index_html`
- `map_server::assets::tests::serve_embedded_index_returns_html`
- `map_server::routes::tests::index_returns_200_html`

Separately, `just gate` runs `lint-js` (`npx eslint web/map/`) which fails in the
offline jail (node ESM loader error) and aborts the gate **before** `test-all`/`build`.

**Why it's a trap for the orchestrator.** These failures are pure fork-environment
artifacts — they are absent on the coordination tree (`.dispatch/SL-<n>`), which holds a
real `web/map/dist/`. A Rust-only delta (e.g. SL-133 PHASE-04, which touched only
`src/priority` + scan/facet/config) cannot cause them; seeing them after a clean delta
must NOT be read as a regression.

**How to apply.**
- Don't run the funnel verify inside the worker fork and trust a map_server / lint-js
  RED. Either skip `lint-js` (run `cargo clippy` + `cargo test --bin doctrine` + `cargo
  build` directly) or, better, do the funnel **import onto the coordination tree** and
  verify there — it has the embed, so the 3 failures vanish and the real signal shows.
- Decisive disambiguation: run the suspect tests on base `B` (the coord HEAD, no delta).
  Green on `B` + delta touches no related code ⇒ environmental, proceed.
- doctrine is a **bin crate**, not a lib: filter with `cargo test --bin doctrine <name>`;
  `cargo test --lib` errors "no library targets found".

Sibling of [[mem_019eeac33cf373d3949d04a6f9780351]] (same root cause on the *coordination*
worktree, where the missing artifact fails to **compile** rather than fail a runtime
test). Generalises the nix-build observation in `crane-strips-non-rust-embeds`. See also
[[mem_019ebc8e4662732185d4356a9a0f0ad6]] (stale `CARGO_MANIFEST_DIR` fork binaries).
Surfaced by SL-133 PHASE-04 dispatch funnel.
