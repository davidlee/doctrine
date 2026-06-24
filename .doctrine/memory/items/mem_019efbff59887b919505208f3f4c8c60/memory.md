# Claude Code docs ground-truth source

Authoritative source for **Claude Code** docs (hooks, MCP, subagents, plugins,
settings, CLI) — to be used instead of answering from training memory or a
subagent, both of which hallucinate parameters.

## Source

- **Index:** `https://code.claude.com/docs/llms.txt` (the `/en/llms.txt` variant
  404s; `docs.claude.com/en/docs/claude-code/*` 301-redirects here).
- **Any page, raw markdown:** `https://code.claude.com/docs/en/<page>.md`. The
  `.md` suffix returns the full literal source, not a rendered/summarised page.

## Method (zero-hallucination)

`curl` works from inside the jail (outbound network is open). Fetch the `.md` and
read the literal bytes yourself — no model in the loop:

```
curl -sS https://code.claude.com/docs/en/plugins-reference.md
```

Two haiku-risk layers this avoids:
1. A subagent (`claude-code-guide`, `Explore`) answering from training memory —
   highest risk; never trust for doc facts.
2. `WebFetch`'s internal small-model relay — paraphrases; lower risk but still
   lossy. `curl …/<page>.md` beats it because the bytes reach you unmediated.

## Key pages (all `code.claude.com/docs/en/`)

`plugins.md`, `plugins-reference.md` (manifest schema, dir layout, version mgmt,
monitors, LSP, debugging), `hooks.md`, `hooks-reference.md`, `mcp.md`,
`sub-agents.md`, `skills.md`, `settings.md`, `cli-reference.md`.

## Caveat

Docs track the latest Claude Code; pin findings to the **installed** version
(`claude --version` → currently **2.1.181**). Behaviour that matters is verified
empirically against the installed binary, not assumed from docs
([[mem.pattern.dispatch.worktreecreate-replace-base-control]]).

See also [[signpost.doctrine.reference-docs]], [[fact.doctrine.cli-source-of-truth]].
