import type {
  RawGraph,
  CatalogNode,
  Edge,
  ActionabilityView,
  ConceptMap,
  CmNode,
  CmEdge,
  Neighbourhood,
  CmNeighbourhood,
} from './types';
import { state } from './state';

export { state } from './state';

/* ------------------------------------------------------------------ */
/*  Private helpers                                                    */
/* ------------------------------------------------------------------ */

function padId(n: number): string {
  return (n < 100 ? (n < 10 ? '00' : '0') : '') + String(n);
}

function splitPrefix(s: string): { prefix: string; num: number } | null {
  const lastHyphen = s.lastIndexOf('-');
  if (lastHyphen <= 0) return null;
  const prefix = s.substring(0, lastHyphen);
  const numStr = s.substring(lastHyphen + 1);
  if (!/^[A-Za-z]+$/.test(prefix) || !/^\d+$/.test(numStr)) return null;
  return { prefix: prefix.toUpperCase(), num: parseInt(numStr, 10) };
}

function sortedNodeIds(graph: { nodes: Map<string, CatalogNode> }): string[] {
  const keys = Array.from(graph.nodes.keys());
  keys.sort();
  return keys;
}

function looseCanonical(query: string): string | null {
  // Find first digit position.
  let firstDigit = -1;
  for (let i = 0; i < query.length; i++) {
    if (/[0-9]/.test(query.charAt(i))) {
      firstDigit = i;
      break;
    }
  }
  if (firstDigit <= 0) return null;

  let prefix = '';
  for (let j = 0; j < firstDigit; j++) {
    const ch = query.charAt(j);
    if (/[A-Za-z]/.test(ch)) {
      prefix += ch.toUpperCase();
    }
  }
  if (prefix === '') return null;

  // Extract numeric digits from remainder.
  let numStr = '';
  for (let k = firstDigit; k < query.length; k++) {
    const d = query.charAt(k);
    if (/[0-9]/.test(d)) {
      numStr += d;
    }
  }
  if (numStr === '') return null;

  const num = parseInt(numStr, 10);
  if (Number.isNaN(num)) return null;

  return prefix + '-' + padId(num);
}

/**
 * Shared BFS core used by both `neighbourhood` (entity graph) and
 * `cmNeighbourhood` (concept map).
 *
 * `expandNeighbours(id)` returns `[{ nodeId, edge }]`.
 * `edgeKey(edge)` returns a unique dedup key for each edge.
 */
function bfsCore<E>(
  startId: string,
  maxDepth: number,
  expandNeighbours: (id: string) => { nodeId: string; edge: E }[],
  edgeKey: (edge: E) => string,
): { nodes: Set<string>; edges: E[] } {
  maxDepth = Math.max(0, Math.min(3, maxDepth));
  if (maxDepth === 0) {
    return { nodes: new Set([startId]), edges: [] };
  }

  const visited = new Set<string>();
  const collectedEdges: E[] = [];
  const collectedEdgeKeys = new Set<string>();
  const queue: { id: string; dist: number }[] = [{ id: startId, dist: 0 }];
  visited.add(startId);

  while (queue.length > 0) {
    const current = queue.shift();
    if (current === undefined) continue;
    if (current.dist >= maxDepth) continue;

    const neighbours = expandNeighbours(current.id);
    for (const nb of neighbours) {
      if (!visited.has(nb.nodeId)) {
        visited.add(nb.nodeId);
        queue.push({ id: nb.nodeId, dist: current.dist + 1 });
      }
      const key = edgeKey(nb.edge);
      if (!collectedEdgeKeys.has(key)) {
        collectedEdgeKeys.add(key);
        collectedEdges.push(nb.edge);
      }
    }
  }

  return { nodes: visited, edges: collectedEdges };
}

/* ------------------------------------------------------------------ */
/*  Public: string encoders                                           */
/* ------------------------------------------------------------------ */

