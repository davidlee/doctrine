# Design SL-162: Runtime-resolve test binary path

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Integration tests resolve the `doctrine` CLI they spawn through a compile-time
constant — `const BIN: &str = env!("CARGO_BIN_EXE_doctrine")` — in 59 test files.
`env!` freezes one absolute path (`<target>/<profile>/doctrine`) into the
compiled test artifact. When that artifact is executed from a namespace whose
filesystem does not expose that exact path, every spawn fails:

```
spawn doctrine: Os { code: 2, kind: NotFound } (tests/…:NN)
```

Resolve the spawn target at **runtime** instead, so it tracks whichever namespace
and profile actually runs the test — extending the fix CHR-014 already shipped
for `env!("CARGO_MANIFEST_DIR")` to the sibling bin-path macro.

## 2. Current State

- `const BIN: &str = env!("CARGO_BIN_EXE_doctrine")` appears in 59 files
  (`tests/e2e_*.rs` + `tests/relation_cli.rs`), each spawning via
  `Command::new(BIN)`. Only 4 currently declare `mod common;`.
- `src/test_support.rs` holds the CHR-014 runtime resolver `repo_root()`
  (single source: `mod test_support;` at `src/main.rs:77`; `#[path]`-included by
  `tests/common/mod.rs` for the integration units).
- `tests/e2e_no_baked_manifest_dir.rs` guards against `env!("CARGO_MANIFEST_DIR")`
  re-creeping, scanning `src/` + `tests/` for an assembled needle. It does **not**
  cover `CARGO_BIN_EXE`.
- Trigger: jail (`/workspace/…`) and host (`/home/…`) bind-mount the **same**
  tree → one in-tree `target/` → one test artifact, served by cargo's fingerprint
  to whichever namespace runs next (the baked path is not a fingerprint input).
  Rebuilding the bin does not recompile the test, so the stale path persists.

## 3. Forces & Constraints

- **F1 — Profile-agnostic.** Both `target/debug/doctrine` and
  `target/release/doctrine` exist; the resolver must not hardcode a profile.
- **F2 — Target-location-agnostic.** `CARGO_TARGET_DIR` is unset here (in-tree
  `target/`), but CHR-014 documents a shared *external* jail target
  (`/home/david/.cargo/doctrine-target-jail`). The resolver must hold whether
  `target/` is in-tree or external. This rules out `repo_root().join("target/…")`,
  which assumes in-tree.
- **F3 — Zero baked absolute path.** The bug *is* the baked path; the fix must not
  reintroduce one in another form.
- **F4 — Behaviour-preservation gate (AGENTS.md).** Shared machinery change;
  existing e2e goldens are the proof — byte-identical stdout/JSON/error text, all
  suites green, unchanged.
- **F5 — ADR-001 layering.** Helper is test-only (`test_support`); no production
  layering impact, no cycles.
- **F6 — Single source.** Resolver lives once in `src/test_support.rs`, reused by
  lib unit tests and integration units via the existing seam — no parallel copy.

## 4. Guiding Principles

Resolve from the running artifact, never from build-time identity. Extend the
blessed CHR-014 pattern rather than invent a parallel mechanism. Keep the
call-site delta minimal, uniform, and greppable so the 59-file sweep is auditable.

## 5. Proposed Design

### 5.1 System Model

```
 std::env::current_exe()                resolved at RUNTIME, per namespace
   = <target>/<profile>/deps/<testbin>-<hash>
        │ pop  (drop exe name)
        ▼
   <target>/<profile>/deps/
        │ pop  (drop deps/)
        ▼
   <target>/<profile>/
        │ push "doctrine" + EXE_SUFFIX
        ▼
   <target>/<profile>/doctrine          the spawn target — no baked path
```

`current_exe()` returns the path as the *current* namespace sees it and already
encodes both `<target>` (in-tree or external) and `<profile>` (debug/release),
satisfying F1–F3 in one step. The `doctrine` bin is a fixed sibling of the test
exe's `deps/` parent — a cargo layout invariant.

