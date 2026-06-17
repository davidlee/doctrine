// Doctrine Map Explorer — concept map rendering (SL-083 PHASE-05)
// Exposed on window.cm. Depends on: model (cmNeighbourhood, buildNodeLabelList,
// buildRelLabelList), dot (cmGraphToDot), api (renderDot), svg (injectHitRects,
// wireHandlers), render (escapeHtml, escapeAttr).
/* global model, dot, api, svg, render */
/* exported cm */

var cm = {};

function renderCmHoverPane(nodeKey, cmData) {
  var pane = document.querySelector('.hover-detail');
  if (!pane) return;
  if (!nodeKey) {
    pane.innerHTML = '<span class="placeholder">Hover a node for details</span>';
    return;
  }
  var label = nodeKey;
  if (cmData) {
    for (var i = 0; i < cmData.nodes.length; i++) {
      if (cmData.nodes[i].key === nodeKey) {
        label = cmData.nodes[i].label;
        break;
      }
    }
  }
  pane.innerHTML = '<div class="hover-detail-content">' +
    '<span class="hover-detail-title">' + render.escapeHtml(label) + '</span>' +
    '<span class="hover-detail-meta">concept map node</span>' +
    '</div>';
}

function diagnosticLine(d) {
  if (!d) return null;
  var keys = Object.keys(d);
  if (keys.length === 0) return null;
  var variant = d[keys[0]];
  if (!variant || typeof variant !== 'object') return null;
  if (typeof variant.line === 'number') return variant.line;
  if (typeof variant.line_a === 'number') return variant.line_a;
  return null;
}

function formatDiagnostic(d) {
  if (!d) return 'Unknown diagnostic';
  var keys = Object.keys(d);
  if (keys.length === 0) return 'Unknown diagnostic';
  var variant = keys[0];
  var v = d[variant] || {};
  switch (variant) {
    case 'CanonicalNodeCollision':
      return 'Node label "' + render.escapeHtml(v.label || '') + '" collides with key "' + render.escapeHtml(v.key || '') + '" (first label "' + render.escapeHtml(v.first_label || '') + '" takes precedence)';
    case 'SelfEdge':
      return 'Self-referencing edge: "' + render.escapeHtml(v.node_key || '') + '" → "' + render.escapeHtml(v.node_key || '') + '"';
    case 'SimilarNodeLabel':
      return 'Similar node labels: "' + render.escapeHtml(v.label_a || '') + '" / "' + render.escapeHtml(v.label_b || '') + '"';
    case 'RelationDrift':
      return 'Relation "' + render.escapeHtml(v.rel_a || '') + '" appears only once — possible typo';
    case 'EntityRefLike':
      return '"' + render.escapeHtml(v.label || '') + '" looks like an entity reference';
    case 'MalformedLine':
      return 'Malformed DSL at "' + render.escapeHtml(v.text || '') + '"';
    case 'EmptyLabel':
      return 'Empty label in DSL';
    case 'DuplicateEdge':
      return 'Duplicate edge: "' + render.escapeHtml(v.from_key || '') + '" > "' + render.escapeHtml(v.rel || '') + '" > "' + render.escapeHtml(v.to_key || '') + '" (first at line ' + (v.existing_line !== undefined ? v.existing_line : '?') + ')';
    default:
      return 'Diagnostic: ' + variant;
  }
}

