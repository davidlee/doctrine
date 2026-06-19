# SL-107 — implementation notes

Durable harvest from the (disposable) phase-01 sheet + the RV-088 audit. The full
audit reasoning lives in `RV-088` (`## Synthesis` / `## Reconciliation Brief`).

## What shipped (PHASE-01, single phase by design — OQ-1)

Hand-port of `candidate/101/review-001`'s integration delta onto `main` (D3 —
port, not merge), base `5e8b2a64` → `1e382acd`, 6 files / +436:

- `src/value.rs` — NEW pure leaf (ADR-001): `ValueFacet`/`ValueConfig`, single
  finite `f64`, present-facet validation, unit `magic_beans`; V1–V7 + deserialize.
- `src/main.rs` — `mod value;`.
- `src/dtoml.rs` — `estimation`/`value` `#[serde(default)]` config fields; no
  eager validation (`dtoml::parse` stays the tolerant shared reader).
- `src/slice.rs` — `SliceDoc` +2 `Option` facet fields, `Eq` dropped (f64),
  fixtures updated, round-trip + malformed tests. **Parsed, not rendered.**
- `src/estimate.rs` — blanket `#![allow(dead_code)]` → item-level `expect`s.
- `install/doctrine.toml.example` — commented `[estimation]`/`[value]`.

Evidence (audit re-run, independent of phase sheet): `just gate` 2194 pass / 0
fail (+20 additive); plain `cargo clippy` clean, no expect fires; `spec validate`
corpus clean; VA-1 no display path in `slice.rs`.

## Durable findings

- **F-1 (→ RV-088 F-4, tolerated) — EX-4 deviation 5 → 6 expect surfaces.** Plan
  said exactly 5 item-level expects; reality is 6. The 6th is a module-level
  `#[cfg_attr(not(test), expect(dead_code, …))]` on `pub(crate) mod display;`,
  covering three unconsumed renderers SL-102 landed on `main` *after* EX-4 was
  authored. Consulted + User-approved (Option 1); zero edits to SL-102's
  `display.rs` (D2 boundary intact); self-clearing (fires unfulfilled when SL-102
  wires display in). Mechanism recorded as memory
  `mem.pattern.lint.module-decl-expect-propagates` (high, verified).
- **F-2 (→ RV-088 F-5, aligned) — baseline reset.** Initial `just gate` was red on
  two unrelated date-dependent supersede tests (real-clock leak). User reset `main`
  to clean green `5e8b2a64` before the port; behaviour-preservation datum sound.

## Standing obligations (not SL-107's)

- SL-102 (display) / SL-103 (graph) must **remove** the relevant `expect`s as they
  consume the helpers — the expects fire unfulfilled otherwise. Tracked by the
  tripwires themselves; no backlog item needed.
