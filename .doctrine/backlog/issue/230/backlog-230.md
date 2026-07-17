# ISS-230: Jail bind dst tilde is a literal relative path — spawns /workspace/doctrine/~ every jail cycle

## Symptom

A stray `~` directory (`/workspace/doctrine/~/.cargo/bin/doctrine`) keeps
reappearing in the repo root. Deleting it is futile — every nested jail/worker
spawn (dispatch / pi-spawn) recreates it.

## Root cause

`flake.nix:113`:

```nix
(ro-bind "${doctrine}/bin/doctrine" (noescape "~/.cargo/bin/doctrine"))
```

The adjacent comment claims *"dst tilde expands in the host launcher shell"* —
it does not. The `~` is inside a quoted string wrapped in `noescape`, so the
launcher shell never sees a bare unquoted `~` and performs no tilde expansion.
bwrap receives the **literal, relative** path `~/.cargo/bin/doctrine` as the
mount destination.

When bwrap must create a missing mountpoint and the destination is relative
(doesn't start with `/`), it creates it relative to the jail cwd — the repo root
`/workspace/doctrine`. So it materialises `/workspace/doctrine/~/.cargo/bin/doctrine`,
and because the repo root is a bound-in **rw host directory**, the `~` dir
survives on the host after the jail exits.

This is a new symptom introduced by the IMP-249 fix (which added this bind line);
IMP-249 itself (stale-bind hook/MCP breakage) is resolved and unaffected.

## Fix

Use the absolute path the comment already hardcodes two lines below in
`add-path "/home/david/.cargo/bin"`:

```nix
(ro-bind "${doctrine}/bin/doctrine" (noescape "/home/david/.cargo/bin/doctrine"))
```

An absolute destination gives bwrap a non-relative mountpoint, so no `~` dir is
ever created. Update or drop the now-inaccurate "tilde expands in the host
launcher shell" comment while there.

## Validation

`flake.nix` changes take effect only on the next jail cycle and can be validated
only on the **host** (nix is absent in-jail): rebuild the jail, spawn a worker,
confirm no `/workspace/doctrine/~` appears.

## Links

- IMP-249 (resolved) — introduced the bind line; the atomic-rename stale-bind
  fix this rides on.
