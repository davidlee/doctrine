@.doctrine/state/boot.md
If you have NOT seen `BOOT-SENTINEL: doctrine-governance-snapshot` anywhere in your context (system prompt or preceding messages), you MUST read the file referenced above now. If you HAVE seen it, you MUST NOT — the content is already in context. If that file does not exist, doctrine is not set up in this checkout: tell the user to install doctrine and run `doctrine install`.

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
- default reviewer: Codex via the **`codex:ask`** skill (David's fork of the
  official Codex plugin). Model tier and effort come from the skill and the GPT
  tiers in the global CLAUDE.md; don't pin model names here.
- pass `--write` so the reviewer raises onto the RV ledger itself; readonly
  isolation prevents an inquisitorial ledger and GPT's adherence makes that a
  poor tradeoff.
- **never `--effort minimal`**: current models reject it (400). `low`+ works;
  use `high` for adversarial review.
- Opus sub-agent is also useful for variety on subsequent passes.

## Delegating to pi (`pi_agent`)

The quotient MCP server's `pi_agent` tool runs one pi agent under a confining
form, on a cheaper model. It is faster and cheaper than a Claude subagent and
quite capable, but looser: lower instruction adherence. Prefer it over Claude
subagents for bounded work; keep judgement and acceptance for yourself.

| profile | capability | use for |
|---|---|---|
| `read` | read-only, no thinking | recon: code maps, call sites, "where is X" |
| `research` | read-only, low thinking | judgement over the tree: governance applicability, tradeoffs, first-pass review |
| `write` | read/write | mechanical edits: renames, fixture updates, boilerplate, doc sweeps |

- **Always pass `cwd`** (the repo root, `/home/david/dev/doctrine`). This project's quotient
  server is launched outside the repo (IMP-522), so the default cwd is not an
  allowed root. Its forms carry the Rust toolchain; a `write` call may root at
  the repo root.
- **Prompts.** Self-contained: name files, the exact question, and the output
  shape. Ask for batched reads and a capped answer — the policy ceiling is a
  handful of requests, and a one-file-per-turn delegate runs out before it answers.
- **Results.** Trust `result` only when `final` is true; otherwise read `state`,
  `causes`, `transcript`. A `refused` run did nothing — fix the call, don't retry it.
  Continue a conversation by passing back `session`.
- **Verify, don't adopt.** Treat output as tier-5 evidence. Spot-check cited
  paths and claims; review every `write` diff (`git diff`) before building on
  it, and run the gate yourself. Never delegate decisions, ledger dispositions,
  or anything that records a user act.
- **Independent threads in parallel:** issue several `pi_agent` calls in one turn.

### `/research` round

Claude's mechanism for the `/research` round — see `.doctrine/governance.md`
§Research agents for the contract it must satisfy. Don't use Claude subagents
for research here: one `pi_agent` call per thread (`read` for code-map threads,
`research` for judgement threads). The delegate is read-only, so write its
`result` to `.doctrine/slice/NNN/research/raw/<thread>.md` yourself, verbatim.

`./scripts/pi-scout` / `./scripts/pi-research` are the superseded shell route;
use them only if quotient is unavailable.

