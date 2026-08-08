# `unsafe_code = "forbid"` is workspace-wide, and `forbid` is not `deny`

`Cargo.toml:200` (`[workspace.lints.rust]`) sets `unsafe_code = "forbid"`. Every
member inherits it through `[lints] workspace = true`, including
`crates/doctrine-control`. There is **zero** `unsafe` anywhere in the repository
— the posture is real, not nominal.

`forbid` differs from `deny` in exactly the way that matters here: it **cannot be
overridden** by `#[allow(unsafe_code)]` or `#[expect(unsafe_code, reason = …)]`
at any scope. Attempting it is itself an error. So the usual escape hatch — one
documented `#[expect]` at one reviewed site — is unavailable without editing the
workspace manifest.

## What this actually blocks

Measured with `rustc --edition 2021` on two-line probe files (SL-248 PHASE-05
planning):

```rust
#![forbid(unsafe_code)]
unsafe { BorrowedFd::borrow_raw(fd) }   // error: usage of an `unsafe` block
unsafe { command.pre_exec(|| Ok(())) }  // error: usage of an `unsafe` block
```

Two whole capabilities fall out of reach as a result:

1. **Anything that starts from a `RawFd`.** `rustix::io::fcntl_setfd` and
   `fcntl_getfd` are `<Fd: AsFd>`; the only route from a raw integer (what
   `/proc/self/fd` yields) to an `AsFd` is `BorrowedFd::borrow_raw`, which is
   `unsafe`. `OwnedFd::from_raw_fd` likewise, and it would close the descriptor
   besides. `close_range` would sidestep it and is **absent from `rustix` 1.1.4**.
2. **Anything between `fork` and `exec`.** `std::os::unix::process::CommandExt::
   pre_exec` is `unsafe fn`. So `setrlimit`-on-the-child, `setsid`-on-the-child
   and friends are unreachable.

`src/worktree/claim_lock.rs:92` is *not* a counter-example: it passes `&File` —
a safe `AsFd` on an owned handle. Owning the descriptor is the whole difference.

## What to do

Do not reach for `#[expect]`; it will not compile and you will lose the time
twice. Either find a route that owns its descriptor, or treat the lint as a
governance question and get a ruling before writing the code — changing
`forbid` to `deny` is a workspace-wide safety-posture change, not a local fix.