export function encodePart(s: string): string {
  let result = '';
  for (let i = 0; i < s.length; i++) {
    const c = s.charAt(i);
    if (/[A-Za-z0-9_-]/.test(c)) {
      result += c;
    } else {
      let hex = c.charCodeAt(0).toString(16);
      if (hex.length === 1) hex = '0' + hex;
      result += '_' + hex;
    }
  }
  return result;
}

export function pascalToSnake(s: string): string {
  return s.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();
}

/* ------------------------------------------------------------------ */
/*  Public: graph normalization                                       */
/* ------------------------------------------------------------------ */

export function normalizeGraph(raw: RawGraph): void {
  const nodes = new Map<string, CatalogNode>();
  const edges: Edge[] = [];
  const edgeById = new Map<string, Edge>();
  const incoming = new Map<string, Edge[]>();
  const outgoing = new Map<string, Edge[]>();

  // Build nodes.
  for (const key of Object.keys(raw.nodes)) {
    const entry = raw.nodes[key];
    if (entry === undefined) continue;
    const sp = splitPrefix(key);
    let kindPrefix = sp !== null ? sp.prefix : '';
    // Memory entities (mem_*) — fall back to kind_label.
    if (kindPrefix === '' && entry.kind_label !== '') kindPrefix = entry.kind_label;
    nodes.set(key, {
      id: key,
      title: entry.title,
      status: entry.status,
      kindPrefix,
      kindLabel: entry.kind_label,
      raw: entry,
    });
  }

  // Build edges.
  for (const edge of raw.edges) {
    // Skip unresolved targets.
    if (edge.target?.Resolved == null) continue;

    const source = edge.source;
    const target = edge.target.Resolved;
    let label = '';
    if (edge.label.Validated !== undefined) {
      label = pascalToSnake(edge.label.Validated);
      // A `references` edge carries an intent role (SL-149); fold it into the label
      // as `references(<role>)` for CLI parity. Edge colour and the static legend
      // key on the bare relation type — `references` is absent from both, so the
      // composed label degrades identically to the bare label (grey, undimmed).
      if (edge.role !== undefined && edge.role !== '') {
        label = `${label}(${pascalToSnake(edge.role)})`;
      }
    } else if (edge.label.Raw !== undefined) {
      label = edge.label.Raw;
    }

    const edgeId = 'e_' + encodePart(source) + '_' + encodePart(label) + '_' + encodePart(target);

    // Coalesce duplicates.
    if (edgeById.has(edgeId)) continue;

    const edgeObj: Edge = {
      id: edgeId,
      source,
      label,
      target,
      raw: edge,
    };

    edgeById.set(edgeId, edgeObj);
    edges.push(edgeObj);

    // Incoming.
    let inc = incoming.get(target);
    if (inc === undefined) {
      inc = [];
      incoming.set(target, inc);
    }
    inc.push(edgeObj);

    // Outgoing.
    let out = outgoing.get(source);
    if (out === undefined) {
      out = [];
      outgoing.set(source, out);
    }
    out.push(edgeObj);
  }

  // Mutate state.graph in place.
  state.graph.nodes = nodes;
  state.graph.edges = edges;
  state.graph.edgeById = edgeById;
  state.graph.incoming = incoming;
  state.graph.outgoing = outgoing;
}

/* ------------------------------------------------------------------ */
/*  Public: lookup / resolution                                       */
/* ------------------------------------------------------------------ */

