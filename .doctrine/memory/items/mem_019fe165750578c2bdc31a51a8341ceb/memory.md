## The bite

SL-248 PHASE-05's parent-side descriptor sweep enumerates `/proc/self/fd`,
snapshots the numbers, then marks each one `CLOEXEC`. In a Rust **test binary**
the harness runs tests on many threads, so a descriptor listed an instant ago
can be closed before it is marked.

The first implementation skipped a descriptor whose `fcntl_getfd` failed but
**propagated** a failing `fcntl_setfd`. Result: one full-suite run in roughly
twenty failed `every_descriptor_above_two_is_marked_close_on_exec_before_the_exec`
with an error the mutation under test could not explain. It surfaced during a
mutation battery, as an *extra* red — which is the only reason it was noticed at
all rather than being written off later as "flaky CI".

## The rule

Skip on `rustix::io::Errno::BADF` **and only that errno**:

```rust
match fcntl_setfd(borrowed, flags | FdFlags::CLOEXEC) {
    Ok(()) => marked = marked.saturating_add(1),
    Err(Errno::BADF) => continue,
    Err(other) => return Err(other.into()),
}
```

A blanket `let _ = fcntl_setfd(...)` is the wrong fix: a sweep that swallows
every failure is a guard that cannot fail, which is the vacuous-guard class one
level down from the bug it papers over.

## Two adjacent notes

- Rust opens its own files `O_CLOEXEC`, so a `std::fs::File` passes against a
  sweep that does nothing. `rustix::io::dup` (plain `dup(2)`) does **not** set
  the flag, so a dup'd handle is the discriminating fixture — with a
  precondition assertion that it starts without `CLOEXEC`.
- The doc comment claimed the skip before the code implemented it. A comment
  describing a behaviour the code only half has is worse than none: it reads as
  a verified property.
