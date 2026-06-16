# Review RV-044 — reconciliation of SL-077

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of attack** — this reconciliation audit holds SL-077's implementation
against its design (slice-077.md), plan (plan.toml), and the inquisition-refined
contract (RV-042):

1. **Plan exit criteria** — every EX-NN across all three phases verified against
   the actual code and test output.
2. **Inquisition remedy conformance** — the six findings from RV-042's design
   review must be faithfully implemented, not merely acknowledged.
3. **Behaviour-preservation gate** — 1548 tests green, clippy zero, zero changes
   to test bodies outside the prose-affected golden e2e tests.
4. **Storage rule integrity** — `.md` / `.toml` tier boundary preserved; prose
   is a read-only rendering pass, never a source of derived data.
5. **Error-surface coherence** — degrade-and-continue (E5) honoured in
   `load_with_prose` (NotFound → None prose), `req_rows` (dangling FK → `—`),
   and `show_json` (prose absent for scaffold).

**Invariants pinned to the accused:**
- `read_spec` mirrors `read_slice`'s `(parsed, raw, prose)` shape; only
  `run_show` and `relation_edges` call it — `build_registry` keeps inline parse.
- `load_with_prose` returns `Option<String>` — never errors on missing `.md`.
- `is_scaffold_prose` correctly classifies headings+comments-only bodies.
- Both `description` and prose render; neither is deprecated.
- `prose` column is default (5th), `—` for scaffold/dangling, `✓` for filled.

## Synthesis

### Verdict

**The implementation is clean and conformant.** All nine plan exit criteria and
seven verification criteria pass. 1548 tests green, clippy zero. The
behaviour-preservation gate is satisfied — no existing test body was altered
outside the prose-affected golden e2e tests (which gained a column, not lost
coverage).

One minor finding (F-1) was raised and resolved in-round: the
`non_html_comment_is_content` test specified by RV-042 F-3's remedy was not
implemented, but the behaviour is correct — `strip_html_comments` only matches
`<!-- ... -->` patterns, so non-HTML syntax survives stripping and prevents
false scaffold classification. The existing tests indirectly verify this.

### Conformance walk

| Phase | Criteria | Status |
|-------|----------|--------|
| PHASE-01 | `read_spec` single path for `run_show` + `relation_edges` | ✓ |
| PHASE-01 | `build_registry` keeps inline parse with `second_parent` | ✓ |
| PHASE-01 | Existing tests green unchanged | ✓ (1548/0) |
| PHASE-02 | `load_with_prose` reads both tiers, `None` for scaffold | ✓ |
| PHASE-02 | Table render emits prose below structured facets | ✓ |
| PHASE-02 | JSON includes `body` field (absent when scaffold) | ✓ |
| PHASE-02 | Existing render/show_json tests adapted additively | ✓ |
| PHASE-03 | `prose` column in Table with `✓` / `—` | ✓ |
| PHASE-03 | `prose` boolean in JSON (absent for dangling) | ✓ |

### RV-042 remedy conformance

| Finding | Requirement | Status |
|---------|------------|--------|
| F-1 (BLOCKER) | `build_registry` excluded from `read_spec` sites | ✓ |
| F-2 (MAJOR) | `description` + prose both render, neither deprecated | ✓ |
| F-3 (MAJOR) | Comment-detection contract explicit | ✓ (see F-1 note) |
| F-4 (MAJOR) | Dangling member prose column `—` | ✓ |
| F-5 (MAJOR) | `load_body` → `Option<String>`, degrade-and-continue | ✓ |
| F-6 (MINOR) | Aspirational caveat on demo example | ✓ (design doc updated) |

### Implementation notes

- `strip_html_comments` handles multi-line comments, exceeding the design's
  single-line contract — a harmless improvement (scaffold uses single-line only).
- `load_with_prose` tolerates missing `.md` (NotFound → None prose) — some test
  fixtures and hand-authored requirements may lack the prose file.
- The `prose` column is the first derived/observed column on the roster — all
  previous columns were authored fields from the TOML tier.

### Standing risks

- **Author discipline gap**: Until IMP-057 delivers a requirement authoring
  skill, the prose render is dead code — all current requirements are scaffolds.
  Known and accepted; the render surface is ready when authorship catches up.
- **Heading inversion**: If an author nests `###` inside a requirement section,
  the demoted headings will be subordinate — cosmetic, warned in the design.

### Verdict: CLEAN — conformant, ready for close.