export function findFocus(
  query: string | null,
  graph: { nodes: Map<string, CatalogNode> },
): string | null {
  // Step 1: null/empty → first sorted node.
  if (query === null || query === '') {
    const sortedIds = sortedNodeIds(graph);
    return sortedIds.length > 0 ? sortedIds[0] ?? null : null;
  }

  // Step 2: exact canonical match case-insensitive.
  const upperQuery = query.toUpperCase();
  if (graph.nodes.has(upperQuery)) {
    return upperQuery;
  }

  // Step 3: loose canonical.
  const norm = looseCanonical(query);
  if (norm !== null && graph.nodes.has(norm)) {
    return norm;
  }

  // Step 4: exact title match case-insensitive.
  const queryLower = query.toLowerCase();
  let titleMatch: string | null = null;
  for (const node of graph.nodes.values()) {
    if (node.title.toLowerCase() === queryLower) {
      titleMatch = node.id;
    }
  }
  if (titleMatch !== null) return titleMatch;

  // Step 5: substring in id, title, status, or kindLabel.
  let best: string | null = null;
  for (const node of graph.nodes.values()) {
    const targets = [
      node.id.toLowerCase(),
      node.title.toLowerCase(),
      node.status.toLowerCase(),
      node.kindLabel.toLowerCase(),
    ];
    for (const target of targets) {
      if (target.includes(queryLower)) {
        if (best === null || node.id.length < best.length) {
          best = node.id;
        }
        break;
      }
    }
  }
  if (best !== null) return best;

  // Step 6: no fallback — return null.
  return null;
}

export function resolveFocus(
  query: string | null,
  graph: { nodes: Map<string, CatalogNode> },
): string | null {
  const result = findFocus(query, graph);
  if (result !== null) return result;

  // Fallback to first sorted node.
  const sortedIds = sortedNodeIds(graph);
  return sortedIds.length > 0 ? sortedIds[0] ?? null : null;
}

export function setActionabilityView(view: ActionabilityView | null): void {
  state.actionabilityView = view;
}

/* ------------------------------------------------------------------ */
/*  Public: focus-change → view transition (SL-110 Item 5)            */
/* ------------------------------------------------------------------ */

type ViewMode = 'semantic' | 'actionability';

/**
 * Pure transition computed on every focus change. DOM/clock-free.
 *
 * Revision 2 (D2 reversed, RV-098 F-5):
 *
 * | case                       | viewMode        | priorityZoomId          |
 * |----------------------------|-----------------|-------------------------|
 * | actionability + member     | 'actionability' | focusId (zoom to it)    |
 * | actionability + non-member | 'semantic'      | null (switch + clear)   |
 * | current is semantic        | 'semantic'      | currentPriorityZoomId   |
 *
 * On the actionability graph, focusing a member zooms to it; focusing anything
 * else switches to Semantic (where it is visible) and clears the now-stale zoom.
 * In Semantic, focus never auto-switches — the user chose it (deliberate
 * asymmetry; Semantic shows everything, so it is never a dead-end). A concept
 * map is never an actionability member, so the old CM-forces-semantic case is
 * subsumed by the non-member row — `requiredMode` and the `node` arg are gone;
 * the only use of `node` was its id, which equals `focusId`. The semantic row
 * echoes `currentPriorityZoomId` (it cannot otherwise express "unchanged").
 */
export function focusTransition(
  current: ViewMode,
  focusId: string | null,
  isActionabilityMember: boolean,
  currentPriorityZoomId: string | null,
): { viewMode: ViewMode; priorityZoomId: string | null } {
  if (current === 'actionability') {
    return isActionabilityMember
      ? { viewMode: 'actionability', priorityZoomId: focusId }
      : { viewMode: 'semantic', priorityZoomId: null };
  }
  return { viewMode: 'semantic', priorityZoomId: currentPriorityZoomId };
}

/* ------------------------------------------------------------------ */
/*  Public: neighbourhood (BFS)                                       */
/* ------------------------------------------------------------------ */

