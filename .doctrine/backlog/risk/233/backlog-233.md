# RSK-233: Clone-ref reservation in a tempdir test writes refs into an ambient git repo

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Since SL-269 (DEC-337), `local` reach on a git root claims ids with a ref
compare-and-swap (CAS) in the clone's common git dir (`CloneRef`). A test that
builds its project in a bare `tempfile::tempdir()` expects "not git". If
`TMPDIR` sits inside a git work tree (the ISS-281 precedent), the backend
detects that outer repo instead: it writes `refs/doctrine/reservation-local/*`
into it and scans its worktrees. `resolve_remote` has the same exposure. This
widens the class to every fresh-id e2e test and to `reseat`'s goldens.

Source: SL-269 PHASE-02 and PHASE-03 phase-sheet risks; RV-409 synthesis.
Related: ISS-468 (fixtures not hermetic against an inherited `GIT_DIR`).
Fix direction: fixtures pin `GIT_CEILING_DIRECTORIES`, or assert "outside git"
before relying on it.
