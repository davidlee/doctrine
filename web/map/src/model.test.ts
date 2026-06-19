/**
 * @vitest-environment jsdom
 *
 * Behaviour-contract tests for model.ts — captures the EXACT observable
 * behaviour of model.js as a contract that the TypeScript rewrite must satisfy.
 *
 * These tests initially FAIL (RED) because model.ts doesn't exist yet.
 * The satisfier creates model.ts to make them pass.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import type {
  RawGraph,
  RawCatalogNode,
  CatalogNode,
  Edge,
  ActionabilityView,
  ConceptMap,
  CmNode,
  CmEdge,
} from './types';

// Mock the state module with the full shape model.js accesses.
// The factory is hoisted by vitest — the state object is created
// inside the factory so it is available when the mock is first used.
vi.mock('./state', () => {
  const state = {
    graph: {
      nodes: new Map<string, CatalogNode>(),
      edges: [] as Edge[],
      edgeById: new Map<string, Edge>(),
      incoming: new Map<string, Edge[]>(),
      outgoing: new Map<string, Edge[]>(),
    },
    actionabilityView: null as ActionabilityView | null,
  };
  return { state };
});

import { state } from './state';
import {
  encodePart,
  pascalToSnake,
  normalizeGraph,
  findFocus,
  resolveFocus,
  neighbourhood,
  kindOrder,
  kinds,
  searchFilter,
  normalizeConceptMap,
  buildNodeLabelList,
  buildRelLabelList,
  setActionabilityView,
  cmNeighbourhood,
  compareEdgesBySource,
  focusTransition,
} from './model';

/* ------------------------------------------------------------------ */
/*  Helpers                                                           */
/* ------------------------------------------------------------------ */

function makeRawNode(overrides: Partial<RawCatalogNode> = {}): RawCatalogNode {
  return {
    title: 'Untitled',
    status: 'active',
    kind_label: '',
    ...overrides,
  };
}

function makeCatalogNode(overrides: Partial<CatalogNode> = {}): CatalogNode {
  return {
    id: 'SL-001',
    title: 'Test Slice',
    status: 'active',
    kindPrefix: 'SL',
    kindLabel: 'Slice',
    raw: { title: 'Test Slice', status: 'active', kind_label: 'Slice' },
    ...overrides,
  };
}

function makeEdge(overrides: Partial<Edge> = {}): Edge {
  return {
    id: 'e_test',
    source: 'SL-001',
    label: 'specs',
    target: 'SL-002',
    raw: {
      source: 'SL-001',
      label: { Validated: 'specs' },
      target: { Resolved: 'SL-002' },
    },
    ...overrides,
  };
}

/** Build a minimal Graph for neighbourhood/findFocus tests. */
function makeGraph(
  nodes: CatalogNode[] = [],
  edges: Edge[] = [],
): { nodes: Map<string, CatalogNode>; edges: Edge[]; incoming: Map<string, Edge[]>; outgoing: Map<string, Edge[]> } {
  const nodeMap = new Map<string, CatalogNode>();
  for (const n of nodes) nodeMap.set(n.id, n);

  const incoming = new Map<string, Edge[]>();
  const outgoing = new Map<string, Edge[]>();

  for (const e of edges) {
    const inc = incoming.get(e.target) ?? [];
    inc.push(e);
    incoming.set(e.target, inc);

    const out = outgoing.get(e.source) ?? [];
    out.push(e);
    outgoing.set(e.source, out);
  }

  return { nodes: nodeMap, edges, incoming, outgoing };
}

/* ------------------------------------------------------------------ */
/*  encodePart                                                        */
/* ------------------------------------------------------------------ */

describe('encodePart', () => {
  it('passes alphanumeric characters through unchanged', () => {
    expect(encodePart('hello123')).toBe('hello123');
  });

  it('passes hyphens through unchanged', () => {
    expect(encodePart('SL-001')).toBe('SL-001');
  });

  it('passes underscores through unchanged', () => {
    expect(encodePart('owning_slice')).toBe('owning_slice');
  });

  it('encodes special characters as _hex', () => {
    // ':' is 0x3a → '_3a'
    expect(encodePart('a:b')).toBe('a_3ab');
  });

  it('encodes space as _20', () => {
    expect(encodePart('hello world')).toBe('hello_20world');
  });

  it('returns empty string for empty input', () => {
    expect(encodePart('')).toBe('');
  });

  it('encodes unicode characters', () => {
    // 'é' is U+00E9, charCodeAt(0)=0xE9 → _e9
    const result = encodePart('café');
    expect(result).toBe('caf_e9');
  });

  it('pads single-digit hex values with leading zero', () => {
    // '\t' (tab) charCodeAt = 9 = 0x9 → '_09'
    expect(encodePart('a\tb')).toBe('a_09b');
  });
});

/* ------------------------------------------------------------------ */
/*  pascalToSnake                                                     */
/* ------------------------------------------------------------------ */

describe('pascalToSnake', () => {
  it('converts "OwningSlice" to "owning_slice"', () => {
    expect(pascalToSnake('OwningSlice')).toBe('owning_slice');
  });

  it('converts "HasParent" to "has_parent"', () => {
    expect(pascalToSnake('HasParent')).toBe('has_parent');
  });

  it('returns already-snake-case unchanged (lowercased)', () => {
    expect(pascalToSnake('owning_slice')).toBe('owning_slice');
  });

  it('lowercases all-uppercase input without inserting underscores', () => {
    expect(pascalToSnake('ABC')).toBe('abc');
  });

  it('handles single word', () => {
    expect(pascalToSnake('Slice')).toBe('slice');
  });

  it('handles consecutive capitals', () => {
    // "URLParser": 'L'(upper) preceded by 'R'(upper) → no _
    // Only 'a'(lower) → 'P'(upper) triggers underscore
    // No lowercase→uppercase transitions exist → just lowercases.
    expect(pascalToSnake('URLParser')).toBe('urlparser');
  });

  it('handles digits before capital', () => {
    expect(pascalToSnake('Edge2Follow')).toBe('edge2_follow');
  });

  it('returns empty string for empty input', () => {
    expect(pascalToSnake('')).toBe('');
  });
});

