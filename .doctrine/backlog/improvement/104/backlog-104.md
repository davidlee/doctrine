# IMP-104: General pi subagent spawn pattern for scout/fix-agent roles (non-dispatch)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Assessed 2026-08-02 — mostly answered, and the obvious residual is constrained

Raised as a stub. Reviewed alongside CHR-051 / ISS-266 / IMP-322, which repaired
the whole `pi` script surface; that pass largely overtakes this item.

**The pattern exists: `scripts/pi-review.sh`.** It is already the general
non-dispatch runner this item asked for — confined (`--ro-bind / /`, rw re-granted
to exactly `~/.pi` and `$OUT_DIR`), read-only by OS guarantee rather than by
prompt instruction, no worktree and no fork (so the whole `isolation: worktree`
moving-base hazard class does not apply), parameterised by label / prompt file /
out dir, with a completion poll and reap that are now the reference for the other
three rpc scripts. A new read-only role — a scout, a census, a correspondence
check — needs a prompt file and a label, not a new script.

**What is left is a DRY question, and it is gated.** The surface is now six
scripts in two arms:

| arm | scripts | shape |
|---|---|---|
| `--mode rpc` | `pi-spawn.sh`, `pi-spawn-confined.sh`, `pi-respawn-nofork.sh`, `pi-review.sh` | fifo + poll + reap |
| `--print` | `pi-scout`, `pi-research` | one-shot, self-exits on stdin EOF |

The four rpc scripts carry four near-copies of the same fifo/poll/reap block.
That is the duplication worth removing — but **do not hoist the `bwrap` flags to
do it**: `src/worktree/jail.rs` (`pi_spawn_core_tokens`, exercised by
`bwrap_core_argv_matches_pi_spawn_core_flags`) *scrapes*
`scripts/pi-spawn-confined.sh`'s `PREFIX=( … )` region and asserts byte-parity
with the Rust `bwrap_core_argv()` builder. A prior worker hoisted exactly this
block and broke the test; see
[[mem.pattern.dispatch.pi-spawn-parity-scraper-couples-to-script-format]]. A
consolidation must either leave that region literally in place or co-update the
scraper deliberately.

The two `--print` scripts are **not** candidates for the same runner: they have
a different lifecycle (no fifo, no poll, no reap — closing stdin is the whole
termination story) and folding them in would import machinery they do not need.
They duplicate each other almost entirely, though, and could reasonably collapse
to one runner taking a profile name.

## Disposition

Left open for a decision on the DRY residual only; the "general pattern" half is
answered. Reasonable outcomes: re-scope to *"collapse the four rpc scripts onto
one poll/reap, `PREFIX` untouched"*, or close as answered and let the duplication
stand — four small scripts with divergent call contracts is not obviously worse
than one runner with four modes.

## Links

- The pattern: `scripts/pi-review.sh`.
- Repaired in the same review pass: **CHR-051**, **ISS-266**, **IMP-322**.
- Durability blocker on the profiles those runners load: **ISS-304**.
- Operational context for the review fan-out the pattern came from: **IMP-024**.
