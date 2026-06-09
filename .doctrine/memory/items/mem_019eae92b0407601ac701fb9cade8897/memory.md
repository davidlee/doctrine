# Pinning a doctrine CLI surface byte-exact with black-box goldens

To prove a CLI surface is unchanged across a refactor (behaviour-preservation
gate), spawn the BUILT binary (`env!("CARGO_BIN_EXE_doctrine")`) and assert
byte-exact stdout/stderr — the in-module unit tests write to `io::stdout()`
WITHOUT capturing it, so they prove helper self-consistency, not the CLI surface.
Prior art: `tests/e2e_adr_cli_golden.rs` (SL-030 PHASE-01),
`tests/e2e_list_conformance.rs`.

Three gotchas that make or break determinism:

- **Hand-seed fixtures with FIXED dates — never `adr new`/`adr status` to build
  one.** Those verbs stamp `clock::today()` into `created`/`updated`, so any
  golden built through them is non-deterministic. Write the `adr-NNN.{toml,md}`
  tree directly under a tempdir (`seed(root,id,toml,md)` helper).
- **Error stderr = anyhow `Debug` shape.** A sourced error prints
  `Error: <ctx>\n\nCaused by:\n    <source>\n`; a bare `bail!` (e.g.
  malformed-refuse) prints `Error: <msg>\n` with NO `Caused by`. Pin the exact
  shape, including the 4-space source indent.
- **Carve out absolute paths.** Errors like `… not found at <ABSOLUTE>/path`
  embed the tempdir, which floats per run — assert the stable prefix
  (`starts_with`) + the relative suffix (`contains`), not full-stderr equality.
  Likewise a real `status` transition bumps `updated`→today: assert it MOVED off
  the seeded value, don't byte-pin it.

JSON surface specifics: `serde_json` pretty output is **BTreeMap key order**
(alphabetical — no `preserve_order` feature) and `write!`/`to_string_pretty`
emit **no trailing newline**. A dynamic object key (governance kinds key the doc
under `stem`, e.g. `"adr"`) forces a hand-built `serde_json::Map`, not the
`json!` macro.

Reusable for any new governance kind's goldens (e.g. `src/policy.rs`, SL-030
PHASE-03). Adjacent: [[mem.pattern.testing.conformance-asserts-surface-not-just-envelope]],
[[mem.pattern.testing.stale-cargo-bin-exe]].
