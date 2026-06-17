/* model.js — data normalization & query layer for Doctrine Map frontend */

var state = {
  graphRaw: null,
  graph: {
    nodes: new Map(),
    edges: [],
    incoming: new Map(),
    outgoing: new Map(),
    edgeById: new Map()
  },
  focusId: null,
  depth: 1,
  markdownCache: new Map(),
  conceptMapCache: new Map(),
  editingConceptMap: false,
  editingNode: null,
  cmFocusNode: null,
  renderedCmFocus: null,
  dotAvailable: false,
  hoveredId: null,
  kindFilter: null,
  graphRenderSeq: 0
};

var model = {};

/* --- helpers --- */

function padId(n) {
  return (n < 100 ? (n < 10 ? '00' : '0') : '') + n;
}

function encodePart(s) {
  var result = '';
  for (var i = 0; i < s.length; i++) {
    var c = s.charAt(i);
    if (/[A-Za-z0-9_\-]/.test(c)) {
      result += c;
    } else {
      var hex = c.charCodeAt(0).toString(16);
      if (hex.length === 1) hex = '0' + hex;
      result += '_' + hex;
    }
  }
  return result;
}
model.encodePart = encodePart;

function pascalToSnake(s) {
  return s.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();
}
model.pascalToSnake = pascalToSnake;

function splitPrefix(s) {
  /* canonical id is PREFIX-NNN; split on the last hyphen */
  var lastHyphen = s.lastIndexOf('-');
  if (lastHyphen <= 0) return null;
  var prefix = s.substring(0, lastHyphen);
  var numStr = s.substring(lastHyphen + 1);
  if (!/^[A-Za-z]+$/.test(prefix) || !/^\d+$/.test(numStr)) return null;
  return { prefix: prefix.toUpperCase(), num: parseInt(numStr, 10) };
}

/* --- normalization --- */

model.normalizeGraph = function(raw) {
  var nodes = new Map();
  var edges = [];
  var edgeById = new Map();
  var incoming = new Map();
  var outgoing = new Map();

  /* build nodes */
  Object.keys(raw.nodes).forEach(function(key) {
    var entry = raw.nodes[key];
    var sp = splitPrefix(key);
    var kindPrefix = sp ? sp.prefix : '';
    nodes.set(key, {
      id: key,
      title: entry.title,
      status: entry.status,
      kindPrefix: kindPrefix,
      kindLabel: entry.kind_label || '',
      raw: entry
    });
  });

  /* build edges */
  (raw.edges || []).forEach(function(edge) {
    /* skip unresolved targets — EdgeTarget is { Resolved | UnresolvedRef | UnvalidatedText } */
    if (!edge.target || !('Resolved' in edge.target)) return;

    /* CatalogKey serializes flat to a canonical string ("SL-003"); EdgeTarget::Resolved
       wraps one; CatalogEdgeLabel is tagged { Validated | Raw } (SL-081 catalog graph). */
    var source = edge.source;
    var target = edge.target.Resolved;
    /* RelationLabel serializes as its PascalCase variant name ("OwningSlice"); the
       rest of the system uses the canonical snake_case form ("owning_slice"). */
    var label = edge.label && edge.label.Validated !== undefined ? pascalToSnake(edge.label.Validated)
      : (edge.label && edge.label.Raw !== undefined ? edge.label.Raw : '');

    /* build edge id using nodes-canonical forms */
    var edgeId = 'e_' + encodePart(source) + '_' + encodePart(label) + '_' + encodePart(target);

    /* coalesce duplicates */
    if (edgeById.has(edgeId)) return;

    var edgeObj = {
      id: edgeId,
      source: source,
      label: label,
      target: target,
      raw: edge
    };

    edgeById.set(edgeId, edgeObj);
    edges.push(edgeObj);

    /* incoming */
    if (!incoming.has(target)) incoming.set(target, []);
    incoming.get(target).push(edgeObj);

    /* outgoing */
    if (!outgoing.has(source)) outgoing.set(source, []);
    outgoing.get(source).push(edgeObj);
  });

  state.graph.nodes = nodes;
  state.graph.edges = edges;
  state.graph.edgeById = edgeById;
  state.graph.incoming = incoming;
  state.graph.outgoing = outgoing;
};

/* --- lookup / resolution --- */

