// Doctrine Map Explorer — entity-graph DOM construction (SL-091 PHASE-06)
// TypeScript rewrite of render.js. Depends on: model (neighbourhood, compareEdgesBySource),
// dot (graphToDot), api (renderDot, fetchMarkdown), svg (injectHitRects, wireHandlers, dimLegend).

import type { Graph, CatalogNode, Edge, Neighbourhood, ActionabilityView, ActionabilityNode, RawEdge } from './types';
import { neighbourhood, compareEdgesBySource } from './model';
import { graphToDot } from './dot';
import { renderDot, fetchMarkdown } from './api';
import { injectHitRects, wireHandlers, dimLegend, type SvgHandlerOpts } from './svg';
import { type GraphViewport } from './viewport';
import { mountZoomPan } from './zoompan';
import { buildHash } from './router';
import markdownit from 'markdown-it';
import DOMPurify from 'dompurify';

// ---------------------------------------------------------------------------
// Module-specific types
// ---------------------------------------------------------------------------

export interface RenderedElements {
  entityList: HTMLElement | null;
  focusHeader: HTMLElement | null;
  graphArea: HTMLElement | null;
  hoverDetail: HTMLElement | null;
  relationshipTable: HTMLElement | null;
  relationshipTableBody: HTMLElement | null;
  markdownPane: HTMLElement | null;
  tableToggle: HTMLElement | null;
  depthSelector: HTMLElement | null;
  cmEdgeTable: HTMLElement | null;
  cmAddEdgeForm: HTMLElement | null;
  cmDiagnosticsPanel: HTMLElement | null;
}

export interface GraphPaneOpts {
  container: HTMLElement;
  graph: Graph;
  focusId: string;
  depth: number;
  dotAvailable: boolean;
  seq: number;
  getCurrentSeq: () => number;
  onNodeClick: (id: string) => void;
  onNodeHoverEnter: (id: string) => void;
  onNodeHoverLeave: (id: string) => void;
  /** Viewport to restore on (re)render so zoom/pan survives — null = fit. */
  initialViewport?: GraphViewport | null;
  /** True when focusId changed since the last render — triggers centring rules. */
  focusChanged?: boolean;
  /** Called on every zoom/pan mutation so app state stays current. */
  onViewportChange?: (vp: GraphViewport) => void;
}

export interface RelationshipTableOpts {
  container: HTMLElement;
  edges?: Edge[];
  graph: Graph;
  focusId: string | null;
  depth: number;
  viewMode?: string;
  actionabilityView?: ActionabilityView | null;
}

export interface EdgeDetailOpts {
  container: HTMLElement | null;
  edge: Edge | null;
  graph: Graph;
  depth: number;
  focusId: string;
}

// ---------------------------------------------------------------------------
// Module-level mutable state
// ---------------------------------------------------------------------------

export const elements: RenderedElements = {
  entityList: null,
  focusHeader: null,
  graphArea: null,
  hoverDetail: null,
  relationshipTable: null,
  relationshipTableBody: null,
  markdownPane: null,
  tableToggle: null,
  depthSelector: null,
  cmEdgeTable: null,
  cmAddEdgeForm: null,
  cmDiagnosticsPanel: null,
};

let _markdownIt: ReturnType<typeof markdownit> | null = null;

// ---------------------------------------------------------------------------
// DOM element factory
// ---------------------------------------------------------------------------

export function el(
  tag: string,
  attrs?: Record<string, string>,
  children?: (HTMLElement | string)[],
): HTMLElement {
  const e = document.createElement(tag);
  if (attrs !== undefined) {
    for (const key of Object.keys(attrs)) {
      const val = attrs[key];
      if (val === undefined) continue;
      if (key === 'className') {
        e.className = val;
      } else if (key === 'textContent') {
        e.textContent = val;
      } else if (key === 'innerHTML') {
        e.innerHTML = val;
      } else {
        e.setAttribute(key, val);
      }
    }
  }
  if (children !== undefined) {
    const kids = Array.isArray(children) ? children : [children];
    for (const c of kids) {
      if (typeof c === 'string') {
        e.appendChild(document.createTextNode(c));
      } else {
        e.appendChild(c);
      }
    }
  }
  return e;
}

