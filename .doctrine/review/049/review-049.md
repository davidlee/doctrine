# Review RV-049 — code-review of IMP-085

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Code-review of the Doctrine Map Explorer frontend (`web/map/`) — a single-page
Graphviz-backed entity browser. The review holds this code against:

- **Modularity**: single-responsibility decomposition, no parallel implementation,
  DRY.
- **State management**: encapsulation, clear ownership, no global mutable soup.
- **Security**: XSS surface (innerHTML, markdown, SVG), CSP readiness.
- **Standards**: semantic HTML, CSS custom properties, accessible UX.
- **Test confidence**: what the tests actually prove vs what they theatre.
- **Extensibility**: how hard it is to add a feature without refactoring.

**Lines of attack**:
1. `app.js` is 1406 lines — the God Object of SPAs. What's worth extracting?
2. Global `state` object with 14+ mutable fields — can anyone trace ownership?
3. Inline HTML string concatenation vs DOM construction — hygiene and escaping.
4. Three `escape*` functions, one dead — DRY enforcement?
5. CSS attribute-selector-on-inline-style — is that really the intent?
6. Vendor files committed without integrity hashes, version metadata, or lockfile.
7. Test strategy: one monolithic HTML file with hand-rolled assert — what confidence?

## Synthesis

**Overall**: acceptable

**Synopsis**

This is a working, feature-complete SPA that does its job. The XSS surface is
reasonably defended (DOMPurify on SVG, markdown-it with html:false, escapeHtml
in most user-facing string paths). The CSS uses custom properties well. The
entity graph BFS, router, and DOT generation are cleanly factored into their own
files. The concept-map authoring UI is a significant feature bolted onto the
same shell without collapsing.

But the code is visibly at its decomposition limit. `app.js` at 1,406 lines is
the undisputed God Object — it owns rendering, interaction, state transitions,
concept-map editing, error handling, and bootstrap in a single IIFE. The global
`state` object has no ownership discipline. Inline HTML concatenation pervades
the larger rendering functions. There is no module system — files communicate
through a shared mutable global namespace with load-order dependencies enforced
only by developer discipline.

The test suite is honest about its scope (it tests the pure data layer —
normalization, BFS, routing, DOT generation, CM neighbourhood) but provides zero
coverage for the 1,000+ lines of DOM rendering and event wiring that constitute
the actual user-facing behaviour. That is theatre that happens to test the right
things by accident.

**Standing risks**:
- The vendor files (markdown-it, DOMPurify, github-markdown.css) are unversioned
  and have no integrity hashes — a supply-chain blind spot.
- The CSS attribute selectors on inline style strings will silently break if
  inline style formatting changes.
- No dark mode toggle means users with atypical OS/browser preference mismatches
  have no recourse.

**Tradeoffs consciously accepted**:
- The 16 findings all land as follow-up (none are blockers) because the code
  ships working behaviour. The decomposition is improvement architecture, not
  defect remediation.
- The monolith pattern is the natural endpoint of "add one more feature" in a
  no-build-step vanilla JS project. The fix is not a patch — it is a module
  strategy decision (ES modules with import maps? a bundler? keep vanilla but
  extract files?) that belongs in IMP-085's planning, not in a drive-by review.
- The three-escape-function situation includes dead code (`encodeAttr`) that
  should be removed trivially; this is captured alongside the broader cleanup.

**Haiku**:

Fourteen hundred lines —
One file to rule them all.
State drifts in the breeze.
