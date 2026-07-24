# IMP-312: SL-227 library and install test-fidelity hardening

Surfaced by the SL-227 post-implementation audit (RV-302). Four test-fidelity
gaps — the runtime behaviour is verified-correct, but the tests that guard it are
weaker than the invariants they claim. None blocked SL-227 close; batched here as
hardening.

## Items

- **RV-302 F-8 (major) — VT-6 NF-002 byte-unchanged test is vacuous.**
  `src/commands/library.rs` `every_verb_leaves_the_repo_byte_unchanged` creates a
  `tempdir()` but never operates any verb against it (verbs render into an
  in-memory `Vec` sink over the embedded resolver), then asserts the untouched
  temp dir is empty — trivially true. It also discards the three failure `Err`s
  with `let _ =` and never exercises the malformed-policy load class. Rewrite:
  run each verb against a real repo root, snapshot the tree bytes before/after,
  and assert each of the four failure classes (incl. malformed-policy).

- **RV-302 F-7 (major) — crux reachability gate is not staleness-proof.**
  `src/install.rs` `every_unprojected_embed_is_a_published_backing` reads both the
  install enumeration (`embedded_filenames()`→`InstallAssets::iter()`) and the
  published set (`PublicationManifest::load()`→`PublicationAssets` embed) from the
  compiled RustEmbed (`debug-embed` on), so an incremental `cargo test` after a
  lone `install/` asset edit can inspect a stale asset set and false-green. The
  guarantee holds at close (fresh build) and the current source-set is sound, but
  the gate does not follow design §8 R3's disk-source pattern — which its sibling
  admission gate (VT-3, via `publication_manifest_bytes`'s documented
  `CARGO_MANIFEST_DIR` read) already uses. Harden the gate to enumerate + admit
  from disk-source.

- **RV-302 F-10 (minor) — VT-5 proves detection, not adapter installation.**
  `src/install.rs` `detect_agents_is_independent_of_the_base_projection_flip`
  asserts only that `detect_agents()` returns `["claude"]` after a base install;
  it never fires the forward per-agent adapter-install leg or checks an adapter
  lands. Extend it to run the forward leg and assert an adapter file lands
  post-flip (the actual NF-004 claim).

- **RV-302 F-6 (nit) — duplicated base-backing literal.**
  `src/publication.rs` `published_set_covers_the_full_projection_complement`
  hardcodes `const BASE_BACKINGS: [&str; 3]`, duplicating the authoritative
  `install/manifest.toml [base].backings` (STD-001). Derive the base from the
  manifest like the crux gate does (fails safe today — false-red on a 4th base
  backing — but drifts from single-source).

## Provenance
Audit RV-302 (SL-227), external adversarial pass by codex/GPT-5.5 (F-7..F-10) +
self-audit (F-6). See `.doctrine/review/302/`.
