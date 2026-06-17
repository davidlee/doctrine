// Doctrine Map Explorer — entity-graph DOM construction (SL-083 PHASE-03)
// Exposed on window.render. Depends on: model (neighbourhood), dot (graphToDot),
// api (renderDot, fetchMarkdown), svg (injectHitRects, wireHandlers, dimLegend).
/* global router, compareEdgesBySource, model, dot, api, svg */
/* exported render */

var render = {};

/* -----------------------------------------------------------------------
 * DOM element factory (moved from app.js)
 * --------------------------------------------------------------------- */
render.el = function(tag, attrs, children) {
  var e = document.createElement(tag);
  if (attrs) {
    Object.keys(attrs).forEach(function (k) {
      if (k === 'className') e.className = attrs[k];
      else if (k === 'textContent') e.textContent = attrs[k];
      else if (k === 'innerHTML') e.innerHTML = attrs[k];
      else e.setAttribute(k, attrs[k]);
    });
  }
  if (children) {
    (Array.isArray(children) ? children : [children]).forEach(function (c) {
      if (typeof c === 'string') e.appendChild(document.createTextNode(c));
      else e.appendChild(c);
    });
  }
  return e;
};

/* -----------------------------------------------------------------------
 * HTML escaping (F-5: moved from app.js; encodeAttr deleted as dead duplicate)
 * --------------------------------------------------------------------- */
render.escapeHtml = function(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
};

render.escapeAttr = function(str) {
  return str.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/'/g, '&#39;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
};

/* -----------------------------------------------------------------------
 * DOM element cache (F-9: eliminate repeated querySelector calls)
 * --------------------------------------------------------------------- */
render.elements = {};

render.cacheElements = function(root) {
  var qs = root.querySelector.bind(root);
  render.elements.entityList = qs('.entity-list');
  render.elements.focusHeader = qs('.focus-header');
  render.elements.graphArea = qs('.graph-area');
  render.elements.hoverDetail = qs('.hover-detail');
  render.elements.relationshipTable = qs('.relationship-table');
  render.elements.relationshipTableBody = qs('.relationship-table tbody');
  render.elements.markdownPane = qs('.markdown-pane');
  render.elements.tableToggle = qs('.table-toggle');
  render.elements.depthSelector = qs('.depth-selector');
  render.elements.cmEdgeTable = qs('.cm-edge-table');
  render.elements.cmAddEdgeForm = qs('.cm-add-edge-form');
  render.elements.cmDiagnosticsPanel = qs('.cm-diagnostics-panel');
};

/* -----------------------------------------------------------------------
 * Entity list + focus header DOM construction
 * --------------------------------------------------------------------- */

/* Single entity-list <li> element (DRY). */
render.buildEntityItem = function(node, focusId, onFocus) {
  var li = document.createElement('li');
  li.className = 'entity-item';
  if (node.id === focusId) li.classList.add('active');
  var t = document.createElement('span'); t.className = 'entity-title'; t.textContent = node.title; li.appendChild(t);
  var p = document.createElement('span'); p.className = 'kind-pill';
  p.setAttribute('data-kind', node.kindPrefix);  // F-7: data-kind selector target
  p.style.background = 'var(--kind-' + node.kindPrefix + ')';
  p.textContent = node.kindPrefix;
  li.appendChild(p);
  li.addEventListener('click', function(id) {
    return function() { onFocus(id); };
  }(node.id));
  return li;
};

/* Options: { container, nodes, focusId, onFocus } */
render.entityList = function(opts) {
  if (!opts.container) return;
  opts.container.innerHTML = '';
  (opts.nodes || []).forEach(function(node) {
    opts.container.appendChild(render.buildEntityItem(node, opts.focusId, opts.onFocus));
  });
};

