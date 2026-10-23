# CHR-181: Classify harness link dirs in the worktree tier map

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The worktree tier map (`src/worktree/allowlist.rs` `WITHHELD` /
`DERIVED_RUNTIME`) classifies only `.doctrine/`-prefixed paths, and the
`.gitignore` parity test reads only `.doctrine/` lines. Harness link dirs that
install writes (`.claude/agents`, `.claude/skills`, `.pi/agents`,
`.agents/skills`) are classified nowhere, so a new framework-owned harness path
can land unclassified (`mem_01a141f2758873f094acf9f3fb42363b`).

Surfaced in SL-285 design (inq-6) after a pre-review noted the previously cited
CHR-043 is resolved and does not track this gap. Out of SL-285's scope: SL-285
regenerates these dirs via install convergence, which needs no classification.
