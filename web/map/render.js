// Doctrine Map Explorer — entity-graph DOM construction (SL-083 PHASE-03)
// Exposed on window.render. Depends on: model (neighbourhood), dot (graphToDot),
// api (renderDot, fetchMarkdown), svg (injectHitRects, wireHandlers, dimLegend).
/* exported render */

var render = {};

/* -----------------------------------------------------------------------
 * HTML escaping (F-5: moved from app.js; encodeAttr deleted as dead duplicate)
 * --------------------------------------------------------------------- */
render.escapeHtml = function(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
};

render.escapeAttr = function(str) {
  return str.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/'/g, '&#39;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
};

/* -----------------------------------------------------------------------
 * DOM element cache (F-9: eliminate repeated querySelector calls)
 * --------------------------------------------------------------------- */
render.elements = {};

render.cacheElements = function(root) {
  var qs = root.querySelector.bind(root);
  render.elements.entityList = qs('.entity-list');
  render.elements.focusHeader = qs('.focus-header');
  render.elements.graphArea = qs('.graph-area');
  render.elements.hoverDetail = qs('.hover-detail');
  render.elements.relationshipTable = qs('.relationship-table');
  render.elements.relationshipTableBody = qs('.relationship-table tbody');
  render.elements.markdownPane = qs('.markdown-pane');
  render.elements.tableToggle = qs('.table-toggle');
  render.elements.depthSelector = qs('.depth-selector');
};
