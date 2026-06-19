// Doctrine Map Explorer — concept map rendering (SL-091 PHASE-08)
// TypeScript rewrite of concept-map.js.
// Depends on: model (cmNeighbourhood, buildNodeLabelList, buildRelLabelList),
// dot (cmGraphToDot), api (renderDot), svg (injectHitRects, wireHandlers),
// render (escapeHtml, escapeAttr).

import type { ConceptMap, CmEdge, CmSelectedField } from './types';
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
  editing: boolean;
  editingNode: { key: string; label: string } | null;
  /** The cell selected by a single click (plain navigation affordance). */
  selectedField?: CmSelectedField | null;
  /** Which selected field is being inline-edited; renders a single input, independent of `editing`. */
  editingField?: 'node' | 'rel' | null;
  onRemoveEdge?: (source: string, rel: string, target: string) => void;
  onRenameNode?: (key: string) => void;
  onSubmitRename?: (label: string) => void;
  onCancelRename?: () => void;
  /** A cell was single-clicked: (edge, which cell). Plain selection, no input. */
  onSelectCell?: (edge: CmEdge, cell: 'from' | 'rel' | 'to') => void;
  /** Inline single-field commit (Enter) for the 'rel' relabel path. */
  onSubmitRelabel?: (value: string) => void;
}

