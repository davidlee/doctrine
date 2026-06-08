# Implementation Plan SL-024: Harden TOML render: escape user free-text through a shared seam

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See doc/glossary.md § reference forms. -->

## Overview

Two phases, ordered by the seam they share. PHASE-01 *creates* the escaping
seam (the `tomlfmt` leaf) and proves it inert by making `memory.rs` its first
consumer with byte-identical output. PHASE-02 *spreads* the seam across the
remaining five renderers and seven templates, driven red→green by adversarial
round-trip tests. The split is the behaviour-preservation gate made physical:
phase one changes the seam's *home* without changing any output; phase two
changes *output*, but only for inputs that were previously broken.

## Sequencing & Rationale

**Why PHASE-01 before PHASE-02 (seam before consumers).** The design's central
guarantee is the *verbatim move* (design D1): the escaper bodies leave
`memory.rs` byte-for-byte, so memory's output cannot change and its suite is the
proof. Establishing the leaf and re-pointing memory *first*, in isolation, lets
that inertia be verified against memory's existing suite alone — before any other
module is in motion. If PHASE-01 is green, the seam is trustworthy; PHASE-02 then
builds on solid ground rather than moving the seam and routing new consumers in
one indivisible churn where a failure has two possible causes.

**Why PHASE-02 is one phase, not five.** The five renderer edits are mechanically
identical (route `title`+`slug` through `toml_string`; drop the template's outer
quotes — design D3 self-quoting). Splitting per module would multiply ceremony
without adding an independent decision point; each renderer instead carries its
own adversarial round-trip test as its slice of the exit evidence. The phase is
cohesive: one convention applied uniformly, one corpus-wide invariant restored.

**The lockstep edit is the phase's sharp edge** (design R1). Template and renderer
change as a pair: removing the renderer's raw splice while leaving the template's
`"{{title}}"` quotes yields `""value""`; the reverse yields a bare unquoted value.
The per-renderer round-trip test — a *real* value must re-parse and round-trip —
catches both halves of a half-applied edit.

**Spec is the deliberate odd one out** (inquisition.md Charge 1). Four modules
(adr, slice, requirement, backlog) already own a direct `render_*_toml` →
`toml::from_str` round-trip test; PHASE-02 extends those four with the hostile
input. Spec has *no* such direct test — its only round-trip reads from disk via
`fresh`/`read_meta`. PHASE-02 therefore authors a **new** spec test that calls
the private `render_spec_toml` directly. This is not an oversight to paper over:
routing a hostile `--slug` through spec's disk path would strike `<id>-<slug>`
symlink creation *before* the TOML round-trip — a false-red from the wrong
stratum. The direct call is mandatory, not stylistic.

**The deferred boundaries are conscious.** `render_*_md` body splices stay raw —
markdown is free-form prose, never structurally parsed (the storage rule). The
runtime phase sheet's `{{name}}` splice (`state.rs:336`, design OQ-2) and explicit
`--slug` normalisation (design OQ-1) are orthogonal follow-ups, not folded in.

## Notes

The rust-embed re-embed footgun applies to every template edit in PHASE-02: a
lone `install/templates/*.toml` change is invisible until the embedding crate
recompiles — `touch src/install.rs && cargo build` after editing templates, and
the round-trip tests (which run against the *embedded* asset) will catch a stale
embed if forgotten.
