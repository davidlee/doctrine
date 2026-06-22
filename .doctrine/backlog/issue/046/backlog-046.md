# ISS-046: backlog needs CLI rejects SL-prefixed slice targets

## Reproduction

```bash
doctrine backlog needs IMP-120 SL-138
# Error: unknown backlog prefix `SL` in `SL-138` (expected ISS/IMP/CHR/RSK/IDE)
```

## Expected

`doctrine backlog needs` should accept any valid entity ref as a prerequisite
target, including slices (SL-NNN), specs (SPEC-NNN), ADRs (ADR-NNN), etc.
A backlog item can legitimately depend on a slice being completed — the data
model and actionability graph already support this (IMP-120 `needs: SL-138`
works when authored via direct TOML edit).

## Actual

The CLI validates all `<PREREQS>` args as backlog items (ISS/IMP/CHR/RSK/IDE
prefixes). Slice and spec refs are rejected.

## Impact

Users must hand-edit TOML to add cross-kind `needs` edges. The actionability
graph correctly renders these edges once authored, so the graph pipeline is
ready — only the CLI gate is too narrow.

## Fix

Widen the prerequisite validation in `doctrine backlog needs` to accept any
entity ref that the catalog can resolve, or at minimum add SL and SPEC
prefixes alongside the existing backlog prefixes.

**Design constraint:** the CLI must not maintain its own list of allowed
entity prefixes. It should import the definition from the domain layer so
the two can't drift apart. The data model already accepts cross-kind `needs`
edges — the CLI gate should reflect the domain, not duplicate and restrict it.
