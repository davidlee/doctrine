/**
 * @vitest-environment jsdom
 *
 * Behaviour-contract tests for router.ts — captures the EXACT observable
 * behaviour of router.js as a contract that the TypeScript rewrite must satisfy.
 *
 * These tests initially FAIL (RED) because router.ts doesn't exist yet.
 * The satisfier creates router.ts to make them pass.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { Route } from './types';

// Mock the state module — router.ts will import from here.
// The real state module path is decided by the satisfier; update the mock path
// to match if the satisfier chooses a different location.
vi.mock('./state', () => ({
  state: {
    depth: 1,
    cmFocusNode: null as { key: string; label: string } | null,
  },
}));

import { state } from './state';
import { parseHash, buildHash, setFocus, setEdge, clampDepth } from './router';

/* ------------------------------------------------------------------ */
/*  clampDepth                                                        */
/* ------------------------------------------------------------------ */

describe('clampDepth', () => {
  it('returns the value when it is in [0,3]', () => {
    expect(clampDepth(0)).toBe(0);
    expect(clampDepth(1)).toBe(1);
    expect(clampDepth(2)).toBe(2);
    expect(clampDepth(3)).toBe(3);
  });

  it('clamps negative values to 0', () => {
    expect(clampDepth(-1)).toBe(0);
    expect(clampDepth(-100)).toBe(0);
  });

  it('clamps values above 3 to 3', () => {
    expect(clampDepth(4)).toBe(3);
    expect(clampDepth(100)).toBe(3);
  });

  it('returns 0 for NaN', () => {
    expect(clampDepth(NaN)).toBe(0);
  });
});

/* ------------------------------------------------------------------ */
/*  parseHash                                                         */
/* ------------------------------------------------------------------ */

