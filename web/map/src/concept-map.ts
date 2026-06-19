// Doctrine Map Explorer — concept map rendering (SL-091 PHASE-08)
// TypeScript rewrite of concept-map.js.
// Depends on: model (cmNeighbourhood, buildNodeLabelList, buildRelLabelList),
// dot (cmGraphToDot), api (renderDot), svg (injectHitRects, wireHandlers),
// render (escapeHtml, escapeAttr).

import type { ConceptMap, CmEdge, CmCell, CmEditOp, CmEditingCell } from './types';
import { cmNeighbourhood, buildNodeLabelList, buildRelLabelList } from './model';
import { cmGraphToDot } from './dot';
import { renderDot } from './api';
import { injectHitRects, wireHandlers, type SvgHandlerOpts } from './svg';
import { mountZoomPan } from './zoompan';
import { escapeHtml, escapeAttr } from './render';
import DOMPurify from 'dompurify';

// ---------------------------------------------------------------------------
// Exported interfaces
// ---------------------------------------------------------------------------

export interface CmDiagramOpts {
  container: HTMLElement | null;
  cm: ConceptMap;
  focusKey: string | null;
  dotAvailable: boolean;
  seq: number;
  getCurrentSeq?: () => number;
  onClick?: (key: string) => void;
  onHoverEnter?: (key: string) => void;
  onHoverLeave?: () => void;
}

export interface CmEdgeTableOpts {
  container: HTMLElement | null;
  cm: ConceptMap | null;
  focusKey: string | null;
  depth: number;
  /** Scope toggle: single instance (off) vs all rows sharing the label (on). */
  editAll: boolean;
  /** Which cell's hover-pencil is active (the inline input); null = none. */
  editingCell: CmEditingCell | null;
  /** The `[ ] edit all` checkbox flipped. */
  onToggleEditAll?: (checked: boolean) => void;
  /** A cell's pencil was clicked → begin inline-editing (edge, which cell). */
  onPencil?: (edge: CmEdge, cell: CmCell) => void;
  /** A node cell label was plain-clicked → cmFocus highlight (rel is inert). */
  onNodeClick?: (edge: CmEdge, cell: 'from' | 'to') => void;
  /** Inline input committed (Enter) with the new value. */
  onSubmitEdit?: (value: string) => void;
  /** Inline input cancelled (Esc). */
  onCancelEdit?: () => void;
  onRemoveEdge?: (source: string, rel: string, target: string) => void;
}

export interface CmAddEdgeFormOpts {
  container: HTMLElement | null;
  cm: ConceptMap | null;
  onSubmit?: (source: string, rel: string, target: string) => void;
}

// ---------------------------------------------------------------------------
// Module-private helpers
// ---------------------------------------------------------------------------

function renderCmHoverPane(nodeKey: string | null, cmData: ConceptMap | null | undefined): void {
  const pane = document.querySelector('.hover-detail');
  if (pane === null) return;
  if (nodeKey === null || nodeKey === '') {
    pane.innerHTML = '<span class="placeholder">Hover a node for details</span>';
    return;
  }
  let label: string = nodeKey;
  if (cmData != null) {
    for (const n of cmData.nodes) {
      if (n.key === nodeKey) {
        label = n.label;
        break;
      }
    }
  }
  pane.innerHTML =
    '<div class="hover-detail-content">' +
    '<span class="hover-detail-title">' +
    escapeHtml(label) +
    '</span>' +
    '<span class="hover-detail-meta">concept map node</span>' +
    '</div>';
}

function diagnosticLine(d: unknown): number | null {
  if (typeof d !== 'object' || d === null) return null;
  const obj = d as Record<string, unknown>;
  const keys = Object.keys(obj);
  if (keys.length === 0) return null;
  const firstKey = keys[0];
  if (firstKey === undefined) return null;
  const variant = obj[firstKey];
  if (typeof variant !== 'object' || variant === null) return null;
  const v = variant as Record<string, unknown>;
  if (typeof v.line === 'number') return v.line;
  if (typeof v.line_a === 'number') return v.line_a;
  return null;
}

function strOr(v: unknown): string {
  return typeof v === 'string' ? v : '';
}

