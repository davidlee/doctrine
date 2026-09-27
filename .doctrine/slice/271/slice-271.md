# Codex MCP registration during install

## Context

Codex does not read `.mcp.json`. It reads `mcp_servers.<id>` from
`.codex/config.toml` (project layer, loaded for trusted projects). `boot install`
wires the doctrine MCP server for the **Claude** arm only (CHR-013: `plan_mcp` /
`install_mcp` → `.mcp.json`); the `Harness::Codex` arm of `install_refresh`
carries `mcp: RefreshOutcome::None`. A codex-driven session therefore reaches no
doctrine MCP tools (the `doctrine serve --mcp` review/funnel surface) unless a
human runs `codex mcp add` by hand — which has no scope flag and only ever
writes the **user** layer (`~/.codex/config.toml`), so it is not an install
posture.

IMP-111 holds the surface evidence (docs + a local probe against codex 0.155.1),
including the two behaviours that make this more than a port: codex does **not**
interpolate `${VAR:-default}` in `command` (it is exec'd literally), and the MCP
child environment is **whitelisted** (`env_vars`), so `DOCTRINE_BIN` must be
forwarded explicitly. IMP-111 also records the pre-design decision list D1–D5
this slice must settle.

## Scope & Objectives

Register the doctrine MCP server with codex from `boot install`, in
`.codex/config.toml`, to the same standard as the Claude arm:

- edit-preserving merge (comments and unrelated keys survive — this is a
  user-owned shared file, not a doctrine-owned dotfile)
- idempotent: a current entry is a no-op; no rewrite thrash
- no-clobber: a foreign or customised `doctrine` entry is left untouched and a
  manual snippet printed (fail-soft on malformed TOML)
- no host path in the tracked artefact (POL-002 / SL-195) while preserving the
  `DOCTRINE_BIN` override path — the portable form runs through a POSIX `sh`
  (DEC-323), so that dependency must be **declared** (POL-002 facet 3), never
  silently acquired
- report the leg distinctly in the install output (DEC-325: one report field, the
  file named by the harness), and **disclose** per POL-003 facet 3: the trust-gated
  skip, and — probed from the harness itself, `codex features list`, not from the
  file — whether codex hooks are enabled, warning when they are off or the state
  is unknown (DEC-329)
- extend the Claude leg as little as the report seam requires (D1); no parallel
  planner where a shared policy pays (D2)

## Non-Goals

- No write to the **user** layer (`~/.codex/config.toml`) and no shelling of
  `codex mcp add` — the project layer is the posture.
- No write of `[features] hooks = true`: install registers the server and *probes*
  the hooks state (DEC-329); enabling a client tool's feature switch stays the
  human's act.
- No change to the Claude `.mcp.json` behaviour beyond whatever the report-seam
  refactor (D1) touches; its suite is the behaviour-preservation proof.
- Not the codex **hooks** activation flow (`[features] hooks = true` +
  `/hooks`); whether install additionally ensures that flag once it writes the
  file is D5 and may be explicitly deferred here.
- Not the trust prompt itself — that is the human's act.
- No new MCP server, tool, or capability; no pi/cursor MCP surface.

## Affected surface (coarse)

`src/boot.rs` (codex arm of `install_refresh`, the `plan_mcp` / `install_mcp`
seam, `RefreshReport` + `wire()` reporting, `write_codex_activation`); possibly
`src/install.rs` (harness marker / gitignore handling, only if the baked-vs-
portable form decision needs it); the boot install tests (unit suite in
`src/boot.rs`, install e2e under `tests/`).

## Risks & assumptions

- **Risk — comparator thrash (the SL-195 `F-1` class).** The no-op comparison
  must track every form the installer can emit; a comparator testing the wrong
  form rewrites the entry on every install. Fix the emitted-form set before
  writing the predicate (D3).
- **Risk — parallel planner** duplicating the JSON decision table (D2). The
  hook merge core already demonstrates the abstract-axis answer (ADR-001).
- **Assumption —** `.codex/config.toml` is the project layer and `mcp_servers`
  is not among the keys a project layer may not override (re-verify against the
  live reference during design; the probe confirmed the path works).
- **Assumption —** the operator's "local MCP just works" holds inside a
  `trusted` project; the untrusted skip is silent (probe), hence the disclosure
  objective.

## Settled decisions (design run `dr-01a0e17c`)

D1–D5 from IMP-111 are settled; the run holds them, one accepted `DEC` per node:

| node | decision | record |
|---|---|---|
| command form | portable env literal through `sh -c` + `env_vars = ["DOCTRINE_BIN"]`; declare the shell dependency | DEC-323 |
| ownership + emitted-form set | own the wrapper shape only (`command = "sh"` + our line; `env_vars` part of current); plain literal, `/bin/sh` and baked abspaths are foreign | DEC-332 (supersedes the second half of DEC-324) |
| sharing boundary | separate planner + shell per arm; one shared classification enum + decision table | DEC-328 |
| report seam | one `mcp` field; `wire()` names the file from the harness | DEC-325 |
| disclosure + file ownership | MCP registration only; probe `codex features list` for the hooks state and warn; disclose the trust skip | DEC-329 |

Research (`research/research.md`) adds two inputs: there is **no mechanism** that
resolves a file's tracking status (the codex arm passes `CommandForm::Baked` as a
literal), so the portable form is chosen *unconditionally*, mirroring `plan_mcp`;
and POL-002 facet 3 makes the `sh` dependency a declaration duty rather than a
taste call.

Still owed at close (not a design question): a SPEC-011 revision introducing **two**
members — one retro-covering the shipped Claude `.mcp.json` arm, one for the codex
`mcp_servers` leg — with their `REQ` ids minted by the revision. No MCP
registration requirement exists for either harness today.

The adversarial design pass (`RV-399`, rev 29) landed 14 findings, all disposed
`fix-now` and verified. The material changes to scope: the ownership narrowing
above, the report now stating what was *written* rather than claiming activation,
and the two-member revision shape. See `.doctrine/slice/271/notes.md` for what a
further pass would probe.

## Verification / closure intent

- Planner unit tests mirroring the `plan_mcp_*` cases (absent → wire, ours +
  current → no-op, ours + stale → refresh/migrate, foreign → fallback, malformed
  → fallback, non-object/non-table shapes), plus the emitted-form matrix (D3).
- e2e: `boot install` into a temp repo carrying a `.codex` marker writes the
  entry; a second run is a documented no-op; an existing user key
  (`[features] hooks = true`, comments) survives byte-for-byte.
- No host abspath in the committed form, asserted as the Claude leg already is.
- `doctrine check gate` green at close (fresh binary against the real corpus).
- Reconcile: SPEC-011's install-wiring requirements likely need a new member
  (a codex MCP leg) — decide at `/reconcile`, via REV if the spec layer changes.

## Summary

(To be written at close.)

## Follow-Ups

- IMP-111 is the source item; on close, resolve it (`promoted`).
- IMP-245 (Cursor as a harness) inherits the same per-harness MCP question.
