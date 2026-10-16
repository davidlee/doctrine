`Path`/`PathBuf` equality compares components, so `"/repo/"` and `"/repo"` are equal.
A test that asserts a path value will pass even when the code produces a trailing
separator that later shows up in user-facing text.

Seen in SL-282: `PathBuf::join("")` (an empty `git rev-parse --show-prefix`) left a
trailing `/`; the PHASE-01 unit rows compared `PathBuf`s and missed it, and the
advisory printed `… in /tmp/x/repo/`. Caught only by an e2e text assertion in
PHASE-02.

How to apply: when a path feeds rendered output, assert on `as_os_str()` (or the
rendered string), and avoid joining a possibly-empty component — re-collect
components or skip the join.
