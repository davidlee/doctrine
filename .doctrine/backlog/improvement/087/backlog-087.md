# IMP-087: Add manual theme toggle (light/dark/auto) to Doctrine Map Explorer

**Origin**: RV-049 F-11 (code-review of IMP-085)

`style.css` uses `@media (prefers-color-scheme: dark)` only — no manual override.
A three-state toggle (light | dark | auto) with a `data-theme` attribute on
`<html>` and localStorage persistence is standard practice and required for
WCAG 2.2 advisory compliance.

The CSS custom properties architecture already supports this — it's a JS toggle
away. The dark-mode overrides for `.cm-diagnostics-panel` are currently hardcoded
inside the media query rather than inheriting from custom properties; that
hardcoding should also be addressed.
