# Manual verification checklist — SL-083

Run: 2026-06-17. All 16 items confirmed pass.

| # | Item | Result | Notes |
|---|------|--------|-------|
| 1 | Load — sidebar, focus, graph | ✅ | |
| 2 | Click node — focus change, hash, highlight | ✅ | |
| 3 | Search "SL" + Enter | ✅ | |
| 4 | Arrow-down keyboard nav + Enter | ✅ | |
| 5 | Kind filter toggle | ✅ | |
| 6 | Depth buttons | ✅ | |
| 7 | Refresh button | ✅ | |
| 8 | CM entity click — CM diagram, edge table, diagnostics | ✅ | |
| 9 | Toggle CM edit mode | ✅ | |
| 10 | Add/remove CM edge — graph updates (fixed in 24a3169) | ✅ | Re-render bug found & fixed during verification |
| 11 | Rename CM node | ✅ | |
| 12 | Dark mode toggle | ✅ | |
| 13 | Fullscreen markdown | ✅ | |
| 14 | Edge detail view — click edge label | ✅ | |
| 15 | Hide relations table — checkbox + localStorage persist | ✅ | |
| 16 | Entity graph → CM → entity graph — no stale panels | ✅ | |
