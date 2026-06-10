# SL-038 Design — cordage scale harness

Status: draft (design stage). Canonical for design intent; scope in `slice-038.md`.

## 1. Purpose

Land a committed, reproducible regression gate + findings note for three already-
confirmed scale cliffs (RSK-002, RSK-003). Measure-and-red only. The probe that
confirmed the numbers (2026-06-11) was deleted; this slice makes the evidence durable
and turns it into reds the eventual fixes green.

## 2. Constraints (canon)

- **Zero-dep** — `crates/cordage/Cargo.toml` gains no `[dependencies]`. std-only
  harness. (D1 in scope.)
- **Pure/imperative split** — `std::time::Instant`, `std::process`, `std::env` live
  only in the example/test consumer, never in cordage `src/`. cordage stays pure.
- **Leaf (ADR-001)** — the harness consumes cordage's public API; nothing in `src/`
  learns about it.
- **Black-box** — generators use only the public builder surface (`GraphBuilder`,
  `OverlayConfig`, `EdgeAttrs`, `OrderSpec`/`OrderLayer`, `explain`/`reachable`/…),
  mirroring the existing `tests/*.rs` style.

## 3. Components

```
crates/cordage/
  examples/scale_harness.rs   NEW  arg-driven measurement binary; subprocess target
  tests/scale_cliffs.rs       NEW  #[ignore] red tests
.doctrine/slice/038/notes.md  NEW  findings: confirmed numbers + harness as reproducer
```

No shared generator module: each generator is a ~10-line public-API loop. The
example owns the canonical copies; the tests that need a graph in-process
(`explain`) build their own tiny inline graph (the diamond is ~8 lines). Minor,
bounded duplication is preferred over an `#[path]`/module hack to share code between
an `examples/` bin and a `tests/` bin (they cannot import each other). If the
duplication exceeds ~3 generators, revisit — but at this size it does not.

## 4. Generators (public-API only)

Signatures (in `examples/scale_harness.rs`; the diamond reappears inline in the test):

```rust
// Linear spine 0→1→…→(n-1) on one Reject overlay referenced by the order spec.
// Drives Tarjan + level_of recursion depth = n. Returns the built Graph (or the
// build attempt — at large n this is where the SIGABRT happens, by intent).
fn deep_chain(n: u32) -> cordage::Graph;

// `layers` diamond stages: each stage splits to 2 then rejoins. Predecessor-path
// count from source to sink = 2^layers (exact, deterministic). Acyclic.
fn diamond(layers: u32) -> (cordage::Graph, /*source*/ NodeId, /*sink*/ NodeId, OverlayId);

// One Evict overlay carrying a dense cycle over `nodes` vertices (`edges` ≈ a near-
// complete digraph), forcing the eviction-to-fixpoint pass to recompute SCCs per
// evicted edge. Drives the RSK-003 quadratic.
fn dense_evict(nodes: u32, edges_per_node: u32) -> cordage::Graph;
```

## 5. Measurement example — CLI contract

```
scale_harness --cliff overflow  --n N           # build deep_chain(N); may SIGABRT
scale_harness --cliff explain   --layers L      # build diamond(L); print path count + time
scale_harness --cliff quadratic --n N           # build dense_evict(N,..); print build time
```

- Parses args via `std::env::args` (no clap — zero-dep). Unknown args → nonzero exit
  with a usage line on stderr.
- Prints one CSV line to stdout: `cliff,param,metric,value` (e.g.
  `explain,24,paths,16777216`). Timing via `std::time::Instant`.
- The `overflow` path simply builds and exits 0 if it survives; if the recursion
  overflows, the runtime aborts the process (rc 134) — that *is* the signal the
  overflow test reads. No catch, no special handling.

## 6. Red tests — `tests/scale_cliffs.rs` (all `#[ignore]`)

### 6.1 explain — exact, deterministic (RSK-002)
Builds `diamond(L)` inline, asserts the path count is exactly `2^L`:
```rust
#[test] #[ignore = "exponential; demonstrates RSK-002, not a gate run by default"]
fn explain_path_count_is_exponential_in_diamond_depth() {
    let layers = 18;                          // 2^18 = 262_144 paths — the test itself
    let (g, _src, sink, ov) = diamond(layers); // must not OOM (each path ~2L NodeIds;
    let ex = g.explain(sink);                  // ~300–500MB already at layers=20)
    let n = ex.paths().get(&ov).map(Vec::len).unwrap_or(0);
    assert_eq!(n, 1usize << layers);          // exact: proves 2^layers growth
}
```
No timing, no flake — the count itself is the proof. `layers = 18` keeps the *test
process* tractable (~100MB) while the 2^layers curve is unmistakable; the manual
example (§5) can push higher to show the OOM cliff.

