# REQ-318: Run-ledger object-db sourcing

## Statement

The run ledger — `journal.toml`, `boundaries.toml`, `orthogonal.toml` under
`.doctrine/dispatch/<N>/` — is **tree-read from the `dispatch/<N>` branch tip**
(`read_path_at` against the object db), never the working filesystem, and identically in
stage-1 and stage-2. A ledger file that is not committed to the dispatch branch reads as
absent (e.g. an uncommitted `boundaries.toml` yields zero phase cuts — ISS-039).

## Rationale

Reading from the object db rather than a checkout makes the ledger value
checkout-independent: audit can run from the root while the coordination tree is
elsewhere, and stage-1 and stage-2 observe the same bytes. The cost — that an
uncommitted ledger file is invisible — is an operational constraint the funnel must
honour, not a model defect.
