# IMP-130: Land the RV-116 operator guard in SPEC-021 / close+audit skills: check review_surface candidate drift before /close --source

## Motivation
SPEC-022 (REQ-317) states the *substrate* fact: a repair on a `review_surface` candidate
does not auto-propagate to trunk — integration consumes the admitted `close_target` OID.
But the operator-facing **enforcement** has no home. Both review passes on SPEC-022 (the
web-gpt review's F2 and codex's finding-4) agreed a guard is needed, and codex was right
that it does not belong in the substrate spec — it is process (SPEC-021 / the close+audit
skills).

## Scope
A normative check the orchestrator/agent applies before creating or admitting a
`close_target`: determine whether the reviewed `review_surface` candidate has drifted
from its recorded `merge_oid` or carries repair commits (the `candidate status` drift
report is the mechanical signal). If it has, `/close` MUST NOT use
`--source refs/heads/review/<N>` unless the omission is deliberate and documented —
otherwise the repair is silently dropped from the trunk payload. Land it in SPEC-021
(process) and/or the `close` + `audit` SKILL.md guidance.

## Links
Born from the SPEC-022 review (IMP-128). Resolves the enforcement half of RV-116; the
substrate half is REQ-317 / SPEC-022 D1. Related: SL-068 (candidate layer), SPEC-021.
