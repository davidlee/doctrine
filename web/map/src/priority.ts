import { graphStratify, sugiyama } from 'd3-dag';
import { select } from 'd3-selection';
import { zoom as d3zoom, zoomIdentity } from 'd3-zoom';
import { hoverDetailHtml } from './render';
import type { ActionabilityView, ActionabilityNode } from './types';

export interface LayoutNode extends ActionabilityNode {
  x?: number;
  y?: number;
  consequence?: number;
}

export interface LayoutEdge {
  source: LayoutNode;
  target: LayoutNode;
  kind: string;
}

export interface Viewport {
  x: number;
  y: number;
  k: number;
}

export interface PriorityRenderOpts {
  container: HTMLElement;
  view: ActionabilityView;
  zoomId: string | null;
  /** Viewport to restore on (re)render so pan/zoom survives — null = fit. */
  initialTransform?: Viewport | null;
  /** One-shot: animate to `zoomId`'s node (from `initialTransform`). */
  animateToZoom?: boolean;
  onNodeClick: (id: string) => void;
  onNodeHoverEnter?: (id: string) => void;
  onNodeHoverLeave?: () => void;
  onBackgroundClick?: () => void;
  /** Called with every viewport change so the caller can persist it. */
  onTransform?: (t: Viewport) => void;
  getCurrentSeq?: () => number;
}

export interface BBox {
  minX: number;
  minY: number;
  width: number;
  height: number;
}

const SVG_NS = 'http://www.w3.org/2000/svg';
const DEFAULT_VW = 960;
const DEFAULT_VH = 600;
const ZOOM_SCALE = 5;
const BBOX_PAD = 40;
const NODE_HALF_HEIGHT = 14;
const SCALE_EXTENT: [number, number] = [0.2, 8];
const ZOOM_DURATION_MS = 150;

/** Node box width — shared by the renderer and the fit-box computation. */
function nodeWidth(id: string): number {
  return Math.max(72, id.length * 7 + 16);
}

/**
 * Fit-to-content bounding box over the laid-out nodes (their centres plus
 * half-extents), padded. Empty / coordinate-less input falls back to the
 * default frame so the SVG always has a valid viewBox.
 */
export function layoutBBox(nodes: LayoutNode[], pad = BBOX_PAD): BBox {
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  for (const n of nodes) {
    if (n.x === undefined || n.y === undefined) continue;
    const hw = nodeWidth(n.id) / 2;
    minX = Math.min(minX, n.x - hw);
    maxX = Math.max(maxX, n.x + hw);
    minY = Math.min(minY, n.y - NODE_HALF_HEIGHT);
    maxY = Math.max(maxY, n.y + NODE_HALF_HEIGHT);
  }
  if (!Number.isFinite(minX)) {
    return { minX: 0, minY: 0, width: DEFAULT_VW, height: DEFAULT_VH };
  }
  return {
    minX: minX - pad,
    minY: minY - pad,
    width: maxX - minX + 2 * pad,
    height: maxY - minY + 2 * pad,
  };
}

/**
 * The viewport transform (d3-zoom space) that centres `node` within `bbox`
 * at scale `k`: applied to the zoom layer, screen = k·p + (x, y).
 */
export function zoomToNode(node: LayoutNode, bbox: BBox, k: number): { x: number; y: number; k: number } {
  const cx = bbox.minX + bbox.width / 2;
  const cy = bbox.minY + bbox.height / 2;
  return { x: cx - k * (node.x ?? 0), y: cy - k * (node.y ?? 0), k };
}

interface StratifyDatum {
  id: string;
  parentIds: string[];
  data: ActionabilityNode;
}

function emptyLayout(): { nodes: LayoutNode[]; edges: LayoutEdge[] } {
  return { nodes: [], edges: [] };
}

