# Notes SL-012: memory-record symlink tolerance

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Deliberate mirror drift — IMPR-003 batching NOT ported (A-1)

forgettable's DE-010 bundled **two** changes: symlink tolerance **and** IMPR-003 —
batching untracked regular-file hashing into one `git hash-object --stdin-paths`
fork (`untracked_hashes`, `forgettable/src/git_context.rs:830`). doctrine's re-sync
mirrored only the **symlink half** and deliberately left `untracked_fingerprint`
(`src/git.rs`) forking `git hash-object -- <path>` once per untracked file.

This is **output-equivalent but source-divergent**: a git blob oid is
fork-count-invariant, so the `untracked_fingerprint` / `checkout_state_id` bytes are
identical either way (conformance holds — DEC-010-06, golden vector unmoved). The
two files are therefore structurally further apart than before, by intent.

The next forgettable re-syncer: read doctrine's per-path untracked hashing as
**intentional, not rot**. If you port IMPR-003 to "catch up", you reintroduce
forgettable's Finding A — a path containing `\n` cannot ride LF-separated
`--stdin-paths` and needs the per-argv fallback. `tracked_symlink_repoint_is_dirty`
and `untracked_newline_in_name_is_deterministic` (both `src/git.rs`, added in the
SL-012 audit follow-up) pin the safe behaviour; keep them green through any port.

(A-4's `is_ok_and`-swallows-lstat-failure footgun is **accepted as inherited** —
the fix belongs upstream in forgettable, not in the mirror.)
