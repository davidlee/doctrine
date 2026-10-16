# IMP-515: Install codex agent defs into .codex/agents

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Codex loads subagent definitions from `.codex/agents/*.toml`. Doctrine ships
agent defs for claude (`install/agents/claude/*.md`, four defs) and pi
(`install/agents/pi/dispatch-worker.md`), but none for codex. After SL-281 the
codex harness is registered skills-only (`skills_via = "universal"`), so a
codex user gets no dispatch worker or capsule defs.

## Direction

- Author codex def sources under `install/agents/codex/` in codex's TOML shape.
- Give the `[harness.codex]` registry entry an `agents = ".codex/agents"` link
  dir. SL-281's convergence (canon under `.doctrine/agents/codex/`, symlinked
  into the harness dir, gitignored via the managed block) then applies as-is.
- Open: whether the worker-role bake (`WORKER_RESOLVE_MARKER`, a `.contains`
  match plus `str::replace`) works inside a TOML string field, or needs a
  format-aware expansion. Same question for the `traits:` frontmatter, which is
  YAML-in-markdown today.
- Parity target: the def set claude ships (dispatch worker; capsule tier if
  codex can run it).

## Related

- SL-281 — install artefact ownership; ships the harness registry this extends.
- IMP-468 — plural agent-def install (claude).
