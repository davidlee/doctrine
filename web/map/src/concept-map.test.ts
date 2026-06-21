/**
 * Unit tests for the pure op-selection seams of the concept-map edit surface
 * (SL-110 PHASE-07, item 4 Revision 2):
 *   - cmEditOp(cell, editAll) — the 4-cell × scope matrix → backend op.
 *   - cmCellEndpoint(cell)    — frontend node cell → backend endpoint literal.
 */

import { describe, it, expect } from 'vitest';
import { cmEditOp, cmCellEndpoint } from './concept-map';

describe('cmEditOp', () => {
  it('node cell, edit-all OFF → rename_node_occurrence (single endpoint)', () => {
    expect(cmEditOp('from', false)).toBe('rename_node_occurrence');
    expect(cmEditOp('to', false)).toBe('rename_node_occurrence');
  });

  it('node cell, edit-all ON → rename_node (label-global)', () => {
    expect(cmEditOp('from', true)).toBe('rename_node');
    expect(cmEditOp('to', true)).toBe('rename_node');
  });

  it('relation cell, edit-all OFF → relabel_edge (single edge)', () => {
    expect(cmEditOp('rel', false)).toBe('relabel_edge');
  });

  it('relation cell, edit-all ON → relabel_rel_all (every edge sharing the rel)', () => {
    expect(cmEditOp('rel', true)).toBe('relabel_rel_all');
  });
});

describe('cmCellEndpoint', () => {
  it("maps 'from' → 'source'", () => {
    expect(cmCellEndpoint('from')).toBe('source');
  });

  it("maps 'to' → 'target'", () => {
    expect(cmCellEndpoint('to')).toBe('target');
  });
});
