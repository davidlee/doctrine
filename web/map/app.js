// Doctrine Map Explorer — SPA shell (SL-072 PHASE-07)
//
// Hash routing: #/focus/SL-001 or #/focus/SL-001?depth=2
// Default route (# or missing hash) shows the entity list.
//
// Security: markdown-it configured with html:false; DOMPurify.sanitize()
// applied before innerHTML.  SVG from /api/dot/svg is rendered via <img>
// (data-uri), never injected as inline HTML.

(function () {
  'use strict';

  /* -----------------------------------------------------------------------
   * State
   * --------------------------------------------------------------------- */
  var graphData = null;      // cached /api/graph response
  var dotAvailable = false;  // from /api/health dot.ok
  var md = null;             // markdown-it instance (lazy)
  var currentFocusId = null; // currently displayed entity (for re-render)

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
   * Hash routing
   * --------------------------------------------------------------------- */
  function parseHash() {
    var h = location.hash.slice(1); // strip #
    if (!h) return { view: 'list' };

    var match = h.match(/^\/focus\/([A-Z]+-\d+)(?:\?(.*))?$/);
    if (match) {
      var id = match[1];
      var params = new URLSearchParams(match[2] || '');
      var depth = parseInt(params.get('depth') || '1', 10);
      return { view: 'focus', id: id, depth: depth };
    }

    return { view: 'unknown' };
  }

  /* -----------------------------------------------------------------------
   * API helpers
   * --------------------------------------------------------------------- */
  function apiGet(path) {
    return fetch(path).then(function (r) {
      if (!r.ok) throw new Error(r.status + ' ' + r.statusText);
      return r;
    });
  }

  function apiGetJSON(path) {
    return apiGet(path).then(function (r) { return r.json(); });
  }

  function apiGetText(path) {
    return apiGet(path).then(function (r) { return r.text(); });
  }

  /* -----------------------------------------------------------------------
   * Render: entity list view
   * --------------------------------------------------------------------- */
  function renderList(container) {
    if (!graphData) {
      container.innerHTML = '';
      container.appendChild(el('p', { className: 'loading' }, ['Loading graph…']));
      return;
    }

    var nodes = graphData.nodes || {};
    var ids = Object.keys(nodes).sort();

    container.innerHTML = '';

    var layout = el('div', { className: 'layout' });

    // Sidebar
    var sidebar = el('nav', { className: 'sidebar' });
    sidebar.appendChild(el('h2', { textContent: 'Entities' }));
    var list = el('ul', { className: 'entity-list' });
    ids.forEach(function (id) {
      var node = nodes[id];
      var label = node && node.title ? id + ' — ' + node.title : id;
      var li = el('li', {}, [
        el('a', { href: '#/focus/' + id, textContent: label })
      ]);
      list.appendChild(li);
    });
    sidebar.appendChild(list);
    layout.appendChild(sidebar);

    // Main area — placeholder
    var main = el('main', { className: 'content' });
    main.appendChild(el('p', { className: 'placeholder', textContent: 'Select an entity from the sidebar.' }));
    layout.appendChild(main);

    // DOT editor
    if (dotAvailable) {
      layout.appendChild(renderDotEditor());
    }

    container.appendChild(layout);
  }

  /* -----------------------------------------------------------------------
   * Render: focused entity view (markdown)
   * --------------------------------------------------------------------- */
  function renderFocus(container, id, depth) {
    currentFocusId = id;

    container.innerHTML = '';

    var layout = el('div', { className: 'layout' });

    // Sidebar — compact entity list
    var sidebar = el('nav', { className: 'sidebar' });
    sidebar.appendChild(el('h2', { textContent: 'Entities' }));
    if (graphData) {
      var ids = Object.keys(graphData.nodes || {}).sort();
      var list = el('ul', { className: 'entity-list entity-list--compact' });
      ids.forEach(function (sid) {
        var node = graphData.nodes[sid];
        var label = node && node.title ? sid + ' — ' + node.title : sid;
        var className = sid === id ? 'active' : '';
        var li = el('li', { className: className }, [
          el('a', { href: '#/focus/' + sid, textContent: label })
        ]);
        list.appendChild(li);
      });
      sidebar.appendChild(list);
    }
    layout.appendChild(sidebar);

    // Main area — markdown render
    var main = el('main', { className: 'content' });
    main.appendChild(el('h2', { textContent: id }));
    var mdContainer = el('div', { className: 'markdown-body loading-md' });
    mdContainer.appendChild(el('p', { textContent: 'Loading markdown…' }));
    main.appendChild(mdContainer);
    layout.appendChild(main);

    // DOT editor
    if (dotAvailable) {
      layout.appendChild(renderDotEditor());
    }

    container.appendChild(layout);

    // Fetch markdown asynchronously
    var url = '/api/entity/' + encodeURIComponent(id) + '/markdown';
    apiGetText(url)
      .then(function (text) {
        if (currentFocusId !== id) return; // stale
        mdContainer.className = 'markdown-body';
        mdContainer.innerHTML = renderMarkdown(text);
      })
      .catch(function (err) {
        if (currentFocusId !== id) return; // stale
        mdContainer.className = 'markdown-body';
        mdContainer.innerHTML = '';
        mdContainer.appendChild(
          el('div', { className: 'error' }, ['Failed to load markdown: ' + err.message])
        );
      });
  }

  /* -----------------------------------------------------------------------
   * DOT editor panel
   * --------------------------------------------------------------------- */
  function renderDotEditor() {
    var panel = el('div', { className: 'dot-panel' });

    var heading = el('h3', { textContent: 'DOT Graph Editor' });
    panel.appendChild(heading);

    var textarea = el('textarea', {
      className: 'dot-input',
      placeholder: 'digraph {\n  a -> b;\n}',
      rows: '6'
    });
    panel.appendChild(textarea);

    var actions = el('div', { className: 'dot-actions' });

    var renderBtn = el('button', { textContent: 'Render SVG' });
    renderBtn.addEventListener('click', function () {
      var dot = textarea.value.trim();
      if (!dot) return;
      renderBtn.disabled = true;
      renderBtn.textContent = 'Rendering…';
      renderDotSvg(panel, dot)
        .finally(function () {
          renderBtn.disabled = false;
          renderBtn.textContent = 'Render SVG';
        });
    });
    actions.appendChild(renderBtn);

    panel.appendChild(actions);

    // Output container
    panel.appendChild(el('div', { className: 'dot-output', id: 'dot-output' }));

    return panel;
  }

  function renderDotSvg(panel, dotText) {
    var output = panel.querySelector('#dot-output');
    output.innerHTML = '';
    output.appendChild(el('p', { className: 'loading', textContent: 'Rendering DOT…' }));

    return fetch('/api/dot/svg', {
      method: 'POST',
      body: dotText
    })
      .then(function (r) {
        if (!r.ok) return r.text().then(function (msg) { throw new Error(msg); });
        return r.text();
      })
      .then(function (svg) {
        output.innerHTML = '';
        // Render via <img> data-uri — never inject SVG as inline HTML
        var img = el('img', {
          src: 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svg),
          alt: 'Rendered DOT graph'
        });
        output.appendChild(img);
      })
      .catch(function (err) {
        output.innerHTML = '';
        output.appendChild(
          el('div', { className: 'error' }, ['DOT render failed: ' + err.message])
        );
      });
  }

  /* -----------------------------------------------------------------------
   * SVG Graph rendering (PHASE-03) — rendering pipeline + stale-render guard
   * --------------------------------------------------------------------- */
  function escapeHtml(str) {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
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
   * Bootstrap
   * --------------------------------------------------------------------- */
  function bootstrap() {
    var app = document.getElementById('app');

    // Fetch health (check dot availability) and graph in parallel
    Promise.all([
      apiGetJSON('/api/health').catch(function () { return { ok: false, dot: { ok: false }, graph: { ok: false } }; }),
      apiGetJSON('/api/graph').catch(function () { return null; })
    ]).then(function (results) {
      var health = results[0];
      graphData = results[1];
      dotAvailable = !!(health && health.dot && health.dot.ok);

      render();

      // Listen for hash changes
      window.addEventListener('hashchange', render);
    }).catch(function (err) {
      app.innerHTML = '';
      app.appendChild(
        el('div', { className: 'error' }, ['Failed to initialise: ' + err.message])
      );
    });
  }

  function render() {
    var route = parseHash();
    var app = document.getElementById('app');

    if (route.view === 'focus') {
      renderFocus(app, route.id, route.depth);
    } else {
      renderList(app);
    }
  }

  // Kick off
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', bootstrap);
  } else {
    bootstrap();
  }
})();