export function neighbourhood(
  focusId: string,
  depth: number,
  graph: { outgoing: Map<string, Edge[]>; incoming: Map<string, Edge[]> },
): Neighbourhood {
  function expandNeighbours(id: string): { nodeId: string; edge: Edge }[] {
    const result: { nodeId: string; edge: Edge }[] = [];
    const outEdges = graph.outgoing.get(id) ?? [];
    const inEdges = graph.incoming.get(id) ?? [];
    for (const e of outEdges) {
      result.push({ nodeId: e.target, edge: e });
    }
    for (const e of inEdges) {
      result.push({ nodeId: e.source, edge: e });
    }
    return result;
  }

  return bfsCore(focusId, depth, expandNeighbours, (e) => e.id);
}

/* ------------------------------------------------------------------ */
/*  Public: kind ordering & comparison                                */
/* ------------------------------------------------------------------ */

export const kindOrder: Record<string, number> = {
  PRD: 1, SPEC: 1,
  ADR: 2, POL: 2,
  STD: 3,
  SL: 4,
  ISS: 5, IMP: 5, CHR: 5, RSK: 5,
  REV: 6,
  RV: 7,
  REQ: 8,
  IDE: 9,
  REC: 10, MEM: 10,
  ASM: 11, DEC: 11,
  QUE: 12, CON: 12,
  CM: 20,
};

export function compareNodes(a: CatalogNode, b: CatalogNode): number {
  const ordA = kindOrder[a.kindPrefix] ?? 99;
  const ordB = kindOrder[b.kindPrefix] ?? 99;
  if (ordA !== ordB) return ordA - ordB;

  const suffixA = a.id.split('-').pop();
  const rawNumA = parseInt(suffixA ?? '', 10);
  const numA = Number.isNaN(rawNumA) ? 0 : rawNumA;

  const suffixB = b.id.split('-').pop();
  const rawNumB = parseInt(suffixB ?? '', 10);
  const numB = Number.isNaN(rawNumB) ? 0 : rawNumB;

  if (numA !== numB) return numA - numB;

  if (a.id < b.id) return -1;
  if (a.id > b.id) return 1;
  return 0;
}

export function compareEdgesBySource(ea: Edge, eb: Edge): number {
  const sa = state.graph.nodes.get(ea.source);
  const sb = state.graph.nodes.get(eb.source);
  if (sa === undefined || sb === undefined) {
    return ea.id < eb.id ? -1 : 1;
  }
  return compareNodes(sa, sb);
}

/* ------------------------------------------------------------------ */
/*  Public: kind aggregation                                          */
/* ------------------------------------------------------------------ */

export function kinds(nodes: Map<string, CatalogNode>): Map<string, number> {
  const counts = new Map<string, number>();
  nodes.forEach((node) => {
    const kp = node.kindPrefix;
    counts.set(kp, (counts.get(kp) ?? 0) + 1);
  });

  // Sort by prefix alphabetically.
  const sorted = new Map<string, number>();
  const keys = Array.from(counts.keys());
  keys.sort();
  for (const k of keys) {
    const v = counts.get(k);
    if (v !== undefined) {
      sorted.set(k, v);
    }
  }
  return sorted;
}

/* ------------------------------------------------------------------ */
/*  Public: search / filter                                           */
/* ------------------------------------------------------------------ */

export function searchFilter(
  query: string | null,
  graph: { nodes: Map<string, CatalogNode> },
): CatalogNode[] {
  const results: CatalogNode[] = [];
  if (query === null || query === '') {
    for (const node of graph.nodes.values()) {
      results.push(node);
    }
    results.sort(compareNodes);
    return results;
  }

  const q = query.toLowerCase();
  for (const node of graph.nodes.values()) {
    if (node.id.toLowerCase().includes(q) ||
        node.title.toLowerCase().includes(q)) {
      results.push(node);
    }
  }
  results.sort(compareNodes);
  return results;
}

/* ------------------------------------------------------------------ */
/*  Public: concept map                                               */
/* ------------------------------------------------------------------ */

