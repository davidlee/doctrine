# CLI id-form normalisation

## Problem

The CLI surface is split into three id-form conventions for the SAME entity:

1. **PREFIXED-only** (`SL-123`): ~15 verbs via `integrity::parse_canonical_ref`
2. **BARE-only** (`123`): ~16 verbs via raw `u32` arg (no `value_parser`)
3. **BOTH** accepted: ~30 verbs via `governance::parse_entity_ref`

This is the single most expensive recurring friction in the case notes
(RFC-011), appearing 10+ times across multiple sessions (SL-166, SL-163, SL-177,
SL-184, SL-185). Every agent pays 1-2 retries per verb first-reach, every
session. The AGENTS.md boot guardrail says "cite the prefixed canonical id
everywhere" — but half the CLI surface rejects it, and the rejection error is
opaque (`invalid digit found in string` from `parse::<u32>`, not "expected bare
slice number like 123, not SL-123").

## Audit

Two parsing functions in play:

| Function | Location | Accepts |
|---|---|---|
| `parse_entity_ref(prefix, label, ref)` | `src/governance.rs:222` | Both `SL-123` and `123` (case-insensitive prefix) |
| `parse_canonical_ref(ref)` | `src/integrity.rs:448` | Only `SL-123` (rejects bare `123`) |
| raw `u32` (no value_parser) | various | Only bare `123` |

### Verbs that accept BOTH forms (via `parse_entity_ref`)

**slice**: `new` (n/a — scaffolding), `design`, `plan`, `phases`, `notes`, `phase`,
`status`, `show`, `paths`, `conformance`, `record-delta`, `verify-vt`

**review**: `show`, `raise`, `dispose`, `verify`, `contest`, `withdraw`, `prime`, `status`

**governance**: `adr show`, `policy show`, `standard show`, `rfc show`

**other**: `rec show`, `coverage show/record/verify/forget`, `backlog show/inspect/edit`

### Verbs that require PREFIXED-only (via `parse_canonical_ref`)

**review**: `new --target`

**relation**: `link <source>`, `link <target>`, `unlink <source>`, `unlink <target>`,
`needs <source>`, `needs <target>`, `after <source>`, `after <target>`,
`supersede <NEW> <OLD>`

**facets**: `estimate set/clear <target>`, `value set/clear <target>`,
`risk set/clear <target>`

**tags**: `tag add/remove <target>`

**exploration**: `inspect <id>`, `blockers <id>`, `explain <id>`, `map focus <id>`

**other**: `reseat <ref>`

### Verbs that require BARE-only (raw `u32`)

**slice**: `selector add <id>`, `selector note <id>`, `selector list <id>`,
`selector rm <id>`

**dispatch**: `setup --slice`, `resume --slice`, `prepare-review --slice`,
`sync --slice`, `refresh-base --slice`, `conclude --slice`,
`candidate create --slice`, `candidate status --slice`,
`candidate admit --slice`, `candidate close --slice`, `candidate status`
(plus `--id` flag for candidate admit/close)

## Fix approach

### Phase 1: eliminate BARE-only (low risk, no parse function change)

Add `value_parser = parse_cli_id` to every `u32` arg in:

- `src/slice.rs`: `SelectorCommand::{Add, Note, List, Rm}` — 4 places
- `src/dispatch.rs`: all `slice: u32` fields — ~12 places

`parse_cli_id` is `slice::parse_ref` → `governance::parse_entity_ref` which
already handles both forms (with test coverage at `governance.rs:1040`). This
change is purely mechanical — add the `value_parser` attribute.

### Phase 2: accept both in parse_canonical_ref consumers (higher impact)

Two options:

**A: Unify on `parse_entity_ref`** — replace `parse_canonical_ref` calls with
`parse_entity_ref(prefix, label, ref)` in every verb that currently uses it.
~15 call sites. Each call site needs the kind's prefix and a label string
(e.g. `"SL"`, `"a slice"`).

**B: Modify `parse_canonical_ref`** to accept bare numbers — if the input has no
`-`, treat it as a bare number and derive the kind by context. More invasive
(needs kind-context injection) but touches fewer call sites.

RECOMMENDATION: Option A — it is the simpler, safer unification. We already have
a proven dual-form parser. Use it everywhere.

### Phase 3: improve error messages

At minimum, replace the raw `u32` parse error (`invalid digit found in string`)
with a clap-level error that says "expected bare slice number (e.g., 123, not
SL-123)" — this is the one-line fallback if phase 1 can't land in one go.

## Acceptance criteria

- `doctrine slice selector add SL-227` works (currently: parse error)
- `doctrine dispatch setup --slice SL-227` works (currently: parse error)
- `doctrine link 227 governed_by ADR-001` works (currently: "not a canonical ref")
- `doctrine needs 227 SL-001` works (currently: "not a canonical ref")
- `doctrine review new --facet design --target 227` works (currently: "not a canonical ref")
- All existing tests pass unchanged (the parser already has dual-form coverage)

## Case notes reference

.doctrine/rfc/011/case-notes.md — this issue documented 10+ times across
sessions for SL-166, SL-163, SL-177, SL-184, SL-185, SL-172, SL-173.
