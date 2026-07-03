# IMP-239: Onboard --model copy: reflect repeatable trait-set contract

## Context

SL-192 made `--model` repeatable (conjunctive trait-set targeting). The agent-
facing onboard copy still describes it single-valued:

- `src/mcp_server/tools.rs:1222` — `PROMPT_RESOLVE_MODEL_CMD = "doctrine prompt
  resolve --band model --model <id>"`.

Understates (not wrong) the new contract: multiple `--model` occurrences compose
a context trait set (membership), and a selector's pinned set is a conjunction
(intersection targeting). The engine change deliberately did not touch onboard
wiring — that surface is SL-187's (delivery), fenced as a non-goal in SL-192
design §7.

## Ask

Update the onboard/model-band guidance to reflect repeatable `--model` (compose
multiple trait keys). `prompt model-keys` needs no change — it reflects authored
model-band labels, which is its contract.

## Provenance

SL-192 audit RV-238 F-2 (follow-up disposition). Design §7 tracked follow-up.