model.findFocus = function(query, graph) {
  /* step 1: null/empty → first sorted node */
  if (query === null || query === '') {
    var sortedIds = sortedNodeIds(graph);
    return sortedIds.length > 0 ? sortedIds[0] : null;
  }

  /* step 2: exact canonical match case-insensitive */
  if (graph.nodes.has(query.toUpperCase())) {
    return query.toUpperCase();
  }

  /* step 3: loose canonical */
  var norm = looseCanonical(query);
  if (norm && graph.nodes.has(norm)) {
    return norm;
  }

  /* step 4: exact title match case-insensitive */
  var queryLower = query.toLowerCase();
  var titleMatch = null;
  graph.nodes.forEach(function(node) {
    if (node.title.toLowerCase() === queryLower) {
      titleMatch = node.id;
    }
  });
  if (titleMatch !== null) return titleMatch;

  /* step 5: substring in id, title, status, or kind */
  var best = null;
  graph.nodes.forEach(function(node) {
    var targets = [
      node.id.toLowerCase(),
      node.title.toLowerCase(),
      node.status.toLowerCase(),
      node.kindLabel.toLowerCase()
    ];
    for (var t = 0; t < targets.length; t++) {
      if (targets[t].indexOf(queryLower) !== -1) {
        if (best === null || node.id.length < best.length) {
          best = node.id;
        }
        break;
      }
    }
  });
  if (best !== null) return best;

  /* step 6: no fallback — return null */
  return null;
};

model.resolveFocus = function(query, graph) {
  var result = model.findFocus(query, graph);
  if (result !== null) return result;

  /* fallback to first sorted node */
  var sortedIds = sortedNodeIds(graph);
  return sortedIds.length > 0 ? sortedIds[0] : null;
};

/* --- neighbourhood (BFS) --- */

// Shared BFS core — used by both neighbourhood (entity graph) and
// cmNeighbourhood (concept map). expandNeighbours(id) returns
// [{ nodeId, edge }]; edgeKey(edge) returns a unique dedup key
// (defaults to edge.id).
function bfsCore(startId, maxDepth, expandNeighbours, edgeKey) {
  maxDepth = Math.max(0, Math.min(3, maxDepth));
  if (maxDepth === 0) {
    return { nodes: new Set([startId]), edges: [] };
  }

  var visited = new Set();
  var collectedEdges = [];
  var collectedEdgeKeys = new Set();
  var queue = [{ id: startId, dist: 0 }];
  visited.add(startId);

  while (queue.length > 0) {
    var current = queue.shift();
    if (current.dist >= maxDepth) continue;

    var neighbours = expandNeighbours(current.id);
    for (var i = 0; i < neighbours.length; i++) {
      var nb = neighbours[i];
      if (!visited.has(nb.nodeId)) {
        visited.add(nb.nodeId);
        queue.push({ id: nb.nodeId, dist: current.dist + 1 });
      }
      var key = edgeKey ? edgeKey(nb.edge) : nb.edge.id;
      if (!collectedEdgeKeys.has(key)) {
        collectedEdgeKeys.add(key);
        collectedEdges.push(nb.edge);
      }
    }
  }

  return { nodes: visited, edges: collectedEdges };
}

model.neighbourhood = function(focusId, depth, graph) {
  function expandNeighbours(id) {
    var result = [];
    var outEdges = graph.outgoing.get(id) || [];
    var inEdges = graph.incoming.get(id) || [];
    for (var o = 0; o < outEdges.length; o++) {
      result.push({ nodeId: outEdges[o].target, edge: outEdges[o] });
    }
    for (var n = 0; n < inEdges.length; n++) {
      result.push({ nodeId: inEdges[n].source, edge: inEdges[n] });
    }
    return result;
  }

  return bfsCore(focusId, depth, expandNeighbours, function(e) { return e.id; });
};

/* --- kind priority ordering (SL-075 D6) --- */

model.kindOrder = {
  PRD: 1, SPEC: 1, ADR: 2, POL: 2, STD: 3, SL: 4,
  ISS: 5, IMP: 5, CHR: 5, RSK: 5, REV: 6, RV: 7,
  REQ: 8, IDE: 9, REC: 10, ASM: 11, DEC: 11, QUE: 12, CON: 12, CM: 20
};

function compareNodes(a, b) {
  var ordA = model.kindOrder[a.kindPrefix] || 99;
  var ordB = model.kindOrder[b.kindPrefix] || 99;
  if (ordA !== ordB) return ordA - ordB;
  var numA = parseInt(a.id.split('-').pop(), 10) || 0;
  var numB = parseInt(b.id.split('-').pop(), 10) || 0;
  if (numA !== numB) return numA - numB;
  return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
}

// Used by app.js via global scope
// eslint-disable-next-line no-unused-vars
function compareEdgesBySource(ea, eb) {
  var sa = state.graph.nodes.get(ea.source);
  var sb = state.graph.nodes.get(eb.source);
  if (!sa || !sb) return ea.id < eb.id ? -1 : 1;
  return compareNodes(sa, sb);
}

/* --- kind aggregation --- */

model.kinds = function(nodes) {
  var counts = new Map();
  nodes.forEach(function(node) {
    var kp = node.kindPrefix;
    counts.set(kp, (counts.get(kp) || 0) + 1);
  });

  /* sort by prefix alphabetically */
  var sorted = new Map();
  var keys = [];
  counts.forEach(function(_, k) { keys.push(k); });
  keys.sort();
  keys.forEach(function(k) { sorted.set(k, counts.get(k)); });
  return sorted;
};

