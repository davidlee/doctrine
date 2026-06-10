# cordage scale harness — durable red tests + findings for the confirmed scale cliffs

## Context

SPEC-001 **H1** was revised (2026-06-11, `54bd3f4`) from a "small corpus
(tens–hundreds)" premise to a real target of **~tens of thousands of nodes**. The
recompute-per-query claim survives only if cordage's build is genuinely **O(V+E)
and non-recursive**.

That guarantee has now been **empirically refuted** by a post-close probe (2026-06-11,
codex + Opus agree; the probe harness was run then **deleted** — it left numbers in
RSK-002 / RSK-003 but no committed, reproducible artifact). Three confirmed cliffs,
all reachable **inside** the target scale:

- **RSK-003 overflow (impact high — a crash).** `Tarjan::strongconnect`
  (`resolve.rs:321`) and `level_of` (`resolve.rs:545`) recurse with depth = graph
  depth. Both **SIGABRT (rc 134) at chain depth ~80k** — inside tens-of-thousands,
  not beyond. The two overflows are independent; `level_of` overflows on a clean
  acyclic chain with no cycle present.
- **RSK-003 quadratic (distinct, same risk).** The eviction-to-fixpoint passes
  (`pass2_evict` `resolve.rs:198`, `evict_layer_cycles` `resolve.rs:478`) recompute a
  full SCC pass per evicted edge — O(E·(V+E)); `participates` (`resolve.rs:224`)
  rescans all components per candidate, compounding it. Measured ~quadratic: 3.5s at
  100 nodes/9.9k edges → 59s at 200 nodes/39.8k edges.
- **RSK-002 explain exponential.** `explain()` enumerates all predecessor paths;
  16.7M chains at 24 diamond layers in 1.1s, **2^layers** growth, OOM/hang beyond.
  `extend_chains` (`query.rs:150/158`) clones the suffix per branch — O(path_len)
  copy on top of the exponential count.

The discovery question ("do these break in-target?") is **answered: yes.** What the
deleted probe did not leave is a **durable, committed regression gate** and a
consolidated findings record. That is this slice.

Governing canon: SPEC-001 §Hypotheses (H1), §Performance posture; ADR-001 (cordage
is a LEAF — the harness is a separate consumer, never inverts the dependency); the
cordage **zero-dependency contract** (`crates/cordage/Cargo.toml` has no
`[dependencies]`) and the **pure/imperative split** (`std::time::Instant` is impure —
stays out of the pure crate).

## Scope & Objectives

Land the **durable evidence the probe didn't leave** — a committed, reproducible red
harness and a findings note. Measure-and-red only; **no fixes**.

- **Graph generators** (deterministic, public-API only): a deep-chain (linear spine
  of N nodes), a diamond/lattice (parametrised by layers), and a dense-cycle Evict
  overlay (drives the eviction fixpoint).
- **Measurement example** — `crates/cordage/examples/scale_harness.rs`, arg-driven
  (`--cliff overflow|quadratic|explain --n N [--layers L]`), std-only
  (`std::time::Instant`). One run = one measurement; on the overflow path it
  deliberately SIGABRTs. Doubles as the **subprocess target** for the overflow test.
- **Red tests** — `crates/cordage/tests/scale_cliffs.rs`, `#[ignore]`d (long /
  deliberately-crashing, off the default gate):
  - **explain** — deterministic, exact: `explain(sink).paths()[ov].len() == 2^layers`.
    A clean non-flaky red for RSK-002.
  - **overflow** — spawns the example as a **subprocess** at a target-scale depth and
    asserts the child terminates by signal (rc 134). Demonstrates the crash without
    aborting the test process (a stack overflow is uncatchable in-process — see R1).
  - **quadratic** — measures eviction build-time across two edge densities and
    **records** the ratio (printed); coarse sanity bound only, not a flake-prone hard
    timing assertion.
- **Findings note** (`notes.md`) — consolidates the confirmed numbers with the
  committed harness as their reproducer, and states plainly what H1 can honestly
  assert and which fixes the evidence justifies.

## Non-Goals

- **No fixes.** Iterative rewrite, eviction redesign, `explain()` redesign are the
  follow-up slices below. This slice reds; it does not green.
