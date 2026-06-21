# Review RV-125 — design of SL-134

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition arraigns SL-134's design.md — the architectural plan for
`doctrine risk set` / `doctrine risk clear`. The heretic claims to extend
`facet_write` with a mixed-type `FacetField` / `set_facet_mixed`, to mirror the
estimate/value pattern, and to gate writes on `kind = "risk"`.

**Lines of attack:**

1. Does `set_facet_mixed` honour the behaviour-preservation gate? Every existing
   `set_facet` caller and VT test must remain green unchANGED — additive only.
2. Is `FacetField` (Str + Arr) complete for the risk facet's TOML shape?
   `likelihood` / `impact` are strings in the scaffold; `origin` is a string;
   `controls` is an array. No other managed keys. No Float variant (D5).
3. Does the pure/impure split hold? `set_facet_mixed` is pure (toml_edit, anyhow,
   std); the command layer is impure. No disk, clock, or git in the pure layer.
4. Is the edit-preserving contract satisfied? Unknown sibling keys in `[facet]`
   survive set/clear; no full reserialize; no-op on identical values per type.
5. Are there missing validations, error paths, or silent corruptions? Shape
   errors (scalar where table expected), kind-gate edge cases (non-backlog
   entities), partial-write consistency.
6. Does the ARGS design leverage clap correctly? `RiskLevel::ValueEnum` for
   `--likelihood`/`--impact`; `Vec<String>` for `--controls`; optional flags;
   at-least-one guard.
7. Are the design decisions (D1–D6) defensible and complete? Attack each from
   the invariants above and the design skill's state machine.

**Invariants pinned to the accused (from the domain_map):**
- Behaviour-preservation gate, pure/impure split, ADR-001 leaf boundaries,
  edit-preserving mutation, no parallel vocabulary, idempotence, forward-compat,
  kind gate enforcement.
