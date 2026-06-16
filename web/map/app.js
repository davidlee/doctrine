// Doctrine Map Explorer — SPA shell (SL-073)
// Hash routing: #/focus/SL-001 or #/focus/SL-001?depth=2
// Security: markdown-it html:false; DOMPurify.sanitize() applied before innerHTML.
// SVG from /api/dot/svg is sanitized via DOMPurify SVG profile, then injected as inline DOM.

(function () {
  'use strict';

  /* -----------------------------------------------------------------------
   * State
   * --------------------------------------------------------------------- */
  var md = null;             // markdown-it instance (lazy)

  /* -----------------------------------------------------------------------
   * Utilities
   * --------------------------------------------------------------------- */
  function el(tag, attrs, children) {
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
  }

  function showError(container, msg) {
    container.innerHTML = '';
    container.appendChild(
      el('div', { className: 'error' }, [
        el('p', { textContent: 'Error: ' + msg })
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
   * Interactive UI wiring (PHASE-05) — kind filter, search, depth, refresh,
   * entity list, relationship table, focus header
   * --------------------------------------------------------------------- */
  function renderEntityList() {
    var list = document.querySelector('.entity-list');
    if (!list) return;

    var nodes = [];
    state.graph.nodes.forEach(function(node) { nodes.push(node); });

    if (state.kindFilter) {
      nodes = nodes.filter(function(node) { return state.kindFilter.has(node.kindPrefix); });
    }

    nodes.sort(function(a, b) { return a.id < b.id ? -1 : a.id > b.id ? 1 : 0; });

    list.innerHTML = '';
    nodes.forEach(function(node) {
      var li = document.createElement('li');
      li.className = 'entity-item';
      if (node.id === state.focusId) li.classList.add('active');

      var title = document.createElement('span');
      title.className = 'entity-title';
      title.textContent = node.title;
      li.appendChild(title);

      var pill = document.createElement('span');
      pill.className = 'kind-pill';
      pill.style.background = 'var(--kind-' + node.kindPrefix + ')';
      pill.textContent = node.kindPrefix;
      li.appendChild(pill);

      li.addEventListener('click', (function(id) {
        return function() { router.setFocus(id, state.depth); };
      })(node.id));

      list.appendChild(li);
    });
  }

  function renderFocusHeader() {
    var header = document.querySelector('.focus-header');
    if (!header) return;

    if (!state.focusId) {
      header.innerHTML = '<span class="placeholder">Entity title — kind · status</span>';
      return;
    }

    var node = state.graph.nodes.get(state.focusId);
    if (!node) {
      header.innerHTML = '<span class="placeholder">Entity title — kind · status</span>';
      return;
    }

    header.innerHTML = '<span>' + escapeHtml(node.title) + '</span>' +
      ' <span class="kind-pill" style="background:var(--kind-' + escapeHtml(node.kindPrefix) + ')">' + escapeHtml(node.kindPrefix) + '</span>' +
      ' <span class="status">' + escapeHtml(node.status) + '</span>';
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

    edges.sort(function(a, b) { return a.id < b.id ? -1 : a.id > b.id ? 1 : 0; });

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
          var label = cbs[j].parentNode.textContent.trim();
          var prefixes = label.split('/');
          for (var k = 0; k < prefixes.length; k++) {
            state.kindFilter.add(prefixes[k]);
          }
        }
      }
    }
  }

  function applyFilters() {
    renderEntityList();
    renderRelationshipTable();
  }

  function wireFilterCheckboxes() {
    var cbs = document.querySelectorAll('.kind-checkbox input[type="checkbox"]');
    for (var i = 0; i < cbs.length; i++) {
      cbs[i].addEventListener('change', function() {
        collectKindFilter();
        applyFilters();
      });
    }
    collectKindFilter();
  }

  function wireSearch() {
    var input = document.querySelector('.search-input');
    if (!input) return;

    input.addEventListener('input', function() {
      var query = input.value.trim();
      var list = document.querySelector('.entity-list');
      if (!list) return;

      if (!query) { renderEntityList(); return; }

      var results = model.searchFilter(query, state.graph);
      if (state.kindFilter) {
        results = results.filter(function(node) { return state.kindFilter.has(node.kindPrefix); });
      }

      list.innerHTML = '';
      results.forEach(function(node) {
        var li = document.createElement('li');
        li.className = 'entity-item';
        if (node.id === state.focusId) li.classList.add('active');
        var t = document.createElement('span'); t.className = 'entity-title'; t.textContent = node.title; li.appendChild(t);
        var p = document.createElement('span'); p.className = 'kind-pill';
        p.style.background = 'var(--kind-' + node.kindPrefix + ')'; p.textContent = node.kindPrefix; li.appendChild(p);
        li.addEventListener('click', (function(id) {
          return function() { router.setFocus(id, state.depth); };
        })(node.id));
        list.appendChild(li);
      });
    });

    input.addEventListener('keydown', function(e) {
      if (e.key === 'Enter') {
        var query = input.value.trim();
        if (!query) return;
        var result = model.findFocus(query, state.graph);
        if (result) {
          router.setFocus(result, state.depth);
        } else {
          var list = document.querySelector('.entity-list');
          if (list) {
            list.innerHTML = '<li class="entity-item"><span class="placeholder">No match for \'' + escapeHtml(query) + '\'</span></li>';
          }
        }
      } else if (e.key === 'Escape') {
        input.value = '';
        renderEntityList();
      }
    });
  }

  function wireDepthButtons() {
    var btns = document.querySelectorAll('.depth-btn');
    for (var i = 0; i < btns.length; i++) {
      btns[i].addEventListener('click', (function(d) {
        return function() {
          state.depth = d;
          var allBtns = document.querySelectorAll('.depth-btn');
          for (var j = 0; j < allBtns.length; j++) {
            allBtns[j].classList.toggle('active', parseInt(allBtns[j].textContent, 10) === d);
          }
          if (state.focusId) { router.setFocus(state.focusId, d); }
        };
      })(parseInt(btns[i].textContent, 10)));
    }
  }

  function wireRefresh() {
    var btn = document.querySelector('.refresh-btn');
    if (!btn) return;
    btn.addEventListener('click', function() {
      state.markdownCache.clear();
      state.graphRenderSeq += 1;
      api.refreshGraph().then(function() {
        return api.fetchGraph();
      }).then(function(raw) {
        model.normalizeGraph(raw);
        if (state.focusId) {
          state.focusId = model.resolveFocus(state.focusId, state.graph);
        }
        render();
      }).catch(function(err) {
        var app = document.getElementById('app');
        showError(app, 'Failed to refresh: ' + err.message);
      });
    });
  }

  /* -----------------------------------------------------------------------
   * SVG Graph rendering (PHASE-03) — rendering pipeline + stale-render guard
   * --------------------------------------------------------------------- */
  function escapeHtml(str) {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
  }

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
    html += '<span class="hover-detail-title">' + node.id + ': ' + escapeHtml(node.title) + '</span>';
    html += '<span class="hover-detail-meta">' + node.kindLabel + ' \u00b7 ' + node.status + '</span>';
    html += '</div>';
    pane.innerHTML = html;
  }

  function wireSvgHandlers(svgEl, edges) {
    var groups = svgEl.querySelectorAll('.node');
    for (var i = 0; i < groups.length; i++) {
      var g = groups[i];
      var titleEl = g.querySelector('title');
      if (!titleEl) continue;
      var nodeId = titleEl.textContent.trim();

      g.classList.add('doctrine-node');

      g.addEventListener('click', (function(id) {
        return function() {
          router.setFocus(id, state.depth);
        };
      })(nodeId));

      g.addEventListener('mouseenter', (function(id) {
        return function() {
          state.hoveredId = id;
          renderHoverPane(id);
        };
      })(nodeId));

      g.addEventListener('mouseleave', function() {
        state.hoveredId = null;
        renderHoverPane(null);
      });
    }
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
        wireSvgHandlers(svgEl, nb.edges);
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

  function renderMarkdownPane(container, id) {
    // Cache check
    if (state.markdownCache.has(id)) {
      container.innerHTML = renderMarkdown(state.markdownCache.get(id));
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
      container.innerHTML = renderMarkdown(text);
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
  function bootstrap() {
    // Wire interactive surfaces
    wireFilterCheckboxes();
    wireSearch();
    wireDepthButtons();
    wireRefresh();

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

      render();
      window.addEventListener('hashchange', render);
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
        container.innerHTML = '<p class="error">Edge ' + escapeHtml(id) + ' not found in graph</p>';
      }
      return;
    }

    var srcNode = state.graph.nodes.get(edge.source);
    var tgtNode = state.graph.nodes.get(edge.target);
    var originFile = edge.raw && edge.raw.origin && edge.raw.origin.file ? edge.raw.origin.file : '-';

    var html = '<div class="edge-detail">';
    html += '<h2>Edge: ' + escapeHtml(edge.id) + '</h2>';
    html += '<table class="edge-detail-table">';
    html += '<tr><th>Edge ID</th><td>' + escapeHtml(edge.id) + '</td></tr>';
    html += '<tr><th>Source</th><td><a href="#' + router.buildHash('focus', edge.source, state.depth) + '">' + escapeHtml(edge.source) + '</a>' + (srcNode ? ' &mdash; ' + escapeHtml(srcNode.title) : '') + '</td></tr>';
    html += '<tr><th>Label</th><td>' + escapeHtml(edge.label) + '</td></tr>';
    html += '<tr><th>Target</th><td><a href="#' + router.buildHash('focus', edge.target, state.depth) + '">' + escapeHtml(edge.target) + '</a>' + (tgtNode ? ' &mdash; ' + escapeHtml(tgtNode.title) : '') + '</td></tr>';
    html += '<tr><th>Origin file</th><td>' + escapeHtml(originFile) + '</td></tr>';
    html += '</table>';
    html += '<p class="edge-detail-back"><a href="#' + router.buildHash('focus', state.focusId, state.depth) + '">&larr; Back to ' + escapeHtml(state.focusId) + '</a></p>';
    html += '</div>';

    if (container) container.innerHTML = html;
  }

  function render() {
    var route = router.parseHash();

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
      var mdPane = document.querySelector('.markdown-pane');
      if (mdPane) mdPane.innerHTML = '<span class="placeholder">[Markdown content]</span>';
      var tbody = document.querySelector('.relationship-table tbody');
      if (tbody) tbody.innerHTML = '<tr><td colspan="5"><span class="placeholder">[Relationship table]</span></td></tr>';
      renderEntityList();
      renderFocusHeader();
      return;
    }

    renderEntityList();
    renderFocusHeader();

    if (state.focusId) {
      var graphArea = document.querySelector('.graph-area');
      if (graphArea) renderGraphPane(graphArea, state.focusId, state.depth);
    }

    renderHoverPane(null);
    renderRelationshipTable();

    if (state.focusId) {
      var mdPane = document.querySelector('.markdown-pane');
      if (mdPane) renderMarkdownPane(mdPane, state.focusId);
    }
  }

  // Kick off
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', bootstrap);
  } else {
    bootstrap();
  }
})();