function formatDiagnostic(d: unknown): string {
  if (typeof d !== 'object' || d === null) return 'Unknown diagnostic';
  const obj = d as Record<string, unknown>;
  const keys = Object.keys(obj);
  if (keys.length === 0) return 'Unknown diagnostic';
  const firstKey = keys[0];
  if (firstKey === undefined) return 'Unknown diagnostic';
  const v: Record<string, unknown> = (obj[firstKey] as Record<string, unknown> | null | undefined) ?? {};

  switch (firstKey) {
    case 'CanonicalNodeCollision':
      return 'Node label "' + escapeHtml(strOr(v.label)) + '" collides with key "' + escapeHtml(strOr(v.key)) + '" (first label "' + escapeHtml(strOr(v.first_label)) + '" takes precedence)';
    case 'SelfEdge':
      return 'Self-referencing edge: "' + escapeHtml(strOr(v.node_key)) + '" → "' + escapeHtml(strOr(v.node_key)) + '"';
    case 'SimilarNodeLabel':
      return 'Similar node labels: "' + escapeHtml(strOr(v.label_a)) + '" / "' + escapeHtml(strOr(v.label_b)) + '"';
    case 'RelationDrift':
      return 'Relation "' + escapeHtml(strOr(v.rel_a)) + '" appears only once — possible typo';
    case 'EntityRefLike':
      return '"' + escapeHtml(strOr(v.label)) + '" looks like an entity reference';
    case 'MalformedLine':
      return 'Malformed DSL at "' + escapeHtml(strOr(v.text)) + '"';
    case 'EmptyLabel':
      return 'Empty label in DSL';
    case 'DuplicateEdge':
      return 'Duplicate edge: "' + escapeHtml(strOr(v.from_key)) + '" > "' + escapeHtml(strOr(v.rel)) + '" > "' + escapeHtml(strOr(v.to_key)) + '" (first at line ' + (typeof v.existing_line === 'number' ? String(v.existing_line) : '?') + ')';
    default:
      return 'Diagnostic: ' + escapeHtml(firstKey);
  }
}

// ---------------------------------------------------------------------------
// Pure: op selection (item 4 matrix)
// ---------------------------------------------------------------------------

/**
 * The backend op for an edit, by (cell-kind × scope). Pure, total.
 *
 *   | cell | edit-all OFF             | edit-all ON        |
 *   |------|-------------------------|--------------------|
 *   | node | rename_node_occurrence  | rename_node        |
 *   | rel  | relabel_edge            | relabel_rel_all    |
 */
export function cmEditOp(cell: CmCell, editAll: boolean): CmEditOp {
  if (cell === 'rel') return editAll ? 'relabel_rel_all' : 'relabel_edge';
  return editAll ? 'rename_node' : 'rename_node_occurrence';
}

/** Map a frontend node cell to the backend endpoint literal (rename_node_occurrence). */
export function cmCellEndpoint(cell: 'from' | 'to'): 'source' | 'target' {
  return cell === 'from' ? 'source' : 'target';
}

// ---------------------------------------------------------------------------
// Exported render functions
// ---------------------------------------------------------------------------

export function renderDiagram(opts: CmDiagramOpts): void {
  const { container, cm: cmData, focusKey, dotAvailable, seq, getCurrentSeq } = opts;

  if (container === null) return;

  const dotText: string = cmGraphToDot(cmData, focusKey);

  if (!dotAvailable) {
    container.innerHTML = '<p class="error">Graphviz not available.</p><pre>' + escapeHtml(dotText) + '</pre>';
    return;
  }

  container.innerHTML = '<p class="loading">Rendering diagram…</p>';

  renderDot(dotText)
    .then((svgText: string) => {
      if (getCurrentSeq !== undefined && seq !== getCurrentSeq()) return;
      const clean = DOMPurify.sanitize(svgText, { USE_PROFILES: { svg: true } });
      container.innerHTML = clean;
      const svgEl: SVGSVGElement | null = container.querySelector('svg');
      if (svgEl !== null) {
        injectHitRects(svgEl);
        const handlerOpts: SvgHandlerOpts = {
          extractId: (g: SVGGElement): string | null => {
            const t = g.querySelector('title');
            if (t === null) return null;
            return t.textContent.trim();
          },
          onClick: opts.onClick ?? ((_key: string): void => { void _key; }),
          onHoverEnter: (key: string): void => {
            renderCmHoverPane(key, cmData);
            opts.onHoverEnter?.(key);
          },
          onHoverLeave: (): void => {
            renderCmHoverPane(null, cmData);
            opts.onHoverLeave?.();
          },
        };
        wireHandlers(svgEl, handlerOpts);
        // Zoom/pan + fit-to-viewport (IMP-100). Fit-on-load, no persistence —
        // concept-map focus churn doesn't carry a viewport, so onViewportChange
        // is omitted.
        mountZoomPan(container, svgEl, {});
      }
    })
    .catch(() => {
      if (getCurrentSeq !== undefined && seq !== getCurrentSeq()) return;
      container.innerHTML = '<p class="error">Graphviz not available</p>';
    });
}

