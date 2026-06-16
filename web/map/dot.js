/* dot.js — DOT generation for Doctrine Map frontend */
/* global state */

var dot = {};

dot.dotQuote = function(s) {
  return s.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n');
};

dot.nodeAttrs = function(node, focusId, depth) {
  var kind = node.kindPrefix;
  var isFocus = (node.id === focusId);

  var fill, font, shape;

  // Kind colour → Graphviz fill/font mapping (design §2)
  switch (kind) {
    case 'SL':
      fill = '#4A90D9'; font = '#ffffff'; shape = 'box,rounded'; break;
    case 'ADR': case 'POL':
      fill = '#7B4FBF'; font = '#ffffff'; shape = 'box'; break;
    case 'STD':
      fill = '#9B59B6'; font = '#ffffff'; shape = 'box'; break;
    case 'PRD': case 'SPEC':
      fill = '#E67E22'; font = '#222222'; shape = 'box,rounded'; break;
    case 'REQ':
      fill = '#F39C12'; font = '#222222'; shape = 'box'; break;
    case 'ISS': case 'IMP': case 'CHR': case 'RSK':
      fill = '#C0392B'; font = '#ffffff'; shape = 'box'; break;
    case 'IDE':
      fill = '#27AE60'; font = '#222222'; shape = 'box'; break;
    case 'RV':
      fill = '#1ABC9C'; font = '#222222'; shape = 'box'; break;
    case 'REC':
      fill = '#95A5A6'; font = '#222222'; shape = 'box'; break;
    case 'ASM': case 'DEC':
      fill = '#3498DB'; font = '#ffffff'; shape = 'box'; break;
    case 'QUE': case 'CON':
      fill = '#8E44AD'; font = '#ffffff'; shape = 'box'; break;
    case 'REV':
      fill = '#A04000'; font = '#ffffff'; shape = 'box'; break;
    default:
      fill = '#95A5A6'; font = '#222222'; shape = 'box'; break;
  }

  return {
    label: node.id,
    fillcolor: fill,
    fontcolor: font,
    shape: shape,
    penwidth: isFocus ? 3.0 : 1.0,
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
    var attrs = dot.nodeAttrs(node, focusId, depth);
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
