/**
 * @vitest-environment jsdom
 *
 * Behaviour-contract tests for dot.ts — captures the EXACT observable
 * behaviour of dot.js as a contract that the TypeScript rewrite must satisfy.
 *
 * These tests initially FAIL (RED) because dot.ts doesn't exist yet.
 * The satisfier creates dot.ts to make them pass.
 */

import { describe, it, expect, vi } from 'vitest';
import type { CatalogNode, Edge } from './types';

// Mock the state module — dot.ts will import from here for graphToDot.
vi.mock('./state', () => ({
  state: {
    graph: {
      nodes: new Map<string, CatalogNode>(),
    },
  },
}));

import { state } from './state';
import {
  dotQuote,
  graphToDot,
  cmGraphToDot,
  NODE_STYLES,
  DEFAULT_NODE_STYLE,
  nodeAttrs,
  edgeAttrs,
  escapeStringContent,
} from './dot';

/* ------------------------------------------------------------------ */
/*  Helpers                                                           */
/* ------------------------------------------------------------------ */

function makeNode(overrides: Partial<CatalogNode> = {}): CatalogNode {
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

/* ------------------------------------------------------------------ */
/*  dotQuote                                                          */
/* ------------------------------------------------------------------ */

describe('dotQuote', () => {
  it('escapes backslash', () => {
    expect(dotQuote('a\\b')).toBe('a\\\\b');
  });

  it('escapes double-quote', () => {
    expect(dotQuote('a"b')).toBe('a\\"b');
  });

  it('escapes newline', () => {
    expect(dotQuote('a\nb')).toBe('a\\nb');
  });

  it('returns normal text unchanged', () => {
    expect(dotQuote('hello world')).toBe('hello world');
  });

  it('returns empty string for empty input', () => {
    expect(dotQuote('')).toBe('');
  });

  it('escapes multiple occurrences', () => {
    expect(dotQuote('a\\b\\c')).toBe('a\\\\b\\\\c');
  });

  it('escapes mixed characters', () => {
    expect(dotQuote('line1\n"quoted"\\path')).toBe('line1\\n\\"quoted\\"\\\\path');
  });
});

/* ------------------------------------------------------------------ */
/*  escapeStringContent                                               */
/* ------------------------------------------------------------------ */

describe('escapeStringContent', () => {
  it('escapes backslash', () => {
    expect(escapeStringContent('a\\b')).toBe('a\\\\b');
  });

  it('escapes double-quote', () => {
    expect(escapeStringContent('a"b')).toBe('a\\"b');
  });

  it('escapes newline', () => {
    expect(escapeStringContent('a\nb')).toBe('a\\nb');
  });

  it('escapes >', () => {
    expect(escapeStringContent('a>b')).toBe('a\\>b');
  });

  it('escapes ]', () => {
    expect(escapeStringContent('a]b')).toBe('a\\]b');
  });

  it('escapes }', () => {
    expect(escapeStringContent('a}b')).toBe('a\\}b');
  });

  it('returns normal text unchanged', () => {
    expect(escapeStringContent('hello world')).toBe('hello world');
  });

  it('returns empty string for empty input', () => {
    expect(escapeStringContent('')).toBe('');
  });
});

/* ------------------------------------------------------------------ */
/*  NODE_STYLES & DEFAULT_NODE_STYLE                                  */
/* ------------------------------------------------------------------ */

describe('NODE_STYLES', () => {
  const expectedKeys = [
    'SL', 'ADR', 'POL', 'STD', 'PRD', 'SPEC',
    'REQ', 'ISS', 'IMP', 'CHR', 'RSK', 'IDE',
    'RV', 'REC', 'ASM', 'DEC', 'QUE', 'CON',
    'REV', 'CM',
  ];

  it('has exactly 21 kind-prefix keys', () => {
    expect(Object.keys(NODE_STYLES)).toHaveLength(21);
  });

  it.each(expectedKeys)('has entry for %s', (key) => {
    expect(NODE_STYLES).toHaveProperty(key);
  });

  it.each(expectedKeys)('%s has fill, font, shape properties', (key) => {
    const style = NODE_STYLES[key] as { fill: string; font: string; shape: string };
    expect(style).toBeDefined();
    expect(style).toHaveProperty('fill');
    expect(style).toHaveProperty('font');
    expect(style).toHaveProperty('shape');
    expect(typeof style.fill).toBe('string');
    expect(typeof style.font).toBe('string');
    expect(typeof style.shape).toBe('string');
  });

  it('SL has correct values', () => {
    expect(NODE_STYLES.SL).toEqual({ fill: '#4A90D9', font: '#ffffff', shape: 'box,rounded' });
  });

  it('ADR has correct values', () => {
    expect(NODE_STYLES.ADR).toEqual({ fill: '#7B4FBF', font: '#ffffff', shape: 'box' });
  });

  it('PRD has correct values', () => {
    expect(NODE_STYLES.PRD).toEqual({ fill: '#E67E22', font: '#222222', shape: 'box,rounded' });
  });

  it('REC has correct values (last in numeric order)', () => {
    expect(NODE_STYLES.REC).toEqual({ fill: '#95A5A6', font: '#222222', shape: 'box' });
  });

  it('CM has correct values', () => {
    expect(NODE_STYLES.CM).toEqual({ fill: '#16A085', font: '#ffffff', shape: 'box' });
  });
});

describe('DEFAULT_NODE_STYLE', () => {
  it('has fill, font, shape properties', () => {
    expect(DEFAULT_NODE_STYLE).toHaveProperty('fill');
    expect(DEFAULT_NODE_STYLE).toHaveProperty('font');
    expect(DEFAULT_NODE_STYLE).toHaveProperty('shape');
  });

  it('is the grey fallback', () => {
    expect(DEFAULT_NODE_STYLE).toEqual({ fill: '#95A5A6', font: '#222222', shape: 'box' });
  });
});

/* ------------------------------------------------------------------ */
/*  nodeAttrs                                                         */
/* ------------------------------------------------------------------ */

describe('nodeAttrs', () => {
  it('returns style from NODE_STYLES for known kindPrefix', () => {
    const node = makeNode({ id: 'SL-001', kindPrefix: 'SL' });
    const attrs = nodeAttrs(node, null);

    expect(attrs.fillcolor).toBe('#4A90D9');
    expect(attrs.fontcolor).toBe('#ffffff');
    expect(attrs.shape).toBe('box,rounded');
  });

  it('falls back to DEFAULT_NODE_STYLE for unknown kindPrefix', () => {
    const node = makeNode({ id: 'XYZ-001', kindPrefix: 'XYZ' });
    const attrs = nodeAttrs(node, null);

    expect(attrs.fillcolor).toBe('#95A5A6');
    expect(attrs.fontcolor).toBe('#222222');
    expect(attrs.shape).toBe('box');
  });

  it('sets penwidth=3 when node.id matches focusId', () => {
    const node = makeNode({ id: 'SL-001' });
    const attrs = nodeAttrs(node, 'SL-001');

    expect(attrs.penwidth).toBe(3);
  });

  it('sets penwidth=1 when node.id does not match focusId', () => {
    const node = makeNode({ id: 'SL-001' });
    const attrs = nodeAttrs(node, 'SL-999');

    expect(attrs.penwidth).toBe(1);
  });

  it('sets penwidth=1 when focusId is null', () => {
    const node = makeNode({ id: 'SL-001' });
    const attrs = nodeAttrs(node, null);

    expect(attrs.penwidth).toBe(1);
  });

  it('label is the node id', () => {
    const node = makeNode({ id: 'REQ-060' });
    const attrs = nodeAttrs(node, null);

    expect(attrs.label).toBe('REQ-060');
  });

  it('tooltip includes id, title, kindLabel, and status', () => {
    const node = makeNode({
      id: 'SL-001',
      title: 'Test Slice',
      kindLabel: 'Slice',
      status: 'active',
    });
    const attrs = nodeAttrs(node, null);

    expect(attrs.tooltip).toBe('SL-001: Test Slice · Slice · active');
  });

  it('tooltip falls back to kindPrefix when kindLabel is empty', () => {
    const node = makeNode({
      id: 'SL-001',
      title: 'Test Slice',
      kindLabel: '',
      kindPrefix: 'SL',
      status: 'draft',
    });
    const attrs = nodeAttrs(node, null);

    expect(attrs.tooltip).toBe('SL-001: Test Slice · SL · draft');
  });
});

/* ------------------------------------------------------------------ */
/*  edgeAttrs                                                         */
/* ------------------------------------------------------------------ */

describe('edgeAttrs', () => {
  it('returns correct color for "specs" label', () => {
    const edge = makeEdge({ label: 'specs' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#4A90D9');
    expect(attrs.fontcolor).toBe('#2563eb');
  });

  it('returns correct color for "supersedes" label', () => {
    const edge = makeEdge({ label: 'supersedes' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#E67E22');
    expect(attrs.fontcolor).toBe('#c2410c');
  });

  it('returns correct color for "governed_by" label', () => {
    const edge = makeEdge({ label: 'governed_by' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#7B4FBF');
    expect(attrs.fontcolor).toBe('#6d28d9');
  });

  it('returns correct color for "consumes" label', () => {
    const edge = makeEdge({ label: 'consumes' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#27AE60');
    expect(attrs.fontcolor).toBe('#166534');
  });

  it('returns correct color for "slices" label', () => {
    const edge = makeEdge({ label: 'slices' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#16A085');
    expect(attrs.fontcolor).toBe('#0f766e');
  });

  it('returns correct color for "reviews" label', () => {
    const edge = makeEdge({ label: 'reviews' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#64748b');
    expect(attrs.fontcolor).toBe('#475569');
  });

  it('returns correct color for "drift" label', () => {
    const edge = makeEdge({ label: 'drift' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#C0392B');
    expect(attrs.fontcolor).toBe('#991b1b');
  });

  it('returns correct color for "requirements" label', () => {
    const edge = makeEdge({ label: 'requirements' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#4A90D9');
    expect(attrs.fontcolor).toBe('#2563eb');
  });

  it('returns correct color for "revises" label', () => {
    const edge = makeEdge({ label: 'revises' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#E67E22');
    expect(attrs.fontcolor).toBe('#c2410c');
  });

  it('falls back to #888888/#555555 for unknown edge label', () => {
    const edge = makeEdge({ label: 'unknown_label' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.color).toBe('#888888');
    expect(attrs.fontcolor).toBe('#555555');
  });

  it('returns the edge label in the output', () => {
    const edge = makeEdge({ label: 'specs' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.label).toBe('specs');
  });

  it('returns the edge id as tooltip', () => {
    const edge = makeEdge({ id: 'e_someEdge' });
    const attrs = edgeAttrs(edge, 1);

    expect(attrs.tooltip).toBe('e_someEdge');
  });
});

/* ------------------------------------------------------------------ */
/*  graphToDot                                                        */
/* ------------------------------------------------------------------ */

describe('graphToDot', () => {
  it('returns a string starting with "digraph G {"', () => {
    const dot = graphToDot({ nodes: new Set<string>(), edges: [] }, null, 1);
    expect(dot.startsWith('digraph G {')).toBe(true);
  });

  it('returns a string ending with "}"', () => {
    const dot = graphToDot({ nodes: new Set<string>(), edges: [] }, null, 1);
    expect(dot.trimEnd().endsWith('}')).toBe(true);
  });

  it('contains rankdir=LR', () => {
    const dot = graphToDot({ nodes: new Set<string>(), edges: [] }, null, 1);
    expect(dot).toContain('rankdir=LR');
  });

  it('contains bgcolor="transparent"', () => {
    const dot = graphToDot({ nodes: new Set<string>(), edges: [] }, null, 1);
    expect(dot).toContain('bgcolor="transparent"');
  });

  it('empty neighbourhood produces only structural lines', () => {
    const dot = graphToDot({ nodes: new Set<string>(), edges: [] }, null, 1);
    const lines = dot.split('\n');

    // Should not contain any node or edge statements
    expect(lines.filter((l) => l.includes('->')).length).toBe(0);
    expect(lines.filter((l) => l.includes('[label=')).length).toBe(0);
  });

  it('single-node neighbourhood produces one node statement', () => {
    const node = makeNode({ id: 'SL-001', kindPrefix: 'SL' });
    state.graph.nodes.set('SL-001', node);

    const dot = graphToDot(
      { nodes: new Set(['SL-001']), edges: [] },
      null,
      1,
    );

    expect(dot).toContain('"SL-001" [');
    expect(dot).toContain('label="SL-001"');
    expect(dot).toContain('fillcolor="#4A90D9"');
    expect(dot).toContain('penwidth=1');
  });

  it('focus node gets penwidth=3', () => {
    const node = makeNode({ id: 'SL-001', kindPrefix: 'SL' });
    state.graph.nodes.set('SL-001', node);

    const dot = graphToDot(
      { nodes: new Set(['SL-001']), edges: [] },
      'SL-001',
      1,
    );

    expect(dot).toContain('penwidth=3');
    expect(dot).not.toContain('penwidth=1');
  });

  it('includes edge statements sorted by edge id', () => {
    const src = makeNode({ id: 'SL-001', kindPrefix: 'SL' });
    const tgt = makeNode({ id: 'SL-002', kindPrefix: 'SL' });
    state.graph.nodes.set('SL-001', src);
    state.graph.nodes.set('SL-002', tgt);

    const edgeB: Edge = makeEdge({ id: 'e_b', source: 'SL-001', target: 'SL-002', label: 'specs' });
    const edgeA: Edge = makeEdge({ id: 'e_a', source: 'SL-001', target: 'SL-002', label: 'supersedes' });

    const dot = graphToDot(
      { nodes: new Set(['SL-001', 'SL-002']), edges: [edgeB, edgeA] },
      null,
      1,
    );

    // e_a should appear before e_b in the output (sorted)
    const idxA = dot.indexOf('e_a');
    const idxB = dot.indexOf('e_b');
    expect(idxA).toBeLessThan(idxB);
    expect(idxA).toBeGreaterThan(-1);
    expect(idxB).toBeGreaterThan(-1);
  });

  it('skips nodes not found in state.graph.nodes', () => {
    const dot = graphToDot(
      { nodes: new Set(['NONEXISTENT']), edges: [] },
      null,
      1,
    );

    expect(dot).not.toContain('NONEXISTENT');
  });

  it('sorts node ids in output', () => {
    const nodeB = makeNode({ id: 'SL-002', kindPrefix: 'SL' });
    const nodeA = makeNode({ id: 'ADR-001', kindPrefix: 'ADR' });
    state.graph.nodes.set('SL-002', nodeB);
    state.graph.nodes.set('ADR-001', nodeA);

    const dot = graphToDot(
      { nodes: new Set(['SL-002', 'ADR-001']), edges: [] },
      null,
      1,
    );

    // ADR-001 sorts before SL-002
    const idxAdr = dot.indexOf('ADR-001');
    const idxSl = dot.indexOf('SL-002');
    expect(idxAdr).toBeLessThan(idxSl);
    expect(idxAdr).toBeGreaterThan(-1);
    expect(idxSl).toBeGreaterThan(-1);
  });

  it('escapes node id with dots/colons via dotQuote', () => {
    const node = makeNode({ id: 'mem_019ed32d16b178629d58a6e1e1a0a797', kindPrefix: 'SL' });
    state.graph.nodes.set('mem_019ed32d16b178629d58a6e1e1a0a797', node);

    const dot = graphToDot(
      { nodes: new Set(['mem_019ed32d16b178629d58a6e1e1a0a797']), edges: [] },
      null,
      1,
    );

    expect(dot).toContain('mem_019ed32d16b178629d58a6e1e1a0a797');
  });
});

/* ------------------------------------------------------------------ */
/*  cmGraphToDot                                                      */
/* ------------------------------------------------------------------ */

describe('cmGraphToDot', () => {
  it('returns a string starting with "digraph concept_map {"', () => {
    const dot = cmGraphToDot({ nodes: [], edges: [] }, null);
    expect(dot.startsWith('digraph concept_map {')).toBe(true);
  });

  it('returns a string ending with "}"', () => {
    const dot = cmGraphToDot({ nodes: [], edges: [] }, null);
    expect(dot.trimEnd().endsWith('}')).toBe(true);
  });

  it('contains rankdir=LR', () => {
    const dot = cmGraphToDot({ nodes: [], edges: [] }, null);
    expect(dot).toContain('rankdir=LR');
  });

  it('contains bgcolor="transparent"', () => {
    const dot = cmGraphToDot({ nodes: [], edges: [] }, null);
    expect(dot).toContain('bgcolor="transparent"');
  });

  it('contains default node and edge attributes', () => {
    const dot = cmGraphToDot({ nodes: [], edges: [] }, null);
    expect(dot).toContain('node [shape=record');
    expect(dot).toContain('edge [color="#4A90D9"');
  });

  it('empty input produces only structural lines', () => {
    const dot = cmGraphToDot({ nodes: [], edges: [] }, null);
    const lines = dot.split('\n');

    expect(lines.filter((l) => l.includes('->')).length).toBe(0);
  });

  it('node with label appears in output', () => {
    const dot = cmGraphToDot(
      { nodes: [{ key: 'myKey', label: 'My Label' }], edges: [] },
      null,
    );

    expect(dot).toContain('"myKey"');
    expect(dot).toContain('My Label');
  });

  it('focusKey adds penwidth=3.0 to matching node', () => {
    const dot = cmGraphToDot(
      { nodes: [{ key: 'myKey', label: 'My Label' }], edges: [] },
      'myKey',
    );

    expect(dot).toContain('penwidth=3.0');
  });

  it('non-matching focusKey does not add penwidth', () => {
    const dot = cmGraphToDot(
      { nodes: [{ key: 'myKey', label: 'My Label' }], edges: [] },
      'otherKey',
    );

    expect(dot).not.toContain('penwidth=3.0');
  });

  it('no focusKey does not add penwidth', () => {
    const dot = cmGraphToDot(
      { nodes: [{ key: 'myKey', label: 'My Label' }], edges: [] },
      null,
    );

    expect(dot).not.toContain('penwidth=');
  });

  it('sorts nodes by key', () => {
    const dot = cmGraphToDot(
      {
        nodes: [
          { key: 'zebra', label: 'Z' },
          { key: 'alpha', label: 'A' },
        ],
        edges: [],
      },
      null,
    );

    const idxAlpha = dot.indexOf('alpha');
    const idxZebra = dot.indexOf('zebra');
    expect(idxAlpha).toBeLessThan(idxZebra);
    expect(idxAlpha).toBeGreaterThan(-1);
    expect(idxZebra).toBeGreaterThan(-1);
  });

  it('edge with rel appears in output', () => {
    const dot = cmGraphToDot(
      {
        nodes: [
          { key: 'a', label: 'A' },
          { key: 'b', label: 'B' },
        ],
        edges: [{ from_key: 'a', from_label: 'A', rel: 'relatesTo', to_key: 'b', to_label: 'B' }],
      },
      null,
    );

    expect(dot).toContain('->');
    expect(dot).toContain('relatesTo');
  });

  it('escapes special characters in keys and labels via escapeStringContent', () => {
    const dot = cmGraphToDot(
      {
        nodes: [{ key: 'key"with>quotes', label: 'label"with>quotes' }],
        edges: [],
      },
      null,
    );

    // The escaped form should appear in the output
    expect(dot).toContain('key\\"with\\>quotes');
    expect(dot).toContain('label\\"with\\>quotes');
  });
});
