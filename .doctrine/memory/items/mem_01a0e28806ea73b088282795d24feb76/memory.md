Launching `./scripts/pi-research` / `./scripts/pi-scout` as detached subshells
(`( ./scripts/pi-research < p > out & )`) inside one Bash tool call leaves
**0-byte output and empty stderr**: the processes are killed when the call
returns. A follow-up `pgrep` wait loop exits at once and reports success.

Run the threads in one call as plain background jobs plus `wait`, with the
harness's `run_in_background` so the call itself is tracked:

    ./scripts/pi-research < t1 > raw/t1.md & ./scripts/pi-scout < t2 > raw/t2.md & wait

Run from the repo root. Seen in the SL-273 research round, 2026-09-27.
