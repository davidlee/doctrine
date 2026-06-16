/* router.js — hash-based routing for Doctrine Map frontend */

var router = {};

function clampDepth(d) { return Math.max(0, Math.min(3, d)); }

router.parseHash = function() {
  var h = window.location.hash.slice(1);
  if (!h) return { view: 'focus', id: null, depth: state.depth };

  var focusMatch = h.match(/^\/focus\/([A-Z]+-\d+)(?:\?depth=(\d))?$/);
  if (focusMatch) {
    return {
      view: 'focus',
      id: focusMatch[1],
      depth: focusMatch[2] ? clampDepth(parseInt(focusMatch[2], 10)) : state.depth
    };
  }

  var edgeMatch = h.match(/^\/edge\/(e_[A-Za-z0-9_-]+)(?:\?depth=(\d))?$/);
  if (edgeMatch) {
    return {
      view: 'edge',
      id: edgeMatch[1],
      depth: edgeMatch[2] ? clampDepth(parseInt(edgeMatch[2], 10)) : state.depth
    };
  }

  return { view: 'focus', id: null, depth: state.depth };
};

router.buildHash = function(view, id, depth) {
  var base = '#/' + view + '/' + id;
  if (depth !== state.depth) {
    base += '?depth=' + depth;
  }
  return base;
};

router.setFocus = function(id, depth) {
  if (typeof depth === 'undefined') depth = state.depth;
  window.location.hash = router.buildHash('focus', id, depth);
};

router.setEdge = function(edgeId, depth) {
  if (typeof depth === 'undefined') depth = state.depth;
  window.location.hash = router.buildHash('edge', edgeId, depth);
};