/* --- search / filter --- */

model.searchFilter = function(query, graph) {
  var results = [];
  if (query === null || query === '') {
    graph.nodes.forEach(function(node) { results.push(node); });
    results.sort(compareNodes);
    return results;
  }

  var q = query.toLowerCase();
  graph.nodes.forEach(function(node) {
    if (node.id.toLowerCase().indexOf(q) !== -1 ||
        node.title.toLowerCase().indexOf(q) !== -1) {
      results.push(node);
    }
  });
  results.sort(compareNodes);
  return results;
};

/* --- internal helpers --- */

function sortedNodeIds(graph) {
  var keys = [];
  graph.nodes.forEach(function(_, k) { keys.push(k); });
  keys.sort();
  return keys;
}

function looseCanonical(query) {
  /* find first digit position */
  var firstDigit = -1;
  for (var i = 0; i < query.length; i++) {
    if (/[0-9]/.test(query.charAt(i))) {
      firstDigit = i;
      break;
    }
  }
  if (firstDigit <= 0) return null;

  var prefix = '';
  for (var j = 0; j < firstDigit; j++) {
    var ch = query.charAt(j);
    if (/[A-Za-z]/.test(ch)) {
      prefix += ch.toUpperCase();
    }
  }
  if (prefix === '') return null;

  /* extract numeric digits from remainder */
  var numStr = '';
  for (var k = firstDigit; k < query.length; k++) {
    var d = query.charAt(k);
    if (/[0-9]/.test(d)) {
      numStr += d;
    }
  }
  if (numStr === '') return null;

  var num = parseInt(numStr, 10);
  if (isNaN(num)) return null;

  return prefix + '-' + padId(num);
}

/* --- concept map normalization --- */

model.normalizeConceptMap = function(raw) {
  return {
    id: raw.id,
    title: raw.title,
    status: raw.status,
    description: raw.description || '',
    dslHash: raw.dsl_hash || '',
    nodes: raw.nodes || [],
    edges: raw.edges || [],
    diagnostics: raw.diagnostics || []
  };
};

model.buildNodeLabelList = function(cm) {
  if (!cm || !cm.nodes) return [];
  var labels = [];
  var seen = {};
  for (var i = 0; i < cm.nodes.length; i++) {
    var label = cm.nodes[i].label;
    if (!seen[label]) {
      seen[label] = true;
      labels.push(label);
    }
  }
  return labels;
};

model.buildRelLabelList = function(cm) {
  if (!cm || !cm.edges) return [];
  var rels = [];
  var seen = {};
  for (var i = 0; i < cm.edges.length; i++) {
    var rel = cm.edges[i].rel;
    if (!seen[rel]) {
      seen[rel] = true;
      rels.push(rel);
    }
  }
  return rels;
};

model.cmNeighbourhood = function(cm, focusKey, depth) {
  if (cm === null || cm === undefined) return { nodes: [], edges: [] };
  if (focusKey === null || focusKey === undefined) {
    return { nodes: cm.nodes || [], edges: cm.edges || [] };
  }
  depth = Math.max(0, Math.min(3, depth));

  var edges = cm.edges || [];

  /* Build undirected adjacency map with edge references for bfsCore */
  var adj = {};
  for (var i = 0; i < edges.length; i++) {
    var e = edges[i];
    if (!adj[e.from_key]) adj[e.from_key] = [];
    adj[e.from_key].push({ nodeId: e.to_key, edge: e });
    if (!adj[e.to_key]) adj[e.to_key] = [];
    adj[e.to_key].push({ nodeId: e.from_key, edge: e });
  }

  /* Ensure focusKey exists in the node set */
  var nodeKeySet = {};
  for (var j = 0; j < cm.nodes.length; j++) {
    nodeKeySet[cm.nodes[j].key] = true;
  }
  if (!nodeKeySet[focusKey]) {
    /* Graceful fallback: focusKey not in nodes → return all */
    return { nodes: cm.nodes, edges: edges };
  }

  /* BFS node traversal via shared core */
  function expandNeighbours(key) {
    return adj[key] || [];
  }

  var result = bfsCore(focusKey, depth, expandNeighbours, function(e) {
    return e.from_key + '\x00' + e.rel + '\x00' + e.to_key;
  });

  /* Filter nodes to visited set (preserving original order) */
  var filteredNodes = [];
  for (var n = 0; n < cm.nodes.length; n++) {
    if (result.nodes.has(cm.nodes[n].key)) {
      filteredNodes.push(cm.nodes[n]);
    }
  }

  /* Filter edges: both ends in visited (preserving original order) */
  var filteredEdges = [];
  for (var m = 0; m < edges.length; m++) {
    if (result.nodes.has(edges[m].from_key) && result.nodes.has(edges[m].to_key)) {
      filteredEdges.push(edges[m]);
    }
  }

  return { nodes: filteredNodes, edges: filteredEdges };
};
