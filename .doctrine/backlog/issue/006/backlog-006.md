# ISS-006: scan_coverage double-walks slice dir + slug symlink, over-counting coverage keys

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced during SL-044 B·P2 (the reconcile writer). `coverage_scan::scan_coverage`
walks a slice's coverage twice — once via the numeric dir (`slice/NNN/`) and once
via the slug-alias symlink — so a single coverage cell's `CoverageKey` is returned
twice. The reconcile writer worked around it with a `distinct_keys` dedup (a REC
must not cite the same 4-tuple twice), but the over-count is latent in the SL-042 P3
scan itself and will mislead any future consumer that trusts the multiplicity (e.g.
the B·P3 closure-gate coverage reader, or RSK-006 perf reasoning).

Origin: SL-042 PHASE-03 (`scan_coverage`). Fix belongs in the scan (skip symlinked
alias dirs, or canonicalise + dedupe at the walk), not in every caller. Until then,
callers must dedupe keys themselves.

Related: SL-044, RSK-006 (coverage scan/staleness perf).