/* Options: { container, focusId, graph } */
render.focusHeader = function(opts) {
  var container = opts.container;
  if (!container) return;

  if (!opts.focusId) {
    container.innerHTML = '<span class="placeholder">Entity title \u2014 kind \u00b7 status</span>';
    return;
  }

  var node = opts.graph.nodes.get(opts.focusId);
  if (!node) {
    container.innerHTML = '<span class="placeholder">Entity title \u2014 kind \u00b7 status</span>';
    return;
  }

  container.innerHTML = '<span>' + render.escapeHtml(node.title) + '</span>' +
    ' <span class="kind-pill" data-kind="' + render.escapeAttr(node.kindPrefix) + '" style="background:var(--kind-' + render.escapeHtml(node.kindPrefix) + ')">' + render.escapeHtml(node.kindPrefix) + '</span>' +
    ' <span class="status">' + render.escapeHtml(node.status) + '</span>';
};

/* -----------------------------------------------------------------------
 * View mode: toggle entity-graph vs concept-map vs edge UI visibility (D5)
 * --------------------------------------------------------------------- */
render.setViewMode = function(mode) {
  // Depth selector: visible in entity-graph and concept-map, hidden in edge
  if (render.elements.depthSelector) {
    render.elements.depthSelector.style.display = (mode === 'edge') ? 'none' : '';
  }

  // Relationship table: visible in entity-graph and actionability, hidden in edge/concept-map
  if (render.elements.relationshipTable) {
    render.elements.relationshipTable.style.display = (mode === 'entity-graph' || mode === 'actionability') ? '' : 'none';
  }

  // Table toggle: visible in entity-graph and actionability
  if (render.elements.tableToggle) {
    render.elements.tableToggle.style.display = (mode === 'entity-graph' || mode === 'actionability') ? '' : 'none';
  }

  // CM containers: hide/clear when leaving concept-map mode (crash-clearing gate)
  if (mode !== 'concept-map') {
    var cmEdgeTable = render.elements.cmEdgeTable;
    if (cmEdgeTable) { cmEdgeTable.style.display = 'none'; cmEdgeTable.innerHTML = ''; }
    var cmAddForm = render.elements.cmAddEdgeForm;
    if (cmAddForm) { cmAddForm.style.display = 'none'; cmAddForm.innerHTML = ''; }
    var cmDiagPanel = render.elements.cmDiagnosticsPanel;
    if (cmDiagPanel) { cmDiagPanel.style.display = 'none'; cmDiagPanel.innerHTML = ''; }
  }
};

/* -----------------------------------------------------------------------
 * Relationship table DOM construction
 * Options: { container, edges, graph, focusId, depth }
 * --------------------------------------------------------------------- */
function setRelationshipTableHeadings(container, headings) {
  var table = container ? container.closest('table') : null;
  var headerRow = table ? table.querySelector('thead tr') : null;
  var i;
  if (!headerRow) return;
  headerRow.innerHTML = '';
  for (i = 0; i < headings.length; i++) {
    var th = document.createElement('th');
    th.textContent = headings[i];
    headerRow.appendChild(th);
  }
}

