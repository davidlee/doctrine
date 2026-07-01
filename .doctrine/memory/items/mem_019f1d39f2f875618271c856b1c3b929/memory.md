# Reinstalling ~/.cargo/bin/doctrine on macOS needs an ad-hoc re-sign or the hook binary SIGKILLs (exit 137)

cp a fresh cargo build over ~/.cargo/bin/doctrine on macOS invalidates the code signature → kernel kills every invocation with exit 137; codesign --force --sign - fixes it.

**Symptom.** After `cp target/debug/doctrine ~/.cargo/bin/doctrine`, EVERY
invocation of the copied binary dies with exit 137 (128+9 = SIGKILL) — even
`--version`. `target/debug/doctrine` (run in place) works fine; only the copy is
killed. Not a sandbox artifact (reproduces with the harness sandbox disabled).

**Cause.** macOS applies an ad-hoc code signature to the cargo-built Mach-O in
place. Overwriting the existing signed `~/.cargo/bin/doctrine` with a raw `cp`
leaves a binary whose signature no longer matches its bytes; the kernel's code-
signing enforcement kills it on exec.

**Fix.** Re-sign the copied binary ad-hoc:
```
cp target/debug/doctrine ~/.cargo/bin/doctrine
codesign --force --sign - ~/.cargo/bin/doctrine
```
(Or `cargo install --path .`, which re-signs. `codesign` also rewrites the
Mach-O, so the copy ends up a slightly different size than the source — expected.)

**Why it matters here.** The claude PreToolUse hook
(`.claude/skills/doctrine/hooks/hooks.json`) invokes the absolute
`~/.cargo/bin/doctrine worktree pretooluse`. An unsigned/mismatched copy means the
hook exits 137 on every tool call → `|| exit 2` blocks the tool, or the hook is
silently ineffective — either way the live jail path is not exercised. Any
SL-183-style live in-situ run that reinstalls the hook binary must re-sign after
the copy. See [[mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement]].