- **No new cordage dependency.** Zero-dep contract binding even for the harness; no
  `[dependencies]` added to cordage; no `criterion`; no bench member (decided — see
  Decisions). std-only.
- **No SPEC-001 / H1 re-revision here.** H1 already states the target. Any trailing
  wording reconcile post-fix is a separate edit.
- **No adapter / policy / CLI work** — cordage core only.

## Decisions

- **D1 — harness placement: std-only, in-crate.** Resolved (was OQ-1). Measurement
  example under `crates/cordage/examples/` + `#[ignore]` red tests under
  `crates/cordage/tests/`; `std::time::Instant` only. *Rejected:* a separate
  `crates/cordage-bench/` member dev-depping `criterion` — it pays new-member lint
  tax + a jail offline-fetch risk for stats on the *easy* half, while the headline
  cliff (a crash) is uncatchable by criterion and needs a hand-rolled subprocess
  harness regardless. A bench member is the right home for *sustained, regression-
  tracked* benching later — a follow-up, not this throwaway harness.

## Affected Surface

- **New:** `crates/cordage/examples/scale_harness.rs`,
  `crates/cordage/tests/scale_cliffs.rs`, `.doctrine/slice/038/notes.md` (findings).
- **Read-only under measurement:** `crates/cordage/src/resolve.rs` (Tarjan, level,
  eviction), `crates/cordage/src/query.rs` (`explain` / `extend_chains`),
  `crates/cordage/src/lib.rs` (`GraphBuilder`, build pipeline).
- **Untouched:** `crates/cordage/Cargo.toml` (zero-dep), workspace `Cargo.toml`.

## Risks, Assumptions, Open Questions

- **R1 (resolved into the design) — a stack overflow is uncatchable in-process.** It
  hits a guard page → SIGSEGV → the runtime aborts the *process*; `catch_unwind` does
  not see it, and a small-`stack_size` thread does not isolate it. The overflow red is
  therefore a **self-re-exec subprocess** assertion (the test re-runs `current_exe`
  with an env flag + `--exact` filter; the child aborts, the parent reads the signal),
  never an in-process `#[should_panic]`. (`CARGO_BIN_EXE_` is not set for examples, so
  self-re-exec, not example-spawn — design.md §6.2.)
- **A1 — generators are envelope, not workload.** Deep-chain / diamond are the worst
  cases for their respective cliffs; real graphs sit between. The harness bounds the
  envelope, by intent.
- **A2 — `Instant` impurity contained** by construction: the harness is a separate
  consumer (example/test), never cordage `src/`.
- **OQ-2 — allocation numbers.** Wall-clock-first; peak-allocation (allocator shim /
  external `time -v`) is out of scope for v1, noted as a gap in findings. The probe's
  OOM observations already bound the explain blow-up qualitatively.

## Verification / Closure Intent

Done when:

- `examples/scale_harness.rs` + `tests/scale_cliffs.rs` exist, std-only, committed.
- `cargo test -p cordage --ignored` runs the reds: the explain test asserts exact
  2^layers; the overflow test asserts a subprocess rc-134 at target depth; the
  quadratic test prints its ratio.
- `notes.md` consolidates the confirmed numbers, citing the harness as reproducer,
  and states H1's honest position + the justified fixes.
- cordage zero-dep intact (`Cargo.toml` unchanged); `just check` green (the harness
  is `#[ignore]`d / an example, off the default gate).
- The three fixes are filed as follow-up slices (below), not patched here.

## Follow-Ups (fixes — now justified measured work, each TDD-greened against this slice's reds)

- **Fix A (mechanical):** iterative `strongconnect` + `level_of` (explicit stack) —
  reds → greens the overflow (RSK-003 primary).
- **Fix B (algorithmic):** one-pass / incremental-SCC eviction; drop the per-edge
  Tarjan restart and the `participates` rescan — greens the quadratic (RSK-003
  secondary).
- **Fix C (redesign):** `explain()` returns a predecessor sub-DAG (or direct + one
  canonical chain), policy enumerates on demand — greens the exponential (RSK-002).
  Larger: changes the `Vec<Vec<NodeId>>` return shape, touches F47 semantics +
  downstream consumers.
- Trailing SPEC-001 H1 wording reconcile once the fixes land.
