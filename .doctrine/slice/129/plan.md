# Implementation Plan SL-129: Corpus-wide entity id→path helper (entity::id_path over KINDS)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases: add the helpers and field (PHASE-01), then remove the duplication
and replace all callers (PHASE-02). Strictly sequential — PHASE-02 depends on
the helpers existing.

## Sequencing & Rationale

**PHASE-01 first** because adding a field to a `const Kind { ... }` struct is
atomic — every initializer must be updated simultaneously or the build breaks.
The helpers (`id_path`, `rel_path`) land in the same commit so they're
immediately available. At this point `KindRef` and `GovKind` still carry their
own `stem`; the build compiles with the duplication.

**PHASE-02** removes the now-redundant `stem` fields from `KindRef` and
`GovKind`, updates KINDS rows and `g.stem` refs, and replaces every inline
`format!("<stem>-{name}.{ext}")` production site with `entity::id_path` or
`entity::rel_path`. The replacements are pure text substitution — every new
call produces the identical path the `format!` produced. Doing them all at once
avoids intermediate states where some files use the helper and others don't.

After SL-129 is closed, SL-115 can sequence with `doctrine link SL-115 after
SL-129` (already authored) and relocate the now-thinner shells without stale
id→path sites to manage.

## Notes

- `meta.rs` internals (`read_meta`, `read_id`) excluded — they already abstract
  path construction behind a `stem` parameter with a kind-root (not project-root)
  argument. Switching them to `entity::id_path` would require refactoring their
  callers to pass the project root instead of the kind root — out of scope.
- Test assertion full-path strings (~53 sites like
  `format!(".doctrine/slice/{id:03}/slice-{id:03}.toml")`) excluded — they are
  intentionally concrete for test failure readability.
- Sub-kinds (DESIGN_KIND, PLAN_KIND, NOTES_KIND) get `stem: ""` as a sentinel:
  `debug_assert!` in `id_path` catches accidental calls.
