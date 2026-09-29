# ISS-506: prose_cite doctor leg silently skips degraded reads

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctor_checks::prose_cite_findings` (`src/doctor_checks.rs:266`) returns an
empty finding list when its glob pattern is not UTF-8 or cannot be built, and
`continue`s past a matched file it cannot read. Doctor then reports the prose
check clean over a corpus it did not fully read — STD-003 (no silent skip).

SL-273 built its sibling leg `lib_citation_findings` to disclose all three
cases (could-not-run, glob pattern, per-file read with the walk continuing);
tests `lib_citation_unreadable_file_disclosed_sibling_reported` and
`lib_citation_manifest_not_admitted_disclosed` are the model. The regex
compile failures are constant patterns and can stay as they are, or become
`expect`s. Related: ISS-446 (same defect class in `lifecycle_findings`).
Surfaced by SL-273 design §4.3; captured at its audit (RV-414).
