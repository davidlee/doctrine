import { graphStratify, sugiyama } from 'd3-dag';
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

export interface PriorityRenderOpts {
  container: HTMLElement;
  view: ActionabilityView;
  zoomId: string | null;
  onNodeClick: (id: string) => void;
  getCurrentSeq?: () => number;
}

const SVG_NS = 'http://www.w3.org/2000/svg';
const DEFAULT_VW = 960;
const DEFAULT_VH = 600;
const ZOOM_SCALE = 5;

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
  const { container, view, zoomId, onNodeClick } = opts;

  const layout = layoutGraph(view);
  const { nodes, edges } = layout;

  // eslint-disable-next-line no-restricted-syntax
  container.innerHTML = '';

  const svg = document.createElementNS(SVG_NS, 'svg');
  svg.setAttribute('viewBox', `0 0 ${String(DEFAULT_VW)} ${String(DEFAULT_VH)}`);
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

  // Zoom layer
  const zoomLayer = document.createElementNS(SVG_NS, 'g');
  zoomLayer.setAttribute('class', 'priority-zoom-layer');
  if (zoomId !== null) {
    const zn = nodeMap.get(zoomId);
    if (zn?.x !== undefined && zn.y !== undefined) {
      const tx = DEFAULT_VW / 2 - zn.x * ZOOM_SCALE;
      const ty = DEFAULT_VH / 2 - zn.y * ZOOM_SCALE;
      zoomLayer.setAttribute('transform', `translate(${tx.toFixed(1)} ${ty.toFixed(1)}) scale(${String(ZOOM_SCALE)})`);
    }
  }
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
    const idLen = node.id.length;
    const nw = Math.max(72, idLen * 7 + 16);

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

    // Hover classes (no external callbacks in the new API)
    group.addEventListener('mouseenter', () => {
      group.classList.add('priority-node--hover');
    });
    group.addEventListener('mouseleave', () => {
      group.classList.remove('priority-node--hover');
    });

    zoomLayer.appendChild(group);
  }

  container.appendChild(svg);
}