/* ------------------------------------------------------------------ */
/*  normalizeGraph                                                    */
/* ------------------------------------------------------------------ */

describe('normalizeGraph', () => {
  beforeEach(() => {
    state.graph.nodes.clear();
    state.graph.edges = [];
    state.graph.edgeById.clear();
    state.graph.incoming.clear();
    state.graph.outgoing.clear();
  });

  it('populates state.graph.nodes from raw nodes', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode({ title: 'Slice One', status: 'active', kind_label: 'Slice' }),
      },
      edges: [],
    };

    normalizeGraph(raw);

    expect(state.graph.nodes.size).toBe(1);
    const node = state.graph.nodes.get('SL-001');
    expect(node?.id).toBe('SL-001');
    expect(node?.title).toBe('Slice One');
    expect(node?.status).toBe('active');
    expect(node?.kindLabel).toBe('Slice');
    expect(node?.raw).toEqual(raw.nodes['SL-001']);
  });

  it('extracts kindPrefix from canonical id via splitPrefix', () => {
    const raw: RawGraph = {
      nodes: {
        'ADR-002': makeRawNode({ kind_label: 'ADR' }),
      },
      edges: [],
    };

    normalizeGraph(raw);

    expect(state.graph.nodes.get('ADR-002')?.kindPrefix).toBe('ADR');
  });

  it('falls back to kind_label for kindPrefix when id has no hyphen prefix', () => {
    const raw: RawGraph = {
      nodes: {
        'mem_019ed32d16b178629d58a6e1e1a0a797': makeRawNode({ kind_label: 'MEM' }),
      },
      edges: [],
    };

    normalizeGraph(raw);

    expect(state.graph.nodes.get('mem_019ed32d16b178629d58a6e1e1a0a797')?.kindPrefix).toBe('MEM');
  });

  it('leaves kindPrefix empty when neither splitPrefix nor kind_label is available', () => {
    const raw: RawGraph = {
      nodes: {
        somekey: makeRawNode({ kind_label: '' }),
      },
      edges: [],
    };

    normalizeGraph(raw);

    expect(state.graph.nodes.get('somekey')?.kindPrefix).toBe('');
  });

  it('populates edges from resolved targets only', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
        'SL-002': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Validated: 'Specs' },
          target: { Resolved: 'SL-002' },
        },
      ],
    };

    normalizeGraph(raw);

    expect(state.graph.edges).toHaveLength(1);
  });

  it('skips edges with unresolved targets (no Resolved key)', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Raw: 'some text' },
          target: {},
        },
      ],
    };

    normalizeGraph(raw);

    expect(state.graph.edges).toHaveLength(0);
  });

  it('skips edges with null target', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Raw: 'some text' },
          target: null,
        },
      ],
    };

    normalizeGraph(raw);

    expect(state.graph.edges).toHaveLength(0);
  });

  it('converts Validated edge labels via pascalToSnake', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
        'SL-002': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Validated: 'OwningSlice' },
          target: { Resolved: 'SL-002' },
        },
      ],
    };

    normalizeGraph(raw);

    expect(state.graph.edges.map((e) => e.label)).toEqual(['owning_slice']);
  });

  it('uses Raw edge labels as-is', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
        'SL-002': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Raw: 'custom-label' },
          target: { Resolved: 'SL-002' },
        },
      ],
    };

    normalizeGraph(raw);

    expect(state.graph.edges.map((e) => e.label)).toEqual(['custom-label']);
  });

  it('uses empty string when label has no Validated or Raw', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
        'SL-002': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: {},
          target: { Resolved: 'SL-002' },
        },
      ],
    };

    normalizeGraph(raw);

    expect(state.graph.edges.map((e) => e.label)).toEqual(['']);
  });

  it('builds edge id as e_{encodePart(source)}_{encodePart(label)}_{encodePart(target)}', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
        'SL-002': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Validated: 'Specs' },
          target: { Resolved: 'SL-002' },
        },
      ],
    };

    normalizeGraph(raw);

    expect(state.graph.edges.map((e) => e.id)).toEqual(['e_SL-001_specs_SL-002']);
  });

  it('coalesces duplicate edge ids', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
        'SL-002': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Validated: 'Specs' },
          target: { Resolved: 'SL-002' },
        },
        {
          source: 'SL-001',
          label: { Validated: 'Specs' },
          target: { Resolved: 'SL-002' },
        },
      ],
    };

    normalizeGraph(raw);

    expect(state.graph.edges).toHaveLength(1);
  });

  it('populates edgeById map', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
        'SL-002': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Validated: 'Specs' },
          target: { Resolved: 'SL-002' },
        },
      ],
    };

    normalizeGraph(raw);

    expect(state.graph.edgeById.size).toBe(1);
    expect(state.graph.edgeById.get('e_SL-001_specs_SL-002')?.id).toBe('e_SL-001_specs_SL-002');
  });

  it('populates incoming map', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
        'SL-002': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Validated: 'Specs' },
          target: { Resolved: 'SL-002' },
        },
      ],
    };

    normalizeGraph(raw);

    const incomingEdges = state.graph.incoming.get('SL-002');
    expect(incomingEdges).toBeDefined();
    expect(incomingEdges).toHaveLength(1);
    expect(incomingEdges?.map((e) => e.source)).toEqual(['SL-001']);
  });

  it('populates outgoing map', () => {
    const raw: RawGraph = {
      nodes: {
        'SL-001': makeRawNode(),
        'SL-002': makeRawNode(),
      },
      edges: [
        {
          source: 'SL-001',
          label: { Validated: 'Specs' },
          target: { Resolved: 'SL-002' },
        },
      ],
    };

    normalizeGraph(raw);

    const outgoingEdges = state.graph.outgoing.get('SL-001');
    expect(outgoingEdges).toBeDefined();
    expect(outgoingEdges).toHaveLength(1);
    expect(outgoingEdges?.map((e) => e.target)).toEqual(['SL-002']);
  });

  it('mutates state.graph in place', () => {
    const raw: RawGraph = {
      nodes: { 'SL-001': makeRawNode() },
      edges: [],
    };

    const nodesBefore = state.graph.nodes;
    const edgesBefore = state.graph.edges;

    normalizeGraph(raw);

    // model.js replaces the Map/array references (does not mutate in place).
    expect(state.graph.nodes).not.toBe(nodesBefore);
    expect(state.graph.edges).not.toBe(edgesBefore);
  });

  it('handles empty raw graph', () => {
    const raw: RawGraph = { nodes: {}, edges: [] };
    normalizeGraph(raw);

    expect(state.graph.nodes.size).toBe(0);
    expect(state.graph.edges).toHaveLength(0);
    expect(state.graph.edgeById.size).toBe(0);
    expect(state.graph.incoming.size).toBe(0);
    expect(state.graph.outgoing.size).toBe(0);
  });
});

