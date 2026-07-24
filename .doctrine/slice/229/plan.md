# Implementation Plan SL-229: Pre-design research stage v1

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases, code-first: the verb before the skill that names it, the skill
before the hooks that point at it. Each phase is independently landable and
leaves the tree green.

## Sequencing & Rationale

- **PHASE-01 (Rust)** goes first because both prose phases reference the verb;
  authoring the skill against a verb that doesn't parse yet would put the
  contract ahead of the surface. TDD red/green/refactor on the engine leaf;
  the CLI variant and guard arm ride the same phase because they are one
  feature (design D2) and the wiring pattern is already ✓-verified
  (research.md § coverage-verb wiring). The `is_stale_against` decision
  (consume vs remove, design § engine) is resolved here — the suppression
  does not survive the phase either way, and absence-of-suppression is
  checked by VA-1 since a keyword mandate cannot assert absence.
- **PHASE-02 (contract)** authors the `/research` skill as the single
  conventions surface plus the two install-side sockets. Separated from
  PHASE-03 so the contract exists and is reviewable (VA-1 against design §
  artefact contract) before four other skills start pointing at it.
- **PHASE-03 (hooks + ritual)** is last and smallest: four 1–3 line advisory
  edits, then the embed ritual — which is its own phase-exit concern because
  a bare `cargo build` after `plugins/`-only edits is a silent no-op
  (mem.pattern.distribution.skill-refresh-command); the ritual is
  `touch src/install.rs && cargo build && ./target/debug/doctrine install
  -s <id> -y` per touched skill.

Dogfood note: this slice's own research round already exists
(`research/research.md`, gitignored) — it is the design's evidence base and
the model for the PHASE-02 skeleton. The baseline stamped there was
hand-written before the verb existed; PHASE-01's e2e check can `--restamp` it
as a live fixture.

## Notes

- Behaviour-preservation gate: `contentset.rs` gains a consumer; the existing
  contentset + review warm-cache suites must stay green **unchanged**.
- Closure evidence (slice-level, not phase-level): one further real slice
  driven through the round, observations to RFC-011 case-notes
  (slice-229.md § Verification).
- Escalation boundary: any temptation to make the advisory blocking is an
  ADR conversation, not a skill tweak (design D6, ADR-003).
