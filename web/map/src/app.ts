import './style.css'

import type { Edge, ConceptMap } from './types'
import { state } from './state'
import { normalizeGraph, resolveFocus, setActionabilityView, neighbourhood, cmNeighbourhood, compareEdgesBySource } from './model'
import { parseHash, buildHash, setFocus } from './router'
import { ApiError, fetchGraph, fetchActionabilityGraph, refreshGraph, fetchHealth, fetchConceptMap, mutateConceptMap } from './api'
import { elements, el, cacheElements, escapeHtml, focusHeader, setPageMode, relationshipTable, hoverPane, markdownPane, graphPane, edgeDetail } from './render'
import { renderFilteredEntities, wireFilters, wireSearch, wireDepthButtons, wireRefresh } from './search'
import { renderDiagram, renderEdgeTable, renderAddEdgeForm, renderDiagnostics, renderEditToggle } from './concept-map'
import { renderGraph } from './priority'
import { applyFocusHighlight } from './svg'

// ---------------------------------------------------------------------------
// safeStorage — localStorage wrapper (module-private)
// ---------------------------------------------------------------------------

const safeStorage = {
  get(k: string, fb: string): string {
    try {
      const v = localStorage.getItem(k)
      return v ?? fb
    } catch {
      return fb
    }
  },
  set(k: string, v: string): void {
    try {
      localStorage.setItem(k, v)
    } catch {
      /* degrade silently */
    }
  },
}

// ---------------------------------------------------------------------------
// goto — navigate to entity (module-private)
// ---------------------------------------------------------------------------

function goto(id: string): void {
  setFocus(id, state.depth)
}

// ---------------------------------------------------------------------------
// showError — display error in container (module-private)
// ---------------------------------------------------------------------------

function showError(container: HTMLElement, msg: string): void {
  container.replaceChildren()
  container.appendChild(el('div', { className: 'error' }, [el('p', { textContent: 'Error: ' + msg })]))
}

// ---------------------------------------------------------------------------
// buildTableEdges — collect + filter + sort edges (module-private)
// ---------------------------------------------------------------------------

function buildTableEdges(): Edge[] {
  const nb = neighbourhood(state.focusId ?? '', state.depth, state.graph)
  let filtered = nb.edges
  const kindFilter = state.kindFilter
  if (kindFilter !== null) {
    filtered = filtered.filter((e) => {
      const s = state.graph.nodes.get(e.source)
      return s !== undefined && kindFilter.has(s.kindPrefix)
    })
  }
  filtered.sort(compareEdgesBySource)
  return filtered
}

// ---------------------------------------------------------------------------
// applyFilters — re-render entity list + relationship table (module-private)
// ---------------------------------------------------------------------------

function applyFilters(): void {
  const entityListEl = elements.entityList
  const relTableBody = elements.relationshipTableBody
  if (entityListEl === null || relTableBody === null) return

  const qEl = document.querySelector<HTMLInputElement>('.search-input')

  renderFilteredEntities({
    list: entityListEl,
    graph: state.graph,
    query: qEl?.value ?? '',
    kindFilter: state.kindFilter,
    focusId: state.focusId,
    onFocus: goto,
  })

  if (state.viewMode === 'actionability') {
    relationshipTable({
      container: relTableBody,
      graph: state.graph,
      focusId: state.focusId,
      depth: state.depth,
      viewMode: 'actionability',
      actionabilityView: state.actionabilityView,
    })
  } else {
    relationshipTable({
      container: relTableBody,
      edges: buildTableEdges(),
      graph: state.graph,
      focusId: state.focusId,
      depth: state.depth,
      viewMode: 'semantic',
    })
  }
}

// ---------------------------------------------------------------------------
// isConceptMap — check if focusId is a concept map entity (module-private)
// ---------------------------------------------------------------------------

function isConceptMap(focusId: string | null): boolean {
  if (focusId === null) return false
  const n = state.graph.nodes.get(focusId)
  return n?.kindPrefix === 'CM'
}