/* ------------------------------------------------------------------ */
/*  findFocus                                                         */
/* ------------------------------------------------------------------ */

describe('findFocus', () => {
  it('returns first sorted node when query is null', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-002', title: 'B' }),
      makeCatalogNode({ id: 'SL-001', title: 'A' }),
    ]);

    // sorted alphabetically by id: SL-001 < SL-002
    expect(findFocus(null, graph)).toBe('SL-001');
  });

  it('returns first sorted node when query is empty string', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'ADR-001' }),
      makeCatalogNode({ id: 'SL-001' }),
    ]);

    // sorted alphabetically: ADR-001 < SL-001
    expect(findFocus('', graph)).toBe('ADR-001');
  });

  it('returns null when query is empty and graph has no nodes', () => {
    const graph = makeGraph([]);
    expect(findFocus('', graph)).toBeNull();
  });

  it('returns null when query is null and graph has no nodes', () => {
    const graph = makeGraph([]);
    expect(findFocus(null, graph)).toBeNull();
  });

  it('matches exact canonical id case-insensitively', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001' }),
    ]);

    expect(findFocus('sl-001', graph)).toBe('SL-001');
  });

  it('matches exact canonical id with same case', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001' }),
    ]);

    expect(findFocus('SL-001', graph)).toBe('SL-001');
  });

  it('resolves loose canonical "sl09" to "SL-009"', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-009' }),
    ]);

    expect(findFocus('sl09', graph)).toBe('SL-009');
  });

  it('resolves loose canonical "sl9" to "SL-009" (pads to 3 digits)', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-009' }),
    ]);

    expect(findFocus('sl9', graph)).toBe('SL-009');
  });

  it('resolves loose canonical "prd1" to "PRD-001"', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'PRD-001' }),
    ]);

    expect(findFocus('prd1', graph)).toBe('PRD-001');
  });

  it('resolves loose canonical with hyphen "sl-09" to "SL-009"', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-009' }),
    ]);

    expect(findFocus('sl-09', graph)).toBe('SL-009');
  });

  it('matches exact title case-insensitively', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001', title: 'My Slice' }),
      makeCatalogNode({ id: 'SL-002', title: 'Other' }),
    ]);

    expect(findFocus('my slice', graph)).toBe('SL-001');
  });

  it('title matching returns last match when multiple titles collide', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001', title: 'Same Title' }),
      makeCatalogNode({ id: 'SL-002', title: 'Same Title' }),
    ]);

    // Iterates all nodes — last one wins
    expect(findFocus('same title', graph)).toBe('SL-002');
  });

  it('matches substring in node id', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'PRD-010', title: 'Something Else' }),
    ]);

    expect(findFocus('prd', graph)).toBe('PRD-010');
  });

  it('matches substring in node title', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001', title: 'Authentication Module' }),
    ]);

    expect(findFocus('auth', graph)).toBe('SL-001');
  });

  it('matches substring in node status', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001', status: 'draft' }),
    ]);

    expect(findFocus('dra', graph)).toBe('SL-001');
  });

  it('matches substring in kindLabel', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001', kindLabel: 'Slice' }),
    ]);

    expect(findFocus('lic', graph)).toBe('SL-001');
  });

  it('chooses shortest id when multiple nodes match substring', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001', title: 'Authentication' }),
      makeCatalogNode({ id: 'SL-100', title: 'Authentication Flow' }),
    ]);

    // Both match "auth" in title, SL-001 is shorter
    expect(findFocus('auth', graph)).toBe('SL-001');
  });

  it('returns null when no match found', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001', title: 'First' }),
    ]);

    expect(findFocus('nonexistent', graph)).toBeNull();
  });

  it('step 2 (exact) takes priority over step 3 (loose)', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-009' }),
    ]);

    // "SL-009" matches exactly
    expect(findFocus('SL-009', graph)).toBe('SL-009');
  });

  it('step 3 (loose) takes priority over step 4 (title)', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-009', title: 'SL09' }),
    ]);

    // "sl09" loose-canonical resolves to "SL-009" which exists
    expect(findFocus('sl09', graph)).toBe('SL-009');
  });

  it('step 4 (title) takes priority over step 5 (substring)', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-999', title: 'Substring Match' }),
      makeCatalogNode({ id: 'SL-001', title: 'Exact' }),
    ]);

    // "exact" matches title of SL-001 exactly
    expect(findFocus('exact', graph)).toBe('SL-001');
  });
});

/* ------------------------------------------------------------------ */
/*  resolveFocus                                                      */
/* ------------------------------------------------------------------ */

