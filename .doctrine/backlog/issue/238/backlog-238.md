# ISS-238: candidate create deadlocks on large trees: git_stdin writes all stdin before reading stdout

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`doctrine dispatch candidate create --slice 227 --role review_surface --payload
impl_bundle --base refs/heads/main --label ... --worktree` hangs indefinitely
(observed at SL-227 fix-now, 2026-07-25). No ref/row/worktree is written; the
process sits at ~0% CPU.

## Root cause (confirmed)

A classic full-pipe deadlock in `src/git.rs::git_stdin`:

```rust
child.stdin.take()?.write_all(raw)?;   // writes ENTIRE stdin first...
let output = child.wait_with_output()?; // ...only reads stdout AFTER
```

`git_stdin` writes the whole input to the child's stdin **before** reading any
stdout. `custom_merge_driver_paths` (an SL-212 create/ingest provenance guard,
`src/git.rs`) enumerates the *entire merge tree* (`ls-tree -r -z`) and batches the
full NUL-joined path list through `check_attr_merge_z` →
`git check-attr merge -z --stdin`. For a large tree (doctrine itself: thousands of
files), both the stdin path-list and git's stdout (`<path>NUL merge NUL <value>NUL`
triples) exceed the ~64KB pipe buffer: git blocks writing stdout because doctrine
isn't reading it; doctrine blocks in `write_all` because git isn't draining stdin.
Deadlock. Diagnosed via `/proc/<pid>/wchan` = `anon_pipe_write` on BOTH the parent
(`dispatch candidate create`) and the child (`git check-attr`).

Deterministic for any tree whose check-attr I/O exceeds the pipe buffer; SL-227's
command-tier reorg (many moved files) is likely the first large candidate create
since the batched-guard landed. `hash_object_stdin` shares the same `git_stdin`
seam and is latently exposed for large blob content.

## Impact

Blocks `dispatch candidate create` (review_surface + close_target) and
`dispatch candidate ingest` on large-diff slices — i.e. the standard `/audit`
fix-now-repair substrate (SL-165) and close's `close_target` projection. Real
`git merge --no-ff` in `worktree/land.rs` is NOT affected (git manages its own
attr I/O), so `sync --integrate`/close-without-fix-now still work.

## Fix

Make `git_stdin` (and by symmetry the check-attr batch) drain stdout concurrently
with the stdin write — spawn a thread for the `write_all` (dropping the stdin
handle to close it) while the main thread `wait_with_output`s, or read stdout on a
thread. ~10 lines, TDD with a large-input regression that reliably exceeds the
pipe buffer. Behaviour-preservation: existing small-input callers must stay green.

## Resolution — fixed (2026-07-25)

Landed as prescribed, in `f53fc81a` *"fix(ISS-238): drain git_stdin stdout
concurrently with the write"* (`src/git.rs`, +51/-7). On `main` since the
2026-07-25 edge promotion.

- **Mechanism.** `git_stdin` now feeds stdin from a `std::thread::scope` thread
  while the calling thread drains stdout/stderr via `wait_with_output`; the moved
  `stdin` handle drops at thread end, giving git its EOF. A broken pipe from the
  write is bind-and-ignored deliberately — when git exits early (e.g. bad args)
  its *stderr* is the real diagnostic, and propagating `EPIPE` would mask it.
- **Regression.** `custom_merge_driver_paths_does_not_deadlock_on_large_tree`
  (`src/git.rs` tests) drives a ~2000-file tree through the batched check-attr
  path; it must complete rather than hang, and still flag only the custom driver.
- **Latent exposure closed too.** The fix sits in `git_stdin` itself, so
  `hash_object_stdin` — flagged above as sharing the seam — is covered by
  construction, not left as a residual.
- **Behaviour preservation.** Full gate green on the merged tree (zero test
  failures); existing small-input callers unchanged.

Surfaced by SL-227's fix-now candidate create; the command-tier reorg was the
first tree large enough to exceed the pipe buffer since the SL-212 batched
provenance guard landed.