// ---------------------------------------------------------------------------
// refreshCmView — re-render everything (module-private)
// ---------------------------------------------------------------------------

function refreshCmView(): void {
  renderView()
}

// ---------------------------------------------------------------------------
// wireTableToggle — persist table hide preference (module-private)
// ---------------------------------------------------------------------------

function wireTableToggle(): void {
  const cb = document.getElementById('hide-relations') as HTMLInputElement | null
  const table = document.querySelector<HTMLElement>('.relationship-table')
  if (cb === null || table === null) return

  const hidden = safeStorage.get('doctrine-map-hide-relations', '0') === '1'
  cb.checked = hidden
  table.classList.toggle('hidden', hidden)

  cb.addEventListener('change', () => {
    table.classList.toggle('hidden', cb.checked)
    safeStorage.set('doctrine-map-hide-relations', cb.checked ? '1' : '0')
  })
}

// ---------------------------------------------------------------------------
// bootstrap — initialise the SPA (module-private)
// ---------------------------------------------------------------------------

function bootstrap(): void {
  cacheElements(document)
  wireTableToggle()

  const entityListEl = elements.entityList
  if (entityListEl === null) return

  const viewBtns = document.querySelectorAll<HTMLElement>('.view-btn')
  for (const btn of viewBtns) {
    btn.addEventListener('click', function (this: HTMLElement) {
      const dv = this.getAttribute('data-view')
      state.viewMode = dv === 'actionability' ? 'actionability' : 'semantic'
      state.priorityZoomId = null
      renderView()
    })
  }

  wireFilters({
    container: document,
    onChange: (filterSet) => {
      state.kindFilter = filterSet
      applyFilters()
    },
  })

  const searchInput = document.querySelector<HTMLInputElement>('.search-input')

  wireSearch({
    input: searchInput,
    list: entityListEl,
    graph: state.graph,
    getFocusId: () => state.focusId,
    getKindFilter: () => state.kindFilter,
    onFocus: goto,
  })

  wireDepthButtons({
    container: document,
    onDepthChange: (d) => {
      if (state.focusId !== null) setFocus(state.focusId, d)
    },
  })

  const refreshBtn = document.querySelector<HTMLButtonElement>('.refresh-btn')

  wireRefresh({
    button: refreshBtn,
    onRefresh: () => {
      state.markdownCache.clear()
      state.conceptMapCache.clear()
      state.cmFocusNode = null
      state.graphRenderSeq += 1
      state.actionabilityView = null

      refreshGraph()
        .then(() => fetchGraph())
        .then((raw) => {
          normalizeGraph(raw)
          if (state.focusId !== null) {
            state.focusId = resolveFocus(state.focusId, state.graph)
          }
          renderView()
        })
        .catch((err: unknown) => {
          const appContainer = document.getElementById('app')
          if (appContainer !== null) {
            showError(appContainer, 'Failed to refresh: ' + (err instanceof Error ? err.message : 'Unknown error'))
          }
        })
    },
  })

  window.addEventListener('hashchange', renderView)

  void Promise.all([
    fetchHealth().catch(() => ({ dot: { ok: false }, graph: { ok: false } })),
    fetchGraph().catch(() => null),
    fetchActionabilityGraph().catch(() => null),
  ]).then((results) => {
    const health = results[0]
    const raw = results[1]
    const aview = results[2]

    state.dotAvailable = health.dot.ok

    if (raw !== null) normalizeGraph(raw)

    if (aview !== null) setActionabilityView(aview)

    if (state.focusId === null && state.graph.nodes.size > 0) {
      state.focusId = resolveFocus(null, state.graph)
      if (state.focusId !== null) {
        setFocus(state.focusId, state.depth)
        return
      }
    }

    renderView()
  }).catch((err: unknown) => {
    const appContainer = document.getElementById('app')
    if (appContainer !== null) {
      showError(appContainer, 'Failed to initialise: ' + (err instanceof Error ? err.message : 'Unknown error'))
    }
  })
}

