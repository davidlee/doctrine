# Run doctrine install and boot from the freshly built in-tree binary, not PATH

> **Key is a misnomer.** `…jail-binary-for-skill-install` dates from the shared
> `CARGO_TARGET_DIR` era; that redirect is gone (SL-156 / ADR-008 D-B1). Memory
> keys are immutable once recorded, so the key stays — read it as
> *"fresh-binary-for-skill-install"*. Corrected 2026-07-25 against live evidence.

## The trap (unchanged, and the reason this memory exists)

`plugins/**` and `install/**` are RustEmbed-compiled into the binary. `doctrine
install` and `doctrine boot` read the **currently-running binary's** embedded
assets — NOT the on-disk files. So *which binary you invoke* is load-bearing,
and `cargo build` printing `Finished` proves nothing about the binary on `PATH`.

## Which binary is current

| Binary | Location | Has latest embedded assets? |
|---|---|---|
| release / PATH | `~/.cargo/bin/doctrine` | **No** — stale from last release; read-only in the jail |
| **in-tree build** | `./target/debug/doctrine` | **Yes** — after `touch src/install.rs && cargo build` |
| old jail target | `~/.cargo/doctrine-target-jail/debug/doctrine` | **Leftover — ignore.** Pre-SL-156 path, no longer written |

**Corrected claim.** This memory previously said `./target/debug/doctrine` was a
symlink made stale by a `CARGO_TARGET_DIR` redirect, and prescribed the jail
path. That is now exactly inverted. Verified in-jail 2026-07-25:
`CARGO_TARGET_DIR` unset; `cargo metadata` → `target_directory:
/workspace/doctrine/target`; no `target-dir` in any `.cargo/config.toml`;
`./target/debug/doctrine` a real 250MB executable freshly built; the jail path
last written weeks earlier. Each worktree builds into its own gitignored in-tree
`target/` (SL-156, ADR-008 D-B1) — cargo's default, no redirect.

## The consequence when you get it wrong

Not just stale skills — **`boot` regen from a stale-embed binary silently rolls
back governance**, and `boot --check` self-reports clean. Observed 2026-07-25:
the boot snapshot's command spine had lost `slice research`, `explore graph`,
*and* SL-227's `library` verb, because something ran `boot` from an older
binary. Regen from `./target/debug/doctrine` restored all three. The snapshot is
gitignored runtime state, so it is cheap to repair — but until someone notices,
every session boots with degraded governance context.
See [[mem.fact.doctrine.boot-regen-binary-embed-divergence]].

## The fix

After editing embedded assets (`plugins/**` or `install/**`):

```bash
touch src/install.rs && cargo build   # RustEmbed has no rerun-if-changed
./target/debug/doctrine boot
./target/debug/doctrine install -s <id> -y
```

Prove the re-embed took by grepping the built binary — note `-a`, or grep
suppresses output on a binary and the empty result reads as a miss:

```bash
grep -a -c "<a distinctive string you just added>" target/debug/doctrine
```

If you are somewhere the in-tree path may not hold, resolve it rather than
hardcoding either path — this form is correct in both eras:

```bash
TARGET_DIR=$(cargo metadata --format-version=1 | jq -r '.target_directory')
```

Related: [[mem.pattern.build.rust-embed-no-rerun]] — the recompile footgun this
one builds on (recompiling matters; so does then picking the right binary).
[[mem.pattern.distribution.skill-refresh-command]] — the skill-refresh sequence.
