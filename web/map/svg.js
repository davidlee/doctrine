/* svg.js — shared SVG DOM manipulation for Doctrine Map frontend
 *
 * Handles hit-rect injection, click/hover handler wiring, focus highlight,
 * and legend dimming. Shared by entity graph and concept map rendering.
 */

var svg = {};

// Inject transparent hit-rect as first child of every <g class="node">.
// Idempotent — skips nodes that already have a hit-rect child.
svg.injectHitRects = function(svgEl) {
  var groups = svgEl.querySelectorAll('.node');
  for (var i = 0; i < groups.length; i++) {
    var g = groups[i];

    // Skip if already injected
    var existing = g.querySelector('[data-doctrine-hit]');
    if (existing) continue;

    try {
      var bbox = g.getBBox();
      if (bbox.width > 0 && bbox.height > 0) {
        var hitRect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        hitRect.setAttribute('x', bbox.x);
        hitRect.setAttribute('y', bbox.y);
        hitRect.setAttribute('width', bbox.width);
        hitRect.setAttribute('height', bbox.height);
        hitRect.setAttribute('fill', 'transparent');
        hitRect.setAttribute('stroke', 'none');
        hitRect.setAttribute('data-doctrine-hit', 'true');
        g.insertBefore(hitRect, g.firstChild);
      }
    } catch (_) { /* getBBox may fail on detached nodes */ }
  }
};

// Wire click + mouseenter/mouseleave on every <g class="node">.
// extractId: function(g) → string — reads <title> (CM) or <text> (entity graph).
// handlers: { onClick(id), onHoverEnter(id), onHoverLeave() }
svg.wireHandlers = function(svgEl, extractId, handlers) {
  var groups = svgEl.querySelectorAll('.node');
  for (var i = 0; i < groups.length; i++) {
    var g = groups[i];
    var nodeId = extractId(g);
    if (!nodeId) continue;

    g.classList.add('doctrine-node');

    g.addEventListener('click', (function(id) {
      return function() { handlers.onClick(id); };
    })(nodeId));

    g.addEventListener('mouseenter', (function(id) {
      return function() { handlers.onHoverEnter(id); };
    })(nodeId));

    g.addEventListener('mouseleave', function() {
      handlers.onHoverLeave();
    });
  }
};

// Apply/remove .doctrine-node--focus on the SVG <g> where extractId(g) === newId.
// extractId: function(g) → string — same contract as wireHandlers.
// oldId: previous focus. newId: current focus.
svg.applyFocusHighlight = function(svgEl, newId, oldId, extractId) {
  if (!svgEl) return;

  // Remove old focus
  if (oldId) {
    var oldNodes = svgEl.querySelectorAll('.doctrine-node--focus');
    for (var i = 0; i < oldNodes.length; i++) {
      oldNodes[i].classList.remove('doctrine-node--focus');
    }
  }

  // Apply new focus
  if (newId) {
    var groups = svgEl.querySelectorAll('.node');
    for (var j = 0; j < groups.length; j++) {
      if (extractId(groups[j]) === newId) {
        groups[j].classList.add('doctrine-node--focus');
        break;
      }
    }
  }
};

// Dim legend items whose edge labels are absent from the given neighbourhood.
svg.dimLegend = function(neighbourhood) {
  var items = document.querySelectorAll('.legend-item');
  if (!items.length) return;
  var edgeLabels = new Set();
  for (var ei = 0; ei < neighbourhood.edges.length; ei++) {
    edgeLabels.add(neighbourhood.edges[ei].label.toLowerCase());
  }
  for (var i = 0; i < items.length; i++) {
    var labels = (items[i].getAttribute('data-labels') || '').split(',');
    var anyPresent = false;
    for (var j = 0; j < labels.length; j++) {
      if (edgeLabels.has(labels[j].trim())) { anyPresent = true; break; }
    }
    items[i].classList.toggle('legend-dimmed', !anyPresent);
  }
};
