# REV REV-007 — reconcile SL-139

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile SL-139 (RV-136 F-4): the `paths` verb is now a proven, tested verb across
all 13 entity commands. SPEC-013's uniform command grammar section currently
describes the shared verb set as `new`, `list`, `show`, and (for lifecycle kinds)
`status` — it must be amended to include `paths`.

### Before (SPEC-013.md, Uniform command grammar)

> The verbs within are the shared set — `new`, `list`, `show`, and (for lifecycle
> kinds) `status` — so the invocation shape is identical across kinds:
> `doctrine <kind> <verb>`.

### After

> The verbs within are the shared set — `new`, `list`, `show`, `paths`, and (for
> lifecycle kinds) `status` — so the invocation shape is identical across kinds:
> `doctrine <kind> <verb>`.
