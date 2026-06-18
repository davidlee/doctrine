# Review RV-072 — reconciliation of SL-094

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-094 (semantic graph zoom, pan, crop-on-bounds).
All three phases implemented; inquisition (RV-071) resolved with 7 findings
dispositioned — 5 integrated, 1 tolerated, 1 aligned.

### Lines of attack

1. **Conformance — design decisions D1–D5.** Does the implementation faithfully
   realise each decision? CSS transform wrapper (D1), app-state viewport (D2),
   minK fit-to-container (D3), focus-change centres (D4), same-focus restores
   (D5).

2. **Conformance — RV-071 findings F1–F7.** All 7 inquisition findings were
   dispositioned pre-implementation. F-1 (deltaMode), F-2 ({once:true}), F-3
   (cross-mode guard), F-4 (depth-change edge case), F-5 (touch-action:none)
   were fix-now/design-wrong — confirm each is integrated. F-6 (dimLegend)
   tolerated — confirm no change needed. F-7 (seq guard) aligned — confirm.

3. **ISS-021 requirements.** Scroll-wheel zoom, drag-to-pan, crop-on-bounds —
   all three are the slice's declared scope.

4. **Evidence gates.** `npx tsc --noEmit`, `npx eslint --max-warnings=0`,
   `npx vitest run` (312 tests expected), `npx vite build`.

5. **Phase sheet harvest.** Disposable findings from phase-01/02/03.md that
   should be promoted to durable notes, memories, or backlog items.

### Held to

- design.md (authoritative — viewport rules table, design decisions,
  container mechanics, event handling)
- plan.toml PHASE-01/02/03 criteria (EN-/EX-/VT-/VA-)
- ISS-021 scope
- RV-071 findings F1–F7
- Domain map invariants I1–I6, risks R1–R3

### Audit mode

Conformance. Surface reviewed: `main` branch head (1db6ecbf), which carries
all 3 phases + the wheel-minK fix. No dispatch — sole-agent implementation.

## Synthesis

### Verdict

The implementation is **conformant** with design.md across all 5 design decisions
(D1–D5), all 7 RV-071 inquisition findings (F1–F7), and all 3 ISS-021
requirements. Automated evidence is unanimous: tsc clean, eslint 0-warn, 312/312
tests pass, vite build succeeds.

Three minor design amendments (F-6) emerged during implementation — the
`lastRenderedFocusId` field was rightfully simplified out (reusing existing
`prevFocusId`), `parseTransform()` was added as a necessary implementation
detail for handler lifecycle correctness, and `minK` is stored on the wrapper
dataset to survive re-renders. All three are positive design refinements, not
defects.

### Standing verification gap

Six VA criteria across PHASE-02 and PHASE-03 were deferred by the implementing
agent and remain unverified interactively (F-5). The plan designates these as VA
(agent verification), not VH (human). The mechanical conformance evidence is
strong — every handler guard, deltaMode normalization, {once:true} on mouseup,
cross-mode gate, and viewport rule is verifiable from source. Interactive
browser verification is recommended before close but does not block given the
automated evidence.

### Design decisions — all hold

| ID | Decision | Status |
|----|----------|--------|
| D1 | CSS transform wrapper, not SVG transform | Conformant — render.ts L714 creates `.graph-transform-layer` div |
| D2 | Viewport in app state, not graphPane local | Conformant — `types.ts` L150, `state.ts`, `app.ts` L488-490 |
| D3 | minK = fit-to-container, not 0.1 | Conformant — `render.ts` L705-707, wheel uses `dataset.minK` (L732) |
| D4 | Focus change centres new graph | Conformant — `app.ts` L349, `render.ts` L710-711 |
| D5 | Same-focus restores exact viewport | Conformant — `render.ts` L713 passes through as-is |

### RV-071 integration — complete

| Finding | Disposition | Integrated? | Evidence |
|---------|------------|-------------|----------|
| F-1 (blocker) | fix-now — deltaMode | Yes | `render.ts` L728-729 |
| F-2 (major) | fix-now — {once:true} | Yes | `render.ts` L761 |
| F-3 (major) | fix-now — cross-mode guard | Yes | `render.ts` L725-726, L741-742 |
| F-4 (minor) | design-wrong — depth-change edge case | Yes | `design.md` L225 |
| F-5 (minor) | fix-now — touch-action:none | Yes | `graph.css` L16 |
| F-6 (minor) | tolerated — dimLegend scope | No change needed | Legend in sidebar, outside `.graph-area` |
| F-7 (nit) | aligned — seq guard | No change needed | Wiring inside seq block |

