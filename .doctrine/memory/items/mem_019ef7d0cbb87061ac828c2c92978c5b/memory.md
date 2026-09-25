# Jail id reservation needs DOCTRINE_RESERVATION_FALLBACK

In the bubblewrap jail, any doctrine command that **allocates a new id** (`slice new`,
`adr new`, `backlog new`, `spec new`, `memory record`, …) fails at the id-reservation
step:

```
Error: reach=auto: reservation remote origin unreachable and local fallback declined.
  git command failed: fetch origin +refs/doctrine/reservation/* ...
  fatal: cannot exec '.../git-ssh-disabled': No such file or directory
```

Cause: the reservation reach tries the remote origin first, but git ssh is disabled in
the jail (`git-ssh-disabled` shim).

## Fix

Prefix the command with the env var to allocate the id locally:

```bash
DOCTRINE_RESERVATION_FALLBACK=1 doctrine slice new "Title"
```

(Equivalent: set `[reservation] allow-local-fallback=true` in config.) The command then
prints `reservation reach degraded to local` and proceeds. Read/query commands are
unaffected — only id-minting needs it.

See [[mem.signpost.project.orientation]] for the jail layout.

## Correction (2026-09-25) — the premise is gone; do not set this variable

Recorded 2026-06-24, when the reservation reach was `auto` and tried the remote
first. False since **2026-07-03**, when `.doctrine/doctrine.toml` gained:

    [reservation]
    allow-local-fallback = true
    reach = "local"

With `reach = "local"` the backend is LocalFs outright — no remote contact, no
`git-ssh-disabled` shim, no fallback decision. The workaround above is therefore
**unnecessary**, and the variable is no longer exported: `flake.nix` dropped it
from the jail env on 2026-09-25 (commit `67df42ec8`).

Verified 2026-09-25 from inside the jail with the variable unset: an
id-reserving verb (`doctrine backlog new chore …`) succeeded, and
`cargo test --bin doctrine reserve::tests` was 19/19 green.

If the quoted error ever appears, the project config has been changed back to a
remote-reaching mode (`reach = "auto"`/`"shared"` with a configured but
unreachable `origin`). Fix the **config** (set `reach = "local"`, or
`allow-local-fallback = true`) rather than reintroducing the env export — and
note that setting the variable reddens `reserve::tests::vt3_auto_degradation…`,
which asserts the fail-closed default (`ISS-483`).
