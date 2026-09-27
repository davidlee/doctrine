# Notes SL-269: Review-capable worktree locus

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-27 · design:inquiring (drafting-ready) · 50bf9a9cf

### Produced
- re-scoped as RFC-032 slice 4 (D9 → D7) — the declared-locus approach overlapped D7's rejected home-tree option (commit 22dbfa6f8)
- ISS-484 fixed outside the slice: `review new` guards before allocating; MCP description corrected (120b8b321, 0e2c52208)
- design run: 11 inquiries disposed; scope/verification updated (cfaebf913)
- minted: ISS-496 — reseat dangler scan ignores `.toml` edges (split from inq-5)
- research: prompt staged at `research/prompt.md`; `raw/thread{1,2}-*.md` exist, not yet distilled into `research.md`; explore.research was discharged `skipped`
- no code yet; gate last green at 120b8b321 (`doctrine check commit`)

### Learned
- mem.pattern.review.mcp-bypasses-worker-guard — review admission must live in `resolve_review_root`
- mem_019eb741539075c380783b4cff747fec — updated: `review new` now refuses on a fork; SL-269 supersedes

### Open
- DEC-337 — clone-wide local reservation (ref CAS + live-worktree scan)
- DEC-338 — review admission refused only in a worker process
- ASM-012 — design-run mint relies on the CLI worker guard (no MCP design tools)
