# cordage scale spike — characterise build/query cost and red the recursion + path-enumeration cliffs

## Context

SPEC-001 **H1** was revised (2026-06-11, commit `54bd3f4`) from a "small corpus
(tens–hundreds)" premise to a real target of **~tens of thousands of nodes**. The
recompute-per-query claim survives only if cordage's build is genuinely **O(V+E)
and non-recursive**. That guarantee is now load-bearing and **unverified**.

Two cliffs are already identified by static review of `crates/cordage/src` (the
SL-036 post-close perf review):

- **RSK-003 — recursion-depth stack overflow (impact high, a crash).**
  `Tarjan::strongconnect` (`resolve.rs:321`) and `level_of` (`resolve.rs:545`)
  recurse with depth = graph depth. A deep chain at multi-thousand scale overflows
  the 8 MB main-thread stack → panic. Secondary: the eviction-to-fixpoint pass
  (`pass2_evict` `resolve.rs:198`, `evict_layer_cycles` `resolve.rs:478`) rebuilds a
  full Tarjan per evicted edge — O((1+K)·(V+E)), quadratic on heavily-tangled input.
- **RSK-002 — `explain()` path enumeration exponential** on diamond/lattice
  topologies (`query.rs`). Independent of build; topology-driven, not node-count.

PHASE-02 of SL-036 consciously deferred the recursion concern as a non-concern at
H1/H2 (`state/slice/036/phases/phase-02.md:47`); that deferral is void under the
revised H1. This spike is the **gate** that converts the static suspicion into
measured fact before any fix is scoped.

Governing canon: SPEC-001 §Hypotheses (H1, revised), §Performance posture; ADR-001
(cordage is a LEAF — the harness must not invert the dependency); the cordage
**zero-dependency contract** (`crates/cordage/Cargo.toml` has no `[dependencies]`)
and the **pure/imperative split** (`std::time::Instant` is impure — it may not
enter the pure crate).

## Scope & Objectives

Deliver **measurement and evidence**, not fixes:

- **Graph generators** — a deep-chain (linear spine of N nodes) generator to drive
  recursion depth, and a diamond/lattice generator (parametrised by depth/width) to
  drive `explain()` path count. Reusable fixtures, deterministic.
- **Measurement harness** — wall-clock (`std::time::Instant`) and, where cheap,
  allocation/peak observation across build (`resolve`) and query (`reachable`,
  `spine_path`, `evaluate`, `explain`) as N and depth grow.
- **Red the cliffs** — find and record the depth at which `strongconnect` /
  `level_of` overflow the stack; the eviction-fixpoint cost vs cycle density vs N;
  the `explain()` blow-up curve vs diamond depth. Confirm or refute the O(V+E)
  build claim for the acyclic / near-acyclic common case at ~tens of thousands.
- **Findings note** — numbers + a short written characterisation: where it falls
  over, where it holds, what H1 can honestly assert post-measurement.

**Harness placement is an open architecture call (see Open Questions) — `/consult`
it before building, do not default it.**

## Non-Goals

- **No fixes.** The recursion → iterative rewrite, an `explain()` sub-DAG/lazy
  redesign, or an incremental-SCC eviction loop are **follow-up slices**, scoped
  from this spike's evidence. This slice measures and reds; it does not green.
- **No new cordage runtime dependency.** The zero-dep contract is binding even for
  benches; the harness may not add `[dependencies]` to the cordage crate.
- **No `criterion`-in-cordage.** If a bench framework is chosen it lives in a
  separate non-published member, never in cordage proper.
- **No SPEC-001 / H1 re-revision here.** H1 already states the target; this slice
  supplies the evidence behind it. Any wording reconcile post-measurement is a
  trailing edit, not the deliverable.
- **No adapter/policy/CLI work** — cordage core only.

## Affected Surface

- **New:** generators + harness — placement TBD (`crates/cordage/examples/`,
  `#[ignore]` tests under `crates/cordage/tests/`, or a new
  `crates/cordage-bench/` member). The placement decision picks the path.
- **Read-only under measurement:** `crates/cordage/src/resolve.rs` (Tarjan, level,
  eviction), `crates/cordage/src/query.rs` (`explain` / `predecessor_paths`),
  `crates/cordage/src/lib.rs` (build pipeline / `GraphBuilder`).
- **Possibly touched:** workspace `Cargo.toml` `members` (only if a bench member is
  chosen).

## Risks, Assumptions, Open Questions

- **OQ-1 (blocking, `/consult` before build) — harness placement.** Two shapes,
  each honours zero-dep + pure split differently:
  (a) **std-only**, `#[ignore]`d tests or `examples/` using `std::time::Instant` —
  no new dep, harness is a separate consumer of cordage, crude numbers; or
  (b) **separate non-published bench member** `crates/cordage-bench/` that
  dev-deps `criterion` and depends on cordage by path — richer stats, more
  scaffolding, keeps cordage itself zero-dep. Decide via `/consult`.
- **R1 — measuring the wrong thing.** A stack overflow aborts the process; naive
  timing loops won't capture the threshold cleanly. The harness must bracket the
  overflow depth (e.g. spawn a bounded-stack thread, or step depth and catch the
  abort) rather than just time successful runs.
- **A1 — generators are representative.** Deep-chain and diamond are the worst
  cases for the two cliffs respectively; real doctrine graphs sit between. The
  spike characterises the envelope, not a production workload.
- **A2 — `Instant` impurity is contained.** The harness is a separate consumer, so
  `Instant` never enters cordage source; this holds by construction once placement
  is decided.
- **OQ-2 — allocation measurement.** Whether to attempt peak-allocation numbers
  (needs a global allocator shim or external `time -v`) or stay wall-clock-only for
  v1 of the spike. Lean wall-clock-first; note the gap.

## Verification / Closure Intent

Done when:

- The generators + harness exist and run, at the placement chosen via `/consult`.
- Measured numbers exist for: recursion-overflow depth (RSK-003 primary),
  eviction-fixpoint cost vs cycle density (RSK-003 secondary), `explain()` blow-up
  vs diamond depth (RSK-002), and acyclic build cost vs N at ~tens of thousands.
- A findings note records the curves and states, with evidence, whether H1's
  linear+iterative guarantee holds as written or must change.
- Each surfaced fix is a filed follow-up (`backlog new` or grown RSK), not silently
  patched here.
- cordage's zero-dep contract is intact (`Cargo.toml` unchanged or only a bench
  member added); `just check` green.

## Follow-Ups

- Recursion → iterative rewrite of `strongconnect` + `level_of` (reds RSK-003).
- `explain()` sub-DAG / lazy-enumeration redesign (reds RSK-002).
- Possible incremental-SCC eviction loop if the fixpoint quadratic bites.
- Trailing SPEC-001 H1 wording reconcile once numbers are in.
