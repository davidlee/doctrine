# SL-038 Design — cordage scale harness

Status: **locked** (2026-06-11, post-inquisition — RSK-004 folded in (D5); C1/C2
mended, C3 carried to plan as a VT gate). Canonical for design intent; scope in
`slice-038.md`; hostile pass in `inquisition.md`.

## 1. Purpose

Land a committed, reproducible regression gate + findings note for the confirmed
scale cliffs in cordage. Measure-and-red only. The probe that confirmed three of the
numbers (2026-06-11) was deleted; this slice makes that evidence durable and turns it
into reds the eventual fixes green.

Four cliffs in scope — three **probe-confirmed** (RSK-002 explain, RSK-003 overflow,
RSK-003 quadratic) and one **analytical-only, first-measured-here** (RSK-004
evaluate). RSK-004 was folded in (D5): it is the same class and the same red shape as
the eviction quadratic (§6.4 mirrors §6.3), so a 4th measured red here is far cheaper
than a later separate harness slice. The findings note (§7) flags RSK-004's distinct
provenance precisely — the deleted probe never measured it.

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
example owns the canonical copies; the tests build their own in-file copies of the
generators they need (`explain` → `diamond`; `overflow` + `evaluate` → `deep_chain`,
one generator shared across the two reds; `quadratic` → `dense_evict`). That is **3
generators** duplicated across the `examples/`↔`tests/` boundary — at the revisit
threshold, not over it. Minor, bounded duplication is preferred over an
`#[path]`/module hack to share code between an `examples/` bin and a `tests/` bin
(they cannot import each other). If a 4th generator appears, revisit.

## 4. Generators (public-API only)

Signatures (in `examples/scale_harness.rs`; the diamond reappears inline in the test):

```rust
// Linear spine 0→1→…→(n-1) on one overlay referenced by the order spec. Drives
// Tarjan + level_of recursion depth = n (the overflow cliff) AND, re-used at a
// sub-overflow n, the evaluate cliff: reachable(k) walks the n-k-node suffix, so
// the per-node BFS sum is Σ(n-k) = O(n²). Returns the built Graph plus the spine
// overlay id (the overflow path ignores the id; evaluate needs it for the
// ChannelSpec). At large n the build attempt is where the SIGABRT happens, by intent.
fn deep_chain(n: u32) -> (cordage::Graph, OverlayId);

// `layers` diamond stages: each stage splits to 2 then rejoins. Predecessor-path
// count from source to sink = 2^layers (exact, deterministic). Acyclic.
fn diamond(layers: u32) -> (cordage::Graph, /*source*/ NodeId, /*sink*/ NodeId, OverlayId);

// One Evict overlay carrying a dense cycle over `nodes` vertices (`edges` ≈ a near-
// complete digraph), forcing the eviction-to-fixpoint pass to recompute SCCs per
// evicted edge. Drives the RSK-003 quadratic.
fn dense_evict(nodes: u32, edges_per_node: u32) -> cordage::Graph;
```

The evaluate cliff (RSK-004) reuses the **deep-chain spine** rather than a dedicated
generator: it is the right demonstrator (sparse, E=O(V), reachable depth O(V) — so
the current O(V·(V+E)) query stands apart from the O(V+E) topo-fold fix), and reusing
it honours D4. The two cliffs that share the spine differ only in `n`: the overflow
red builds at target depth (~80k → abort); the evaluate red builds at a **sub-overflow
n** (the build must *succeed* so the query cost is isolated — tune empirically, ~5–12k,
well under the ~80k overflow threshold).

## 5. Measurement example — CLI contract

