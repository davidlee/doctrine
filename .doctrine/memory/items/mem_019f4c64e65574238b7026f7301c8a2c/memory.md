# Auditing a dispatched slice's evidence ref in a fresh worktree: supply web/map/dist first

When `/audit` runs against a `/dispatch`-landed slice, the code lives on
immutable `review/*` + `phase/*` evidence refs, not on edge/main. To verify
behaviour independently (build, `slice verify-vt`, `clippy`) you spin a detached
scratch worktree on `review/<slice>`. Two traps waste cycles:

1. **`cargo build` fails immediately** with a RustEmbed error —
   `#[derive(RustEmbed)] folder '.../web/map/dist/' does not exist`. That
   web-map bundle is a *derived, gitignored* asset absent from any fresh
   checkout (same family as the flake `srcWithDist` embed roots). Fix before
   building: `cp -r <built-tree>/web/map/dist/. <worktree>/web/map/dist/`. The
   error is easy to misdiagnose — `tail -5` on the build log buries it under
   `icu_*` candidate noise, and `BUILD_DONE rc=$?` after a pipe captures the
   echo's exit, not cargo's. Grep the log for `^error`; never read `$?` through
   a pipe.

2. **`slice conformance` reads clean on neither tree pre-integrate.** Run from
   the parent tree (edge) it over-reports `undeclared` when the slice's authored
   selectors were remediated on the coordination surface and haven't landed yet
   — read the canonical registry directly instead
   (`git show <coord-or-review-ref>:.doctrine/slice/NNN/slice-NNN.toml`). Run
   from the detached worktree it reports `incomplete — partial coverage`
   because runtime phase-completion state is gitignored and absent there. The
   edge/coord split is inherent to the dispatch funnel before stage-2 integrate.

Corollary: trust the canonical authored tier from the coordination tip
(`dispatch/<slice>`), confirm the `review/<slice>` impl-bundle is byte-identical
to it in the authored tier, and record the reviewed surface in the RV `## Brief`.
The stale edge copies of `design.md` / `slice-NNN.toml` are *superseded* by
stage-2 integrate at `/close`, not corrected by `/reconcile`.

Related: [[mem.signpost.doctrine.audit]], [[mem.signpost.doctrine.dispatch]],
[[mem.pattern.dispatch.verify-governance-freshness-before-distilling-worker]].
