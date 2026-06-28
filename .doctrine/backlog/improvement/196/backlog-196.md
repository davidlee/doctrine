# IMP-196: Golden hermeticity lint — flag goldens reading the live corpus

**Source:** SL-168 postmortem §5d.3 (root of F-2). **Home:** RFC-005.

Workers capture byte-exact golden output by running the bin against the live
project root; that output carries volatile fields (e.g. `commits_behind HEAD`), so
the golden breaks on any later commit. Workers lack hermetic-fixture intuition.

**Fix direction:** a clippy-like lint flagging golden tests that invoke
`CARGO_BIN_EXE_doctrine` against the live root without a hermetic fixture path
override. Static counterpart to IMP-195's runtime regen.

Related: RFC-005; IMP-195.