// ---------------------------------------------------------------------------
// renderView — main render orchestrator (module-private)
// ---------------------------------------------------------------------------

function renderView(): void {
  const entityListEl = elements.entityList
  const focusHeaderEl = elements.focusHeader
  const graphAreaEl = elements.graphArea
  const hoverDetailEl = elements.hoverDetail
  const relTableBody = elements.relationshipTableBody

  if (entityListEl === null || focusHeaderEl === null || hoverDetailEl === null || relTableBody === null) return

  const route = parseHash()
  const prevFocusId = state.focusId
  const prevDepth = state.depth

  if (route.view === 'focus') state.focusId = route.id
  state.depth = Math.max(0, Math.min(3, route.depth))

  if (route.view === 'edge' && state.focusId === null && state.graph.nodes.size > 0) {
    state.focusId = resolveFocus(null, state.graph)
  }

  // ---- Edge detail mode ----
  if (route.view === 'edge') {
    edgeDetail({
      container: graphAreaEl,
      edge: state.graph.edgeById.get(route.id ?? '') ?? null,
      graph: state.graph,
      depth: state.depth,
      focusId: state.focusId ?? '',
    })

    hoverPane({ container: hoverDetailEl, node: null })
    setPageMode('edge')

    const mp = document.querySelector<HTMLElement>('.markdown-pane')
    if (mp !== null) {
      mp.replaceChildren(el('span', { className: 'placeholder', textContent: '[Markdown content]' }))
    }

    const tbody = document.querySelector<HTMLElement>('.relationship-table tbody')
    if (tbody !== null) {
      tbody.replaceChildren(
        el('tr', {}, [
          el('td', { colspan: '5' }, [
            el('span', { className: 'placeholder', textContent: '[Relationship table]' }),
          ]),
        ]),
      )
    }

    renderFilteredEntities({
      list: entityListEl,
      graph: state.graph,
      query: '',
      kindFilter: state.kindFilter,
      focusId: state.focusId,
      onFocus: goto,
    })

    focusHeader({ container: focusHeaderEl, focusId: state.focusId, graph: state.graph })
    return
  }

  // ---- Focus / main mode ----
  const qEl = document.querySelector<HTMLInputElement>('.search-input')

  renderFilteredEntities({
    list: entityListEl,
    graph: state.graph,
    query: qEl?.value ?? '',
    kindFilter: state.kindFilter,
    focusId: state.focusId,
    onFocus: goto,
  })

  focusHeader({ container: focusHeaderEl, focusId: state.focusId, graph: state.graph })
  hoverPane({ container: hoverDetailEl, node: null })

  // Highlight active depth button
  const depthBtns = document.querySelectorAll<HTMLElement>('.depth-btn')
  for (const depthBtn of depthBtns) {
    const dataDepth = parseInt(depthBtn.getAttribute('data-depth') ?? '0', 10)
    depthBtn.classList.toggle('active', dataDepth === state.depth)
  }

  // View change detection
  const graphArea = document.querySelector<HTMLElement>('.graph-area')
  const focusChanged = state.focusId !== prevFocusId
  const depthChanged = state.depth !== prevDepth
  // eslint-disable-next-line @typescript-eslint/prefer-optional-chain
  const graphMissing = graphArea === null || graphArea.querySelector('svg') === null
  // A pure view-mode toggle changes none of focus/depth/graph-presence, so the
  // semantic branch must also fire when the view mode itself changed (ISS-020).
  const viewModeChanged = state.viewMode !== state.renderedViewMode

  const currentCmKey = state.cmFocusNode?.key ?? null
  const cmFocusChanged = state.focusId !== null && isConceptMap(state.focusId) && currentCmKey !== state.renderedCmFocus
  const cmCacheChanged = state.focusId !== null && isConceptMap(state.focusId) && state.cmCacheMutationSeq !== state.renderedCmCacheSeq

  // Clear cached concept map on focus change away from a CM
  if (focusChanged && prevFocusId !== null && isConceptMap(prevFocusId)) {
    state.conceptMapCache.delete(prevFocusId)
  }

  // ---- Actionability view ----
  if (state.viewMode === 'actionability') {
    if (state.actionabilityView === null) {
      if (graphArea !== null) {
        graphArea.replaceChildren(el('p', { className: 'loading', textContent: 'Loading actionability graph…' }))
      }
      fetchActionabilityGraph()
        .then((result) => {
          setActionabilityView(result)
          renderView()
        })
        .catch((err: unknown) => {
          if (graphArea !== null) {
            const msg = err instanceof Error ? err.message : 'Unknown error'
            graphArea.replaceChildren(el('p', { className: 'error', textContent: 'Failed to load actionability graph: ' + escapeHtml(msg) }))
          }
        })
    } else if (graphArea !== null) {
      const actionabilityNodes = Array.isArray(state.actionabilityView.nodes)
        ? state.actionabilityView.nodes
        : []

      if (actionabilityNodes.length > 0) {
        renderGraph({
          container: graphArea,
          view: state.actionabilityView,
          zoomId: state.priorityZoomId,
          initialTransform: state.priorityTransform,
          animateToZoom: state.priorityZoomPending,
          onNodeClick: (id) => {
            // Zoom to the clicked node (re-renders via hash → focus) and update
            // the detail pane. IMP-092.
            state.priorityZoomId = id
            state.priorityZoomPending = true
            goto(id)
          },
          onBackgroundClick: () => {
            if (state.priorityZoomId !== null || state.priorityTransform !== null) {
              state.priorityZoomId = null
              state.priorityTransform = null
              state.priorityZoomPending = false
              renderView()
            }
          },
          onTransform: (t) => {
            state.priorityTransform = t
          },
        })
        // Consume the one-shot — later re-renders restore the viewport without
        // re-animating to the (still-highlighted) node.
        state.priorityZoomPending = false
      } else if (state.focusId !== null) {
        const focusNode = state.graph.nodes.get(state.focusId)
        const isWorkKind = focusNode !== undefined &&
          (focusNode.kindPrefix === 'SL' ||
           focusNode.kindPrefix === 'ISS' ||
           focusNode.kindPrefix === 'IMP' ||
           focusNode.kindPrefix === 'CHR' ||
           focusNode.kindPrefix === 'RSK')

        if (isWorkKind) {
          graphArea.replaceChildren(
            el('p', { className: 'placeholder', textContent: state.focusId + ' is ' + (focusNode.status === '' ? 'terminal' : focusNode.status) + ' — terminal items don\'t appear in the actionability graph.' }),
          )
        } else {
          graphArea.replaceChildren(
            el('p', { className: 'placeholder', textContent: 'This entity has no dep/seq edges — switch to a work entity (SL/backlog) or use Semantic view.' }),
          )
        }
      } else {
        graphArea.replaceChildren(
          el('p', { className: 'placeholder', textContent: 'No eligible work items found.' }),
        )
      }
    }

    // ---- Entity-graph (semantic) view ----
  } else if (graphArea !== null && (focusChanged || depthChanged || graphMissing || cmFocusChanged || cmCacheChanged || viewModeChanged)) {
    if (focusChanged && !depthChanged && state.focusId !== null) {
      const svgEl = graphArea.querySelector('svg')
      if (svgEl !== null) {
        applyFocusHighlight(svgEl, state.focusId, prevFocusId, (g) => {
          const t = g.querySelector('text')
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
          if (t !== null) return (t.textContent ?? '').trim()
          const ti = g.querySelector('title')
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
          return ti !== null ? (ti.textContent ?? '').trim() : ''
        })
      }
    }

    if (state.focusId !== null) {
      if (isConceptMap(state.focusId)) {
        renderCmGraph(graphArea)
      } else {
        state.renderedCmFocus = null
        state.graphRenderSeq += 1
        graphPane({
          container: graphArea,
          graph: state.graph,
          focusId: state.focusId,
          depth: state.depth,
          dotAvailable: state.dotAvailable,
          seq: state.graphRenderSeq,
          getCurrentSeq: () => state.graphRenderSeq,
          onNodeClick: goto,
          onNodeHoverEnter: (id) => {
            state.hoveredId = id
            const node = state.graph.nodes.get(id) ?? null
            hoverPane({ container: hoverDetailEl, node })
          },
          onNodeHoverLeave: () => {
            state.hoveredId = null
            hoverPane({ container: hoverDetailEl, node: null })
          },
        })
      }
    }
  }

  // Toggle legend visibility based on view mode
  const legendItems = document.querySelector<HTMLElement>('.edge-legend .legend-items')
  const priorityLegend = document.querySelector<HTMLElement>('.priority-legend')

  if (state.viewMode === 'actionability') {
    relationshipTable({
      container: relTableBody,
      graph: state.graph,
      focusId: state.focusId,
      depth: state.depth,
      viewMode: 'actionability',
      actionabilityView: state.actionabilityView,
    })
    if (priorityLegend !== null) priorityLegend.style.display = ''
    if (legendItems !== null) legendItems.style.display = 'none'
  } else {
    relationshipTable({
      container: relTableBody,
      edges: buildTableEdges(),
      graph: state.graph,
      focusId: state.focusId,
      depth: state.depth,
      viewMode: 'semantic',
    })
    if (legendItems !== null) legendItems.style.display = ''
    if (priorityLegend !== null) priorityLegend.style.display = 'none'
  }

  // Highlight active view toggle button
  const viewBtns = document.querySelectorAll<HTMLElement>('.view-btn')
  for (const viewBtn of viewBtns) {
    viewBtn.classList.toggle('active', viewBtn.getAttribute('data-view') === state.viewMode)
  }

  const isCm = state.viewMode === 'semantic' && state.focusId !== null && isConceptMap(state.focusId)

  setPageMode(
    state.viewMode === 'actionability'
      ? 'actionability'
      : isCm
        ? 'concept-map'
        : 'entity-graph',
  )

  if (isCm && route.cmFocus !== null) {
    const cachedCm = state.conceptMapCache.get(state.focusId ?? '')
    let label = route.cmFocus
    if (cachedCm !== undefined) {
      for (const node of cachedCm.nodes) {
        if (node.key === route.cmFocus) {
          label = node.label
          break
        }
      }
    }
    if (state.cmFocusNode?.key !== route.cmFocus) {
      state.cmFocusNode = { key: route.cmFocus, label }
    }
  } else if (focusChanged) {
    state.cmFocusNode = null
  }

  renderEditToggle({
    header: focusHeaderEl,
    editing: state.editingConceptMap,
    onToggle: () => {
      state.editingConceptMap = !state.editingConceptMap
      if (!state.editingConceptMap) state.editingNode = null
      renderView()
    },
  })

  if (isCm) {
    renderCmEdgeTable()
    renderCmAddEdgeForm()
    renderCmDiagnostics()
  }

  if (state.focusId !== null) {
    const mdPane = document.querySelector<HTMLElement>('.markdown-pane')
    if (mdPane !== null) {
      markdownPane({
        container: mdPane,
        id: state.focusId,
        cache: state.markdownCache,
        currentFocusId: state.focusId,
      })
    }
  }

  // Record what we just rendered so a pure view-mode toggle re-renders (ISS-020).
  state.renderedViewMode = state.viewMode
}