describe('resolveFocus', () => {
  it('returns result from findFocus when match found', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001' }),
    ]);

    expect(resolveFocus('sl-001', graph)).toBe('SL-001');
  });

  it('falls back to first sorted node when no match', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-002' }),
      makeCatalogNode({ id: 'ADR-001' }),
    ]);

    expect(resolveFocus('NONEXISTENT', graph)).toBe('ADR-001');
  });

  it('falls back to first sorted node when query is null and graph has nodes', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001' }),
    ]);

    expect(resolveFocus(null, graph)).toBe('SL-001');
  });

  it('returns null when graph is empty', () => {
    const graph = makeGraph([]);

    expect(resolveFocus('anything', graph)).toBeNull();
  });
});

/* ------------------------------------------------------------------ */
/*  neighbourhood                                                     */
/* ------------------------------------------------------------------ */

describe('neighbourhood', () => {
  /*
   * Test topology:
   *   SL-001 --specs--> SL-002
   *   SL-001 --governed_by--> ADR-001
   *   SL-003 --specs--> SL-001
   *   SL-004  (isolated)
   */
  const nodes = [
    makeCatalogNode({ id: 'SL-001', kindPrefix: 'SL' }),
    makeCatalogNode({ id: 'SL-002', kindPrefix: 'SL' }),
    makeCatalogNode({ id: 'ADR-001', kindPrefix: 'ADR' }),
    makeCatalogNode({ id: 'SL-003', kindPrefix: 'SL' }),
    makeCatalogNode({ id: 'SL-004', kindPrefix: 'SL' }),
  ];

  const edges = [
    makeEdge({ id: 'e_SL-001_specs_SL-002', source: 'SL-001', target: 'SL-002', label: 'specs' }),
    makeEdge({ id: 'e_SL-001_governed_by_ADR-001', source: 'SL-001', target: 'ADR-001', label: 'governed_by' }),
    makeEdge({ id: 'e_SL-003_specs_SL-001', source: 'SL-003', target: 'SL-001', label: 'specs' }),
  ];

  const graph = makeGraph(nodes, edges);

  it('depth 0 returns only the focus node', () => {
    const result = neighbourhood('SL-001', 0, graph);

    expect(result.nodes.size).toBe(1);
    expect(result.nodes.has('SL-001')).toBe(true);
    expect(result.edges).toHaveLength(0);
  });

  it('depth 1 returns focus node plus direct neighbours', () => {
    const result = neighbourhood('SL-001', 1, graph);

    // SL-001 + outgoing (SL-002, ADR-001) + incoming (SL-003) = 4
    expect(result.nodes.size).toBe(4);
    expect(result.nodes.has('SL-001')).toBe(true);
    expect(result.nodes.has('SL-002')).toBe(true);
    expect(result.nodes.has('ADR-001')).toBe(true);
    expect(result.nodes.has('SL-003')).toBe(true);
    expect(result.nodes.has('SL-004')).toBe(false);

    // 3 edges total (2 outgoing + 1 incoming)
    expect(result.edges).toHaveLength(3);
  });

  it('depth 2 returns focus node plus neighbours-of-neighbours', () => {
    const result = neighbourhood('SL-001', 2, graph);

    // Depth 0: SL-001
    // Depth 1: SL-002, ADR-001, SL-003
    // Depth 2: neighbours only connect back to SL-001, so 4 total
    expect(result.nodes.size).toBe(4);
  });

  it('includes both outgoing and incoming edges', () => {
    const result = neighbourhood('SL-001', 1, graph);

    const edgeIds = result.edges.map((e) => e.id);
    expect(edgeIds).toContain('e_SL-001_specs_SL-002');
    expect(edgeIds).toContain('e_SL-001_governed_by_ADR-001');
    expect(edgeIds).toContain('e_SL-003_specs_SL-001');
  });

  it('deduplicates edges by id', () => {
    const dupEdges = [
      ...edges,
      makeEdge({ id: 'e_SL-001_specs_SL-002', source: 'SL-001', target: 'SL-002', label: 'specs' }),
    ];
    const dupGraph = makeGraph(nodes, dupEdges);

    const result = neighbourhood('SL-001', 1, dupGraph);

    // Still only 3 unique edges
    expect(result.edges).toHaveLength(3);
  });

  it('clamps depth to 0 for negative values', () => {
    const result = neighbourhood('SL-001', -1, graph);

    expect(result.nodes.size).toBe(1);
    expect(result.edges).toHaveLength(0);
  });

  it('clamps depth to 3 for values above 3', () => {
    const result = neighbourhood('SL-001', 10, graph);

    expect(result.nodes.size).toBe(4);
  });

  it('returns only focus node for isolated node at any depth', () => {
    const result = neighbourhood('SL-004', 2, graph);

    expect(result.nodes.size).toBe(1);
    expect(result.nodes.has('SL-004')).toBe(true);
    expect(result.edges).toHaveLength(0);
  });

  it('depth 0 edges is empty array', () => {
    const result = neighbourhood('SL-001', 0, graph);

    expect(result.edges).toEqual([]);
  });

  it('nodes is a Set', () => {
    const result = neighbourhood('SL-001', 1, graph);

    expect(result.nodes).toBeInstanceOf(Set);
  });
});

/* ------------------------------------------------------------------ */
/*  kindOrder                                                         */
/* ------------------------------------------------------------------ */

describe('kindOrder', () => {
  it('is a static record with expected entries', () => {
    expect(kindOrder).toBeDefined();
    expect(typeof kindOrder).toBe('object');
  });

  it('PRD is 1', () => {
    expect(kindOrder.PRD).toBe(1);
  });

  it('ADRs are 2', () => {
    expect(kindOrder.ADR).toBe(2);
  });

  it('SL is 4', () => {
    expect(kindOrder.SL).toBe(4);
  });

  it('REQ is 8', () => {
    expect(kindOrder.REQ).toBe(8);
  });

  it('MEM is 10', () => {
    expect(kindOrder.MEM).toBe(10);
  });

  it('CM is 20 (highest)', () => {
    expect(kindOrder.CM).toBe(20);
  });

  it('has exactly 21 keys', () => {
    expect(Object.keys(kindOrder)).toHaveLength(21);
  });
});

