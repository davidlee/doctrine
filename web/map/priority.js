/* global d3 */

(function() {
  'use strict';

  function emptyLayout() {
    return { nodes: [], edges: [] };
  }

  function layoutGraph(view) {
    var i;

    if (!view || !Array.isArray(view.nodes) || view.nodes.length === 0) return emptyLayout();

    var edges = Array.isArray(view.edges) ? view.edges : [];
    var parentsByTarget = new Map();
    for (i = 0; i < edges.length; i++) {
      if (!edges[i] || !edges[i].source || !edges[i].target) continue;
      if (!parentsByTarget.has(edges[i].target)) parentsByTarget.set(edges[i].target, []);
      parentsByTarget.get(edges[i].target).push(edges[i].source);
    }

    var stratifyData = [];
    for (i = 0; i < view.nodes.length; i++) {
      if (!view.nodes[i] || !view.nodes[i].id) continue;
      stratifyData.push({
        id: view.nodes[i].id,
        parentIds: parentsByTarget.get(view.nodes[i].id) || [],
        data: view.nodes[i]
      });
    }
    if (stratifyData.length === 0) return emptyLayout();

    var dag = d3.graphStratify()(stratifyData);
    var layout = d3.sugiyama().nodeSize([72, 28]).gap([20, 30])(dag);

    var nodes = [];
    Array.from(dag.nodes()).forEach(function(node) {
      var base = node.data && node.data.data ? node.data.data : {};
      nodes.push(Object.assign({}, base, {
        x: typeof node.x === 'number' ? node.x : 0,
        y: typeof node.y === 'number' ? node.y : 0
      }));
    });

    return { nodes: nodes, edges: edges, width: layout.width || 960, height: layout.height || 600 };
  }

  function renderGraph(opts) {
    var container = opts && opts.container;
    var layout = opts && opts.layout;
    var focusId = opts && opts.focusId;
    var zoomId = opts && opts.zoomId;
    var onZoomToggle = opts && opts.onZoomToggle;
    var onNodeClick = opts && opts.onNodeClick;
    var onNodeHoverEnter = opts && opts.onNodeHoverEnter;
    var onNodeHoverLeave = opts && opts.onNodeHoverLeave;
    var svgNs = 'http://www.w3.org/2000/svg';
    var i;

    if (!container) return;
    container.innerHTML = '';

    var svg = document.createElementNS(svgNs, 'svg');
    var vw = layout && layout.width ? layout.width : 960;
    var vh = layout && layout.height ? layout.height : 600;
    svg.setAttribute('viewBox', '0 0 ' + vw + ' ' + vh);
    svg.setAttribute('width', '100%');
    svg.setAttribute('height', '100%');

    var defs = document.createElementNS(svgNs, 'defs');
    var marker = document.createElementNS(svgNs, 'marker');
    marker.setAttribute('id', 'needs-arrow');
    marker.setAttribute('markerWidth', '10');
    marker.setAttribute('markerHeight', '7');
    marker.setAttribute('refX', '9');
    marker.setAttribute('refY', '3.5');
    marker.setAttribute('orient', 'auto');
    var markerPath = document.createElementNS(svgNs, 'path');
    markerPath.setAttribute('d', 'M0,0 L10,3.5 L0,7 z');
    markerPath.setAttribute('fill', 'var(--priority-needs-edge, #C0392B)');
    marker.appendChild(markerPath);
    defs.appendChild(marker);
    svg.appendChild(defs);

    var nodes = layout && Array.isArray(layout.nodes) ? layout.nodes : [];
    var edges = layout && Array.isArray(layout.edges) ? layout.edges : [];
    var nodeMap = new Map();
    for (i = 0; i < nodes.length; i++) nodeMap.set(nodes[i].id, nodes[i]);

    // Zoom layer — all graph content lives in this <g> so we can transform it.
    var zoomLayer = document.createElementNS(svgNs, 'g');
    zoomLayer.setAttribute('class', 'priority-zoom-layer');
    var ZOOM_SCALE = 5;
    if (zoomId) {
      var zn = nodeMap.get(zoomId);
      if (zn) {
        var tx = vw / 2 - zn.x * ZOOM_SCALE;
        var ty = vh / 2 - zn.y * ZOOM_SCALE;
        zoomLayer.setAttribute('transform', 'translate(' + tx.toFixed(1) + ' ' + ty.toFixed(1) + ') scale(' + ZOOM_SCALE + ')');
      }
    }
    svg.appendChild(zoomLayer);

    // Background click to zoom out.
    svg.addEventListener('click', function(e) {
      if (e.target === svg && zoomId && typeof onZoomToggle === 'function') {
        onZoomToggle(null);
      }
    });

    for (i = 0; i < edges.length; i++) {
      var edge = edges[i];
      var source = nodeMap.get(edge.source);
      var target = nodeMap.get(edge.target);
      var line;
      if (!source || !target) continue;

      line = document.createElementNS(svgNs, 'line');
      line.setAttribute('x1', source.x);
      line.setAttribute('y1', source.y);
      line.setAttribute('x2', target.x);
      line.setAttribute('y2', target.y);
      line.setAttribute('class', edge.kind === 'needs' ? 'priority-edge priority-needs-edge' : 'priority-edge priority-after-edge');
      if (edge.kind === 'needs') line.setAttribute('marker-end', 'url(#needs-arrow)');
      zoomLayer.appendChild(line);
    }

    for (i = 0; i < nodes.length; i++) {
      var node = nodes[i];
      var nw = Math.max(72, (String(node.id || '').length * 7) + 16);
      var group = document.createElementNS(svgNs, 'g');
      var rect = document.createElementNS(svgNs, 'rect');
      var text = document.createElementNS(svgNs, 'text');
      var classes = 'priority-node priority-' + (node.actionability || 'terminal');

      if (node.id === focusId) classes += ' priority-node--focus';
      if (node.id === zoomId) classes += ' priority-node--zoom';
      group.setAttribute('class', classes);
      group.setAttribute('transform', 'translate(' + node.x + ' ' + node.y + ')');

      rect.setAttribute('x', -nw / 2);
      rect.setAttribute('y', -14);
      rect.setAttribute('width', nw);
      rect.setAttribute('height', 28);
      rect.setAttribute('rx', 6);
      rect.setAttribute('ry', 6);
      group.appendChild(rect);

      text.setAttribute('text-anchor', 'middle');
      text.setAttribute('dominant-baseline', 'middle');
      text.textContent = node.id;
      group.appendChild(text);

      if ((node.consequence || 0) > 0) {
        var badge = document.createElementNS(svgNs, 'g');
        var circle = document.createElementNS(svgNs, 'circle');
        var badgeText = document.createElementNS(svgNs, 'text');
        badge.setAttribute('class', 'priority-consequence-badge');
        badge.setAttribute('transform', 'translate(' + ((nw / 2) - 6) + ' -10)');
        circle.setAttribute('r', '8');
        badge.appendChild(circle);
        badgeText.setAttribute('text-anchor', 'middle');
        badgeText.setAttribute('dominant-baseline', 'middle');
        badgeText.textContent = String(node.consequence);
        badge.appendChild(badgeText);
        group.appendChild(badge);
      }

      if (typeof onNodeClick === 'function') {
        group.addEventListener('click', function(e, id) {
          return function(evt) {
            evt.stopPropagation();
            if (typeof onZoomToggle === 'function') {
              onZoomToggle(zoomId === id ? null : id);
            }
            onNodeClick(id);
          };
        }(node.id));
      }
      if (typeof onNodeHoverEnter === 'function') {
        group.addEventListener('mouseenter', function(id, currentGroup) {
          return function() {
            currentGroup.classList.add('priority-node--hover');
            onNodeHoverEnter(id);
          };
        }(node.id, group));
      }
      if (typeof onNodeHoverLeave === 'function') {
        group.addEventListener('mouseleave', function(currentGroup) {
          return function() {
            currentGroup.classList.remove('priority-node--hover');
            onNodeHoverLeave();
          };
        }(group));
      }

      zoomLayer.appendChild(group);
    }

    container.appendChild(svg);
  }

  window.priority = { layoutGraph: layoutGraph, renderGraph: renderGraph };
})();
