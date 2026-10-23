# Codex MCP registers at user level with an absolute path

## Context

`boot install` writes the doctrine MCP server into the project
`.codex/config.toml` as a portable shell wrapper (DEC-323, REQ-484, SL-271):

```toml
[mcp_servers.doctrine]
command = "sh"
args = ["-c", 'exec "${DOCTRINE_BIN:-doctrine}" serve --mcp']
env_vars = ["DOCTRINE_BIN"]
```

EVD-032 shows this fails in interactive codex. Interactive codex spawns MCP
servers from a long-lived app-server daemon, and the daemon's PATH is frozen
when it starts. If that PATH lacks the doctrine install directory, the wrapper
exits 127 and codex reports `handshaking with MCP server failed … initialize
response`. `codex exec` spawns MCP servers itself with the caller's PATH, which
is why SL-271's probe passed. A hand-written absolute `command` fixed it in two
repos.

```
interactive codex ──▶ app-server daemon (PATH frozen at daemon start)
                        └─▶ sh -c 'exec "${DOCTRINE_BIN:-doctrine}" …' ──▶ 127 ──▶ handshake fails
```

Every PATH-dependent form is fragile under the daemon (inference; the plain
`command = "doctrine"` was not probed). An absolute path is the only form that
doesn't depend on the daemon's PATH. POL-002 forbids writing a machine path into
a file that client repos commit. The user-level `~/.codex/config.toml` is not
such a file. Restarting the daemon from `install` was considered and rejected
(EVD-032 § Daemon restart).

## Scope & Objectives

1. **Register at user level.** `boot install`'s codex arm writes
   `[mcp_servers.doctrine]` to the user-level codex config (`$CODEX_HOME/config.toml`,
   default `~/.codex/config.toml`). The entry is `command = <absolute path to
   doctrine>`, `args = ["serve", "--mcp"]`. The write is narrow-path and
   edit-preserving, as today: own only shapes doctrine has emitted, refresh a
   stale owned entry, leave foreign entries alone, and print a snippet when
   ownership is unclear.
2. **Retire the project-level entry.** An owned project-level entry (the wrapper
   forms in `CODEX_MCP_WRAPPER_FORMS`) is removed, so it can't shadow or
   contradict the user-level one. Foreign project-level entries are left alone
   and disclosed.
3. **Governance.** Supersede DEC-323 (and DEC-332 where its ownership shape
   changes) with a new decision. Revise REQ-484 (SPEC-011) through a Revision
   (ADR-013). Update the README declaration of the `sh` dependency.
4. **Disclosure.** The install report names the user-level file it wrote and the
   path it baked. It also carries the daemon-restart hint (EVD-032): an already-
   running codex daemon keeps its old config until restarted.
5. **Name the pattern.** This is doctrine's first user-level install write. Record
   the rule for when `install` may write outside the repo, so later arms follow
   it rather than reinventing it.

## Non-Goals

- **Claude (`.mcp.json`) and pi arms.** They don't share the daemon failure.
  Whether to move them to user level for uniformity is an open question for
  design to record (a QUE or a backlog item), not to implement here.
- **Codex hooks (`.codex/hooks.json`).** Already baked (`the_codex_arm_renders_the_baked_form`);
  unchanged unless design finds the same daemon issue there.
- **Daemon management.** No `codex app-server daemon restart` from `install`.
- **Post-write verification** (`codex mcp get doctrine`) remains IMP-497.

## Risks, assumptions, open questions

- **Losing the `DOCTRINE_BIN` override.** The wrapper let the jail and dispatch
  point codex at a dev binary. An absolute path doesn't. Design must say what
  replaces it (e.g. bake `DOCTRINE_BIN` when set at install time, or accept the
  loss for codex).
- **Which path to bake.** The running executable (`current_exe`, which could be
  `./target/debug/doctrine` during dev), the PATH resolution, or the installed
  location? Baking a dev build into a user-wide file affects every repo.
- **User-level means every repo.** The entry applies to every codex project,
  including repos without doctrine. `doctrine serve --mcp` must behave sensibly
  outside a doctrine checkout (assumption to verify).
- **Layer precedence (unverified).** Does a project-level `[mcp_servers.doctrine]`
  override the user-level one in codex? It determines whether retiring the
  project entry is required or merely tidy.
- **Jail.** Inside the bwrap jail, `~/.codex` may be read-only or absent, and
  the binary path differs (`/workspace`). Install must degrade with a disclosed
  outcome (STD-003), not fail.
- **ISS-528 interaction.** A foreign entry holds the install "incomplete". The
  same classification now applies at user level.
- **`CODEX_HOME` honoured.** Assumed: codex reads `$CODEX_HOME/config.toml` when
  set. Verify.

## Affected surface

- `src/boot.rs`: codex MCP arm (`CODEX_CONFIG_REL`, `CODEX_MCP_WRAPPER*`,
  `codex_mcp_entry`, `classify_codex_entry`, `plan_codex_mcp`,
  `install_codex_mcp`) and its tests.
- `tests/` install end-to-end coverage for the codex arm.
- `README.md`: codex MCP section and the `sh` dependency declaration.
- Governance: DEC-323, DEC-332, REQ-484 / SPEC-011.

## Verification intent

- Unit: plan/classify matrix for user-level entries (absent, owned-current,
  owned-stale, foreign, malformed) and for project-level retirement.
- E2E: install against a temp `CODEX_HOME` writes the absolute-path entry,
  removes an owned project wrapper, and leaves foreign entries alone.
- VH: interactive codex in this repo connects to doctrine after
  `doctrine install`, with a daemon whose PATH lacks the doctrine directory.

## Summary

## Follow-Ups

- Uniformity question for Claude and pi user-level registration (record during design).
- IMP-497: post-write `codex mcp get` assertion.
