# EVD-032: Interactive codex spawns MCP from a daemon whose PATH lacks doctrine

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Observation (2026-10-23)

The DEC-323 codex entry (`command = "sh"`, `args = ["-c", 'exec "${DOCTRINE_BIN:-doctrine}" serve --mcp']`,
`env_vars = ["DOCTRINE_BIN"]`) fails in interactive codex and works in `codex exec`.

Interactive codex shows:

```
MCP client for `doctrine` failed to start: MCP startup failed: handshaking with
MCP server failed: connection closed: initialize response
```

## Mechanism

```
interactive codex ──▶ app-server daemon (PATH frozen at daemon start)
                        └─▶ sh -c 'exec "${DOCTRINE_BIN:-doctrine}" …' ──▶ not found (127) ──▶ handshake fails
codex exec ─────────▶ spawns MCP itself with the caller's PATH ──▶ works
```

- Interactive codex delegates MCP startup to a long-lived `codex app-server daemon`
  (release 0.163.0 under `~/.codex/packages/app-server-daemon/`, running since
  2026-10-17). The PATH codex on the shell was 0.159.1.
- The daemon's PATH (read from `/proc/<pid>/environ`) does not contain
  `~/.local/share/doctrine/bin`, the only `doctrine` on the user's shell PATH.
- Replaying the wrapper under that PATH reproduces it:
  `env -i PATH="$daemon_path" sh -c 'exec "${DOCTRINE_BIN:-doctrine}" serve --mcp'`
  → `sh: line 1: exec: doctrine: not found`, exit 127. `sh` itself resolves
  (`/run/current-system/sw/bin/sh`).
- Control: an isolated `CODEX_HOME` probe running `codex exec` (0.159.1) with the same
  wrapper spawned the server, resolved `doctrine` from the caller PATH, and listed
  the full doctrine tool set. So did `codex exec` in this repo.
- User-side confirmation: replacing the entry with an absolute `command` path
  (`/home/david/.local/share/doctrine/bin/doctrine`, `args = ["serve", "--mcp"]`)
  fixed interactive codex in this repo and in a second repo.

## Bearing on DEC-323

DEC-323's rationale ("preserves the Claude leg's semantics — PATH default + env
override") assumed the MCP child inherits the user's shell environment. Under the
daemon it inherits the daemon's, which can be arbitrarily stale. Inference (not
probed): the rejected plain `command = "doctrine"` fails the same way, so every
PATH-dependent form is fragile; only an absolute path is daemon-independent.

Probe verified on codex 0.159.1 (exec) / 0.163.0 (daemon). Bears on IMP-497
(re-verify the codex MCP seam) and ISS-528 (foreign entry holds install incomplete).


## Daemon restart as a remedy (2026-10-23)

`codex app-server daemon` (codex 0.159.1; the `app-server` group is marked
experimental) offers `start | restart | stop | update | version | bootstrap`.
`daemon version` reports `"backend":"pid"`: a plain background process, not a
systemd service. Inference (not tested): with the `pid` backend, a restart
inherits the caller's PATH and fixes things until the next restart.

Rejected as an installer action:

- Restarting interrupts running codex work (help text: `update` "may interrupt
  running work").
- The fix doesn't last: any client can start the daemon. A Claude-plugin codex
  app-server was observed running with a PATH that lacks the doctrine directory.
- Under `bootstrap` (a managed service), the environment comes from the service
  definition, not from whoever ran the restart.
- An experimental command surface means harness-specific behaviour would keep
  growing (POL-003).

Viable as a printed hint next to the install's codex step.


## Layer precedence and serving outside a checkout (2026-10-23, SL-286 research)

Probed on codex-cli 0.159.1 with an isolated `CODEX_HOME`:

- When the user layer (`$CODEX_HOME/config.toml`) and the project layer
  (`.codex/config.toml`) both define `[mcp_servers.doctrine]`, codex lists and
  spawns the **project** entry. A stale project entry therefore shadows a
  user-level one.
- `CODEX_HOME` is honoured as the user layer.
- `doctrine serve --mcp` (0.46.10) with no project marker above the cwd exits
  with "No project root found", which codex reports as a handshake failure. In
  a git repo without `.doctrine/` it starts, serves its tools, returns empty
  results and writes nothing.
