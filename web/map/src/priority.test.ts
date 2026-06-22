/**
 * @vitest-environment jsdom
 *
 * IMP-092 — viewport helpers for the actionability (priority) graph.
 *  - layoutBBox: fit-to-content bounding box over laid-out nodes (replaces the
 *    hardcoded 960x600 viewBox so the whole DAG fits on load).
 *  - zoomToNode: the d3-zoom transform that centres + scales one node within
 *    the fit box (the zoom-to-selected target).
 *
 * RED until both helpers are exported from priority.ts.
 */

import { describe, it, expect } from 'vitest';
import { layoutBBox, zoomToNode } from './priority';
import type { LayoutNode } from './priority';

function node(id: string, x: number, y: number): LayoutNode {
  return {
    id,
    title: id,
    kind: 'slice',
    status: 'open',
    actionability: 'actionable',
    score: 0,
    rank: 0,
    blockers: [],
    x,
    y,
  };
}

describe('layoutBBox', () => {
  it('falls back to the default frame when there are no nodes', () => {
    expect(layoutBBox([])).toEqual({ minX: 0, minY: 0, width: 960, height: 600 });
  });

  it('frames a single node by its half-extents plus padding', () => {
    // id 'SL-001' (len 6) → width max(72, 6*7+16=58)=72 → half-width 36; half-height 14; pad 40.
    expect(layoutBBox([node('SL-001', 100, 100)])).toEqual({
      minX: 24,
      minY: 46,
      width: 152,
      height: 108,
    });
  });

  it('spans the union of all nodes', () => {
    const b = layoutBBox([node('A', 0, 0), node('B', 400, 200)]);
    // node half-width = max(72, len*7+16)/2 = 36 for single-char ids; pad 40.
    expect(b.minX).toBe(0 - 36 - 40);
    expect(b.minY).toBe(0 - 14 - 40);
    expect(b.width).toBe(400 + 36 + 36 + 80);
    expect(b.height).toBe(200 + 14 + 14 + 80);
  });

  it('ignores nodes without computed coordinates', () => {
    const orphan: LayoutNode = {
      id: 'Z',
      title: 'Z',
      kind: 'slice',
      status: 'open',
      actionability: 'actionable',
      score: 0,
      rank: 0,
      blockers: [],
    };
    expect(layoutBBox([node('SL-001', 100, 100), orphan])).toEqual(
      layoutBBox([node('SL-001', 100, 100)]),
    );
  });
});

describe('zoomToNode', () => {
  it('centres the node at the box centre at the given scale', () => {
    const bbox = { minX: 0, minY: 0, width: 960, height: 600 };
    // centre (480,300); k=5 → x = 480 - 5*100, y = 300 - 5*200.
    expect(zoomToNode(node('X', 100, 200), bbox, 5)).toEqual({ x: -20, y: -700, k: 5 });
  });

  it('honours a non-zero box origin', () => {
    const bbox = { minX: 100, minY: 100, width: 200, height: 200 };
    // centre (200,200); k=2 → x = 200 - 2*50, y = 200 - 2*50.
    expect(zoomToNode(node('X', 50, 50), bbox, 2)).toEqual({ x: 100, y: 100, k: 2 });
  });
});
