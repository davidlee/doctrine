# Implementation Plan SL-023: Ship knowledge tiers (ADR-005)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases realise ADR-005's three tiers (PUSH / PULL-reference / thin skills),
following `design.md` D9. Each phase is a shippable, independently-green unit; the
order is driven by **pointer dependency** — a tier-2 doc must exist before the PUSH
digest or a skill points at it.

## Sequencing & Rationale

1. **PHASE-01 — Ship the glossary.** First because it is the foundational
   pointer target and self-contained: relocate, drop the unshipped `entity-model`
   link, repair the dead `doc/glossary.md` path the shipped templates already carry,
   breadcrumb the frozen citations. Closes the oldest defect (authoritative-but-
   unshipped) and proves the embed path before anything points at it.

2. **PHASE-02 — Author `using-doctrine.md`.** The second pointer target. Depends on
   nothing structural, but ordered after the glossary so its outbound pointer
   (vocabulary → glossary) resolves to a shipped doc. Its payload is deliberately the
   *operator* surface (verbs, hand-editing, read-via-show) — disjoint from
   routing-process.md (workflow) and glossary (vocabulary).

3. **PHASE-03 — Push the reference-forms delta.** After both tier-2 docs exist, so
   the reference-docs pointer line names real, shipped targets. The block is rules
   only — the full tables stay in the glossary, keeping PUSH compact and avoiding a
   second full copy (the R-A5 drift guard). Pure asset edit; no `boot_sequence`
   change.

4. **PHASE-04 — De-dup the named skill sites.** Last, because every replacement is a
   *pointer* to a target authored in PHASE-02/03. Evidence-bound to the six sites
   named in `design.md` D6 — not a corpus sweep. The ratified restate line (R-OQ-4)
   is the acceptance bar: MAY name a verb / cite a rule by name; MUST NOT reproduce
   flag syntax, option tables, or storage-tier mechanics prose.

## Notes

- **rust-embed footgun (R-A1).** PHASE-01/02/03 edit embedded assets — a change is
  invisible until a full crate rebuild: `cargo clean -p doctrine && cargo build`,
  then `./target/debug/doctrine boot`. Verify against the rebuilt binary, never the
  stale PATH bin.
- **Verification split.** VT where mechanical (asset presence, install-plan step,
  boot-render assertion, flag-syntax grep guard); VA for judgement (unique payload,
  reachability, no `--help` reproduction). No VH — no human-only acceptance gate.
- **Scope discipline.** PHASE-04 touches only the six named sites; the MAY-permitted
  one-line pointers (`slice:37`, `plan:40`) and the canonical `slice new "<title>"`
  incantation (`slice:22`) are explicitly out.
