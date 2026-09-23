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
- **Pasted source beats summaries.** When the user pastes docs, API params, or
  payload fields, trust that it is what they say it is, and rank it above a
  subagent's answer, a web-search summary, or your own recollection — those are
  where hallucinated parameters come from. On conflict, correct course to the
  pasted text immediately; don't defend the prior claim. It can still be stale
  or partial: if it conflicts with the installed binary or another primary
  source, say so rather than silently choosing.
- **Clarifying questions: prose, not multiple-choice.** In `/design` (and other
  clarifying loops) present forks as prose with options + a recommendation and let
  the user reply free-text — they often reframe the question itself. Don't reach
  for the AskUserQuestion tool.

## Reviewer
- default reviewer: **codex-cli MCP** (`mcp__codex-cli__codex` / `…__review`) — a
  third-party npm wrapper (`npx -y codex-mcp-server`) that shells the codex CLI.
- the old `codex` MCP server entry is **dead** — codex-cli 0.154.0 removed the
  `mcp-server` subcommand, so it fails `CONNECTION_CLOSED` on every reconnect.
  Nothing to repoint it at; use codex-cli. (Verified 2026-09-14.)
- **always pass `model`** — default to `gpt-6-sol`, medium or high effort. The wrapper's own default
  (`gpt-5.3-codex`) 400s on a ChatGPT account, and its model enum is stale; ask
  `codex debug models` for what the account actually serves.
- **never `reasoningEffort: "minimal"`** — codex attaches `web_search`
  unconditionally and the API rejects that pairing. `low`+ works; use `high` for
  adversarial review.
- pass `sandbox: "workspace-write"` so the reviewer raises onto the RV ledger
  itself; readonly isolation prevents an inquisitorial ledger and GPT's adherence
  makes that a poor tradeoff.
- Opus sub-agent is also useful for variety on subsequent passes.

## Research
- DON'T use subagents 
- do use `./scripts/pi-scout` (quicker, cheaper) or `./scripts/pi-research`
  (smarter) usage: takes a prompt via stdin or arg; returns results on stdout.
- note: outside the jail (i.e pwd is not in `/workspace/doctrine`) these agents
  have no API keys in the environment; pass prompts to the user to run for you.

