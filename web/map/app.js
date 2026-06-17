// Doctrine Map Explorer — SPA shell (SL-073)
// Hash routing: #/focus/SL-001 or #/focus/SL-001?depth=2
// Security: markdown-it html:false; DOMPurify.sanitize() applied before innerHTML.
// SVG from /api/dot/svg is sanitized via DOMPurify SVG profile, then injected as inline DOM.
/* global state, model, api, router, dot, svg, render, compareNodes, compareEdgesBySource */

(function () {
  'use strict';

  /* -----------------------------------------------------------------------
   * State
   * --------------------------------------------------------------------- */
  var md = null;             // markdown-it instance (lazy)

  /* -----------------------------------------------------------------------
   * Utilities
   * --------------------------------------------------------------------- */
  function showError(container, msg) {
    container.innerHTML = '';
    container.appendChild(
      render.el('div', { className: 'error' }, [
        render.el('p', { textContent: 'Error: ' + msg })
      ])
    );
  }

  /* -----------------------------------------------------------------------
   * Markdown rendering (safe pipeline)
   * --------------------------------------------------------------------- */
  function renderMarkdown(text) {
    if (!md) {
      md = window.markdownit({ html: false, linkify: true, typographer: true });
    }
    var raw = md.render(text);
    return window.DOMPurify.sanitize(raw);
  }

  /* -----------------------------------------------------------------------
   * Entity list node collection helpers
   * --------------------------------------------------------------------- */

  // Collect and filter entity nodes for the sidebar list.
  // query: search string (null/empty = all). Returns sorted node array.
  function collectListNodes(query) {
    var nodes;
    if (query && query.trim()) {
      nodes = model.searchFilter(query.trim(), state.graph);
    } else {
      nodes = [];
      state.graph.nodes.forEach(function(node) { nodes.push(node); });
    }
    if (state.kindFilter) {
      nodes = nodes.filter(function(node) { return state.kindFilter.has(node.kindPrefix); });
    }
    nodes.sort(compareNodes);
    return nodes;
  }

  function renderFilteredEntities() {
    var input = document.querySelector('.search-input');
    var nodes = collectListNodes(input ? input.value : '');
    render.entityList({
      container: render.elements.entityList,
      nodes: nodes,
      focusId: state.focusId,
      onFocus: function(id) { router.setFocus(id, state.depth); }
    });
  }

  function renderRelationshipTable() {
    var tbody = document.querySelector('.relationship-table tbody');
    if (!tbody) return;

    if (!state.focusId) {
      tbody.innerHTML = '<tr><td colspan="5"><span class="placeholder">[Relationship table]</span></td></tr>';
      return;
    }

    var nb = model.neighbourhood(state.focusId, state.depth, state.graph);
    var edges = nb.edges;

    if (state.kindFilter) {
      edges = edges.filter(function(edge) {
        var src = state.graph.nodes.get(edge.source);
        return src && state.kindFilter.has(src.kindPrefix);
      });
    }

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
      srcA.href = '#' + router.buildHash('focus', edge.source, state.depth);
      srcA.textContent = edge.source;
      srcCell.appendChild(srcA);
      tr.appendChild(srcCell);

      var srcTitle = document.createElement('td');
      var srcNode = state.graph.nodes.get(edge.source);
      srcTitle.textContent = srcNode ? srcNode.title : '';
      tr.appendChild(srcTitle);

      var labelCell = document.createElement('td');
      var labelA = document.createElement('a');
      labelA.href = '#' + router.buildHash('edge', edge.id, state.depth);
      labelA.className = 'edge-id-link';
      labelA.textContent = edge.label;
      labelA.title = 'Edge: ' + edge.id;
      labelCell.appendChild(labelA);
      tr.appendChild(labelCell);

      var tgtCell = document.createElement('td');
      var tgtA = document.createElement('a');
      tgtA.href = '#' + router.buildHash('focus', edge.target, state.depth);
      tgtA.textContent = edge.target;
      tgtCell.appendChild(tgtA);
      tr.appendChild(tgtCell);

      var tgtTitle = document.createElement('td');
      var tgtNode = state.graph.nodes.get(edge.target);
      tgtTitle.textContent = tgtNode ? tgtNode.title : '';
      tr.appendChild(tgtTitle);

      tbody.appendChild(tr);
    });
  }

  function collectKindFilter() {
    var cbs = document.querySelectorAll('.kind-checkbox input[type="checkbox"]');
    var allOn = true;
    for (var i = 0; i < cbs.length; i++) {
      if (!cbs[i].checked) { allOn = false; break; }
    }
    if (allOn) {
      state.kindFilter = null;
    } else {
      state.kindFilter = new Set();
      for (var j = 0; j < cbs.length; j++) {
        if (cbs[j].checked) {
          var kinds = (cbs[j].getAttribute('data-kinds') || '').split(',');
          for (var k = 0; k < kinds.length; k++) {
            var kp = kinds[k].trim();
            if (kp) state.kindFilter.add(kp);
          }
        }
      }
    }
  }

  function applyFilters() {
    renderFilteredEntities();
    renderRelationshipTable();
  }

  function wireFilterCheckboxes() {
    // Toggle-all checkbox
    var toggleAll = document.querySelector('.toggle-all-cb');
    var kindCbs = document.querySelectorAll('.kind-checkbox input[type="checkbox"]');

    if (toggleAll) {
      toggleAll.addEventListener('change', function() {
        for (var i = 0; i < kindCbs.length; i++) {
          kindCbs[i].checked = toggleAll.checked;
        }
        collectKindFilter();
        applyFilters();
      });
    }

    // Individual kind checkboxes
    for (var i = 0; i < kindCbs.length; i++) {
      kindCbs[i].addEventListener('change', function() {
        collectKindFilter();
        applyFilters();
        // Sync toggle-all state
        if (toggleAll) {
          var allOn = true;
          for (var j = 0; j < kindCbs.length; j++) {
            if (!kindCbs[j].checked) { allOn = false; break; }
          }
          toggleAll.checked = allOn;
        }
      });
    }
    collectKindFilter();
  }

  function wireSearch() {
    var input = document.querySelector('.search-input');
    if (!input) return;

    input.addEventListener('input', function() {
      var nodes = collectListNodes(input.value.trim());
      render.entityList({
        container: render.elements.entityList,
        nodes: nodes,
        focusId: state.focusId,
        onFocus: function(id) { router.setFocus(id, state.depth); }
      });
    });

    input.addEventListener('keydown', function(e) {
      var list = document.querySelector('.entity-list');
      var items = list ? list.querySelectorAll('.entity-item') : [];

      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        e.preventDefault();
        if (items.length === 0) return;
        if (typeof state.listNavIndex === 'undefined' || state.listNavIndex < 0) {
          state.listNavIndex = e.key === 'ArrowDown' ? 0 : items.length - 1;
        } else {
          state.listNavIndex += (e.key === 'ArrowDown' ? 1 : -1);
          if (state.listNavIndex >= items.length) state.listNavIndex = 0;
          if (state.listNavIndex < 0) state.listNavIndex = items.length - 1;
        }
        for (var i = 0; i < items.length; i++) {
          items[i].classList.toggle('nav-highlight', i === state.listNavIndex);
        }
        // Scroll highlighted item into view
        if (items[state.listNavIndex]) {
          items[state.listNavIndex].scrollIntoView({ block: 'nearest' });
        }
      } else if (e.key === 'Enter') {
        // If keyboard nav highlight is active, select that item
        if (typeof state.listNavIndex !== 'undefined' && state.listNavIndex >= 0 && items.length > 0 && items[state.listNavIndex]) {
          e.preventDefault();
          items[state.listNavIndex].click();
          state.listNavIndex = undefined;
          return;
        }
        // Otherwise, use findFocus to resolve the query
        var query = input.value.trim();
        if (!query) return;
        var result = model.findFocus(query, state.graph);
        if (result) {
          router.setFocus(result, state.depth);
          state.listNavIndex = undefined;
        } else {
          if (list) {
            list.innerHTML = '<li class="entity-item"><span class="placeholder">No match for \'' + render.escapeHtml(query) + '\'</span></li>';
          }
        }
      } else if (e.key === 'Escape') {
        input.value = '';
        input.blur();
        state.listNavIndex = undefined;
        render.entityList({
          container: render.elements.entityList,
          nodes: collectListNodes(''),
          focusId: state.focusId,
          onFocus: function(id) { router.setFocus(id, state.depth); }
        });
      }
    });
  }

  function wireDepthButtons() {
    var btns = document.querySelectorAll('.depth-btn');
    for (var i = 0; i < btns.length; i++) {
      btns[i].addEventListener('click', (function(d) {
        return function() {
          var allBtns = document.querySelectorAll('.depth-btn');
          for (var j = 0; j < allBtns.length; j++) {
            allBtns[j].classList.toggle('active', parseInt(allBtns[j].getAttribute('data-depth'), 10) === d);
          }
          if (state.focusId) { router.setFocus(state.focusId, d); }
        };
      })(parseInt(btns[i].getAttribute('data-depth'), 10)));
    }
  }

  function wireRefresh() {
    var btn = document.querySelector('.refresh-btn');
    if (!btn) return;
    btn.addEventListener('click', function() {
      state.markdownCache.clear();
      state.conceptMapCache.clear();
      state.cmFocusNode = null;
      state.graphRenderSeq += 1;
      api.refreshGraph().then(function() {
        return api.fetchGraph();
      }).then(function(raw) {
        model.normalizeGraph(raw);
        if (state.focusId) {
          state.focusId = model.resolveFocus(state.focusId, state.graph);
        }
        renderView();
      }).catch(function(err) {
        var app = document.getElementById('app');
        showError(app, 'Failed to refresh: ' + err.message);
      });
    });
  }

  /* -----------------------------------------------------------------------
   * SVG Graph rendering (PHASE-03) — rendering pipeline + stale-render guard
   * --------------------------------------------------------------------- */
  function renderHoverPane(nodeId) {
    var pane = document.querySelector('.hover-detail');
    if (!pane) return;

    if (!nodeId) {
      pane.innerHTML = '<span class="placeholder">Hover a node for details</span>';
      return;
    }

    var node = state.graph.nodes.get(nodeId);
    if (!node) {
      pane.innerHTML = '<span class="placeholder">Node not found</span>';
      return;
    }

    var html = '<div class="hover-detail-content">';
    html += '<span class="hover-detail-title">' + node.id + ': ' + render.escapeHtml(node.title) + '</span>';
    html += '<span class="hover-detail-meta">' + node.kindLabel + ' \u00b7 ' + node.status + '</span>';
    html += '</div>';
    pane.innerHTML = html;
  }

  function renderGraphPane(container, focusId, depth) {
    depth = Math.max(0, Math.min(3, depth));

    var nb = model.neighbourhood(focusId, depth, state.graph);
    var dotText = dot.graphToDot(nb, focusId, depth);

    state.graphRenderSeq += 1;
    var seq = state.graphRenderSeq;

    if (!state.dotAvailable) {
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
      if (seq !== state.graphRenderSeq) return;
      var clean = window.DOMPurify.sanitize(svgText, { USE_PROFILES: { svg: true } });
      container.innerHTML = clean;
      var svgEl = container.querySelector('svg');
      if (svgEl) {
        svg.injectHitRects(svgEl);
        svg.wireHandlers(svgEl, function(g) {
          var t = g.querySelector('text');
          return t ? t.textContent.trim() : '';
        }, {
          onClick: function(id) { router.setFocus(id, state.depth); },
          onHoverEnter: function(id) { state.hoveredId = id; renderHoverPane(id); },
          onHoverLeave: function() { state.hoveredId = null; renderHoverPane(null); }
        });
        svg.dimLegend(nb);
      }
    }).catch(function(err) {
      if (seq !== state.graphRenderSeq) return;
      container.innerHTML = '';
      var errMsg = document.createElement('p');
      errMsg.className = 'error';
      errMsg.textContent = 'Graphviz not available';
      container.appendChild(errMsg);
      var pre = document.createElement('pre');
      pre.textContent = dotText;
      container.appendChild(pre);
    });
  }

  /* -----------------------------------------------------------------------
   * Markdown rendering (PHASE-04) — fetch, cache, link policy, error states
   * --------------------------------------------------------------------- */
  function applyLinkPolicy(container) {
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
        // Relative link — strip href, preserve text
        var span = document.createElement('span');
        span.textContent = a.textContent;
        a.parentNode.replaceChild(span, a);
      }
    }
  }

  function wireMarkdownPane(container) {
    var btn = container.querySelector('.fullscreen-toggle');
    if (btn) {
      btn.addEventListener('click', function() {
        container.classList.toggle('fullscreen');
      });
    }
  }

  function renderMarkdownPane(container, id) {
    function wrapContent(innerHTML) {
      return '<div class="markdown-toolbar">' +
        '<span class="markdown-toolbar-title">' + render.escapeHtml(id) + '</span>' +
        '<button class="fullscreen-toggle" title="Toggle fullscreen">&square;</button>' +
        '</div>' +
        '<div class="markdown-body">' + innerHTML + '</div>';
    }

    // Cache check
    if (state.markdownCache.has(id)) {
      container.innerHTML = wrapContent(renderMarkdown(state.markdownCache.get(id)));
      wireMarkdownPane(container);
      applyLinkPolicy(container);
      return;
    }

    container.innerHTML = '';
    var loading = document.createElement('p');
    loading.className = 'loading';
    loading.textContent = 'Loading markdown…';
    container.appendChild(loading);

    api.fetchMarkdown(id).then(function(text) {
      // Stale-request guard
      if (state.focusId !== id) return;

      state.markdownCache.set(id, text);
      container.innerHTML = wrapContent(renderMarkdown(text));
      wireMarkdownPane(container);
      applyLinkPolicy(container);
    }).catch(function(err) {
      // Stale-request guard
      if (state.focusId !== id) return;

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
  }

  /* -----------------------------------------------------------------------
   * Bootstrap + render loop (PHASE-05)
   * --------------------------------------------------------------------- */
  function wireTableToggle() {
    var cb = document.getElementById('hide-relations');
    var table = document.querySelector('.relationship-table');
    if (!cb || !table) return;
    // Restore persisted state
    var hidden = false;
    try { hidden = localStorage.getItem('doctrine-map-hide-relations') === '1'; } catch (_) { /* eslint-disable-line no-unused-vars */ }
    cb.checked = hidden;
    table.classList.toggle('hidden', hidden);
    cb.addEventListener('change', function() {
      table.classList.toggle('hidden', cb.checked);
      try { localStorage.setItem('doctrine-map-hide-relations', cb.checked ? '1' : '0'); } catch (_) { /* eslint-disable-line no-unused-vars */ }
    });
  }

  function bootstrap() {
    // Populate DOM element cache (F-9)
    render.cacheElements(document);

    // Wire interactive surfaces
    wireTableToggle();
    wireFilterCheckboxes();
    wireSearch();
    wireDepthButtons();
    wireRefresh();

    // Register hashchange listener early — before any async work — so
    // clicks and navigation work immediately, even during data load.
    window.addEventListener('hashchange', renderView);

    // Fetch health and graph in parallel
    Promise.all([
      api.fetchHealth().catch(function () { return { dot: { ok: false }, graph: { ok: false } }; }),
      api.fetchGraph().catch(function () { return null; })
    ]).then(function (results) {
      var health = results[0];
      var raw = results[1];

      state.dotAvailable = !!(health && health.dot && health.dot.ok);

      if (raw) {
        model.normalizeGraph(raw);
      }

      if (!state.focusId && state.graph.nodes.size > 0) {
        state.focusId = model.resolveFocus(null, state.graph);
        if (state.focusId) {
          router.setFocus(state.focusId, state.depth);
          return;
        }
      }

      renderView();
    }).catch(function (err) {
      var app = document.getElementById('app');
      showError(app, 'Failed to initialise: ' + err.message);
    });
  }

  function renderEdgeDetail(id) {
    var container = document.querySelector('.graph-area');
    var edge = state.graph.edgeById.get(id);
    if (!edge) {
      if (container) {
        container.innerHTML = '<p class="error">Edge ' + render.escapeHtml(id) + ' not found in graph</p>';
      }
      return;
    }

    var srcNode = state.graph.nodes.get(edge.source);
    var tgtNode = state.graph.nodes.get(edge.target);
    var originFile = edge.raw && edge.raw.origin && edge.raw.origin.file ? edge.raw.origin.file : '-';

    var html = '<div class="edge-detail">';
    html += '<h2>Edge: ' + render.escapeHtml(edge.id) + '</h2>';
    html += '<table class="edge-detail-table">';
    html += '<tr><th>Edge ID</th><td>' + render.escapeHtml(edge.id) + '</td></tr>';
    html += '<tr><th>Source</th><td><a href="#' + router.buildHash('focus', edge.source, state.depth) + '">' + render.escapeHtml(edge.source) + '</a>' + (srcNode ? ' &mdash; ' + render.escapeHtml(srcNode.title) : '') + '</td></tr>';
    html += '<tr><th>Label</th><td>' + render.escapeHtml(edge.label) + '</td></tr>';
    html += '<tr><th>Target</th><td><a href="#' + router.buildHash('focus', edge.target, state.depth) + '">' + render.escapeHtml(edge.target) + '</a>' + (tgtNode ? ' &mdash; ' + render.escapeHtml(tgtNode.title) : '') + '</td></tr>';
    html += '<tr><th>Origin file</th><td>' + render.escapeHtml(originFile) + '</td></tr>';
    html += '</table>';
    html += '<p class="edge-detail-back"><a href="#' + router.buildHash('focus', state.focusId, state.depth) + '">&larr; Back to ' + render.escapeHtml(state.focusId) + '</a></p>';
    html += '</div>';

    if (container) container.innerHTML = html;
  }

  /* Instant pre-render focus highlight on the current SVG.
   * Applied before the async graph re-render to give immediate visual
   * feedback when the user clicks a different node at the same depth. */
  function renderView() {
    var route = router.parseHash();
    var prevFocusId = state.focusId;
    var prevDepth = state.depth;
    var mdPane;

    if (route.view === 'focus') {
      state.focusId = route.id;
    }
    state.depth = Math.max(0, Math.min(3, route.depth));

    if (route.view === 'edge' && !state.focusId && state.graph.nodes.size > 0) {
      state.focusId = model.resolveFocus(null, state.graph);
    }

    if (route.view === 'edge') {
      renderEdgeDetail(route.id);
      renderHoverPane(null);
      render.setViewMode('edge');
      mdPane = document.querySelector('.markdown-pane');
      if (mdPane) mdPane.innerHTML = '<span class="placeholder">[Markdown content]</span>';
      var tbody = document.querySelector('.relationship-table tbody');
      if (tbody) tbody.innerHTML = '<tr><td colspan="5"><span class="placeholder">[Relationship table]</span></td></tr>';
      render.entityList({
        container: render.elements.entityList,
        nodes: collectListNodes(''),
        focusId: state.focusId,
        onFocus: function(id) { router.setFocus(id, state.depth); }
      });
      render.focusHeader({
        container: render.elements.focusHeader,
        focusId: state.focusId,
        graph: state.graph
      });
      return;
    }

    // Sidebar / header / table always update synchronously
    renderFilteredEntities();
    render.focusHeader({
      container: render.elements.focusHeader,
      focusId: state.focusId,
      graph: state.graph
    });
    renderRelationshipTable();
    renderHoverPane(null);

    // Sync depth button active state
    var depthBtns = document.querySelectorAll('.depth-btn');
    for (var di = 0; di < depthBtns.length; di++) {
      depthBtns[di].classList.toggle('active', parseInt(depthBtns[di].getAttribute('data-depth'), 10) === state.depth);
    }

    // Graph: always re-render on focus change (BFS is centre-centric).
    // Apply instant highlight on old SVG for same-depth focus switch so
    // the user sees immediate feedback before the async render completes.
    var graphArea = document.querySelector('.graph-area');
    var focusChanged = (state.focusId !== prevFocusId);
    var depthChanged = (state.depth !== prevDepth);
    var graphMissing = !graphArea || !graphArea.querySelector('svg');

    /* Evict old CM cache on focus change — forces re-fetch on return */
    if (focusChanged && prevFocusId && isConceptMap(prevFocusId)) {
      state.conceptMapCache.delete(prevFocusId);
    }

    if (graphArea && (focusChanged || depthChanged || graphMissing)) {
      if (focusChanged && !depthChanged && state.focusId) {
        var svgEl = graphArea.querySelector('svg');
        if (svgEl) {
          svg.applyFocusHighlight(svgEl, state.focusId, prevFocusId, function(g) {
            var t = g.querySelector('text');
            if (t) return t.textContent.trim();
            var title = g.querySelector('title');
            return title ? title.textContent.trim() : '';
          });
        }
      }
      if (state.focusId) {
        if (isConceptMap(state.focusId)) {
          renderConceptMap();
        } else {
          renderGraphPane(graphArea, state.focusId, state.depth);
        }
      }
    }

    /* Show/hide entity-graph vs concept-map UI elements */
    var isCm = state.focusId && isConceptMap(state.focusId);
    render.setViewMode(isCm ? 'concept-map' : 'entity-graph');

    /* Reconcile cmFocusNode from URL hash */
    if (isCm && route.cmFocus) {
      var cachedCm = state.conceptMapCache.get(state.focusId);
      var label = route.cmFocus;
      if (cachedCm) {
        for (var ci = 0; ci < cachedCm.nodes.length; ci++) {
          if (cachedCm.nodes[ci].key === route.cmFocus) {
            label = cachedCm.nodes[ci].label;
            break;
          }
        }
      }
      if (!state.cmFocusNode || state.cmFocusNode.key !== route.cmFocus) {
        state.cmFocusNode = { key: route.cmFocus, label: label };
      }
    } else if (focusChanged) {
      state.cmFocusNode = null;
    }

    // CM-specific UI elements
    renderEditToggle();
    if (isCm && !focusChanged && !depthChanged) {
      // Same focus, re-render non-graph CM UI
      renderCmEdgeTable();
      renderAddEdgeForm();
      renderCmDiagnostics();
    }

    if (state.focusId) {
      mdPane = document.querySelector('.markdown-pane');
      if (mdPane) renderMarkdownPane(mdPane, state.focusId);
    }
  }

  /* -----------------------------------------------------------------------
   * Concept Map rendering (PHASE-04)
   * --------------------------------------------------------------------- */
  function isConceptMap(focusId) {
    var node = state.graph.nodes.get(focusId);
    return node && node.kindPrefix === 'CM';
  }

  function renderCmHoverPane(nodeKey) {
    var pane = document.querySelector('.hover-detail');
    if (!pane) return;
    if (!nodeKey) {
      pane.innerHTML = '<span class="placeholder">Hover a node for details</span>';
      return;
    }
    var cm = state.conceptMapCache.get(state.focusId);
    var label = nodeKey;
    if (cm) {
      for (var i = 0; i < cm.nodes.length; i++) {
        if (cm.nodes[i].key === nodeKey) {
          label = cm.nodes[i].label;
          break;
        }
      }
    }
    pane.innerHTML = '<div class="hover-detail-content">' +
      '<span class="hover-detail-title">' + render.escapeHtml(label) + '</span>' +
      '<span class="hover-detail-meta">concept map node</span>' +
      '</div>';
  }

  /* -----------------------------------------------------------------------
   * Concept Map authoring UI (PHASE-05)
   * --------------------------------------------------------------------- */

  function renderCmEdgeTable() {
    var container = document.querySelector('.cm-edge-table');
    if (!container) return;

    var cm = state.conceptMapCache.get(state.focusId);
    if (!cm) {
      container.innerHTML = '';
      container.style.display = 'none';
      return;
    }

    container.style.display = 'block';
    var edges = cm.edges || [];

    /* BFS-filtered edges in view mode when focal node is set */
    if (!state.editingConceptMap && state.cmFocusNode) {
      var filtered = model.cmNeighbourhood(cm, state.cmFocusNode.key, state.depth);
      edges = filtered.edges;
    }
    var editingKey = state.editingNode ? state.editingNode.key : null;
    var editingLabel = state.editingNode ? state.editingNode.label : '';

    var html = '<table class="cm-edges">';
    html += '<thead><tr><th>Source</th><th>Relation</th><th>Target</th>';
    if (state.editingConceptMap) {
      html += '<th></th>';
    }
    html += '</tr></thead><tbody>';

    if (edges.length === 0) {
      html += '<tr><td colspan="' + (state.editingConceptMap ? '4' : '3') + '"><span class="placeholder">No edges</span></td></tr>';
    } else {
      edges.forEach(function(edge) {
        html += '<tr class="cm-edge-row">';

        // Source cell — render input if this node is being renamed
        html += '<td>';
        if (editingKey && edge.from_key === editingKey && state.editingConceptMap) {
          html += '<input type="text" class="cm-rename-input" data-key="' + render.escapeAttr(editingKey) + '" value="' + render.escapeAttr(editingLabel) + '">';
        } else {
          html += '<span class="cm-edge-label' + (state.editingConceptMap ? ' cm-editable-node" data-key="' + render.escapeAttr(edge.from_key) + '" data-label="' + render.escapeAttr(edge.from_label) : '') + '">' + render.escapeHtml(edge.from_label) + '</span>';
        }
        html += '</td>';

        html += '<td>' + render.escapeHtml(edge.rel) + '</td>';

        // Target cell — render input if this node is being renamed
        html += '<td>';
        if (editingKey && edge.to_key === editingKey && state.editingConceptMap) {
          html += '<input type="text" class="cm-rename-input" data-key="' + render.escapeAttr(editingKey) + '" value="' + render.escapeAttr(editingLabel) + '">';
        } else {
          html += '<span class="cm-edge-label' + (state.editingConceptMap ? ' cm-editable-node" data-key="' + render.escapeAttr(edge.to_key) + '" data-label="' + render.escapeAttr(edge.to_label) : '') + '">' + render.escapeHtml(edge.to_label) + '</span>';
        }
        html += '</td>';

        if (state.editingConceptMap) {
          html += '<td><button class="cm-remove-btn" data-source="' + render.escapeAttr(edge.from_label) + '" data-rel="' + render.escapeAttr(edge.rel) + '" data-target="' + render.escapeAttr(edge.to_label) + '" title="Remove edge">✕</button></td>';
        }
        html += '</tr>';
      });
    }

    html += '</tbody></table>';
    container.innerHTML = html;

    // Wire remove buttons
    if (state.editingConceptMap) {
      var removeBtns = container.querySelectorAll('.cm-remove-btn');
      for (var i = 0; i < removeBtns.length; i++) {
        (function(btn) {
          btn.addEventListener('click', function() {
            handleRemoveEdge(
              btn.getAttribute('data-source'),
              btn.getAttribute('data-rel'),
              btn.getAttribute('data-target')
            );
          });
        })(removeBtns[i]);
      }

      // Wire inline rename on edge table node labels (non-editing-node cells)
      var editableNodes = container.querySelectorAll('.cm-editable-node');
      for (var j = 0; j < editableNodes.length; j++) {
        (function(el) {
          el.addEventListener('click', function() {
            startRenameNode(el.getAttribute('data-key'));
          });
        })(editableNodes[j]);
      }

      // Wire rename input(s) — Enter submits, Escape cancels
      var renameInputs = container.querySelectorAll('.cm-rename-input');
      for (var k = 0; k < renameInputs.length; k++) {
        (function(inp) {
          // Focus the first one
          if (k === 0) inp.focus();
          inp.addEventListener('keydown', function(ev) {
            if (ev.key === 'Enter') {
              ev.preventDefault();
              handleRenameNodeSubmit(inp.value);
            } else if (ev.key === 'Escape') {
              ev.preventDefault();
              state.editingNode = null;
              refreshCmView();
            }
          });
        })(renameInputs[k]);
      }
    }
  }

  function renderCmDiagnostics() {
    var panel = document.querySelector('.cm-diagnostics-panel');
    if (!panel) return;

    if (state.editingConceptMap) {
      panel.style.display = 'none';
      return;
    }

    var cm = state.conceptMapCache.get(state.focusId);
    if (!cm || !cm.diagnostics || cm.diagnostics.length === 0) {
      panel.style.display = 'none';
      return;
    }

    var html = '<h3>Diagnostics</h3>';
    for (var i = 0; i < cm.diagnostics.length; i++) {
      var d = cm.diagnostics[i];
      var msg = formatDiagnostic(d);
      var line = diagnosticLine(d);
      var prefix = line !== null ? ('line ' + line + ': ') : '';
      html += '<div class="cm-diag-item">\u26A0 ' + render.escapeHtml(prefix + msg) + '</div>';
    }
    panel.innerHTML = html;
    panel.style.display = 'block';
  }

  /* Extract the line number from a diagnostic object, or null. */
  function diagnosticLine(d) {
    if (!d) return null;
    var keys = Object.keys(d);
    if (keys.length === 0) return null;
    var variant = d[keys[0]];
    if (!variant || typeof variant !== 'object') return null;
    // CanonicalNodeCollision uses 'line' (not 'first_line')
    if (typeof variant.line === 'number') return variant.line;
    // SimilarNodeLabel / RelationDrift use line_a
    if (typeof variant.line_a === 'number') return variant.line_a;
    return null;
  }

  /* Format a diagnostic variant into a human-readable message. */
  function formatDiagnostic(d) {
    if (!d) return 'Unknown diagnostic';
    var keys = Object.keys(d);
    if (keys.length === 0) return 'Unknown diagnostic';
    var variant = keys[0];
    var v = d[variant] || {};

    switch (variant) {
      case 'CanonicalNodeCollision':
        return 'Node label "' + render.escapeHtml(v.label || '') + '" collides with key "' + render.escapeHtml(v.key || '') + '" (first label "' + render.escapeHtml(v.first_label || '') + '" takes precedence)';
      case 'SelfEdge':
        return 'Self-referencing edge: "' + render.escapeHtml(v.node_key || '') + '" \u2192 "' + render.escapeHtml(v.node_key || '') + '"';
      case 'SimilarNodeLabel':
        return 'Similar node labels: "' + render.escapeHtml(v.label_a || '') + '" / "' + render.escapeHtml(v.label_b || '') + '"';
      case 'RelationDrift':
        return 'Relation "' + render.escapeHtml(v.rel_a || '') + '" appears only once — possible typo';
      case 'EntityRefLike':
        return '"' + render.escapeHtml(v.label || '') + '" looks like an entity reference';
      case 'MalformedLine':
        return 'Malformed DSL at "' + render.escapeHtml(v.text || '') + '"';
      case 'EmptyLabel':
        return 'Empty label in DSL';
      case 'DuplicateEdge':
        return 'Duplicate edge: "' + render.escapeHtml(v.from_key || '') + '" > "' + render.escapeHtml(v.rel || '') + '" > "' + render.escapeHtml(v.to_key || '') + '" (first at line ' + (v.existing_line !== undefined ? v.existing_line : '?') + ')';
      default:
        return 'Diagnostic: ' + variant;
    }
  }

  /* Expose for test harness */
  window.renderCmDiagnostics = renderCmDiagnostics;

  function renderAddEdgeForm() {
    var container = document.querySelector('.cm-add-edge-form');
    if (!container) return;

    if (!state.editingConceptMap) {
      container.style.display = 'none';
      return;
    }

    container.style.display = 'block';
    var cm = state.conceptMapCache.get(state.focusId);
    var labels = model.buildNodeLabelList(cm);
    var rels = model.buildRelLabelList(cm);

    var html = '<form class="add-edge-form" onsubmit="return false;">';
    html += '<div class="add-edge-fields">';
    html += '<input type="text" class="cm-input cm-source" list="cm-source-list" placeholder="Source">';
    html += '<datalist id="cm-source-list">' + labels.map(function(l) { return '<option value="' + render.escapeAttr(l) + '">'; }).join('') + '</datalist>';
    html += '<input type="text" class="cm-input cm-rel" list="cm-rel-list" placeholder="relation">';
    html += '<datalist id="cm-rel-list">' + rels.map(function(r) { return '<option value="' + render.escapeAttr(r) + '">'; }).join('') + '</datalist>';
    html += '<input type="text" class="cm-input cm-target" list="cm-target-list" placeholder="Target">';
    html += '<datalist id="cm-target-list">' + labels.map(function(l) { return '<option value="' + render.escapeAttr(l) + '">'; }).join('') + '</datalist>';
    html += '<button type="submit" class="cm-add-btn">Add edge</button>';
    html += '</div>';
    html += '<div class="cm-add-error" style="display:none;"></div>';
    html += '</form>';

    container.innerHTML = html;

    var form = container.querySelector('.add-edge-form');
    form.addEventListener('submit', function() {
      var source = form.querySelector('.cm-source').value;
      var rel = form.querySelector('.cm-rel').value;
      var target = form.querySelector('.cm-target').value;
      handleAddEdge(source, rel, target);
    });
  }

  function updateConceptMapCache(data) {
    var cm = state.conceptMapCache.get(state.focusId);
    if (!cm) return;
    cm.nodes = data.nodes || cm.nodes;
    cm.edges = data.edges || cm.edges;
    cm.diagnostics = data.diagnostics || [];
    cm.dslHash = data.dsl_hash || cm.dslHash;
  }

  function refreshCmView() {
    renderConceptMap();
    renderCmEdgeTable();
    renderAddEdgeForm();
    renderCmDiagnostics();
  }

  function handleAddEdge(source, rel, target) {
    var errorEl = document.querySelector('.cm-add-error');
    if (errorEl) { errorEl.style.display = 'none'; errorEl.textContent = ''; }

    // Client-side trim validation
    source = (source || '').trim();
    rel = (rel || '').trim();
    target = (target || '').trim();

    if (!source) { showCmFormError('Source must not be empty'); return; }
    if (!rel) { showCmFormError('Relation must not be empty'); return; }
    if (!target) { showCmFormError('Target must not be empty'); return; }

    var cm = state.conceptMapCache.get(state.focusId);
    var baseHash = cm ? cm.dslHash : undefined;

    api.mutateConceptMap(state.focusId, 'add_edge', { source: source, rel: rel, target: target }, baseHash)
      .then(function(data) {
        var form = document.querySelector('.add-edge-form');
        if (form) {
          form.querySelector('.cm-source').value = '';
          form.querySelector('.cm-rel').value = '';
          form.querySelector('.cm-target').value = '';
        }
        updateConceptMapCache(data);
        refreshCmView();
      }).catch(function(err) {
        handleMutationError(err);
      });
  }

  function handleRemoveEdge(source, rel, target) {
    var cm = state.conceptMapCache.get(state.focusId);
    var baseHash = cm ? cm.dslHash : undefined;

    api.mutateConceptMap(state.focusId, 'remove_edge', { source: source, rel: rel, target: target }, baseHash)
      .then(function(data) {
        updateConceptMapCache(data);
        refreshCmView();
      }).catch(function(err) {
        handleMutationError(err);
      });
  }

  function startRenameNode(key) {
    if (!state.editingConceptMap) return;
    var cm = state.conceptMapCache.get(state.focusId);
    if (!cm) return;

    var label = key;
    for (var i = 0; i < cm.nodes.length; i++) {
      if (cm.nodes[i].key === key) {
        label = cm.nodes[i].label;
        break;
      }
    }

    state.editingNode = { key: key, label: label };
    renderCmEdgeTable();
  }

  function handleRenameNodeSubmit(newLabel) {
    var oldLabel = state.editingNode ? state.editingNode.label : '';
    state.editingNode = null;

    var newTrimmed = (newLabel || '').trim();
    if (!newTrimmed) {
      showCmFormError('New label must not be empty');
      refreshCmView();
      return;
    }

    var cm = state.conceptMapCache.get(state.focusId);
    var baseHash = cm ? cm.dslHash : undefined;

    api.mutateConceptMap(state.focusId, 'rename_node', { old_label: oldLabel, new_label: newTrimmed }, baseHash)
      .then(function(data) {
        updateConceptMapCache(data);
        refreshCmView();
      }).catch(function(err) {
        if (err.status === 409) {
          var body = typeof err.body === 'string' ? JSON.parse(err.body) : err.body;
          var existingLabel = body.existing_label || '';
          showCmFormError('Rename would collide with existing node \'' + existingLabel + '\'');
        } else {
          handleMutationError(err);
        }
        refreshCmView();
      });
  }

  function handleStaleWrite() {
    // Auto-refetch and notify
    var errorEl = document.querySelector('.cm-add-error');
    if (!errorEl) return;
    errorEl.textContent = 'Concept map was modified elsewhere — data refreshed';
    errorEl.style.display = 'block';
    errorEl.className = 'cm-add-error cm-notice';
    window.setTimeout(function() { if (errorEl) errorEl.style.display = 'none'; }, 4000);

    api.fetchConceptMap(state.focusId).then(function(cm) {
      state.conceptMapCache.set(state.focusId, cm);
      refreshCmView();
    }).catch(function() {});
  }

  function handleMutationError(err) {
    if (err.status === 409) {
      var body;
      try { body = typeof err.body === 'string' ? JSON.parse(err.body) : err.body; } catch (_e) { body = {}; /* eslint-disable-line no-unused-vars */ }
      if (body.error === 'stale_concept_map') {
        handleStaleWrite();
        return;
      }
      if (body.error === 'duplicate_edge') {
        showCmFormError('This edge already exists at line ' + (body.line || '?'));
        return;
      }
      if (body.error === 'node_collision') {
        showCmFormError('Rename would collide with existing node \'' + (body.existing_label || '') + '\'');
        return;
      }
    }
    if (err.status === 400) {
      var b400;
      try { b400 = typeof err.body === 'string' ? JSON.parse(err.body) : err.body; } catch (_e2) { b400 = {}; /* eslint-disable-line no-unused-vars */ }
      if (b400.error === 'empty_field') {
        showCmFormError(b400.message || 'Field must not be empty');
        return;
      }
    }
    if (err.status === 404) {
      showCmFormError('Edge no longer exists — it may have been removed elsewhere');
      return;
    }
    showCmFormError('Error: ' + render.escapeHtml(err.message || 'Unknown error'));
  }

  function showCmFormError(message) {
    var errorEl = document.querySelector('.cm-add-error');
    if (errorEl) {
      errorEl.textContent = message;
      errorEl.style.display = 'block';
      errorEl.className = 'cm-add-error cm-error';
    }
  }

  function renderEditToggle() {
    var header = document.querySelector('.focus-header');
    if (!header) return;

    // Remove existing toggle button
    var existing = header.querySelector('.cm-edit-toggle');
    if (existing) existing.remove();

    if (!state.focusId || !isConceptMap(state.focusId)) return;

    var btn = document.createElement('button');
    btn.className = 'cm-edit-toggle';
    btn.textContent = state.editingConceptMap ? 'Done' : 'Edit';
    btn.addEventListener('click', function() {
      state.editingConceptMap = !state.editingConceptMap;
      if (!state.editingConceptMap) {
        state.editingNode = null;
      }
      renderEditToggle();
      renderCmEdgeTable();
      renderAddEdgeForm();
      renderCmDiagnostics();
      // Re-render SVG to wire/unwire click handlers
      if (state.editingConceptMap) {
        // Re-render to wire click handlers
        renderConceptMap();
      }
    });
    header.appendChild(btn);
  }

  function renderConceptMap() {
    var graphArea = document.querySelector('.graph-area');
    if (!graphArea) return;

    var id = state.focusId;

    if (!state.conceptMapCache.has(id)) {
      graphArea.innerHTML = '<p class="loading">Loading concept map…</p>';
      api.fetchConceptMap(id).then(function(cm) {
        state.conceptMapCache.set(id, cm);
        /* Cold-cache label fix: refresh cmFocusNode.label from freshly-cached nodes */
        if (state.cmFocusNode && state.focusId === id) {
          for (var ci = 0; ci < cm.nodes.length; ci++) {
            if (cm.nodes[ci].key === state.cmFocusNode.key) {
              state.cmFocusNode.label = cm.nodes[ci].label;
              break;
            }
          }
        }
        renderConceptMap();
      }).catch(function(err) {
        if (state.focusId !== id) return;
        graphArea.innerHTML = '<p class="error">Failed to load concept map: ' + render.escapeHtml(err.message) + '</p>';
      });
      return;
    }

    var cm = state.conceptMapCache.get(id);

    /* Apply BFS neighbourhood filtering */
    var filtered = state.cmFocusNode
      ? model.cmNeighbourhood(cm, state.cmFocusNode.key, state.depth)
      : model.cmNeighbourhood(cm, null, state.depth);
    var focusKey = state.cmFocusNode ? state.cmFocusNode.key : null;

    var dotText = dot.cmGraphToDot(filtered, focusKey);

    state.graphRenderSeq += 1;
    var seq = state.graphRenderSeq;

    if (!state.dotAvailable) {
      graphArea.innerHTML = '<p class="error">Graphviz not available.</p><pre>' + render.escapeHtml(dotText) + '</pre>';
      return;
    }

    graphArea.innerHTML = '<p class="loading">Rendering diagram…</p>';

    api.renderDot(dotText).then(function(svgText) {
      if (seq !== state.graphRenderSeq) return;
      var clean = window.DOMPurify.sanitize(svgText, { USE_PROFILES: { svg: true } });
      graphArea.innerHTML = clean;
      var svgEl = graphArea.querySelector('svg');
      if (svgEl) {
        svg.injectHitRects(svgEl);
        svg.wireHandlers(svgEl, function(g) {
          var t = g.querySelector('title');
          return t ? t.textContent.trim() : '';
        }, {
          onClick: function(key) {
            if (state.editingConceptMap) {
              startRenameNode(key);
              return;
            }
            /* View-mode click: toggle cmFocusNode */
            var cm = state.conceptMapCache.get(state.focusId);
            var label = key;
            if (cm) {
              for (var ci = 0; ci < cm.nodes.length; ci++) {
                if (cm.nodes[ci].key === key) {
                  label = cm.nodes[ci].label;
                  break;
                }
              }
            }
            if (state.cmFocusNode && state.cmFocusNode.key === key) {
              state.cmFocusNode = null;
            } else {
              state.cmFocusNode = { key: key, label: label };
            }
            window.location.hash = router.buildHash('focus', state.focusId, state.depth);
            refreshCmView();
          },
          onHoverEnter: function(key) { renderCmHoverPane(key); },
          onHoverLeave: function() { renderCmHoverPane(null); }
        });
      }
      // Render authoring UI sub-views after graph render
      renderCmEdgeTable();
      renderAddEdgeForm();
      renderCmDiagnostics();
      renderEditToggle();
      render.setViewMode('concept-map');
    }).catch(function(err) {
      if (seq !== state.graphRenderSeq) return;
      graphArea.innerHTML = '<p class="error">Graphviz not available</p>';
    });
  }

  // Kick off
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', bootstrap);
  } else {
    bootstrap();
  }
})();
