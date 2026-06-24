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
