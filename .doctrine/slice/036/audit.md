# Audit — SL-036 cordage graph core crate

Conformance audit (post-implementation, tied to the slice). Mode: **conformance**.
Reconciles the implemented 5 phases against `design.md` (locked 2026-06-10, F1–F48),
`plan.toml`, SPEC-001, and the repo governance (ADR-001/004, lint posture). No
spec/contracts engine exists; reconciliation is against `design.md` + SPEC-001.

Auditor run: 2026-06-11. Tree at PHASE-05 tip (`94d2c8b` golden net; `449e07f`
explain). PHASE-05 still `in_progress` in the state sheet — the flip to `completed`
is deliberately left to this audit / `/close` (executor's call, notes.md:395).

## Gate evidence (independently re-run)

| Check | Result |
|---|---|
| `cargo test -p cordage` | **75 passed, 0 failed** across 10 suites (adjacency 5, build_validation 8, channels 9, construction 3, denylist 3, explain 8, golden_net 8, ordering 9, reachability 8, resolution 14) + 0 doctests |
| `cargo clippy -p cordage` (plain; NOT `--all-targets`) | **zero warnings** |
| `just check` (whole workspace fmt+lint+test+build) | **green** (exit 0) |
| Zero-dep contract | **held.** `crates/cordage/Cargo.toml` has NO `[dependencies]` section (only the documented "do not add" comment); `cargo tree -p cordage` shows `cordage` alone, no doctrine path dep |

## Per-criterion evidence

### PHASE-05 EX (plan.toml 139–145)

- **EX-1** — `Explanation { node, order_key, paths: BTreeMap<OverlayId, Vec<Vec<NodeId>>>, evicted }`
  declared flat in `lib.rs` (~315 design shape), accessors only, no `String`/role fields (F13).
  F47 termination in `query::predecessor_paths`/`chains_to_root`/`extend_chains`: chain ends at a
  root OR the first node of a degraded post-arity SCC (`degraded_sccs[overlay]`), SCC members are
  endpoints never walked through; in-SCC node → `[[n]]`. Visited-set is a defensive secondary guard.
  **Evidence:** `tests/explain.rs` (8), src/query.rs:80–185. **aligned.**
- **EX-2** — `explain(n).evicted` = `provenance.evictions()` filtered to `src==n || dst==n` (F26),
  order-preserving over the `(overlay,edge)` provenance sort (lib.rs:854–860).
  **Evidence:** `tests/explain.rs::explain_evicted_filters_to_n_as_src_or_dst` (n-as-dst + n-as-src
  present, unrelated absent, exactly 2). **aligned.**
- **EX-3** — property tests: naive independent SCC oracle (mutual-reachability closure, NOT Tarjan)
  + topo edge-respect witness; edge-insertion permutation (Heap's algorithm, hand-rolled) →
  byte-identical `order_key`/`Channel`/`Provenance`/`Explanation`.
  **Evidence:** `tests/golden_net.rs` (8). **aligned.**
- **EX-4** — denylist scan over `crates/cordage/**` (code, docs, manifest, tests) wired as a
  `#[test]`, hand-rolled walk + whole-word case-insensitive matcher, root via `CARGO_MANIFEST_DIR`,
  self-guarded twice (skip-by-name + fragment-assembled literals), live-matcher guard, walk-found-
  src/lib.rs guard. **Evidence:** `tests/denylist.rs` (3). **aligned** (curation dispositioned below).
- **EX-5** — clippy zero, `just check` green, full suite green. **aligned** (re-run above).

### PHASE-05 VT (plan.toml 146–152)

- **VT-1** (explain on cycles, §9 F47): a↔b + a→x → `explain(x).paths[ov]=[[a,x]]`,
  `explain(a)=[[a]]`, terminates, cycle in `Provenance.cycles` not in paths — all asserted. **aligned.**
- **VT-2** (F26 endpoint filter): src + dst evictions present, unrelated absent. **aligned.**
- **VT-3** (determinism+, §9 F24): permutation byte-identity + naive-oracle SCC/topo. **aligned.**
- **VT-4** (REQ-077 full): build-twice → identical order_key + Provenance + contributor traces
  (Max-channel contributor trace pinned explicitly). **aligned.**
- **VT-5** (REQ-079 boundary final): no doctrine dep, structural-id-only suite, denylist green over
  code/docs/tests. **aligned.**

### Prior-phase criteria (spot-reconciled, suites green)

PHASE-01..04 EX/VT are evidenced by their suites (construction/build_validation/adjacency,
resolution, ordering, reachability/channels), all green and unchanged under the PHASE-05 additions
(behaviour-preservation gate holds — the explain field-add and the golden net are additive). The
F30/F46 two-SCC split (authored→diagnostic, post-arity→degradation) is pinned by
`resolution.rs::arity_breaks_authored_reject_cycle_diagnostic_still_emitted`. No drift found.

## Disposition of the two executor-flagged findings

### Finding 1 — REQ-076 cyclic-view: clean downstream child of a degraded cycle is Degraded, not Finite

**Expected (executor's first intuition):** on Reject `{0,1}` cycle + surviving `0→2`, node 2 stays
`Finite`. **Observed:** node 2 is `Degraded` (full-downstream taint), so the whole region degrades.

**Conformance check.** This is **conformant, not a regression.** The design mandates exactly this:
taint seeds = degraded post-arity SCC members of *spec-referenced* overlays (F31), propagated to
**every U-descendant** (design §5.4 pass 4, ~296–298; edge case "degraded SCC with an outbound edge
(`a↔b`,`b→c`) → `c` Degraded", F32). The test fixture wires an `OrderSpec` over the overlay
(`build_in_order` always does, golden_net.rs:79), so the overlay IS spec-referenced and taint
correctly applies.

Critically, taint sets only the `Level` **variant tag**, not the longest-path **depth**: source
confirms `materialize_keys` reads `longest_levels(U)` for the depth, then overlays
`Degraded(depth)` vs `Finite(depth)` (resolve.rs:515–524). The depth is identical regardless of
taint → the surviving acyclic edge `0→2` stays order-respected (`0` precedes `2` in `ordered()`),
which is the true REQ-076 "no false topo" witness. The test asserts both: all three Degraded AND
`pos(0) < pos(2)` (golden_net.rs:438–481). This is the documented full-downstream conservatism
(notes.md round-4 known-open; design §10 Lock), deliberately accepted for v1.

**Disposition: aligned.** The executor's first test cut (expecting node 2 Finite) was a *test* bug,
correctly fixed; the production behaviour matches design. Durable risk (conservatism extent)
harvested below.

### Finding 2 — denylist curation (A3): `product` excluded from forbidden scope

**Decision under review:** the executor curated the literal denylist from SPEC-001 D2 / Appendix B —
included Appendix B domain nouns (task/project/habit/backlog) + time/scheduling/commitment/urgency
terms (deadline/schedule/calendar/lateness/urgency/urgent/commitment/capacity/resurface), forced
`lib.rs` `backlog`→`domain` rewording, but **excluded bare `product`**, arguing "product-neutral"
is the crate's self-description not domain semantics.

**Conformance check.** SPEC-001 D2/Appendix B (verified at spec-001.md:131–134 and via
`doctrine spec show SPEC-001`) reads: *"forbidden core: task/project/habit terms, deadline /
scheduled-for / best-before, lateness cost, remaining-work, commitment pressure, urgency scoring,
calendar/capacity, sequential/parallel policy, resurfacing, **product defaults**."* The prohibition
is on product *vocabulary/defaults* — semantic interpretation bleeding in — **not** the literal
ASCII word "product." Every occurrence of "product" in the crate is the phrase **"product-neutral"**
(README.md:6, lib.rs:4,6, denylist.rs disclaimers) — a *disclaimer of* the boundary, the exact
opposite of a violation. A whole-word `product` denylist entry would flag the crate's own boundary
self-description: a guaranteed false positive that would force removing the word that *states* the
contract.

**Disposition: aligned.** Excluding bare `product` is correct. The concrete domain nouns carry the
actual prohibition and are all in the list. The call is defensible, documented in the test rustdoc
(denylist.rs:40–43), and required no `/consult`. **No change.** (Minor residual risk harvested:
the denylist matches base forms only, no stemming — see R-A below.)

## New findings (auditor)

- **N-1 (informational, no action).** The denylist matcher is base-form / whole-word only (no
  stemming): `schedule` hits but `rescheduled`/`scheduling` does not (denylist.rs:163–166 documents
  this). For the *current* clean tree this is moot (zero forbidden tokens of any inflection), and a
  future leak would most plausibly arrive as a base form. Acceptable for v1; widening to stem is a
  cheap follow-up only if a real inflected leak ever slips. **Disposition: tolerated drift**
  (conscious, documented in-test, zero current exposure).
- **N-2 (informational).** `explain`'s `paths: Vec<Vec<NodeId>>` enumerates every chain to root —
  exponential on a diamond lattice (notes.md design-stage known-open; F47 bounds *termination*, not
  combinatorics). NOT exercised by any VT (fixtures are ≤3 nodes by design). This is a pre-existing,
  design-acknowledged, first-consumer-owned risk, NOT introduced by PHASE-05. **Disposition:
  follow-up (deferred to first consumer)** — already on the design's Lock known-open list; harvested
  below. No backlog item filed (the design Lock already owns it; filing would duplicate).

No conformance *defects* found. No production-code change made or warranted.

## Durable risks harvested (from phase sheets + notes → here)

- **R-A — full-downstream taint conservatism.** A single degraded cycle near the root degrades the
  entire downstream region in `U` (all `Degraded`). Sound under REQ-076 (degrade-not-falsify) and
  edge-respecting (depth preserved), but blunt. Gentler still-not-false alternative: condensation
  ordering (SCC members tie at equal level). Core-internal, no interface impact — revisit on first
  consumer complaint. (notes.md round-3/4; design §10.)
- **R-B — explain path-enumeration blowup** (N-2). `Vec<Vec<NodeId>>` is exponential in depth on a
  diamond lattice. Fix direction: return the predecessor sub-DAG (or direct + one canonical chain),
  policy enumerates on demand. First-consumer-owned. (notes.md design-stage; design §10 Lock.)
- **R-C — pre-consumer API churn (R2 realised).** Opaque-handle-heavy API
  (OverlayId/OrderSpec/ChannelSpec/seed maps) means semantically-wrong-but-valid wiring compiles;
  expect a usage-driven interface rev when the adapter/policy slices land. Cheap while
  workspace-internal. (notes.md; design R2.)
- **R-D — `Against` orientation untested by a VT.** D2 resolved→oriented `U` re-map is implemented
  but every VT fixture uses `Along` (notes.md PHASE-03). First `Against` consumer should add
  coverage. Low risk (path exists, exercised indirectly), but a genuine coverage gap.

## Closure readiness

`audit.md`, `design.md`, `plan.toml`, and `notes.md` tell a coherent story: all PHASE-05 EX-1..5 /
VT-1..5 satisfied by tests/evidence; prior phases green and behaviour-preserved; both flagged
findings dispositioned **aligned**; no conformance defect; zero-dep + boundary contracts held.
PHASE-05 is ready to flip `completed` and the slice ready for `/close`.

**Leave-alone (not part of SL-036, untouched by this audit):** `AGENTS.md` + emacs lockfile,
modified `backlog/improvement/009|014`, untracked `memory/items/*` dirs (SL-037 / live user work).
