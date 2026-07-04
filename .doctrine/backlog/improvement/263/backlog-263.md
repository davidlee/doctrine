# IMP-263: spec req show — read a requirement statement in one call

## Problem

`doctrine spec req` has only `list` (roster of id/label/kind/status) and
`status` (transition). There is no `show` verb to read a requirement's
**statement** (title + body). To read REQ-NNN's statement, an agent must either
grep `.doctrine/requirement/NNN/requirement-NNN.toml` directly for the title
field, or render the full parent spec via `spec show SPEC-NNN`. Neither is a
one-call path.

Witnessed once but structurally costly:
- ISS-206/REV-019 session: reading a requirement's statement for a REV
  `modify` action cost ~6 wasted calls (tried `spec req show`, `rec show` →
  no such verb, `spec req list --json` → rows omit statement, fell back to raw
  grep)

A REV that targets requirements by statement content needs "show me REQ-NNN's
statement" as a single read. The current surface forces a raw-file read against
the storage rule.

## Fix direction

- **`doctrine spec req show <REQ-NNN>`**: renders the requirement's title,
  statement body (from the MD), status, and parent spec. Reuse the existing
  entity-show rendering path used by `rec show` / `adr show`.
- Not a `doctrine req show` top-level verb (that's `rec` territory) — keep it
  under `spec req` as the requirement namespace already lives there.
- Low risk: read-only, one new CLI arm, existing entity-show render helpers.

## Related

- RFC-011 case-notes: `[reconcile-REV; ISS-206/REV-019 session]`
- IMP-096 (requirements capture and refinement skills)
