import type { RawGraph, ActionabilityView, ConceptMap } from './types';

// ---------------------------------------------------------------------------
// ApiError
// ---------------------------------------------------------------------------

export class ApiError extends Error {
  status: number;
  body: string;
  endpoint: string;

  constructor(message: string, status: number, body: string, endpoint: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.body = body;
    this.endpoint = endpoint;
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function extractErrorMessage(body: unknown, fallback: string): string {
  if (
    typeof body === 'object' &&
    body !== null &&
    'message' in body
  ) {
    const msg = (body as Record<string, unknown>).message;
    if (typeof msg === 'string') return msg;
  }
  return fallback;
}

function asString(value: unknown, fallback: string): string {
  if (typeof value === 'string') return value;
  if (typeof value === 'number') return String(value);
  return fallback;
}

function asCmNodeArray(value: unknown): ConceptMap['nodes'] {
  return Array.isArray(value) ? (value as ConceptMap['nodes']) : [];
}

function asCmEdgeArray(value: unknown): ConceptMap['edges'] {
  return Array.isArray(value) ? (value as ConceptMap['edges']) : [];
}

function asDiagnosticsArray(value: unknown): ConceptMap['diagnostics'] {
  return Array.isArray(value) ? (value as ConceptMap['diagnostics']) : [];
}

function normalizeConceptMap(raw: Record<string, unknown>): ConceptMap {
  return {
    id: asString(raw.id, ''),
    title: asString(raw.title, ''),
    status: asString(raw.status, ''),
    description: asString(raw.description, ''),
    dslHash: asString(raw.dsl_hash, ''),
    nodes: asCmNodeArray(raw.nodes),
    edges: asCmEdgeArray(raw.edges),
    diagnostics: asDiagnosticsArray(raw.diagnostics),
  };
}

// ---------------------------------------------------------------------------
// API functions
// ---------------------------------------------------------------------------

export async function fetchGraph(): Promise<RawGraph> {
  const r = await fetch('/api/graph');
  if (!r.ok) {
    const body = await r.text();
    throw new ApiError('Failed to fetch graph', r.status, body, '/api/graph');
  }
  return (await r.json()) as RawGraph;
}

export async function fetchActionabilityGraph(): Promise<ActionabilityView> {
  const r = await fetch('/api/survey');
  if (!r.ok) {
    const body = await r.text();
    throw new ApiError(
      'Failed to fetch actionability graph',
      r.status,
      body,
      '/api/survey',
    );
  }
  return (await r.json()) as ActionabilityView;
}

export async function refreshGraph(): Promise<{ ok: boolean }> {
  const r = await fetch('/api/refresh', { method: 'POST' });
  if (!r.ok) {
    const body = await r.text();
    throw new ApiError('Failed to refresh', r.status, body, '/api/refresh');
  }
  return (await r.json()) as { ok: boolean };
}

export async function fetchHealth(): Promise<{
  ok: boolean;
  dot: { ok: boolean; version?: string };
  graph: { ok: boolean };
}> {
  const r = await fetch('/api/health');
  if (!r.ok) {
    const body = await r.text();
    throw new ApiError(
      'Failed to fetch health',
      r.status,
      body,
      '/api/health',
    );
  }
  return (await r.json()) as {
    ok: boolean;
    dot: { ok: boolean; version?: string };
    graph: { ok: boolean };
  };
}

export async function renderDot(dotText: string): Promise<string> {
  const r = await fetch('/api/dot/svg', {
    method: 'POST',
    headers: { 'Content-Type': 'text/plain' },
    body: dotText,
  });
  if (!r.ok) {
    const body = await r.text();
    throw new ApiError('DOT render failed', r.status, body, '/api/dot/svg');
  }
  return r.text();
}

export async function fetchMarkdown(id: string): Promise<string> {
  const r = await fetch(
    '/api/entity/' + encodeURIComponent(id) + '/markdown',
  );
  if (!r.ok) {
    const body = await r.text();
    throw new ApiError(
      'Failed to fetch markdown',
      r.status,
      body,
      '/api/entity/' + id + '/markdown',
    );
  }
  return r.text();
}

export async function fetchConceptMap(id: string): Promise<ConceptMap> {
  const r = await fetch('/api/concept-map/' + encodeURIComponent(id));
  if (!r.ok) {
    const body: unknown = await r.json();
    const message = extractErrorMessage(body, 'Failed to fetch concept map');
    throw new ApiError(
      message,
      r.status,
      JSON.stringify(body),
      '/api/concept-map/' + id,
    );
  }
  const data: unknown = await r.json();
  return normalizeConceptMap(data as Record<string, unknown>);
}

export async function mutateConceptMap(
  id: string,
  action: string,
  params: Record<string, string>,
  baseHash?: string,
): Promise<unknown> {
  const body: Record<string, string> = { action, ...params };
  if (baseHash !== undefined) {
    body.base_hash = baseHash;
  }
  const r = await fetch('/api/concept-map/' + encodeURIComponent(id), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  const data: unknown = await r.json();
  if (!r.ok) {
    const message = extractErrorMessage(data, 'Mutation failed');
    throw new ApiError(
      message,
      r.status,
      JSON.stringify(data),
      '/api/concept-map/' + id,
    );
  }
  return data;
}
