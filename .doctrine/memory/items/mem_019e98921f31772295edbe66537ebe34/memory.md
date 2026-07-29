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

## Stronger option when the marker set is a parameter (ISS-281, 2026-07-29)

`root::find_from(start, markers)` takes its markers as an **argument**, so a unit
test can pass a bespoke name (`.iss281-test-marker`) that no real ancestor
carries. That removes the ancestry assumption outright — no candidate scan, no
`marker_free_base`, cannot flake. Prefer it for anything calling `find_from`
directly; see `src/root.rs::tests`.

`marker_free_base` is still the right tool when the marker set is NOT yours to
choose — `root::find` with `default_markers()`, or an end-to-end binary run. Note
it lives in `tests/common/mod.rs`, so it is unavailable to `src/` unit tests.

Third option, cheapest where it fits: pick an anchor that fails
`fs::canonicalize` (a nonexistent path). Discovery folds to `None` before the
walk is reached, so ancestry is irrelevant. This is what
`memory::ambient_surface_tests::vt9_no_discoverable_root_emits_nothing` now does.

**The hazard recurred.** `/tmp/.git` was present again on the dev host
2026-07-24..29 and silently broke a "rootless tempdir" premise — vt9 passed on
`/tmp` having no memory corpus rather than on there being no root. A test whose
premise depends on TMPDIR ancestry should **assert the premise**, not just the
consequence, or the failure reads as a pass. See ISS-281.
