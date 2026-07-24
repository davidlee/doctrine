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
