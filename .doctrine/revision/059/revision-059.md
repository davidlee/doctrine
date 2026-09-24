# REV REV-059 — Extend SPEC-011 to the codex hook registry and generated pi extensions

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-263 adds two harness surfaces to `boot install`: a **codex hook registry**
(`SessionStart` emit hook plus a `PreToolUse` memory-surface hook across codex's
`Bash` / `apply_patch` matchers, with handler-level `additionalContextLimit` and
`timeout` and handler-field canonicality), and the **generated pi extensions**
including the new neutral-wire `surface.ts` adapter. SPEC-011 (*Boot snapshot*)
governs `boot install` but describes only the Claude hook set and the two older
pi extensions (`index.ts`, `mcp.ts`); it names neither codex's hooks nor the
surface adapter. Its responsibilities and membership are extended here.

This follows the shape REV-049 established for SL-250: a spec-level `modify` row
for the responsibilities prose, plus `introduce` rows for the new member
requirements. Per REV-049's settled granularity, the new obligations are their
own requirements rather than one widened FR.

### Capability altitude

The change belongs in SPEC-011 (container, `descends from PRD-007`), not a new
spec: it extends the existing `boot install` responsibilities rather than
introducing a capability. It rides PRD-004 (memory) through the `memory surface`
command the codex and pi legs wire; PRD-004's own reconciliation is REV-058,
separate.

### SPEC-011 responsibilities — before/after

**Before** (the `boot install` responsibility that names the Claude hook set
only):

> ... merge doctrine's owned Claude hook SET into the settings file the
> `[install] claude-settings-scope` key selects — in that file's required command
> form — preserving every foreign hook and key, then evict this project's owned
> entries from the abandoned sibling scope and report the sweep.

**After** (every wired harness's owned hook set, plus the generated pi
extensions):

> ... merge doctrine's owned hook registry into each wired harness's settings
> file — the Claude hook set into the scope-selected settings file (in that
> file's required command form, preserving every foreign hook and key, then
> evicting this project's owned entries from the abandoned sibling scope and
> reporting the sweep), and the codex hook registry into `.codex/hooks.json`
> (handler-level context limit and timeout on the handler, never the matcher
> group) — and plan and install the generated pi extensions, reporting each as
> its own leg.

The `responsibilities:` frontmatter list in `spec-011.toml` mirrors the same
extension, and the **Responsibilities** prose paragraph is extended to match.

### Why two requirements, not one

`FR-010` (codex hook registry) and `FR-011` (generated pi extensions) are distinct
surfaces with distinct failure modes: a stale handler field that must heal, versus
a generated file that must regenerate on change while never clobbering a foreign
one. One widened FR would verify as nothing — the defect REV-049 settled against.

### Requirement statements

See the `[[change]]` payload for the `FR-010` and `FR-011` statements; they are
frozen at introduce:

- **`FR-010`** — the codex registry, its handlers' `additionalContextLimit` /
  `timeout`, and handler-field canonicality.
- **`FR-011`** — the generated pi extension triad (one descriptor core, per-file
  `RefreshReport` leg) and `surface.ts`'s neutral-wire adapter contract.

### Not in this revision

The `memory surface` command itself (the neutral request, the three codecs, the
patch reader, the neutral envelope) is PRD-004's recall mechanism, not a
`boot install` concern; SPEC-011 names only that the codex/pi legs *wire* it. The
`surface.ts` envelope shape is doctrine-owned and pinned by `FR-011`; its decoder
is governed by the memory capability, not here.
