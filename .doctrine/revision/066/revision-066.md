# REV REV-066 — reconcile SL-271

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-271 adds a second MCP registration arm to `boot install`: the Codex arm now
writes `[mcp_servers.doctrine]` into a project `.codex/config.toml` beside the
Claude arm that has merged `mcpServers.doctrine` into `.mcp.json` since CHR-013.
SPEC-011 (*Boot snapshot*) governs `boot install`, but **no requirement in it
covers MCP server registration for either harness** — RV-403 `F-2` — so the
behaviour had no acceptance criterion except this slice's own artefacts, and the
design reached for `REQ-186`, which governs the Claude hook-set merge, for a leg
it does not govern.

This is the write that closes the gap: two requirements, one per arm, plus the
scope-enumeration and responsibilities prose that names the leg. It follows the
shape REV-049 (SL-250) and REV-059 (SL-263) established: a spec-level `modify`
row for the prose, plus `introduce` rows for the new members.

### Capability altitude

The change belongs in SPEC-011 (container, `descends from PRD-007`), not a new
spec: it extends the existing `boot install` responsibilities rather than
introducing a capability. The two arms are distinct surfaces with distinct
failure modes — a `serde_json` merge at `mcpServers.doctrine` versus a
`toml_edit` narrow-path write behind a strict wrapper ownership predicate — so
each gets its own requirement rather than one widened FR (REV-049's settled
granularity).

### Why two requirements, not one

`FR-012` (Claude `.mcp.json`) and `FR-013` (codex `[mcp_servers]`) are different
files, formats, ownership predicates and disclosure contracts. One widened "MCP
registration" FR would verify as nothing — the defect REV-049 settled against.

### SPEC-011 scope enumeration — before/after

**Before** (the Overview's closing inventory):

> ... this spec ... owns only what is specific to *the projection and its
> wiring*: the pure assembly seam, the content-diff cache key, the section
> source-kind taxonomy and its marker fallback, the `@`-import, harness hook
> registries and generated pi extension installer, and the `--check` disk
> sentry.

**After:** the inventory names MCP server registration:

> ... harness hook registries, MCP server registration, and generated pi
> extension installer, and the `--check` disk sentry.

### SPEC-011 responsibilities — before/after

The `boot install` responsibility names the `@`-import and the two hook
registries. It gains the MCP registration posture as a distinct entry: register
the server edit-preservingly, own only a shape doctrine has emitted, preserve
foreign content, print a snippet rather than clobber, and disclose a write
rather than activation. The `boot install` section prose and the `Concerns` /
`D6` fail-soft language are extended to match.

### Requirement statements

See the `[[change]]` payload for the `FR-012` and `FR-013` statements; they are
frozen at introduce:

- **`FR-012`** — the Claude `.mcp.json` registration: the portable (or owned
  legacy) invocation, stale-refresh, foreign preservation, and the
  manual-snippet fallback.
- **`FR-013`** — the codex `[mcp_servers.doctrine]` registration: the
  narrow-path edit-preserving write, the strict wrapper ownership predicate, the
  no-host-abspath portable form, foreign preservation, the TOML-snippet
  fallback, and written-not-active disclosure.

### Not in this revision

The hook-state disclosure mechanism (the `codex features list` probe) is a
slice-local design decision, not a SPEC-011 contract; RV-403 `F-1` corrects it
through the slice's design back-edge, not here. `FR-013` states only that the
registration report claims a write, never activation.

## Reconcile narrative (SL-271)

- [RV-403 F-2]: no SPEC-011 requirement governed MCP server registration for
  either harness, and design sec-5.4 cited `REQ-186` (the Claude hook-set merge)
  for the codex leg's disclosure failure mode. Fixed by two requirements —
  `FR-012` retro-covering the Claude arm shipped under CHR-013, `FR-013` for the
  codex leg — plus the Overview scope enumeration, the responsibilities list and
  prose, and the `boot install` / Concerns mechanism prose.
