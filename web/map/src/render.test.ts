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

import { describe, it, expect } from 'vitest';
import { hoverDetailHtml, hoverPane } from './render';

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
