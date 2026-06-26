# SL-162 notes — runtime-resolve test binary path

## Closure (RV-171)

Clean audit. 59-file `const BIN = env!("CARGO_BIN_EXE_doctrine")` sweep replaced
with the runtime `test_support::doctrine_bin()` resolver (`current_exe()` sibling),
extending the CHR-014 pattern from `CARGO_MANIFEST_DIR` to the bin path. Guard
`e2e_no_baked_manifest_dir.rs` → `e2e_no_baked_paths.rs`, generalised to ban both
macros via fragment-assembled needles.

Behaviour-preservation gate held: every swept e2e suite green, goldens
byte-identical. Conformance clean (0 undeclared / 0 undelivered / 63 conformant).

## Durable carry-forward

- **VH-1 unexercised in-jail.** The cross-namespace proof (run a formerly-failing
  suite in one namespace, then the other, no recompile) cannot be demonstrated from
  a single namespace. In-jail correctness is by construction (no baked path); the
  only direct proof is a human cross-namespace run. Accepted, marked VH.
- **Lost `CARGO_BIN_EXE_*` build-graph link.** A missing bin now surfaces as a
  runtime spawn error, not a link error — mitigated by running via `cargo test`.

## Dependency surfaced (out of scope)

- **ISS-054** — `main` is red on `e2e_estimate_non_blocking::no_facet_symbol_outside_allowlist`
  (`src/knowledge.rs` outside the NF-001 facet allowlist). Pre-existing, unrelated
  to SL-162 (the slice touches neither file). `just check` is red on this alone, not
  on any SL-162 surface. Fix before promoting edge→main.