```
scale_harness --cliff overflow  --n N           # build deep_chain(N); may SIGABRT
scale_harness --cliff explain   --layers L      # build diamond(L); print path count + time
scale_harness --cliff quadratic --n N           # build dense_evict(N,..); print build time
scale_harness --cliff evaluate  --n N           # build deep_chain(N) sub-overflow; time evaluate() over the spine
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
        let _ = deep_chain(80_000);           // CHILD: build aborts (rc 134); tuple unused
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
    let t1 = time(|| dense_evict(50, 50));     // PHASE-01 debug-pinned: 2.2s
    let t2 = time(|| dense_evict(100, 100));   // 41s — ratio 18.5× (super-quadratic)
    eprintln!("eviction ratio {:.1}x for 4x edges", t2.as_secs_f64()/t1.as_secs_f64());
    assert!(t2 < std::time::Duration::from_secs(120));  // sanity, not a tight gate
}
```
The printed ratio (probe saw ~17×; PHASE-01 debug measured 18.5×) is the evidence; the
assert only guards against a hang masquerading as a pass. **N pair (50,100), not the
release-probe-shaped (100,200):** `cargo test` runs DEBUG (~10× the probe's release
numbers), where n=200 ≈ ~700s would blow the 120s bound and run ~12 min
(`mem.pattern.testing.debug-vs-release-scale-timing`); 50/100 keeps the cliff (41s ≪
120s) and the same super-quadratic ratio.

### 6.4 evaluate — measured, recorded, coarse bound (RSK-004, first-measured-here)
Same shape as §6.3 — a measured-ratio red, not an exact-count or subprocess one.
`evaluate()` runs one `reachable()` BFS per node (`query.rs:256`, the `for ord in
0..node_count` loop); over the deep-chain spine each BFS walks the suffix, so the
whole call is O(V²). Builds the spine at two **sub-overflow** sizes, evaluates an
idempotent channel (`Combinator::Any` over `Direction::Along` — descendants along the
spine) seeded once at the spine head (the `NodeId` the builder returns — opaque ids
have no public ctor), times each, prints the ratio, asserts only a loose upper bound:
```rust
#[test] #[ignore = "slow; records the evaluate() per-node-BFS quadratic for RSK-004"]
fn evaluate_scales_quadratically_in_node_count() {
    let (g1, ov1, h1) = deep_chain(2_000);   // sub-overflow: build MUST succeed
    let (g2, ov2, h2) = deep_chain(4_000);   // so query-time cost is isolated
    // Any's seed domain is ValueKind::Flag (query.rs:431) → Flag(true) is in-domain.
    // Opaque ids: seed the head NodeId the builder returned, never `NodeId(0)`.
    let s1 = BTreeMap::from([(h1, ChannelValue::Flag(true))]);
    let s2 = BTreeMap::from([(h2, ChannelValue::Flag(true))]);
    let t1 = time(|| g1.evaluate(ChannelSpec::new(ov1, Combinator::Any, Direction::Along), &s1));
    let t2 = time(|| g2.evaluate(ChannelSpec::new(ov2, Combinator::Any, Direction::Along), &s2));
    eprintln!("evaluate ratio {:.1}x for 2x nodes", t2.as_secs_f64()/t1.as_secs_f64());
    assert!(t2 < std::time::Duration::from_secs(120));   // sanity, not a tight gate
}
```
2× nodes → ~4× time is the quadratic signal (vs ~2× for the O(V+E) topo-fold fix).
`Direction::Along` traverses the spine toward descendants so `reachable(k)` returns the
`n-1-k`-node suffix; `Direction::None` would yield `∅` and **destroy the signal** — it
is forbidden here. The output `Channel` is near-empty (the lone seed reaches only the
spine head's own fold set) — irrelevant: the cost is the **unconditional** per-node BFS, which
runs before any fold regardless of seeds. Build is excluded from the timed closure (graphs are built *before*
`time(||…)`), so the measurement is the query, not the O(V) acyclic build. Unlike §6.3
this red builds **no dense graph** — the cost is purely the per-node re-BFS, which is
what isolates RSK-004 from the RSK-003 eviction quadratic.

**N-pair tuning (impl):** both `n` must be (a) safely **sub-overflow** so the spine
build succeeds — the recursion margin is large (overflow ~80k; the PHASE-01-pinned
2–4k is ~1/20 of that), validated empirically not assumed; and (b) large enough that the
larger `evaluate` runs long enough (target ≳ a few hundred ms) for the recorded ratio
to clear scheduler noise — if both calls are sub-ms the 4× signal blurs. PHASE-01 debug
measured (2000,4000) → 1.8s/7.7s, ratio 4.25×, ~9.5s total. The *recorded
ratio* is the evidence; the `< 120s` assert only guards a hang.

## 7. Findings note (`notes.md`)

Consolidates: the four measurements, the harness commands that reproduce each, the
in-target verdict (overflow ~80k, eviction quadratic, explain 2^layers, evaluate
O(V²) per-node BFS), H1's honest position (recompute is fine *only after* Fix A/B/D;
explain needs Fix C before any deep-lattice consumer), and the OQ-2 allocation gap.
Cites RSK-002/003/004 and the fix follow-ups. **Provenance is stated precisely:** the
RSK-002/003 numbers are the deleted probe's, *re-confirmed* by this harness; the
RSK-004 number is **first measured here** — the probe never ran it (it was filed
analytically, after the probe, from the `query.rs:256` read), so the harness is its
sole empirical source, not a reproducer of a prior run.

