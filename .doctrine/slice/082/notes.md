# Notes SL-082: Dispose of doc/* as legacy heretical practice; rehome & archive

## Summary

Executed all 5 phases in serial (self/auto). 4 conventional commits on main.
`doc/` removed; every reference repointed to canonical entity surface.

## Key findings

- Design §3.2 reference map (K1–K11) missed `reconcile/SKILL.md:20` — caught during `rg` sweep and corrected.
- `doctrine claude install` is now `doctrine install`.
- `mem.pattern.build.rust-embed-no-rerun`: install template changes don't need re-embed.
- SL-095 pre-existing e2e test failure persists (unrelated).
- Catalog e2e tests flaked once (false-RED from shared CARGO_TARGET_DIR).

## Commits

- `a86a267` feat(SL-082): repoint all source code doc/ references (PHASE-01)
- `1d9f34e` feat(SL-082): repoint skill doc/ references (PHASE-02)
- `dcc98f8` feat(SL-082): remove doc/* references from install templates (PHASE-03)
- `36f1b43` feat(SL-082): update memory record doc/ references (PHASE-04)
- `e78b927` feat(SL-082): remove doc/ directory (PHASE-05)
