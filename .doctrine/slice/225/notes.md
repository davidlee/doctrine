# Notes SL-225: Funnel false-red elimination

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## 2026-07-24 — DEC-003 design inquisition

RV-291 tried DEC-003's zero-engine reversal and sustained three findings:

- F-1 blocker: `DOCTRINE_BIN` is resolved when the MCP server loads, but the
  coord build it must name is normally created later by `dispatch setup`; the
  design needs an explicit restart/rebind lifecycle or a distinct stable
  project-local indirection before `/plan`.
- F-2 major: `design.md` still asserts the rejected `current_exe()` publication
  at lines 152-153.
- F-3 major: VT-1 proves recipe argv precedence with a stub, not elimination of
  ISS-218 through the actual `worker_commit` gate.

All findings are terminal with `design-wrong`; no follow-up or tolerated drift
was sanctioned. RV-291, this note, and the RFC-011 CLI-friction case-note form a
dedicated authored review commit. `just gate` passed after the review (existing
doctor warnings only).
