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

## Disposition — resolved 2026-08-03

Both arms consolidated; the residual described above is discharged.

**`--print` arm.** `scripts/pi-agent` now holds the whole lifecycle and takes a
profile name. `pi-scout` and `pi-research` are shims setting only the profile
and the default thinking level, so the entry points `CLAUDE.md` and
`.doctrine/governance.md` name are unchanged and no caller moved. A latent hole
closed on the way: an empty or whitespace-only query was spending a real API
call on an empty prompt and exiting 0 with no output — `pi-scout < /dev/null`,
which was the documented workaround while ISS-266 was open.

**`--mode rpc` arm.** The 25 byte-identical poll/reap lines in all four spawn
scripts moved to `scripts/lib/pi-reap.sh` (`pi_await_and_reap`); 260 lines out,
37 in. `PREFIX=( … )` was left literally in place, so the `jail.rs` parity
scraper is unaffected and `bwrap_core_argv_matches_pi_spawn_core_flags` passes.

Hardened on extraction: pids are validated as integers > 1 before any signal
rather than defaulted, because `kill -9 -0` signals the *caller's* process group
— shared machinery with a `${pi:-0}` fallback would fell its own orchestrator
(see [[mem.pattern.shell.default-must-be-inert-in-its-consumer]]).

Tested — `just pi-reap-test` (`scripts/lib/pi-reap-test.sh`), 15 assertions, no
pi/API/network: whole-tree reap against a grandchild target, `agent_settled`
detected behind 700KB of padding (the ISS-293 geometry), elapsed-vs-backstop
reporting and the burn warning, and every invalid-pid case with a positive
control that the calling shell survives. Not wired into `check`/`gate` — it
spawns real processes and sleeps.

The helper documents two preconditions it cannot enforce: `set -m` before the
caller's first background job, and being called from the shell owning the pids.
The second bites through `$(...)` — command substitution is a subshell, so
capturing the report breaks the `wait` and leaves the zombies the function
exists to prevent.

## Links

- The pattern: `scripts/pi-review.sh`.
- Repaired in the same review pass: **CHR-051**, **ISS-266**, **IMP-322**.
- Durability blocker on the profiles those runners load: **ISS-304**.
- Operational context for the review fan-out the pattern came from: **IMP-024**.