describe('parseHash', () => {
  beforeEach(() => {
    state.depth = 1;
    state.cmFocusNode = null;
  });

  describe('empty or missing hash', () => {
    it('returns focus view with null id and state defaults when hash is empty', () => {
      window.location.hash = '';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBeNull();
      expect(route.depth).toBe(state.depth);
      expect(route.cmFocus).toBeNull();
    });

    it('returns focus view with null id when hash is just "#"', () => {
      window.location.hash = '#';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBeNull();
    });
  });

  describe('focus view — prefix-numbered entities', () => {
    it('parses a simple focus hash with a prefix-numbered entity id', () => {
      window.location.hash = '#/focus/SL-001';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBe('SL-001');
      expect(route.depth).toBe(state.depth);
      expect(route.cmFocus).toBeNull();
    });

    it('parses ADR prefix entities', () => {
      window.location.hash = '#/focus/ADR-002';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBe('ADR-002');
    });

    it('parses REQ prefix entities', () => {
      window.location.hash = '#/focus/REQ-060';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBe('REQ-060');
    });

    it('extracts depth from query string', () => {
      window.location.hash = '#/focus/SL-001?depth=2';
      const route: Route = parseHash();

      expect(route.id).toBe('SL-001');
      expect(route.depth).toBe(2);
    });

    it('clamps depth to 0 when query param is negative', () => {
      window.location.hash = '#/focus/SL-001?depth=-5';
      const route: Route = parseHash();

      expect(route.depth).toBe(0);
    });

    it('clamps depth to 3 when query param exceeds 3', () => {
      window.location.hash = '#/focus/SL-001?depth=10';
      const route: Route = parseHash();

      expect(route.depth).toBe(3);
    });

    it('clamps non-numeric depth param to 0', () => {
      window.location.hash = '#/focus/SL-001?depth=abc';
      const route: Route = parseHash();

      expect(route.depth).toBe(0);
    });

    it('extracts cmFocus from query string', () => {
      window.location.hash = '#/focus/SL-001?cmFocus=someKey';
      const route: Route = parseHash();

      expect(route.cmFocus).toBe('someKey');
    });

    it('URL-decodes the cmFocus query param', () => {
      window.location.hash = '#/focus/SL-001?cmFocus=hello%20world';
      const route: Route = parseHash();

      expect(route.cmFocus).toBe('hello world');
    });

    it('extracts both depth and cmFocus from query string', () => {
      window.location.hash = '#/focus/SL-001?depth=2&cmFocus=myKey';
      const route: Route = parseHash();

      expect(route.id).toBe('SL-001');
      expect(route.depth).toBe(2);
      expect(route.cmFocus).toBe('myKey');
    });

    it('extracts both cmFocus and depth when params appear in reverse order', () => {
      window.location.hash = '#/focus/SL-001?cmFocus=myKey&depth=2';
      const route: Route = parseHash();

      expect(route.depth).toBe(2);
      expect(route.cmFocus).toBe('myKey');
    });
  });

  describe('focus view — memory entities', () => {
    it('parses a memory entity hash with a 32-char hex id', () => {
      window.location.hash = '#/focus/mem_019ed32d16b178629d58a6e1e1a0a797';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBe('mem_019ed32d16b178629d58a6e1e1a0a797');
      expect(route.depth).toBe(state.depth);
      expect(route.cmFocus).toBeNull();
    });

    it('parses a memory entity with uppercase hex', () => {
      window.location.hash = '#/focus/mem_ABCDEF0123456789ABCDEF0123456789';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBe('mem_ABCDEF0123456789ABCDEF0123456789');
    });

    it('parses a memory entity with depth param', () => {
      window.location.hash = '#/focus/mem_019ed32d16b178629d58a6e1e1a0a797?depth=3';
      const route: Route = parseHash();

      expect(route.id).toBe('mem_019ed32d16b178629d58a6e1e1a0a797');
      expect(route.depth).toBe(3);
    });

    it('parses a memory entity with cmFocus param', () => {
      window.location.hash = '#/focus/mem_019ed32d16b178629d58a6e1e1a0a797?cmFocus=someKey';
      const route: Route = parseHash();

      expect(route.cmFocus).toBe('someKey');
    });
  });

  describe('edge view', () => {
    it('parses an edge hash', () => {
      window.location.hash = '#/edge/e_someEdgeId';
      const route: Route = parseHash();

      expect(route.view).toBe('edge');
      expect(route.id).toBe('e_someEdgeId');
      expect(route.depth).toBe(state.depth);
      expect(route.cmFocus).toBeNull();
    });

    it('parses an edge hash with depth param', () => {
      window.location.hash = '#/edge/e_someEdgeId?depth=1';
      const route: Route = parseHash();

      expect(route.view).toBe('edge');
      expect(route.id).toBe('e_someEdgeId');
      expect(route.depth).toBe(1);
    });

    it('always returns cmFocus null for edge views, even when param is present', () => {
      window.location.hash = '#/edge/e_someEdgeId?cmFocus=someKey';
      const route: Route = parseHash();

      expect(route.view).toBe('edge');
      expect(route.cmFocus).toBeNull();
    });

    it('parses edge id with hyphens and underscores', () => {
      window.location.hash = '#/edge/e_edge-with_hyphens_and_underscores';
      const route: Route = parseHash();

      expect(route.id).toBe('e_edge-with_hyphens_and_underscores');
    });
  });

  describe('malformed or unrecognized hashes', () => {
    it('falls back to focus view with null id for unrecognized patterns', () => {
      window.location.hash = '#/garbage';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBeNull();
      expect(route.depth).toBe(state.depth);
      expect(route.cmFocus).toBeNull();
    });

    it('falls back for focus path with invalid entity id', () => {
      window.location.hash = '#/focus/invalidId';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBeNull();
    });

    it('falls back for edge path without e_ prefix', () => {
      window.location.hash = '#/edge/invalidEdge';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBeNull();
    });

    it('falls back when hash has no leading slash', () => {
      window.location.hash = '#focus/SL-001';
      const route: Route = parseHash();

      expect(route.view).toBe('focus');
      expect(route.id).toBeNull();
    });
  });
});

/* ------------------------------------------------------------------ */
/*  buildHash                                                         */
/* ------------------------------------------------------------------ */

