`rustix::event::poll` does not work on file descriptors for `/dev/tty` or
`/dev/null` on macOS; rustix's own docs (rustix 1.1.4 `src/event/poll.rs`)
direct callers to `rustix::event::select`, which does work on those fds.

doctrine ships prebuilt binaries for macOS and Linux (README), so any
terminal query/reply exchange that waits on the controlling tty with a
deadline must use `select` (or cfg-split) rather than `poll`.

Surfaced by RV-368 F-12 against SL-245's kitty graphics support probe.

Also: in rustix 1.x `select` is an `unsafe fn`, and this workspace denies
`unsafe_code` with a two-site budget (`Cargo.toml` lints). A safe portable
alternative for a bounded tty reply read is a termios timed read: raw mode with
`VMIN=0`, `VTIME=n` (tenths of a second) on a blocking fd, looping `read`
until complete or deadline. SL-245 chose this over `select` (DEC-259).