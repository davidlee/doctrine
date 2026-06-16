# Render requirement prose in spec show

## Context

Two closely related spec-surface improvements:

1. **IMP-037 — no `read_spec` reader.** `spec.rs` has multiple sites that inline-parse
   a Spec from TOML (`relation_edges`, `show`, `scan`). Each re-implements the
   same read → parse pattern. Extracting one reader for the simpler sites removes
   the duplication and gives future spec-adjacent work a single seam.

2. **IMP-058 — requirement prose invisible.** `doctrine spec show` renders
   each member requirement's structured TOML fields (title, kind, status,
   description, acceptance criteria), but never reads or renders the
   requirement's `.md` prose tier (Statement/Rationale). The `.md` file exists
   on disk but is invisible, leaving a spec's "readable whole" incomplete.

Bundling them is natural: the reader extraction (IMP-037) is the clean
foundation for the prose extension (IMP-058). A dedicated read seam avoids
adding yet another inline parse.

## Scope & Objectives

1. **Extract a `read_spec` reader** — one function that reads + parses a spec's
   TOML + prose from its directory, replacing the two simpler inline parse sites in
   `spec.rs` (`relation_edges`, `show`). Mirrors `read_slice`'s `(parsed, raw-toml,
   prose-body)` signature. `build_registry` keeps its inline parse (non-trivial
   `second_parent` error handling). Behaviour-preservation gate: existing suites
   stay green unchanged.

2. **Read the requirement `.md` prose body** — extend the requirement reader
   (or add a companion `load_with_prose`) so `spec show` can resolve both tiers
   for each member requirement. The `.md` path is the well-known sister file
   `requirement-NNN.md` beside the already-read `requirement-NNN.toml`.

3. **Render the prose in `spec show` table output** — each requirement's
   Statement/Rationale body renders below its structured fields, preserving the
   spec's existing layout. Empty scaffolds (all headings contain only comments)
   are detected and omitted — no noise for unfilled requirements.

4. **Include the prose in `spec show --json` output** — the JSON envelope gains
   a `body` field on each member requirement (absent when scaffold).

5. **Add `prose` column to `spec req list`** — `✓` for filled, `—` for scaffold.
   Added to default columns and `ReqJsonRow`.

## Non-Goals

- **No standalone `doctrine requirement show` verb.** Requirements remain
  spec-mediated in v1.
- **No requirement authoring changes.** Reading and rendering only (IMP-057 is
  separate).
- **No validation or correctness work.** Purely read-and-render; no new rules.

## Affected Surface

- `src/spec.rs` — `read_spec` extraction (2 call sites → 1), prose wiring in
  `render()` and `show_json()`, `prose` column in `spec req list`
- `src/requirement.rs` — prose reader extension (companion or return-type change)

## Summary

Two-phase: consolidate the spec reader, then add the requirement prose render.
Small surface, pure read path, no new authored fields.

## Follow-Ups

- IMP-057 — Requirement authoring/review skill to help agents fill the Statement
  and Rationale sections that this slice makes visible.
- Standalone `doctrine requirement show` — if requirements ever need a
  spec-independent inspection surface.
