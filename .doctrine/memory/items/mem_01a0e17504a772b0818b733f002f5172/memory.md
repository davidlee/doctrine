Codex (verified 0.155.1) treats `config.toml` values as **literals** and gives MCP
children a **filtered** environment. Both bite anyone porting the Claude arm's
`.mcp.json` entry (`.doctrine` PORTABLE_EXEC, `${DOCTRINE_BIN:-doctrine}`) to a
codex `mcp_servers.<id>` block.

1. **No interpolation.** `command = "${DOCTRINE_BIN:-doctrine}"` is exec'd as that
   literal string — the server never spawns. `codex mcp list` prints the command
   verbatim. There is no shell in the spawn path.
2. **Filtered child env.** With `DOCTRINE_BIN` exported in the parent, an MCP
   server (even one launched via `sh -c`) does **not** see it. Forward it with
   `env_vars = ["DOCTRINE_BIN"]` on the entry; without the whitelist the wrapper's
   `${DOCTRINE_BIN:-fallback}` takes the fallback.

Working shape (probe-verified; the handler script logged its own argv):

```toml
[mcp_servers.doctrine]
command = "sh"
args = ["-c", "exec \"${DOCTRINE_BIN:-doctrine}\" serve --mcp"]
env_vars = ["DOCTRINE_BIN"]
```

`command = "doctrine"` needs no shell but loses the `DOCTRINE_BIN` override — the
case that matters in the jail/dispatch, where PATH `doctrine` is the read-only
`~/.cargo/bin` binary (IMP-249).

Reproduction recipe: isolated `CODEX_HOME` whose `config.toml` marks the scratch
project trusted; run `codex exec --skip-git-repo-check "…"` in it. MCP servers
spawn at session init, *before* the model call, so an unauthenticated run still
exercises the spawn path. Without the trust entry the project layer is skipped
silently.

See IMP-111 (codex MCP registration during install) for the install-side work.