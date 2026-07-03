# IMP-252: doctor: suppress known-noise warnings (raw memory labels, reference-doc exemplars, runtime notes)

`doctrine doctor` currently produces ~495 warnings on a clean corpus, ~87% of which
are known noise. Three categories dominate:

## 1. Raw Label (374 warnings, 215 entities)

Every memory→entity edge carries `CatalogEdgeLabel::Raw` — memories use free-text
relation labels (`related`, `descends_from`, `superseded_by`) by design. The
`raw_label_findings` check flags every single one as a warning, flooding the
output with expected behaviour.

**Fix:** Either (a) suppress Raw edges from memories entirely (they're expected),
(b) report a single count line rather than per-edge, or (c) add a
`--skip-raw-labels` flag. Option (b) is the lowest-touch: one informational line
"N memory edges use raw labels" instead of 374 warnings.

## 2. Prose Citation — reference docs (3 warnings)

`.doctrine/glossary.md` uses `POL-NNN` and `STD-NNN` as exemplar placeholders
showing the reference form. These aren't real citations and should not be
flagged.

**Fix:** Skip prose citation scanning in `.doctrine/glossary.md` (and any other
known reference/install docs that use exemplar IDs).

## 3. Prose Citation — runtime/gitignored paths (1 warning)

`.doctrine/rfc/011/case-notes.md` mentions `RV-NNN` in a hypothetical scenario
("would collide with 16+ already-allocated RVs"). This is informal runtime
instrumentation, not an authoritative citation. The entire `.doctrine/rfc/` tree
is gitignored.

**Fix:** Skip prose citation scanning under `.doctrine/rfc/` (gitignored runtime
notes).

## Remaining noise (out of scope for now)

Closed-slice phase/design docs reference ASM-N, DEC-N, EVD-N, and placeholder
SL-NNN / REQ-NNN — in-phase tracking entities that were never materialized or
placeholder future-dependency refs. These are static historical documents; editing them is
make-work. A stretch goal: skip prose citation scanning in terminal-status slices.

## Acceptance criteria

- `doctrine doctor` on a clean corpus produces zero Raw Label warnings from
  memory edges (or a single informational count line)
- `doctrine doctor` on a clean corpus produces zero Prose Citation warnings from
  `glossary.md` and `rfc/` paths
- Existing doctor tests still pass; new tests cover the suppression paths
