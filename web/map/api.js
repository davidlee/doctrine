/* api.js — HTTP layer for Doctrine Map frontend */

function ApiError(message, status, body, endpoint) {
  this.name = 'ApiError';
  this.message = message;
  this.status = status;
  this.body = body;
  this.endpoint = endpoint;
}
ApiError.prototype = Object.create(Error.prototype);
ApiError.prototype.constructor = ApiError;

var api = {};

api.fetchGraph = function() {
  return fetch('/api/graph').then(function(r) {
    if (!r.ok) return r.text().then(function(body) {
      throw new ApiError('Failed to fetch graph', r.status, body, '/api/graph');
    });
    return r.json();
  });
};

api.fetchHealth = function() {
  return fetch('/api/health').then(function(r) {
    if (!r.ok) return r.text().then(function(body) {
      throw new ApiError('Failed to fetch health', r.status, body, '/api/health');
    });
    return r.json();
  });
};

api.renderDot = function(dotText) {
  return fetch('/api/dot/svg', {
    method: 'POST',
    headers: { 'Content-Type': 'text/plain' },
    body: dotText
  }).then(function(r) {
    if (!r.ok) return r.text().then(function(body) {
      throw new ApiError('DOT render failed', r.status, body, '/api/dot/svg');
    });
    return r.text();
  });
};
