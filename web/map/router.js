/* router.js — hash-based routing for Doctrine Map frontend */
/* global state */

var router = {};

function clampDepth(d) { return Math.max(0, Math.min(3, d)); }

router.parseHash = function() {
  var h = window.location.hash.slice(1);
  if (!h) return { view: 'focus', id: null, depth: state.depth, cmFocus: null };

  var focusMatch = h.match(/^\/focus\/([A-Z]+-\d+)(?:\?(.+))?$/);
  if (focusMatch) {
    var params = parseQueryString(focusMatch[2]);
    return {
      view: 'focus',
      id: focusMatch[1],
      depth: params.depth !== undefined ? clampDepth(parseInt(params.depth, 10)) : state.depth,
      cmFocus: params.cmFocus !== undefined ? decodeURIComponent(params.cmFocus) : null
    };
  }

  var edgeMatch = h.match(/^\/edge\/(e_[A-Za-z0-9_-]+)(?:\?(.+))?$/);
  if (edgeMatch) {
    var ep = parseQueryString(edgeMatch[2]);
    return {
      view: 'edge',
      id: edgeMatch[1],
      depth: ep.depth !== undefined ? clampDepth(parseInt(ep.depth, 10)) : state.depth,
      cmFocus: null
    };
  }

  return { view: 'focus', id: null, depth: state.depth, cmFocus: null };
};

function parseQueryString(qs) {
  var result = {};
  if (!qs) return result;
  var pairs = qs.split('&');
  for (var i = 0; i < pairs.length; i++) {
    var kv = pairs[i].split('=');
    if (kv.length === 2) {
      result[kv[0]] = kv[1];
    }
  }
  return result;
}

router.buildHash = function(view, id, depth) {
  var base = '#/' + view + '/' + id;
  var params = [];
  if (depth !== state.depth) {
    params.push('depth=' + depth);
  }
  if (state.cmFocusNode && state.cmFocusNode.key) {
    params.push('cmFocus=' + encodeURIComponent(state.cmFocusNode.key));
  }
  if (params.length > 0) {
    base += '?' + params.join('&');
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
