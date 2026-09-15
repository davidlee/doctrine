# CHR-072: Verify the -X support probe on macOS

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

SL-245's kitty support probe waits for the terminal's reply with termios timed
reads (`VMIN=0`, `VTIME=1`) on `/dev/tty` (DEC-259). That was chosen because
`poll` does not work on `/dev/tty` on macOS and `select` is `unsafe` in rustix
(mem.fact.rustix.poll-dev-tty-macos). Timed reads are POSIX behaviour, but no
one has run the probe on macOS; doctrine ships macOS binaries.

## Check

SL-245 design sec-8 VH step 6 and the sec-9 assumption: on a macOS host, in
kitty or ghostty, run `doctrine graph <small focus> --depth 1 -X`. The image
appears and no stray reply text lands in the shell input. If the probe instead
refuses as unconfirmed, the timed read is not behaving and the tty seam needs a
macOS path.