// ---------------------------------------------------------------------------
// Cache frequently-accessed DOM elements
// ---------------------------------------------------------------------------

export function cacheElements(root: Document): void {
  elements.entityList = root.querySelector('.entity-list');
  elements.focusHeader = root.querySelector('.focus-header');
  elements.graphArea = root.querySelector('.graph-area');
  elements.hoverDetail = root.querySelector('.hover-detail');
  elements.relationshipTable = root.querySelector('.relationship-table');
  elements.relationshipTableBody = root.querySelector('.relationship-table tbody');
  elements.markdownPane = root.querySelector('.markdown-pane');
  elements.tableToggle = root.querySelector('.table-toggle');
  elements.depthSelector = root.querySelector('.depth-selector');
  elements.cmEdgeTable = root.querySelector('.cm-edge-table');
  elements.cmAddEdgeForm = root.querySelector('.cm-add-edge-form');
  elements.cmDiagnosticsPanel = root.querySelector('.cm-diagnostics-panel');
}

// ---------------------------------------------------------------------------
// HTML escaping
// ---------------------------------------------------------------------------

export function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

export function escapeAttr(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

// ---------------------------------------------------------------------------
// Entity list helper (module-private)
// ---------------------------------------------------------------------------

function buildEntityItem(
  node: CatalogNode,
  focusId: string | null,
  onFocus: (id: string) => void,
): HTMLElement {
  const li = document.createElement('li');
  li.className = 'entity-item';
  const tSpan = document.createElement('span');
  tSpan.className = 'entity-title';
  tSpan.textContent = node.title;
  li.appendChild(tSpan);

  const p = document.createElement('span');

  if (node.id === focusId) {
    li.classList.add('entity-item--active');
    p.classList.add('kind-pill--active');
  }
  p.className = 'kind-pill';
  p.setAttribute('data-kind', node.kindPrefix);
  p.style.background = 'var(--kind-' + node.kindPrefix + ')';
  p.textContent = node.kindPrefix;
  li.appendChild(p);

  li.addEventListener('click', ((id: string) => {
    return () => { onFocus(id); };
  })(node.id));
  return li;
}

// ---------------------------------------------------------------------------
// Entity list
// ---------------------------------------------------------------------------

interface EntityListOpts {
  container: HTMLElement;
  nodes: CatalogNode[];
  focusId: string | null;
  onFocus: (id: string) => void;
}

export function entityList(opts: EntityListOpts): void {
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition, @typescript-eslint/strict-boolean-expressions
  if (!opts.container) return;
  opts.container.innerHTML = '';
  for (const node of opts.nodes) {
    opts.container.appendChild(buildEntityItem(node, opts.focusId, opts.onFocus));
  }
}

// ---------------------------------------------------------------------------
// Focus header
// ---------------------------------------------------------------------------

interface FocusHeaderOpts {
  container: HTMLElement;
  focusId: string | null;
  graph: Graph;
}

export function focusHeader(opts: FocusHeaderOpts): void {
  const container = opts.container;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition, @typescript-eslint/strict-boolean-expressions
  if (!container) return;

  // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
  if (!opts.focusId) {
    container.innerHTML =
      '<span class="placeholder">Entity title \u2014 kind \u00b7 status</span>';
    return;
  }

  const node = opts.graph.nodes.get(opts.focusId);
  // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
  if (!node) {
    container.innerHTML =
      '<span class="placeholder">Entity title \u2014 kind \u00b7 status</span>';
    return;
  }

  container.innerHTML =
    '<span>' +
    escapeHtml(node.title) +
    '</span>' +
    ' <span class="kind-pill" data-kind="' +
    escapeAttr(node.kindPrefix) +
    '" style="background:var(--kind-' +
    escapeHtml(node.kindPrefix) +
    ')">' +
    escapeHtml(node.kindPrefix) +
    '</span>' +
    ' <span class="status">' +
    escapeHtml(node.status) +
    '</span>';
}

// ---------------------------------------------------------------------------
// View mode: toggle entity-graph vs concept-map vs edge UI visibility
// ---------------------------------------------------------------------------

export function setPageMode(mode: 'entity-graph' | 'actionability' | 'concept-map' | 'edge'): void {
  const layout = document.querySelector<HTMLElement>('.layout');
  // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
  if (layout) layout.dataset.pageMode = mode;

  // CM containers: hide AND clear when leaving concept-map mode
  if (mode !== 'concept-map') {
    const cmEdgeTable = elements.cmEdgeTable;
    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    if (cmEdgeTable) {
      cmEdgeTable.classList.add('u-hidden');
      cmEdgeTable.innerHTML = '';
    }
    const cmAddForm = elements.cmAddEdgeForm;
    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    if (cmAddForm) {
      cmAddForm.classList.add('u-hidden');
      cmAddForm.innerHTML = '';
    }
    const cmDiagPanel = elements.cmDiagnosticsPanel;
    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    if (cmDiagPanel) {
      cmDiagPanel.classList.add('u-hidden');
      cmDiagPanel.innerHTML = '';
    }
  }
}

// ---------------------------------------------------------------------------
// Relationship table helper (module-private)
// ---------------------------------------------------------------------------

function setRelationshipTableHeadings(container: HTMLElement | null, headings: string[]): void {
  const table = container?.closest('table') ?? null;
  const headerRow = table?.querySelector('thead tr') ?? null;
  // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
  if (!headerRow) return;
  headerRow.innerHTML = '';
  for (const heading of headings) {
    const th = document.createElement('th');
    th.textContent = heading;
    headerRow.appendChild(th);
  }
}

// ---------------------------------------------------------------------------
// Relationship table
// ---------------------------------------------------------------------------

export function relationshipTable(opts: RelationshipTableOpts): void {
  const tbody = opts.container;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition, @typescript-eslint/strict-boolean-expressions
  if (!tbody) return;

  if (opts.viewMode === 'actionability') {
    const nodes =
      opts.actionabilityView != null && Array.isArray(opts.actionabilityView.nodes)
        ? opts.actionabilityView.nodes
        : [];
    setRelationshipTableHeadings(tbody, [
      'id',
      'kind',
      'status',
      'actionability',
      'blockers',
      'consequence',
      'title',
    ]);
    tbody.innerHTML = '';

    if (nodes.length === 0) {
      tbody.innerHTML =
        '<tr><td colspan="7"><span class="placeholder">[No actionability data to show]</span></td></tr>';
      return;
    }

    for (const node of nodes) {
      const tr = document.createElement('tr');
      tr.style.cursor = 'pointer';
      tr.addEventListener('click', () => {
        window.location.hash = '#' + buildHash('focus', node.id, opts.depth);
      });

      const idCell = document.createElement('td');
      const idLink = document.createElement('a');
      idLink.href = '#' + buildHash('focus', node.id, opts.depth);
      idLink.textContent = node.id;
      idCell.appendChild(idLink);
      tr.appendChild(idCell);

      const kindCell = document.createElement('td');
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      kindCell.textContent = node.kind || '';
      tr.appendChild(kindCell);

      const statusCell = document.createElement('td');
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      statusCell.textContent = node.status || '';
      tr.appendChild(statusCell);

      const actionabilityCell = document.createElement('td');
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      actionabilityCell.textContent = node.actionability || '';
      tr.appendChild(actionabilityCell);

      const blockersCell = document.createElement('td');
      blockersCell.textContent = Array.isArray(node.blockers)
        ? node.blockers.join(', ')
        : '';
      tr.appendChild(blockersCell);

      const nodeExtra = node as ActionabilityNode & { consequence?: unknown };
      const consequenceCell = document.createElement('td');
      consequenceCell.textContent =
        nodeExtra.consequence != null
          ? // eslint-disable-next-line @typescript-eslint/no-base-to-string
            String(nodeExtra.consequence)
          : '';
      tr.appendChild(consequenceCell);

      const titleCell = document.createElement('td');
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      titleCell.textContent = node.title || '';
      tr.appendChild(titleCell);

      tbody.appendChild(tr);
    }
    return;
  }

  setRelationshipTableHeadings(tbody, [
    'src_id',
    'src_title',
    'label',
    'tgt_id',
    'tgt_title',
  ]);

  // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
  if (!opts.focusId) {
    tbody.innerHTML =
      '<tr><td colspan="5"><span class="placeholder">[Relationship table]</span></td></tr>';
    return;
  }

  const edges = opts.edges ?? [];
  const graph = opts.graph;
  const depth = opts.depth;

  edges.sort(compareEdgesBySource);

  tbody.innerHTML = '';
  if (edges.length === 0) {
    tbody.innerHTML =
      '<tr><td colspan="5"><span class="placeholder">[No relationships to show]</span></td></tr>';
    return;
  }

  for (const edge of edges) {
    const tr = document.createElement('tr');

    const srcCell = document.createElement('td');
    const srcA = document.createElement('a');
    srcA.href = '#' + buildHash('focus', edge.source, depth);
    srcA.textContent = edge.source;
    srcCell.appendChild(srcA);
    tr.appendChild(srcCell);

    const srcTitle = document.createElement('td');
    const srcNode = graph.nodes.get(edge.source);
    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    srcTitle.textContent = srcNode ? srcNode.title : '';
    tr.appendChild(srcTitle);

    const labelCell = document.createElement('td');
    const labelA = document.createElement('a');
    labelA.href = '#' + buildHash('edge', edge.id, depth);
    labelA.className = 'edge-id-link';
    labelA.textContent = edge.label;
    labelA.title = 'Edge: ' + edge.id;
    labelCell.appendChild(labelA);
    tr.appendChild(labelCell);

    const tgtCell = document.createElement('td');
    const tgtA = document.createElement('a');
    tgtA.href = '#' + buildHash('focus', edge.target, depth);
    tgtA.textContent = edge.target;
    tgtCell.appendChild(tgtA);
    tr.appendChild(tgtCell);

    const tgtTitle = document.createElement('td');
    const tgtNode = graph.nodes.get(edge.target);
    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    tgtTitle.textContent = tgtNode ? tgtNode.title : '';
    tr.appendChild(tgtTitle);

    tbody.appendChild(tr);
  }
}

// ---------------------------------------------------------------------------
// Hover detail pane
// ---------------------------------------------------------------------------

export interface HoverableNode {
  id: string;
  title: string;
  kindLabel: string;
  status: string;
}

interface HoverPaneOpts {
  container: HTMLElement;
  node: HoverableNode | null;
}

export function hoverPane(opts: HoverPaneOpts): void {
  const pane = opts.container;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition, @typescript-eslint/strict-boolean-expressions
  if (!pane) return;

  if (opts.node === null) {
    pane.innerHTML = '<span class="placeholder">Hover a node for details</span>';
    return;
  }

  const node = opts.node;
  pane.innerHTML =
    '<div class="hover-detail-content">' +
    '<span class="hover-detail-title">' +
    node.id +
    ': ' +
    escapeHtml(node.title) +
    '</span>' +
    '<span class="hover-detail-meta">' +
    node.kindLabel +
    ' \u00b7 ' +
    node.status +
    '</span>' +
    '</div>';
}

// ---------------------------------------------------------------------------
// Markdown rendering pipeline (module-private)
// ---------------------------------------------------------------------------

function renderMarkdown(text: string): string {
  _markdownIt ??= markdownit({ html: false, linkify: true, typographer: true });
  const raw = _markdownIt.render(text);
  return DOMPurify.sanitize(raw);
}

// ---------------------------------------------------------------------------
// Link policy (module-private)
// ---------------------------------------------------------------------------

function applyLinkPolicy(container: HTMLElement): void {
  const links = container.querySelectorAll('a');
  for (const a of links) {
    const href = a.getAttribute('href') ?? '';
    if (href.startsWith('http://') || href.startsWith('https://')) {
      a.setAttribute('target', '_blank');
      a.setAttribute('rel', 'noopener noreferrer');
    } else if (href.startsWith('#')) {
      // Anchor link — preserve
    } else if (href.length > 0) {
      const span = document.createElement('span');
      span.textContent = a.textContent;
      a.parentNode?.replaceChild(span, a);
    }
  }
}

// ---------------------------------------------------------------------------
// Fullscreen toggle (module-private)
// ---------------------------------------------------------------------------

function wireFullscreenToggle(container: HTMLElement): void {
  const btn = container.querySelector('.fullscreen-toggle');
  // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
  if (btn) {
    btn.addEventListener('click', () => {
      container.classList.toggle('fullscreen');
      const body = container.querySelector<HTMLElement>('.markdown-body');
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      if (body) body.classList.toggle('markdown-body--fullscreen');
    });
  }
}

// ---------------------------------------------------------------------------
// Markdown pane
// ---------------------------------------------------------------------------

interface MarkdownPaneOpts {
  container: HTMLElement;
  id: string;
  cache: Map<string, string>;
  currentFocusId?: string;
}

function wrapContent(innerHTML: string, id: string): string {
  return (
    '<div class="markdown-toolbar">' +
    '<span class="markdown-toolbar-title">' +
    escapeHtml(id) +
    '</span>' +
    '<button class="fullscreen-toggle" title="Toggle fullscreen">&square;</button>' +
    '</div>' +
    '<div class="markdown-body">' +
    innerHTML +
    '</div>'
  );
}

export function markdownPane(opts: MarkdownPaneOpts): void {
  const container = opts.container;
  const id = opts.id;
  const cache = opts.cache;
  const currentFocusId = opts.currentFocusId;

  // Cache hit
  const cached = cache.get(id);
  if (cached !== undefined) {
    container.innerHTML = wrapContent(renderMarkdown(cached), id);
    wireFullscreenToggle(container);
    applyLinkPolicy(container);
    return;
  }

  // Cache miss — fetch
  container.innerHTML = '';
  const loading = document.createElement('p');
  loading.className = 'loading';
  loading.textContent = 'Loading markdown…';
  container.appendChild(loading);

  fetchMarkdown(id)
    .then((text: string) => {
      if (currentFocusId !== id) return;
      cache.set(id, text);
      container.innerHTML = wrapContent(renderMarkdown(text), id);
      wireFullscreenToggle(container);
      applyLinkPolicy(container);
    })
    .catch((err: unknown) => {
      if (currentFocusId !== id) return;
      container.innerHTML = '';
      const e = err as { status?: number; message?: string };
      if (e.status === 404) {
        const msg = document.createElement('p');
        msg.className = 'muted';
        msg.textContent = 'No markdown body for ' + id;
        container.appendChild(msg);
      } else if (e.status === 501) {
        const info = document.createElement('p');
        info.className = 'info';
        info.textContent = 'Markdown not implemented for requirements';
        container.appendChild(info);
      } else {
        const error = document.createElement('p');
        error.className = 'error';
        error.textContent = 'Failed to load markdown: ' + (e.message ?? '');
        container.appendChild(error);
      }
    });
}

// ---------------------------------------------------------------------------
// Entity-graph SVG rendering
// ---------------------------------------------------------------------------

export function graphPane(opts: GraphPaneOpts): void {
  const container = opts.container;
  const graph = opts.graph;
  const focusId = opts.focusId;
  const depth = Math.max(0, Math.min(3, opts.depth));
  const dotAvailable = opts.dotAvailable;
  const seq = opts.seq;

  const nb: Neighbourhood = neighbourhood(focusId, depth, graph);
  const dotText = graphToDot(nb, focusId, depth);

  if (!dotAvailable) {
    container.innerHTML = '';
    const errMsg = document.createElement('p');
    errMsg.className = 'error';
    errMsg.textContent = 'Graphviz not available. DOT source:';
    container.appendChild(errMsg);
    const pre = document.createElement('pre');
    pre.textContent = dotText;
    container.appendChild(pre);
    return;
  }

  container.innerHTML = '';
  const loading = document.createElement('p');
  loading.className = 'loading';
  loading.textContent = 'Rendering graph…';
  container.appendChild(loading);

  renderDot(dotText)
    .then((svgText: string) => {
      if (seq !== opts.getCurrentSeq()) return;
      const clean = DOMPurify.sanitize(svgText, {
        USE_PROFILES: { svg: true },
      });
      container.innerHTML = clean;
      const svgEl = container.querySelector('svg');
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      if (svgEl) {
        // ── SVG internals: hit-rects, handlers, legend (unchanged) ──────────
        injectHitRects(svgEl);
        const handlerOpts: SvgHandlerOpts = {
          extractId: (g: SVGGElement): string | null => {
            const textEl = g.querySelector('text');
            const tc = textEl?.textContent ?? null;
            return tc?.trim() ?? null;
          },
          onClick: opts.onNodeClick,
          onHoverEnter: opts.onNodeHoverEnter,
          onHoverLeave: opts.onNodeHoverLeave as unknown as () => void,
        };
        wireHandlers(svgEl, handlerOpts);
        const edgeLabels: string[] = [];
        for (const e of nb.edges) {
          edgeLabels.push(e.label);
        }
        dimLegend(svgEl, edgeLabels);

        // ── Zoom/pan wrapper (SL-094, extracted IMP-100) ────────────────────
        mountZoomPan(container, svgEl, {
          initialViewport: opts.initialViewport,
          focusChanged: opts.focusChanged,
          onViewportChange: opts.onViewportChange,
        });
      }
    })
    .catch(() => {
      if (seq !== opts.getCurrentSeq()) return;
      container.innerHTML = '';
      const errMsg2 = document.createElement('p');
      errMsg2.className = 'error';
      errMsg2.textContent = 'Graphviz not available';
      container.appendChild(errMsg2);
      const pre2 = document.createElement('pre');
      pre2.textContent = dotText;
      container.appendChild(pre2);
    });
}

// ---------------------------------------------------------------------------
// Edge detail view
// ---------------------------------------------------------------------------

export function edgeDetail(opts: EdgeDetailOpts): void {
  const container = opts.container;
  const edge = opts.edge;
  const graph = opts.graph;
  const depth = opts.depth;
  const focusId = opts.focusId;

  // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
  if (!edge) {
    if (container !== null) {
      container.innerHTML = '<p class="error">Edge not found in graph</p>';
    }
    return;
  }

  const srcNode = graph.nodes.get(edge.source);
  const tgtNode = graph.nodes.get(edge.target);
  const rawWithOrigin = edge.raw as RawEdge & { origin?: { file?: string } };
  const originFile = rawWithOrigin.origin?.file ?? '-';

  const html =
    '<div class="edge-detail">' +
    '<h2>Edge: ' +
    escapeHtml(edge.id) +
    '</h2>' +
    '<table class="edge-detail-table">' +
    '<tr><th>Edge ID</th><td>' +
    escapeHtml(edge.id) +
    '</td></tr>' +
    '<tr><th>Source</th><td><a href="#' +
    buildHash('focus', edge.source, depth) +
    '">' +
    escapeHtml(edge.source) +
    '</a>' +
    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    (srcNode ? ' &mdash; ' + escapeHtml(srcNode.title) : '') +
    '</td></tr>' +
    '<tr><th>Label</th><td>' +
    escapeHtml(edge.label) +
    '</td></tr>' +
    '<tr><th>Target</th><td><a href="#' +
    buildHash('focus', edge.target, depth) +
    '">' +
    escapeHtml(edge.target) +
    '</a>' +
    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    (tgtNode ? ' &mdash; ' + escapeHtml(tgtNode.title) : '') +
    '</td></tr>' +
    '<tr><th>Origin file</th><td>' +
    escapeHtml(originFile) +
    '</td></tr>' +
    '</table>' +
    '<p class="edge-detail-back"><a href="#' +
    buildHash('focus', focusId, depth) +
    '">&larr; Back to ' +
    escapeHtml(focusId) +
    '</a></p>' +
    '</div>';

  if (container !== null) {
    container.innerHTML = html;
  }
}
