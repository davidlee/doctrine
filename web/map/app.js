// Doctrine Map Explorer — SPA shell (SL-073)
// Hash routing: #/focus/SL-001 or #/focus/SL-001?depth=2
// Security: markdown-it html:false; DOMPurify.sanitize() applied before innerHTML.
// SVG from /api/dot/svg is sanitized via DOMPurify SVG profile, then injected as inline DOM.
/* global state, model, api, router, dot, svg, render, search, cm, compareEdgesBySource */

(function () {
  'use strict';

  /* -----------------------------------------------------------------------
   * State
   * --------------------------------------------------------------------- */

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
   * Entity list node collection helpers
   * --------------------------------------------------------------------- */

  // Build edges array for render.relationshipTable from neighbourhood.
  // Applies kindFilter and returns sorted edges.
  function buildTableEdges() {
    var nb = model.neighbourhood(state.focusId, state.depth, state.graph);
    var edges = nb.edges;
    if (state.kindFilter) {
      edges = edges.filter(function(edge) {
        var src = state.graph.nodes.get(edge.source);
        return src && state.kindFilter.has(src.kindPrefix);
      });
    }
    edges.sort(compareEdgesBySource);
    return edges;
  }

  function applyFilters() {
    search.renderFilteredEntities({
      list: render.elements.entityList,
      graph: state.graph,
      query: document.querySelector('.search-input') ? document.querySelector('.search-input').value : '',
      kindFilter: state.kindFilter,
      focusId: state.focusId,
      onFocus: function(id) { router.setFocus(id, state.depth); }
    });
    render.relationshipTable({
      container: render.elements.relationshipTableBody,
      edges: buildTableEdges(),
      graph: state.graph,
      focusId: state.focusId,
      depth: state.depth
    });
  }

  /* -----------------------------------------------------------------------
   * SVG Graph rendering (PHASE-03) — rendering pipeline + stale-render guard
   * --------------------------------------------------------------------- */
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

    search.wireFilters({
      container: document,
      onChange: function(filterSet) {
        state.kindFilter = filterSet;
        applyFilters();
      }
    });

    search.wireSearch({
      input: document.querySelector('.search-input'),
      list: render.elements.entityList,
      graph: state.graph,
      getFocusId: function() { return state.focusId; },
      getKindFilter: function() { return state.kindFilter; },
      onFocus: function(id) { router.setFocus(id, state.depth); }
    });

    search.wireDepthButtons({
      container: document,
      onDepthChange: function(d) {
        if (state.focusId) { router.setFocus(state.focusId, d); }
      }
    });

    search.wireRefresh({
      button: document.querySelector('.refresh-btn'),
      onRefresh: function() {
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
      }
    });

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
      var edge = state.graph.edgeById.get(route.id);
      render.edgeDetail({
        container: render.elements.graphArea,
        edge: edge,
        graph: state.graph,
        depth: state.depth,
        focusId: state.focusId
      });
      render.hoverPane({ container: render.elements.hoverDetail, node: null });
      render.setViewMode('edge');
      mdPane = document.querySelector('.markdown-pane');
      if (mdPane) mdPane.innerHTML = '<span class="placeholder">[Markdown content]</span>';
      var tbody = document.querySelector('.relationship-table tbody');
      if (tbody) tbody.innerHTML = '<tr><td colspan="5"><span class="placeholder">[Relationship table]</span></td></tr>';
      search.renderFilteredEntities({
        list: render.elements.entityList,
        graph: state.graph,
        query: '',
        kindFilter: state.kindFilter,
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
    search.renderFilteredEntities({
      list: render.elements.entityList,
      graph: state.graph,
      query: document.querySelector('.search-input') ? document.querySelector('.search-input').value : '',
      kindFilter: state.kindFilter,
      focusId: state.focusId,
      onFocus: function(id) { router.setFocus(id, state.depth); }
    });
    render.focusHeader({
      container: render.elements.focusHeader,
      focusId: state.focusId,
      graph: state.graph
    });
    render.relationshipTable({
      container: render.elements.relationshipTableBody,
      edges: buildTableEdges(),
      graph: state.graph,
      focusId: state.focusId,
      depth: state.depth
    });
    render.hoverPane({ container: render.elements.hoverDetail, node: null });

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
          renderCmGraph(graphArea);
        } else {
          state.graphRenderSeq += 1;
          render.graphPane({
            container: graphArea,
            graph: state.graph,
            focusId: state.focusId,
            depth: state.depth,
            dotAvailable: state.dotAvailable,
            seq: state.graphRenderSeq,
            getCurrentSeq: function() { return state.graphRenderSeq; },
            onNodeClick: function(id) { router.setFocus(id, state.depth); },
            onNodeHoverEnter: function(id) {
              state.hoveredId = id;
              render.hoverPane({ container: render.elements.hoverDetail, node: state.graph.nodes.get(id) });
            },
            onNodeHoverLeave: function() {
              state.hoveredId = null;
              render.hoverPane({ container: render.elements.hoverDetail, node: null });
            }
          });
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
    cm.renderEditToggle({
      header: render.elements.focusHeader,
      editing: state.editingConceptMap,
      onToggle: function() {
        state.editingConceptMap = !state.editingConceptMap;
        if (!state.editingConceptMap) state.editingNode = null;
        renderView();
      }
    });
    if (isCm) {
      renderCmEdgeTable();
      renderAddEdgeForm();
      renderCmDiagnostics();
    }

    if (state.focusId) {
      mdPane = document.querySelector('.markdown-pane');
      if (mdPane) render.markdownPane({
        container: mdPane,
        id: state.focusId,
        cache: state.markdownCache,
        currentFocusId: state.focusId
      });
    }
  }

  /* -----------------------------------------------------------------------
  /* -----------------------------------------------------------------------
   * Concept Map rendering (PHASE-05)
   * --------------------------------------------------------------------- */
  function isConceptMap(focusId) {
    var node = state.graph.nodes.get(focusId);
    return node && node.kindPrefix === 'CM';
  }

  function renderCmGraph(container) {
    if (!container) return;
    var id = state.focusId;
    if (!state.conceptMapCache.has(id)) {
      container.innerHTML = '<p class="loading">Loading concept map…</p>';
      api.fetchConceptMap(id).then(function(cm) {
        state.conceptMapCache.set(id, cm);
        if (state.cmFocusNode && state.focusId === id) {
          for (var ci = 0; ci < cm.nodes.length; ci++) {
            if (cm.nodes[ci].key === state.cmFocusNode.key) { state.cmFocusNode.label = cm.nodes[ci].label; break; }
          }
        }
        renderView();
      }).catch(function(err) {
        if (state.focusId !== id) return;
        container.innerHTML = '<p class="error">Failed to load concept map: ' + render.escapeHtml(err.message) + '</p>';
      });
      return;
    }
    var cm = state.conceptMapCache.get(id);
    var filtered = state.cmFocusNode
      ? model.cmNeighbourhood(cm, state.cmFocusNode.key, state.depth)
      : model.cmNeighbourhood(cm, null, state.depth);
    var focusKey = state.cmFocusNode ? state.cmFocusNode.key : null;
    state.graphRenderSeq += 1;
    var seq = state.graphRenderSeq;
    cm.renderDiagram({
      container: container, cm: filtered, focusKey: focusKey, depth: state.depth,
      editing: state.editingConceptMap, dotAvailable: state.dotAvailable,
      seq: seq, getCurrentSeq: function() { return state.graphRenderSeq; },
      onClick: function(key) {
        if (state.editingConceptMap) { startRenameNode(key); return; }
        var cmData = state.conceptMapCache.get(state.focusId), label = key;
        if (cmData) {
          for (var ci = 0; ci < cmData.nodes.length; ci++) {
            if (cmData.nodes[ci].key === key) { label = cmData.nodes[ci].label; break; }
          }
        }
        if (state.cmFocusNode && state.cmFocusNode.key === key) { state.cmFocusNode = null; }
        else { state.cmFocusNode = { key: key, label: label }; }
        window.location.hash = router.buildHash('focus', state.focusId, state.depth);
        renderView();
      },
      onHoverEnter: null, onHoverLeave: null
    });
  }

  function renderCmEdgeTable() {
    var cmCache = state.conceptMapCache.get(state.focusId);
    cm.renderEdgeTable({
      container: document.querySelector('.cm-edge-table'), cm: cmCache,
      focusKey: state.cmFocusNode ? state.cmFocusNode.key : null, depth: state.depth,
      editing: state.editingConceptMap, editingNode: state.editingNode,
      onRemoveEdge: handleRemoveEdge, onRenameNode: startRenameNode,
      onSubmitRename: handleRenameNodeSubmit,
      onCancelRename: function() { state.editingNode = null; renderView(); }
    });
  }

  function renderAddEdgeForm() {
    var cmCache = state.conceptMapCache.get(state.focusId);
    cm.renderAddEdgeForm({
      container: document.querySelector('.cm-add-edge-form'), cm: cmCache,
      editing: state.editingConceptMap, onSubmit: handleAddEdge
    });
  }

  function renderCmDiagnostics() {
    var panel = document.querySelector('.cm-diagnostics-panel');
    if (!panel) return;
    if (state.editingConceptMap) { panel.style.display = 'none'; return; }
    var cmCache = state.conceptMapCache.get(state.focusId);
    if (!cmCache || !cmCache.diagnostics || cmCache.diagnostics.length === 0) { panel.style.display = 'none'; return; }
    cm.renderDiagnostics({ container: panel, diagnostics: cmCache.diagnostics });
  }

  /* Backward-compat shim for test.html */
  window.renderCmDiagnostics = function() {
    var panel = document.querySelector('.cm-diagnostics-panel');
    if (!panel) return;
    if (state.editingConceptMap) { panel.style.display = 'none'; return; }
    var cmCache = state.conceptMapCache.get(state.focusId);
    cm.renderDiagnostics({ container: panel, diagnostics: (cmCache && cmCache.diagnostics) || [] });
  };

  function updateConceptMapCache(data) {
    var cm = state.conceptMapCache.get(state.focusId);
    if (!cm) return;
    cm.nodes = data.nodes || cm.nodes;
    cm.edges = data.edges || cm.edges;
    cm.diagnostics = data.diagnostics || [];
    cm.dslHash = data.dsl_hash || cm.dslHash;
  }

  function refreshCmView() { renderView(); }

  function handleAddEdge(source, rel, target) {
    var errorEl = document.querySelector('.cm-add-error');
    if (errorEl) { errorEl.style.display = 'none'; errorEl.textContent = ''; }
    source = (source || '').trim(); rel = (rel || '').trim(); target = (target || '').trim();
    if (!source) { showCmFormError('Source must not be empty'); return; }
    if (!rel) { showCmFormError('Relation must not be empty'); return; }
    if (!target) { showCmFormError('Target must not be empty'); return; }
    var cm = state.conceptMapCache.get(state.focusId);
    var baseHash = cm ? cm.dslHash : undefined;
    api.mutateConceptMap(state.focusId, 'add_edge', { source: source, rel: rel, target: target }, baseHash)
      .then(function(data) {
        var form = document.querySelector('.add-edge-form');
        if (form) { form.querySelector('.cm-source').value = ''; form.querySelector('.cm-rel').value = ''; form.querySelector('.cm-target').value = ''; }
        updateConceptMapCache(data);
        refreshCmView();
      }).catch(function(err) { handleMutationError(err); });
  }

  function handleRemoveEdge(source, rel, target) {
    var cm = state.conceptMapCache.get(state.focusId);
    var baseHash = cm ? cm.dslHash : undefined;
    api.mutateConceptMap(state.focusId, 'remove_edge', { source: source, rel: rel, target: target }, baseHash)
      .then(function(data) { updateConceptMapCache(data); refreshCmView(); })
      .catch(function(err) { handleMutationError(err); });
  }

  function startRenameNode(key) {
    if (!state.editingConceptMap) return;
    var cm = state.conceptMapCache.get(state.focusId);
    if (!cm) return;
    var label = key;
    for (var i = 0; i < cm.nodes.length; i++) { if (cm.nodes[i].key === key) { label = cm.nodes[i].label; break; } }
    state.editingNode = { key: key, label: label };
    renderCmEdgeTable();
  }

  function handleRenameNodeSubmit(newLabel) {
    var oldLabel = state.editingNode ? state.editingNode.label : '';
    state.editingNode = null;
    var newTrimmed = (newLabel || '').trim();
    if (!newTrimmed) { showCmFormError('New label must not be empty'); refreshCmView(); return; }
    var cm = state.conceptMapCache.get(state.focusId);
    var baseHash = cm ? cm.dslHash : undefined;
    api.mutateConceptMap(state.focusId, 'rename_node', { old_label: oldLabel, new_label: newTrimmed }, baseHash)
      .then(function(data) { updateConceptMapCache(data); refreshCmView(); })
      .catch(function(err) {
        if (err.status === 409) {
          var body = typeof err.body === 'string' ? JSON.parse(err.body) : err.body;
          showCmFormError('Rename would collide with existing node \'' + (body.existing_label || '') + '\'');
        } else { handleMutationError(err); }
        refreshCmView();
      });
  }

  function handleStaleWrite() {
    var errorEl = document.querySelector('.cm-add-error');
    if (!errorEl) return;
    errorEl.textContent = 'Concept map was modified elsewhere — data refreshed';
    errorEl.style.display = 'block'; errorEl.className = 'cm-add-error cm-notice';
    window.setTimeout(function() { if (errorEl) errorEl.style.display = 'none'; }, 4000);
    api.fetchConceptMap(state.focusId).then(function(cm) { state.conceptMapCache.set(state.focusId, cm); refreshCmView(); }).catch(function() {});
  }

  function handleMutationError(err) {
    if (err.status === 409) {
      var body;
      try { body = typeof err.body === 'string' ? JSON.parse(err.body) : err.body; } catch (_e) { body = {}; }
      if (body.error === 'stale_concept_map') { handleStaleWrite(); return; }
      if (body.error === 'duplicate_edge') { showCmFormError('This edge already exists at line ' + (body.line || '?')); return; }
      if (body.error === 'node_collision') { showCmFormError('Rename would collide with existing node \'' + (body.existing_label || '') + '\''); return; }
    }
    if (err.status === 400) {
      var b400;
      try { b400 = typeof err.body === 'string' ? JSON.parse(err.body) : err.body; } catch (_e2) { b400 = {}; }
      if (b400.error === 'empty_field') { showCmFormError(b400.message || 'Field must not be empty'); return; }
    }
    if (err.status === 404) { showCmFormError('Edge no longer exists — it may have been removed elsewhere'); return; }
    showCmFormError('Error: ' + render.escapeHtml(err.message || 'Unknown error'));
  }

  function showCmFormError(message) {
    var errorEl = document.querySelector('.cm-add-error');
    if (errorEl) { errorEl.textContent = message; errorEl.style.display = 'block'; errorEl.className = 'cm-add-error cm-error'; }
  }
  // Kick off
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', bootstrap);
  } else {
    bootstrap();
  }
})();
