# Review RV-012 — reconciliation of SL-054

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-054 (table cell wrapping for terminal-width-constrained
output) against `design.md` (incl. the PHASE-03 dep erratum), `plan.toml`, ADR-001
(module layering / pure-imperative split), and the SL-053 behaviour-preservation
gate. Self-audit: reviewer == author.

Lines of attack:

1. **None-invariance** — does `term_width = None` render byte-identical to the
   SL-053 `Disabled` path? Did any golden move? (the determinism property: piped
   output stays width-free.)
2. **Pure/imperative split** — does any tty/ioctl/env read leak into the pure
   layer (`src/listing.rs`), or are both impurities (isatty + `crossterm::
   terminal::size()`) confined to `src/tty.rs` and injected as plain values?
3. **`force_no_tty()` unconditional** (D2/D6) — the styling tty-consult stays off
   under both arrangement arms.
4. **Dependency weight** — is the erratum's achievable invariant ("no new
   *compiled crate*", not "no `Cargo.toml` edit") actually met? Single crossterm
   in the lock, events-feature crates absent.
5. **Behaviour-preservation gate** — SL-053 in-crate + black-box suites green
   UNCHANGED under the shared list-spine churn.
6. **Heuristic soundness** — is `grid_min_width`'s comfy-table coupling guarded
   by a test that can detect its own breakage? Is the end-to-end wrap (the user
   objective) actually exercised, or only its parts?

Carried in from the code-review pass (this session): two 🟠 risk findings the
user VH-accepted (end-to-end wrap unverified; grid-floor sliver-blindness), two
🟡 fixes already landed (`aa5a3a2`), one 🔵 deferral backlogged (IMP-044), one 🔵
withdrawn. This ledger records their disposition for the close trail.

## Synthesis

SL-054 conforms to its design and governance, and every gap is consciously
dispositioned. No blockers; the close-gate is clear.

**Conformance — verified green.**

- **None-invariance holds.** No golden moved (`git status` clean of
  golden/snapshot files); `term_width = None` renders byte-identical to the
  SL-053 `Disabled` path. The determinism property — piped output is width-free —
  is intact, pinned by the unchanged SL-053 exact-shape tests.
- **Pure/imperative split holds (ADR-001).** Both impurities (isatty probe +
  `crossterm::terminal::size()` ioctl) are confined to `src/tty.rs` and injected
  as a plain `Option<u16>`; `src/listing.rs` stays a pure leaf. `force_no_tty()`
  is unconditional under both arrangement arms (D2/D6).
- **Dependency invariant met.** The erratum's achievable claim — "no new
  *compiled crate*", not "no `Cargo.toml` edit" — is verified: a single
  `crossterm 0.29.0` in the lock, and every events-feature crate
  (`mio`/`signal-hook*`/`derive_more`) absent. `default-features = false` did its
  job; net compiled weight is zero.
- **Behaviour-preservation gate green.** `just check` passes (full bin +
  e2e suites); the shared list-spine churn (RenderOpts through ~10 sites,
  including the easy-to-miss `rec.rs` empty-result branch) preserved every
  existing suite, exactly as the `Default` = deterministic-path design predicted.

**Standing risks — consciously accepted.**

- **F-1 (tolerated).** The end-to-end wrap — the user objective — has no pty
  coverage; the live shell-width→`render_table` seam is design-scoped-out and
  VH-verified manually this session. Bracketed by the None-invariance goldens and
  the pure unit tests; only the isatty/ioctl plumbing between them is unautomated.
- **F-2 (follow-up → CHR-005).** `grid_min_width`'s comfy-table 7.2.2 coupling is
  guarded by tests that pin the formula and "wraps to >2 lines", not sliver-
  freedom — they can't detect their own breakage on a comfy bump. comfy is
  workspace-pinned, so the exposure is a deliberate-upgrade event, captured as a
  test-hardening chore.

**Reconciled in-slice.** F-3 (MIN_WRAP_WIDTH comment honesty) and F-4 (dangling
erratum cross-ref) were fixed before this audit landed (`aa5a3a2`).

**Deferred by design.** F-5 — the RenderOpts bundle is half-applied: priority
human surfaces and the `render_table` primitive take a bare `term_width`, a
sanctioned §7 D1 carve-out (priority monochrome, colour deferred). Captured as
IMP-044 for when priority gains colour.

**Tradeoff accepted at slice level.** This slice buys terminal-aware wrapping at
the cost of two test-confidence gaps (F-1 seam, F-2 floor) on heuristics that are
correct-by-inspection and VH-verified but not regression-fenced end-to-end. Both
are owned (design scope-out / CHR-005) rather than silently shipped. Audit-ready
for `/close`.
