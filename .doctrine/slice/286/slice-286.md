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

1. **Register at user level, with consent.** `boot install`'s codex arm runs
   `codex mcp add doctrine -- <absolute doctrine path> serve --mcp` after a
   dedicated consent prompt (default No; asked only when the entry would change;
   `-y` grants consent). It reads the current entry with
   `codex mcp get doctrine --json` (ASM-015). `mcp add` replaces the whole table,
   so dropped keys are listed in the prompt (DEC-403, DEC-406, EVD-033). Declining
   changes neither layer and reports "declined". A binary inside the installed
   project is never baked (DEC-404). No codex on PATH, a bad `CODEX_HOME`, or a
   codex failure degrades to printing the `codex mcp add` command (DEC-408).
2. **Retire the project-level entry; respect overrides.** An owned project-level
   entry (the wrapper forms in `CODEX_MCP_WRAPPER_FORMS`) is removed, since codex
   prefers the project layer (only after consent). A foreign project entry
   whose command contains a path separator is a
   deliberate override: untouched, reported, not held incomplete. A foreign
   PATH-dependent one is warned and holds install incomplete (DEC-406; partially
   resolves ISS-528).
3. **Rootless MCP server.** `serve --mcp` completes the handshake with no tools
   and a disclosed reason when no doctrine project resolves — no root, or a root
   without `.doctrine/` (DEC-405; disclosure visibility is ASM-014).
4. **Governance.** DEC-406 supersedes DEC-323 and DEC-332; DEC-407 supersedes
   DEC-325. Revise REQ-484 (SPEC-011) through a Revision (ADR-013). Drop the
   README declaration of the `sh` dependency for codex (DEC-409).
   **At reconcile**, update the README: the user-level entry and its scope, the
   consent prompt and that `-y` grants it, the project-level override slot,
   codex's per-folder trust prompt, and the codex CLI as a declared dependency.
   The `-y` help text says it grants user-level consent.
5. **Disclosure.** The install report names the user-level file it wrote and the
   path it baked, plus one line for project-level retirement, override or shadow
   (DEC-407). It also carries the daemon-restart hint (EVD-032): an already-
   running codex daemon keeps its old config until restarted.
6. **Name the pattern.** DEC-410 records when `install` may write outside the
   repo.

## Non-Goals

- **Claude (`.mcp.json`) and pi arms.** They don't share the daemon failure.
  Whether to move them to user level for uniformity is IDE-069.
- **Codex hooks (`.codex/hooks.json`).** Already baked (`the_codex_arm_renders_the_baked_form`);
  unchanged unless design finds the same daemon issue there.
- **Daemon management.** No `codex app-server daemon restart` from `install`.
- **Post-write verification** (`codex mcp get doctrine`) remains IMP-497.
- **Writing a project-level override.** Manual only; doctrine never writes a
  machine path into a project file (POL-002).
- **Nix-store baked paths** breaking after GC: RSK-234.

## Risks, assumptions, open questions

Resolved in design by DEC-403..DEC-410; kept as the scoping record.

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
  `install_codex_mcp`), the report path (`wire()`, `mcp_rel`,
  `RefreshOutcome::PrintedFallback`) and their tests; a `codex` CLI runner seam
  (injectable for tests); the install consent prompt (DEC-381 flow).
- `src/mcp_server/`, `src/commands/serve.rs`: rootless `serve --mcp`.
- `tests/` install end-to-end coverage for the codex arm (hermetic `CODEX_HOME`).
- `README.md`: codex MCP section and the `sh` dependency declaration.
- Governance: DEC-323, DEC-332, DEC-325 (superseded), REQ-484 / SPEC-011.

## Verification intent

- Unit: plan/classify matrix for user-level entries (absent, current, changed
  command, extra keys preserved, malformed) and for project-level entries
  (owned wrapper → removed, path-separator override incl. relative, bare
  PATH-dependent command, empty command, wrongly-typed fields).
- Unit: consent — prompt shown only on change, default No, `-y` consents,
  decline changes neither layer; degradation — no codex on PATH, empty /
  relative / missing `CODEX_HOME`, codex failure; dry-run never calls `add`
  nor retires the project entry.
- E2E: a fake `codex` on PATH records the `mcp add` argv.
- Unit/E2E: bake guard refuses a binary inside the project root; rootless
  `serve --mcp` handshakes with no tools outside a doctrine project.
- VH: ASM-014 — where codex shows the server's initialize instructions.
- Probe: ASM-015 — `codex mcp get` in a trusted project with a project entry.
- E2E: install against a temp `CODEX_HOME` writes the absolute-path entry,
  removes an owned project wrapper, and leaves foreign entries alone.
- VH: interactive codex in this repo connects to doctrine after
  `doctrine install`, with a daemon whose PATH lacks the doctrine directory.

## Summary

## Follow-Ups

- Uniformity question for Claude and pi user-level registration (record during design).
- IMP-497: post-write `codex mcp get` assertion.
