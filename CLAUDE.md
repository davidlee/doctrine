@.doctrine/state/boot.md
If you have NOT seen `BOOT-SENTINEL: doctrine-governance-snapshot` anywhere in your context (system prompt or preceding messages), you MUST read the file referenced above now. If you HAVE seen it, you MUST NOT — the content is already in context.

@AGENTS.md

# Claude-specific

docs/claude has claude official docs cache - trust these, not
hallucination-ridden haiku summaries of web versions.

Use doctrine memory (the mcp tool). DON'T use claude built-in memory.

## Working with this user

- **Buffered replies.** This user's input can arrive buffered behind your tool
  use — a terse reply (`yes`, `no not yet`, a bare number) may answer a question
  from *several turns back*, not your latest. When a short reply doesn't cleanly
  match your last prompt, map it to the open question it best fits; if ambiguous,
  ask which. Avoid firing many questions ahead of pending tool calls.
- **Pasted source is ground truth.** When the user pastes docs, API params, or
  payload fields, treat it as authoritative over a subagent's answer or your own
  recollection — subagents hallucinate parameters. When they conflict, the pasted
  text wins immediately; correct course, don't defend the prior claim.
- **Clarifying questions: prose, not multiple-choice.** In `/design` (and other
  clarifying loops) present forks as prose with options + a recommendation and let
  the user reply free-text — they often reframe the question itself. Don't reach
  for the AskUserQuestion tool.

## Reviewer
- default reviewer: codex mcp — use default (GPT-5.5) for external adversarial reviews.
- Opus sub-agent is also useful for variety on subsequent passes.