cm.renderDiagram = function(opts) {
  var container = opts.container, cmData = opts.cm, focusKey = opts.focusKey;
  var dotAvailable = opts.dotAvailable, seq = opts.seq, getCurrentSeq = opts.getCurrentSeq;
  var onClick = opts.onClick, onHoverEnter = opts.onHoverEnter, onHoverLeave = opts.onHoverLeave;
  if (!container) return;
  var dotText = dot.cmGraphToDot(cmData, focusKey);
  if (!dotAvailable) {
    container.innerHTML = '<p class="error">Graphviz not available.</p><pre>' + render.escapeHtml(dotText) + '</pre>';
    return;
  }
  container.innerHTML = '<p class="loading">Rendering diagram…</p>';
  api.renderDot(dotText).then(function(svgText) {
    if (getCurrentSeq && seq !== getCurrentSeq()) return;
    var clean = window.DOMPurify.sanitize(svgText, { USE_PROFILES: { svg: true } });
    container.innerHTML = clean;
    var svgEl = container.querySelector('svg');
    if (svgEl) {
      svg.injectHitRects(svgEl);
      svg.wireHandlers(svgEl, function(g) {
        var t = g.querySelector('title');
        return t ? t.textContent.trim() : '';
      }, {
        onClick: onClick,
        onHoverEnter: function(key) { renderCmHoverPane(key, cmData); if (onHoverEnter) onHoverEnter(key); },
        onHoverLeave: function() { renderCmHoverPane(null, cmData); if (onHoverLeave) onHoverLeave(); }
      });
    }
  }).catch(function() {
    if (getCurrentSeq && seq !== getCurrentSeq()) return;
    container.innerHTML = '<p class="error">Graphviz not available</p>';
  });
};

cm.renderEdgeTable = function(opts) {
  var container = opts.container, cmData = opts.cm, focusKey = opts.focusKey;
  var depth = opts.depth, editing = opts.editing, editingNode = opts.editingNode;
  var onRemoveEdge = opts.onRemoveEdge, onRenameNode = opts.onRenameNode;
  var onSubmitRename = opts.onSubmitRename, onCancelRename = opts.onCancelRename;
  if (!container) return;
  if (!cmData) { container.innerHTML = ''; container.style.display = 'none'; return; }
  container.style.display = 'block';
  var edges = cmData.edges || [];
  if (!editing && focusKey) {
    var filtered = model.cmNeighbourhood(cmData, focusKey, depth);
    edges = filtered.edges;
  }
  var editingKey = editingNode ? editingNode.key : null;
  var editingLabel = editingNode ? editingNode.label : '';
  var html = '<table class="cm-edges"><thead><tr><th>Source</th><th>Relation</th><th>Target</th>';
  if (editing) html += '<th></th>';
  html += '</tr></thead><tbody>';
  if (edges.length === 0) {
    html += '<tr><td colspan="' + (editing ? '4' : '3') + '"><span class="placeholder">No edges</span></td></tr>';
  } else {
    edges.forEach(function(edge) {
      html += '<tr class="cm-edge-row"><td>';
      if (editingKey && edge.from_key === editingKey && editing) {
        html += '<input type="text" class="cm-rename-input" data-key="' + render.escapeAttr(editingKey) + '" value="' + render.escapeAttr(editingLabel) + '">';
      } else {
        html += '<span class="cm-edge-label' + (editing ? ' cm-editable-node" data-key="' + render.escapeAttr(edge.from_key) + '" data-label="' + render.escapeAttr(edge.from_label) : '') + '">' + render.escapeHtml(edge.from_label) + '</span>';
      }
      html += '</td><td>' + render.escapeHtml(edge.rel) + '</td><td>';
      if (editingKey && edge.to_key === editingKey && editing) {
        html += '<input type="text" class="cm-rename-input" data-key="' + render.escapeAttr(editingKey) + '" value="' + render.escapeAttr(editingLabel) + '">';
      } else {
        html += '<span class="cm-edge-label' + (editing ? ' cm-editable-node" data-key="' + render.escapeAttr(edge.to_key) + '" data-label="' + render.escapeAttr(edge.to_label) : '') + '">' + render.escapeHtml(edge.to_label) + '</span>';
      }
      html += '</td>';
      if (editing) html += '<td><button class="cm-remove-btn" data-source="' + render.escapeAttr(edge.from_label) + '" data-rel="' + render.escapeAttr(edge.rel) + '" data-target="' + render.escapeAttr(edge.to_label) + '" title="Remove edge">✕</button></td>';
      html += '</tr>';
    });
  }
  html += '</tbody></table>';
  container.innerHTML = html;
  if (editing) {
    var removeBtns = container.querySelectorAll('.cm-remove-btn');
    for (var i = 0; i < removeBtns.length; i++) (function(btn) { btn.addEventListener('click', function() { if (onRemoveEdge) onRemoveEdge(btn.getAttribute('data-source'), btn.getAttribute('data-rel'), btn.getAttribute('data-target')); }); })(removeBtns[i]);
    var editableNodes = container.querySelectorAll('.cm-editable-node');
    for (var j = 0; j < editableNodes.length; j++) (function(el) { el.addEventListener('click', function() { if (onRenameNode) onRenameNode(el.getAttribute('data-key')); }); })(editableNodes[j]);
    var renameInputs = container.querySelectorAll('.cm-rename-input');
    for (var k = 0; k < renameInputs.length; k++) (function(inp) { if (k === 0) inp.focus(); inp.addEventListener('keydown', function(ev) { if (ev.key === 'Enter') { ev.preventDefault(); if (onSubmitRename) onSubmitRename(inp.value); } else if (ev.key === 'Escape') { ev.preventDefault(); if (onCancelRename) onCancelRename(); } }); })(renameInputs[k]);
  }
};

