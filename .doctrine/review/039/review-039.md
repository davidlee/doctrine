# Review RV-039 — reconciliation of SL-073

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-closure code-review fix audit. A `/code-review` on SL-073 (2026-06-16)
produced 18 findings across four severity levels. 8 high-severity findings
(🔴 blocking + 🟠 important) were addressed in three fix commits:
`e21ccfd`, `bd1ab75`, `b030cd4`. This audit verifies each fix is correctly
applied and dispositioned.

Lines of attack:
1. Security pipeline correctness (stale comment, escapeHtml completeness, focus header escaping)
2. Data integrity (BFS edge dedup, depth clamping in URL + state)
3. Refresh pipeline (api.refreshGraph → wireRefresh chain)
4. UI correctness (kind pill contrast, SVG double-navigation)
5. JS syntax and gate (node --check, just check)

## Synthesis

All 8 findings verified and dispositioned `fix-now`. Gate green: `just check`
1471 passed, zero failures; all JS files pass `node --check`.

**Fixes applied (3 commits):**
- `e21ccfd` — kind pill contrast (style.css attribute selectors for light-fill kinds)
- `bd1ab75` — stale comment, escapeHtml completeness, focus header escaping,
  refresh pipeline chaining, depth clamp in render(), bootstrap double-render fix,
  edge view null-focusId guard
- `b030cd4` — api.refreshGraph added to api.js, BFS edge dedup in model.js,
  parseHash depth clamp in router.js, DOT URL attribute removed from dot.js

**Standing: 10 minor/optional findings deferred.** These were consciously
accepted as `tolerated` — they do not affect correctness or the Hard Contracts:
- 🟡 9. Direct edge URL without prior focus redirects (edge case, rare)
- 🟡 10. render() double-invocation on bootstrap (already fixed by #6 in bd1ab75)
- 🟡 11. Depth button active state not synced on URL navigation (cosmetic)
- 🟡 12. Markdown link-policy stripping loses child formatting (no affected content today)
- 🟡 13. Edge detail not-found silently fails on null container (DOM stable)
- 🟡 14. Edge view keeps focus-header from last focus (design ambiguity, not a bug)
- 🔵 15-18. Code hygiene suggestions (DOM coupling, dead code, redundant fallback, looseCanonical comment)

**No unresolved blockers.** The slice was already closed; these are post-closure
quality improvements. All 8 Hard Contracts remain satisfied.
