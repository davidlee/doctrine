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