### 6.2 overflow — self-re-exec subprocess, signal-asserted (RSK-003 primary)
A stack overflow is uncatchable in-process (guard page → SIGSEGV → process abort);
and a small-`stack_size` thread does **not** isolate it — Rust's overflow handler
`abort()`s the whole process regardless of which thread overflowed. So the crash must
happen in a **child process**. The robust mechanism is **self-re-exec** (no example-
path resolution — `CARGO_BIN_EXE_` is not set for `examples/` targets): the test
re-runs its own test binary with an env flag and an exact-name filter; the child
branch builds the chain and dies, the parent asserts the child's exit signal.
```rust
#[test] #[ignore = "re-execs itself to crash a child; demonstrates RSK-003"]
fn deep_chain_overflows_inside_target_scale() {
    if std::env::var_os("CORDAGE_OVERFLOW_CHILD").is_some() {
        let _ = deep_chain(80_000);           // CHILD: overflow → process abort (rc 134)
        return;
    }
    let exe = std::env::current_exe().expect("test bin path");
    let status = std::process::Command::new(exe)
        .args(["--exact", "deep_chain_overflows_inside_target_scale", "--ignored"])
        .env("CORDAGE_OVERFLOW_CHILD", "1")
        .status().expect("spawn child");
    assert!(!status.success());               // signal/rc-134 — the cliff, in-target
}
```
`var_os` not `var` (`mem.pattern.lint.disallowed-methods-env-var`). Generators are
defined in the test file (the child branch calls `deep_chain` directly) — no cross-
target import. The example (§5) keeps its own copy for manual CSV runs; it is not on
this test's critical path.

### 6.3 quadratic — measured, recorded, coarse bound (RSK-003 secondary)
Builds `dense_evict` at two densities, times each, prints the ratio, asserts only a
loose upper bound (e.g. the larger build completes < 120s) to avoid timing flake:
```rust
#[test] #[ignore = "slow; records the eviction-fixpoint quadratic for RSK-003"]
fn eviction_fixpoint_scales_superlinearly() {
    let t1 = time(|| dense_evict(100, 100));
    let t2 = time(|| dense_evict(200, 200));
    eprintln!("eviction ratio {:.1}x for 4x edges", t2.as_secs_f64()/t1.as_secs_f64());
    assert!(t2 < std::time::Duration::from_secs(120));  // sanity, not a tight gate
}
```
The printed ratio (probe saw ~17×) is the evidence; the assert only guards against a
hang masquerading as a pass.

## 7. Findings note (`notes.md`)

Consolidates: the three confirmed measurements, the harness commands that reproduce
each, the in-target verdict (overflow ~80k, eviction quadratic, explain 2^layers),
H1's honest position (recompute is fine *only after* Fix A/B; explain needs Fix C
before any deep-lattice consumer), and the OQ-2 allocation gap. Cites RSK-002/003 and
the fix follow-ups.

## 8. Verification alignment

- New evidence: two harness files + findings note. No existing cordage test changes
  (behaviour-preservation — the 75 green tests stay untouched).
- Default gate unaffected: reds are `#[ignore]`d; `just check` / `cargo test -p
  cordage` stay green. The reds run explicitly via `cargo test -p cordage --ignored`.
- Zero-dep invariant re-asserted: `crates/cordage/Cargo.toml` diff is empty.
- **Clippy coverage gap (accepted):** the gate runs plain `cargo clippy` (lib + bins
  only, never `--all-targets`), so it does **not** lint `examples/` or `tests/`. The
  harness's lint cleanliness is therefore not enforced by `just check` — lint it
  manually (`cargo clippy -p cordage --examples --tests`) before commit. This is also
  why the harness is an `examples/` target, not a `[[bin]]` (a bin *would* be gated,
  but reintroduces the `CARGO_BIN_EXE` coupling §6.2 avoids).

## 9. Open questions

- **OQ-2 — allocation numbers:** wall-clock-first; peak-alloc out of scope v1 (noted
  in findings). Probe's OOM observation already bounds explain qualitatively.
- **Bin-path resolution (§6.2):** confirm `CARGO_BIN_EXE_scale_harness` is exposed
  for an `examples/` target under the jail; fallback is a `target/examples/` lookup.
  Implementation-time detail, not a design fork.

## 10. Decisions log

- **D1 (from scope) — std-only in-crate harness**, criterion/bench-member rejected.
- **D2 — overflow is asserted out-of-process via self-re-exec** (`current_exe` +
  env flag + `--exact` filter, child aborts, parent reads the signal). Forced by R1
  (uncatchable in-process; a small-stack thread does not isolate it either).
  Self-re-exec over example-spawn because `CARGO_BIN_EXE_` is not set for examples.
- **D3 — explain red is a deterministic count assertion** (`2^layers`), not a timing
  test — the only flake-free way to red an exponential.
- **D4 — no shared generator module**; bounded inline duplication beats an
  `examples`↔`tests` import hack at this size.
