import type { CatalogNode, Edge, Neighbourhood, CmNeighbourhood } from './types';
import { state } from './state';

/* ------------------------------------------------------------------ */
/*  String escaping                                                   */
/* ------------------------------------------------------------------ */

export function dotQuote(s: string): string {
  return s.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n');
}

export function escapeStringContent(s: string): string {
  return s
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/>/g, '\\>')
    .replace(/]/g, '\\]')
    .replace(/}/g, '\\}');
}

/* ------------------------------------------------------------------ */
/*  Style constants                                                   */
/* ------------------------------------------------------------------ */

export const NODE_STYLES: Record<string, { fill: string; font: string; shape: string }> = {
  SL:  { fill: '#4A90D9', font: '#ffffff', shape: 'box,rounded' },
  ADR: { fill: '#7B4FBF', font: '#ffffff', shape: 'box' },
  POL: { fill: '#7B4FBF', font: '#ffffff', shape: 'box' },
  STD: { fill: '#9B59B6', font: '#ffffff', shape: 'box' },
  PRD: { fill: '#E67E22', font: '#222222', shape: 'box,rounded' },
  SPEC:{ fill: '#E67E22', font: '#222222', shape: 'box,rounded' },
  REQ: { fill: '#F39C12', font: '#222222', shape: 'box' },
  ISS: { fill: '#C0392B', font: '#ffffff', shape: 'box' },
  IMP: { fill: '#C0392B', font: '#ffffff', shape: 'box' },
  CHR: { fill: '#C0392B', font: '#ffffff', shape: 'box' },
  RSK: { fill: '#C0392B', font: '#ffffff', shape: 'box' },
  IDE: { fill: '#27AE60', font: '#222222', shape: 'box' },
  RV:  { fill: '#1ABC9C', font: '#222222', shape: 'box' },
  REC: { fill: '#95A5A6', font: '#222222', shape: 'box' },
  ASM: { fill: '#3498DB', font: '#ffffff', shape: 'box' },
  DEC: { fill: '#3498DB', font: '#ffffff', shape: 'box' },
  QUE: { fill: '#8E44AD', font: '#ffffff', shape: 'box' },
  CON: { fill: '#8E44AD', font: '#ffffff', shape: 'box' },
  REV: { fill: '#A04000', font: '#ffffff', shape: 'box' },
  CM:  { fill: '#16A085', font: '#ffffff', shape: 'box' },
};

export const DEFAULT_NODE_STYLE: { fill: string; font: string; shape: string } = {
  fill: '#95A5A6',
  font: '#222222',
  shape: 'box',
};

export const EDGE_STYLES: Record<string, { color: string; style?: string }> = {};

const _EDGE_COLORS: Record<string, { color: string; fontcolor: string }> = {
  specs:          { color: '#4A90D9', fontcolor: '#2563eb' },
  requirements:   { color: '#4A90D9', fontcolor: '#2563eb' },
  descends_from:  { color: '#4A90D9', fontcolor: '#2563eb' },
  parent:         { color: '#4A90D9', fontcolor: '#2563eb' },
  members:        { color: '#4A90D9', fontcolor: '#2563eb' },
  supersedes:     { color: '#E67E22', fontcolor: '#c2410c' },
  revises:        { color: '#E67E22', fontcolor: '#c2410c' },
  governed_by:    { color: '#7B4FBF', fontcolor: '#6d28d9' },
  related:        { color: '#7B4FBF', fontcolor: '#6d28d9' },
  decision_ref:   { color: '#7B4FBF', fontcolor: '#6d28d9' },
  consumes:        { color: '#27AE60', fontcolor: '#166534' },
  interactions:    { color: '#27AE60', fontcolor: '#166534' },
  contextualizes:  { color: '#27AE60', fontcolor: '#166534' },
  slices:         { color: '#16A085', fontcolor: '#0f766e' },
  owning_slice:   { color: '#16A085', fontcolor: '#0f766e' },
  reviews:        { color: '#64748b', fontcolor: '#475569' },
  drift:          { color: '#C0392B', fontcolor: '#991b1b' },
};

/* ------------------------------------------------------------------ */
/*  Node / Edge attribute builders                                    */
/* ------------------------------------------------------------------ */

interface NodeAttrs {
  label: string;
  fillcolor: string;
  fontcolor: string;
  shape: string;
  penwidth: number;
  tooltip: string;
}

