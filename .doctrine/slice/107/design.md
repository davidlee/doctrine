# Design: SL-107 — Estimate & Value facet integration

> **Authority.** Module internals (types, parse, normalise, validate, config) are
> already designed and locked in **SL-101 `design.md` §3–§7** and realised in
> `src/estimate.rs` (on `main`) + `src/value.rs` (on `candidate/101/review-001`).
> This doc does **not** re-derive them. It specifies only the *integration delta*
> that SL-101 designed but never landed on `main`, scoped **narrow** (display →
> SL-102, graph → SL-103).

## 1. Problem

SL-101 closed `done` but its delivery to `main` is incomplete (memory
`mem.fact.doctrine.sl-101-facets-unintegrated`): `src/estimate.rs` is present but
dead (blanket `#![allow(dead_code)]`, zero live refs), `src/value.rs` is absent,
and neither `dtoml.rs` nor `SliceDoc` carries the facets. The reconciled contract
(REV-002: PRD-014 / SPEC-020) is authoritative and unchanged by this slice.

The cause was a dropped dispatch funnel: the audit-fixed candidate
(`candidate/101/review-001`) was never re-created clean / admitted, reconciliation
went straight to `main`, the candidate orphaned, and the slice closed through a
gate that should have blocked. SL-107 is the remediation: a fresh slice (SL-101 is
terminal, ADR-009) that ports the candidate's integration delta onto current `main`.

## 2. Current vs target behaviour

| | Current (`main`) | Target (SL-107) |
|---|---|---|
| `src/value.rs` | absent | present (V1–V7), `mod value;` declared |
| `src/estimate.rs` dead-code | blanket `#![allow(dead_code)]` | item-level `expect(dead_code)` on display/graph helpers only |
| `EstimateFacet`/`ValueFacet` types | unreferenced | referenced from `SliceDoc` (live) |
| `doctrine.toml` `[estimation]`/`[value]` | not parsed | parsed into `DoctrineToml` (`#[serde(default)]`) |
| `SliceDoc` facet fields | none | `estimate`/`value` optional, parsed (not rendered) |
| config example | no facet sections | commented `[estimation]`/`[value]` |
| **display / graph** | none | **unchanged — out of scope (SL-102/103)** |

Behaviour the User sees is unchanged: facets parse if authored, are silently
carried, and never displayed or gated. The only externally observable change is
that a malformed authored `[estimate]`/`[value]` on a slice toml now errors at
parse (validation goes live via the `SliceDoc` serde path).

## 3. Port source & per-file reconciliation

Port `candidate/101/review-001` by hand. Divergence of `main` from the candidate
base (`ec2de06`), per target file:

| File | main drift | reconciliation |
|---|---|---|
| `src/value.rs` | none (absent) | adopt candidate verbatim |
| `src/dtoml.rs` | none | adopt candidate's two-field delta |
| `install/doctrine.toml.example` | none | adopt candidate delta |
| `src/slice.rs` | 3+/2- | apply SliceDoc fields + fixture updates onto main |
| `src/main.rs` | 176+/17- (unrelated CLI) | insert `mod value;` only |
| `src/estimate.rs` | 15+/8- | **attr-only** — main vs candidate differ *solely* in dead-code treatment; both already carry `espresso_shots`. Swap blanket allow → item-level `expect`. Logic/tests untouched. |

`estimate.rs` is the only file both sides touched, and the diff is purely the
dead-code attribute style — no logic merge. This makes the port low-risk; it still
runs through plan → phases → TDD, and the behaviour-preservation gate is the proof.

## 4. The dead-code decision (D1)

**Decision:** replace `estimate.rs`'s module-wide `#![allow(dead_code)]` with
item-level `#[cfg_attr(not(test), expect(dead_code, reason = "consumed by SL-102
display / SL-103 graph"))]` on exactly the still-unused surface:
`DEFAULT_LOWER_CONFIDENCE`, `DEFAULT_UPPER_CONFIDENCE`, `resolve_unit`,
`resolve_confidence`, `parse_optional`. Same treatment already on `value.rs`
(`resolve_unit`, `parse_optional`).

