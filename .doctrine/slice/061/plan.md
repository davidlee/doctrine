# Implementation Plan SL-061: Rewire /code-review and /inquisition onto the RV review ledger

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-061 rewires the two remaining unstructured review skills, `/code-review` and
`/inquisition`, onto ADR-007's RV ledger while extracting the already-proven
`/audit` mechanics into one shared `review-ledger.md`. The plan keeps the
keystone first: the shared protocol doc must absorb `/audit` losslessly before
the two new consumers depend on it.

The slice is intentionally light on production code. The only expected Rust
touch is `src/skills.rs` test/fixture fallout from moving `code-review` from the
`review` plugin into doctrine core. The work is therefore verified by existing
embed/install tests, `just gate`, and focused VA smokes of the skills' RV flows.

## Sequencing & Rationale

- **PHASE-01 first - prove the shared mechanics.** `/audit` is the pilot that
  already works on RV. Moving its inline mechanics into `review-ledger.md` is the
  behavior-preservation gate: if the shared doc cannot express `/audit`'s
  target, prime, raise, disposition, synthesis, close-gate, anti-escape, and
  phase-sheet harvest behavior, the DRY extraction is not legitimate. This phase
  also locks the ledger/prose trigger matrix discovered by the external
  inquisition, so later skills cannot route durable findings around RV.

- **PHASE-02 and PHASE-03 after PHASE-01.** `/code-review` and `/inquisition`
  both consume the shared protocol but are otherwise file-disjoint. They can be
  executed serially or in parallel after PHASE-01. PHASE-02 carries the plugin
  relocation and skill-discovery test fallout, so it owns the marketplace and
  `src/skills.rs` updates. PHASE-03 stays inside doctrine's inquisition skill
  and deliberately avoids a facet enum change: posture is carried by
  `--raiser inquisitor`, and the facet is chosen from the target aspect.

- **PHASE-04 last - refresh and reconcile ownership.** Re-embedding is last
  because it should happen after all SKILL.md edits settle. IMP-023 and the
  planned follow-up backlog items are closure cleanup, not prerequisites for the
  skill rewrites. The final smoke repeats all three review consumers against the
  shared doc so audit receives one coherent slice.

## Notes

- `install/review-ledger.md` is a top-level install asset; the installer copies
  embedded files except `manifest.toml`. PHASE-01 confirms whether the existing
  install asset coverage is sufficient or should gain a specific regression test.
- The prose fallback is not a second mode for normal review. It is a last-resort
  escape for explicitly throwaway one-shots with no durable subject and no
  durable findings. Any existing doctrine subject or durable diff must use or
  create an RV target.
- `inquisition.md` remains gitignored legacy for old runs, like `audit.md`; new
  closure-grade inquisitions use RV.
