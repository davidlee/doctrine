/**
 * @vitest-environment jsdom
 *
 * SL-110 PHASE-03 — hoverDetailHtml content builder.
 *
 * The on-graph actionability tooltip and the side detail pane share ONE
 * content builder. This builder escapes EVERY interpolated field (id, title,
 * kindLabel, status) — closing the latent raw-HTML-injection gap where the
 * pane previously escaped only `title`.
 *
 * RED until `hoverDetailHtml` is exported from render.ts and `hoverPane`
 * delegates to it.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { hoverDetailHtml, hoverPane, relationshipTable } from './render';
import { parseHash } from './router';
import { state } from './state';
import type { CatalogNode, Edge, Graph } from './types';

const FIELDS = ['id', 'kindLabel', 'status', 'title'] as const;

describe('hoverDetailHtml', () => {
  it('renders the id, title, kindLabel and status', () => {
    const html = hoverDetailHtml({ id: 'SL-110', title: 'Hover tooltip', kindLabel: 'slice', status: 'open' });
    expect(html).toContain('SL-110');
    expect(html).toContain('Hover tooltip');
    expect(html).toContain('slice');
    expect(html).toContain('open');
    expect(html).toContain('hover-detail-content');
    expect(html).toContain('hover-detail-title');
    expect(html).toContain('hover-detail-meta');
  });

  for (const field of FIELDS) {
    it(`escapes unsafe characters in ${field}`, () => {
      const unsafe = '<img src=x onerror=alert(1)>&"';
      const node = { id: 'SL-1', title: 't', kindLabel: 'k', status: 's' };
      node[field] = unsafe;
      const html = hoverDetailHtml(node);
      // The raw, unescaped injection payload must NOT appear verbatim.
      expect(html).not.toContain('<img');
      expect(html).not.toContain('onerror=alert(1)>');
      // The escaped forms must be present instead.
      expect(html).toContain('&lt;img');
      expect(html).toContain('&amp;');
      expect(html).toContain('&quot;');
    });
  }
});

describe('hoverPane delegation', () => {
  it('renders the escaped hoverDetailHtml content for a non-null node', () => {
    const container = document.createElement('div');
    const node = { id: 'SL-110', title: '<b>x</b>', kindLabel: 'slice', status: 'open' };
    hoverPane({ container, node });
    expect(container.innerHTML).toBe(hoverDetailHtml(node));
    // The injected markup is escaped, not live DOM.
    expect(container.querySelector('b')).toBeNull();
  });

  it('renders the placeholder for a null node', () => {
    const container = document.createElement('div');
    hoverPane({ container, node: null });
    expect(container.innerHTML).toContain('placeholder');
  });
});

/*
 * RV-098 F-6 — semantic relationship-table links must round-trip through
 * parseHash. The anchors are built from buildHash, which already returns a
 * '#'-prefixed string; an extra '#' prefix yields '##/focus/…', which parseHash
 * cannot match, clearing focus and emptying the table (pre-existing SL-091).
 */
describe('relationship table links round-trip through parseHash', () => {
  beforeEach(() => {
    state.depth = 1;
    state.cmFocusNode = null;
  });

  function catalogNode(id: string): CatalogNode {
    return {
      id,
      title: `${id} title`,
      status: 'open',
      kindPrefix: 'SL',
      kindLabel: 'slice',
      raw: { title: '', status: '', kind_label: '' },
    };
  }

  const edge: Edge = {
    id: 'e_supersedes',
    source: 'SL-003',
    label: 'Supersedes',
    target: 'SL-002',
    raw: { source: 'SL-003', label: { Validated: 'Supersedes' }, target: { Resolved: 'SL-002' } },
  };

  function graph(): Graph {
    return {
      nodes: new Map([['SL-003', catalogNode('SL-003')], ['SL-002', catalogNode('SL-002')]]),
      edges: [edge],
      incoming: new Map([['SL-002', [edge]]]),
      outgoing: new Map([['SL-003', [edge]]]),
      edgeById: new Map([['e_supersedes', edge]]),
    };
  }

  function resolve(href: string): ReturnType<typeof parseHash> {
    window.location.hash = href;
    return parseHash();
  }

  it('source/target id-links focus the node; the relation link opens edge detail — no double hash', () => {
    const tbody = document.createElement('tbody');
    relationshipTable({ container: tbody, edges: [edge], graph: graph(), focusId: 'SL-003', depth: 1, viewMode: 'semantic' });

    const hrefs = Array.from(tbody.querySelectorAll('a')).map((a) => a.getAttribute('href') ?? '');
    expect(hrefs).toHaveLength(3);
    for (const href of hrefs) {
      expect(href.startsWith('##')).toBe(false);
    }

    const [srcHref, lblHref, tgtHref] = hrefs;
    const src = resolve(srcHref ?? '');
    expect(src.view).toBe('focus');
    expect(src.id).toBe('SL-003');

    const lbl = resolve(lblHref ?? '');
    expect(lbl.view).toBe('edge');
    expect(lbl.id).toBe('e_supersedes');

    const tgt = resolve(tgtHref ?? '');
    expect(tgt.view).toBe('focus');
    expect(tgt.id).toBe('SL-002');
  });
});
