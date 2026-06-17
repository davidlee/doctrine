# Review RV-065 — code-review of SL-073

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Code review of `web/map/style.css` — the 861-line stylesheet for the Doctrine
Map Explorer. The file is attributed to SL-073 PHASE-01 but has organically
accreted CSS for SL-075 (SVG node styling), SL-076 (concept map authoring UI),
and the priority/DAG view without modularisation.

**Lines of attack:**

1. **Architecture & modularity.** Single 861-line file spanning 4+ subsystems —
   base layout, SVG nodes, concept-map authoring, priority/dag view. The
   SL-073 scope doc says this is "layout: sidebar + main (graph + markdown)."
   It has become a dumping ground. No file-level modularisation; no clear
   ownership boundaries.

2. **Custom property system completeness.** A second `:root {}` block at line
   ~776 redefines/extensions priority properties outside the canonical block.
   `--border-light` and `--bg-card` are used throughout the CM section with
   hardcoded fallbacks but are never defined — dead code masking missing tokens.
   Hardcoded colours litter the CM section (`#fde8e8`, `#c0392b`, `#13876b`,
   etc.) that never participate in the theme system.

3. **Naming convention coherence.** BEM (`doctrine-node--focus`), flat
   (`cm-edge-row`), and SMACSS-ish state classes (`.active`, `.hidden`) coexist
   without apparent convention. Selector specificity is inconsistent — some
   selectors are dangerously broad (`.hidden`), others overly specific
   (`.markdown-pane.fullscreen .markdown-body`).

4. **Dark mode coverage.** The `prefers-color-scheme: dark` block handles the
   base theme and CM diagnostics — but CM form controls, priority SVG nodes,
   and error/notice backgrounds have hardcoded light-only colours with no dark
   variants.

5. **Source order discipline.** `:root { --link: #2563eb; }` appears on line 76,
   after the main `:root` block closed on line 63 — a scattered token definition
   that makes the cascade harder to audit.

**Invariants this review holds the code to:**
- SL-073 scope: style.css is "layout: sidebar + main" — not a catch-all
- CSS custom properties must be complete: no undefined vars masked by fallbacks
- Dark mode coverage must be systematic, not ad-hoc per slice
- Naming conventions must be internally consistent
- No second `:root` block redefining properties after intermediate rules

## Synthesis

**Overall: acceptable.**

**Synopsis.** `web/map/style.css` is the 861-line accretion layer of four
slices — SL-073 (base layout), SL-075 (SVG node styling), SL-076 (concept map
authoring), and the priority/DAG view — all piled into a single flat file. It
is not *broken*; it renders correctly in both colour schemes and every
interactive feature ships its CSS. But it is the poster child for
organisational debt: each subsequent slice dumped its styles here because
"that's where the CSS goes," and no one pushed back.

Ten findings, none blocking:

- **3 🟠 major.** (F-1) The monolithic file has zero modularisation across 4+
  subsystems — no imports, no per-slice files, no documented ownership
  boundaries. (F-2) A second `:root {}` block at line 775 redefines priority
  custom properties 710 lines after the canonical block closed — forcing a
  full-file scan to audit the design token system. (F-3) `--border-light` and
  `--bg-card` are used throughout the CM section with hardcoded fallbacks but
  are *never defined* — dead custom property names masking what should be real
  design tokens.

- **5 🟡 minor.** (F-4) 18 distinct hardcoded hex colours in the CM section
  bypass the custom property system entirely. (F-5) Three naming conventions
  (BEM, flat, SMACSS-ish) coexist with no rule. (F-6) Dark mode covers the
  base layout but leaves CM form controls, priority SVG nodes, and error
  states in light-only purgatory. (F-8) JS inline `style.display` manipulation
  competes with CSS class-based toggling — two visibility mechanisms, same
  file. (F-10) The `.hidden` class name is dangerously broad — a utility-class
  collision magnet if any CSS framework ever enters the picture.

- **2 🔵 nit.** (F-7) `--link` scattered three places instead of living in the
  canonical `:root` block. (F-9) Compound selectors like
  `.markdown-pane.fullscreen .markdown-body` encode DOM hierarchy that will
  break on any refactor.

👍 The base layout is crisp — the grid sidebar/main split, the custom property
palette for 22 entity kinds, and the `prefers-color-scheme` toggle are clean
and well-judged. The section comment headers (`/* ---- Name ---- */`) provide
useful internal signposting, and the kind-pill pattern (background via custom
property, white/dark text gated by `data-kind`) is a nice bit of CSS-only
conditional styling. The SVG node highlight classes (`.doctrine-node--focus`,
`.doctrine-node--hover`) are well-scoped and composable.

**Standing risks / tradeoffs.** The primary risk is continued accretion — the
next slice that needs CSS will add another 200 lines to this file because
"that's the pattern." Every line added increases the blast radius of any
future refactor. The undefined `--border-light`/`--bg-card` tokens are a
particularly insidious trap: they look like they participate in the theme
system but don't. Someone will try to "fix dark mode for concept map" by
defining these tokens in `:root` and get surprises when the fallback values
were masking layout assumptions.

**Haiku.**
> four slices, one file —
> tokens unmade, fallbacks mask;
> the stylesheet accrues.