/* ------------------------------------------------------------------ */
/*  kinds                                                             */
/* ------------------------------------------------------------------ */

describe('kinds', () => {
  it('returns an empty map for no nodes', () => {
    const graph = makeGraph([]);

    const result = kinds(graph.nodes);

    expect(result.size).toBe(0);
  });

  it('counts nodes by kindPrefix', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'SL-001', kindPrefix: 'SL' }),
      makeCatalogNode({ id: 'SL-002', kindPrefix: 'SL' }),
      makeCatalogNode({ id: 'ADR-001', kindPrefix: 'ADR' }),
    ]);

    const result = kinds(graph.nodes);

    expect(result.get('ADR')).toBe(1);
    expect(result.get('SL')).toBe(2);
  });

  it('sorts results alphabetically by prefix', () => {
    const graph = makeGraph([
      makeCatalogNode({ id: 'PRD-001', kindPrefix: 'PRD' }),
      makeCatalogNode({ id: 'ADR-001', kindPrefix: 'ADR' }),
      makeCatalogNode({ id: 'SL-001', kindPrefix: 'SL' }),
    ]);

    const result = kinds(graph.nodes);

    const keys = [...result.keys()];
    expect(keys).toEqual(['ADR', 'PRD', 'SL']);
  });

  it('returns a Map', () => {
    const graph = makeGraph([makeCatalogNode({ kindPrefix: 'SL' })]);

    const result = kinds(graph.nodes);

    expect(result).toBeInstanceOf(Map);
  });
});

/* ------------------------------------------------------------------ */
/*  searchFilter                                                      */
/* ------------------------------------------------------------------ */

describe('searchFilter', () => {
  const nodes = [
    makeCatalogNode({ id: 'PRD-001', title: 'Product', kindPrefix: 'PRD' }),
    makeCatalogNode({ id: 'ADR-002', title: 'Architecture', kindPrefix: 'ADR' }),
    makeCatalogNode({ id: 'SL-010', title: 'Authentication Slice', kindPrefix: 'SL' }),
    makeCatalogNode({ id: 'SL-001', title: 'Core Slice', kindPrefix: 'SL' }),
    makeCatalogNode({ id: 'REQ-060', title: 'Auth Requirement', kindPrefix: 'REQ' }),
  ];

  const graph = makeGraph(nodes);

  it('returns all nodes sorted by compareNodes when query is null', () => {
    const result = searchFilter(null, graph);

    // Sorted by kindOrder: PRD(1) < ADR(2) < SL(4) < REQ(8)
    // SL-001 < SL-010 by numeric suffix
    expect(result.map((n) => n.id)).toEqual([
      'PRD-001', 'ADR-002', 'SL-001', 'SL-010', 'REQ-060',
    ]);
  });

  it('returns all nodes sorted when query is empty string', () => {
    const result = searchFilter('', graph);

    expect(result.map((n) => n.id)).toEqual([
      'PRD-001', 'ADR-002', 'SL-001', 'SL-010', 'REQ-060',
    ]);
  });

  it('filters by case-insensitive substring in id', () => {
    const result = searchFilter('req', graph);

    expect(result.map((n) => n.id)).toEqual(['REQ-060']);
  });

  it('filters by case-insensitive substring in title', () => {
    const result = searchFilter('auth', graph);

    // "auth" matches title of SL-001 ("Core Slice" → no), SL-010 ("Authentication Slice"), REQ-060 ("Auth Requirement")
    expect(result.map((n) => n.id)).toEqual(['SL-010', 'REQ-060']);
  });

  it('matches substring in id or title', () => {
    // "sl" matches id of SL-xxx nodes and title of Core Slice, Authentication Slice
    const result = searchFilter('sl', graph);

    expect(result.map((n) => n.id)).toEqual(['SL-001', 'SL-010']);
  });

  it('results are sorted by compareNodes', () => {
    const result = searchFilter('adr', graph);

    expect(result.map((n) => n.id)).toEqual(['ADR-002']);
  });

  it('returns empty array when no match', () => {
    const result = searchFilter('zzz_nonexistent', graph);

    expect(result).toHaveLength(0);
  });
});

/* ------------------------------------------------------------------ */
/*  compareEdgesBySource                                              */
/* ------------------------------------------------------------------ */