### 5.2 Interfaces & Contracts

New, in `src/test_support.rs` (beside `repo_root`):

```rust
/// The built `doctrine` binary, resolved at RUNTIME from the running test exe.
/// CHR-014 / SL-162: never bake the path via `env!("CARGO_BIN_EXE_doctrine")` —
/// a shared target serves one artifact across namespaces/profiles, so the baked
/// path NotFounds in the namespace that did not compile it.
pub(crate) fn doctrine_bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("resolve current_exe for doctrine_bin");
    p.pop();                                    // drop test-exe name → …/deps/
    p.pop();                                    // drop deps/          → …/<profile>/
    p.push(format!("doctrine{}", std::env::consts::EXE_SUFFIX));
    p
}
```

Re-exported in `tests/common/mod.rs`:

```rust
#![allow(dead_code, unused_imports)]   // shared helpers: not every includer uses every fn (D5 / R4)
pub(crate) use test_support::{doctrine_bin, repo_root};
```

The inner attribute covers the `#[path]`-included `test_support`, so a crate that
uses only `doctrine_bin` does not trip `dead_code` on the unused `repo_root`
helper (see D5 / R4).

Per-file call-site shape (decision D1 — local wrapper, uniform across all 59):

```rust
mod common;                                     // added where absent (55 files)
fn bin() -> std::path::PathBuf { common::doctrine_bin() }
// …
Command::new(bin())                             // was Command::new(BIN)
```

The `const BIN` line is deleted; `Command::new(BIN)` → `Command::new(bin())` is
the only call-site delta.

### 5.3 Data, State & Ownership

No persistent state. `doctrine_bin()` is a pure function of `current_exe()`.
Ownership unchanged: `src/test_support.rs` is the single source; `tests/common`
re-exports; test files consume.

### 5.4 Lifecycle, Operations & Dynamics

Resolution happens per `Command::new(bin())` call, at test runtime. No
build-graph dependency on the bin is declared (we lose the `CARGO_BIN_EXE_*`
"bin is a dep" guarantee); the established workflow already runs via `cargo test`,
which builds workspace bins before tests. A missing bin now surfaces as a runtime
spawn error rather than a build-time link error — acceptable and rare.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** No source line outside the guard's own assembled needle contains
  `env!("CARGO_BIN_EXE` or `env!("CARGO_MANIFEST_DIR")`.
- **A1** `doctrine` sits at `<target>/<profile>/doctrine`, sibling of the test
  exe's `deps/` parent (cargo default layout). Holds for in-tree and external
  `CARGO_TARGET_DIR`; would break only under a non-cargo bespoke layout (none
  here).
- **EC-1** `current_exe()` failure → `expect` panic with a clear message; a test
  that cannot locate its own exe cannot meaningfully run.
- **EC-2** Windows: `EXE_SUFFIX` yields `doctrine.exe`. Project runs Linux-only
  today; included for correctness at no cost.

## 6. Open Questions & Unknowns

None blocking. (Resolver strategy, call-site shape, and guard rename are decided —
see §7.)

## 7. Decisions, Rationale & Alternatives

- **D1 — `current_exe()` sibling, not `repo_root().join`.** `repo_root()` would
  reintroduce assumptions (in-tree target, known profile) that F1/F2 forbid;
  under the shared external jail target it would resolve a wrong path. Rejected.
- **D2 — Local `bin()` wrapper per file** (vs direct `common::doctrine_bin()` at
  each site). Smallest, uniform call-site delta (`BIN`→`bin()`); preserves the
  one-symbol-per-file shape the suite already uses; trivially greppable.
- **D3 — Generalise + rename the guard** to `tests/e2e_no_baked_paths.rs`, banning
  both `env!("CARGO_MANIFEST_DIR")` and `env!("CARGO_BIN_EXE…")` via two
  fragment-assembled needles. Filename then matches its broader job. The resolver
  uses `current_exe`, so it never self-trips.