// ---------------------------------------------------------------------------
// renderCmGraph — fetch & render concept map diagram (module-private)
// ---------------------------------------------------------------------------

function renderCmGraph(container: HTMLElement): void {
  const id = state.focusId
  if (id === null) return

  if (!state.conceptMapCache.has(id)) {
    container.replaceChildren(el('p', { className: 'loading', textContent: 'Loading concept map…' }))
    fetchConceptMap(id)
      .then((cm) => {
        state.conceptMapCache.set(id, cm)
        if (state.cmFocusNode !== null && state.focusId === id) {
          for (const node of cm.nodes) {
            if (node.key === state.cmFocusNode.key) {
              state.cmFocusNode = { key: node.key, label: node.label }
              break
            }
          }
        }
        renderView()
      })
      .catch((err: unknown) => {
        if (state.focusId !== id) return
        const msg = err instanceof Error ? err.message : 'Unknown error'
        container.replaceChildren(el('p', { className: 'error', textContent: 'Failed to load concept map: ' + escapeHtml(msg) }))
      })
    return
  }

  const cmCache = state.conceptMapCache.get(id)
  if (cmCache === undefined) return

  const focusKey = state.cmFocusNode?.key ?? null
  const filtered = focusKey !== null
    ? cmNeighbourhood(cmCache, focusKey, state.depth)
    : cmNeighbourhood(cmCache, null, state.depth)

  state.renderedCmFocus = focusKey
  state.renderedCmCacheSeq = state.cmCacheMutationSeq
  state.graphRenderSeq += 1
  const seq = state.graphRenderSeq

  renderDiagram({
    container,
    cm: filtered as unknown as ConceptMap,
    focusKey,
    dotAvailable: state.dotAvailable,
    seq,
    getCurrentSeq: () => state.graphRenderSeq,
    onClick: (key) => {
      if (state.editingConceptMap) {
        startRenameNode(key)
        return
      }
      const cmData = state.conceptMapCache.get(state.focusId ?? '')
      let label = key
      if (cmData !== undefined) {
        for (const cmNode of cmData.nodes) {
          if (cmNode.key === key) {
            label = cmNode.label
            break
          }
        }
      }
      if (state.cmFocusNode !== null && state.cmFocusNode.key === key) {
        state.cmFocusNode = null
      } else {
        state.cmFocusNode = { key, label }
      }
      window.location.hash = buildHash('focus', state.focusId ?? '', state.depth)
      renderView()
    },
  })
}

