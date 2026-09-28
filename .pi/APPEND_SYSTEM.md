# pi harness (project-local supplement)

## Research

Use pi's native subagent facility for the `/research` round — not the
`./scripts/pi-scout` / `./scripts/pi-research` launcher shims. Those exist for
harnesses with no native subagent facility; under pi they run the same agent defs
(`.pi/agents/scout.md`, `researcher.md`) through more machinery and less
reliably. The contract both must satisfy is in `.doctrine/governance.md`
§Research agents.

Name the model and thinking level on every spawn; a bare agent name leaves the
tier implicit and the reader cannot tell what they got. Default to your current
model — but the project defs pin their own (`scout`/`researcher` →
`deepseek/deepseek-flash`, `planner`/`dispatch-worker`/`context-builder` →
`deepseek/deepseek-v4-pro`), so an omitted `model` gives you the pin, not your
model. Pass `model` explicitly when you mean to inherit yours, with a `:high` /
`:off` suffix when the task warrants it; copy the exact provider/id from
`{action:"models"}`, never an agent name.
