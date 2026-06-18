# Review RV-071 — design of SL-094

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

## Inquisition against the SL-094 design artifact

The design proposes vanilla zoom/pan/crop for the DOT/Graphviz `graphPane()`
view — a CSS-transform wrapper `<div>` inside `.graph-area`, driven by wheel
(zoom) and mousedown/mousemove (pan) events, with viewport persistence and a
fit-to-container lower bound.

**Lines of interrogation:**

1. **Vanilla purity.** Does the design introduce any d3 dependency on the DOT
   path? Does it lean on Graphviz SVG internals?

2. **Coexistence with svg.ts.** Hit-rects use `getBBox()` — does the CSS
   transform wrapper break, displace, or distort them? Do focus/hover classes
   and `dimLegend()` survive?

3. **State & viewport rules.** Are the focus-change centring rules (clamp to
   minK, preserve k if above) complete and non-surprising? Is there a reset
   path the user would expect but the design omits?

4. **Mechanical correctness.** SVG dimension reading (pt vs px), event handler
   lifecycle (accumulation), wrapper recreation per render, drag coordinate
   math (divide-by-k) — are there hidden bugs?

5. **Boundary conditions.** Empty graph, zero-size SVG, browser resize, rapid
   scroll, fast drag off-window — does the design handle or acknowledge each?

6. **Slice-design coherence.** Does the slice scope (`slice-094.md`) agree with
   the design on what's in scope, out of scope, and deferred to follow-up?

7. **Doctrinal alignment.** Are ADRs (especially ADR-001 layering, ADR-004
   outbound relations), conventions, and the mortal sins respected?

**Held to:** domain_map invariants I1–I6 and risks R1–R3. The design artifact
(`.doctrine/slice/094/design.md`) is the primary accused; the slice scope
(`slice-094.md`) is its accomplice; ISS-021 is the complaining witness.

## Synthesis

### Verdict

The design is **sound in architecture but incomplete in three mechanical details**
that must be resolved before implementation. No doctrinal violations. No scope
drift. The vanilla-SVG approach correctly avoids d3 and leaves Graphviz SVG
internals untouched. The CSS-transform-wrapper strategy cleanly isolates the
zoom/pan mechanic from hit-rects, handler wiring, focus highlighting, and legend
dimming — domain_map invariants I1 and I2 hold.

Three blockers of implementation correctness (F-1 through F-3) are conceded by
the responder with concrete fixes that are small in surface area:

- **F-1 (blocker, fix-now):** Wheel delta must be normalized via `deltaMode`
  before applying the 0.002 scale factor, or the zoom will be ~30× too slow in
  Firefox and erratic on trackpads. Risk R3 was already identified in the
  domain_map but not addressed in the design.

- **F-2 (major, fix-now):** Drag listeners must use `{ once: true }` or an
  `AbortController` to prevent stale-listener accumulation when `mouseup` fires
  outside the viewport. A one-line change to the `mouseup` listener
  registration.

- **F-3 (major, fix-now):** Zoom/pan handlers on `.graph-area` must gate on
  the presence of `.graph-transform-layer` in the DOM — a two-line guard that
  prevents silent interference with the d3.zoom path on the actionability view
  and the concept-map renderer.

Two minor design amendments (F-4, F-5) sharpen the artifact:

- **F-4 (minor, design-wrong):** The design must explicitly acknowledge the
  depth-change edge case in D5's viewport-restoration rule.

- **F-5 (minor, fix-now):** `touch-action: none` on `.graph-area` is a
  single-property CSS addition needed to make drag-to-pan work on touch
  devices; it does not depend on the pinch-to-zoom follow-up.

F-6 (tolerated imprecision in the `dimLegend` scope claim) and F-7 (seq-flow
correctness, aligned) require no changes.

### Elements Found Clean

- **Vanilla purity (interrogation line 1):** No d3 import. No Graphviz SVG
  parsing. CSS transform on a wrapper `<div>` is SVG-agnostic.

