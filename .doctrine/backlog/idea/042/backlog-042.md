# IDE-042: Model-tier dispatch worker prompts via hymns cascade

## Problem

Dispatch worker prompts are currently one-size-fits-all — the same contract
ships to every model regardless of adherence tier. High-adherence models
(Opus, GPT-5.x) don't need defensive guardrails like "never delete tests
without authorization" — they follow the contract. Lower-adherence models
(DeepSeek, smaller models) benefit from more explicit, repetitive, defensive
prompting.

Loading Opus with guardrails it doesn't need wastes tokens and context
window. Sending DeepSeek a lean contract invites the SL-222 PHASE-09 failure
mode (worker stubbed logic, deleted tests, reported green).

## Mechanism

The hymns cascade (`doctrine prompt resolve --band model --model <id> --role
worker`) already supports model-band prompt supplements. Currently
unused for the worker role — the hymn corpus has no model-tier worker hymns.

This card proposes authoring model-tier worker hymns that layer defensive
guardrails onto the base worker contract. The base contract stays lean (Opus
tier). Lower-tier models get a supplement that adds:
- Explicit "never delete tests without orchestrator authorization"
- Stronger deviation-reporting mandate ("any skipped test, stubbed logic, or
  triage mismatch IS a deviation — report it")
- Adversarial self-check step ("re-read the design's expected outcomes; does
  your output match?")
- Repetition of key constraints at multiple points in the prompt

## Related

- IMP-042 (code-review as worker-integrity gate) — complementary defence
- SL-222 PHASE-09 — canonical example of the failure mode
- `install/hymns/` — hymns corpus structure
- `doctrine prompt resolve` — the resolution mechanism
