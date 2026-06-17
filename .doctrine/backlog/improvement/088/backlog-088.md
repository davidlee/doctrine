# IMP-088: Adopt test framework for web/map frontend with DOM rendering coverage

**Origin**: RV-049 F-12 (code-review of IMP-085)

`test.html` is a single monolithic file with hand-rolled `assert`/`assertEqual`.
It tests the pure data layer (normalization, BFS, routing, DOT) but provides zero
coverage for DOM rendering, event wiring, or concept-map editing — the 1,000+
lines that constitute user-facing behaviour.

## Options

- **Vitest + jsdom**: Lightweight, fast, good jsdom support. Requires npm.
- **Playwright/Cypress**: Browser-automation e2e tests against a running
  `doctrine map serve`. Higher confidence, slower.
- **QUnit in browser**: Zero-dependency, runs in test.html. Simplest adoption.
