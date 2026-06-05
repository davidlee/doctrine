---
name: record-memory
description: Use when you discover or confirm a durable fact, constraint, invariant, convention, sharp edge, footgun, or reusable workflow that a future agent would otherwise rediscover — capture it before it is lost to conversation context. Triggers at phase/slice wrap-up too.
---

# Record Memory

Conversation context does not persist. When knowledge would save a future agent
meaningful time, write a memory record immediately. Phase and slice wrap-up are
mandatory prompts: scan `notes.md`, the active phase sheet, audit findings, and
fresh gotchas for durable guidance worth keeping.

Promote only durable guidance — repeatable workflows, sharp edges, invariants,
subsystem facts. Not conversational state.

## 1. Choose the narrowest type that fits

`fact` (atomic checkable truth) · `pattern` (repeatable recipe / command
sequence) · `system` (subsystem map + pointers, not a spec) · `concept` (stable
mental model / terminology) · `signpost` ("start here" navigation) · `thread`
(short-lived working set, expires fast).

## 2. Record it

```
doctrine memory record --type <type> "Title" \
  --summary "one line" --tag <tag> [--key mem.<type>.<domain>.<subject>]
```

Scaffolds a TOML + body under `.doctrine/memory/items/`. The **born git anchor
is captured automatically** — do not hand-author it.

## 3. Scope it so it will be found

- `--path-scope <file>` — exact file(s); strongest match.
- `--glob <pattern>` — subsystem relevance (e.g. `src/auth/**`).
- `--command <tok>` — tied to a command flow (token-prefix).
- `--tag <tag>` — stable categorization; do **not** overload tags as scope.

## 4. Set the risk axes (no flag yet — edit the TOML)

`record` defaults `[trust] trust_level = "medium"`, `[ranking] severity =
"none"`, `weight = 0`. For risky or drift-prone memories, edit the scaffolded
TOML:

- **trust_level** = confidence: `low` (inferred, unvalidated) · `medium`
  (derived from reasonable context — default) · `high` (verified against code /
  specs / direct observation).
- **severity** / **weight** = how much it matters if wrong or ignored.

Calibrate honestly: creating a memory is authoring, not verification.

## 5. Attest if you verified it

```
doctrine memory verify <UID|KEY>
```

Stamps `verified_sha` against the working tree (refuses a dirty tree — no false
attestation). Unattested memories read as lower trust.

**Holdback caution:** `retrieve` suppresses low-trust ∧ high-severity memories
(non-bypassable). A high-severity claim you have not verified will be held back
until you raise its trust by attesting it — by design. Set trust to match reality.

## 6. Keep the body short and executable

Put "do X" steps in bullets with exact command snippets. Reference related
artifacts and memories inline with `[[uid]]` / `[[key]]` (cheaper than relations).
If the item would become an ADR or an evergreen spec, STOP — `doctrine adr new`
or author under `doc/*` instead. Memory is a pointer/recipe layer, not canon.

## 7. Sanity-check surfaceability

```
doctrine memory find --path-scope <file> --type <type>
```

Confirm it appears under the context a future agent will actually query.
