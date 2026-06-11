# SL-039 — audit

Conformance audit (post-implementation, slice-tied). Hand-authored — no audit
scaffold yet (known CLI gap). Reconciled against `design.md`, `plan.toml`, the
phase `VT-`/`EX-` criteria, and ADR-001/ADR-004.

Audited at HEAD `061c2b3` (PHASE-04). All four phases `completed`; `slice list`
rollup `4/4` (⚠ = `slice-039.toml` status still `proposed` — a `/close`
reconcile step, no lifecycle verb yet).

## Evidence run (2026-06-11, HEAD 061c2b3)

| check | result |
|---|---|
| `cargo test -p cordage` | green (all suites; 4 scale-cliff tests `ignored` by design) |
| adapter unit `backlog_order::tests` | 15 passed |
| backlog model unit `backlog::` | 66 passed |
| e2e `e2e_backlog_order_golden` (5 goldens) | 5 passed |
| VT-10 `no_pub_crate_signature_leaks_a_cordage_id` | passed |
| `git show --stat <c> -- crates/cordage/` ×5 SL-039 commits | 0 files each |
| `cargo tree -p cordage` | `cordage v0.1.0` alone, no doctrine in subtree |
| `just check` (lint + test + format) | green, zero clippy warnings |

## Disposition by criterion

### PHASE-01 — data model (`41a0279`)
- **EX-1..EX-4, VT-1** (needs/after/triggers serde model, template seeding, `show`
  renders three axes, cordage path dep) — **aligned.** Round-trip + render pinned
  by VT-1 unit tests (part of the 66 green); `list` goldens untouched.
- **EX-5** (`just check` green) — **aligned.**

### PHASE-02 — pure adapter `src/backlog_order.rs` (`7af085e`, corrective re-exec)
- **EX-1..EX-3, VT-2/3/4/8** (build/overlays/OrderSpec, the B→A flip, dep_cycles +
  overrides, exposure tiering, permutation-determinism) — **aligned.** Covered by
  the 15 adapter unit tests; corrective re-exec re-locked the PRD-009 vocab + the
  genuine `(rank,age,src,dst)` eviction.
- **EX-4 / VT-10** (no opaque cordage id in any `pub(crate)` signature; genuine
  eviction key) — **aligned.** VT-10 token-absence audit green.
- **EX-5** (cordage-side lint bans honoured) — **aligned.**

### PHASE-03 — CLI surface (`12fb716`)
- **EX-1..EX-4, VT-5/6/7** (`backlog needs`/`after`/`order`, cycle refusal naming
  members, genuine-key eviction, honest terminal/absent-endpoint record) —
  **aligned.** 11 unit + 5 black-box goldens; `list` goldens unchanged.
- **EX-5** (`just check` green) — **aligned.**

### PHASE-04 — leaf invariant + R-C harvest (`061c2b3`)
- **EX-1 / VT-9** (no `crates/cordage/**` diff; cordage stays a pure leaf) —
  **aligned.** Zero cordage diff across all 5 SL-039 commits; `cargo tree`
  confirms dependency-free leaf. The one `src/backlog_order.rs` delta (`12fb716`,
  17 lines, `expect(dead_code)` removal) is an *adapter* change in the doctrine
  crate, **not** a cordage leaf edit — EX-1 holds.
- **EX-2 / VA-1** (R-C interface finding recorded) — **aligned.** NULL result: the
  full cordage public surface drove a real consumer with zero API bend (EN-2 held
  all three phases). Recorded in `notes.md` PHASE-04 § +
  `mem.system.engine.cordage-rc-budget-closed-null` (trust=high).

## Findings & follow-ups

- **F1 — dup-`ItemId` bimap fragility (DD4).** Duplicate `ItemId` in `&[OrderInput]`
  silently corrupts the `by_item`/`by_node` bimap; the pure adapter does not fail
  loud on the precondition violation. Cannot arise in production today (PHASE-03's
  `project` enforces distinct ItemIds at the projection boundary, DD4). Latent for
  any future adapter caller that bypasses `project`. **Disposition: follow-up —
  filed `RSK-005`** (likelihood low / impact high). Owed-before-close obligation
  (`mem.system.lifecycle.defer-needs-backlog-before-close`) satisfied.
- **F2 — D-split `Resolution` taxonomy (design §9 OQ-D).** Satisfied-vs-abandoned
  resolution taxonomy was rejected for D-min, "captured as a follow-up IMP if it
  bites." **Disposition: tolerated drift — not owed this slice.** Did not bite in
  the audit; no backlog item filed (deliberate, per the design's own condition).

## Code review (adversarial sub-agent pass, 2026-06-11)

Net SL-039 code diff (`3add407..HEAD` over `src/backlog.rs`, `src/backlog_order.rs`,
`src/main.rs`, `tests/e2e_backlog_order_golden.rs`; ~2335 insertions) reviewed
against `design.md`, `plan.toml`, ADR-001/004, and the cordage source.

**Verdict: shippable as-is — no must-fix findings.** Each load-bearing claim
verified against the cordage source (not the design's paraphrase): B→A flip on
both edges, genuine `(rank,age,src,dst)` eviction (age tests discriminate the real
key from the retired `(0,0)` stand-in), `Contradicted` cross-layer filter correct
(`overlay() == after_overlay`), exposure-as-within-level-fallback (never lifts
across levels), honest-record drop tiers (AbsentDrop + Dangling) both surfaced,
bounded R-C kill mechanically test-asserted. Purity held, ADR-001/004 + every
clippy ban respected. Two LOW observations (`classify_dangling` unreachable arm,
`by_item.get` dead-defensive `else`) — both already documented dead-defensive,
**no fix required**. Disposition: **aligned.**

## Closure readiness

`design.md`, `plan.toml`, the implementation, and `notes.md` tell a coherent
story. Every EX/VT dispositioned **aligned**; one latent finding routed to
`RSK-005`; one tolerated drift documented. Ready for `/close`: confirm rollup
`4/4`, reconcile `slice-039.toml` `proposed`→`done`, land the close commit.
