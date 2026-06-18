# SL-098 — Implementation plan rationale

## Overview

SL-098 is a **skill-guidance slice, not a code slice**. No Rust changes, no
new entity machinery — the operations (`spec req add`, `doctrine link`, REV
`introduce`/`create`/`move`) already exist. The work is prose edits to five
skill files (the `/design → /plan → /audit → /reconcile → /close` reconcile
loop) that make implied-requirement discovery and canonical placement a natural
part of every pass. Five phases, one skill edit per phase, in data-flow order.

This is the **second pass** of the plan. The first pass survived a post-plan
technical review (no ledger raised) that found one substantive fact-check
failure (B1: multi-spec CLI semantics), two cohesion problems (C1: orphan
placement duplicating REV step 4; C2: no-op gate), a storage-rule overclaim
(D1), and several exit-criteria gaps (E1/E3/E2) plus verification thinness (F1).
The design was amended to resolve all of them; this plan mirrors the amended
design. Finding-to-section map: B1/B2/C1/C2/D1/E2/F2 → design §3/§4/§6/§7; E1/
E3/F1/G1/G2/G3/G4 are pure-plan fixes folded in below.

## Key decisions (from the amended design)

- **`design-requirements.toml` sidecar** is the authoritative TOML home for
  implied requirements (`[[implied]]` rows, REQ-DNN handles local to a design).
  `design.md` carries a prose reference only. (RV-078 F-2, F-8.)
- **`plan.md` prose list** — not a pipe-table, not `plan.toml [requirements]` —
  carries the REQ-D → phase verification mapping. This is a **trade, not a
  resolution** of F-3 (D1): the mapping is authored and agent-read, which is the
  category F-3 flagged, but it has no v1 TOML home and crossing the design/plan
  ownership boundary to put it in the sidecar is worse. The plan skill step 5
  prohibition is amended to name this as a permitted exception. When the
  requirement registry lands, the mapping graduates to `plan.toml`.
- **Orphan placement lives at `/reconcile` step 4f**, not 2b (C1). It *is* REV
  authoring, so it sits inside the existing step 4 (4a–4e) and reuses the
  discover/collision/narrative/split machinery.
- **No-op gate amended** (C2): orphan entries count as brief content; the gate
  does not fire while any orphan is unplaced.
- **Multi-spec placement = sibling REQ-NNNs with traced lineage** (B1), not one
  REQ shared across specs — the CLI mints a new REQ per `introduce` and no
  re-member verb exists (verified empirically: no REQ appears in >1 spec). The
  genuine multi-membership question is deferred to IMP-096.
- **Stuck is non-terminal** (E2): close refuses a stuck orphan and returns to
  reconcile. Withdrawn is a design-level retraction (edit the sidecar + record
  in the narrative) — orphans are not RV findings, so "withdrawn" is not the
  finding disposition.
- **Close check is advisory** (F2): no binary refuses close on an unplaced
  orphan; the §11e walkthrough is the backstop. The real-teeth fix (orphan
  status riding the RV ledger) is filed as a follow-up IMP.
- **Collect-decisions ≠ clarifying-questions** (B2): distinct activities, both
  kept. Clarification is exploratory (one at a time); collect-decisions is
  decision-confirmation (batched).

## Skill-file baseline (the F-1 resolution, re-verified)

`plugins/doctrine/skills/*/SKILL.md` is the **authoritative master**. The
`.doctrine/skills/`, `.agents/skills/`, and `.dirge/skills/` trees are
gitignored install targets — editing them is lost work (confirmed: only
`plugins/` paths are `git ls-files`-tracked; the install trees are byte-identical
copies). All five phases edit the plugins masters. Prior SL-098 work that
touched `.dirge/skills/` was the root cause of RV-078 F-1 (a phantom duplicate
requirements pass); this plan re-verified the baseline before authoring.

## Scope (G1)

The slice scope body (`slice-098.md`) names five skills — `/spec-product` and
`/spec-tech` are deferred to **IMP-096** (design §10), being upstream of the
reconcile loop. (The scope body was amended in this pass to match — previously
it named seven skills, a drift the first plan flagged.) This plan follows
design §8.

## No dogfooding (G2)

SL-098 is a skill-guidance slice with no implied requirements of its own, so the
§11a–§11e walkthroughs run against hypothetical artefact state, not SL-098's
own. This is inherent to the slice kind and bounds verification confidence —
hence the two VH coherence checks (F1) rather than relying on VA alone.

## Phase ordering

The phases follow the REQ-DNN data flow. Each phase edits one skill and is
verified by its corresponding design §11 walkthrough, runnable against the
artefact state produced by prior phases.

### PHASE-01 — /design (collect-decisions + requirements-pass + sidecar)

