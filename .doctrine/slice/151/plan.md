# Implementation Plan SL-151: Non-contiguous TOML sections cause opaque parse failures

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two changes, sequenced (design § Summary): a shared canonical-id parse wrapper
on the read seam (PHASE-01), then a proactive well-formedness check on the
`validate` seam (PHASE-02). The split follows a hard dependency — PHASE-02's
`scan_kind` calls `read_id`, whose signature PHASE-01 changes — and a clean
verification boundary (read-path diagnostics vs `validate` detection).

## Sequencing & Rationale

**PHASE-01 first — the signature-bearing change.** Adding `prefix` to
`read_meta`/`read_id`/`read_metas` is compile-forcing across the whole caller
surface; doing it first lets PHASE-02 build on a stable `read_id` signature. The
breadth is mechanical but wide and the compiler enforces completeness — the
authoritative caller list is the design's **Caller survey** (corrected against
grep in RV-155), not a module count. Two traps the survey already disarmed:
`lazyspec.rs:468` and `catalog/scan.rs:429` are *forced* `read_meta` callers
(not out-of-scope), and the real list funnel is `read_metas(stem)` — kind-generic
at `governance.rs:71`, so one site threads `g.kind.prefix` for every governance
kind. Prefix descends from callers because `meta.rs` sits below `integrity::KINDS`
(ADR-001) — it cannot derive the prefix itself.

**PHASE-02 second — the `validate` augmentation.** A single phase: `scan_kind`
gains a `diagnostics` out-param and a second, schema-agnostic `toml::Value` parse
(not `read_meta`, which hard-fails on status-less kinds like review). It is
reached only through `id_integrity_findings` → `validate`, never the catalog's
`scan_entities`, so there is no catalog performance regression.

**One phase each, not finer.** PHASE-01's caller threading cannot be sub-split
without leaving the tree non-compiling between steps; the compiler is the
completeness oracle, so it lands as one coherent unit. PHASE-02 is one focused
seam.

## Notes

- **Verification ids mirror the design VT table** (VT-1..VT-7, VA-1, VH-1) so a
  single vocabulary spans design → plan → execute; each id appears once across
  the slice.
- **Behaviour-preservation gate** (AGENTS.md): the read seam is shared entity
  machinery — existing suites prove it. Their assertions stay; only call
  signatures gain the `prefix` arg. Any assertion change is a red flag.
- **F-7 (tolerated, RV-155):** `parse_entity_toml`'s home — `dtoml.rs` vs the
  more cohesive `meta.rs`. Both are pure ADR-001 leaves; the call is the
  implementer's at PHASE-01. No correctness impact either way.
- **No string-matching on error text** (mem.pattern.parse.toml-error-
  classification-fragile): the wrapper adds context *around* the existing error
  with no conditional on its text. Zero version-fragility.