// ---------------------------------------------------------------------------
// renderCmEdgeTable — render concept map edge table (module-private)
// ---------------------------------------------------------------------------

function renderCmEdgeTable(): void {
  const cmCache = state.conceptMapCache.get(state.focusId ?? '') ?? null
  renderEdgeTable({
    container: document.querySelector<HTMLElement>('.cm-edge-table'),
    cm: cmCache,
    focusKey: state.cmFocusNode?.key ?? null,
    depth: state.depth,
    editing: state.editingConceptMap,
    editingNode: state.editingNode,
    onRemoveEdge: handleRemoveEdge,
    onRenameNode: startRenameNode,
    onSubmitRename: handleRenameNodeSubmit,
    onCancelRename: () => {
      state.editingNode = null
      renderView()
    },
  })
}

// ---------------------------------------------------------------------------
// renderCmAddEdgeForm — render add-edge form (module-private)
// ---------------------------------------------------------------------------

function renderCmAddEdgeForm(): void {
  const cmCache = state.conceptMapCache.get(state.focusId ?? '') ?? null
  renderAddEdgeForm({
    container: document.querySelector<HTMLElement>('.cm-add-edge-form'),
    cm: cmCache,
    editing: state.editingConceptMap,
    onSubmit: handleAddEdge,
  })
}

