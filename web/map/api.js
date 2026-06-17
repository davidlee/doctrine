/* api.js — HTTP layer for Doctrine Map frontend */
/* global model */

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

api.refreshGraph = function() {
  return fetch('/api/refresh', { method: 'POST' }).then(function(r) {
    if (!r.ok) return r.text().then(function(body) {
      throw new ApiError('Failed to refresh', r.status, body, '/api/refresh');
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

api.fetchMarkdown = function(id) {
  return fetch('/api/entity/' + encodeURIComponent(id) + '/markdown').then(function(r) {
    if (!r.ok) {
      return r.text().then(function(body) {
        throw new ApiError('Failed to fetch markdown', r.status, body,
          '/api/entity/' + id + '/markdown');
      });
    }
    return r.text();
  });
};

api.fetchConceptMap = function(id) {
  return fetch('/api/concept-map/' + encodeURIComponent(id)).then(function(r) {
    if (!r.ok) {
      return r.json().then(function(body) {
        throw new ApiError(
          body.message || 'Failed to fetch concept map',
          r.status,
          JSON.stringify(body),
          '/api/concept-map/' + id
        );
      });
    }
    return r.json().then(function(data) {
      return model.normalizeConceptMap(data);
    });
  });
};

api.mutateConceptMap = function(id, action, params, baseHash) {
  var body = Object.assign({ action: action }, params);
  if (baseHash !== undefined) body.base_hash = baseHash;
  return fetch('/api/concept-map/' + encodeURIComponent(id), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body)
  }).then(function(r) {
    return r.json().then(function(data) {
      if (!r.ok) {
        throw new ApiError(
          data.message || 'Mutation failed',
          r.status,
          JSON.stringify(data),
          '/api/concept-map/' + id
        );
      }
      return data;
    });
  });
};
