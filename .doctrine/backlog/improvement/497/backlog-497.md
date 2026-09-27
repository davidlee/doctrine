# IMP-497: Re-verify the codex MCP config seam against a live codex and assert it post-write

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

SL-271 writes the doctrine MCP server into a client's `.codex/config.toml`
(`[mcp_servers.doctrine]`) during `boot install`, but nothing confirms that the
written table is the one codex actually reads. The write rests on two things the
slice could not settle from doctrine:

- the codex **config seam** — the project-layer override of `mcp_servers` and
  its `command`/`args`/`env_vars` shape — was probed at codex 0.155.1 and is
  recorded as a POL-003 facet 2 version delta (SL-271 design sec-5.5); and
- the report claims only what doctrine *wrote*, never that codex loaded it
  (the trust-gated project layer may skip it silently).

## Proposed work

1. Re-verify the seam against the then-current codex release (the reference may
   have moved since 0.155.1).
2. Add a post-write harness-side assertion — `codex mcp get doctrine` (or the
   then-current equivalent) — so an install can state the entry is recognised,
   not merely written.

Deliberately excluded from SL-271 (SL-271 design sec-6): the slice's report
tells the truth about the write; this item closes the verification gap later.