// ---------------------------------------------------------------------------
// renderCmDiagnostics — render concept map diagnostics (exported)
// ---------------------------------------------------------------------------

export function renderCmDiagnostics(): void {
  const p = document.querySelector<HTMLElement>('.cm-diagnostics-panel')
  if (p === null) return
  if (state.editingConceptMap) {
    p.style.display = 'none'
    return
  }
  const c = state.conceptMapCache.get(state.focusId ?? '')
  renderDiagnostics({
    container: p,
    diagnostics: c?.diagnostics ?? [],
  })
}

// ---------------------------------------------------------------------------
// updateConceptMapCache — patch cm cache in place (module-private)
// ---------------------------------------------------------------------------

function updateConceptMapCache(data: Record<string, unknown>): void {
  const cm = state.conceptMapCache.get(state.focusId ?? '')
  if (cm === undefined) return

  if (Array.isArray(data.nodes)) cm.nodes = data.nodes as ConceptMap['nodes']
  if (Array.isArray(data.edges)) cm.edges = data.edges as ConceptMap['edges']
  if (Array.isArray(data.diagnostics)) cm.diagnostics = data.diagnostics as ConceptMap['diagnostics']
  if (typeof data.dsl_hash === 'string') cm.dslHash = data.dsl_hash

  state.cmCacheMutationSeq += 1
}

