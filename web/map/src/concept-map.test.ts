/**
 * @vitest-environment jsdom
 *
 * Unit tests for the pure cell-selection bit extracted in SL-110 PHASE-04:
 * cmSelectedFieldFromCell builds a label-carrying CmSelectedField from a
 * clicked CmEdge + which cell was clicked.
 *
 * RED-first: cmSelectedFieldFromCell does not exist yet.
 */

import { describe, it, expect } from 'vitest';
import { cmSelectedFieldFromCell } from './concept-map';
import type { CmEdge } from './types';

describe('cmSelectedFieldFromCell', () => {
  const edge: CmEdge = {
    from_key: 'user-story',
    from_label: 'User Story',
    rel: 'refines',
    to_key: 'epic',
    to_label: 'Epic',
  };

  it("'from' cell selects the source node, carrying its key + label", () => {
    expect(cmSelectedFieldFromCell(edge, 'from')).toEqual({
      kind: 'node',
      key: 'user-story',
      label: 'User Story',
    });
  });

  it("'to' cell selects the target node, carrying its key + label", () => {
    expect(cmSelectedFieldFromCell(edge, 'to')).toEqual({
      kind: 'node',
      key: 'epic',
      label: 'Epic',
    });
  });

  it("'rel' cell selects the relation, carrying both endpoint labels + rel", () => {
    expect(cmSelectedFieldFromCell(edge, 'rel')).toEqual({
      kind: 'rel',
      from_label: 'User Story',
      rel: 'refines',
      to_label: 'Epic',
    });
  });

  it('captures the CLICKED label, not a derived key (same key / different label)', () => {
    // `User-Story` and `User Story` derive the same key but are distinct labels;
    // the selection must carry the label exactly as it appears on the clicked edge.
    const ambiguous: CmEdge = {
      from_key: 'user-story',
      from_label: 'User-Story',
      rel: 'refines',
      to_key: 'epic',
      to_label: 'Epic',
    };
    const selected = cmSelectedFieldFromCell(ambiguous, 'from');
    expect(selected).toEqual({ kind: 'node', key: 'user-story', label: 'User-Story' });
    // Same key as the canonical edge, but the label must differ.
    expect(selected.kind === 'node' && selected.label).toBe('User-Story');
  });
});
