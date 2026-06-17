# Review RV-056 — reconciliation of SL-084

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface:** candidate/084/review-001 (3-way merge of review/084 onto main)

**Lines of attack:**

1. **Pi dispatch-worker agent definition (`.pi/agents/dispatch-worker.md`)** —
   does it carry the full worker contract (source-only mutation, single
   non-merge commit, verify-before-commit, structured report, no `.doctrine/`
   writes)? Are pi tool names correct (`read, edit, write, bash`)? Is
   DOCTRINE_WORKER prefix instruction present (not persistent export)? Is the
   model field `deepseek/deepseek-v4-pro` present?

2. **Router harness detection (`dispatch/SKILL.md`)** — is the routing table
   three-way (pi, codex, claude)? Do the detection markers match design D2
   (PI_HOME for pi, CLAUDECODE for claude, self-belief for codex)? Is the
   mismatch/unknown refusal prose present? Does "codex/pi" lump remain anywhere?

3. **Harness→spawn table (`dispatch-subprocess/SKILL.md`)** — does the table
   have pi and codex rows with correct spawn mechanisms? Is pi-subagents
   extension detection documented with refuse-on-missing? Is codex row marked
   as legacy placeholder/untested? Are existing spawn variants preserved?

4. **Cross-reference integrity** — do dispatch↔dispatch-subprocess,
   dispatch↔dispatch-agent cross-references resolve? Are YAML frontmatters
   valid?

5. **Design conformance** — does implementation match design D1 (table within
   dispatch-subprocess), D2 (three-way detection), D3 (extension detection),
   D4 (agent-def model field), D5 (agent-def contract shape), D6 (pi spawn
   via subagent with cwd binding)?

## Synthesis

**Verdict: clean — no findings.** All four phases conform to design.md and
plan.toml. 3 files changed (95 insertions, 20 deletions), all confined to the
authored scope.

**Evidence by line of attack:**

1. **Pi agent-def** — `.pi/agents/dispatch-worker.md` carries the full worker
   contract (5 obligations), pi-native tool set (`read, edit, write, bash`),
   DOCTRINE_WORKER prefix instruction (not persistent export), and direct
   `model: deepseek/deepseek-v4-pro` field. Matches design D5 exactly.

2. **Router detection** — Routing table split into three explicit rows (pi,
   codex, claude). Detection prose includes env-marker table (PI_HOME, CLAUDECODE,
   codex=unknown) with detection order. Mismatch and unknown refusal prose both
   present. No residual "codex/pi" lump anywhere. Quick Reference updated.

3. **Spawn table** — `dispatch-subprocess/SKILL.md` has harness→spawn table with
   pi row (`subagent(agent="dispatch-worker", task, cwd)`) and codex row (`codex
   exec`, legacy placeholder, untested). D3 pi-subagents extension detection with
   refuse-on-missing text. Existing spawn variants (unconfined `env -C`, confined
   bwrap) preserved; bwrap note updated to "codex only".

4. **Cross-references** — dispatch→dispatch-subprocess, dispatch→dispatch-agent
   paths resolve. YAML frontmatters valid (verified by `just check` on
   coordination tree: 1589 passed, 0 failed).

5. **Design conformance** — all six design decisions implemented faithfully:
   D1 (table in dispatch-subprocess), D2 (three-way detection with env
   cross-check), D3 (extension detection + refuse), D4 (direct model: field),
   D5 (contract shape), D6 (pi subagent spawn with cwd binding).

**Standing risks:**

- **RSK-4 (pi subagent no bwrap confinement):** pi workers inherit orchestrator
  filesystem permissions. Acceptable — same posture as Claude arm; marker is
  primary identity; R-5 belt is real protection. Documented in skill.
- **RSK-5 (fork branch IS the phase ref):** `gc --force` before
  `dispatch sync --prepare-review` would destroy deliverables. Documented in
  harness-table residual column.
- **Codex arm untested:** codex `codex exec` row marked as legacy placeholder;
  env-marker characterization deferred to codex spike. Explicit non-goal.

**Tradeoffs consciously accepted:**

- Pi self-arm is prompt-only (no env parameter on `subagent` tool). Same as
  Claude arm; proved in operation.
- No installer templating for model — direct `model:` field in agent-def YAML.
  Deferred to future unified `doctrine install` verb.
- PHASE-04 verification ran `doctrine claude install` from main tree (old
  source). Installed skills match current trunk; will match new source after
  integration. Not a defect — a lifecycle artifact.

## Reconciliation Brief

No governance or spec findings. The implementation matches the design exactly.
All phase criteria (EN/EX/VT) satisfied. No REV required. No per-slice edits
needed beyond the already-committed source changes.

## Reconciliation Outcome

**No-op reconcile.** All findings were aligned (implementation matches design
and plan exactly). No governance/spec items to write. No per-slice edits needed.
No REV required. Reconcile pass complete — handoff to /close.
