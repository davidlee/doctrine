# Implementation Plan SL-234: Review prime ignores non-file selector entries

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-234 is one atomic repair: retain Git entry type while expanding review
selectors, then exclude entries that cannot represent hashable tracked file
content. The change and its regression share the existing private review seam.

## Sequencing & Rationale

One phase is sufficient because no public interface, persisted schema, or
cross-module contract changes. The phase begins red with a real Git repository
containing a committed slug-style symlink to a directory. It then replaces the
path-only listing with staged, NUL-delimited index records, filters to regular
blob modes, and finishes by exercising RV-315 itself.

Keeping the parser and test in `src/review.rs` avoids a new module and preserves
ADR-001's existing `review` command → `contentset` leaf dependency. The
content-set implementation is deliberately not edited; its unchanged tests are
behaviour-preservation evidence.

## Notes

If staged Git output cannot be parsed without broadening the change beyond the
private review seam, stop and return to SL-234 design rather than teaching the
shared content-set leaf to ignore invalid inputs.
