# Corpus dir-walks must skip the NNN-slug symlink alias beside each numeric entity dir

Doctrine seats a `NNN-slug -> NNN` symlink beside the numeric canonical dir for
**every** entity kind (slice / adr / backlog / spec / memory). So any impure
corpus walk of the form `read_dir(.doctrine/<kind>/*)` that reads a file under
each child visits the same file **twice** — once via `NNN/`, once via the
`NNN-slug` alias — and silently doubles every record it collects.

Caught as ISS-006: `coverage_scan::collect_matching_entries` walked
`.doctrine/slice/*/coverage.toml` and yielded each coverage entry twice; the
reconcile writer masked it with `distinct_keys`, but the raw scan over-counted
for every other consumer (close-gate reader, RSK-006 perf reasoning).

**Fix at the walk, not the caller:** skip symlinked dir entries —
`if entry.file_type().is_ok_and(|t| t.is_symlink()) { continue; }`. The numeric
canonical dir is never a symlink. `DirEntry::file_type` does not follow the link
(lstat semantics on unix), so it classifies the alias correctly without a
`canonicalize` round-trip.

**Immune by construction:** reads that join the concrete `format!("{id:03}")`
path (e.g. `slice_local_covered_reqs`) never see the alias — only `read_dir`
enumeration does. Caller-side dedupe (`distinct_keys`) is a symptom mask and may
still be wanted to guard genuine authoring dups, but it does not fix the scan.