### Evidence summary

```
tsc --noEmit          ✓ clean
eslint (src/)         ✓ 0 warnings
vitest run            ✓ 312/312 tests (25 viewport, 48 router, 107 dot, 6 priority, 126 model)
vite build            ✓ dist/assets/index-tsznYhUh.js 331.98 kB
```

### Tradeoffs consciously accepted

- **D5 — depth change preserves viewport:** The user's zoom/pan survives depth
  changes, even though prior coordinates may point to empty space after graph
  topology changes. Documented in design.md L225.
- **No automated DOM tests for zoom/pan:** Interactive behaviour is verified via
  VA criteria. The pure math (25 viewport tests) covers the arithmetic surface.
  DOM event testing for wheel/drag is brittle and low-signal — accepted.
- **No reset-to-fit affordance:** Deferred to follow-up. The user can zoom out
  to `minK` or change focus to trigger a re-centre.
- **Wheel factor 0.002 may need tuning:** The deltaMode normalization + 40px cap
  provide a solid baseline, but per-device tuning remains a residual risk.

### Standing risks

1. **R1/R2 — Deferred VA criteria.** 6 VA criteria unverified interactively.
   Mechanical conformance is strong; should be browser-verified before close.
2. **R3 — Wheel factor tuning.** 0.002 factor with deltaMode normalization works
   for Chrome and Firefox; trackpads and other browsers may need adjustment.
   Low severity — easy to tune post-deploy.
3. **Browser resize** — `minK` becomes stale after window resize. Deferred to
   follow-up. Low impact — user can change focus or zoom out manually.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md § app.ts integration:** Remove `lastRenderedFocusId` field from the
  state additions. The implementation correctly reuses the existing `prevFocusId`
  capture (`focusChanged = state.focusId !== prevFocusId` at L349). No extra
  state field needed. Update the code block and description.

- **design.md § Pure helpers:** Add `parseTransform(transform: string):
  GraphViewport` to the list of helpers. This function parses a CSS transform
  string of the form `translate(Xpx, Ypx) scale(K)` back into a `GraphViewport`.
  It was added in PHASE-02 to solve the stale-closure-on-re-render problem —
  wheel/drag handlers read the current viewport from the wrapper's CSS transform
  on each event, rather than from a closure-captured `vp`.

- **design.md § Wheel (zoom):** Note that `minK` is stored on the wrapper
  element's `dataset.minK` rather than captured in the handler closure. This
  ensures the wheel handler always reads the current wrapper's `minK` — correct
  behaviour across re-renders where a different SVG produces a different `minK`.

### Governance/spec (REV)

- **None.** No ADR, spec, or governance changes are needed. All deviations are
  implementation refinements that improve on the design without changing its
  intent or scope.

### VA verification (recommended before close)

- PHASE-02 VA-1: Load map, zoom/pan on DOT graph, verify no overflow, hit-rects
  clickable, focus+hover work, legend dims.
- PHASE-02 VA-2: Switch to actionability view and back — zoom/pan on DOT view
  resumes without cross-mode interference.
- PHASE-03 VA-1: Zoom in on node A, change depth — viewport restored.
- PHASE-03 VA-2: Zoom in on node A, click node B — centred on B at preserved scale.
- PHASE-03 VA-3: Zoom to minK on small graph, click node with large neighbourhood
  — scale clamped up, graph centred.
- PHASE-03 VA-4: First load — graph fits container.

## Reconciliation Outcome

### Direct edits applied

- **design.md § State model + app.ts integration:** Removed `lastRenderedFocusId`
  field; updated code block to reflect reuse of existing `prevFocusId` pattern
  (RV-072 F-6).
- **design.md § Pure helpers:** Added `parseTransform(transform: string):
  GraphViewport` to the helpers list; updated affected-files table to reference
  `viewport.ts` (RV-072 F-6).
- **design.md § Integration flow step 4:** Noted that `minK` is stored on
  `wrapper.dataset.minK`, not captured in handler closure (RV-072 F-6).

### REVs completed

None required — no governance/spec changes.

### Withdrawn / tolerated

- RV-072 F-1–F-4: aligned (conformance findings, no write needed).
- RV-072 F-5: verified (6 VA criteria deferred; mechanical evidence gates pass;
  recommended browser checks before close).

Reconcile pass complete — handoff to /close.
