# Implementation Plan SL-174: Prebuilt binary distribution

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases turning design.md (D1–D5) into a tag-triggered prebuilt-binary
pipeline. They are ordered by **dependency and risk**, not by user-facing
prominence: the embed gate the workflow depends on comes first; the workflow
(carrying the riskiest proof) second; the install channels that consume its
assets last.

## Sequencing & Rationale

**PHASE-01 (smoke gate) first** because it is both the dependency PHASE-02
consumes (the workflow calls `scripts/smoke.sh`) and the mitigation for the
dominant risk R1 (a binary shipping a broken embed). Crucially it is the one
piece provable **locally** — built against the jail's own Linux binary — so the
correctness mechanism is validated before any CI exists. VT-2 deliberately
proves the gate *fails* on a web-build-less binary; a gate that never catches the
fault it guards against is worthless.

**PHASE-02 (workflow) second** because it both depends on PHASE-01 and carries
the single riskiest step: x86_64 cross-compiled on the arm64 runner re-enters the
exact `-liconv` link-failure domain the slice exists to escape (R3/F1). It is
placed before the install channels so that the real artifacts — and the proof the
cross-link works — exist before anything is built to consume them. Its decisive
verification (VH-1) can only land on a real tag push; the native macos-13
fallback is pre-armed in case cross-link fails.

**PHASE-03 (channels + docs) last** because `install.sh` and the binstall
metadata consume the release assets PHASE-02 produces, and the docs describe
channels that must already exist. Docs ride with the channels they document
rather than a separate phase — they share the asset-naming contract (§5.2) and
must change together to avoid R2 drift.

## Notes

- Much of this slice's decisive verification is VH, not VT: there is no in-jail
  harness for "does a GitHub macOS runner build and link." The local VTs prove
  the *logic* (smoke checks, shell mapping, Cargo parse); the VHs prove the
  *real pipeline* on a tag push. This is inherent to a CI/release slice, not a
  planning gap.
- Asset names are a contract shared by three consumers (workflow, install.sh,
  binstall). Treat a rename as a breaking change edited in all three at once.
- SPEC-009 reconcile (binary delivery as new evergreen surface) is a
  close/reconcile-time follow-up, not a phase.
