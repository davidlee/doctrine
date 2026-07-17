# ISS-225: prepare-review ledger commit clobbers object-db conclude rows from a stale coord working tree

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at SL-220 PHASE-07 conclude (2026-07-17). Sequence:

1. `dispatch_conclude_phase` lands the boundary row **working-tree-free**
   (object-db compose; the ref `dispatch/220` advances fb49eaf2 → 2c055d29;
   the coord checkout is untouched — by design, SL-199).
2. `dispatch sync --prepare-review` opens by committing the boundaries
   ledger **from the coord working tree** — which is still the pre-conclude
   checkout. That commit (ac2f1e33, "ledger: boundaries") re-committed the
   stale file, **deleting the PHASE-07 row conclude had just landed**
   (-6 lines).
3. prepare-review's own completeness gate then failed on the gap it
   created: "completed phase PHASE-07 has no recorded source-delta row" —
   a self-inflicted halt with a misleading signature (the row *was*
   recorded, two commits earlier).

Recovery: `git restore --source=2c055d29 -- .doctrine/dispatch/220/
boundaries.toml` + restore commit fc58eb60 on `dispatch/220`, re-run. The
same staleness also left journal.toml as a staged deletion in the coord
tree, which blocked `git worktree remove` until restored.

Root cause: a write/read seam split — funnel MCP verbs write the ledger
via the object db, prepare-review reads (and commits) it via the working
tree. Any conclude not followed by a manual working-tree sync poisons the
next prepare-review. Today's guard is pure operator ritual
(`git restore --source=HEAD --staged --worktree -- <ledger paths>` after
every object-db write), documented nowhere the funnel enforces.

Candidate fixes (either kills the class):
- prepare-review sources its ledger commit from the **ref**, not the
  working tree (only commit a working-tree ledger that is strictly newer,
  or refuse on divergence) — localised in dispatch.rs around the
  ledger-commit step;
- or `dispatch_conclude_phase` (and every object-db ledger writer)
  checkout-syncs the coord working tree for the paths it wrote, closing
  the gap at the source.

Related, distinct: ISS-212 / IMP-272 (phase-completion flip read from the
primary tree — the *other* stale-state halt in the same beat); ISS-039
(closed — ledger never committed at all); ISS-224 (conclude oids stored
verbatim). Detailed narrative: RFC-011 case-notes entries
`[dispatch; SL-220 PHASE-07 conclude]` (edge 3eeb449a).
