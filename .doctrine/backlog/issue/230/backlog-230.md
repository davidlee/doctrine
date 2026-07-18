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

## CORRECTION (2026-07-19) — original root cause above is WRONG

The diagnosis above (`flake.nix:113` tilde bind) is falsified. Investigation
(`/rigour`) reproduced the actual mechanism; the real cause is in a different
subsystem entirely.

**Actual root cause:** `cargo` runs with `CARGO_HOME` set to a **literal,
unexpanded `~/.cargo`** in the host shell. Cargo does not tilde-expand
`CARGO_HOME`, so it resolves relative to cwd and creates `./~/.cargo/registry/…`
in the repo root. Trigger is any cargo invocation in the repo — observed via
`just lint` → `cargo clippy` → `Updating crates.io index`.

The literal tilde came from a recent **nushell** config change: `~` only expands
in nushell in command-position bare paths, **not** inside a string literal
assigned to an env var (`$env.CARGO_HOME = "~/.cargo"` stores the literal `~`).
Fixed host-side by expanding the tilde (`"~/.cargo" | path expand`). Not in any
tracked repo config (`.envrc`, justfiles, flake devshell all clean).

**Why the filed diagnosis was wrong:**
- bwrap resolves a *relative* bind dest against the **new root `/`** (→ ephemeral
  `/~`), NOT the jail cwd — reproduced directly, and already documented at
  `scripts/pi-spawn-confined.sh:34-35`. It cannot produce a cwd-relative,
  host-persistent `/workspace/doctrine/~`.
- The observed path had a `.cargo/registry` cargo cache under it — a
  `CARGO_HOME` fingerprint, not a doctrine-bin mount.
- Symptom is jail-independent: reproduces on the bare host and under codex's own
  sandbox, never in the doctrine jail (where `CARGO_HOME` resolves correctly).

**Collateral:** commit `7021221f5 "fix: flake path"` (tilde→absolute in
`flake.nix:113`) chased this phantom; it is harmless (absolute dst is equally
correct) but fixed nothing here. `noescape` renders the dst raw/unquoted into the
launcher shell, so the original tilde *did* expand — the comment was right and the
diagnosis inverted it. Left as-is; no revert needed.

**Resolution:** fixed in host nushell env; no doctrine code change required.

## Links

- IMP-249 (resolved) — introduced the bind line; the atomic-rename stale-bind
  fix this rides on.
- Superseded diagnosis: the "Root cause"/"Fix"/"Validation" sections above are
  retained as the audit trail of the misdiagnosis.