cm.renderDiagnostics = function(opts) {
  var container = opts.container, diagnostics = opts.diagnostics;
  if (!container) return;
  if (!diagnostics || diagnostics.length === 0) { container.style.display = 'none'; return; }
  var html = '<h3>Diagnostics</h3>';
  for (var i = 0; i < diagnostics.length; i++) {
    var d = diagnostics[i], msg = formatDiagnostic(d), line = diagnosticLine(d);
    var prefix = line !== null ? ('line ' + line + ': ') : '';
    html += '<div class="cm-diag-item">⚠ ' + render.escapeHtml(prefix + msg) + '</div>';
  }
  container.innerHTML = html;
  container.style.display = 'block';
};

cm.renderAddEdgeForm = function(opts) {
  var container = opts.container, cmData = opts.cm, editing = opts.editing, onSubmit = opts.onSubmit;
  if (!container) return;
  if (!editing) { container.style.display = 'none'; return; }
  container.style.display = 'block';
  var labels = model.buildNodeLabelList(cmData), rels = model.buildRelLabelList(cmData);
  var html = '<form class="add-edge-form" onsubmit="return false;"><div class="add-edge-fields">';
  html += '<input type="text" class="cm-input cm-source" list="cm-source-list" placeholder="Source">';
  html += '<datalist id="cm-source-list">' + labels.map(function(l) { return '<option value="' + render.escapeAttr(l) + '">'; }).join('') + '</datalist>';
  html += '<input type="text" class="cm-input cm-rel" list="cm-rel-list" placeholder="relation">';
  html += '<datalist id="cm-rel-list">' + rels.map(function(r) { return '<option value="' + render.escapeAttr(r) + '">'; }).join('') + '</datalist>';
  html += '<input type="text" class="cm-input cm-target" list="cm-target-list" placeholder="Target">';
  html += '<datalist id="cm-target-list">' + labels.map(function(l) { return '<option value="' + render.escapeAttr(l) + '">'; }).join('') + '</datalist>';
  html += '<button type="submit" class="cm-add-btn">Add edge</button></div><div class="cm-add-error" style="display:none;"></div></form>';
  container.innerHTML = html;
  var form = container.querySelector('.add-edge-form');
  form.addEventListener('submit', function() { if (onSubmit) onSubmit(form.querySelector('.cm-source').value, form.querySelector('.cm-rel').value, form.querySelector('.cm-target').value); });
};

cm.renderEditToggle = function(opts) {
  var header = opts.header, editing = opts.editing, onToggle = opts.onToggle;
  if (!header) return;
  var existing = header.querySelector('.cm-edit-toggle');
  if (existing) existing.remove();
  var btn = document.createElement('button');
  btn.className = 'cm-edit-toggle';
  btn.textContent = editing ? 'Done' : 'Edit';
  btn.addEventListener('click', function() { if (onToggle) onToggle(); });
  header.appendChild(btn);
};
