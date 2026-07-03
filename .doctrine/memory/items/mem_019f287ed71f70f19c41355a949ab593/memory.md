# Worker resolve band-filter asymmetry: agent-def bake excludes project band, session cascade includes it

Two distinct worker hymn resolutions coexist, and they compose **different bands**:

- **Agent-def bake** — `resolve_worker_role_body` (`src/install.rs`) →
  `worker_context(traits)` (`src/hymns.rs`) builds `BandFilter::Only([Role])`,
  adding `Model` iff the def declares `traits:`. It **excludes** `preamble`,
  `harness`, `stage`, and **`project`**. This is what gets baked into a dispatch
  worker's subagent-def system prompt at install time (the
  `{{ prompt resolve --role worker }}` marker expansion).
- **Session cascade** — `prompt resolve` / `prompt explain --role worker`
  (`src/commands/prompt.rs`) call `build_ctx` with no `--band` →
  `BandFilter::All`. It **includes** `project` (and preamble/harness/stage). This
  is what a worker's SessionStart hook delivers at runtime.

**Consequence for where client habits live.** A `project`-band overlay snippet
(e.g. doctrine's `.doctrine/hymns/project/doctrine-rust-conventions.md`) reaches a
worker via the **session cascade**, NOT via the baked agent-def contract. So
placing repo-specific client habits in the `project` band is correct *only because*
the runtime delivery path is All-bands. If a habit must be baked into the agent-def
system prompt itself, it has to ride the `role` (or `model`) band instead — the
bake band-filters `project` out.

Surfaced at SL-191 PHASE-07 (overlay reconciliation) and confirmed against the
plan: EX-3 / VA-1 verify `prompt explain --role worker` (All-bands), which is why
the project-band home satisfies the criteria. See [[mem.signpost.doctrine.dispatch-claude-arm-wrong-base]]
for the adjacent bake-time pitfall.
