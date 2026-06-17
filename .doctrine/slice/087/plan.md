# Implementation Plan SL-087: Boot snapshot token efficiency & correctness hardening

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

A single phase replaces the full-metadata memory table in the boot snapshot with
a compact reference instruction + key listing. The change is narrow — two files
(`src/memory.rs`, `src/boot.rs`) — and the existing test suite provides a strong
safety net against regressions.

## Sequencing & Rationale

**Single phase.** The change has no internal dependencies: `boot_keys()` is a
pure addition to `memory`, and the `produce()` arm update is a drop-in
replacement. Both can be implemented, tested, and verified in one pass. No
multi-phase staging is warranted — the old code path (`list_rows` with full
table render) is replaced atomically, not incrementally migrated.

TDD order within the phase:
1. Write the `boot_keys()` unit test (VT-1) — red.
2. Implement `boot_keys()` on `memory` — green.
3. Write the `produce(Memory, ...)` integration tests (VT-2, VT-3) — red.
4. Update the `SourceKind::Memories` arm in `produce()` — green.
5. Refactor, verify VT-4 (existing tests unchanged), then `just gate`.

The existing `produce_markers_a_non_exec_source_and_carries_the_exec_path` test
already validates that an empty Memory source → marker (VT-2 covers this by
construction — the test requires no change).

## Notes

- `boot_keys()` reuses `collect_all()` — no new filesystem read path (ADR-005,
  pure/imperative split).
- The `list_rows` function is not removed; it remains the CLI `memory list`
  backend. Only the boot producer call site changes.
- Uid fallback for keyless memories: `Memory.key` is `Option<String>`. The
  single keyless memory (`mem_019ecf85…`) renders its uid per the design's F-1
  remedy.