The foundation. `/design` *produces* `design-requirements.toml` and the
`## Implied Requirements` prose reference — every downstream skill reads this
format, so the producer must define it first. Inserts "Collect decisions"
(state 2) and "Requirements pass" (state 4) around the renumbered "Ask
clarifying questions" (state 3, unchanged), plus the adversarial-review attack
vector. **Both** the numbered list and the `<Process State Machine>` XML block
are updated in sync (E1). Rewords the collect-decisions "read the design doc"
instruction to handle greenfield (G3).

Done first because PHASE-02 and PHASE-03 reference the sidecar format that
PHASE-01 canonicalises.

### PHASE-02 — /plan (requirements verification narrative)

Reads the sidecar, maps each REQ-DNN to verifying phase(s), records the mapping
in `plan.md` as a **prose list** (not a table — D1). Acknowledges
`plan.toml [requirements]` stays empty in v1. Amends step 5's storage-rule
prohibition to permit authored, agent-read verification narrative. The close
check (PHASE-05) reads this list, so its shape must be defined before close.

Soft-depends on PHASE-01 (format canonical before a consumer references it).

### PHASE-03 — /audit (orphan survey + brief sub-section)

Reads the sidecar, surfaces still-orphaned REQ-DNNs in the reconciliation brief
nested under `Governance/spec (REV)` as `#### Orphaned requirements (REV
introduce)` (F-6). Survey output is held for brief-writing at step 5 (G4). The
brief is the handoff surface to `/reconcile`, so its orphan sub-section must
exist before PHASE-04 can consume it. Handles legacy designs (no sidecar)
gracefully.

Soft-depends on PHASE-01 (sidecar format); hard-feeds PHASE-04 (brief shape).
**Carries the intermediate VH-1 coherence check** (F1): once design→plan→audit
are all written, a human confirms no drift introduced in PHASE-01 has
propagated — catching it at half the cost rather than letting it travel through
two more phases.

### PHASE-04 — /reconcile (no-op gate amendment + orphan placement at 4f)

Amends the no-op gate so orphan content counts (C2). Adds sub-step 4f (orphan
placement) inside the existing REV-authoring step 4 (C1) — not at 2b. Consumes
the brief's orphan sub-section (from PHASE-03), determines spec homes, authors
REV `introduce`/`create`/`move` rows, records REQ-DNN → REQ-NNN mappings.
Multi-spec placement = sibling REQ-NNNs with traced lineage (B1). Defines
stuck (non-terminal) and withdrawn (design-level retraction) outcomes (E2).
Carries the `/consult` altitude guardrail (IMP-097 deferred). The
reconciliation outcome it produces is the check target for close (PHASE-05).

Hard-depends on PHASE-03 (brief orphan sub-section) and VH-1 (coherence so far).

### PHASE-05 — /close (orphan advisory check with read-path)

The advisory enforcement point. Reads `plan.md ## Requirements verification`
(PHASE-02's output) and follows the read-path `review-NNN.md → revision-NNN.md`
(E3) to the reconciliation outcome (PHASE-04's mappings); refuses close on any
stuck or absent orphan (E2). Named honestly as advisory, not binary-enforced
(F2). The end-to-end §11e walkthrough — including the negative case — is the
final verification, plus a VH-1 coherence check that all five skills tell one
lifecycle story.

Depends on PHASE-02 (read source) and PHASE-04 (check target).

## Verification posture

These are prose edits to skill files; there is no automated test that judges
"the requirements pass is correctly specified." Verification is therefore
**VA (agent)** — an agent reads each edited skill and confirms the section is
present, well-formed, and matches the design — with **two VH (human) coherence
checks** (F1): an intermediate one after PHASE-03 (catches drift at half cost)
and a final end-to-end one at PHASE-05. `just gate` is not a phase criterion
(no code changes); it remains a close-time hygiene check.

## Close-time deliverables (not phases)

- **Memories (design §9):** the REQ-D → REQ-NNN lifecycle pattern, the
  `design-requirements.toml` concept, and the REQ-DNN local-handle concept are
  recorded via `/record-memory` after the skill edits land (during /execute
  wrap-up or /close, not as a planned phase).
- **Signpost:** IMP-096 / IMP-097 cross-referenced from the affected skills.
- **Follow-up IMP (F2):** orphan status riding the RV ledger for real
  close-gate enforcement — filed during /close.

## Risk

Low and decreasing across phases. PHASE-01 carries the most weight (it defines
the format everyone references, restructures the /design state machine, and must
keep two state representations in sync); the rest are additive sub-steps to
skills that already work. The chief risk is skill-prose drift between phases —
a phrase introduced in PHASE-01 that PHASE-03 silently contradicts — which the
intermediate VH-1 at PHASE-03 and the final VH-1 at PHASE-05 are sized to catch
(F1).
