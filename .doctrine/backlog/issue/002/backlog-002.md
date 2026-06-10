# ISS-002: branch-point-check trusts --base unresolved — symbolic base defeats the guard

Surfaced by the SL-031 re-audit (code-review pass).

## Defect

`worktree::run_branch_point_check` (`src/worktree.rs`) resolves `--head` to a sha
via `git rev-parse HEAD` when the flag is absent, but passes `--base` through to
`matches(base, head)` verbatim. `matches` is raw string equality (`base == head`).

The verb's whole job is to refuse committing a batch onto a moved base. A symbolic
`--base` silently defeats it:

- `--base HEAD` (symbolic) vs a resolved `--head` sha → never equal → false
  "moved" → an unnecessary re-dispatch (safe direction, but wrong reason);
- both symbolic (`--base HEAD --head HEAD`) → `"HEAD" == "HEAD"` → **false
  stationary** — the guard passes against a base it never resolved, the unsafe
  direction.

The intended caller (the `/dispatch` SKILL) captures `B = git rev-parse HEAD` and
passes a resolved sha, so the shipped funnel is safe in contract. But a safety
verb should not trust its safety input to be pre-resolved.

## Fix

`rev-parse` (or sha-shape-validate) `--base` in `run_branch_point_check`, the same
way `--head` is resolved. Keep `matches` a pure leaf; resolution belongs in the
impure shell. Add a VT row: symbolic base vs resolved head ⇒ error/non-stationary.

Links: SL-031 §5.2 (C-V), [[mem.pattern.doctrine.tdd-loop]].
