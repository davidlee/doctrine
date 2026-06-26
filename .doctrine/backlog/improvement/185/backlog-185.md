# IMP-185: CI: constructor test for doctrine_bin() resolver

Scoped from RV-169 F-2 (Inquisition on SL-162 design).

Add a test that verifies `test_support::doctrine_bin()` returns a path
that exists and is executable in the current namespace. This serves as
a CI-runnable regression guard for the A1 layout assumption (`doctrine`
at `<target>/<profile>/doctrine`, sibling of the test exe's `deps/`
parent).

Currently VH-1 is the only cross-namespace proof; a constructor test adds
automated defence-in-depth. Should run under `cargo test` in any namespace
and fail fast if the layout assumption breaks.

Refs: SL-162, RV-169
