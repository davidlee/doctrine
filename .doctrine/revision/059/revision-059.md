# REV REV-059 — Extend SPEC-011 to the codex hook registry and generated pi extensions

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-263 adds two harness surfaces to `boot install`: a **codex hook registry**
(`SessionStart` emit hook plus a `PreToolUse` memory-surface hook across codex's
`Bash` / `apply_patch` matchers, with handler-level `additionalContextLimit` and
`timeout` on the memory-surface handlers and handler-field canonicality), and
the **generated pi extensions** including the new neutral-wire `surface.ts`
adapter. SPEC-011 (*Boot snapshot*) governs `boot install` but its body still
calls codex import-only, describes only the Claude hook set, and does not name
the existing pi extension generators or the new surface adapter. Its
responsibilities, mechanism prose and membership are extended here.

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

**After:** retain that Claude responsibility and append two distinct ones:

> Merge doctrine's owned codex hook registry into `.codex/hooks.json`,
> preserving foreign entries and placing the memory-surface context limit and
> timeout on each handler, never the matcher group.
>
> Plan and install the generated pi extensions (`index.ts`, `mcp.ts`,
> `surface.ts`) through their shared ownership-aware descriptor core, and report
> each file's outcome as its own leg.

The `responsibilities` list in `spec-011.toml` gains those two entries, and the
**Responsibilities** prose paragraph is extended to match. Keeping the Claude
responsibility intact preserves the scope and sweep contract of `REQ-186`,
`REQ-476` and `REQ-477`.

### SPEC-011 mechanism prose — before/after

The body has further stale claims that a responsibilities-only edit would leave
behind:

- **Overview:** replace the closing inventory's “`@`-import + Claude hook-set
  installer” with “`@`-import, harness hook registries and generated pi extension
  installer.”
- **`boot install` section:** retain the import and Claude settings paragraphs;
  replace “Codex is import-only (no hook set — one hook, baked form)” with a
  paragraph naming the codex `SessionStart` and `PreToolUse` registry, the
  ownership-preserving `.codex/hooks.json` merge, handler-level limit/timeout,
  the runtime trust step, and the generated pi extension triad. The section's
  “then does two things” remains accurate as import wiring followed by
  harness-specific refresh.
- **Concerns / D6:** extend the foreign-preservation and fail-soft language to
  the codex registry and the pi ownership marker. Do not imply the installer can
  verify that codex has trusted a hook merely because its file was written.

### Why two requirements, not one

`FR-010` (codex hook registry) and `FR-011` (generated pi extensions) are distinct
surfaces with distinct failure modes: a stale handler field that must heal, versus
a generated file that must regenerate on change while never clobbering a foreign
one. One widened FR would verify as nothing — the defect REV-049 settled against.

### Requirement statements

See the `[[change]]` payload for the `FR-010` and `FR-011` statements; they are
frozen at introduce:

- **`FR-010`** — the codex registry, foreign-entry preservation, the
  memory-surface handlers' `additionalContextLimit` / `timeout`, and
  handler-field canonicality.
- **`FR-011`** — the generated pi extension triad (one descriptor core, per-file
  `RefreshReport` leg) and `surface.ts`'s neutral-wire adapter contract.

### Not in this revision

The `memory surface` command itself (the neutral request, the three codecs, the
patch reader, the neutral envelope) is a memory-engine mechanism, not a
`boot install` concern. SPEC-011 names only that the codex/pi legs *wire* it.
`FR-011` names the adapter's use of the neutral envelope; it does not define the
envelope schema or the memory-side decoder. `REV-060` stages that memory-engine
contract against `SPEC-007`.
