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
  `DOCTRINE_BIN` override path
- report the leg distinctly in the install output, and **disclose** the
  trust-gated skip rather than implying the entry is live (POL-003 facet 3 —
  opt-in and disclosed; research corrected the earlier STD-003 citation, whose
  scope fence excludes client-owned harness config)
- declare any host-tool dependency the portable form acquires (POL-002 facet 3):
  the `sh -c` wrapper is a POSIX-shell dependency and must not be silent
- extend the Claude leg as little as the report seam requires (D1); no parallel
  planner where a shared policy pays (D2)

## Non-Goals

- No write to the **user** layer (`~/.codex/config.toml`) and no shelling of
  `codex mcp add` — the project layer is the posture.
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

## Open questions

D1–D5 are carried in IMP-111's *Design decisions to settle*: the report seam;
policy-vs-rendering; the ownership/emitted-form set; portable-vs-baked for a file
that is **not** in the install `[gitignore].entries`; and the trust-skip
disclosure / `[features] hooks = true` ownership call. `/design` settles them; none
needs further research.

Research (`research/research.md`) adds two inputs: there is **no mechanism** that
resolves a file's tracking status (the codex arm passes `CommandForm::Baked` as a
literal), so D4's clean answer is portable *unconditionally*, mirroring `plan_mcp`;
and POL-002 facet 3 makes the `sh` dependency a declaration duty rather than a
taste call.

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
