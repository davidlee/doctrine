# Stray ./~ in repo root = literal-tilde CARGO_HOME, not a bwrap/jail bind

**Symptom.** A `./~` directory keeps reappearing in the repo root, containing
`./~/.cargo/registry/…` (a cargo cache). Deleting it is futile — the next cargo
run recreates it.

**Root cause.** `cargo` was invoked with `CARGO_HOME` (or `$HOME`) set to a
**literal, unexpanded `~`**. Cargo does not tilde-expand `CARGO_HOME`, and
Rust's `create_dir_all` does not expand `~`, so `~/.cargo` resolves *relative to
cwd* → `./~/.cargo`. Triggered by any cargo command in the repo (`just lint` →
`cargo clippy` → `Updating crates.io index` is the usual one caught).

**How the literal tilde gets there.** Shell configs that quote the tilde:
`export CARGO_HOME="~/.cargo"` (bash/zsh) or `$env.CARGO_HOME = "~/.cargo"`
(**nushell** — `~` only expands in command-position bare paths, never inside a
string literal). Fix by expanding it: nushell `("~/.cargo" | path expand)` or
`($nu.home-path | path join ".cargo")`; POSIX `export CARGO_HOME="$HOME/.cargo"`.

**Diagnostic.** In the offending shell: `$env.CARGO_HOME?` — if it prints
`~/.cargo`, that's the bug. Nushell sweep: `$env | transpose k v | where v =~ '^~'`.

**Debugging trap (ISS-230).** This was mis-filed as a doctrine bwrap/jail bind
bug (`flake.nix` `noescape "~/.cargo/bin/doctrine"`). It is NOT. Discriminators
that rule out bwrap:
- bwrap resolves a *relative* bind dest against the **new root `/`** (→ ephemeral
  `/~`), not the jail cwd — so it cannot create a cwd-relative, host-persistent
  `repo/~`. (Documented at `scripts/pi-spawn-confined.sh:34-35`.)
- A `.cargo/registry` cache under the `~` is a `CARGO_HOME` fingerprint, not a
  mount point.
- The symptom is **jail-independent**: reproduces on the bare host and under
  codex's own sandbox, never in a jail where `CARGO_HOME` resolves correctly.

General lesson: when a stray path contains a literal `~`, suspect a **quoted
tilde in an env-var assignment**, not the sandbox/mount layer. Match the *cache
contents* to the tool (here `.cargo/registry` → cargo → `CARGO_HOME`) before
blaming bind machinery. See [[mem_019ed423fc7a708181e3506c56d09326]]
(CARGO_TARGET_DIR in the jail) for the adjacent cargo-env territory.
