// API shapes
import type { GraphViewport } from './viewport';

export interface RawGraph {
  nodes: Record<string, RawCatalogNode>;
  edges: RawEdge[];
}

export interface RawCatalogNode {
  title: string;
  status: string;
  kind_label: string;
}

export interface RawEdge {
  source: string;
  label: { Validated?: string; Raw?: string };
  target: { Resolved?: string } | null;
}

export interface ActionabilityView {
  kind: string;
  nodes: ActionabilityNode[];
  edges: ActionabilityEdge[];
}

export interface ActionabilityNode {
  id: string;
  title: string;
  kind: string;
  status: string;
  actionability: string;
  score: number;
  rank: number;
  blockers: string[];
}

export interface ActionabilityEdge {
  source: string;
  target: string;
  kind: string;
}

// Normalized internal types
export interface Graph {
  nodes: Map<string, CatalogNode>;
  edges: Edge[];
  incoming: Map<string, Edge[]>;
  outgoing: Map<string, Edge[]>;
  edgeById: Map<string, Edge>;
}

export interface CatalogNode {
  id: string;
  title: string;
  status: string;
  kindPrefix: string;
  kindLabel: string;
  raw: RawCatalogNode;
}

export interface Edge {
  id: string;
  source: string;
  label: string;
  target: string;
  raw: RawEdge;
}

export interface Route {
  view: 'focus' | 'edge';
  id: string | null;
  depth: number;
  cmFocus: string | null;
}

export interface ConceptMap {
  id: string;
  title: string;
  status: string;
  description: string;
  dslHash: string;
  nodes: CmNode[];
  edges: CmEdge[];
  diagnostics: unknown[];
}

export interface CmNode {
  key: string;
  label: string;
}

export interface CmEdge {
  from_key: string;
  from_label: string;
  rel: string;
  to_key: string;
  to_label: string;
  line?: number;
}

export interface Neighbourhood {
  nodes: Set<string>;
  edges: Edge[];
}

export interface CmNeighbourhood {
  nodes: CmNode[];
  edges: CmEdge[];
}

/** Which cell of an edge-table row a pencil/click targets. */
export type CmCell = 'from' | 'rel' | 'to';

/** The backend op a (cell × edit-all scope) edit resolves to. */
export type CmEditOp =
  | 'rename_node_occurrence'
  | 'rename_node'
  | 'relabel_edge'
  | 'relabel_rel_all';

/**
 * The cell whose hover-pencil is active (the inline `<input>`). Carries the
 * full edge LABELS (to locate the row) plus which segment is being edited.
 * Identities are label-based — every CM mutation matches by label, and distinct
 * labels can derive the same key (`User Story` vs `User-Story`) — so the labels,
 * not a derived key, are what the rename/relabel submits.
 */
export interface CmEditingCell {
  from_label: string;
  rel: string;
  to_label: string;
  cell: CmCell;
}

// Mutable application state
export interface AppState {
  // Graph data
  graphRaw: RawGraph | null;
  graph: Graph;

  // Navigation
  focusId: string | null;
  depth: number;

  // Caches
  markdownCache: Map<string, string>;
  conceptMapCache: Map<string, ConceptMap>;

  // Concept map editing
  /** Scope toggle: a single instance (off) vs all rows sharing the label (on). */
  cmEditAll: boolean;
  /** Which cell's hover-pencil is active (the inline input); null = none. */
  cmEditingCell: CmEditingCell | null;
  cmFocusNode: CmNode | null;
  renderedCmFocus: string | null;
  cmCacheMutationSeq: number;
  renderedCmCacheSeq: number;

  // Rendering flags
  dotAvailable: boolean;
  hoveredId: string | null;
  viewMode: 'semantic' | 'actionability';
  renderedViewMode: 'semantic' | 'actionability' | null;
  actionabilityView: ActionabilityView | null;
  priorityZoomId: string | null;
  priorityTransform: { x: number; y: number; k: number } | null;
  priorityZoomPending: boolean;
  kindFilter: Set<string> | null;
  graphRenderSeq: number;
  /** Viewport state for the semantic (DOT/Graphviz) graph zoom/pan. null = fit on next render. */
  graphViewport: GraphViewport | null;
}
