# Codex agent defs

## Context

Codex loads subagent definitions from `.codex/agents/*.toml`. Doctrine ships
agent defs for claude (`install/agents/claude/`: dispatch-worker plus three
capsule defs) and pi (`install/agents/pi/dispatch-worker.md`), none for codex.
Since SL-281 the `[harness.codex]` registry entry is skills-only
(`skills_via = "universal"`), so a codex user gets no worker or capsule defs
(IMP-515).

The defs that carry `WORKER_RESOLVE_MARKER` are baked at install time: the
marker is replaced with the resolved worker role body (`install_agent_def` →
`bake_worker_def`, `src/install.rs`). Two problems meet in that step:

- **Format.** The bake is a `.contains` match plus `str::replace` over
  markdown, and `traits:` is read from YAML frontmatter. A codex def is TOML;
  the baked body must land as a valid TOML string, and traits need a TOML home.
- **Global installs** (ISS-520). `--global` writes canon to
  `$HOME/.doctrine/agents/`, shared by every project, but bakes from the
  installing project's hymn overlay and seal set. The last project to install
  decides every other project's worker context.

Both are the same function; design it once.

## Scope & Objectives

1. Author codex def sources under `install/agents/codex/` in codex's TOML
   shape. Parity target: claude's set — dispatch worker at minimum; capsule
   tier if codex can run it (to be established).
2. Add `agents = ".codex/agents"` to `[harness.codex]` so SL-281's convergence
   (canon under `.doctrine/agents/codex/`, symlinked into the harness dir,
   gitignored by the managed block) applies unchanged.
3. Make the bake format-aware: markdown and TOML defs both expand the marker
   correctly, and both carry traits.
4. Fix the global bake (ISS-520): pick one of — bake framework corpus only
   for global; refuse marker defs globally; or another option design surfaces.

## Non-Goals

- Codex/pi dispatch ledger symmetry (IMP-171).
- Serving skills to non-claude harnesses via `.agents/skills` (IMP-406) —
  wanted soon, separate slice.
- The pi-extension leak on codex boot install (ISS-479).
- Install UX and hymn starters (SL-282).

## Affected surface

- `install/agents/codex/**` (new).
- `install/manifest.toml` — `[harness.codex]`.
- `src/install.rs` — `install_agent_def`, `bake_worker_def`,
  `resolve_worker_role_body`, global-install path.
- `src/hymns.rs` — `worker_context(traits)` if traits parsing moves.
- SPEC-009 (Install & distribution) and the agent-def parity contract from
  SL-084; changes via REV.
- `tests/` — install integration tests.

## Risks, assumptions, open questions

- **Codex def schema.** Fields, how the system prompt is carried, whether
  codex honours tool/sandbox restrictions in a def. Establish from codex's own
  docs or a probe, not recall.
- **Capsule tier on codex.** The capsule defs spawn subagents of their own;
  whether codex supports nested spawning decides parity scope.
- **Bake input after SL-282.** SL-282 stops install writing hymn starters,
  which changes the disk corpus the bake reads. Sequence the slices
  deliberately.
- **Global bake choice** changes what a global-install user gets; it's a
  user-visible behaviour decision.

## Verification / closure intent

- `doctrine install --agent codex` writes `.doctrine/agents/codex/*.toml` and
  links them into `.codex/agents/`; the files parse as TOML and codex loads
  them.
- Baked codex dispatch-worker carries the resolved worker contract; claude/pi
  bake output unchanged (existing suites green).
- Global install no longer depends on which project ran last.
- SPEC-009 reconciled via REV; IMP-515 and ISS-520 resolved.

## Summary

## Follow-Ups

- IMP-406 — `.agents/skills` delivery for non-claude harnesses; touches the
  same registry entry.
- IMP-171 — needed only if codex should run dispatch end to end.
