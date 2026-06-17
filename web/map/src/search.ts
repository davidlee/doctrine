// Doctrine Map Explorer — search, keyboard nav, filters, depth (SL-091 PHASE-07)
// TypeScript rewrite of search.js. Depends on: model (searchFilter, findFocus, compareNodes),
// render (entityList, escapeHtml).

import type { CatalogNode, Graph } from './types';
import { searchFilter, findFocus, compareNodes } from './model';
import { entityList, escapeHtml } from './render';

// ---------------------------------------------------------------------------
// Kind filter collection
// ---------------------------------------------------------------------------

interface CollectKindFilterResult {
  allOn: boolean;
  filterSet: Set<string>;
}

function collectKindFilterInternal(container: Document | HTMLElement): CollectKindFilterResult {
  const cbs = container.querySelectorAll<HTMLInputElement>('.kind-checkbox input[type="checkbox"]');
  let allOn = true;
  for (const cb of cbs) {
    if (!cb.checked) {
      allOn = false;
      break;
    }
  }
  const filterSet = new Set<string>();
  for (const cb of cbs) {
    if (cb.checked) {
      const kinds = (cb.getAttribute('data-kinds') ?? '').split(',');
      for (const k of kinds) {
        const kp = k.trim();
        if (kp !== '') filterSet.add(kp);
      }
    }
  }
  return { allOn, filterSet };
}

/**
 * Collect checkbox state from kind filter checkboxes within container.
 * Returns Set<string> | null (null = all on, no filter active).
 */
export function collectKindFilter(container: Document | HTMLElement): Set<string> | null {
  const { allOn, filterSet } = collectKindFilterInternal(container);
  return allOn ? null : filterSet;
}

// ---------------------------------------------------------------------------
// Render filtered entities
// ---------------------------------------------------------------------------

export interface RenderFilteredEntitiesOpts {
  list: HTMLElement;
  graph: Graph;
  query: string;
  kindFilter: Set<string> | null;
  focusId: string | null;
  onFocus: (id: string) => void;
}

/** Compose model.searchFilter → render.entityList. No duplicated DOM construction. */
export function renderFilteredEntities(opts: RenderFilteredEntitiesOpts): void {
  let nodes = searchFilter(opts.query, opts.graph);
  const kf = opts.kindFilter;
  if (kf !== null) {
    nodes = nodes.filter((node: CatalogNode): boolean => kf.has(node.kindPrefix));
  }
  nodes.sort(compareNodes);
  entityList({
    container: opts.list,
    nodes,
    focusId: opts.focusId,
    onFocus: opts.onFocus,
  });
}

// ---------------------------------------------------------------------------
// Wire search input with keyboard navigation
// ---------------------------------------------------------------------------

export interface WireSearchOpts {
  input: HTMLInputElement | null;
  list: HTMLElement;
  graph: Graph;
  getFocusId: () => string | null;
  getKindFilter: () => Set<string> | null;
  onFocus: (id: string) => void;
}

/**
 * Wire search input with keyboard navigation.
 * listNavIndex is closure-local — never on global state.
 */