describe('compareEdgesBySource', () => {
  beforeEach(() => {
    state.graph.nodes.clear();
  });

  it('returns negative when source of first edge sorts before source of second', () => {
    state.graph.nodes.set('SL-001', makeCatalogNode({ id: 'SL-001', kindPrefix: 'SL' }));
    state.graph.nodes.set('SL-002', makeCatalogNode({ id: 'SL-002', kindPrefix: 'SL' }));

    const ea = makeEdge({ id: 'e_1', source: 'SL-001', target: 'SL-003', label: 'specs' });
    const eb = makeEdge({ id: 'e_2', source: 'SL-002', target: 'SL-004', label: 'specs' });

    expect(compareEdgesBySource(ea, eb)).toBeLessThan(0);
  });

  it('returns positive when source of first edge sorts after source of second', () => {
    state.graph.nodes.set('SL-001', makeCatalogNode({ id: 'SL-001', kindPrefix: 'SL' }));
    state.graph.nodes.set('SL-002', makeCatalogNode({ id: 'SL-002', kindPrefix: 'SL' }));

    const ea = makeEdge({ id: 'e_1', source: 'SL-002', target: 'SL-003', label: 'specs' });
    const eb = makeEdge({ id: 'e_2', source: 'SL-001', target: 'SL-004', label: 'specs' });

    expect(compareEdgesBySource(ea, eb)).toBeGreaterThan(0);
  });

  it('returns zero when sources are equal', () => {
    state.graph.nodes.set('SL-001', makeCatalogNode({ id: 'SL-001', kindPrefix: 'SL' }));

    const ea = makeEdge({ id: 'e_same', source: 'SL-001', target: 'SL-002' });
    const eb = makeEdge({ id: 'e_same', source: 'SL-001', target: 'SL-003' });

    expect(compareEdgesBySource(ea, eb)).toBe(0);
  });

  it('falls back to edge id comparison when both sources are not in state.graph.nodes', () => {
    const ea = makeEdge({ id: 'e_a', source: 'MISSING-1', target: 'SL-003' });
    const eb = makeEdge({ id: 'e_b', source: 'MISSING-2', target: 'SL-004' });

    // Falls back: e_a < e_b → -1
    expect(compareEdgesBySource(ea, eb)).toBeLessThan(0);
  });

  it('falls back to edge id comparison when first source is missing', () => {
    state.graph.nodes.set('SL-002', makeCatalogNode({ id: 'SL-002', kindPrefix: 'SL' }));

    const ea = makeEdge({ id: 'e_a', source: 'MISSING', target: 'SL-003' });
    const eb = makeEdge({ id: 'e_b', source: 'SL-002', target: 'SL-004' });

    // First source missing → fallback via edge ids: e_a < e_b → -1
    expect(compareEdgesBySource(ea, eb)).toBeLessThan(0);
  });

  it('falls back to edge id comparison when second source is missing', () => {
    state.graph.nodes.set('SL-001', makeCatalogNode({ id: 'SL-001', kindPrefix: 'SL' }));

    const ea = makeEdge({ id: 'e_x', source: 'SL-001', target: 'SL-003' });
    const eb = makeEdge({ id: 'e_y', source: 'MISSING', target: 'SL-004' });

    // Second source missing → fallback via edge ids: e_x < e_y → -1
    expect(compareEdgesBySource(ea, eb)).toBeLessThan(0);
  });

  it('uses kindOrder-based comparison via compareNodes', () => {
    state.graph.nodes.set('ADR-001', makeCatalogNode({ id: 'ADR-001', kindPrefix: 'ADR' }));
    state.graph.nodes.set('SL-001', makeCatalogNode({ id: 'SL-001', kindPrefix: 'SL' }));

    const ea = makeEdge({ id: 'e_1', source: 'ADR-001', target: 'SL-003' });
    const eb = makeEdge({ id: 'e_2', source: 'SL-001', target: 'SL-004' });

    // ADR(2) < SL(4) → ADR edge sorts first
    expect(compareEdgesBySource(ea, eb)).toBeLessThan(0);
  });
});

/* ------------------------------------------------------------------ */
/*  normalizeConceptMap                                               */
/* ------------------------------------------------------------------ */

describe('normalizeConceptMap', () => {
  it('maps a full raw CM to ConceptMap shape', () => {
    const raw = {
      id: 'cm-001',
      title: 'Test Concept Map',
      status: 'active',
      description: 'A test CM',
      dsl_hash: 'abc123',
      nodes: [{ key: 'n1', label: 'Node 1' }],
      edges: [{ from_key: 'n1', from_label: 'Node 1', rel: 'relatesTo', to_key: 'n2', to_label: 'Node 2' }],
      diagnostics: [{ severity: 'info', message: 'ok' }],
    };

    const cm = normalizeConceptMap(raw);

    expect(cm.id).toBe('cm-001');
    expect(cm.title).toBe('Test Concept Map');
    expect(cm.status).toBe('active');
    expect(cm.description).toBe('A test CM');
    expect(cm.dslHash).toBe('abc123');
    expect(cm.nodes.map((n) => n.key)).toEqual(['n1']);
    expect(cm.nodes.map((n) => n.label)).toEqual(['Node 1']);
    expect(cm.edges.map((e) => e.from_key)).toEqual(['n1']);
    expect(cm.edges.map((e) => e.rel)).toEqual(['relatesTo']);
    expect(cm.edges.map((e) => e.to_key)).toEqual(['n2']);
    expect(cm.diagnostics).toHaveLength(1);
  });

  it('defaults description to empty string when missing', () => {
    const raw = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
    };

    const cm = normalizeConceptMap(raw);

    expect(cm.description).toBe('');
  });

  it('defaults dslHash to empty string when missing', () => {
    const raw = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
    };

    const cm = normalizeConceptMap(raw);

    expect(cm.dslHash).toBe('');
  });

  it('defaults nodes to empty array when missing', () => {
    const raw = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
    };

    const cm = normalizeConceptMap(raw);

    expect(cm.nodes).toEqual([]);
  });

  it('defaults edges to empty array when missing', () => {
    const raw = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
    };

    const cm = normalizeConceptMap(raw);

    expect(cm.edges).toEqual([]);
  });

  it('defaults diagnostics to empty array when missing', () => {
    const raw = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
    };

    const cm = normalizeConceptMap(raw);

    expect(cm.diagnostics).toEqual([]);
  });
});

/* ------------------------------------------------------------------ */
/*  buildNodeLabelList                                                */
/* ------------------------------------------------------------------ */

describe('buildNodeLabelList', () => {
  it('returns unique deduplicated node labels', () => {
    const cm: ConceptMap = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
      description: '',
      dslHash: '',
      nodes: [
        { key: 'n1', label: 'Label A' },
        { key: 'n2', label: 'Label B' },
        { key: 'n3', label: 'Label A' },
      ],
      edges: [],
      diagnostics: [],
    };

    const labels = buildNodeLabelList(cm);

    expect(labels).toHaveLength(2);
    expect(labels).toContain('Label A');
    expect(labels).toContain('Label B');
  });

  it('returns empty array when cm is null', () => {
    expect(buildNodeLabelList(null)).toEqual([]);
  });

  it('returns empty array when cm is undefined', () => {
    expect(buildNodeLabelList(undefined)).toEqual([]);
  });

  it('returns empty array when cm has no nodes', () => {
    const cm: ConceptMap = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
      description: '',
      dslHash: '',
      nodes: [],
      edges: [],
      diagnostics: [],
    };

    const labels = buildNodeLabelList(cm);

    expect(labels).toEqual([]);
  });

  it('preserves insertion order (first occurrence)', () => {
    const cm: ConceptMap = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
      description: '',
      dslHash: '',
      nodes: [
        { key: 'n1', label: 'Z' },
        { key: 'n2', label: 'A' },
        { key: 'n3', label: 'Z' },
      ],
      edges: [],
      diagnostics: [],
    };

    const labels = buildNodeLabelList(cm);

    expect(labels).toEqual(['Z', 'A']);
  });
});