export function layoutGraph(view: ActionabilityView): { nodes: LayoutNode[]; edges: LayoutEdge[] } {
  if (view.nodes.length === 0) return emptyLayout();

  const { edges: viewEdges } = view;
  const edges = Array.isArray(viewEdges) ? viewEdges : [];

  const parentsByTarget = new Map<string, string[]>();
  for (const edge of edges) {
    if (edge.source.length === 0 || edge.target.length === 0) continue;
    const existing = parentsByTarget.get(edge.target);
    if (existing === undefined) {
      parentsByTarget.set(edge.target, [edge.source]);
    } else {
      existing.push(edge.source);
    }
  }

  const stratifyData: StratifyDatum[] = [];
  for (const node of view.nodes) {
    if (node.id.length === 0) continue;
    const pids = parentsByTarget.get(node.id);
    stratifyData.push({
      id: node.id,
      parentIds: pids ?? [],
      data: node,
    });
  }
  if (stratifyData.length === 0) return emptyLayout();

  const dag = graphStratify()(stratifyData);
  sugiyama().nodeSize([72, 28] as const).gap([20, 30] as const)(dag);

  const layoutNodes: LayoutNode[] = [];
  const nodeMap = new Map<string, LayoutNode>();
  for (const dagNode of dag.nodes()) {
    const datum: StratifyDatum = dagNode.data;
    const base = datum.data;
    const con = (base as unknown as { consequence?: unknown }).consequence;
    const ln: LayoutNode = {
      ...base,
      x: dagNode.x,
      y: dagNode.y,
    };
    if (typeof con === 'number') {
      ln.consequence = con;
    }
    layoutNodes.push(ln);
    nodeMap.set(ln.id, ln);
  }

  const layoutEdges: LayoutEdge[] = [];
  for (const edge of edges) {
    const source = nodeMap.get(edge.source);
    const target = nodeMap.get(edge.target);
    if (source === undefined || target === undefined) continue;
    layoutEdges.push({ source, target, kind: edge.kind });
  }

  return { nodes: layoutNodes, edges: layoutEdges };
}