export function wireSearch(opts: WireSearchOpts): void {
  if (opts.input === null) return;
  const input: HTMLInputElement = opts.input;

  const list = opts.list;
  let listNavIndex: number | undefined;

  function renderCurrentList(): void {
    renderFilteredEntities({
      list,
      graph: opts.graph,
      query: input.value,
      kindFilter: opts.getKindFilter(),
      focusId: opts.getFocusId(),
      onFocus: opts.onFocus,
    });
  }

  input.addEventListener('input', () => {
    listNavIndex = undefined;
    renderCurrentList();
  });

  input.addEventListener('keydown', (e: KeyboardEvent) => {
    const items = list.querySelectorAll<HTMLElement>('.entity-item');

    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (items.length === 0) return;

      let navIdx: number;
      if (typeof listNavIndex === 'undefined' || listNavIndex < 0) {
        navIdx = e.key === 'ArrowDown' ? 0 : items.length - 1;
      } else {
        let tmp = listNavIndex + (e.key === 'ArrowDown' ? 1 : -1);
        if (tmp >= items.length) tmp = 0;
        if (tmp < 0) tmp = items.length - 1;
        navIdx = tmp;
      }
      listNavIndex = navIdx;

      for (let i = 0; i < items.length; i++) {
        const item = items[i];
        if (item !== undefined) {
          item.classList.toggle('entity-item--nav-highlight', i === navIdx);
        }
      }
      const highlighted = items[navIdx];
      if (highlighted !== undefined) {
        highlighted.scrollIntoView({ block: 'nearest' });
      }
    } else if (e.key === 'Enter') {
      if (
        typeof listNavIndex !== 'undefined' &&
        listNavIndex >= 0 &&
        items.length > 0
      ) {
        const navItem = items[listNavIndex];
        if (navItem !== undefined) {
          e.preventDefault();
          navItem.click();
          listNavIndex = undefined;
          return;
        }
      }
      const query = input.value.trim();
      if (query === '') return;
      const result = findFocus(query, opts.graph);
      if (result !== null) {
        opts.onFocus(result);
        listNavIndex = undefined;
      } else {
        // eslint-disable-next-line no-restricted-syntax
        list.innerHTML =
          '<li class="entity-item"><span class="placeholder">No match for \'' +
          escapeHtml(query) +
          '\'</span></li>';
      }
    } else if (e.key === 'Escape') {
      input.value = '';
      input.blur();
      listNavIndex = undefined;
      renderCurrentList();
    }
  });
}

// ---------------------------------------------------------------------------
// Wire kind filter checkboxes
// ---------------------------------------------------------------------------

export interface WireFiltersOpts {
  container: Document | HTMLElement;
  onChange: (filterSet: Set<string> | null) => void;
}

/** Wire kind filter checkboxes. onChange(filterSet): callback with Set<string> | null. */
export function wireFilters(opts: WireFiltersOpts): void {
  const container = opts.container;
  const toggleAll = container.querySelector<HTMLInputElement>('.toggle-all-cb');
  const kindCbs = container.querySelectorAll<HTMLInputElement>('.kind-checkbox input[type="checkbox"]');

  if (toggleAll !== null) {
    toggleAll.addEventListener('change', () => {
      for (const cb of kindCbs) {
        cb.checked = toggleAll.checked;
      }
      opts.onChange(collectKindFilter(container));
    });
  }

  for (const cb of kindCbs) {
    cb.addEventListener('change', () => {
      opts.onChange(collectKindFilter(container));
      // Sync toggle-all state
      if (toggleAll !== null) {
        let allOn = true;
        for (const kcb of kindCbs) {
          if (!kcb.checked) {
            allOn = false;
            break;
          }
        }
        toggleAll.checked = allOn;
      }
    });
  }

  // Initial collection on bootstrap
  opts.onChange(collectKindFilter(container));
}

// ---------------------------------------------------------------------------
// Wire depth buttons
// ---------------------------------------------------------------------------

export interface WireDepthButtonsOpts {
  container: Document | HTMLElement;
  onDepthChange: (d: number) => void;
}

/** Wire depth buttons. onDepthChange(depth): callback with numeric depth. */
export function wireDepthButtons(opts: WireDepthButtonsOpts): void {
  const btns = opts.container.querySelectorAll<HTMLElement>('.depth-btn');
  for (const btn of btns) {
    const depthStr = btn.getAttribute('data-depth');
    if (depthStr === null) continue;
    const depth = parseInt(depthStr, 10);
    if (Number.isNaN(depth)) continue;
    btn.addEventListener('click', () => {
      opts.onDepthChange(depth);
    });
  }
}

// ---------------------------------------------------------------------------
// Wire refresh button
// ---------------------------------------------------------------------------

export interface WireRefreshOpts {
  button: HTMLButtonElement | null;
  onRefresh: () => void;
}

/** Wire refresh button. onRefresh(): callback when clicked. */
export function wireRefresh(opts: WireRefreshOpts): void {
  if (opts.button === null) return;
  opts.button.addEventListener('click', () => {
    opts.onRefresh();
  });
}
