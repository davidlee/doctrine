# SL-094 implementation notes

## Commits

```
1db6ecbf fix(SL-094): wheel handler reads minK from wrapper dataset, not closure
0ae47e7f feat(SL-094): PHASE-03 — app state integration + viewport persistence
72a309b7 feat(SL-094): PHASE-02 — CSS + graphPane() zoom/pan wiring
535090ab feat(SL-094): PHASE-01 — pure viewport helpers + types + 19 tests
```

## Surprises & adaptations

- **parseTransform() emergence.** The `dataset.zoomWired` guard prevents handler
  re-wiring across re-renders, but re-renders create a new wrapper with a fresh
  `vp` local. Wheel/drag handlers captured the old `vp` in closure → stale
  coordinates on second render. Solution: handlers read viewport from the
  wrapper's CSS transform string on every event via `parseTransform()`. Cleaner
  than handler teardown/rewiring.

- **minK on dataset, not in closure.** Same stale-closure problem applied to
  `minK` — re-renders may produce a different SVG with different `minK`. Fix
  (1db6ecbf): store `minK` on `wrapper.dataset.minK`, read it on each wheel
  event.

- **`lastRenderedFocusId` unnecessary.** The design proposed a new state field
  `lastRenderedFocusId`. The existing `prevFocusId` capture at `app.ts` L349
  already computes `focusChanged = state.focusId !== prevFocusId` — functionally
  equivalent, cleaner (no extra field). Implementation rightfully simplified.

## Design amendments needed

See RV-072 Reconciliation Brief:
1. design.md § app.ts integration: remove `lastRenderedFocusId`
2. design.md § Pure helpers: add `parseTransform()`
3. design.md § Wheel (zoom): note `minK` on wrapper dataset

## Deferred VA verification

6 VA criteria (PHASE-02 VA-1/VA-2, PHASE-03 VA-1–VA-4) not interactively
verified. All automated gates pass. Recommended browser verification before
close. See RV-072 Reconciliation Brief for the full list.

## Standing risks

- Wheel factor 0.002 may need per-device tuning (RV-071 R3 residual).
- Browser resize makes `minK` stale — deferred follow-up.
- No reset-to-fit affordance — deferred follow-up.
- Click-vs-drag disambiguation on `.doctrine-node` — deferred follow-up.
- No pinch-to-zoom — deferred follow-up.

## Follow-ups (backlog candidates)

- Pinch-to-zoom (touch gesture recognition)
- Resize/reflow handler (re-fit on window resize)
- Reset-to-fit affordance (double-click background)
- Click-vs-drag disambiguation on `.doctrine-node`
- Zoom-to-selected (animate to clicked node's centre)