render.relationshipTable = function(opts) {
  var tbody = opts.container;
  if (!tbody) return;

  if (opts.viewMode === 'actionability') {
    var nodes = opts.actionabilityView && Array.isArray(opts.actionabilityView.nodes) ? opts.actionabilityView.nodes : [];
    setRelationshipTableHeadings(tbody, ['id', 'kind', 'status', 'actionability', 'blockers', 'consequence', 'title']);
    tbody.innerHTML = '';

    if (nodes.length === 0) {
      tbody.innerHTML = '<tr><td colspan="7"><span class="placeholder">[No actionability data to show]</span></td></tr>';
      return;
    }

    nodes.forEach(function(node) {
      var tr = document.createElement('tr');
      tr.style.cursor = 'pointer';
      tr.addEventListener('click', function() {
        window.location.hash = '#' + router.buildHash('focus', node.id, opts.depth);
      });
      var idCell = document.createElement('td');
      var idLink = document.createElement('a');
      idLink.href = '#' + router.buildHash('focus', node.id, opts.depth);
      idLink.textContent = node.id;
      idCell.appendChild(idLink);
      tr.appendChild(idCell);

      var kindCell = document.createElement('td');
      kindCell.textContent = node.kind || '';
      tr.appendChild(kindCell);

      var statusCell = document.createElement('td');
      statusCell.textContent = node.status || '';
      tr.appendChild(statusCell);

      var actionabilityCell = document.createElement('td');
      actionabilityCell.textContent = node.actionability || '';
      tr.appendChild(actionabilityCell);

      var blockersCell = document.createElement('td');
      blockersCell.textContent = Array.isArray(node.blockers) ? node.blockers.join(', ') : '';
      tr.appendChild(blockersCell);

      var consequenceCell = document.createElement('td');
      consequenceCell.textContent = node.consequence !== null && node.consequence !== undefined ? String(node.consequence) : '';
      tr.appendChild(consequenceCell);

      var titleCell = document.createElement('td');
      titleCell.textContent = node.title || '';
      tr.appendChild(titleCell);

      tbody.appendChild(tr);
    });
    return;
  }

  setRelationshipTableHeadings(tbody, ['src_id', 'src_title', 'label', 'tgt_id', 'tgt_title']);

  if (!opts.focusId) {
    tbody.innerHTML = '<tr><td colspan="5"><span class="placeholder">[Relationship table]</span></td></tr>';
    return;
  }

  var edges = opts.edges;
  var graph = opts.graph;
  var depth = opts.depth;

  edges.sort(compareEdgesBySource);

  tbody.innerHTML = '';
  if (edges.length === 0) {
    tbody.innerHTML = '<tr><td colspan="5"><span class="placeholder">[No relationships to show]</span></td></tr>';
    return;
  }

  edges.forEach(function(edge) {
    var tr = document.createElement('tr');

    var srcCell = document.createElement('td');
    var srcA = document.createElement('a');
    srcA.href = '#' + router.buildHash('focus', edge.source, depth);
    srcA.textContent = edge.source;
    srcCell.appendChild(srcA);
    tr.appendChild(srcCell);

    var srcTitle = document.createElement('td');
    var srcNode = graph.nodes.get(edge.source);
    srcTitle.textContent = srcNode ? srcNode.title : '';
    tr.appendChild(srcTitle);

    var labelCell = document.createElement('td');
    var labelA = document.createElement('a');
    labelA.href = '#' + router.buildHash('edge', edge.id, depth);
    labelA.className = 'edge-id-link';
    labelA.textContent = edge.label;
    labelA.title = 'Edge: ' + edge.id;
    labelCell.appendChild(labelA);
    tr.appendChild(labelCell);

    var tgtCell = document.createElement('td');
    var tgtA = document.createElement('a');
    tgtA.href = '#' + router.buildHash('focus', edge.target, depth);
    tgtA.textContent = edge.target;
    tgtCell.appendChild(tgtA);
    tr.appendChild(tgtCell);

    var tgtTitle = document.createElement('td');
    var tgtNode = graph.nodes.get(edge.target);
    tgtTitle.textContent = tgtNode ? tgtNode.title : '';
    tr.appendChild(tgtTitle);

    tbody.appendChild(tr);
  });
};

/* -----------------------------------------------------------------------
 * Hover detail pane
 * Options: { container, node }
 *   node: graph node object, or null to show placeholder
 * --------------------------------------------------------------------- */
render.hoverPane = function(opts) {
  var pane = opts.container;
  if (!pane) return;

  if (!opts.node) {
    pane.innerHTML = '<span class="placeholder">Hover a node for details</span>';
    return;
  }

  var node = opts.node;
  var html = '<div class="hover-detail-content">';
  html += '<span class="hover-detail-title">' + node.id + ': ' + render.escapeHtml(node.title) + '</span>';
  html += '<span class="hover-detail-meta">' + node.kindLabel + ' \u00b7 ' + node.status + '</span>';
  html += '</div>';
  pane.innerHTML = html;
};

