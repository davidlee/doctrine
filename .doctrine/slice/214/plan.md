# Implementation Plan SL-214: Knowledge authoring skill

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Docs-only slice: six markdown targets, zero Rust changes. Three phases mirror
the design's own load-bearing sequence — author the artefacts, make them
reachable, then prove the pipe by pushing the first real records through it.

## Sequencing & Rationale

**PHASE-01 (author)** produces everything that lives under `plugins/` — the
new skill plus the three D3 capture pointers — in one coherent editing pass,
because POL-002 review (VA-1) is one grep sweep over one file set. The two
sharp edges are front-loaded here: the YAML frontmatter colon footgun
(mem.pattern.skills.yaml-frontmatter-colons) lands exactly where OQ-1's
trigger-phrase description is written, and the POL-002 boundary means the
ADR-017 gating *teaching* ships as skill content without citing a repo-local
ADR id a fresh client cannot resolve.

**PHASE-02 (distribute + route)** exists as a separate phase because ADR-009
F14 makes install-before-routing-row an ordering constraint, not a detail:
the row must never point at a shipped-but-unreachable skill. The phase
boundary is the enforcement seam — PHASE-02 cannot start until the plugins/
content is committed, and within it the order is fixed (touch
`src/install.rs` → build → install → row → `using-doctrine.md` sentence →
boot). The design's original `src/skills.rs` premise went stale when IMP-226
consolidated into `install.rs`; design.md was reconciled before this plan was
authored.

**PHASE-03 (dogfood)** runs last because it *consumes* the installed skill —
authoring the first DEC/ASM records through the routed surface is the only
validation that touches the chicken-egg risk named in the slice scope. Kept
deliberately small (one DEC, one ASM, per adversarial finding 2): the point is
teaching-by-example and a non-zero census, not volume. Full population
pressure is SL-215's harvest side, out of scope here by explicit boundary.

## Notes

- VT mandates in PHASE-01/02 are keyword floors over the authored markdown —
  proportionate for a docs slice; judgment criteria (POL-002 cleanliness,
  record well-formedness) stay VA, sequencing confirmation VH.
- PHASE-03's records are `.doctrine/` authored entities created by CLI verbs
  at run time — file paths and record ids are not predictable at plan time, so
  it carries no VT mandate; VA-1/VA-2 do the checking.
- Execution posture: solo (`/execute`), not `/dispatch` — PHASE-02 and
  PHASE-03 both write `.doctrine/` authored + runtime state, which dispatch
  workers must not touch.