- **Coexistence with svg.ts (line 2):** `getBBox()` operates in SVG coordinate
  space, unaffected by CSS transforms on ancestor divs. `wireHandlers` and
  `injectHitRects` query within the SVG subtree. `.doctrine-node` cursor
  (pointer) inherits correctly over the container's `grab`.

- **Core viewport rules (line 3):** Fit-to-container `minK` (I4), focus-change
  centre-with-clamp, same-focus restoration (I3) — all internally consistent.
  The depth-change edge case is acknowledged via F-4.

- **Coordinate math (line 4):** `newK = k * (1 - delta * factor)` with
  cursor-relative re-anchoring (`cx - (newK/k)*(cx - x)`) is correct.
  Pan-tracking via `(clientX - origin.x) / k` correctly undoes scale for 1:1
  mouse tracking.

- **SVG dimension reading (line 4):** `getBoundingClientRect()` returns CSS
  pixels, correctly translating Graphviz's point-based output. Zero-size
  guard with `minK = 1` fallback is adequate.

- **Event handler lifecycle (line 4):** The `dataset.zoomWired` guard
  (satisfying I5) correctly prevents duplicate wiring across re-renders. The
  async `seq` check prevents stale-render handlers from being attached (F-7).

- **Slice-design coherence (line 6):** Scope and design agree on in-scope items
  (scroll-wheel zoom, drag-to-pan, crop-on-bounds) and deferred follow-ups
  (pinch-to-zoom, resize/reflow, reset-to-fit, click-vs-drag, zoom-to-selected).

- **Doctrinal alignment (line 7):** No ADR-001 layering violation (the frontend
  is not in the Rust tier system). ADR-004's outbound-relations paradigm does
  not apply. The pure-helpers extraction (`fitViewport`, `applyFocusChange`,
  `clampViewport`) aligns with the pure/imperative split principle.

### Standing Risks

1. **R3 (domain_map):** Wheel delta browser variance. **Mitigated** by F-1's
   fix-now disposition — normalize via `deltaMode`. Residual risk: the 0.002
   factor itself may still need tuning per-device; acceptable as a
   post-implementation tuning knob.

2. **R2 (domain_map):** CSS transform wrapper vs `getBBox()`. **Resolved** —
   the wrapper is outside the SVG, so SVG-internal coordinate queries are
   unaffected. No residual risk.

3. **Browser resize** (design §Open questions): Deferred to follow-up. If
   the window resizes, `minK` becomes stale. The user can zoom out manually
   or change focus to trigger a re-centre. Low severity for v1; should be
   captured as a backlog item.

4. **Reset-to-fit affordance** (design §Open questions): Also deferred. No
   way to reset zoom to fit-to-container without changing focus. User impact:
   minor (scroll-wheel can zoom out to `minK`).

5. **Click-vs-drag disambiguation on nodes** (design §Open questions): The
   design's mousedown handler correctly skips `.doctrine-node` targets, so
   panning doesn't interfere with node clicks. However, a small drag starting
   on a node still fires the node's click — existing behaviour, not a
   regression. Disambiguation deferred.

### Tradeoffs Consciously Accepted

- **D5 — depth change preserves viewport:** The user's zoom/pan is not
  discarded when they adjust depth. The cost (F-4) is that the prior
  coordinates may point to empty space after graph topology changes.
  Accepted with documentation.

- **D3 — minK = fit-to-container, not 0.1:** Prevents losing the graph
  entirely. The user cannot zoom out to arbitrarily small scales, which is
  desirable for this use case.

- **Vanilla JS over d3:** The DOT path stays dependency-light. The cost is
  writing 40–60 lines of event handler code that d3.zoom provides for free.
  Accepted: the priority.ts path already uses d3; keeping the DOT path vanilla
  avoids coupling the two rendering pipelines.

- **No reset-to-fit affordance in v1:** The design is scope-disciplined.
  Double-click-to-reset is trivial to add later and does not affect the core
  zoom/pan architecture.

- **No touch pinch-to-zoom:** Deferred. The `touch-action: none` addition
  (F-5) ensures single-finger touch-pan works; pinch requires separate gesture
  recognition.

HERESIS URITOR; DOCTRINA MANET.
