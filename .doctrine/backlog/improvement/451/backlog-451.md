# IMP-451: concept-map export -X

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

SL-245 ships `--render` / `-X` on `doctrine graph` only. `concept-map export`
also emits DOT and should opt into the same `terminal_image` seam (IDE-046: one
mechanism the verbs opt into). Cut from SL-245 on 2026-09-15 to reduce scope.

## Already designed

SL-245's locked design covers it; read it rather than re-deriving:

- design sec-6 *CLI surface → concept-map export*: `format` becomes
  `Option<ExportFormat>` with `required_unless_present = "render"`; the
  dispatch arm resolves `format.unwrap_or(ExportFormat::Dot)`; `ExportFormat`
  gains `Display`; `run_export` takes `render` and follows `run_graph`'s
  prepare-then-write shape (DEC-258).
- design sec-8 VT e2e rows: `--format mermaid -X` refused as format-not-DOT;
  `-X` with no `--format` off a terminal gives the not-a-terminal message, not
  clap's missing-argument error. VH step 3.

## Notes

- The three existing `run_export` unit tests (`src/concept_map.rs`) call it
  directly and will need the added render argument.
- At SL-245 reconcile, ISS-242 was to be annotated that the flag joins the
  ungoverned concept-map surface; that annotation moves here.