/* ------------------------------------------------------------------ */
/*  buildRelLabelList                                                 */
/* ------------------------------------------------------------------ */

describe('buildRelLabelList', () => {
  it('returns unique deduplicated edge rels', () => {
    const cm: ConceptMap = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
      description: '',
      dslHash: '',
      nodes: [],
      edges: [
        { from_key: 'a', from_label: 'A', rel: 'relatesTo', to_key: 'b', to_label: 'B' },
        { from_key: 'b', from_label: 'B', rel: 'owns', to_key: 'c', to_label: 'C' },
        { from_key: 'a', from_label: 'A', rel: 'relatesTo', to_key: 'c', to_label: 'C' },
      ],
      diagnostics: [],
    };

    const rels = buildRelLabelList(cm);

    expect(rels).toHaveLength(2);
    expect(rels).toContain('relatesTo');
    expect(rels).toContain('owns');
  });

  it('returns empty array when cm is null', () => {
    expect(buildRelLabelList(null)).toEqual([]);
  });

  it('returns empty array when cm is undefined', () => {
    expect(buildRelLabelList(undefined)).toEqual([]);
  });

  it('returns empty array when cm has no edges', () => {
    const cm: ConceptMap = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
      description: '',
      dslHash: '',
      nodes: [],
      edges: [],
      diagnostics: [],
    };

    const rels = buildRelLabelList(cm);

    expect(rels).toEqual([]);
  });

  it('preserves insertion order (first occurrence)', () => {
    const cm: ConceptMap = {
      id: 'cm-001',
      title: 'Test',
      status: 'active',
      description: '',
      dslHash: '',
      nodes: [],
      edges: [
        { from_key: 'a', from_label: 'A', rel: 'ZRel', to_key: 'b', to_label: 'B' },
        { from_key: 'b', from_label: 'B', rel: 'ARel', to_key: 'c', to_label: 'C' },
        { from_key: 'a', from_label: 'A', rel: 'ZRel', to_key: 'c', to_label: 'C' },
      ],
      diagnostics: [],
    };

    const rels = buildRelLabelList(cm);

    expect(rels).toEqual(['ZRel', 'ARel']);
  });
});

/* ------------------------------------------------------------------ */
/*  setActionabilityView                                              */
/* ------------------------------------------------------------------ */

describe('setActionabilityView', () => {
  beforeEach(() => {
    state.actionabilityView = null;
  });

  it('sets state.actionabilityView to the given view', () => {
    const view: ActionabilityView = {
      kind: 'test',
      nodes: [],
      edges: [],
    };

    setActionabilityView(view);

    expect(state.actionabilityView).toBe(view);
  });

  it('sets state.actionabilityView to null', () => {
    const view: ActionabilityView = {
      kind: 'test',
      nodes: [],
      edges: [],
    };

    setActionabilityView(view);
    expect(state.actionabilityView).toBe(view);

    setActionabilityView(null);
    expect(state.actionabilityView).toBeNull();
  });
});

/* ------------------------------------------------------------------ */
/*  cmNeighbourhood                                                   */
/* ------------------------------------------------------------------ */