/* -----------------------------------------------------------------------
 * Markdown rendering pipeline (safe: markdown-it → DOMPurify)
 * --------------------------------------------------------------------- */
var _markdownIt = null;

render.renderMarkdown = function(text) {
  if (!_markdownIt) {
    _markdownIt = window.markdownit({ html: false, linkify: true, typographer: true });
  }
  var raw = _markdownIt.render(text);
  return window.DOMPurify.sanitize(raw);
};

function _applyLinkPolicy(container) {
  var links = container.querySelectorAll('a');
  for (var i = 0; i < links.length; i++) {
    var a = links[i];
    var href = a.getAttribute('href') || '';
    if (href.indexOf('http://') === 0 || href.indexOf('https://') === 0) {
      a.setAttribute('target', '_blank');
      a.setAttribute('rel', 'noopener noreferrer');
    } else if (href.indexOf('#') === 0) {
      // Anchor link — preserve
    } else if (href) {
      var span = document.createElement('span');
      span.textContent = a.textContent;
      a.parentNode.replaceChild(span, a);
    }
  }
}

function _wireFullscreenToggle(container) {
  var btn = container.querySelector('.fullscreen-toggle');
  if (btn) {
    btn.addEventListener('click', function() {
      container.classList.toggle('fullscreen');
    });
  }
}

/* Options: { container, id, cache, currentFocusId } */
render.markdownPane = function(opts) {
  var container = opts.container;
  var id = opts.id;
  var cache = opts.cache;
  var currentFocusId = opts.currentFocusId;

  function wrapContent(innerHTML) {
    return '<div class="markdown-toolbar">' +
      '<span class="markdown-toolbar-title">' + render.escapeHtml(id) + '</span>' +
      '<button class="fullscreen-toggle" title="Toggle fullscreen">&square;</button>' +
      '</div>' +
      '<div class="markdown-body">' + innerHTML + '</div>';
  }

  // Cache check
  if (cache.has(id)) {
    container.innerHTML = wrapContent(render.renderMarkdown(cache.get(id)));
    _wireFullscreenToggle(container);
    _applyLinkPolicy(container);
    return;
  }

  container.innerHTML = '';
  var loading = document.createElement('p');
  loading.className = 'loading';
  loading.textContent = 'Loading markdown…';
  container.appendChild(loading);

  api.fetchMarkdown(id).then(function(text) {
    if (currentFocusId !== id) return;
    cache.set(id, text);
    container.innerHTML = wrapContent(render.renderMarkdown(text));
    _wireFullscreenToggle(container);
    _applyLinkPolicy(container);
  }).catch(function(err) {
    if (currentFocusId !== id) return;
    container.innerHTML = '';
    if (err.status === 404) {
      var msg = document.createElement('p');
      msg.className = 'muted';
      msg.textContent = 'No markdown body for ' + id;
      container.appendChild(msg);
    } else if (err.status === 501) {
      var info = document.createElement('p');
      info.className = 'info';
      info.textContent = 'Markdown not implemented for requirements';
      container.appendChild(info);
    } else {
      var error = document.createElement('p');
      error.className = 'error';
      error.textContent = 'Failed to load markdown: ' + err.message;
      container.appendChild(error);
    }
  });
};

/* -----------------------------------------------------------------------
 * Entity-graph SVG rendering (async: DOT → API → DOMPurify → SVG DOM)
 * Options: { container, graph, focusId, depth, dotAvailable, seq,
 *            getCurrentSeq, onNodeClick, onNodeHoverEnter, onNodeHoverLeave }
 * --------------------------------------------------------------------- */