describe('buildHash', () => {
  beforeEach(() => {
    state.depth = 1;
    state.cmFocusNode = null;
  });

  describe('focus view', () => {
    it('builds a basic focus hash when depth matches state', () => {
      expect(buildHash('focus', 'SL-001', 1)).toBe('#/focus/SL-001');
    });

    it('adds depth param when depth differs from state.depth', () => {
      expect(buildHash('focus', 'SL-001', 2)).toBe('#/focus/SL-001?depth=2');
    });

    it('adds cmFocus param when state.cmFocusNode.key is set', () => {
      state.cmFocusNode = { key: 'myKey', label: 'My Label' };
      expect(buildHash('focus', 'SL-001', 1)).toBe('#/focus/SL-001?cmFocus=myKey');
    });

    it('URL-encodes the cmFocus key', () => {
      state.cmFocusNode = { key: 'key with spaces', label: 'Label' };
      expect(buildHash('focus', 'SL-001', 1)).toBe('#/focus/SL-001?cmFocus=key%20with%20spaces');
    });

    it('combines depth and cmFocus params with & when both apply', () => {
      state.cmFocusNode = { key: 'myKey', label: 'My Label' };
      expect(buildHash('focus', 'SL-001', 2)).toBe('#/focus/SL-001?depth=2&cmFocus=myKey');
    });

    it('omits depth param when depth equals state.depth even if cmFocus is present', () => {
      state.cmFocusNode = { key: 'myKey', label: 'My Label' };
      expect(buildHash('focus', 'SL-001', 1)).toBe('#/focus/SL-001?cmFocus=myKey');
    });

    it('omits cmFocus when state.cmFocusNode is null', () => {
      state.cmFocusNode = null;
      expect(buildHash('focus', 'SL-001', 2)).toBe('#/focus/SL-001?depth=2');
    });

    it('omits cmFocus when state.cmFocusNode.key is empty string', () => {
      state.cmFocusNode = { key: '', label: '' };
      expect(buildHash('focus', 'SL-001', 1)).toBe('#/focus/SL-001');
    });
  });

  describe('edge view', () => {
    it('builds a basic edge hash', () => {
      expect(buildHash('edge', 'e_someEdgeId', 1)).toBe('#/edge/e_someEdgeId');
    });

    it('adds depth param for edge hash when depth differs', () => {
      expect(buildHash('edge', 'e_someEdgeId', 3)).toBe('#/edge/e_someEdgeId?depth=3');
    });

    it('includes cmFocus for edge hash when state.cmFocusNode is set', () => {
      state.cmFocusNode = { key: 'myKey', label: 'My Label' };
      expect(buildHash('edge', 'e_someEdgeId', 1)).toBe('#/edge/e_someEdgeId?cmFocus=myKey');
    });
  });
});

/* ------------------------------------------------------------------ */
/*  setFocus                                                          */
/* ------------------------------------------------------------------ */

describe('setFocus', () => {
  beforeEach(() => {
    state.depth = 1;
    state.cmFocusNode = null;
    window.location.hash = '';
  });

  it('sets window.location.hash to a focus hash', () => {
    setFocus('SL-001', 1);
    expect(window.location.hash).toBe('#/focus/SL-001');
  });

  it('defaults depth to state.depth when second argument is omitted', () => {
    state.depth = 2;
    setFocus('SL-001');
    // state.depth=2, depth arg defaults to 2, buildHash sees depth===state.depth → no param
    expect(window.location.hash).toBe('#/focus/SL-001');
  });

  it('includes depth param when explicit depth differs from state.depth', () => {
    setFocus('SL-001', 3);
    expect(window.location.hash).toBe('#/focus/SL-001?depth=3');
  });

  it('includes cmFocus param when state.cmFocusNode is set', () => {
    state.cmFocusNode = { key: 'myKey', label: 'My Label' };
    setFocus('SL-001', 1);
    expect(window.location.hash).toBe('#/focus/SL-001?cmFocus=myKey');
  });
});

/* ------------------------------------------------------------------ */
/*  setEdge                                                           */
/* ------------------------------------------------------------------ */

describe('setEdge', () => {
  beforeEach(() => {
    state.depth = 1;
    state.cmFocusNode = null;
    window.location.hash = '';
  });

  it('sets window.location.hash to an edge hash', () => {
    setEdge('e_someEdgeId', 1);
    expect(window.location.hash).toBe('#/edge/e_someEdgeId');
  });

  it('defaults depth to state.depth when second argument is omitted', () => {
    state.depth = 3;
    setEdge('e_someEdgeId');
    // state.depth=3, depth arg defaults to 3, buildHash sees depth===state.depth → no param
    expect(window.location.hash).toBe('#/edge/e_someEdgeId');
  });

  it('includes depth param when explicit depth differs from state.depth', () => {
    setEdge('e_someEdgeId', 2);
    expect(window.location.hash).toBe('#/edge/e_someEdgeId?depth=2');
  });

  it('includes cmFocus param when state.cmFocusNode is set', () => {
    state.cmFocusNode = { key: 'myKey', label: 'My Label' };
    setEdge('e_someEdgeId', 1);
    expect(window.location.hash).toBe('#/edge/e_someEdgeId?cmFocus=myKey');
  });
});
