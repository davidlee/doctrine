// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { extractNodeId } from './svg';

const SVGNS = 'http://www.w3.org/2000/svg';

function nodeGroup(parts: { title?: string; text?: string }): SVGGElement {
  const g = document.createElementNS(SVGNS, 'g');
  g.setAttribute('class', 'node');
  if (parts.title !== undefined) {
    const t = document.createElementNS(SVGNS, 'title');
    t.textContent = parts.title;
    g.appendChild(t);
  }
  if (parts.text !== undefined) {
    const tx = document.createElementNS(SVGNS, 'text');
    tx.textContent = parts.text;
    g.appendChild(tx);
  }
  return g;
}

describe('extractNodeId', () => {
  it('reads the id from <title> (DOT node name), not the visible <text> label', () => {
    // Graphviz puts the node name in <title> and the label in <text>. For a
    // memory node these differ: title is the mem_ uid, text is the human title.
    const g = nodeGroup({
      title: 'mem_019e95a992607db3a9805d492e69ff97',
      text: 'Entity-engine identity + claim seam',
    });
    expect(extractNodeId(g)).toBe('mem_019e95a992607db3a9805d492e69ff97');
  });

  it('returns the id for numbered entities where title equals label', () => {
    const g = nodeGroup({ title: 'SL-201', text: 'SL-201' });
    expect(extractNodeId(g)).toBe('SL-201');
  });

  it('returns null when there is no <title>', () => {
    const g = nodeGroup({ text: 'orphan' });
    expect(extractNodeId(g)).toBeNull();
  });
});