export function renderEdgeTable(opts: CmEdgeTableOpts): void {
  const { container, cm: cmData, focusKey, depth, editAll, editingCell } = opts;

  if (container === null) return;

  if (cmData === null) {
    container.innerHTML = '';
    container.classList.add('u-hidden');
    return;
  }

  container.classList.remove('u-hidden');

  // No edit MODE: the table always shows the cmFocus neighbourhood when a node
  // is focused, else every edge (item 4 Revision 2 — editing is per-cell inline).
  let edges = cmData.edges;
  if (focusKey !== null && focusKey !== '') {
    edges = cmNeighbourhood(cmData, focusKey, depth).edges;
  }

  // The active inline cell matches by full edge labels + segment. Place the
  // input on the FIRST matching row only (identical triples can recur).
  let inputPlaced = false;
  const isEditing = (edge: CmEdge, cell: CmCell): boolean =>
    !inputPlaced && editingCell !== null
    && editingCell.cell === cell
    && edge.from_label === editingCell.from_label
    && edge.rel === editingCell.rel
    && edge.to_label === editingCell.to_label;

  const cellHtml = (edge: CmEdge, cell: CmCell, label: string): string => {
    if (isEditing(edge, cell)) {
      inputPlaced = true;
      return '<input type="text" class="cm-cell-input" value="' + escapeAttr(label) + '">';
    }
    const nodeCls = cell === 'rel' ? '' : ' cm-edge-node';
    return '<span class="cm-edge-cell' + nodeCls + '" data-cell="' + cell + '">'
      + escapeHtml(label) + '</span>'
      + '<button class="cm-pencil" data-cell="' + cell + '" title="Edit">✎</button>';
  };

  let html = '<label class="cm-edit-all"><input type="checkbox" class="cm-edit-all-cb"'
    + (editAll ? ' checked' : '') + '> edit all</label>';
  html += '<table class="cm-edges"><thead><tr><th>Source</th><th>Relation</th><th>Target</th><th></th></tr></thead><tbody>';

  if (edges.length === 0) {
    html += '<tr><td colspan="4"><span class="placeholder">No edges</span></td></tr>';
  } else {
    for (const edge of edges) {
      html += '<tr class="cm-edge-row">'
        + '<td>' + cellHtml(edge, 'from', edge.from_label) + '</td>'
        + '<td>' + cellHtml(edge, 'rel', edge.rel) + '</td>'
        + '<td>' + cellHtml(edge, 'to', edge.to_label) + '</td>'
        + '<td><button class="cm-remove-btn" data-source="' + escapeAttr(edge.from_label)
        + '" data-rel="' + escapeAttr(edge.rel) + '" data-target="' + escapeAttr(edge.to_label)
        + '" title="Remove edge">✕</button></td>'
        + '</tr>';
    }
  }
  html += '</tbody></table>';

  container.innerHTML = html;

  // Edit-all scope checkbox.
  const editAllCb: HTMLInputElement | null = container.querySelector('.cm-edit-all-cb');
  editAllCb?.addEventListener('change', () => {
    opts.onToggleEditAll?.(editAllCb.checked);
  });

  // Per-row wiring — map each row back to its source edge by index.
  const rows: NodeListOf<Element> = container.querySelectorAll('.cm-edge-row');
  rows.forEach((row, i) => {
    const edge = edges[i];
    if (edge === undefined) return;

    // Pencils → begin inline edit.
    const pencils: NodeListOf<Element> = row.querySelectorAll('.cm-pencil');
    for (const p of pencils) {
      p.addEventListener('click', () => {
        const cell = p.getAttribute('data-cell');
        if (cell === 'from' || cell === 'rel' || cell === 'to') opts.onPencil?.(edge, cell);
      });
    }

    // Plain node-cell click → cmFocus (relation cell is inert).
    const nodeCells: NodeListOf<Element> = row.querySelectorAll('.cm-edge-node');
    for (const c of nodeCells) {
      c.addEventListener('click', () => {
        const cell = c.getAttribute('data-cell');
        if (cell === 'from' || cell === 'to') opts.onNodeClick?.(edge, cell);
      });
    }

    // Remove edge.
    const rm = row.querySelector('.cm-remove-btn');
    rm?.addEventListener('click', () => {
      opts.onRemoveEdge?.(
        rm.getAttribute('data-source') ?? '',
        rm.getAttribute('data-rel') ?? '',
        rm.getAttribute('data-target') ?? '',
      );
    });
  });

  // Inline input — Enter commits, Esc cancels.
  const input: HTMLInputElement | null = container.querySelector('.cm-cell-input');
  if (input !== null) {
    input.focus();
    input.addEventListener('keydown', (ev: KeyboardEvent) => {
      if (ev.key === 'Enter') {
        ev.preventDefault();
        opts.onSubmitEdit?.(input.value);
      } else if (ev.key === 'Escape') {
        ev.preventDefault();
        opts.onCancelEdit?.();
      }
    });
  }
}