render.graphPane = function(opts) {
  var container = opts.container;
  var graph = opts.graph;
  var focusId = opts.focusId;
  var depth = Math.max(0, Math.min(3, opts.depth));
  var dotAvailable = opts.dotAvailable;
  var seq = opts.seq;

  var nb = model.neighbourhood(focusId, depth, graph);
  var dotText = dot.graphToDot(nb, focusId, depth);

  if (!dotAvailable) {
    container.innerHTML = '';
    var errMsg = document.createElement('p');
    errMsg.className = 'error';
    errMsg.textContent = 'Graphviz not available. DOT source:';
    container.appendChild(errMsg);
    var pre = document.createElement('pre');
    pre.textContent = dotText;
    container.appendChild(pre);
    return;
  }

  container.innerHTML = '';
  var loading = document.createElement('p');
  loading.className = 'loading';
  loading.textContent = 'Rendering graph…';
  container.appendChild(loading);

  api.renderDot(dotText).then(function(svgText) {
    if (seq !== opts.getCurrentSeq()) return;
    var clean = window.DOMPurify.sanitize(svgText, { USE_PROFILES: { svg: true } });
    container.innerHTML = clean;
    var svgEl = container.querySelector('svg');
    if (svgEl) {
      svg.injectHitRects(svgEl);
      svg.wireHandlers(svgEl, function(g) {
        var t = g.querySelector('text');
        return t ? t.textContent.trim() : '';
      }, {
        onClick: opts.onNodeClick,
        onHoverEnter: opts.onNodeHoverEnter,
        onHoverLeave: opts.onNodeHoverLeave
      });
      svg.dimLegend(nb);
    }
  }).catch(function(err) {
    if (seq !== opts.getCurrentSeq()) return;
    container.innerHTML = '';
    var errMsg2 = document.createElement('p');
    errMsg2.className = 'error';
    errMsg2.textContent = 'Graphviz not available';
    container.appendChild(errMsg2);
    var pre2 = document.createElement('pre');
    pre2.textContent = dotText;
    container.appendChild(pre2);
  });
};

/* -----------------------------------------------------------------------
 * Edge detail view
 * Options: { container, edge, graph, depth, focusId }
 * --------------------------------------------------------------------- */
render.edgeDetail = function(opts) {
  var container = opts.container;
  var edge = opts.edge;
  var graph = opts.graph;
  var depth = opts.depth;
  var focusId = opts.focusId;

  if (!edge) {
    if (container) {
      container.innerHTML = '<p class="error">Edge not found in graph</p>';
    }
    return;
  }

  var srcNode = graph.nodes.get(edge.source);
  var tgtNode = graph.nodes.get(edge.target);
  var originFile = edge.raw && edge.raw.origin && edge.raw.origin.file ? edge.raw.origin.file : '-';

  var html = '<div class="edge-detail">';
  html += '<h2>Edge: ' + render.escapeHtml(edge.id) + '</h2>';
  html += '<table class="edge-detail-table">';
  html += '<tr><th>Edge ID</th><td>' + render.escapeHtml(edge.id) + '</td></tr>';
  html += '<tr><th>Source</th><td><a href="#' + router.buildHash('focus', edge.source, depth) + '">' + render.escapeHtml(edge.source) + '</a>' + (srcNode ? ' &mdash; ' + render.escapeHtml(srcNode.title) : '') + '</td></tr>';
  html += '<tr><th>Label</th><td>' + render.escapeHtml(edge.label) + '</td></tr>';
  html += '<tr><th>Target</th><td><a href="#' + router.buildHash('focus', edge.target, depth) + '">' + render.escapeHtml(edge.target) + '</a>' + (tgtNode ? ' &mdash; ' + render.escapeHtml(tgtNode.title) : '') + '</td></tr>';
  html += '<tr><th>Origin file</th><td>' + render.escapeHtml(originFile) + '</td></tr>';
  html += '</table>';
  html += '<p class="edge-detail-back"><a href="#' + router.buildHash('focus', focusId, depth) + '">&larr; Back to ' + render.escapeHtml(focusId) + '</a></p>';
  html += '</div>';

  if (container) container.innerHTML = html;
};
