# Review RV-003 — reconciliation of SL-042

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-042 (observe substrate, SPEC-002 A) — all four phases
landed on `sl-042-coord` and **merged into main** (no-ff, disjoint from the
settled SL-043). Reconcile the implementation against `design.md`, `plan.toml`
(PHASE-01..04 EN/EX/VT), and governance (SPEC-002 D1–D9, ADR-003/009/001/004).

**Topology note.** RV verbs refuse on a worktree fork (IMP-024 — the turn baton
lives in the parent tree's gitignored state), so the coord branch was merged to
main first; this ledger and `/close` run on main where code + notes + baton sit
together.

**Gate evidence (merged main, `DOCTRINE_WORKER` unset, plain clippy):** `cargo
fmt --check` (0), `cargo clippy` (0, compiled clean), `cargo test` (917 lib +
all e2e green, 1 ignored = the 2000-tier perf probe), `cargo build` (0).

**Lines of attack — the load-bearing invariants:**
- **NF-001 (INV-1).** No `ReqStatus = f(CoverageStatus)`. Held structurally:
  `drift()` returns `Verdict` (coverage.rs:249, test :688); the two SL-028 enums
  never reference each other (requirement.rs:84–88); coverage and authored status
  in distinct stores. Import-edge enforcement is vacuous here (no writer) → Slice B.
- **NF-002.** Stale VH/VA `Verified` flagged, never auto-demoted — P4 lock tests.
- **No parallel impl.** Staleness rides `git::commits_touching`; `contentset.rs`
  is off the coverage path (runtime-composed guard, coverage_scan.rs:538).
- **INV-2.** Composite/drift derived, never persisted.

**Where the bodies are buried — four known designed deferrals (not drift):**
EX-2 dead-code "genuinely used" lands at the Slice-B consumer (self-clearing
suppressions carried on coverage.rs:32 / coverage_scan.rs:23 / git::head_sha:933);
NF-001 verified structurally not as a test-of-absence (design §5.5/§9 F4);
perf-spike EX-5 triggers recorded but unfired (no cliff at 2000, no consumer yet);
the Slice-B dependent (reconcile writer + closure gate + import-edge guard) must
be captured in backlog before close (defer-needs-backlog).

## Synthesis

**Verdict: audit-ready for close.** SL-042 delivers the SPEC-002 observe
substrate as designed across PHASE-01..04. The gate is green on merged main
(fmt/clippy/test 917+e2e/build all 0, the sole `#[ignore]` is the 2000-tier perf
probe). Every load-bearing invariant holds with evidence, and the four open items
are **conscious, designed deferrals to the Slice-B consumer — not drift.** No
blocker findings; the close-gate has nothing to refuse.

**What is genuinely proven now.** The two-tier separation is structural, not
aspirational: `drift()` returns a `Verdict` that cannot carry a status write; the
SL-028 enums keep their never-reference property; coverage and authored status
live in distinct files. The folds are pure and deterministic with staleness
resolved in the shell (F1); `composite`/`drift` persist nothing (INV-2). The §5.2
coherence predicate is total by construction — the codex X-1 totality hole
(`Pending`/`InProgress` + fresh `Verified` → `Divergent`, the *accept* case) and
the X-2 4-tuple key collision are both closed in the landed verdict matrix
(VT-2). NF-002 is locked: a stale VH **and** VA `Verified` is flagged
`IsStale::Stale` while its `CoverageStatus` stays `Verified` — surfaced, never
auto-demoted. H1 graduated from hypothesis to fact (EX-1): `git::commits_touching`
consumes coverage's `(git_anchor, touched_paths)` granularity verbatim, no leaf
widening, no fork — the no-parallel-impl line held, with a runtime-composed
`contentset::` guard proving the rival leaf is off the coverage path.

**Standing risks / consciously accepted tradeoffs.**
- **EX-2 dead-code (F-1).** The whole coverage leaf + scan shell are dead in the
  clippy(bins/lib) build — no consumer until the Slice-B reader. The plan/design
  premise that P2 makes `CoverageStatus` "genuinely used" was optimistic; the
  resolution (self-clearing `not(test) expect(dead_code)` on the leaf, shell, and
  `git::head_sha`) is correct and retires itself at the consumer. Lands at IMP-030.
- **NF-001 import-edge enforcement (F-2).** Held structurally here; its
  load-bearing import-edge clause is vacuous until a status-writer exists, so the
  real enforcement is owned by Slice B / IMP-030. Not a gap in SL-042's scope.
- **Perf (F-3).** Both cost axes linear, no cliff at N=2000; the conditioned
  reverse-index / staleness-batching triggers are unfired and captured in RSK-006,
  sized against the future reader.
- **OQ-2 / OQ-3** remain deferred per design (knowledge_record sequencing;
  composite precedence — `Indeterminate` reads as drift at the gate for v1).

**Close-out posture.** All deferred-but-needed work is captured before close
(defer-needs-backlog): **IMP-030** (the Slice-B reconcile/close half — reconcile
writer, closure gate, NF-001 import-edge enforcement; carries the F-1 dead-code
and F-3 perf deferrals that resolve at the consumer) and **RSK-006** (the
conditioned perf triggers). The slice's hand-edited status (`proposed`) diverges
from the 4/4 phase rollup (the ⚠ in `slice list`) — reconcile to a terminal
status at `/close`.
