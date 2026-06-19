# CHR-015: Break conduct↔dtoml engine-tier import cycle (only core SCC after SL-111; surfaced by SL-112 design probe)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Design sketch (lightweight chore, governance §"small backlog items")

### The cycle (exactly two edges)

- `dtoml.rs` — `DoctrineToml.conduct: crate::conduct::ConductConfig` →
  **dtoml→conduct** (type). *Correct layering*: the aggregate reader holds each
  leaf sub-config's shape. Identical, downward edge for the four siblings
  `verify` / `estimate` / `value` / `dispatch_config`.
- `conduct.rs::parse` — `crate::dtoml::parse(text)?.conduct` →
  **conduct→dtoml** (fn call). *The wart.* The lone back-edge; no sibling has a
  `parse` wrapper. It is conduct's only production `crate::` reference, so it is
  the entire cycle.

`conduct ↔ dtoml` is the only remaining strongly-connected component in the
engine core after SL-111 (SL-112 design §2). SL-112 pins it with
`TANGLE_BASELINE[Engine] = 1`; removing the edge here lets that baseline tighten
to `0`. Decoupled by the ratchet — order-independent with SL-112.

### Fix — D1: delete the back-edge, repoint the sole caller

1. Delete `conduct::parse` (the wrapper that delegates back to `dtoml::parse`).
   conduct then imports only `std` + `serde` → a true leaf in production.
2. Repoint the one production caller, `slice.rs::load_conduct`, from
   `crate::conduct::parse(&text)` to `crate::dtoml::parse(&text)?.conduct` —
   the established sibling pattern (cf. `coverage_store.rs`:
   `dtoml::parse(&text)?.verification`).

Forward edge `dtoml→conduct` (the `ConductConfig` type field) stays — it is
correct, downward, and symmetric with the four siblings.

**Rejected alternative** — invert ownership (conduct owns the read; dtoml drops
the type): breaks SL-057 D2 ("`dtoml` is THE single `doctrine.toml` reader") and
the symmetric sibling pattern. D1 is effectively forced.

### Fix — D2: re-home conduct's test parse calls

conduct's `#[cfg(test)] mod tests` builds fixtures via the (now-deleted) local
`parse` (~12 call sites; the private serde-only `ConductConfig` fields have no
in-module constructor, so the resolve-precedence tests genuinely need it).

Add a **test-local helper** inside `mod tests`:

```rust
fn parse(text: &str) -> anyhow::Result<ConductConfig> {
    Ok(crate::dtoml::parse(text)?.conduct)
}
```

Zero call-site churn; one `crate::dtoml` reference, confined to `#[cfg(test)]`.
The SL-112 gate governs the **production graph only** — `#[cfg(test)]` edges are
out of contract by construction (SL-112 design §4), and `verify.rs` tests already
reference `crate::dtoml`. So this re-introduces no governed cycle.

### Verification (behaviour-preserving)

Pure refactor — no behaviour change. The existing suites are the proof
(behaviour-preservation gate, AGENTS.md):

- conduct's full test module stays green unchanged (via the test-local `parse`).
- dtoml + slice + coverage_store suites stay green.
- `just gate` clean (clippy zero-warnings, `cargo test --workspace`).

No new assertion is strictly required; the cycle's absence is a structural fact
SL-112's gate will assert durably. Optional: a one-line note in conduct's module
doc that it is now a pure leaf with no `crate::` production deps.

### Footgun (recorded, not actionable)

The tracked slug alias `015-conduct-dtoml-cycle → 015` is a **symlink** (the
repo convention — every backlog item has one). `rm -rf <slug-alias>/` with a
trailing slash *follows the link and deletes the target dir's contents*. Operate
on the link itself (no trailing slash) if ever removing one; never with a
trailing slash.
