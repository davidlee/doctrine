# rebuild-stale misses stale test binaries

`just rebuild-stale` does `touch src/main.rs && cargo build` — it forces a fresh
**bin** only. Under the shared `CARGO_TARGET_DIR`, cargo's fingerprint can still
serve a stale **test** artifact: a test binary compiled from old source survives the
bin rebuild untouched.

**Tell:** a `cargo test` / `just gate` run executes a test whose name is **not in the
current source** (a deleted/renamed test still running), or a test reads a path/fixture
from a tree that no longer matches HEAD. Confirmed this session: `just gate` red on
`e2e_dispatch_sync::record_boundary_also_writes_the_arm_neutral_registry`, a name
absent from the source — a stale test binary, not a real regression. Compounded when
the test shells the jail's RO `~/.cargo/bin/doctrine` (also stale, and unbuildable
in-jail — use the build-target bin).

**How to apply:** to clear it, retouch the **specific test source** (`touch
tests/<file>.rs`) or `cargo clean -p doctrine`, then re-run — don't trust the RED.
`rebuild-stale` is not sufficient for test-binary staleness.

This is footgun #2 — the stale-artifact axis of the shared target, tracked by IMP-004.
Distinct from the path-baking axis (footgun #1), fixed in CHR-014 and recorded at
[[mem.fact.testing.runtime-manifest-dir]].
