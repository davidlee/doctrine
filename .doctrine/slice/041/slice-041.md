# Resolve branch-point-check --base in the impure shell

## Context

ISS-002 (surfaced by the SL-031 re-audit, §5.2 C-V). `worktree::run_branch_point_check`
resolves `--head` to a sha (`git rev-parse HEAD`) when the flag is absent, but
passes `--base` through to `matches(base, head)` verbatim. `matches` is raw
string equality (`base == head`).

The verb's whole job is to refuse committing a batch onto a moved base. A
symbolic `--base` silently defeats it:

- `--base HEAD` (symbolic) vs a resolved `--head` sha → never equal → false
  "moved" — unnecessary re-dispatch (safe direction, wrong reason);
- both symbolic (`--base HEAD --head HEAD`) → `"HEAD" == "HEAD"` → **false
  stationary** — the guard passes against a base it never resolved (the unsafe
  direction).

The shipped `/dispatch` funnel captures `B = git rev-parse HEAD` and passes a
resolved sha, so the contract is currently safe in practice. A safety verb must
not trust its safety input to be pre-resolved.

## Scope & Objectives

- Resolve `--base` to a sha in the impure shell of `run_branch_point_check`,
  the same way `--head` is resolved, before the `matches` compare.
- Keep `matches` a pure leaf (ADR-001): ref-equality only; resolution stays in
  the shell.
- Add a VT row: symbolic base vs resolved head ⇒ non-stationary / error, and
  both-symbolic ⇒ no longer false-stationary.

## Non-Goals

- No change to `matches` semantics (still string ref-equality).
- No change to the `/dispatch` SKILL contract (already passes a resolved sha).
- No merge-base / branch-point computation — the operation stays a sha-equality
  assert (per the §5.2 naming note).

## Summary

One-surface fix in `src/worktree.rs`: lift `--base` resolution into the impure
shell so the guard can never compare against an unresolved symbolic ref.

## Follow-Ups

- On close, transition ISS-002 → resolved with the SL-041 link (outbound edge,
  ADR-004).
