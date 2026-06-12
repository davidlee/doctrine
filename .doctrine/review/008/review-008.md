# Review RV-008 — reconciliation of SL-049

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciles SL-049's two landed phases against the shared seams they claim to
adopt and the input-validation hygiene they claim to deliver:

- **PHASE-01 / IMP-017** — does `memory list` actually ride the shared column
  model (`Column<R>` / `select_columns` / `render_columns`), preserving the
  F-A10/A11/A12 security scrub + full-uid contract, and is the old `--columns`
  rejection cleanly retired in lockstep across code, doc, and conformance?
- **PHASE-02 / ISS-004** — the slug cap. The phase advertises *filesystem
  safety*; the line of attack is whether the cap is a real wall or only a
  byte-length gate that still admits a path-component injection via an explicit
  `--slug` spliced into the `NNN-slug` symlink name.

Invariants held: no path-significant char reaches a filesystem name; the column
re-grid preserves the newline-forgery scrub; the gate stays green; reconcile
inside the slice, capture model-wide drift as owned work.

## Synthesis

**Outcome: audit-ready, blocker reconciled in-slice.** Four findings, all
terminal.

The clean half (IMP-017) needed nothing: the column-model adoption is textbook —
non-capturing `fn` cells, the security scrub kept on the free-text columns and
documented-as-deliberately-omitted on the closed-vocab ones, the full-uid F-A11
round-trip intact, and the `--columns` rejection retired in lockstep across
`main.rs` doc, `memory.rs`, and the conformance test. No drift.

The blocker lived in the half that *advertised* safety. PHASE-02 reasoned
carefully about a fat multibyte slug overflowing the 255-byte symlink name, then
returned an explicit `--slug` **verbatim** into that very symlink name — guarding
length but not the `/`, `.`, `..`, whitespace and control chars that are the
actual filesystem hazard (F-1). The TOML sink was escaped; the path sink beside
it was open. PHASE-02 widened the attack surface by adding `--slug` to a new verb
(`spec req add`) while the only sanitizer (`derive_slug`) sat on the *derived*
path the escape bypasses.

Reconciled in-slice (88fd7c5), at the **single shared chokepoint** `resolve_slug`
— so `slice new`, `adr new` and `spec req add` are all sanitized at once, no
parallel implementation. The charset is the user's chosen "mirror `derive_slug`"
policy (`^[a-z0-9]([a-z0-9-]*[a-z0-9])?$`): an explicit slug must now conform to
the shape derived slugs already guarantee, unifying both `resolve_slug` branches
under one slug-shape invariant. F-2 backfilled the missing hostile-`--slug`
coverage (e2e VT-4 + a unit table); F-3 reconciled the now-stale `SLUG_MAX`
"may be multibyte" rationale and the `truncate_slug` doc/test inconsistency.

Standing risk, consciously deferred: the column model's `*_DEFAULT`-vs-`*_COLUMNS`
desync is a *runtime* error, not a construction/compile-time one, across every
kind (F-4). Pre-existing and model-wide — out of SL-049's scope, captured as
IMP-038. No tolerated drift remains in scope.

Gate green (`just check`), clippy clean. Rollup 2/2; lifecycle `ready` ⚠ — the
expected hand-status lag, reconciled by `/close`.
