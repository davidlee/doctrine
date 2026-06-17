// Doctrine Map Explorer — SPA shell (SL-073)
// Hash routing: #/focus/SL-001 or #/focus/SL-001?depth=2
// Security: markdown-it html:false; DOMPurify.sanitize() applied before innerHTML.
/* global state, model, api, router, svg, render, search, cm, compareEdgesBySource, priority */

(function () {
  'use strict';

  // F-16: safe localStorage wrapper — degrades silently when storage is unavailable
  var safeStorage = { get: function(k, fb) { try { var v = localStorage.getItem(k); return v !== null ? v : fb; } catch (_) { return fb; } }, set: function(k, v) { try { localStorage.setItem(k, v); } catch (_) {} } };
  var goto = function(id) { router.setFocus(id, state.depth); };

  function showError(container, msg) { container.innerHTML = ''; container.appendChild(render.el('div', { className: 'error' }, [render.el('p', { textContent: 'Error: ' + msg })])); }
  function buildTableEdges() { var nb = model.neighbourhood(state.focusId, state.depth, state.graph), edges = nb.edges; if (state.kindFilter) edges = edges.filter(function(e) { var s = state.graph.nodes.get(e.source); return s && state.kindFilter.has(s.kindPrefix); }); edges.sort(compareEdgesBySource); return edges; }
  function applyFilters() { var qEl = document.querySelector('.search-input'); search.renderFilteredEntities({ list: render.elements.entityList, graph: state.graph, query: qEl ? qEl.value : '', kindFilter: state.kindFilter, focusId: state.focusId, onFocus: goto }); if (state.viewMode === 'actionability') render.relationshipTable({ container: render.elements.relationshipTableBody, graph: state.graph, focusId: state.focusId, depth: state.depth, viewMode: 'actionability', actionabilityView: state.actionabilityView }); else render.relationshipTable({ container: render.elements.relationshipTableBody, edges: buildTableEdges(), graph: state.graph, focusId: state.focusId, depth: state.depth, viewMode: 'semantic' }); }
  function isConceptMap(focusId) { var n = state.graph.nodes.get(focusId); return n && n.kindPrefix === 'CM'; }
  function refreshCmView() { renderView(); }

  function wireTableToggle() {
    var cb = document.getElementById('hide-relations'), table = document.querySelector('.relationship-table');
    if (!cb || !table) return;
    var hidden = safeStorage.get('doctrine-map-hide-relations', '0') === '1'; cb.checked = hidden; table.classList.toggle('hidden', hidden);
    cb.addEventListener('change', function() { table.classList.toggle('hidden', cb.checked); safeStorage.set('doctrine-map-hide-relations', cb.checked ? '1' : '0'); });
  }

  function bootstrap() {
    render.cacheElements(document); wireTableToggle();
    var viewBtns = document.querySelectorAll('.view-btn'), vb;
    for (vb = 0; vb < viewBtns.length; vb++) {
      viewBtns[vb].addEventListener('click', function() {
        state.viewMode = this.getAttribute('data-view') || 'semantic';
        state.priorityZoomId = null;
        renderView();
      });
    }
    search.wireFilters({ container: document, onChange: function(filterSet) { state.kindFilter = filterSet; applyFilters(); } });
    search.wireSearch({ input: document.querySelector('.search-input'), list: render.elements.entityList, graph: state.graph, getFocusId: function() { return state.focusId; }, getKindFilter: function() { return state.kindFilter; }, onFocus: goto });
    search.wireDepthButtons({ container: document, onDepthChange: function(d) { if (state.focusId) router.setFocus(state.focusId, d); } });
    search.wireRefresh({ button: document.querySelector('.refresh-btn'), onRefresh: function() { state.markdownCache.clear(); state.conceptMapCache.clear(); state.cmFocusNode = null; state.graphRenderSeq += 1; state.actionabilityView = null; api.refreshGraph().then(function() { return api.fetchGraph(); }).then(function(raw) { model.normalizeGraph(raw); if (state.focusId) state.focusId = model.resolveFocus(state.focusId, state.graph); renderView(); }).catch(function(err) { showError(document.getElementById('app'), 'Failed to refresh: ' + err.message); }); } });
    window.addEventListener('hashchange', renderView);
    Promise.all([api.fetchHealth().catch(function() { return { dot: { ok: false }, graph: { ok: false } }; }), api.fetchGraph().catch(function() { return null; }), api.fetchActionabilityGraph().catch(function() { return null; })]).then(function(results) { var health = results[0], raw = results[1]; state.dotAvailable = !!(health && health.dot && health.dot.ok); if (raw) model.normalizeGraph(raw); if (results[2]) model.setActionabilityView(results[2]); if (!state.focusId && state.graph.nodes.size > 0) { state.focusId = model.resolveFocus(null, state.graph); if (state.focusId) { router.setFocus(state.focusId, state.depth); return; } } renderView(); }).catch(function(err) { showError(document.getElementById('app'), 'Failed to initialise: ' + err.message); });
  }

  function renderView() {
    var route = router.parseHash(), prevFocusId = state.focusId, prevDepth = state.depth;
    if (route.view === 'focus') state.focusId = route.id;
    state.depth = Math.max(0, Math.min(3, route.depth));
    if (route.view === 'edge' && !state.focusId && state.graph.nodes.size > 0) state.focusId = model.resolveFocus(null, state.graph);

    if (route.view === 'edge') {
      render.edgeDetail({ container: render.elements.graphArea, edge: state.graph.edgeById.get(route.id), graph: state.graph, depth: state.depth, focusId: state.focusId });
      render.hoverPane({ container: render.elements.hoverDetail, node: null }); render.setViewMode('edge');
      var mp = document.querySelector('.markdown-pane'); if (mp) mp.innerHTML = '<span class="placeholder">[Markdown content]</span>';
      var tbody = document.querySelector('.relationship-table tbody'); if (tbody) tbody.innerHTML = '<tr><td colspan="5"><span class="placeholder">[Relationship table]</span></td></tr>';
      search.renderFilteredEntities({ list: render.elements.entityList, graph: state.graph, query: '', kindFilter: state.kindFilter, focusId: state.focusId, onFocus: goto });
      render.focusHeader({ container: render.elements.focusHeader, focusId: state.focusId, graph: state.graph }); return;
    }

    var qEl = document.querySelector('.search-input');
    search.renderFilteredEntities({ list: render.elements.entityList, graph: state.graph, query: qEl ? qEl.value : '', kindFilter: state.kindFilter, focusId: state.focusId, onFocus: goto });
    render.focusHeader({ container: render.elements.focusHeader, focusId: state.focusId, graph: state.graph });
    render.hoverPane({ container: render.elements.hoverDetail, node: null });

    var depthBtns = document.querySelectorAll('.depth-btn'), di;
    for (di = 0; di < depthBtns.length; di++) depthBtns[di].classList.toggle('active', parseInt(depthBtns[di].getAttribute('data-depth'), 10) === state.depth);

    var graphArea = document.querySelector('.graph-area'), focusChanged = state.focusId !== prevFocusId, depthChanged = state.depth !== prevDepth, graphMissing = !graphArea || !graphArea.querySelector('svg');
    var cmFocusChanged = state.focusId && isConceptMap(state.focusId) && (state.cmFocusNode ? state.cmFocusNode.key : null) !== state.renderedCmFocus;
    var cmCacheChanged = state.focusId && isConceptMap(state.focusId) && state.cmCacheMutationSeq !== state.renderedCmCacheSeq;
    if (focusChanged && prevFocusId && isConceptMap(prevFocusId)) state.conceptMapCache.delete(prevFocusId);

    if (state.viewMode === 'actionability') {
      if (!state.actionabilityView) {
        if (graphArea) graphArea.innerHTML = '<p class="loading">Loading actionability graph…</p>';
        api.fetchActionabilityGraph().then(function(result) { model.setActionabilityView(result); renderView(); }).catch(function(err) { if (graphArea) graphArea.innerHTML = '<p class="error">Failed to load actionability graph: ' + render.escapeHtml(err.message) + '</p>'; });
      } else if (graphArea) {
        var actionabilityNodes = Array.isArray(state.actionabilityView.nodes) ? state.actionabilityView.nodes : [];
        if (actionabilityNodes.length > 0) {
          priority.renderGraph({
            container: graphArea,
            layout: priority.layoutGraph(state.actionabilityView),
            focusId: state.focusId,
            zoomId: state.priorityZoomId,
            onZoomToggle: function(id) { state.priorityZoomId = id; renderView(); },
            depth: state.depth,
            onNodeClick: goto,
            onNodeHoverEnter: function(id) {
              var hoveredNode = null, ai;
              state.hoveredId = id;
              for (ai = 0; ai < actionabilityNodes.length; ai++) {
                if (actionabilityNodes[ai].id === id) {
                  hoveredNode = { id: id, title: actionabilityNodes[ai].title || '', kindLabel: actionabilityNodes[ai].kind || '', status: actionabilityNodes[ai].status || '' };
                  break;
                }
              }
              render.hoverPane({ container: render.elements.hoverDetail, node: hoveredNode });
            },
            onNodeHoverLeave: function() { state.hoveredId = null; render.hoverPane({ container: render.elements.hoverDetail, node: null }); }
          });
        } else if (state.focusId) {
          var focusNode = state.graph.nodes.get(state.focusId);
          var isWorkKind = focusNode && (focusNode.kindPrefix === 'SL' || focusNode.kindPrefix === 'ISS' || focusNode.kindPrefix === 'IMP' || focusNode.kindPrefix === 'CHR' || focusNode.kindPrefix === 'RSK');
          if (isWorkKind) {
            graphArea.innerHTML = '<p class="placeholder">' + render.escapeHtml(state.focusId) + ' is ' + (focusNode.status || 'terminal') + ' — terminal items don\'t appear in the actionability graph.</p>';
          } else {
            graphArea.innerHTML = '<p class="placeholder">This entity has no dep/seq edges — switch to a work entity (SL/backlog) or use Semantic view.</p>';
          }
        } else {
          graphArea.innerHTML = '<p class="placeholder">No eligible work items found.</p>';
        }
      }
    } else if (graphArea && (focusChanged || depthChanged || graphMissing || cmFocusChanged || cmCacheChanged)) {
      if (focusChanged && !depthChanged && state.focusId) { var svgEl = graphArea.querySelector('svg'); if (svgEl) svg.applyFocusHighlight(svgEl, state.focusId, prevFocusId, function(g) { var t = g.querySelector('text'); if (t) return t.textContent.trim(); var ti = g.querySelector('title'); return ti ? ti.textContent.trim() : ''; }); }
      if (state.focusId) {
        if (isConceptMap(state.focusId)) renderCmGraph(graphArea);
        else { state.renderedCmFocus = null; state.graphRenderSeq += 1; render.graphPane({ container: graphArea, graph: state.graph, focusId: state.focusId, depth: state.depth, dotAvailable: state.dotAvailable, seq: state.graphRenderSeq, getCurrentSeq: function() { return state.graphRenderSeq; }, onNodeClick: goto, onNodeHoverEnter: function(id) { state.hoveredId = id; render.hoverPane({ container: render.elements.hoverDetail, node: state.graph.nodes.get(id) }); }, onNodeHoverLeave: function() { state.hoveredId = null; render.hoverPane({ container: render.elements.hoverDetail, node: null }); } }); }
      }
    }

    var legendItems = document.querySelector('.edge-legend .legend-items');
    var priorityLegend = document.querySelector('.priority-legend');
    if (state.viewMode === 'actionability') {
      render.relationshipTable({ container: render.elements.relationshipTableBody, graph: state.graph, focusId: state.focusId, depth: state.depth, viewMode: 'actionability', actionabilityView: state.actionabilityView });
      if (priorityLegend) priorityLegend.style.display = '';
      if (legendItems) legendItems.style.display = 'none';
    } else {
      render.relationshipTable({ container: render.elements.relationshipTableBody, edges: buildTableEdges(), graph: state.graph, focusId: state.focusId, depth: state.depth, viewMode: 'semantic' });
      if (legendItems) legendItems.style.display = '';
      if (priorityLegend) priorityLegend.style.display = 'none';
    }

    var viewBtns = document.querySelectorAll('.view-btn'), vi;
    for (vi = 0; vi < viewBtns.length; vi++) viewBtns[vi].classList.toggle('active', viewBtns[vi].getAttribute('data-view') === state.viewMode);

    var isCm = state.viewMode === 'semantic' && state.focusId && isConceptMap(state.focusId); render.setViewMode(state.viewMode === 'actionability' ? 'actionability' : (isCm ? 'concept-map' : 'entity-graph'));
    if (isCm && route.cmFocus) { var cachedCm = state.conceptMapCache.get(state.focusId), label = route.cmFocus; if (cachedCm) for (var ci = 0; ci < cachedCm.nodes.length; ci++) { if (cachedCm.nodes[ci].key === route.cmFocus) { label = cachedCm.nodes[ci].label; break; } } if (!state.cmFocusNode || state.cmFocusNode.key !== route.cmFocus) state.cmFocusNode = { key: route.cmFocus, label: label }; }
    else if (focusChanged) state.cmFocusNode = null;

    cm.renderEditToggle({ header: render.elements.focusHeader, editing: state.editingConceptMap, onToggle: function() { state.editingConceptMap = !state.editingConceptMap; if (!state.editingConceptMap) state.editingNode = null; renderView(); } });
    if (isCm) { renderCmEdgeTable(); renderCmAddEdgeForm(); renderCmDiagnostics(); }

    if (state.focusId) { var mdPane = document.querySelector('.markdown-pane'); if (mdPane) render.markdownPane({ container: mdPane, id: state.focusId, cache: state.markdownCache, currentFocusId: state.focusId }); }
  }

  function renderCmGraph(container) {
    if (!container) return; var id = state.focusId;
    if (!state.conceptMapCache.has(id)) { container.innerHTML = '<p class="loading">Loading concept map…</p>'; api.fetchConceptMap(id).then(function(cm) { state.conceptMapCache.set(id, cm); if (state.cmFocusNode && state.focusId === id) for (var ci = 0; ci < cm.nodes.length; ci++) { if (cm.nodes[ci].key === state.cmFocusNode.key) { state.cmFocusNode.label = cm.nodes[ci].label; break; } } renderView(); }).catch(function(err) { if (state.focusId !== id) return; container.innerHTML = '<p class="error">Failed to load concept map: ' + render.escapeHtml(err.message) + '</p>'; }); return; }
    var cmCache = state.conceptMapCache.get(id), filtered = state.cmFocusNode ? model.cmNeighbourhood(cmCache, state.cmFocusNode.key, state.depth) : model.cmNeighbourhood(cmCache, null, state.depth), focusKey = state.cmFocusNode ? state.cmFocusNode.key : null;
    state.renderedCmFocus = focusKey;
    state.renderedCmCacheSeq = state.cmCacheMutationSeq;
    state.graphRenderSeq += 1; var seq = state.graphRenderSeq;
    cm.renderDiagram({ container: container, cm: filtered, focusKey: focusKey, depth: state.depth, editing: state.editingConceptMap, dotAvailable: state.dotAvailable, seq: seq, getCurrentSeq: function() { return state.graphRenderSeq; }, onClick: function(key) { if (state.editingConceptMap) { startRenameNode(key); return; } var cmData = state.conceptMapCache.get(state.focusId), label = key; if (cmData) for (var ci = 0; ci < cmData.nodes.length; ci++) { if (cmData.nodes[ci].key === key) { label = cmData.nodes[ci].label; break; } } if (state.cmFocusNode && state.cmFocusNode.key === key) state.cmFocusNode = null; else state.cmFocusNode = { key: key, label: label }; window.location.hash = router.buildHash('focus', state.focusId, state.depth); renderView(); }, onHoverEnter: null, onHoverLeave: null });
  }

  function renderCmEdgeTable() { var cmCache = state.conceptMapCache.get(state.focusId); cm.renderEdgeTable({ container: document.querySelector('.cm-edge-table'), cm: cmCache, focusKey: state.cmFocusNode ? state.cmFocusNode.key : null, depth: state.depth, editing: state.editingConceptMap, editingNode: state.editingNode, onRemoveEdge: handleRemoveEdge, onRenameNode: startRenameNode, onSubmitRename: handleRenameNodeSubmit, onCancelRename: function() { state.editingNode = null; renderView(); } }); }
  function renderCmAddEdgeForm() { var cmCache = state.conceptMapCache.get(state.focusId); cm.renderAddEdgeForm({ container: document.querySelector('.cm-add-edge-form'), cm: cmCache, editing: state.editingConceptMap, onSubmit: handleAddEdge }); }
  function renderCmDiagnostics() { var panel = document.querySelector('.cm-diagnostics-panel'); if (!panel) return; if (state.editingConceptMap) { panel.style.display = 'none'; return; } var cmCache = state.conceptMapCache.get(state.focusId); if (!cmCache || !cmCache.diagnostics || cmCache.diagnostics.length === 0) { panel.style.display = 'none'; return; } cm.renderDiagnostics({ container: panel, diagnostics: cmCache.diagnostics }); }

  window.renderCmDiagnostics = function() { var p = document.querySelector('.cm-diagnostics-panel'); if (!p) return; if (state.editingConceptMap) { p.style.display = 'none'; return; } var c = state.conceptMapCache.get(state.focusId); cm.renderDiagnostics({ container: p, diagnostics: (c && c.diagnostics) || [] }); };

  function updateConceptMapCache(data) { var cm = state.conceptMapCache.get(state.focusId); if (!cm) return; cm.nodes = data.nodes || cm.nodes; cm.edges = data.edges || cm.edges; cm.diagnostics = data.diagnostics || []; cm.dslHash = data.dsl_hash || cm.dslHash; state.cmCacheMutationSeq += 1; }
  function handleAddEdge(source, rel, target) { var errEl = document.querySelector('.cm-add-error'); if (errEl) { errEl.style.display = 'none'; errEl.textContent = ''; } source = (source || '').trim(); rel = (rel || '').trim(); target = (target || '').trim(); if (!source) { showCmFormError('Source must not be empty'); return; } if (!rel) { showCmFormError('Relation must not be empty'); return; } if (!target) { showCmFormError('Target must not be empty'); return; } var cm = state.conceptMapCache.get(state.focusId), baseHash = cm ? cm.dslHash : undefined; api.mutateConceptMap(state.focusId, 'add_edge', { source: source, rel: rel, target: target }, baseHash).then(function(data) { var f = document.querySelector('.add-edge-form'); if (f) { f.querySelector('.cm-source').value = ''; f.querySelector('.cm-rel').value = ''; f.querySelector('.cm-target').value = ''; } updateConceptMapCache(data); refreshCmView(); }).catch(function(err) { handleMutationError(err); }); }
  function handleRemoveEdge(source, rel, target) { var cm = state.conceptMapCache.get(state.focusId), baseHash = cm ? cm.dslHash : undefined; api.mutateConceptMap(state.focusId, 'remove_edge', { source: source, rel: rel, target: target }, baseHash).then(function(data) { updateConceptMapCache(data); refreshCmView(); }).catch(function(err) { handleMutationError(err); }); }
  function startRenameNode(key) { if (!state.editingConceptMap) return; var cm = state.conceptMapCache.get(state.focusId); if (!cm) return; var label = key, i; for (i = 0; i < cm.nodes.length; i++) { if (cm.nodes[i].key === key) { label = cm.nodes[i].label; break; } } state.editingNode = { key: key, label: label }; renderCmEdgeTable(); }
  function handleRenameNodeSubmit(newLabel) { var oldLabel = state.editingNode ? state.editingNode.label : ''; state.editingNode = null; var nt = (newLabel || '').trim(); if (!nt) { showCmFormError('New label must not be empty'); refreshCmView(); return; } var cm = state.conceptMapCache.get(state.focusId), baseHash = cm ? cm.dslHash : undefined; api.mutateConceptMap(state.focusId, 'rename_node', { old_label: oldLabel, new_label: nt }, baseHash).then(function(data) { updateConceptMapCache(data); refreshCmView(); }).catch(function(err) { if (err.status === 409) { var body = typeof err.body === 'string' ? JSON.parse(err.body) : err.body; showCmFormError('Rename would collide with existing node \'' + (body.existing_label || '') + '\''); } else handleMutationError(err); refreshCmView(); }); }
  function handleStaleWrite() { var errEl = document.querySelector('.cm-add-error'); if (!errEl) return; errEl.textContent = 'Concept map was modified elsewhere — data refreshed'; errEl.style.display = 'block'; errEl.className = 'cm-add-error cm-notice'; window.setTimeout(function() { if (errEl) errEl.style.display = 'none'; }, 4000); api.fetchConceptMap(state.focusId).then(function(cm) { state.conceptMapCache.set(state.focusId, cm); refreshCmView(); }).catch(function() {}); }
  function handleMutationError(err) { if (err.status === 409) { var body; try { body = typeof err.body === 'string' ? JSON.parse(err.body) : err.body; } catch (_) { body = {}; } if (body.error === 'stale_concept_map') { handleStaleWrite(); return; } if (body.error === 'duplicate_edge') { showCmFormError('This edge already exists at line ' + (body.line || '?')); return; } if (body.error === 'node_collision') { showCmFormError('Rename would collide with existing node \'' + (body.existing_label || '') + '\''); return; } } if (err.status === 400) { var b400; try { b400 = typeof err.body === 'string' ? JSON.parse(err.body) : err.body; } catch (_) { b400 = {}; } if (b400.error === 'empty_field') { showCmFormError(b400.message || 'Field must not be empty'); return; } } if (err.status === 404) { showCmFormError('Edge no longer exists — it may have been removed elsewhere'); return; } showCmFormError('Error: ' + render.escapeHtml(err.message || 'Unknown error')); }
  function showCmFormError(msg) { var errEl = document.querySelector('.cm-add-error'); if (errEl) { errEl.textContent = msg; errEl.style.display = 'block'; errEl.className = 'cm-add-error cm-error'; } }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', bootstrap); else bootstrap();
})();
