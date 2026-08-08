`boot::resolve_exec()` is `std::env::current_exe()`. Under `cargo test` that
resolves to `target/debug/deps/doctrine-<hash>`, whose `file_name()` is
`doctrine-<hash>` — **not** `doctrine`.

`is_doctrine_program` owns a command only if the program half is the portable
literal `${DOCTRINE_BIN:-doctrine}` or a path whose file name is exactly
`doctrine`. So a hook entry written by driving a REAL installer inside a test
(`run_sync_install`, `boot install`) is **not owned by doctrine's own
predicate**.

## What this silently breaks

A fixture seeded by running the installer and then asserted against any
ownership-dependent behaviour — idempotency, refresh-in-place, the SL-250
abandoned-scope sweep — reads as a clean pass while actually exercising the
foreign-entry branch. The sweep finds nothing to evict; a re-install appends a
duplicate instead of refreshing. Nothing fails.

It bit SL-250 PHASE-03: `memory_sync_install_reports_scope_and_eviction` seeded
`.claude/settings.local.json` by running `run_sync_install` at local scope, then
asserted the eviction. The eviction correctly found nothing.

## What to do

Seed ownership-dependent fixtures **literally**, with a `doctrine`-named program
half:

```rust
r#"{"hooks":{"SessionStart":[{"matcher":"startup","hooks":[
     {"type":"command","command":"/abs/doctrine memory sync"}]}]}}"#
```

Unit tests that construct a `HookSpec` directly are already safe — they pass
`Path::new("/abs/doctrine")`. The hazard is specific to tests that go through
`resolve_exec()`.

Related: [[mem.fact.claude.settings-hooks-merge-and-matcher]].
