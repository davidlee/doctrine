# IMP-111: Codex MCP server registration during install (separate config surface from .mcp.json)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

CHR-013 wired the **Claude** arm of MCP server registration into install: a
project-root `.mcp.json` `mcpServers.doctrine = { command: <abs doctrine>, args:
["serve","--mcp"] }` entry, written via `plan_mcp` / `install_mcp` in
`src/boot.rs`, riding the existing `install_baseref` merge core. The
`Harness::Codex` arm of `install_refresh` deliberately carries
`RefreshOutcome::None` for MCP — codex was out of CHR-013's scope.

Codex does not read `.mcp.json`; it reads a TOML `mcp_servers` block in its own
config surface. So the doctrine MCP server (`doctrine serve --mcp`, server id
`doctrine-mcp`, 10 review tools) is still unreachable over MCP for codex-driven
sessions.

### Surface confirmed (2026-09-27)

- **Project-local path is `.codex/config.toml`**, and it is real: "User-level
  configuration lives in `~/.codex/config.toml`. You can also add project-scoped
  overrides in `.codex/config.toml` files … Codex loads project-scoped config
  files only when you trust the project"
  (<https://developers.openai.com/codex/config-reference>); "Codex walks from the
  project root to your current [dir]"
  (<https://developers.openai.com/codex/config-advanced>). Merge order is
  built-ins < `~/.codex/config.toml` < project `.codex/config.toml` < profile <
  `-c` flags < env.
- **Shape** (Codex config reference, `mcp_servers.<id>.*` keys):
  `command`, `args`, `env`, `env_vars`, `cwd`, `enabled`, `url`, … — the naive
  entry is `[mcp_servers.doctrine]\ncommand = "doctrine"\nargs = ["serve",
  "--mcp"]`, but `command` is executed literally (below), so the real shape needs
  the shell wrapper of OQ-5.
- **`codex mcp add` cannot write the project surface.** It has no scope flag;
  the only documented destination is `~/.codex/config.toml` (verified against
  `codex mcp add --help`, codex 0.155.1: options are `-c/--config`, `--env`,
  `--url`, `--enable`, …; there is no `--scope`/`--local`). A manual
  `codex mcp add doctrine -- doctrine serve --mcp` therefore registers *user*-wide
  — the observation that prompted this note. **Install must merge the
  project-local TOML itself**; shelling `codex mcp add` is not an option.
- `-c mcp_servers.doctrine.command=…` (repeated per key) is the only
  CLI-side, file-free alternative; rejected as the install posture (it lives in
  the harness invocation, not in the repo install artefacts).

### Trust gate — OQ-4 answered (operator testing, 2026-09-27)

Local (project-scoped) MCP config **just works**: no trust/approval step at
startup. Only *hooks* need the explicit pass (`[features] hooks = true` +
`/hooks`), which is why the codex arm discloses those three manual steps and
still cannot assume the MCP path needs company. So the MCP writer needs no
install-time caveat beyond the usual malformed/foreign-entry fallback.

**Caveat from the local probe (same day).** The project layer is still
*trust-gated*: in a scratch project codex skipped `.codex/config.toml` entirely
(`codex mcp list` reported "No MCP servers configured yet"; the servers never
spawned) until the isolated user config declared the path trusted. Under
non-interactive `codex exec` there was **no prompt and no error** — the skip is
silent. Consistent with the operator's "just works" (their project is already
trusted), but it means install cannot promise the entry is live in a fresh
project; the trust prompt is the human's, and a silent skip is the failure mode
to disclose.

### Same-file interaction (new)

`.codex/config.toml` is not virgin territory for this repo's *documentation*, and
under this item it becomes doctrine-*written* for the first time:

- `boot.rs` `write_codex_activation` today **instructs** the user to "Ensure
  `[features] hooks = true` in `.codex/config.toml`" — doctrine already depends on
  this file existing and user-edited, while writing only `.codex/hooks.json`
  (`CODE_HOOKS_REL`, JSON).
- So the MCP merge must preserve user stanzas (`[features]`, comments, unrelated
  keys) — reinforcing OQ-2's `toml_edit` edit-preserving choice — and must create
  the file/parent dir when absent. Whether install should *also* ensure
  `[features] hooks = true` once it owns the file is a scope call, not an
  incidental — record it here rather than smuggling it in.

### Env expansion — OQ-5 answered by reproduction (2026-09-27)

**Codex does not interpolate config values.** `${DOCTRINE_BIN:-doctrine}` in
`command` is exec'd as that literal string — no shell, so no expansion, and the
server never starts. This is the operator's failed attempt reproduced locally
(codex 0.155.1, isolated `CODEX_HOME`, project trusted, three probe entries, a
handler script that logs its own argv):

| entry | `command` | result |
|---|---|---|
| literal env form | `${DOCTRINE_BIN:-…/handler.sh}` | **never spawned** |
| absolute path | `/tmp/.../handler.sh` | spawned |
| `sh -c` wrapper | `sh`, args `["-c", "exec \"${DOCTRINE_BIN:-…}\" c1"]` | spawned, `${…:-…}` expanded by the shell |

`codex mcp list` corroborates: it prints the `command` string verbatim.

**Second finding — the child env is filtered.** With `DOCTRINE_BIN` exported in
the parent, the `sh -c` server still saw it unset (the wrapper's
`${DOCTRINE_BIN:-fallback}` took the fallback). Adding
`env_vars = ["DOCTRINE_BIN"]` to the entry made the override land. So codex
whitelists the child environment (`mcp_servers.<id>.env_vars`), and neither the
wrapper nor `env` reaches the parent's variable without it.

**Working shape** (both facts applied):

```toml
[mcp_servers.doctrine]
command = "sh"
args = ["-c", "exec \"${DOCTRINE_BIN:-doctrine}\" serve --mcp"]
env_vars = ["DOCTRINE_BIN"]
```

This preserves `PORTABLE_EXEC`'s semantics (PATH default, env override) in a
committed, machine-path-free file (POL-002, SL-195) — at the cost of a POSIX
`sh` dependency (OQ-6). The plain alternative `command = "doctrine"` needs no
shell but **loses the `DOCTRINE_BIN` override entirely** — which is exactly the
case doctrine cares about: in the jail/dispatch the PATH `doctrine` is the
read-only, possibly stale `~/.cargo/bin/doctrine` (IMP-249), and dispatch
directs agents at the coord build via `DOCTRINE_BIN`.

## Wanted

Register the doctrine MCP server with codex during install, mirroring the Claude
arm's posture: idempotent additive merge, no-clobber of a foreign/customised
entry, fail-soft on malformed config. The command form does **not** transfer
verbatim from Claude: codex needs the `sh -c` wrapper + `env_vars` whitelist
(OQ-5), because it neither interpolates config nor inherits the parent env. Both
stay portable — no host abspath (POL-002, SL-195); IMP-111's original "absolute
exec path stamped" wording predates SL-195 and is stale.

- Add a codex MCP planner/installer beside `plan_mcp`/`install_mcp` (or
  generalise the existing core if the merge shape is close enough — watch for a
  parallel implementation: TOML vs JSON differ, so a shared *core* may not pay).
- Flip the `Harness::Codex` arm of `install_refresh` from `None` to the codex MCP
  outcome; report it in `wire()` alongside the Claude line.

## Open questions

- OQ-1 **Answered →** project-local `.codex/config.toml`, mirroring the Claude
  arm's posture (see *Surface confirmed* above). No global write.
- OQ-4 **Answered →** no trust caveat needed; project-local MCP applies at once
  (operator testing), unlike hooks (see *Trust gate* above).
- OQ-5 **Answered by reproduction →** codex does not expand `${VAR:-default}` in
  `command` (exec'd literally ⇒ server never starts), and the MCP child receives a
  *filtered* env, so `DOCTRINE_BIN` needs `env_vars = ["DOCTRINE_BIN"]`. Use
  `command = "sh"` + `args = ["-c", "exec \"${DOCTRINE_BIN:-doctrine}\" serve
  --mcp"]` + that whitelist.
- OQ-6 Portability of the `sh` wrapper: it buys the env override at the cost of a
  POSIX shell (no native-Windows codex). Acceptable, or does doctrine prefer
  `command = "doctrine"` + documenting that the PATH binary must be the intended
  one? Also check whether the wrapper needs `cwd` pinned so a project `sh` isn't
  picked up.
- OQ-2 TOML merge: codex config is TOML, not JSON — the `serde_json::Value`
  narrow-path mutate in `plan_mcp` does not transfer. A `toml_edit`-based
  edit-preserving merge is the likely shape (don't clobber comments/other keys).
- OQ-3 Ownership predicate: same shape test as Claude (command file-name
  `doctrine` + args `["serve","--mcp"]`), adapted to codex's entry layout.

## Pointers

- `src/boot.rs` — `install_refresh` (the `Harness::Codex` arm to flip),
  `plan_mcp` / `install_mcp` / `is_doctrine_mcp_entry` / `desired_mcp_entry`
  (the Claude precedent, CHR-013), `MCP_REL` / `MCP_SERVER_KEY`.
- CHR-013 — the Claude arm this extends.
- <https://developers.openai.com/codex/config-reference> — `mcp_servers.*` keys,
  project-config precedence, trust gate.
- <https://developers.openai.com/codex/config-advanced> — project config walk
  (project root → cwd).
- Probe recipe (reproducible): isolated `CODEX_HOME` whose `config.toml` marks the
  scratch project trusted (`[projects."<path>"]\ntrust_level = "trusted"`) —
  without it the project layer is skipped *silently* — then
  `CODEX_HOME=… codex exec --skip-git-repo-check "…"` in the scratch dir; MCP
  servers spawn at session init, before the model call, so an unauthenticated
  run still exercises the spawn path.
