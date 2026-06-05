# root::find walks CWD to / — no-root tests need a marker-free temp base

`root::find(None, markers)` (src/root.rs) walks `current_dir()` up to `/` looking
for any marker (`.git`/`.jj`/`.project`/`Cargo.toml`). A stray marker in an
ancestor of the system tempdir — e.g. a leftover `/tmp/.git` — makes it resolve a
root even from a "bare" `tempfile::tempdir()`, so a test meant to exercise the
**no-root** path (e.g. SL-018 `memory sync` Charge XI no-op) instead hits an
incidental empty-repo no-op and the assertion mis-fires.

**Fix:** create the no-root tempdir under a base whose ancestry to `/` is
marker-free. Scan candidates (`/dev/shm`, `/var/tmp`, `std::env::temp_dir()`) and
pick the first clean one with `tempfile::Builder::tempdir_in`; panic loudly if
none is clean (a missed assertion is worse than a failed test). See
`tests/e2e_memory_sync.rs::marker_free_base`.

Conversely, tests that need a root just `create_dir(dir/".git")` to plant a marker.
