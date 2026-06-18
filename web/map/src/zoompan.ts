// zoompan.ts — DOM seam for the graph zoom/pan interaction shared by the
// entity-graph (render.ts) and concept-map (concept-map.ts) views (IMP-100).
//
// Both views render into the same `.graph-area` container, so the wheel/mousedown
// handlers are wired once per element and read live state — the
// `.graph-transform-layer` transform, its `minK`, and the active viewport
// callback — never a closure captured at first wire. Pure viewport math stays in
// viewport.ts; this module owns only the impure DOM wrap + event wiring.

import { fitViewport, applyFocusChange, readSvgDims, parseTransform, panViewport, type GraphViewport } from './viewport';

export interface MountZoomPanOpts {
  /** Viewport to restore — null/undefined = fit the SVG to the container. */
  initialViewport?: GraphViewport | null | undefined;
  /** True when the focused entity changed since the last render — centring rules. */
  focusChanged?: boolean | undefined;
  /** Called on every zoom/pan mutation so caller state stays current. */
  onViewportChange?: ((vp: GraphViewport) => void) | undefined;
}

// Active viewport callback per container. Replaces a stale first-wire closure so
// a single wired handler set serves whichever view currently owns the container.
const callbacks = new WeakMap<HTMLElement, (vp: GraphViewport) => void>();

/**
 * Wrap an injected SVG in the zoom/pan transform layer, apply the target
 * viewport (fit / focus-change / preserve), and wire wheel + drag-to-pan once
 * per container lifetime. The wiring is idempotent via `dataset.zoomWired`; the
 * viewport callback is refreshed on every call.
 */
export function mountZoomPan(
  container: HTMLElement,
  svgEl: SVGSVGElement,
  opts: MountZoomPanOpts,
): void {
  // Register the active callback (or a no-op) for this container's handlers.
  callbacks.set(container, opts.onViewportChange ?? ((): void => { /* no-op */ }));

  const svgDims = readSvgDims(svgEl);
  const cw = container.clientWidth;
  const ch = container.clientHeight;
  const minK = fitViewport(svgDims.w, svgDims.h, cw, ch).k;

  // Compute target viewport per the rules in SL-094 design.md.
  let vp: GraphViewport;
  if (opts.initialViewport == null) {
    vp = fitViewport(svgDims.w, svgDims.h, cw, ch);
  } else if (opts.focusChanged === true) {
    vp = applyFocusChange(opts.initialViewport, minK, svgDims.w, svgDims.h, cw, ch);
  } else {
    vp = opts.initialViewport;
  }

  // Wrap the SVG in the transform layer.
  const wrapper = document.createElement('div');
  wrapper.className = 'graph-transform-layer';
  wrapper.style.transform = `translate(${String(vp.x)}px, ${String(vp.y)}px) scale(${String(vp.k)})`;
  wrapper.dataset.minK = String(minK);
  container.removeChild(svgEl);
  wrapper.appendChild(svgEl);
  container.appendChild(wrapper);

  // Wire zoom/pan handlers once per .graph-area lifetime.
  if (container.dataset.zoomWired === 'true') return;
  container.dataset.zoomWired = 'true';

  const emit = (v: GraphViewport): void => { callbacks.get(container)?.(v); };

  // Wheel → zoom
  container.addEventListener('wheel', (e) => {
    const layer = container.querySelector<HTMLElement>('.graph-transform-layer');
    if (layer === null) return; // cross-mode guard
    e.preventDefault();
    let delta = e.deltaMode === 1 ? e.deltaY * 16 : e.deltaY;
    if (Math.abs(delta) > 40) delta = Math.sign(delta) * 40;
    const rect = container.getBoundingClientRect();
    const cx = e.clientX - rect.left;
    const cy = e.clientY - rect.top;
    const cur = parseTransform(layer.style.transform);
    const curMinK = parseFloat(layer.dataset.minK ?? '1');
    const newK = Math.max(curMinK, Math.min(10, cur.k * (1 - delta * 0.002)));
    const scaleRatio = newK / cur.k;
    const newX = cx - scaleRatio * (cx - cur.x);
    const newY = cy - scaleRatio * (cy - cur.y);
    layer.style.transform = `translate(${String(newX)}px, ${String(newY)}px) scale(${String(newK)})`;
    emit({ x: newX, y: newY, k: newK });
  }, { passive: false });

  // Mousedown → drag to pan
  container.addEventListener('mousedown', (e) => {
    const layer = container.querySelector<HTMLElement>('.graph-transform-layer');
    if (layer === null) return; // cross-mode guard
    // Don't start a pan if the user clicked on a node (handled by svg.ts).
    if (e.target instanceof Element && e.target.closest('.doctrine-node') !== null) return;
    e.preventDefault();
    container.classList.add('grabbing');
    let origin = { x: e.clientX, y: e.clientY };
    const onMove = (me: MouseEvent): void => {
      const cur = parseTransform(layer.style.transform);
      const next = panViewport(cur, me.clientX - origin.x, me.clientY - origin.y);
      origin = { x: me.clientX, y: me.clientY };
      layer.style.transform = `translate(${String(next.x)}px, ${String(next.y)}px) scale(${String(next.k)})`;
      emit(next);
    };
    const onUp = (): void => {
      container.classList.remove('grabbing');
      document.removeEventListener('mousemove', onMove);
    };
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp, { once: true });
  });
}