- **D4 — Extend CHR-014, don't fork it.** Same footgun class, same seam
  (`test_support` + `tests/common`), same guard test — one coherent pattern.
- **D5 — `#![allow(dead_code)]` on `tests/common/mod.rs`** (the standard shared-
  helper idiom). Introducing a second helper means a crate may use a subset;
  without the allow, the unused helper trips `dead_code` under the zero-warning
  gate. Verified: today all 4 includers use `repo_root`, so the subset case is
  new. Alternatives (per-file selective import; splitting helpers into separate
  modules) add churn for no benefit — the included module still carries both fns.

## 8. Risks & Mitigations

- **R1 — 59-file mechanical sweep, transcription error.** Mitigation: the
  behaviour-preservation gate (F4) — any wrong path or dropped call site fails the
  spawn or the golden, caught by `just gate`.
- **R2 — Lost build-graph link to the bin.** Mitigation: keep running via
  `cargo test`; document in the resolver doc-comment (§5.4).
- **R3 — Layout assumption A1 drifts** if a future bespoke target layout lands.
  Mitigation: INV-1 guard + the assumption recorded at the call site; low
  likelihood (cargo default).
- **R4 — Shared-helper `dead_code` under the zero-warning gate** (the subset-use
  case D5 introduces). Mitigation: `#![allow(dead_code)]` on `common/mod.rs`;
  confirm clippy-clean during execute, and widen to `unused_imports` only if the
  re-export itself is flagged (pub(crate) re-exports usually are not). `just gate`
  is the backstop.

## 9. Quality Engineering & Validation

- **VT-1** Formerly-failing suites (`e2e_adr_cli_golden`,
  `e2e_backlog_filter_alias`) pass after migration. *(by test)*
- **VT-2** Full e2e suite green, goldens byte-identical — `just gate`. *(by test)*
- **VT-3** Generalised guard `e2e_no_baked_paths` passes — scanning `src/` +
  `tests/` for both fragment-assembled needles (`CARGO_MANIFEST_DIR`,
  `CARGO_BIN_EXE`). Reintroduction-protection is by construction: both needles run
  the same proven scan as CHR-014's, so a positive pass demonstrates the property
  (no separate negative fixture — matches prior art). *(by test)*
- **VA-1** `grep -rn 'env!("CARGO_BIN_EXE' tests/ src/` returns nothing outside
  the guard's assembled-fragment needle. *(by agent)*
- **VH-1** Cross-namespace proof: run a previously-failing suite in one namespace,
  then the other, **without recompiling between** — both green. *(by human;
  cannot be exercised from a single namespace in CI)*

## 10. Review Notes

Internal adversarial pass (findings integrated above):

- **F1 — `dead_code` on subset-use of shared helpers.** Verified real: today all 4
  `mod common;` includers use `repo_root`; 55 new ones use only `doctrine_bin`.
  Resolved by D5 / R4 (`#![allow(dead_code)]`), the standard `tests/common`
  idiom. Empirical clippy confirmation deferred to execute.
- **F2 — Guard self-trip / comment-skip.** The `doctrine_bin` doc-comment names
  `env!("CARGO_BIN_EXE_doctrine")` in prose; the guard scans `src/` too but skips
  `//`/`///`/`//!` lines, so the mention must stay in a **doc-comment, never
  code**. Recorded at INV-1; the resolver body uses `current_exe`, no macro.
- **F3 — VT-3 over-claim** ("fails if reintroduced" implied a negative test).
  Softened to by-construction, matching CHR-014's guard (no negative fixture).
- **F4 — VH-1 unverifiable in a single namespace.** Accepted and marked VH: the
  cross-namespace run is the only direct proof; in-jail, correctness is by
  construction (no baked path). Not a blocker.
- **Layering / governance:** ADR-001 satisfied (test-only helper, no cycle). No
  POL/STD conflict. No `/consult` trigger — no unresolved tradeoff.
