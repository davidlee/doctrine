# ISS-265: Research tool guidance conflates web and repo agents

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`CLAUDE.md` § Research frames the two research entry points as one job at two
speeds:

> do use `./scripts/pi-scout` (quicker, cheaper) or `./scripts/pi-research`
> (smarter)

They are two different jobs, distinguished by *domain*, not by capability tier:

- `.pi/agents/scout.md` — the **repo** agent. Tools: `read, grep, find, ls,
  bash, write`. Model `deepseek/deepseek-v4-flash`, `--think off` by default.
- `.pi/agents/researcher.md` — the **web** agent. Tools: `read, write,
  web_search, fetch_content, get_search_content` (no `grep`, no `find`, no
  `bash`). System prompt: *"conduct thorough web research … break the question
  into 2-4 searchable facets … search with `web_search`"*. Model
  `deepseek/deepseek-v4-pro`, `--think low`.

## Impact

An agent following the documented cue — "this thread needs judgement, so use the
smarter one" — dispatches a **repo-internal** question to a **web** researcher.
During SL-233 plan-grounding research (2026-07-27) three of six threads were
misrouted this way (spec-descent boundary, RFC-021 crossover, review-ledger
structural analogue). They ran ~20 minutes web-searching for answers that exist
only in this tree, and had to be killed and re-fired on `pi-scout --think high`.

The failure mode is **silent**, which is what makes it worth fixing rather than
just knowing. `pi-research` can `read` any path it is handed, so it does not
error — it returns plausible prose sourced from the open web. Only elapsed time
and `ps` output revealed the misrouting. An agent that trusted the framing would
have folded web-sourced guesses about this repo's internals into an
implementation plan.

## Suggested fix

1. Rewrite the `CLAUDE.md` § Research lines to split by domain, not by "smarter":
   `pi-scout` for anything answerable from this tree (escalate hard questions
   with `--think high|max`, not by switching tools); `pi-research` **only** for
   questions whose answer is on the public web.
2. Consider making the mismatch loud rather than silent — e.g. have
   `researcher.md` state plainly that it has no repo search tools and should
   refuse repo-internal questions, so a misroute fails fast instead of
   confabulating.
3. Unrelated but adjacent, found in the same session and worth folding into any
   fix to these scripts: both wrappers emit ~30 lines of Bun stack trace ahead of
   every result, because `.pi/extensions/doctrine/index.ts:8` (`resolveBoot`)
   shells a stale absolute path
   `/nix/store/…-doctrine-0.31.1/bin/doctrine prompt resolve --role orchestrator`
   that no longer exists in the jail. It fails open, but the trace is written
   into every research artefact and re-read on every consumption. Resolve via
   `${DOCTRINE_BIN:-doctrine}` as `.mcp.json` already does, or suppress the
   extension's stderr in the wrappers.

Recorded from SL-233 preflight; see `.doctrine/rfc/011/case-notes.md`
(`[preflight; sl233-plan-research-20260727]`) for the token-efficiency framing.