## 8. Verification alignment

- New evidence: two harness files + findings note. No existing cordage test changes
  (behaviour-preservation — the 75 green tests stay untouched).
- Default gate unaffected: reds are `#[ignore]`d; `just check` / `cargo test -p
  cordage` stay green. The reds run explicitly via `cargo test -p cordage --ignored`.
- Zero-dep invariant re-asserted: `crates/cordage/Cargo.toml` diff is empty.
- **Clippy coverage gap (accepted):** the gate runs plain `cargo clippy` (lib + bins
  only, never `--all-targets`), so it does **not** lint `examples/` or `tests/`. This
  is also why the harness is an `examples/` target, not a `[[bin]]` (a bin *would* be
  gated, but reintroduces the `CARGO_BIN_EXE` coupling §6.2 avoids).
- **Manual lint gate is `--examples` ONLY** (corrected 2026-06-11 at phase-plan, was
  `--examples --tests`). Empirically established: `cargo clippy -p cordage --tests`
  fails across the **entire existing suite** (`expect_used`, `many_single_char_names`,
  `tests_outside_test_module`, doc lints) because `[lints] workspace = true` makes the
  cordage test targets inherit the full deny set, and `clippy.toml` sets no
  `allow-{unwrap,expect}-in-tests`. Integration tests are not clippy-clean by repo
  convention — that is the documented reason `just check` avoids `--all-targets`
  (CLAUDE.md; `mem.pattern.lint.clippy-denies`). So the **example** must be written
  clippy-clean and is the manual gate (`cargo clippy -p cordage --examples`); the
  **reds** (`tests/scale_cliffs.rs`) follow the existing-test convention — `expect`/
  `unwrap` and short names are fine, and the file is not clippy-gated. Concretely the
  example avoids the deny set: `main() -> Result<…>` returning `Err` for bad args
  (`std::process::exit` is **disallowed** by `clippy.toml`), `writeln!(io::stdout()…)?`
  not `println!` (`print_stdout`/`print_stderr` denied), `var_os` not `var`, `.get()`
  not index (`indexing_slicing`), `try_from` not `as` (`as_conversions`/`cast_*`).

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
- **D5 — RSK-004 folded in as the 4th cliff** (was the one open scope decision;
  resolved with the user 2026-06-11). The evaluate per-node-BFS quadratic is the same
  class and same red shape as the eviction quadratic (§6.4 mirrors §6.3), and the
  public surface (`Graph::evaluate`, `ChannelSpec::new`, `ChannelValue` — verified)
  supports a ~30-line black-box measured-ratio red reusing the deep-chain spine at a
  sub-overflow `n`. A 4th red here is far cheaper than a later separate harness slice,
  and the findings note is more complete (all build- and query-time cliffs in one
  place). *Distinct provenance, stated in §7:* RSK-004 is analytical-only —
  **first measured by this harness**, not a reproducer of the deleted probe. *Not
  folded:* the evaluate fix itself (single reverse-topo fold) is Fix D below, a
  follow-up, not this slice.
