# Self-clearing dead_code suppression for a leaf built ahead of consumers

When a phase plan stands up a pure leaf module *before* the phase that wires its
first consumer (the common doctrine shape: spine/predicate leaf in PHASE-01, kinds
migrate onto it in PHASE-02+), the repo's `unused = deny` (dead_code) rejects every
public symbol that has no non-test caller yet. `#[cfg(test)]` usage does NOT count.

Fix (sanctioned house pattern): a **module-level** `#![expect(dead_code, reason=…)]`
whose reason documents the *self-clearing condition* — i.e. names the consumer phase
that retires it. It is genuinely self-clearing: an `expect` that becomes fulfilled
(the symbols are now used) turns into an `unfulfilled-lint-expectations` error under
`warnings=deny`, so the next phase is *forced* to delete the attribute. No drift.

Do NOT reach for a bare `#[allow(dead_code)]` (banned: allow_attributes) or
per-symbol expects (noisy, and each must be hunted down later).

Precedent: SL-008 PHASE-01 used exactly this on its pure predicate leaf
(`retrieve.rs:16`, since retired). SL-025 PHASE-01 reused it on `listing.rs` (the
read spine). Symbols already consumed via a relocation (e.g. `render_table`'s moved
callers) are exempt and stay outside the suppression's effect.
