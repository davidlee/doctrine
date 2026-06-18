# design-requirements.toml format and handoff points

The `design-requirements.toml` sidecar records implied/derived requirements surfaced during `/design`.

**Format:** `[[implied]]` rows with:
- `handle` — `REQ-DNN` (local design-scoped handle)
- `statement` — the requirement text
- `kind` — `quality` | `constraint` | `feature` | …
- `home_hint` — where the requirement likely belongs (spec hint)
- `descends_from` — which design decision produced it

**Storage rule:** The TOML is authoritative for structured fields. `design.md` `## Implied Requirements` carries one-line summaries only (prose reference, no repeated structured fields — no Statement:/Kind:/Home: field blocks).

**Handoff points:**
- `/plan` reads it for sub-step 2a (requirements mapping)
- `/audit` reads it for sub-step 4a (orphan survey)
- `/reconcile` reads it for step 4f (orphan placement)
