# Design SL-177: Default value floor for valueless actionable kinds

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

SL-176's `fulfils` **value-burndown** reduces a backlog item's priority by the
lifecycle-gated raw `value` of the slices fulfilling it. The subtraction is
**value-denominated**: it only signals anything for entities that carry a value.
A *valueless* work entity (no authored `[value]` facet) scores its value
dimension at `0` today, so burndown has nothing to denominate against and the
item is invisible to the undelivered-value signal.

SL-176 (D-value-floor-sibling, user-locked 2026-06-29) defers the fix to this
sibling slice as a **soft** dependency: give value-bearing actionable kinds a
**default value of 1.0** when none is authored.

## 2. Current State

`base_score` (`src/priority/graph.rs`, value-dim block ~L113) computes:

```rust
let raw = if let Some(ref v) = f.value {
    cfg.coefficients.value * v.value * kw * tag_term / cost
} else {
    0.0
};
```

The `None` branch drops every valueless entity — work entity or knowledge
record alike — to `value_dim = 0`.

The set of "value-bearing actionable" kinds **already exists**, function-locally,
as `WORK_PREFIXES` in `src/priority/surface.rs` (SL-089 D2, the actionability
graph's node set): `["SL", "ISS", "IMP", "CHR", "RSK", "IDE"]` — a slice plus the
five backlog kinds. It is identical to SL-176's {slice, backlog}. `kinds.rs`
already holds the sibling groupings `BACKLOG`, `RECORD` and the `is_record`
predicate, but no `WORK` grouping.

## 3. Forces & Constraints

- **STD-001 (no magic strings).** The value-bearing kind set must have a single
  named source, not a literal copied into a second call site.
- **No parallel implementation (CLAUDE.md).** `WORK_PREFIXES` already encodes the
  set; promote and reuse it rather than minting a parallel constant.
- **ADR-001 layering.** `kinds.rs` is leaf tier; `priority/*` (engine) may depend
  on it. The predicate belongs in the leaf.
- **Storage rule / A-1.** The default is applied at the **scoring seam**, never by
  mutating authored TOML — no derived data written to authored files.
- **Behaviour-preservation gate.** The `base_score_*` suite and the `surface`
  actionability view are the proof: they must stay green; the `surface` refactor
  is set-preserving.
- **SL-176 soft coupling (R12).** SL-176 lands and tests green on explicitly-valued
  entities without this slice; this closes only the valueless case.

## 4. Guiding Principles

Default-when-absent, not a clamp. Single named home for the kind set. One-line
seam change. Tunability deferrable without rework.

## 5. Proposed Design

### 5.1 System Model

Two pure changes over the engine tier, one shared constant promotion:

1. `kinds.rs` gains the `WORK` grouping + `is_work` predicate (mirroring
   `BACKLOG` / `is_record`).
2. `value.rs` gains the `DEFAULT_VALUE` const.
3. `graph.rs::base_score` consults both: a valueless **work** kind scores as if
   `value = DEFAULT_VALUE`; a valueless **record** stays at `0`.
4. `surface.rs` drops its local `WORK_PREFIXES` and consumes `kinds::is_work`.

### 5.2 Interfaces & Contracts

`src/kinds.rs` (leaf):

```rust
/// Value-bearing "work" kinds (SL-089 D2): a slice plus the five backlog kinds.
/// The set that carries a value facet and feeds priority burndown; governance
/// and knowledge records are excluded.
pub(crate) const WORK: &[&str] = &[SL, ISS, IMP, CHR, RSK, IDE];

/// Membership predicate over [`WORK`] — the single source for "is this a
/// value-bearing work kind?" so adding a work kind edits WORK, not call sites.
pub(crate) fn is_work(prefix: &str) -> bool { WORK.contains(&prefix) }
```

`src/value.rs`:

```rust
/// Default value for a value-bearing work entity that authors no `[value]`
/// facet (SL-177; SL-176 D-value-floor-sibling). Applied at the priority
/// scoring seam, never written to authored TOML. A default-when-absent, NOT a
/// min-clamp: an authored value below this is left untouched.
pub(crate) const DEFAULT_VALUE: f64 = 1.0;
```

### 5.3 Data, State & Ownership

No new state, no storage change. The default exists only inside `base_score`'s
pure computation; the authored `[value]` facet (or its absence) is the sole
durable fact. Ownership: the kind set is owned by `kinds.rs`; the default
magnitude by `value.rs` (the facet's home); the application policy by
`graph.rs::base_score`.

### 5.4 Lifecycle, Operations & Dynamics

`base_score` value-dim block becomes:

```rust
let value_dim = {
    let default = kinds::is_work(kind.prefix).then_some(value::DEFAULT_VALUE);
    let effective = f.value.as_ref().map(|v| v.value).or(default);
    let raw = match effective {
        Some(v) => {
            let cost = est_cost(
                f.estimate.as_ref().map(|e| (e.lower, e.upper)),
                ctx,
                &cfg.estimate,
            );
            cfg.coefficients.value * v * cfg.kind_weight(kind.prefix) * tag_term / cost
        }
        None => 0.0, // record kinds with no value → still zero
    };
    if raw.is_finite() { raw } else { 0.0 }
};
```

`f.value.as_ref().map(..).or(default)` is the precedence: **authored value wins**;
the default fills only the `None` case for work kinds; records get `None → 0.0`.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (no-clamp).** For any authored `v`, `effective == v` — the default never
  overrides a present value, including `v < 1.0` and `v == 0.0`.
- **INV-2 (record exclusion).** `is_work(prefix) == false` for every `RECORD`
  kind and every governance kind ⇒ their valueless `value_dim` is unchanged (`0`).
- **INV-3 (set identity).** `WORK == [SL] ∪ BACKLOG` — held by a canary test so a
  new backlog kind can't silently fall out of the work set.
- **Edge — non-finite.** The trailing `is_finite()` guard is preserved; a defaulted
  `1.0` through a degenerate `cost` still passes the same guard.
- **Edge — value 0.0 authored.** `Some(0.0)` ⇒ `effective = 0.0` ⇒ `value_dim = 0`,
  distinct from "absent" — authored zero is a real datum, not a gap.

## 6. Open Questions & Unknowns

- **OQ-1 (resolved — hard constant).** Default is a hard `const DEFAULT_VALUE = 1.0`,
  not config-tunable. SL-176 fixed the magnitude; a tunable default would invite
  divergence from the burndown contract. Tunability is a clean follow-up: swap
  `value::DEFAULT_VALUE` for a `PriorityConfig` field — no seam-logic change.
- **OQ-2 (resolved — reuse `WORK`).** Classification reuses the existing
  actionability-graph work set, promoted to `kinds::WORK` / `is_work`.

## 7. Decisions, Rationale & Alternatives

- **D1 — Apply at the scoring seam, not authored TOML.** Keeps storage honest
  (A-1); the default is a scoring policy, not a fact about the entity. Alternative
  (materialise `value = 1.0` on save) rejected: writes derived data, pollutes
  diffs, and fights the "absent vs zero" distinction.
- **D2 — Promote `WORK_PREFIXES` to `kinds::WORK`/`is_work`.** DRY + STD-001; one
  source for the set. Alternative (copy the literal into graph.rs) rejected as a
  parallel implementation that would drift from the actionability view.
- **D3 — Hard constant (OQ-1).** See above.
- **D4 — Default-when-absent, not min-clamp.** Matches SL-176 ("governs the
  valueless case" only). Naming is `DEFAULT_VALUE`, not `VALUE_FLOOR`, to forbid a
  `max(v, 1.0)` misreading. Alternative (clamp) rejected: would rewrite authored
  sub-1.0 values, an unrequested behaviour change.
- **D5 — Const home in `value.rs`.** The default belongs with the facet it
  defaults, not in `priority/config.rs` (which owns *coefficients*, a different
  axis). `is_work` lives in `kinds.rs` (leaf), reachable by both `priority` sites.

## 8. Risks & Mitigations

- **R1 — Work-set / value-bearing divergence.** `WORK` doubles as the
  actionability-graph node set (SL-089 D2) and the value-bearing set. They are
  identical today; a future kind could be one but not the other. *Mitigation:*
  documented coupling note on `WORK`; if they diverge, split the predicate then
  (YAGNI now). Low likelihood, low cost to split.
- **R2 — SL-176 burndown shape shifts before this lands.** Soft dep. *Mitigation:*
  the seam contract is just "valueless work kind → value_dim as if 1.0"; SL-176
  consumes the resulting `value_dim`, not this slice's internals. Re-check the
  `None`-branch only if SL-176's denomination changes.
- **R3 — `surface.rs` refactor regresses the actionability view.** *Mitigation:*
  set-preserving change (same six prefixes); existing surface tests are the gate.

## 9. Quality Engineering & Validation

TDD, red→green→refactor. New / asserted behaviour:

- **VT — valueless work kind defaults.** Valueless `SL` (and one backlog kind) →
  `value_dim` equals the explicitly-`value = 1.0` computation. Red against current
  `0.0`.
- **VT — record exclusion.** Valueless `ASM` → `value_dim == 0` (unchanged).
- **VT — no-clamp.** Authored `value = 0.3` on `SL` → `value_dim` reflects `0.3`,
  not `1.0`.
- **VT — authored zero ≠ absent.** Authored `value = 0.0` → `value_dim == 0`.
- **VT — `WORK` set identity canary.** `WORK == [SL]+BACKLOG` (mirrors the existing
  `groupings_match_documented_membership` test).
- **Behaviour-preservation.** Existing `base_score_*` suite green unchanged; the
  `surface` actionability-view tests green unchanged after the const promotion.

## 10. Review Notes

(pending adversarial pass)
