# ISS-204: SL-182 jail.rs test bakes env!(CARGO_MANIFEST_DIR) — CHR-014 guard red

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`src/worktree/jail.rs`'s `pi_spawn_core_tokens()` VT-7 test helper builds a path
with `env!("CARGO_MANIFEST_DIR")`. The `tests/e2e_no_baked_paths.rs::no_baked_paths`
guard (CHR-014 / SL-162) bans compile-time path-baking macros — so the FULL `test`
gate (`doctrine check commit` / `gate`) is **red on the baseline**.

## Provenance (verified)

Introduced by SL-182 `b67b6299 feat(SL-182): PHASE-02 TDD T1-T8`. Confirmed present
at a clean detached-HEAD checkout (not an SL-183 artifact). Jail *unit* tests
(`cargo test worktree::jail::tests`) pass — only the e2e guard, which the unit run
does not exercise, catches it.

## Fix

Swap `env!("CARGO_MANIFEST_DIR")` for the runtime form the guard mandates —
`test_support::repo_root()` (join `scripts/pi-spawn-confined.sh` at test time).
Behaviour-identical; test-only; touches no jail logic.

## Cross-slice coordination (2026-07-01)

- **Owned by the SL-182 thread** — SL-182 is being actively worked in a parallel
  context; the fix was handed to that agent thread (David, 2026-07-01). SL-183 must
  NOT edit jail.rs's SL-182 test surface (conflict risk + ownership).
- **Blocks SL-183 PHASE-02 close:** SL-183's pure builders are green in isolation
  (41 jail unit tests) but PHASE-02 cannot flip `completed` on a green full gate
  until this lands. See SL-183 `notes.md`.