**Rationale.** Under the narrow boundary the facet *types* go live (SliceDoc fields
+ dtoml config), but the display/graph *helpers* have no consumer until SL-102/103.
A blanket `allow` would hide that and silently tolerate genuinely-dead future code.
`expect` (not `allow`) is a tripwire: when SL-102/103 wire a helper in, the
`expect` fires "unfulfilled" and forces its removal — the attribute self-documents
the integration debt and cannot rot. `cfg_attr(not(test), …)` because the unit
tests already exercise these fns, so the expectation applies only to non-test
builds. This is the candidate's own choice; SL-107 adopts it unchanged.

**Invariant:** plain `cargo clippy` (bins/lib, per project gate — not
`--all-targets`) must be warning-clean *and* no `expect` may fire. If an `expect`
is unfulfilled, a type/fn became live unexpectedly — stop and reassess scope.

## 5. Code impact

```
src/value.rs                    NEW   ~214 lines (candidate verbatim)
src/main.rs                     +1    (mod value;)
src/estimate.rs                 ~     blanket allow → 5 item-level expects (no logic)
src/dtoml.rs                    +~10  (estimation + value config fields + doc)
src/slice.rs                    +~6   (2 SliceDoc fields + fixture None,None sites)
install/doctrine.toml.example   +~8   (commented sections)
```

No new dependencies. No pure/imperative boundary change — `estimate.rs`/`value.rs`
remain ADR-001 leaves (no clock/disk/rng/git; config is passed in, file read stays
in the shell).

## 6. Verification alignment

| Concern | Evidence |
|---|---|
| Value facet model/parse/validate | V1–V7 (SL-101 design §7.2) — ported with `value.rs` |
| Estimate facet (already on main) | existing E-tests stay green unchanged |
| `dtoml` config wiring | D1/D2 (SL-101 design §7.3): empty → defaults; `[estimation]`/`[value]` set → parsed |
| `SliceDoc` carries facets | round-trip: a slice toml with `[estimate]`/`[value]` parses to `Some`, serialises back; absent → `None` (SL-101 §7.1 E17/E18 pattern) |
| Malformed facet errors at parse | invalid `[estimate]`/`[value]` on a SliceDoc errors (validation live via serde path) |
| **Behaviour preservation** | full existing suite green unchanged (`just gate`); the entity-engine / slice suites are the proof |
| Dead-code hygiene | `cargo clippy` clean, no `expect` fires |
| Non-blocking (NF-001) | structural: no dispatch/execute/audit/close predicate reads facet presence — unchanged, nothing added reads it |
| Display/graph **absent** | `rg 'Estimate:|Value:'` in `slice.rs` display path → no match; SliceDoc field is parse-only |

## 7. Decisions & open questions

- **D1 — item-level `expect(dead_code)`** (§4). Locked.
- **D2 — narrow boundary.** Display = SL-102, graph = SL-103; SL-107 is the shared
  prerequisite both assume. Decided with the User.
- **D3 — port, don't merge.** `main` diverged ~34 commits past the candidate base;
  `merge-tree` conflicts on `estimate.rs`. Hand-port; the candidate is reference.
- **OQ-1 — phase shape.** Single phase (the delta is ~6 mechanical file touches) vs
  two (value module + tests | wiring). Defer to `/plan`; lean single-phase given
  size, with value.rs TDD red/green before wiring.

## 8. Governance

- **ADR-001** (layering, leaf): `estimate.rs`/`value.rs` import only external crates;
  pure tier. Preserved.
- **ADR-009** (lifecycle): SL-101 terminal — SL-107 is a fresh remediation slice,
  `related` to SL-101, `specs` SPEC-020 + PRD-014.
- **Behaviour-preservation gate** (AGENTS.md): shared-machinery suites stay green
  unchanged — the integration must not perturb existing behaviour.
- **Storage rule:** no facet *display* or derived data added to authored prose;
  facets are structured TOML carried on `SliceDoc`.