describe('cmNeighbourhood', () => {
  /*
   * Test CM topology:
   *   n1 --relatesTo--> n2
   *   n1 --owns--> n3
   *   n4 (isolated)
   */
  const cmNodes: CmNode[] = [
    { key: 'n1', label: 'Node 1' },
    { key: 'n2', label: 'Node 2' },
    { key: 'n3', label: 'Node 3' },
    { key: 'n4', label: 'Node 4' },
  ];

  const cmEdges: CmEdge[] = [
    { from_key: 'n1', from_label: 'Node 1', rel: 'relatesTo', to_key: 'n2', to_label: 'Node 2' },
    { from_key: 'n1', from_label: 'Node 1', rel: 'owns', to_key: 'n3', to_label: 'Node 3' },
  ];

  const cm: ConceptMap = {
    id: 'cm-001',
    title: 'Test CM',
    status: 'active',
    description: '',
    dslHash: '',
    nodes: cmNodes,
    edges: cmEdges,
    diagnostics: [],
  };

  it('returns empty nodes and edges when cm is null', () => {
    const result = cmNeighbourhood(null, 'n1', 1);

    expect(result.nodes).toEqual([]);
    expect(result.edges).toEqual([]);
  });

  it('returns empty nodes and edges when cm is undefined', () => {
    const result = cmNeighbourhood(undefined, 'n1', 1);

    expect(result.nodes).toEqual([]);
    expect(result.edges).toEqual([]);
  });

  it('returns all nodes and edges when focusKey is null', () => {
    const result = cmNeighbourhood(cm, null, 1);

    expect(result.nodes).toHaveLength(4);
    expect(result.edges).toHaveLength(2);
  });

  it('returns all nodes and edges when focusKey is undefined', () => {
    const result = cmNeighbourhood(cm, undefined, 1);

    expect(result.nodes).toHaveLength(4);
    expect(result.edges).toHaveLength(2);
  });

  it('returns all nodes and edges when focusKey is not in nodes', () => {
    const result = cmNeighbourhood(cm, 'nonexistent', 1);

    expect(result.nodes).toHaveLength(4);
    expect(result.nodes).toEqual(cmNodes);
    expect(result.edges).toHaveLength(2);
  });

  it('depth 0 returns only focus node', () => {
    const result = cmNeighbourhood(cm, 'n1', 0);

    expect(result.nodes.map((n) => n.key)).toEqual(['n1']);
    expect(result.edges).toHaveLength(0);
  });

  it('depth 1 returns focus node and direct neighbours', () => {
    const result = cmNeighbourhood(cm, 'n1', 1);

    // n1 + n2 + n3 = 3 (n4 is isolated, not visited)
    expect(result.nodes).toHaveLength(3);
    const keys = result.nodes.map((n) => n.key);
    expect(keys).toContain('n1');
    expect(keys).toContain('n2');
    expect(keys).toContain('n3');
    expect(keys).not.toContain('n4');

    // Both edges connect visited nodes
    expect(result.edges).toHaveLength(2);
  });

  it('depth 1 from isolated node returns only that node', () => {
    const result = cmNeighbourhood(cm, 'n4', 1);

    expect(result.nodes.map((n) => n.key)).toEqual(['n4']);
    expect(result.edges).toHaveLength(0);
  });

  it('clamps depth to 0 for negative values', () => {
    const result = cmNeighbourhood(cm, 'n1', -5);

    expect(result.nodes.map((n) => n.key)).toEqual(['n1']);
  });

  it('clamps depth to 3 for values above 3', () => {
    const result = cmNeighbourhood(cm, 'n1', 100);

    // Same as depth 2 (max reachable in this topology)
    expect(result.nodes).toHaveLength(3);
  });

  it('preserves original node order when filtering to visited set', () => {
    // Reorder nodes: put n3 first, n1 last
    const reorderedNodes: CmNode[] = [
      { key: 'n3', label: 'Node 3' },
      { key: 'n2', label: 'Node 2' },
      { key: 'n1', label: 'Node 1' },
      { key: 'n4', label: 'Node 4' },
    ];

    const reorderedCm: ConceptMap = {
      ...cm,
      nodes: reorderedNodes,
    };

    const result = cmNeighbourhood(reorderedCm, 'n1', 1);

    // Should preserve original order: n3, n2, n1 (n4 filtered out)
    expect(result.nodes.map((n) => n.key)).toEqual(['n3', 'n2', 'n1']);
  });

  it('filters edges to only those where both ends are in visited set', () => {
    const extendedEdges: CmEdge[] = [
      ...cmEdges,
      { from_key: 'n2', from_label: 'Node 2', rel: 'relatesTo', to_key: 'n4', to_label: 'Node 4' },
    ];
    const extendedCm: ConceptMap = {
      ...cm,
      edges: extendedEdges,
    };

    const result = cmNeighbourhood(extendedCm, 'n1', 1);

    // n2-n4 edge should be excluded because n4 is not in visited set at depth 1
    expect(result.edges).toHaveLength(2);
    // The edge to n4 should be excluded — only n1-n2 and n1-n3 remain
    const fromKeys = result.edges.map((e) => e.from_key);
    const rels = result.edges.map((e) => e.rel);
    expect(fromKeys).toContain('n1');
    expect(fromKeys).not.toContain('n4');
    expect(rels).toContain('relatesTo');
    expect(rels).toContain('owns');
  });

  it('BFS traverses undirected (both directions)', () => {
    // From n2's perspective: n2 is connected to n1 via relatesTo (incoming)
    const result = cmNeighbourhood(cm, 'n2', 1);

    expect(result.nodes).toHaveLength(2);
    const keys = result.nodes.map((n) => n.key);
    expect(keys).toContain('n2');
    expect(keys).toContain('n1');
  });

  it('depth 2 reaches nodes two hops away', () => {
    // n2 → n1 → n3 (2 hops from n2)
    const result = cmNeighbourhood(cm, 'n2', 2);

    expect(result.nodes).toHaveLength(3);
    const keys = result.nodes.map((n) => n.key);
    expect(keys).toContain('n1');
    expect(keys).toContain('n2');
    expect(keys).toContain('n3');
  });
});

/* ------------------------------------------------------------------ */
/*  focusTransition (SL-110 — focus-change drives the view)            */
/* ------------------------------------------------------------------ */

describe('focusTransition', () => {
  // Revision 2 (D2 reversed, RV-098 F-5): on the actionability graph a member
  // zooms, a non-member switches to Semantic (clearing the stale zoom); Semantic
  // focus never auto-switches (echoes the passed zoom). Signature takes focusId,
  // not a node (the only use was its id == focusId); requiredMode is gone.

  it('actionability + member zooms to the focused id', () => {
    const t = focusTransition('actionability', 'SL-005', true, null);
    expect(t).toEqual({ viewMode: 'actionability', priorityZoomId: 'SL-005' });
  });

  it('actionability + non-member switches to Semantic and clears zoom (the reversal)', () => {
    const t = focusTransition('actionability', 'REQ-010', false, 'SL-009');
    expect(t).toEqual({ viewMode: 'semantic', priorityZoomId: null });
  });

  it('semantic focus never auto-switches — echoes the passed zoom', () => {
    const t = focusTransition('semantic', 'SL-005', false, 'SL-003');
    expect(t).toEqual({ viewMode: 'semantic', priorityZoomId: 'SL-003' });
  });

  it('a CM (never an actionability member) on the actionability graph → Semantic, null', () => {
    const t = focusTransition('actionability', 'CM-001', false, 'SL-007');
    expect(t).toEqual({ viewMode: 'semantic', priorityZoomId: null });
  });

  it('a CM focused while already Semantic just echoes the zoom (no forced clear)', () => {
    const t = focusTransition('semantic', 'CM-002', false, 'SL-007');
    expect(t).toEqual({ viewMode: 'semantic', priorityZoomId: 'SL-007' });
  });

  it('null focusId on semantic is safe — echoes the zoom', () => {
    expect(() => focusTransition('semantic', null, false, null)).not.toThrow();
    const t = focusTransition('semantic', null, false, 'SL-001');
    expect(t).toEqual({ viewMode: 'semantic', priorityZoomId: 'SL-001' });
  });

  it('null focusId on actionability + non-member → Semantic, null', () => {
    const t = focusTransition('actionability', null, false, 'SL-001');
    expect(t).toEqual({ viewMode: 'semantic', priorityZoomId: null });
  });
});