// ---------------------------------------------------------------------------
// showCmFormError — display form-level error (module-private)
// ---------------------------------------------------------------------------

function showCmFormError(msg: string): void {
  const errEl = document.querySelector<HTMLElement>('.cm-add-error')
  if (errEl !== null) {
    errEl.textContent = msg
    errEl.style.display = 'block'
    errEl.className = 'cm-add-error cm-error'
  }
}

// ---------------------------------------------------------------------------
// handleStaleWrite — refresh after concurrent modification (module-private)
// ---------------------------------------------------------------------------

function handleStaleWrite(): void {
  const errEl = document.querySelector<HTMLElement>('.cm-add-error')
  if (errEl === null) return

  errEl.textContent = 'Concept map was modified elsewhere — data refreshed'
  errEl.style.display = 'block'
  errEl.className = 'cm-add-error cm-notice'

  window.setTimeout(() => {
    errEl.style.display = 'none'
  }, 4000)

  const focusId = state.focusId
  if (focusId !== null) {
    fetchConceptMap(focusId)
      .then((cm) => {
        state.conceptMapCache.set(focusId, cm)
        refreshCmView()
      })
      .catch(() => {
        /* ignore fetch failure in stale write recovery */
      })
  }
}

// ---------------------------------------------------------------------------
// handleMutationError — route concept map mutation errors (module-private)
// ---------------------------------------------------------------------------

function handleMutationError(err: unknown): void {
  if (err instanceof ApiError) {
    if (err.status === 409) {
      let body: Record<string, unknown> = {}
      try {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
        body = typeof err.body === 'string' ? JSON.parse(err.body) as Record<string, unknown> : err.body as unknown as Record<string, unknown>
      } catch {
        /* use empty fallback */
      }
      if (body.error === 'stale_concept_map') {
        handleStaleWrite()
        return
      }
      if (body.error === 'duplicate_edge') {
        // eslint-disable-next-line @typescript-eslint/no-base-to-string
        showCmFormError('This edge already exists at line ' + (body.line !== undefined ? String(body.line) : '?'))
        return
      }
      if (body.error === 'node_collision') {
        showCmFormError('Rename would collide with existing node \'' + (typeof body.existing_label === 'string' ? body.existing_label : '') + '\'')
        return
      }
    }

    if (err.status === 400) {
      let b400: Record<string, unknown> = {}
      try {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
        b400 = typeof err.body === 'string' ? JSON.parse(err.body) as Record<string, unknown> : err.body as unknown as Record<string, unknown>
      } catch {
        /* use empty fallback */
      }
      if (b400.error === 'empty_field') {
        showCmFormError(typeof b400.message === 'string' ? b400.message : 'Field must not be empty')
        return
      }
    }

    if (err.status === 404) {
      showCmFormError('Edge no longer exists — it may have been removed elsewhere')
      return
    }
  }

  const msg = err instanceof Error ? err.message : 'Unknown error'
  showCmFormError('Error: ' + escapeHtml(msg === '' ? 'Unknown error' : msg))
}

// ---------------------------------------------------------------------------
// handleAddEdge — submit add-edge mutation (module-private)
// ---------------------------------------------------------------------------

