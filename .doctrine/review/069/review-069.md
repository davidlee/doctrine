# Review RV-069 — design of SL-093

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Design review of SL-093's `design.md` — the blueprint for decomposing
`web/map/style.css` (861 lines) into 6 files with `@layer` cascade
boundaries, resolving all 10 RV-065 findings. SL-093 is `design` status,
`supersedes SL-073`, no plan or implementation yet.

**Lines of attack:**

1. **Does the modularisation actually decompose?** The design collapses the
   original sidebar, graph, markdown, and table sections (~570 lines) into a
   single `layout.css`. That's 65% of the codebase in one file behind a thin
   layer declaration — a rename masquerading as modularisation. If the
   ":root → tokens" and "CM → concept-map.css" extractions are ~203 lines,
   what's left is structurally identical to the original monolith.

2. **Does `@layer` carry its weight or just add ceremony?** `layout` vs
   `components` layer split is a genuine structural improvement — but with
   only 205 lines in two component files, the layer architecture is
   overpowered for the actual decomposition. The design promises "new slices
   add CSS by appending an `@import` to the appropriate layer" — but at 570
   lines, `layout.css` is still the dumping ground.

3. **Are RV-065 findings genuinely resolved or just refiled?** F-1
   (monolithic file) is "resolved" by creating a 570-line `layout.css`. F-8
   (style.display → class toggling) maps 21+ sites — but the design's count
   claims must be verified against actual code. F-6 (dark mode gaps) requires
   token-level dark variants — the design shows them but must cover every
   hardcoded colour in the current source.

4. **Is the `u-hidden` class-based toggling approach correct for all 21
   sites?** `setPageMode()` in render.ts uses `style.display` for semantic
   view-mode switches (show relationship table vs hide it, show depth
   selector vs hide it) — not simple visibility toggles. The design maps
   these to `.u-hidden` classList operations, which conflates "page mode"
   with "element hidden."

5. **Does the token taxonomy cover everything?** The design promotes 17
   CM-specific colours to `--cm-*` tokens with dark variants. But the current
   source defines `--border-light` and `--bg-card` with fallbacks as ghost
   tokens — the design replaces them with `--cm-card-border`
   / `--cm-card-bg`, which is a semantic narrowing from a theme-level concept
   to a component-level one. Worth interrogating.

6. **Cascade fidelity.** The original file's top-to-bottom order IS the
   existing specificity resolution. The design asserts that `@import` order
   preserves this within `layout.css`. But `tokens.css` sits in the `tokens`
   layer, and `concept-map.css` in `components` — the latter always wins
   regardless of specificity. Is there any implicit dependency in the current
   flat file that the layer restructuring could break?

7. **`.cm-diag-item--last` requires JS support.** Replacing `:last-child`
   pseudo-class with a JS-set modifier class means every code path that
   renders diagnostics must remember to set this class. The design
   acknowledges the risk minimally — one sentence. This is a regression in
   robustness for a CSS-only mechanism.

**Invariants this review holds the design to:**
- RV-065 findings must be genuinely resolved, not refiled
- Modular decomposition must reduce blast radius, not just file count
- Every `style.display` site must be mapped to a class-based mechanism
- Custom property definitions must be complete — no undefined vars
- Cascade order must be preserved exactly
- Dark mode coverage must be complete — no gaps
- `@layer` contract must be coherent — no unlayered rules

## Synthesis

**Overall: acceptable.**

**Synopsis.** SL-093's design is earnest and directionally correct — extracting
tokens to a canonical `tokens.css`, splitting the two truly separable component
concerns (concept-map + priority), and using `@layer` to establish cascade
boundaries that will serve future slices. The RV-065 finding-resolution table is
complete and each resolution is at least directionally right.

But the design overstates its own modularisation. Collapsing the planned
sidebar, graph, markdown, and table modules into a single 570-line `layout.css`
— then calling it a 6-file decomposition — is a rhetorical sleight of hand. The
modularity delta from the original file is real: tokens are extracted (80 lines
→ separate file), component CSS is fenced (205 lines → separate files with
layer priority). That's genuine progress. The remaining 570-line `layout.css`
is a principled decision to preserve cascade fidelity, and the design has
committed to making that tradeoff explicit with trigger criteria for future
splitting (F-1).

The `u-hidden` proposal conflates page-mode switches with visibility toggles —
a distinction the design accepted and will address with a container-level
mode-class approach for `setPageMode()` (F-2). The `--cm-card-border`
rename was rightfully challenged: the token names express theme-level semantics
and the design agreed to keep the theme-level names (F-3). The
`.cm-diag-item--last` JS dependency was correctly identified as a robustness
regression — the design will retain `:last-child` instead (F-4).

The style.display audit count was off by one (21 TS assignments, not 22) — a
minor count error that signals the design was composed from memory, not a grep.
The design committed to a reconciled count with line numbers (F-5). The
data-attribute DOM coupling in concept-map.ts is pre-existing (SL-076) and out
of scope; consciously tolerated with a note (F-6).

The cascade-audit (F-7) and Vite dev/prod (F-8) gaps are bookkeeping — the
design accepted both as verification additions.

👍 The `@layer` architecture is the right call for a CSS codebase that will
grow. The token taxonomy (22 kind colours + theme + CM + priority) is thorough
and the dark-mode variants are complete. The naming convention (BEM-modifier
states, `cm-*`/`priority-*` component prefixes, `u-` utilities) is consistent
and enforceable. The RV-065 finding-resolution table covers all 10 findings
with concrete mechanical changes — none are "resolved" by hand-waving. The
verification section, once augmented with the cascade diff and dev/prod checks,
will be rigorous.

**Standing risks.** The 570-line `layout.css` remains a dumping ground in
practice — the design's "future splitting" trigger criteria (300 lines per
section, dedicated maintainer) provide a principled exit but no guarantee.
Every new layout concern will land in `layout.css` by default, and the layer
contract means within-layer specificity fights are resolved by normal cascade —
identical to the original monolith. The Vite dev/prod CSS processing difference
is addressed with a build verification step, but the design does not
investigate Vite's actual `@import` behaviour under `@layer` — the verification
is empirical (build and compare), not analytical.

**Haiku.**
> tokens excised clean —
> but the monolith remains
> renamed, not yet split.
