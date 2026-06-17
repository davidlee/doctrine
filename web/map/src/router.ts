import type { Route } from './types';
import { state } from './state';

/* ------------------------------------------------------------------ */
/*  Module-private helpers                                            */
/* ------------------------------------------------------------------ */

function parseQueryString(qs: string): Record<string, string> {
  const result: Record<string, string> = {};
  if (qs.length === 0) return result;
  for (const pair of qs.split('&')) {
    const eqIdx = pair.indexOf('=');
    if (eqIdx === -1) continue;
    const key = decodeURIComponent(pair.slice(0, eqIdx));
    const value = decodeURIComponent(pair.slice(eqIdx + 1));
    result[key] = value;
  }
  return result;
}

function clampDepth(d: number): number {
  if (Number.isNaN(d) || d < 0) return 0;
  if (d > 3) return 3;
  return Math.floor(d);
}

/* ------------------------------------------------------------------ */
/*  Exports                                                           */
/* ------------------------------------------------------------------ */

export { clampDepth };

export function parseHash(depth?: number): Route {
  const d = depth ?? state.depth;
  const h = window.location.hash.slice(1);
  if (h.length === 0) return { view: 'focus', id: null, depth: d, cmFocus: null };

  // Numbered entities: PREFIX-NNN (e.g. SL-001, ADR-002)
  const focusNumbered = /^\/focus\/([A-Z]+-\d+)(?:\?(.+))?$/.exec(h);
  if (focusNumbered !== null) {
    const [, id, rawQs] = focusNumbered;
    if (id === undefined) return { view: 'focus', id: null, depth: d, cmFocus: null };
    const params = parseQueryString(rawQs ?? '');
    return {
      view: 'focus',
      id,
      depth: params.depth !== undefined ? clampDepth(parseInt(params.depth, 10)) : d,
      cmFocus: params.cmFocus ?? null,
    };
  }

  // Memory entities: mem_<32-hex> (e.g. mem_019ed32d16b178629d58a6e1e1a0a797)
  const focusMem = /^\/focus\/(mem_[0-9a-fA-F]{32})(?:\?(.+))?$/.exec(h);
  if (focusMem !== null) {
    const [, id, rawQs] = focusMem;
    if (id === undefined) return { view: 'focus', id: null, depth: d, cmFocus: null };
    const params = parseQueryString(rawQs ?? '');
    return {
      view: 'focus',
      id,
      depth: params.depth !== undefined ? clampDepth(parseInt(params.depth, 10)) : d,
      cmFocus: params.cmFocus ?? null,
    };
  }

  // Edge entities: e_<alphanumeric + hyphens/underscores>
  const edgeMatch = /^\/edge\/(e_[A-Za-z0-9_-]+)(?:\?(.+))?$/.exec(h);
  if (edgeMatch !== null) {
    const [, id, rawQs] = edgeMatch;
    if (id === undefined) return { view: 'focus', id: null, depth: d, cmFocus: null };
    const params = parseQueryString(rawQs ?? '');
    return {
      view: 'edge',
      id,
      depth: params.depth !== undefined ? clampDepth(parseInt(params.depth, 10)) : d,
      cmFocus: null,
    };
  }

  return { view: 'focus', id: null, depth: d, cmFocus: null };
}

export function buildHash(
  view: string,
  id: string,
  depth: number,
  cmFocusKey?: string | null,
): string {
  let base = `#/${view}/${id}`;
  const params: string[] = [];

  if (depth !== state.depth) {
    params.push(`depth=${String(depth)}`);
  }

  const cm = cmFocusKey ?? state.cmFocusNode?.key;
  if (typeof cm === 'string' && cm.length > 0) {
    params.push(`cmFocus=${encodeURIComponent(cm)}`);
  }

  if (params.length > 0) {
    base += `?${params.join('&')}`;
  }
  return base;
}

export function setFocus(id: string, depth?: number): void {
  const d = depth ?? state.depth;
  window.location.hash = buildHash('focus', id, d);
}

export function setEdge(edgeId: string, depth?: number): void {
  const d = depth ?? state.depth;
  window.location.hash = buildHash('edge', edgeId, d);
}