function handleAddEdge(source: string, rel: string, target: string): void {
  const errEl = document.querySelector<HTMLElement>('.cm-add-error')
  if (errEl !== null) {
    errEl.style.display = 'none'
    errEl.textContent = ''
  }

  const src = source.trim()
  const rl = rel.trim()
  const tgt = target.trim()

  if (src === '') { showCmFormError('Source must not be empty'); return }
  if (rl === '') { showCmFormError('Relation must not be empty'); return }
  if (tgt === '') { showCmFormError('Target must not be empty'); return }

  const cm = state.conceptMapCache.get(state.focusId ?? '')
  const baseHash = cm !== undefined ? cm.dslHash : undefined

  mutateConceptMap(state.focusId ?? '', 'add_edge', { source: src, rel: rl, target: tgt }, baseHash)
    .then((data) => {
      const f = document.querySelector<HTMLFormElement>('.add-edge-form')
      if (f !== null) {
        const srcInput = f.querySelector<HTMLInputElement>('.cm-source')
        const relInput = f.querySelector<HTMLInputElement>('.cm-rel')
        const tgtInput = f.querySelector<HTMLInputElement>('.cm-target')
        if (srcInput !== null) srcInput.value = ''
        if (relInput !== null) relInput.value = ''
        if (tgtInput !== null) tgtInput.value = ''
      }
      updateConceptMapCache(data as Record<string, unknown>)
      refreshCmView()
    })
    .catch((err: unknown) => {
      handleMutationError(err)
    })
}

// ---------------------------------------------------------------------------
// handleRemoveEdge — submit remove-edge mutation (module-private)
// ---------------------------------------------------------------------------

function handleRemoveEdge(source: string, rel: string, target: string): void {
  const cm = state.conceptMapCache.get(state.focusId ?? '')
  const baseHash = cm !== undefined ? cm.dslHash : undefined

  mutateConceptMap(state.focusId ?? '', 'remove_edge', { source, rel, target }, baseHash)
    .then((data) => {
      updateConceptMapCache(data as Record<string, unknown>)
      refreshCmView()
    })
    .catch((err: unknown) => {
      handleMutationError(err)
    })
}

// ---------------------------------------------------------------------------
// startRenameNode — enter rename mode (module-private)
// ---------------------------------------------------------------------------

function startRenameNode(key: string): void {
  if (!state.editingConceptMap) return

  const cm = state.conceptMapCache.get(state.focusId ?? '')
  if (cm === undefined) return

  let label = key
  for (const node of cm.nodes) {
    if (node.key === key) {
      label = node.label
      break
    }
  }

  state.editingNode = { key, label }
  renderCmEdgeTable()
}

// ---------------------------------------------------------------------------
// handleRenameNodeSubmit — submit rename mutation (module-private)
// ---------------------------------------------------------------------------

function handleRenameNodeSubmit(newLabel: string): void {
  const oldLabel = state.editingNode?.label ?? ''
  state.editingNode = null

  const nt = newLabel.trim()
  if (nt === '') {
    showCmFormError('New label must not be empty')
    refreshCmView()
    return
  }

  const cm = state.conceptMapCache.get(state.focusId ?? '')
  const baseHash = cm !== undefined ? cm.dslHash : undefined

  mutateConceptMap(state.focusId ?? '', 'rename_node', { old_label: oldLabel, new_label: nt }, baseHash)
    .then((data) => {
      updateConceptMapCache(data as Record<string, unknown>)
      refreshCmView()
    })
    .catch((err: unknown) => {
      if (err instanceof ApiError && err.status === 409) {
        let body: Record<string, unknown> = {}
        try {
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
          body = typeof err.body === 'string' ? JSON.parse(err.body) as Record<string, unknown> : err.body as unknown as Record<string, unknown>
        } catch {
          /* fallback */
        }
        showCmFormError('Rename would collide with existing node \'' + (typeof body.existing_label === 'string' ? body.existing_label : '') + '\'')
      } else {
        handleMutationError(err)
      }
      refreshCmView()
    })
}

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', bootstrap)
} else {
  bootstrap()
}
