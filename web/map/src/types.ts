// API shapes
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

export interface EditingNode {
  key: string;
  label: string;
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
  editingConceptMap: boolean;
  editingNode: EditingNode | null;
  cmFocusNode: CmNode | null;
  renderedCmFocus: string | null;
  cmCacheMutationSeq: number;
  renderedCmCacheSeq: number;

  // Rendering flags
  dotAvailable: boolean;
  hoveredId: string | null;
  viewMode: 'semantic' | 'actionability';
  actionabilityView: ActionabilityView | null;
  priorityZoomId: string | null;
  kindFilter: Set<string> | null;
  graphRenderSeq: number;
}
