`rustix::event::poll` does not work on file descriptors for `/dev/tty` or
`/dev/null` on macOS; rustix's own docs (rustix 1.1.4 `src/event/poll.rs`)
direct callers to `rustix::event::select`, which does work on those fds.

doctrine ships prebuilt binaries for macOS and Linux (README), so any
terminal query/reply exchange that waits on the controlling tty with a
deadline must use `select` (or cfg-split) rather than `poll`.

Surfaced by RV-368 F-12 against SL-245's kitty graphics support probe.
