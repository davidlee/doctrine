// svg.ts — shared SVG DOM manipulation for Doctrine Map frontend
//
// Handles hit-rect injection, click/hover handler wiring, focus highlight,
// and legend dimming. Shared by entity graph and concept map rendering.

export interface SvgHandlerOpts {
  extractId: (g: SVGGElement) => string | null;
  onClick: (id: string) => void;
  onHoverEnter: (id: string) => void;
  onHoverLeave: () => void;
}

// Inject transparent hit-rect as first child of every <g class="node">.
// Idempotent — skips nodes that already have a hit-rect child.
export function injectHitRects(svgEl: SVGSVGElement): void {
  const groups = svgEl.querySelectorAll<SVGGElement>('.node');
  for (const g of groups) {
    // Skip if already injected
    const existing = g.querySelector('[data-doctrine-hit]');
    if (existing !== null) continue;

    try {
      const bbox = g.getBBox();
      if (bbox.width > 0 && bbox.height > 0) {
        const hitRect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        hitRect.setAttribute('x', String(bbox.x));
        hitRect.setAttribute('y', String(bbox.y));
        hitRect.setAttribute('width', String(bbox.width));
        hitRect.setAttribute('height', String(bbox.height));
        hitRect.setAttribute('fill', 'transparent');
        hitRect.setAttribute('stroke', 'none');
        hitRect.setAttribute('data-doctrine-hit', 'true');
        g.insertBefore(hitRect, g.firstChild);
      }
    } catch { /* getBBox may fail on detached nodes */ }
  }
}

// Wire click + mouseenter/mouseleave on every <g class="node">.
export function wireHandlers(svgEl: SVGSVGElement, opts: SvgHandlerOpts): void {
  const groups = svgEl.querySelectorAll<SVGGElement>('.node');
  for (const g of groups) {
    const nodeId = opts.extractId(g);
    if (nodeId === null) continue;

    g.classList.add('doctrine-node');

    g.addEventListener('click', () => { opts.onClick(nodeId); });
    g.addEventListener('mouseenter', () => { opts.onHoverEnter(nodeId); });
    g.addEventListener('mouseleave', () => { opts.onHoverLeave(); });
  }
}

// Apply/remove .doctrine-node--focus on the SVG <g> whose title matches focusId.
// getTitle: function(g) → string — reads the group's title text.
// prevFocusId: previous focus. focusId: current focus.
export function applyFocusHighlight(
  svgEl: SVGSVGElement,
  focusId: string,
  prevFocusId: string | null,
  getTitle: (g: SVGGElement) => string,
): void {
  // Remove old focus
  if (prevFocusId !== null) {
    const oldNodes = svgEl.querySelectorAll<SVGGElement>('.doctrine-node--focus');
    for (const n of oldNodes) {
      n.classList.remove('doctrine-node--focus');
    }
  }

  // Apply new focus
  const groups = svgEl.querySelectorAll<SVGGElement>('.node');
  for (const g of groups) {
    if (getTitle(g) === focusId) {
      g.classList.add('doctrine-node--focus');
      break;
    }
  }
}

// Dim legend items whose edge labels are absent from edgeLabels.
// Edge labels are compared against data-labels attribute
// (comma-separated, trimmed, lowercased).
export function dimLegend(svgEl: SVGSVGElement, edgeLabels: string[]): void {
  const items = svgEl.ownerDocument.querySelectorAll<HTMLElement>('.legend-item');
  if (items.length === 0) return;
  const labelSet = new Set(edgeLabels.map((l) => l.toLowerCase()));
  for (const item of items) {
    const dataLabels = item.getAttribute('data-labels') ?? '';
    const labels = dataLabels.split(',');
    let anyPresent = false;
    for (const label of labels) {
      if (labelSet.has(label.trim())) {
        anyPresent = true;
        break;
      }
    }
    item.classList.toggle('legend-dimmed', !anyPresent);
  }
}
