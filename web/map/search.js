// Doctrine Map Explorer — search, keyboard nav, filters, depth (SL-083 PHASE-04)
// Exposed on window.search. Depends on: model (searchFilter, findFocus),
// render (entityList, escapeHtml), compareNodes (global sort).
/* global model, render, compareNodes */
/* exported search */

var search = {};

// Collect checkbox state from kind filter checkboxes within container.
// Returns Set<string> | null (null = all on, no filter active).
search.collectKindFilter = function(container) {
  var cbs = container.querySelectorAll('.kind-checkbox input[type="checkbox"]');
  var allOn = true;
  for (var i = 0; i < cbs.length; i++) {
    if (!cbs[i].checked) { allOn = false; break; }
  }
  if (allOn) return null;
  var result = new Set();
  for (var j = 0; j < cbs.length; j++) {
    if (cbs[j].checked) {
      var kinds = (cbs[j].getAttribute('data-kinds') || '').split(',');
      for (var k = 0; k < kinds.length; k++) {
        var kp = kinds[k].trim();
        if (kp) result.add(kp);
      }
    }
  }
  return result;
};

// Compose model.searchFilter → render.entityList.
// No duplicated DOM construction — delegates to render.entityList.
search.renderFilteredEntities = function(opts) {
  var nodes = model.searchFilter(opts.query || '', opts.graph);
  if (opts.kindFilter) {
    nodes = nodes.filter(function(node) { return opts.kindFilter.has(node.kindPrefix); });
  }
  nodes.sort(compareNodes);
  render.entityList({
    container: opts.list,
    nodes: nodes,
    focusId: opts.focusId,
    onFocus: opts.onFocus
  });
};

// Wire search input with keyboard navigation.
// listNavIndex is closure-local — never on global state.
// getKindFilter(): current kind filter Set | null (read dynamically).
// getFocusId(): current focus entity id (read dynamically).
search.wireSearch = function(opts) {
  var input = opts.input;
  var list = opts.list;
  if (!input) return;

  var listNavIndex = undefined;

  function renderCurrentList() {
    search.renderFilteredEntities({
      list: list,
      graph: opts.graph,
      query: input.value,
      kindFilter: opts.getKindFilter ? opts.getKindFilter() : null,
      focusId: opts.getFocusId ? opts.getFocusId() : null,
      onFocus: opts.onFocus
    });
  }

  input.addEventListener('input', function() {
    listNavIndex = undefined;
    renderCurrentList();
  });

  input.addEventListener('keydown', function(e) {
    var items = list ? list.querySelectorAll('.entity-item') : [];

    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (items.length === 0) return;
      if (typeof listNavIndex === 'undefined' || listNavIndex < 0) {
        listNavIndex = e.key === 'ArrowDown' ? 0 : items.length - 1;
      } else {
        listNavIndex += (e.key === 'ArrowDown' ? 1 : -1);
        if (listNavIndex >= items.length) listNavIndex = 0;
        if (listNavIndex < 0) listNavIndex = items.length - 1;
      }
      for (var i = 0; i < items.length; i++) {
        items[i].classList.toggle('nav-highlight', i === listNavIndex);
      }
      if (items[listNavIndex]) {
        items[listNavIndex].scrollIntoView({ block: 'nearest' });
      }
    } else if (e.key === 'Enter') {
      if (typeof listNavIndex !== 'undefined' && listNavIndex >= 0 && items.length > 0 && items[listNavIndex]) {
        e.preventDefault();
        items[listNavIndex].click();
        listNavIndex = undefined;
        return;
      }
      var query = input.value.trim();
      if (!query) return;
      var result = model.findFocus(query, opts.graph);
      if (result) {
        opts.onFocus(result);
        listNavIndex = undefined;
      } else {
        if (list) {
          list.innerHTML = '<li class="entity-item"><span class="placeholder">No match for \'' + render.escapeHtml(query) + '\'</span></li>';
        }
      }
    } else if (e.key === 'Escape') {
      input.value = '';
      input.blur();
      listNavIndex = undefined;
      renderCurrentList();
    }
  });
};

// Wire kind filter checkboxes. onChange(filterSet): callback with Set<string> | null.
search.wireFilters = function(opts) {
  var container = opts.container;
  var toggleAll = container.querySelector('.toggle-all-cb');
  var kindCbs = container.querySelectorAll('.kind-checkbox input[type="checkbox"]');

  if (toggleAll) {
    toggleAll.addEventListener('change', function() {
      for (var i = 0; i < kindCbs.length; i++) {
        kindCbs[i].checked = toggleAll.checked;
      }
      opts.onChange(search.collectKindFilter(container));
    });
  }

  for (var i = 0; i < kindCbs.length; i++) {
    kindCbs[i].addEventListener('change', function() {
      opts.onChange(search.collectKindFilter(container));
      // Sync toggle-all state
      if (toggleAll) {
        var allOn = true;
        for (var j = 0; j < kindCbs.length; j++) {
          if (!kindCbs[j].checked) { allOn = false; break; }
        }
        toggleAll.checked = allOn;
      }
    });
  }
  // Initial collection on bootstrap
  opts.onChange(search.collectKindFilter(container));
};

// Wire depth buttons. onDepthChange(depth): callback with numeric depth.
search.wireDepthButtons = function(opts) {
  var btns = opts.container.querySelectorAll('.depth-btn');
  for (var i = 0; i < btns.length; i++) {
    btns[i].addEventListener('click', (function(d) {
      return function() {
        opts.onDepthChange(d);
      };
    })(parseInt(btns[i].getAttribute('data-depth'), 10)));
  }
};

// Wire refresh button. onRefresh(): callback when clicked.
search.wireRefresh = function(opts) {
  if (!opts.button) return;
  opts.button.addEventListener('click', function() {
    opts.onRefresh();
  });
};
