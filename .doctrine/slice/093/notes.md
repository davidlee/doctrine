# Notes SL-093: CSS modularisation: split monolithic style.css and resolve RV-065 findings

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 completed (2026-06-18)

- 10 CSS files created with @layer boundaries, all verification gates pass
- Build: `npx vite build` passes (bun unavailable in jail; npm scripts use `bun run`)
- `color-mix(in srgb, var(--cm-primary) 20%, transparent)` used for box-shadow
  replacement on `.cm-rename-input:focus` — slight behaviour change from
  `rgba(22, 160, 133, 0.2)` but functionally equivalent
- CM diagnostics dark overrides moved into concept-map.css `@media` block within
  `@layer components`, consuming `--cm-*` tokens (dark variants resolve through cascade)
- `.table-toggle` rules placed in sidebar.css (not table.css — sidebar-scoped)
- `.view-toggle`/`.view-btn` rules placed in priority.css (not table.css — nearest
  to the view they control)
- Dispatch funnel: import from worker `6fdc463d` onto coordination `dispatch/093`
  at `ad6a97ba` (base `ce39c862` → `ad6a97ba`). Fork branch `worker/SL-093/PHASE-01`
  is the native phase unit.
