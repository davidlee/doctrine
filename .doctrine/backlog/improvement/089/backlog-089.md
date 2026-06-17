# IMP-089: Improve web/map semantic HTML: landmarks, sections, ARIA

**Origin**: RV-049 F-13 (code-review of IMP-085)

Key opportunities (non-exhaustive):

- `<div class="layout">` → add `<main>` / `<aside>` pairing
- `<div class="focus-header">` → `<header>`
- `<div class="markdown-pane">` → `<section>` or `<article>`
- `<div class="hover-detail">` → `<aside>` or `<output>`
- Sidebar → `<nav>` with `aria-label`
- Landmark roles where native elements aren't used
