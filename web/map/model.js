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
    /* skip unresolved targets */
    if (!('Resolved' in edge.target)) return;

    var source = edge.source.prefix + '-' + padId(edge.source.id);
    var target = edge.target.Resolved.prefix + '-' + padId(edge.target.Resolved.id);

    /* build edge id using nodes-canonical forms */
    var edgeId = 'e_' + encodePart(source) + '_' + encodePart(edge.label) + '_' + encodePart(target);

    /* coalesce duplicates */
    if (edgeById.has(edgeId)) return;

    var edgeObj = {
      id: edgeId,
      source: source,
      label: edge.label,
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

model.neighbourhood = function(focusId, depth, graph) {
  depth = Math.max(0, Math.min(3, depth));
  if (depth === 0) {
    return { nodes: new Set([focusId]), edges: [] };
  }

  var visited = new Set();
  var collectedEdges = [];
  var collectedEdgeIds = new Set();
  var queue = [{ id: focusId, dist: 0 }];
  visited.add(focusId);

  while (queue.length > 0) {
    var current = queue.shift();
    if (current.dist >= depth) continue;

    /* outgoing edges */
    var outEdges = graph.outgoing.get(current.id) || [];
    outEdges.forEach(function(edge) {
      if (!visited.has(edge.target)) {
        visited.add(edge.target);
        queue.push({ id: edge.target, dist: current.dist + 1 });
      }
      if (!collectedEdgeIds.has(edge.id)) {
        collectedEdgeIds.add(edge.id);
        collectedEdges.push(edge);
      }
    });

    /* incoming edges */
    var inEdges = graph.incoming.get(current.id) || [];
    inEdges.forEach(function(edge) {
      if (!visited.has(edge.source)) {
        visited.add(edge.source);
        queue.push({ id: edge.source, dist: current.dist + 1 });
      }
      if (!collectedEdgeIds.has(edge.id)) {
        collectedEdgeIds.add(edge.id);
        collectedEdges.push(edge);
      }
    });
  }

  return { nodes: visited, edges: collectedEdges };
};

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
    results.sort(function(a, b) { return a.id < b.id ? -1 : 1; });
    return results;
  }

  var q = query.toLowerCase();
  graph.nodes.forEach(function(node) {
    if (node.id.toLowerCase().indexOf(q) !== -1 ||
        node.title.toLowerCase().indexOf(q) !== -1) {
      results.push(node);
    }
  });
  results.sort(function(a, b) { return a.id < b.id ? -1 : 1; });
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