export interface CmAddEdgeFormOpts {
  container: HTMLElement | null;
  cm: ConceptMap | null;
  editing: boolean;
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
// Pure: cell-selection
// ---------------------------------------------------------------------------

/**
 * Build the label-carrying CmSelectedField from a clicked edge + cell.
 *
 * Pure. Captures the clicked cell's LABEL as it appears on the edge — distinct
 * labels can derive the same key (`User Story` vs `User-Story`), and every CM
 * mutation is label-based, so the label (not the key) is what a rename/relabel
 * submits. A node cell additionally retains `key` for the `cmFocus` highlight.
 */
export function cmSelectedFieldFromCell(edge: CmEdge, cell: 'from' | 'rel' | 'to'): CmSelectedField {
  if (cell === 'rel') {
    return { kind: 'rel', from_label: edge.from_label, rel: edge.rel, to_label: edge.to_label };
  }
  if (cell === 'to') {
    return { kind: 'node', key: edge.to_key, label: edge.to_label };
  }
  return { kind: 'node', key: edge.from_key, label: edge.from_label };
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
  const { container, cm: cmData, focusKey, depth, editing, editingNode } = opts;
  const selectedField: CmSelectedField | null = opts.selectedField ?? null;
  const editingField: 'node' | 'rel' | null = opts.editingField ?? null;

  if (container === null) return;

  if (cmData === null) {
    container.innerHTML = '';
    container.classList.add('u-hidden');
    return;
  }

  container.classList.remove('u-hidden');

  let edges = cmData.edges;
  if (!editing && focusKey !== null && focusKey !== '') {
    const filtered = cmNeighbourhood(cmData, focusKey, depth);
    edges = filtered.edges;
  }

  const editingKey: string | null = editingNode !== null ? editingNode.key : null;
  const editingLabel: string = editingNode !== null ? editingNode.label : '';

  // Single-field ('Edit this') arm — independent of `editing`. A node cell is
  // edited via the existing rename input (gated on editingNode + editingField),
  // a rel cell via a dedicated relabel input matching the selected edge.
  const editingNodeCell: boolean = editingField === 'node';
  const selNode = selectedField !== null && selectedField.kind === 'node' ? selectedField : null;
  const selRel = selectedField !== null && selectedField.kind === 'rel' ? selectedField : null;
  const editingRelCell: boolean = editingField === 'rel' && selRel !== null;

  // A given rel cell is edited only on the FIRST edge matching the selected rel
  // identity (a single input, even if the same triple recurs).
  let relInputPlaced = false;
  const isSelectedRelEdge = (edge: CmEdge): boolean =>
    selRel !== null &&
    edge.from_label === selRel.from_label &&
    edge.rel === selRel.rel &&
    edge.to_label === selRel.to_label;

  // Selection affordance (no input): mark the selected cell(s).
  const fromSelected = (edge: CmEdge): boolean => selNode !== null && edge.from_key === selNode.key && edge.from_label === selNode.label;
  const toSelected = (edge: CmEdge): boolean => selNode !== null && edge.to_key === selNode.key && edge.to_label === selNode.label;

  const nodeCellHtml = (edge: CmEdge, side: 'from' | 'to'): string => {
    const key = side === 'from' ? edge.from_key : edge.to_key;
    const label = side === 'from' ? edge.from_label : edge.to_label;
    const isEditing = (editing || editingNodeCell) && editingKey !== null && key === editingKey
      && (editing || (selNode !== null && label === selNode.label));
    if (isEditing) {
      return '<input type="text" class="cm-rename-input" data-key="' + escapeAttr(editingKey) + '" value="' + escapeAttr(editingLabel) + '">';
    }
    const selectedCls = (side === 'from' ? fromSelected(edge) : toSelected(edge)) ? ' cm-selected' : '';
    const editableCls = editing ? ' cm-editable-node' : '';
    return '<span class="cm-edge-cell cm-edge-label' + editableCls + selectedCls + '"'
      + ' data-cell="' + side + '"'
      + ' data-key="' + escapeAttr(key) + '" data-label="' + escapeAttr(label) + '">'
      + escapeHtml(label) + '</span>';
  };

  let html = '<table class="cm-edges"><thead><tr><th>Source</th><th>Relation</th><th>Target</th>';
  if (editing) html += '<th></th>';
  html += '</tr></thead><tbody>';

  if (edges.length === 0) {
    html += '<tr><td colspan="' + (editing ? '4' : '3') + '"><span class="placeholder">No edges</span></td></tr>';
  } else {
    for (const edge of edges) {
      html += '<tr class="cm-edge-row"><td>';
      html += nodeCellHtml(edge, 'from');
      html += '</td><td>';
      if (editingRelCell && !relInputPlaced && isSelectedRelEdge(edge)) {
        relInputPlaced = true;
        html += '<input type="text" class="cm-relabel-input" value="' + escapeAttr(edge.rel) + '">';
      } else {
        const relSelectedCls = isSelectedRelEdge(edge) && editingField === null ? ' cm-selected' : '';
        html += '<span class="cm-edge-cell cm-edge-rel' + relSelectedCls + '" data-cell="rel">' + escapeHtml(edge.rel) + '</span>';
      }
      html += '</td><td>';
      html += nodeCellHtml(edge, 'to');
      html += '</td>';
      if (editing) {
        html += '<td><button class="cm-remove-btn" data-source="' + escapeAttr(edge.from_label) + '" data-rel="' + escapeAttr(edge.rel) + '" data-target="' + escapeAttr(edge.to_label) + '" title="Remove edge">✕</button></td>';
      }
      html += '</tr>';
    }
  }
  html += '</tbody></table>';

  container.innerHTML = html;

  // Cell selection (plain navigation) — active whenever NOT mid edit-all and not
  // inline-editing a field. Maps each cell back to its source edge by index.
  const wireSelection = !editing && editingField === null;
  if (wireSelection) {
    const rows: NodeListOf<Element> = container.querySelectorAll('.cm-edge-row');
    rows.forEach((row, i) => {
      const edge = edges[i];
      if (edge === undefined) return;
      const cells: NodeListOf<Element> = row.querySelectorAll('.cm-edge-cell');
      for (const cellEl of cells) {
        cellEl.addEventListener('click', () => {
          const cell = cellEl.getAttribute('data-cell');
          if (cell === 'from' || cell === 'rel' || cell === 'to') {
            opts.onSelectCell?.(edge, cell);
          }
        });
      }
    });
  }

  if (editing) {
    const removeBtns: NodeListOf<Element> = container.querySelectorAll('.cm-remove-btn');
    for (const btn of removeBtns) {
      btn.addEventListener('click', () => {
        opts.onRemoveEdge?.(btn.getAttribute('data-source') ?? '', btn.getAttribute('data-rel') ?? '', btn.getAttribute('data-target') ?? '');
      });
    }

    const editableNodes: NodeListOf<Element> = container.querySelectorAll('.cm-editable-node');
    for (const el of editableNodes) {
      el.addEventListener('click', () => {
        opts.onRenameNode?.(el.getAttribute('data-key') ?? '');
      });
    }
  }

  // Inline rename input (node) — Enter commits via onSubmitRename, Esc cancels.
  const renameInputs: NodeListOf<HTMLInputElement> = container.querySelectorAll('.cm-rename-input');
  let isFirst = true;
  for (const inp of renameInputs) {
    if (isFirst) {
      inp.focus();
      isFirst = false;
    }
    inp.addEventListener('keydown', (ev: KeyboardEvent) => {
      if (ev.key === 'Enter') {
        ev.preventDefault();
        opts.onSubmitRename?.(inp.value);
      } else if (ev.key === 'Escape') {
        ev.preventDefault();
        opts.onCancelRename?.();
      }
    });
  }

  // Inline relabel input (rel) — Enter commits via onSubmitRelabel, Esc cancels.
  const relabelInput: HTMLInputElement | null = container.querySelector('.cm-relabel-input');
  if (relabelInput !== null) {
    relabelInput.focus();
    relabelInput.addEventListener('keydown', (ev: KeyboardEvent) => {
      if (ev.key === 'Enter') {
        ev.preventDefault();
        opts.onSubmitRelabel?.(relabelInput.value);
      } else if (ev.key === 'Escape') {
        ev.preventDefault();
        opts.onCancelRename?.();
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
  const { container, cm: cmData, editing } = opts;
  if (container === null) return;
  if (!editing) {
    container.classList.add('u-hidden');
    return;
  }
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

export interface CmEditToggleOpts {
  header: HTMLElement | null;
  /** Edit-all mode active (drives the "Edit all"/"Done" label). */
  editing: boolean;
  /** Is a cell currently selected? Gates the "Edit this" button. */
  hasSelection: boolean;
  /** "Edit this" — inline-edit the selected field. */
  onEditThis?: () => void;
  /** "Edit all" — toggle today's global edit-all mode. */
  onToggle?: () => void;
}

export function renderEditToggle(opts: CmEditToggleOpts): void {
  const { header, editing, hasSelection } = opts;
  if (header === null) return;

  const existing = header.querySelector('.cm-edit-controls');
  if (existing !== null) existing.remove();

  const group = document.createElement('span');
  group.className = 'cm-edit-controls';

  const editThis = document.createElement('button');
  editThis.className = 'cm-edit-toggle cm-edit-this';
  editThis.textContent = 'Edit this';
  editThis.title = 'Edit the selected field';
  editThis.disabled = !hasSelection;
  editThis.addEventListener('click', () => {
    opts.onEditThis?.();
  });
  group.appendChild(editThis);

  const editAll = document.createElement('button');
  editAll.className = 'cm-edit-toggle cm-edit-all';
  editAll.textContent = editing ? 'Done' : 'Edit all';
  editAll.addEventListener('click', () => {
    opts.onToggle?.();
  });
  group.appendChild(editAll);

  header.appendChild(group);
}
