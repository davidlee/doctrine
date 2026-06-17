/* dot.js — DOT generation for Doctrine Map frontend */
/* global state */

var dot = {};

dot.dotQuote = function(s) {
  return s.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n');
};

dot.NODE_STYLES = {
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
  CM:  { fill: '#16A085', font: '#ffffff', shape: 'box' }
};
dot.DEFAULT_NODE_STYLE = { fill: '#95A5A6', font: '#222222', shape: 'box' };

dot.nodeAttrs = function(node, focusId) {
  var s = dot.NODE_STYLES[node.kindPrefix] || dot.DEFAULT_NODE_STYLE;
  return {
    label: node.id,
    fillcolor: s.fill,
    fontcolor: s.font,
    shape: s.shape,
    penwidth: (node.id === focusId) ? 3.0 : 1.0,
    tooltip: node.id + ': ' + node.title + ' \u00b7 ' + (node.kindLabel || node.kindPrefix) + ' \u00b7 ' + node.status
  };
};

dot._EDGE_COLORS = {
  'depends':    { color: '#aaaaaa', fontcolor: '#aaaaaa' },
  'requires':   { color: '#aaaaaa', fontcolor: '#aaaaaa' },
  'refines':    { color: '#4A90D9', fontcolor: '#2563eb' },
  'details':    { color: '#4A90D9', fontcolor: '#2563eb' },
  'implements': { color: '#27AE60', fontcolor: '#166534' },
  'satisfies':  { color: '#27AE60', fontcolor: '#166534' },
  'blocks':     { color: '#C0392B', fontcolor: '#991b1b' },
  'parent':     { color: '#7B4FBF', fontcolor: '#6d28d9' },
  'child':      { color: '#7B4FBF', fontcolor: '#6d28d9' },
  'related':    { color: '#95A5A6', fontcolor: '#64748b' },
  'see also':   { color: '#95A5A6', fontcolor: '#64748b' },
  'resolves':   { color: '#E67E22', fontcolor: '#c2410c' },
  'addresses':  { color: '#E67E22', fontcolor: '#c2410c' }
};

dot.edgeAttrs = function(edge, depth) {
  // Edge colour by exact canonical label (design §2 / SL-073 Hard Contracts).
  // Labels are normalized by the backend — they're a controlled vocabulary.
  var key = edge.label.toLowerCase();
  var entry = dot._EDGE_COLORS[key] || { color: '#888888', fontcolor: '#555555' };

  return {
    label: edge.label,
    tooltip: edge.id,
    color: entry.color,
    fontcolor: entry.fontcolor
  };
};

dot.graphToDot = function(neighbourhood, focusId, depth) {
  var lines = [];
  lines.push('digraph G {');
  lines.push('  rankdir=LR;');
  lines.push('  bgcolor="transparent";');
  lines.push('  nodesep=0.45;');
  lines.push('  ranksep=0.8;');
  lines.push('');

  // Sort node ids for determinism
  var sortedIds = [];
  neighbourhood.nodes.forEach(function(_, id) {
    sortedIds.push(id);
  });
  sortedIds.sort();

  // Node statements
  sortedIds.forEach(function(id) {
    var node = state.graph.nodes.get(id);
    if (!node) return;
    var attrs = dot.nodeAttrs(node, focusId);
    lines.push('  "' + dot.dotQuote(id) + '" [' +
      'label="' + dot.dotQuote(attrs.label) + '", ' +
      'style="filled", ' +
      'fillcolor="' + attrs.fillcolor + '", ' +
      'fontcolor="' + attrs.fontcolor + '", ' +
      'shape="' + attrs.shape + '", ' +
      'penwidth=' + attrs.penwidth + ', ' +
      'tooltip="' + dot.dotQuote(attrs.tooltip) + '"' +
      '];');
  });

  // Edge statements — sort by edge id for determinism
  var sortedEdges = neighbourhood.edges.slice().sort(function(a, b) {
    return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
  });

  sortedEdges.forEach(function(edge) {
    var attrs = dot.edgeAttrs(edge, depth);
    lines.push('  "' + dot.dotQuote(edge.source) + '" -> "' + dot.dotQuote(edge.target) + '" [' +
      'label="' + dot.dotQuote(attrs.label) + '", ' +
      'color="' + attrs.color + '", ' +
      'fontcolor="' + attrs.fontcolor + '", ' +
      'tooltip="' + dot.dotQuote(attrs.tooltip) + '"' +
      '];');
  });

  lines.push('}');
  return lines.join('\n');
};

/* --- concept map DOT generation --- */

dot.escapeStringContent = function(s) {
  return s.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n')
    .replace(/>/g, '\\>').replace(/]/g, '\\]').replace(/}/g, '\\}');
};

dot.cmGraphToDot = function(cm, focusKey) {
  var lines = [];
  lines.push('digraph concept_map {');
  lines.push('  rankdir=LR;');
  lines.push('  bgcolor="transparent";');
  lines.push('  nodesep=0.45;');
  lines.push('  ranksep=0.8;');
  lines.push('  node [shape=record, style="filled,rounded", fillcolor="#f8f9fa", color="#4A90D9", fontcolor="#222222", penwidth=1.5];');
  lines.push('  edge [color="#4A90D9", fontcolor="#4A90D9"];');
  lines.push('');

  var sortedNodes = (cm.nodes || []).slice().sort(function(a, b) {
    return a.key < b.key ? -1 : a.key > b.key ? 1 : 0;
  });
  sortedNodes.forEach(function(node) {
    var extra = (focusKey && node.key === focusKey) ? ', penwidth=3.0' : '';
    lines.push('  "' + dot.escapeStringContent(node.key) + '" [label="' + dot.escapeStringContent(node.label) + '"' + extra + '];');
  });

  lines.push('');

  (cm.edges || []).forEach(function(edge) {
    lines.push('  "' + dot.escapeStringContent(edge.from_key) + '" -> "' + dot.escapeStringContent(edge.to_key) +
      '" [label="' + dot.escapeStringContent(edge.rel) + '"];');
  });

  lines.push('}');
  return lines.join('\n');
};
