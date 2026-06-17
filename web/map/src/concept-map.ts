// Doctrine Map Explorer — concept map rendering (SL-091 PHASE-08)
// TypeScript rewrite of concept-map.js.
// Depends on: model (cmNeighbourhood, buildNodeLabelList, buildRelLabelList),
// dot (cmGraphToDot), api (renderDot), svg (injectHitRects, wireHandlers),
// render (escapeHtml, escapeAttr).

import type { ConceptMap } from './types';
import { cmNeighbourhood, buildNodeLabelList, buildRelLabelList } from './model';
import { cmGraphToDot } from './dot';
import { renderDot } from './api';
import { injectHitRects, wireHandlers, type SvgHandlerOpts } from './svg';
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
  onRemoveEdge?: (source: string, rel: string, target: string) => void;
  onRenameNode?: (key: string) => void;
  onSubmitRename?: (label: string) => void;
  onCancelRename?: () => void;
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
      }
    })
    .catch(() => {
      if (getCurrentSeq !== undefined && seq !== getCurrentSeq()) return;
      container.innerHTML = '<p class="error">Graphviz not available</p>';
    });
}

export function renderEdgeTable(opts: CmEdgeTableOpts): void {
  const { container, cm: cmData, focusKey, depth, editing, editingNode } = opts;

  if (container === null) return;

  if (cmData === null) {
    container.innerHTML = '';
    container.style.display = 'none';
    return;
  }

  container.style.display = 'block';

  let edges = cmData.edges;
  if (!editing && focusKey !== null && focusKey !== '') {
    const filtered = cmNeighbourhood(cmData, focusKey, depth);
    edges = filtered.edges;
  }

  const editingKey: string | null = editingNode !== null ? editingNode.key : null;
  const editingLabel: string = editingNode !== null ? editingNode.label : '';

  let html = '<table class="cm-edges"><thead><tr><th>Source</th><th>Relation</th><th>Target</th>';
  if (editing) html += '<th></th>';
  html += '</tr></thead><tbody>';

  if (edges.length === 0) {
    html += '<tr><td colspan="' + (editing ? '4' : '3') + '"><span class="placeholder">No edges</span></td></tr>';
  } else {
    for (const edge of edges) {
      html += '<tr class="cm-edge-row"><td>';
      if (editingKey !== null && edge.from_key === editingKey && editing) {
        html += '<input type="text" class="cm-rename-input" data-key="' + escapeAttr(editingKey) + '" value="' + escapeAttr(editingLabel) + '">';
      } else {
        html += '<span class="cm-edge-label' + (editing ? ' cm-editable-node" data-key="' + escapeAttr(edge.from_key) + '" data-label="' + escapeAttr(edge.from_label) : '') + '">' + escapeHtml(edge.from_label) + '</span>';
      }
      html += '</td><td>' + escapeHtml(edge.rel) + '</td><td>';
      if (editingKey !== null && edge.to_key === editingKey && editing) {
        html += '<input type="text" class="cm-rename-input" data-key="' + escapeAttr(editingKey) + '" value="' + escapeAttr(editingLabel) + '">';
      } else {
        html += '<span class="cm-edge-label' + (editing ? ' cm-editable-node" data-key="' + escapeAttr(edge.to_key) + '" data-label="' + escapeAttr(edge.to_label) : '') + '">' + escapeHtml(edge.to_label) + '</span>';
      }
      html += '</td>';
      if (editing) {
        html += '<td><button class="cm-remove-btn" data-source="' + escapeAttr(edge.from_label) + '" data-rel="' + escapeAttr(edge.rel) + '" data-target="' + escapeAttr(edge.to_label) + '" title="Remove edge">✕</button></td>';
      }
      html += '</tr>';
    }
  }
  html += '</tbody></table>';

  container.innerHTML = html;

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
  }
}

export function renderDiagnostics(opts: { container: HTMLElement | null; diagnostics: unknown[] }): void {
  const { container, diagnostics } = opts;
  if (container === null) return;
  if (diagnostics.length === 0) {
    container.style.display = 'none';
    return;
  }
  let html = '<h3>Diagnostics</h3>';
  for (const d of diagnostics) {
    const msg: string = formatDiagnostic(d);
    const line: number | null = diagnosticLine(d);
    const prefix: string = line !== null ? 'line ' + String(line) + ': ' : '';
    html += '<div class="cm-diag-item">⚠ ' + escapeHtml(prefix + msg) + '</div>';
  }
  container.innerHTML = html;
  container.style.display = 'block';
}

export function renderAddEdgeForm(opts: CmAddEdgeFormOpts): void {
  const { container, cm: cmData, editing } = opts;
  if (container === null) return;
  if (!editing) {
    container.style.display = 'none';
    return;
  }
  container.style.display = 'block';

  const labels: string[] = buildNodeLabelList(cmData);
  const rels: string[] = buildRelLabelList(cmData);

  let html = '<form class="add-edge-form" onsubmit="return false;"><div class="add-edge-fields">';
  html += '<input type="text" class="cm-input cm-source" list="cm-source-list" placeholder="Source">';
  html += '<datalist id="cm-source-list">' + labels.map((l: string): string => '<option value="' + escapeAttr(l) + '">').join('') + '</datalist>';
  html += '<input type="text" class="cm-input cm-rel" list="cm-rel-list" placeholder="relation">';
  html += '<datalist id="cm-rel-list">' + rels.map((r: string): string => '<option value="' + escapeAttr(r) + '">').join('') + '</datalist>';
  html += '<input type="text" class="cm-input cm-target" list="cm-target-list" placeholder="Target">';
  html += '<datalist id="cm-target-list">' + labels.map((l: string): string => '<option value="' + escapeAttr(l) + '">').join('') + '</datalist>';
  html += '<button type="submit" class="cm-add-btn">Add edge</button></div><div class="cm-add-error" style="display:none;"></div></form>';

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

export function renderEditToggle(opts: { header: HTMLElement | null; editing: boolean; onToggle?: () => void }): void {
  const { header, editing } = opts;
  if (header === null) return;

  const existing = header.querySelector('.cm-edit-toggle');
  if (existing !== null) existing.remove();

  const btn = document.createElement('button');
  btn.className = 'cm-edit-toggle';
  btn.textContent = editing ? 'Done' : 'Edit';
  btn.addEventListener('click', () => {
    opts.onToggle?.();
  });
  header.appendChild(btn);
}
