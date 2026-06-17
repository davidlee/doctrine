# Use jail-built binary for doctrine claude install and boot after plugin edits

## The trap

`plugins/doctrine/skills/**` and `install/routing-process.md` are RustEmbed-compiled
into the binary. `doctrine claude install` and `doctrine boot` read the
**currently-running binary's** embedded assets — NOT the on-disk files.

Three binaries exist, only one is current:

| Binary | Location | Has latest plugins? |
|---|---|---|
| AUR release (PATH) | `~/.cargo/bin/doctrine` | **No** — stale from last release |
| Jail build | `/home/david/.cargo/doctrine-target-jail/debug/doctrine` | **Yes** — after `cargo build` |
| `./target/debug/doctrine` | repo-local symlink | **Stale** — CARGO_TARGET_DIR redirects elsewhere |

The trap: `cargo build` prints `Finished` (the crate recompiled), but the PATH
binary and `./target/` are untouched. Running `doctrine boot` or `doctrine claude
install` from PATH produces **stale output** — the boot snapshot shows old routing
rows, and `claude install` refreshes skills from the old embedded master.

## The fix

After editing embedded assets (`plugins/doctrine/skills/**` or `install/`):

```bash
# Force recompile (RustEmbed has no rerun-if-changed)
touch src/install.rs && cargo build

# THEN use the jail binary
export TARGET_DIR=$(cargo metadata --format-version=1 | jq -r '.target_directory')
$TARGET_DIR/debug/doctrine boot
$TARGET_DIR/debug/doctrine claude install
```

Related: [[mem_019e98a783ea7471ac4bfcefdc04ae5e]] — the RustEmbed recompile
footgun (touch src/install.rs). This memory adds the **which binary** dimension:
recompile matters, but so does selecting the right binary afterward.
