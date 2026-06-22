# IMP-157: Select claim reach by configuration without changing callers

This fulfills REQ-021 (PRD-005).

The `Claim` trait in `src/entity.rs` abstracts claim acquisition behind a
one-method interface. Currently only the `LocalFs` backend ships (mkdir-based).
REQ-021 requires:

1. **Automatic selection** — resolve to broader reach (e.g. a shared remote via
   git refs) when available, falling back to single-tree (LocalFs) otherwise,
   surfacing the reduced reach rather than assuming it silently.
2. **Explicit override** — a configuration switch to force a specific backend.
3. **Caller transparency** — the caller (`claim_fresh_id`, `materialise_named`,
   etc.) doesn't change regardless of which backend is active.

Work items:
- Implement a `GitClaim` backend (claim via git ref push/compare-and-swap).
- Wire backend selection into project config (`doctrine.toml`).
- Keep existing `LocalFs` as the fallback default.