export function normalizeConceptMap(raw: unknown): ConceptMap {
  const r = raw as Record<string, unknown>;
  const id = r.id;
  const title = r.title;
  const status = r.status;
  const description = r.description;
  const dslHash = r.dsl_hash;
  const nodes = r.nodes;
  const edges = r.edges;
  const diagnostics = r.diagnostics;

  return {
    id: typeof id === 'string' ? id : '',
    title: typeof title === 'string' ? title : '',
    status: typeof status === 'string' ? status : 'active',
    description: typeof description === 'string' ? description : '',
    dslHash: typeof dslHash === 'string' ? dslHash : '',
    nodes: Array.isArray(nodes) ? (nodes as unknown[] as CmNode[]) : [],
    edges: Array.isArray(edges) ? (edges as unknown[] as CmEdge[]) : [],
    diagnostics: Array.isArray(diagnostics) ? (diagnostics as unknown[]) : [],
  };
}

export function buildNodeLabelList(cm: ConceptMap | null | undefined): string[] {
  if (cm == null) return [];
  if (cm.nodes.length === 0) return [];
  const labels: string[] = [];
  const seen = new Set<string>();
  for (const node of cm.nodes) {
    const label = node.label;
    if (!seen.has(label)) {
      seen.add(label);
      labels.push(label);
    }
  }
  return labels;
}

export function buildRelLabelList(cm: ConceptMap | null | undefined): string[] {
  if (cm == null) return [];
  if (cm.edges.length === 0) return [];
  const rels: string[] = [];
  const seen = new Set<string>();
  for (const edge of cm.edges) {
    const rel = edge.rel;
    if (!seen.has(rel)) {
      seen.add(rel);
      rels.push(rel);
    }
  }
  return rels;
}

export function cmNeighbourhood(
  cm: ConceptMap | null | undefined,
  focusKey: string | null | undefined,
  depth: number,
): CmNeighbourhood {
  if (cm == null) return { nodes: [], edges: [] };
  if (focusKey == null) {
    return { nodes: cm.nodes, edges: cm.edges };
  }
  depth = Math.max(0, Math.min(3, depth));

  const edges = cm.edges;

  // Build undirected adjacency map.
  // Explicit | undefined so that index access returns T | undefined,
  // satisfying strict-boolean-expressions even with noUncheckedIndexedAccess.
  const adj: Record<string, { nodeId: string; edge: CmEdge }[] | undefined> = {};
  for (const e of edges) {
    let fromList = adj[e.from_key];
    if (fromList === undefined) {
      fromList = [];
      adj[e.from_key] = fromList;
    }
    fromList.push({ nodeId: e.to_key, edge: e });

    let toList = adj[e.to_key];
    if (toList === undefined) {
      toList = [];
      adj[e.to_key] = toList;
    }
    toList.push({ nodeId: e.from_key, edge: e });
  }

  // Ensure focusKey exists in the node set.
  const nodeKeySet = new Set<string>();
  for (const n of cm.nodes) {
    nodeKeySet.add(n.key);
  }
  if (!nodeKeySet.has(focusKey)) {
    // Graceful fallback: focusKey not in nodes → return all.
    return { nodes: cm.nodes, edges };
  }

  function expandNeighbours(key: string): { nodeId: string; edge: CmEdge }[] {
    return adj[key] ?? [];
  }

  const result = bfsCore(focusKey, depth, expandNeighbours, (e) => {
    return e.from_key + '\x00' + e.rel + '\x00' + e.to_key;
  });

  // Filter nodes to visited set (preserving original order).
  const filteredNodes: CmNode[] = [];
  for (const n of cm.nodes) {
    if (result.nodes.has(n.key)) {
      filteredNodes.push(n);
    }
  }

  // Filter edges: both ends in visited (preserving original order).
  const filteredEdges: CmEdge[] = [];
  for (const edge of edges) {
    if (result.nodes.has(edge.from_key) && result.nodes.has(edge.to_key)) {
      filteredEdges.push(edge);
    }
  }

  return { nodes: filteredNodes, edges: filteredEdges };
}
