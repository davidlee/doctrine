// viewport.ts — zoom/pan viewport math for the DOT/Graphviz semantic graph view.
//
// Pure helpers: no DOM dependencies. The impure seam (reading SVG dimensions
// from a live element) is `readSvgDims` — callers pass the real element in.

export interface GraphViewport {
  /** translate-x in px, relative to container top-left */
  x: number;
  /** translate-y in px */
  y: number;
  /** scale — 1 = 100%. Clamped to [minK, maxK] by callers. */
  k: number;
}

/**
 * Compute the viewport that fits the SVG inside the container.
 * k is never upscaled beyond 1 (a small SVG stays at natural size).
 */
export function fitViewport(
  svgW: number,
  svgH: number,
  cw: number,
  ch: number,
): GraphViewport {
  // Guard against zero or negative dimensions — Graphviz can't produce those,
  // but a zero-size container or an empty DOM yields nonsense.
  if (svgW <= 0 || svgH <= 0) {
    return { x: 0, y: 0, k: 1 };
  }

  const k = Math.min(cw / svgW, ch / svgH, 1);
  const x = (cw - svgW * k) / 2;
  const y = (ch - svgH * k) / 2;
  return { x, y, k };
}

/**
 * Compute the target viewport after a focus (entity) change.
 *
 * - null prior → fit the new graph to the container.
 * - prior.k < minK → clamp to minK, centre.
 * - prior.k >= minK → preserve k, centre.
 */
export function applyFocusChange(
  vp: GraphViewport | null,
  minK: number,
  svgW: number,
  svgH: number,
  cw: number,
  ch: number,
): GraphViewport {
  if (vp === null) {
    return fitViewport(svgW, svgH, cw, ch);
  }

  const k = vp.k < minK ? minK : vp.k;
  const x = (cw - svgW * k) / 2;
  const y = (ch - svgH * k) / 2;
  return { x, y, k };
}

/**
 * Pan a viewport by a screen-space delta. translate() is applied in the
 * (unscaled) parent coordinate space with transform-origin 0 0, so a screen-px
 * mouse delta maps 1:1 — it is NOT divided by k. Dividing would accelerate the
 * pan when zoomed out (k < 1) and lag it when zoomed in (k > 1). Returns a new
 * object; k is preserved.
 */
export function panViewport(
  vp: GraphViewport,
  dxScreen: number,
  dyScreen: number,
): GraphViewport {
  return { x: vp.x + dxScreen, y: vp.y + dyScreen, k: vp.k };
}

/**
 * Clamp a viewport's scale to [minK, maxK]. Returns a new object.
 */
export function clampViewport(
  vp: GraphViewport,
  minK: number,
  maxK: number,
): GraphViewport {
  return {
    x: vp.x,
    y: vp.y,
    k: Math.max(minK, Math.min(maxK, vp.k)),
  };
}

/**
 * Read the rendered dimensions of an injected SVG element.
 * Uses getBoundingClientRect() — not width/height attributes —
 * because Graphviz outputs in points (pt), not pixels.
 */
/**
 * Parse a viewport from a CSS transform string of the form
 * `translate(Xpx, Ypx) scale(K)`. Returns identity if parsing fails.
 * Callers pass an element's `style.transform` — the function is pure
 * (string → viewport) for testability.
 */
export function parseTransform(transform: string): GraphViewport {
  // Match: translate(Xpx, Ypx) scale(K)  — the format we write.
  const m = /translate\(([\d.-]+)px,\s*([\d.-]+)px\)\s*scale\(([\d.-]+)\)/.exec(transform);
  if (m?.[1] !== undefined && m[2] !== undefined && m[3] !== undefined) {
    return { x: parseFloat(m[1]), y: parseFloat(m[2]), k: parseFloat(m[3]) };
  }
  return { x: 0, y: 0, k: 1 };
}

/**
 * Read the rendered dimensions of an injected SVG element.
 * Uses getBoundingClientRect() — not width/height attributes —
 * because Graphviz outputs in points (pt), not pixels.
 */
export function readSvgDims(svgEl: SVGSVGElement): { w: number; h: number } {
  const r = svgEl.getBoundingClientRect();
  const w = r.width;
  const h = r.height;
  if (w <= 0 || h <= 0) {
    return { w: 1, h: 1 };
  }
  return { w, h };
}
