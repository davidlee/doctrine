# Sweep pub(crate) re-exports before declaring a primitive missing

Before declaring a primitive "missing" — and proposing to build it fresh — sweep
the crate's `pub use` / `pub(crate) use` re-export lines. The symbol may already
exist under a different module path than the one you searched, re-exported into
the namespace where a consumer would reach it. A name-scoped `rg` at the
definition site alone will miss it.

**Why:** declaring a primitive absent when it exists green-lights a **parallel
implementation** — the exact DRY / no-parallel-implementation trap. The duplicate
then drifts from the original.

**How to apply:** before concluding a primitive is absent, run both an `rg` for
the symbol *and* a grep for re-export lines (`pub use`, `pub(crate) use`) across
the crate. Reuse-or-relocate the existing seam; only build what is genuinely
absent.

**Origin:** RV-300 F-7 — REV-032 initially claimed `is_linked_worktree` was
missing and proposed rebuilding worktree-isolation detection. It already existed
as a re-exported primitive. The corrected requirement (SPEC-022 FR-010) reads
"existing seams are reused or relocated rather than reimplemented (e.g.
`is_linked_worktree` for isolation detection); only genuinely absent reads are
newly built."

Related: [[mem.pattern.doctrine.reference-map-suggestive-not-exhaustive]]
(pre-enumerated maps are suggestive, always `rg` after).
