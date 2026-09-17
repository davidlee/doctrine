# `/dev/tty` fstat reports the devnode, not the terminal

To check "is stdout the terminal I am running in?", do **not** compare device
ids. On Linux:

    stdout (a pty)   st_rdev = (136, N)     /dev/pts/N
    open("/dev/tty") st_rdev = (5, 0)       the /dev/tty devnode itself

`/dev/tty` is a character device in its own right. The kernel redirects
*operations* on the resulting fd to the controlling terminal, but the fd's inode
stays `/dev/tty`, so `fstat` reports `(5,0)` forever. **The equality is
unsatisfiable**, and code that relies on it refuses every terminal in existence
while looking entirely reasonable.

## Identify by session instead

A controlling terminal belongs to exactly one session, and a session has at most
one controlling terminal. So two fds reporting the same session ARE the same
terminal — a total, portable identity:

```rust
fn controlling_session<Fd: AsFd>(fd: Fd) -> Option<i32> {
    rustix::termios::tcgetsid(fd).ok().map(|s| s.as_raw_nonzero().get())
}
// Same terminal iff both are Some and equal. Two `None`s are NEVER a match:
// neither fd named a session, so nothing was identified.
```

`tcgetsid` is POSIX, lives in rustix's `termios` feature (no extra feature if
you already use `tcgetattr`/`tcgetwinsize`), returns `ENOTTY` for an fd that is
no session's controlling terminal, and costs one syscall instead of two `fstat`s.

## It also dissolves the `dev_t` portability trap

`Stat::st_rdev` is `u64` on Linux and `dev_t` is `i32` on macOS, so a
device-id comparison needs a platform-conditional conversion — and the obvious
`u64::try_from(..).unwrap_or(0)` maps a negative `dev_t` to `0`, making two
*distinct* terminals compare equal. A pid is `i32` everywhere. (`ISS-456` existed
only because of this, and closed when the comparison moved to sessions.)

## How it shipped

SL-245 `-X`. Seventeen automated verification criteria passed: the pure decision
was tested over injected device ids, and both e2e tests ran with stdout on a pipe
so they only ever reached refusal paths. It took one human typing the command.
See [[mem.pattern.testing.injected-probes-leave-the-adapter-untested]].

Related: [[mem.fact.rustix.poll-dev-tty-macos]] — the other `/dev/tty` trap in
the same seam.