export function renderGraph(opts: PriorityRenderOpts): void {
  const { container, view, zoomId, initialTransform, animateToZoom, onNodeClick, onBackgroundClick, onTransform } = opts;

  const layout = layoutGraph(view);
  const { nodes, edges } = layout;

  // eslint-disable-next-line no-restricted-syntax
  container.innerHTML = '';

  // On-graph hover tooltip — absolutely positioned, hidden until a node is
  // hovered, content sourced from the SAME builder as the side detail pane.
  const tooltip = document.createElement('div');
  tooltip.className = 'priority-tooltip';
  container.appendChild(tooltip);

  const bbox = layoutBBox(nodes);

  const svg = document.createElementNS(SVG_NS, 'svg');
  // Fit-to-content viewBox so the whole DAG is visible on load (IMP-092).
  svg.setAttribute(
    'viewBox',
    `${String(bbox.minX)} ${String(bbox.minY)} ${String(bbox.width)} ${String(bbox.height)}`,
  );
  svg.setAttribute('width', '100%');
  svg.setAttribute('height', '100%');

  // Arrow marker definition
  const defs = document.createElementNS(SVG_NS, 'defs');
  const marker = document.createElementNS(SVG_NS, 'marker');
  marker.setAttribute('id', 'needs-arrow');
  marker.setAttribute('markerWidth', '10');
  marker.setAttribute('markerHeight', '7');
  marker.setAttribute('refX', '9');
  marker.setAttribute('refY', '3.5');
  marker.setAttribute('orient', 'auto');
  const markerPath = document.createElementNS(SVG_NS, 'path');
  markerPath.setAttribute('d', 'M0,0 L10,3.5 L0,7 z');
  markerPath.setAttribute('fill', 'var(--priority-needs-edge, #C0392B)');
  marker.appendChild(markerPath);
  defs.appendChild(marker);
  svg.appendChild(defs);

  // Build node lookup
  const nodeMap = new Map<string, LayoutNode>();
  for (const n of nodes) nodeMap.set(n.id, n);

  // Zoom layer — d3-zoom owns its transform (free pan/zoom + zoom-to-selected).
  const zoomLayer = document.createElementNS(SVG_NS, 'g');
  zoomLayer.setAttribute('class', 'priority-zoom-layer');
  svg.appendChild(zoomLayer);

  // Edges
  for (const edge of edges) {
    const s = edge.source;
    const t = edge.target;
    if (s.x === undefined || s.y === undefined || t.x === undefined || t.y === undefined) continue;

    const line = document.createElementNS(SVG_NS, 'line');
    line.setAttribute('x1', String(s.x));
    line.setAttribute('y1', String(s.y));
    line.setAttribute('x2', String(t.x));
    line.setAttribute('y2', String(t.y));
    line.setAttribute('class', edge.kind === 'needs' ? 'priority-edge priority-needs-edge' : 'priority-edge priority-after-edge');
    if (edge.kind === 'needs') line.setAttribute('marker-end', 'url(#needs-arrow)');
    zoomLayer.appendChild(line);
  }

  // Nodes
  for (const node of nodes) {
    if (node.x === undefined || node.y === undefined) continue;
    const nx = node.x;
    const ny = node.y;
    const nw = nodeWidth(node.id);

    const group = document.createElementNS(SVG_NS, 'g');
    const rect = document.createElementNS(SVG_NS, 'rect');
    const text = document.createElementNS(SVG_NS, 'text');
    let classes = `priority-node priority-${node.actionability === '' ? 'terminal' : node.actionability}`;

    if (node.id === zoomId) classes += ' priority-node--zoom';
    group.setAttribute('class', classes);
    group.setAttribute('transform', `translate(${String(nx)} ${String(ny)})`);

    rect.setAttribute('x', String(-nw / 2));
    rect.setAttribute('y', '-14');
    rect.setAttribute('width', String(nw));
    rect.setAttribute('height', '28');
    rect.setAttribute('rx', '6');
    rect.setAttribute('ry', '6');
    group.appendChild(rect);

    text.setAttribute('text-anchor', 'middle');
    text.setAttribute('dominant-baseline', 'middle');
    text.textContent = node.id;
    group.appendChild(text);

    // Consequence badge
    if ((node.consequence ?? 0) > 0) {
      const badge = document.createElementNS(SVG_NS, 'g');
      const circle = document.createElementNS(SVG_NS, 'circle');
      const badgeText = document.createElementNS(SVG_NS, 'text');
      badge.setAttribute('class', 'priority-consequence-badge');
      badge.setAttribute('transform', `translate(${String((nw / 2) - 6)} -10)`);
      circle.setAttribute('r', '8');
      badge.appendChild(circle);
      badgeText.setAttribute('text-anchor', 'middle');
      badgeText.setAttribute('dominant-baseline', 'middle');
      badgeText.textContent = String(node.consequence);
      badge.appendChild(badgeText);
      group.appendChild(badge);
    }

    // Click handler
    group.addEventListener('click', ((id: string) => {
      return (evt: Event) => {
        evt.stopPropagation();
        onNodeClick(id);
      };
    })(node.id));

    // Hover classes + optional detail-pane callbacks + on-graph tooltip
    group.addEventListener('mouseenter', () => {
      group.classList.add('priority-node--hover');
      // eslint-disable-next-line no-restricted-syntax
      tooltip.innerHTML = hoverDetailHtml({
        id: node.id,
        title: node.title,
        kindLabel: node.kind,
        status: node.status,
      });
      tooltip.classList.add('priority-tooltip--visible');
      if (opts.onNodeHoverEnter !== undefined) opts.onNodeHoverEnter(node.id);
    });
    group.addEventListener('mousemove', (evt: MouseEvent) => {
      const rect = container.getBoundingClientRect();
      const pad = 12;
      let left = evt.clientX - rect.left + pad;
      let top = evt.clientY - rect.top + pad;
      // Keep the tooltip inside the container.
      left = Math.min(left, Math.max(0, rect.width - tooltip.offsetWidth - pad));
      top = Math.min(top, Math.max(0, rect.height - tooltip.offsetHeight - pad));
      tooltip.style.left = `${String(left)}px`;
      tooltip.style.top = `${String(top)}px`;
    });
    group.addEventListener('mouseleave', () => {
      group.classList.remove('priority-node--hover');
      tooltip.classList.remove('priority-tooltip--visible');
      if (opts.onNodeHoverLeave !== undefined) opts.onNodeHoverLeave();
    });

    zoomLayer.appendChild(group);
  }

  container.appendChild(svg);

  // ── Viewport: free pan/zoom + zoom-to-selected (IMP-092) ──────────────────
  const behavior = d3zoom<SVGSVGElement, unknown>()
    .scaleExtent(SCALE_EXTENT)
    .on('zoom', (event: { transform: { x: number; y: number; k: number; toString: () => string } }) => {
      zoomLayer.setAttribute('transform', event.transform.toString());
      if (onTransform !== undefined) {
        onTransform({ x: event.transform.x, y: event.transform.y, k: event.transform.k });
      }
    });
  const sel = select(svg);
  sel.call(behavior);

  // Background click (not on a node — nodes stopPropagation) clears the zoom.
  if (onBackgroundClick !== undefined) {
    svg.addEventListener('click', () => {
      onBackgroundClick();
    });
  }

  // Restore the prior viewport instantly so any animation starts from the
  // user's current position (not the freshly rebuilt identity).
  const start =
    initialTransform != null
      ? zoomIdentity.translate(initialTransform.x, initialTransform.y).scale(initialTransform.k)
      : zoomIdentity;
  sel.call((s) => {
    behavior.transform(s, start);
  });

  // One-shot: a node was just clicked — animate from the current view to it.
  const zn = animateToZoom === true && zoomId !== null ? nodeMap.get(zoomId) : undefined;
  if (zn?.x !== undefined && zn.y !== undefined) {
    const t = zoomToNode(zn, bbox, ZOOM_SCALE);
    const target = zoomIdentity.translate(t.x, t.y).scale(t.k);
    sel
      .transition()
      .duration(ZOOM_DURATION_MS)
      .call((s) => {
        behavior.transform(s, target);
      });
  }
}
