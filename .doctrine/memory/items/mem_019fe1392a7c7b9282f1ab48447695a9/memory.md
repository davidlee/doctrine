# bubblewrap 0.11.2 — five things measured, not read

Measured while planning SL-248 PHASE-05, **nested inside this project's own
bubblewrap jail** (so nesting is not an obstacle to any of it).

## 1. `--tmpfs /tmp` then `--ro-bind X /tmp/sub` works

bwrap **creates the mountpoint inside the tmpfs** and the bind succeeds; the
sandbox sees `/tmp/sub` and its contents. A readable input that resolves beneath
a profile-owned `--tmpfs` is therefore lawful and does not need a special case.

## 2. Reversed, it is silently lost

`--ro-bind X /tmp/sub` *before* `--tmpfs /tmp` leaves `/tmp` **empty** inside —
the tmpfs shadows the earlier bind entirely, and bwrap exits **0**. So mount
assembly order is load-bearing and its failure mode is a missing input with no
diagnostic anywhere. Assert the order, and mutate it to prove the assertion
discriminates.

## 3. bwrap does NOT close inherited descriptors

Parent opens fd 7 without `CLOEXEC` (`7< /etc/hostname`); inside the sandbox
`/proc/self/fd` shows `7 -> /etc/hostname`. Neither `--unshare-all` nor
`--clearenv` reaches this channel: an already-open descriptor is not a namespace,
a mount or an environment entry. If you want it closed, the **parent** must do
it before the fork.

## 4. …but it does close its own `--json-status-fd`

With `--json-status-fd 9`, fd 9 is absent from the child's `/proc/self/fd`. So
the status channel costs nothing against a "no descriptor above 2 crosses the
exec" invariant.

## 5. `execvp` failure is exit 1, and only the status fd distinguishes it

| what happened | bwrap wrapper exit | `--json-status-fd` output |
|---|---|---|
| child exits 42 | 42 | `{ "exit-code": 42 }` |
| child killed by SIGTERM | 143 | `{ "exit-code": 143 }` |
| **`execvp` failed** | **1** | container line only — **no `exit-code`** |

So a missing binary and a capsule that legitimately exits 1 are
**indistinguishable from the exit status alone**. bwrap does write
`bwrap: execvp <path>: …` to stderr, but stderr is shared with the sandboxed
process and is therefore forgeable by it — useless as a trust signal. The
presence or absence of `exit-code` on the status fd is the only parent-side
discriminator.

Note the residual: `{"exit-code": 143}` appears both for a child killed by
`SIGTERM` and for one that called `exit(143)`. Wrapping in `timeout(1)` does not
help — it collapses the same two. Pick unambiguous payloads when testing a
termination taxonomy.

## Related figures, same session

`timeout -k G S` exits **124** on expiry; a process killed by `SIGXFSZ` under
`ulimit -f` yields **153** (= 128 + 25).
