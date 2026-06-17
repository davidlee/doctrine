/**
 * Application state — shared singleton.
 *
 * This is the canonical application state module used by all components.
 * The vitest mock in model.test.ts replaces this module with a subset of
 * the AppState shape for isolated testing.
 */

import type { AppState } from './types';

export const state: AppState = {
  // Graph data
  graphRaw: null,
  graph: {
    nodes: new Map(),
    edges: [],
    incoming: new Map(),
    outgoing: new Map(),
    edgeById: new Map(),
  },

  // Navigation
  focusId: null,
  depth: 1,

  // Caches
  markdownCache: new Map(),
  conceptMapCache: new Map(),

  // Concept map editing
  editingConceptMap: false,
  editingNode: null,
  cmFocusNode: null,
  renderedCmFocus: null,
  cmCacheMutationSeq: 0,
  renderedCmCacheSeq: 0,

  // Rendering flags
  dotAvailable: false,
  hoveredId: null,
  viewMode: 'semantic',
  actionabilityView: null,
  priorityZoomId: null,
  kindFilter: null,
  graphRenderSeq: 0,
};
