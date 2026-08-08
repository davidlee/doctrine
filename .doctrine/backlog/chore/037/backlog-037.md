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

## Overlap assessment vs SL-250 (2026-08-08, SL-250 close)

SL-250 `sec-6` owed this assessment and never recorded it. **Two of the three legs
are mooted; leg 1 survives intact.**

SL-250 removed the automated plugin steps from `doctrine install` (verified by
grep at close), so the marketplace registration these legs were written to observe
no longer happens on the automated path:

1. **P01 `VH-1` / `OQ-4` — SURVIVES.** A hand-written `.mcp.json` whose `command`
   is `${DOCTRINE_BIN:-doctrine}` connecting under `/mcp` is untouched by SL-250,
   and is in fact reinforced by it: the same `PORTABLE_EXEC` literal is now the
   single source across both `.mcp.json` and the hook commands under
   `CommandForm::Portable`, and SL-250's own `VH-1` cold install exercised it on
   the hooks surface. Still worth observing on a live session; still the only
   unobserved half.
2. **P02 `VH-1` — MOOT.** `doctrine install --dev` → live plugin load and
   `claude plugin marketplace list` showing `Source: Directory (<abs root>)`
   cannot be run: the install verb no longer registers a marketplace.
3. **P03 `VH-1` — MOOT.** The repo-move / re-register no-duplicate check
   (`INV-2` / `INV-3`) tests the same removed code path.

**Disposition:** left open on leg 1 alone. Do not close on the strength of legs
2–3 being unrunnable — that is mooting, not observation, and the distinction is
the point of a `VH` leg. Whoever runs leg 1 can close this.

Related: [[IMP-234]] carries the same SL-250 reduction on the design side.
