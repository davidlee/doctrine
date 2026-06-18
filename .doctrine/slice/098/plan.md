# SL-098 — Implementation plan rationale

## Overview

SL-098 is a **skill-guidance slice, not a code slice**. No Rust changes, no
new entity machinery — the operations (`spec req add`, `doctrine link`, REV
`introduce`/`create`/`move`) already exist. The work is prose edits to five
skill files (the `/design → /plan → /audit → /reconcile → /close` reconcile
loop) that make implied-requirement discovery and canonical placement a natural
part of every pass. Five phases, one skill edit per phase, in data-flow order.

The redesign (post RV-078) settled the load-bearing decisions:

- **`design-requirements.toml` sidecar** is the authoritative TOML home for
  implied requirements (`[[implied]]` rows, REQ-DNN handles local to a design).
  `design.md` carries a prose reference only. This resolves F-2 (structured
  data in TOML, not prose) and F-8 (no rotting line anchors).
- **`plan.md` prose table**, not `plan.toml [requirements]`, carries the
  REQ-D → phase verification mapping. This resolves F-3/F-5 (no dead TOML
  fields; the v1 "stay empty" constraint is honoured, not contradicted).
- **Altitude ambiguity** routes to `/consult`; the full framework is deferred
  to IMP-097. This resolves F-4 (no building on sand).
- **Orphan section nests** under `### Governance/spec (REV)` as
  `#### Orphaned requirements (REV introduce)`. This resolves F-6.
- **Per-skill incremental walkthroughs** (§11a–§11e) are the phase verification
  targets. This resolves F-7.

## Skill-file baseline (the F-1 resolution, re-verified)

`plugins/doctrine/skills/*/SKILL.md` is the **authoritative master**. The
`.doctrine/skills/`, `.agents/skills/`, and `.dirge/skills/` trees are
gitignored install targets — editing them is lost work (confirmed: only
`plugins/` paths are `git ls-files`-tracked; the install trees are byte-identical
copies). All five phases edit the plugins masters. Prior SL-098 work that
touched `.dirge/skills/` was the root cause of RV-078 F-1 (a phantom duplicate
requirements pass); this plan re-verified the baseline before authoring.

## Scope — design §8 vs slice scope body

The slice scope body (slice-098.md) names seven skills including `/spec-product`
and `/spec-tech`. Design §8 — the authoritative "Skill file changes" table —
narrows to the five reconcile-loop skills. The spec skills are upstream of the
reconcile loop and are deferred to **IMP-096** (design §10). This plan follows
design §8 (the canonical design reference, per the `/plan` skill). The
scope-body drift is noted for separate reconciliation; it is not a blocker —
the design is canonical.

## Phase ordering

The phases follow the REQ-DNN data flow. Each phase edits one skill and is
verified by its corresponding design §11 walkthrough, runnable against the
artefact state produced by prior phases.

### PHASE-01 — /design (collect-decisions + requirements-pass + sidecar)

The foundation. `/design` *produces* `design-requirements.toml` and the
`## Implied Requirements` prose reference — every downstream skill reads this
format, so the producer must define it first. The phase also inserts the
"Collect decisions" state (batching the old serial "Ask clarifying questions")
and the "Requirements pass" state, plus the adversarial-review attack vector.

**Note on bundling:** "Collect decisions" is a UX restructure (batching
questions) that sits immediately upstream of "Requirements pass" (you survey
decisions for implied requirements). The design §3 presents them as a coherent
unit; this plan keeps them in one phase. RV-078 did not flag this as
out-of-scope.

Done first because PHASE-02 and PHASE-03 reference the sidecar format that
PHASE-01 canonicalises.

### PHASE-02 — /plan (requirements verification table)

Reads the sidecar, maps each REQ-DNN to verifying phase(s), records the mapping
in `plan.md` prose. Explicitly acknowledges `plan.toml [requirements]` stays
empty in v1 — the constraint the original design contradicted (F-5). The close
gate (PHASE-05) reads this table, so its shape must be defined before close.

Soft-depends on PHASE-01 (format canonical before a consumer references it).

### PHASE-03 — /audit (orphan survey + brief sub-section)

Reads the sidecar, surfaces still-orphaned REQ-DNNs in the reconciliation brief
nested under `Governance/spec (REV)`. The brief is the handoff surface to
`/reconcile`, so its orphan sub-section must exist before PHASE-04 can consume
it. Handles legacy designs (no sidecar) gracefully.

Soft-depends on PHASE-01 (sidecar format); hard-feeds PHASE-04 (brief shape).

### PHASE-04 — /reconcile (orphan placement workflow)

Consumes the brief's orphan sub-section (from PHASE-03), determines spec homes,
authors REV `introduce`/`create`/`move` rows, records REQ-DNN → REQ-NNN
mappings. Carries the `/consult` altitude guardrail (IMP-097 deferred). The
reconciliation outcome it produces is the check target for the close gate
(PHASE-05).

Hard-depends on PHASE-03 (brief orphan sub-section must be defined).

### PHASE-05 — /close (orphan deadlock gate)

The sole enforcement point. Reads `plan.md ## Requirements verification`
(PHASE-02's output shape) and the reconciliation outcome (PHASE-04's mappings);
refuses close on any unplaced orphan. The end-to-end §11e walkthrough —
including the negative case — is the final verification, plus a VH coherence
check that all five skills tell one lifecycle story.

Depends on PHASE-02 (read source) and PHASE-04 (check target).

## Verification posture

These are prose edits to skill files; there is no automated test that judges
"the requirements pass is correctly specified." Verification is therefore
**VA (agent)** — an agent reads each edited skill and confirms the section is
present, well-formed, and matches the design — with one **VH (human)**
end-to-end coherence check at PHASE-05. `just gate` is not a phase criterion
(no code changes); it remains a close-time hygiene check.

## Close-time deliverables (not phases)

- **Memories (design §9):** the REQ-D → REQ-NNN lifecycle pattern, the
  `design-requirements.toml` concept, and the REQ-DNN local-handle concept are
  recorded via `/record-memory` after the skill edits land (during /execute
  wrap-up or /close, not as a planned phase).
- **Signpost:** IMP-096 / IMP-097 cross-referenced from the affected skills.

## Risk

Low and decreasing across phases. PHASE-01 carries the most weight (it defines
the format everyone references and restructures the /design state machine); the
rest are additive sub-steps to skills that already work. The chief risk is
skill-prose drift between phases — a phrase introduced in PHASE-01 that
PHASE-03 silently contradicts — which the VH-1 end-to-end coherence check at
PHASE-05 is sized to catch.
