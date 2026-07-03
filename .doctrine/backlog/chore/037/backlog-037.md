# CHR-037: SL-195 live acceptance: run deferred VH-1 legs (--dev install, /mcp env-expand, repo-move refresh)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Harvested from the SL-195 audit (RV-243 F-1, `tolerated`). All three phases
shipped with green test-mode (`VT`) coverage; the human-mode (`VH`) legs were
deferred — no code depends on them and every mechanical sub-leg is confirmed, but
the interactive end-to-end runs need a live `claude` session (and a physical repo
move) not safely performed in the primary `edge` worktree during audit.

**Run on a machine with live CC ≥ 2.1.198:**

1. **P01 VH-1 / OQ-4** — hand-write a `.mcp.json` whose `command` is
   `${DOCTRINE_BIN:-doctrine}`; confirm it connects under `/mcp` (env-expansion
   at load, mcp.md:384).
2. **P02 VH-1** — `doctrine install --dev` on this repo → live plugin load, zero
   network; `git status` clean of any abspath; `claude plugin marketplace list`
   shows `Source: Directory (<abs root>)`.
3. **P03 VH-1** — move/relink the repo dir, then `install --dev` again → the
   registered source updates to the new abspath with **no duplicate** `doctrine`
   marketplace entry (INV-2/INV-3).

Close this chore once all three are observed green. Ref:
`mem.fact.claude.marketplace-add-overwrites-source` (refresh-verb probe).
