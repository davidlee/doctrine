# REV REV-070 — reconcile SL-278

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

<!-- Why this revision: what authored truth needs to change and why, the scope of
     the staged delta, and (for ADR/POL/STD/prose rows) the before/after excerpts
     the structured payload only labels. Seeded at `revision new`. -->

SL-278 (review read surface) replaced the review read path with one
engine-owned view. SL-278's scope (`slice-278.md`, "SPEC-032 read-surface
section reconciled through a REV") and design (sec-6, "SPEC-032 revision")
committed this revision at reconcile. RV-417's brief omitted it; reconcile
caught the omission against the design. One `modify` row on SPEC-032, landed
by hand.

## Reconcile narrative

SPEC-032 says things SL-278 made false or incomplete:

- **Overview** — module lists lack `review_ledger::view` (the projection) and
  `review::census` (the findings census).
- **Fail-safe reads** (`Vocab<T>` bullet) — names "the typed `review::Finding`
  projection used by `show`, the finding index, `--json` and MCP". That type
  is deleted; `review_ledger::FindingView` plays the role, and the census reads
  through it too.
- **Disclosure table** — `review findings` is a new listing surface with the
  same channel as `review list`.
- **Write surface** — command family lacks `findings`; MCP list lacks
  `review_findings`. The `show` bullet says "`show --json` carries the whole
  ledger, journals included" — false: JSON is the depth- and filter-shaped
  `ReviewView`.
- **Read surface** — no section describes depth, selection, filters, the
  census or the tolerant scan. New section added after Write surface.
- **Decisions** — add the SL-278 design decisions that bind the read surface.

### Before / after excerpts

Overview, `review_ledger` bullet — add after `transition`:
"the read projection every review read renders (`view`)".

Overview, `review` bullet — `show`, `list` and `status` (`read`) →
"`show`, `list` and `status` (`read`), the cross-review findings census
(`census`)".

Fail-safe reads — "The typed `review::Finding` projection used by `show`, the
finding index, `--json` and MCP" → "The `review_ledger::FindingView`
projection used by `show`, the census, `--json` and MCP".

Disclosure table — `review list` row → `review list`, `review findings`.

Write surface — family gains `findings`; MCP gains `review_findings`. The
`show` bullet → "**`show`** renders one `ReviewView`; see Read surface."

New `## Read surface` — the view, depth, selection, filter predicate,
default, findings census, tolerant scan (text as landed in SPEC-032).

Decisions — "**SL-278** (DEC-361..DEC-370): one engine-owned view for every
read; depth/selection/filter semantics; the census shares the show filter;
an unreadable ledger is disclosed, not skipped (STD-003), while the close
gate stays fail-closed."

Covers: RV-417 brief omission (design sec-6 obligation); no RV finding.
