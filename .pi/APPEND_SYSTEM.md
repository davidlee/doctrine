# pi harness (project-local supplement)

## Research

Use pi's native subagent facility for the `/research` round — not quotient's
`pi_agent` MCP tool, nor the superseded `./scripts/pi-scout` /
`./scripts/pi-research` shims. Those are how other harnesses (Claude) delegate
to pi; under pi itself they add a confining wrapper and a process hop for
nothing a native spawn can't do. The contract both must satisfy is in `.doctrine/governance.md`
§Research agents.

Name the model and thinking level on every spawn; a bare agent name leaves the
tier implicit and the reader cannot tell what they got. Default to your current
model — but the project defs pin their own (`scout`/`researcher` →
`deepseek/deepseek-flash`, `planner`/`dispatch-worker`/`context-builder` →
`deepseek/deepseek-v4-pro`), so an omitted `model` gives you the pin, not your
model. Pass `model` explicitly when you mean to inherit yours, with a `:high` /
`:off` suffix when the task warrants it; copy the exact provider/id from
`{action:"models"}`, never an agent name.