export function renderDiagnostics(opts: { container: HTMLElement | null; diagnostics: unknown[] }): void {
  const { container, diagnostics } = opts;
  if (container === null) return;
  if (diagnostics.length === 0) {
    container.classList.add('u-hidden');
    return;
  }
  let html = '<h3 class="cm-diagnostics-panel__title">Diagnostics</h3>';
  for (const d of diagnostics) {
    const msg: string = formatDiagnostic(d);
    const line: number | null = diagnosticLine(d);
    const prefix: string = line !== null ? 'line ' + String(line) + ': ' : '';
    html += '<div class="cm-diag-item">⚠ ' + escapeHtml(prefix + msg) + '</div>';
  }
  container.innerHTML = html;
  container.classList.remove('u-hidden');
}

export function renderAddEdgeForm(opts: CmAddEdgeFormOpts): void {
  const { container, cm: cmData } = opts;
  if (container === null) return;
  container.classList.remove('u-hidden');

  const labels: string[] = buildNodeLabelList(cmData);
  const rels: string[] = buildRelLabelList(cmData);

  let html = '<form class="add-edge-form" onsubmit="return false;"><div class="add-edge-fields">';
  html += '<input type="text" class="cm-input cm-source" list="cm-source-list" placeholder="Source">';
  html += '<datalist id="cm-source-list">' + labels.map((l: string): string => '<option value="' + escapeAttr(l) + '">').join('') + '</datalist>';
  html += '<input type="text" class="cm-input cm-rel" list="cm-rel-list" placeholder="relation">';
  html += '<datalist id="cm-rel-list">' + rels.map((r: string): string => '<option value="' + escapeAttr(r) + '">').join('') + '</datalist>';
  html += '<input type="text" class="cm-input cm-target" list="cm-target-list" placeholder="Target">';
  html += '<datalist id="cm-target-list">' + labels.map((l: string): string => '<option value="' + escapeAttr(l) + '">').join('') + '</datalist>';
  html += '<button type="submit" class="cm-add-btn">Add edge</button></div><div class="cm-add-error u-hidden"></div></form>';

  container.innerHTML = html;

  const form: HTMLFormElement | null = container.querySelector('.add-edge-form');
  if (form !== null) {
    const sourceInput: HTMLInputElement | null = form.querySelector('.cm-source');
    const relInput: HTMLInputElement | null = form.querySelector('.cm-rel');
    const targetInput: HTMLInputElement | null = form.querySelector('.cm-target');
    form.addEventListener('submit', () => {
      opts.onSubmit?.(sourceInput?.value ?? '', relInput?.value ?? '', targetInput?.value ?? '');
    });
  }
}