export function nodeAttrs(node: CatalogNode, focusId: string | null): NodeAttrs {
  const s = NODE_STYLES[node.kindPrefix] ?? DEFAULT_NODE_STYLE;
  return {
    label: node.id,
    fillcolor: s.fill,
    fontcolor: s.font,
    shape: s.shape,
    penwidth: node.id === focusId ? 3.0 : 1.0,
    tooltip: `${node.id}: ${node.title} \u00b7 ${node.kindLabel !== '' ? node.kindLabel : node.kindPrefix} \u00b7 ${node.status}`,
  };
}

interface EdgeAttrs {
  label: string;
  tooltip: string;
  color: string;
  fontcolor: string;
}

export function edgeAttrs(edge: Edge, _depth: number): EdgeAttrs {
  void _depth; // retained for API compatibility, not used internally
  const entry = _EDGE_COLORS[edge.label.toLowerCase()] ?? { color: '#888888', fontcolor: '#555555' };
  return {
    label: edge.label,
    tooltip: edge.id,
    color: entry.color,
    fontcolor: entry.fontcolor,
  };
}

/* ------------------------------------------------------------------ */
/*  Entity-graph DOT generation                                       */
/* ------------------------------------------------------------------ */

export function graphToDot(nb: Neighbourhood, focusId: string | null, _depth: number): string {
  const lines: string[] = [];
  lines.push('digraph G {');
  lines.push('  rankdir=LR;');
  lines.push('  bgcolor="transparent";');
  lines.push('  nodesep=0.45;');
  lines.push('  ranksep=0.8;');
  lines.push('');

  // Nodes — sorted by id for determinism
  const sortedIds = Array.from(nb.nodes).sort();

  for (const id of sortedIds) {
    const node = state.graph.nodes.get(id);
    if (node === undefined) continue;
    const attrs = nodeAttrs(node, focusId);
    lines.push(
      `  "${dotQuote(id)}" [` +
      `label="${dotQuote(attrs.label)}", ` +
      `style="filled", ` +
      `fillcolor="${attrs.fillcolor}", ` +
      `fontcolor="${attrs.fontcolor}", ` +
      `shape="${attrs.shape}", ` +
      `penwidth=${String(attrs.penwidth)}, ` +
      `tooltip="${dotQuote(attrs.tooltip)}"` +
      `];`,
    );
  }

  // Edges — sorted by edge id for determinism
  const sortedEdges = [...nb.edges].sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));

  for (const edge of sortedEdges) {
    const attrs = edgeAttrs(edge, _depth);
    lines.push(
      `  "${dotQuote(edge.source)}" -> "${dotQuote(edge.target)}" [` +
      `label="${dotQuote(attrs.label)}", ` +
      `color="${attrs.color}", ` +
      `fontcolor="${attrs.fontcolor}", ` +
      `tooltip="${dotQuote(attrs.tooltip)}"` +
      `];`,
    );
  }

  lines.push('}');
  return lines.join('\n');
}

/* ------------------------------------------------------------------ */
/*  Concept-map DOT generation                                        */
/* ------------------------------------------------------------------ */

export function cmGraphToDot(cm: CmNeighbourhood, focusKey: string | null): string {
  const lines: string[] = [];
  lines.push('digraph concept_map {');
  lines.push('  rankdir=LR;');
  lines.push('  bgcolor="transparent";');
  lines.push('  nodesep=0.45;');
  lines.push('  ranksep=0.8;');
  lines.push(
    '  node [shape=record, style="filled,rounded", ' +
    'fillcolor="#f8f9fa", color="#4A90D9", fontcolor="#222222"];',
  );
  lines.push('  edge [color="#4A90D9", fontcolor="#4A90D9"];');
  lines.push('');

  const sortedNodes = [...cm.nodes].sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));

  for (const node of sortedNodes) {
    const extra = (focusKey !== null && node.key === focusKey) ? ', penwidth=3.0' : '';
    lines.push(
      `  "${escapeStringContent(node.key)}" ` +
      `[label="${escapeStringContent(node.label)}"${extra}];`,
    );
  }

  lines.push('');

  for (const edge of cm.edges) {
    lines.push(
      `  "${escapeStringContent(edge.from_key)}" -> "${escapeStringContent(edge.to_key)}" ` +
      `[label="${escapeStringContent(edge.rel)}"];`,
    );
  }

  lines.push('}');
  return lines.join('\n');
}
