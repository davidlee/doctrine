# SL-100 Design: Memory lifecycle verbs and agent UX hardening

## Status

`design` — locked. All foundational decisions settled. Proceeding to adversarial
review then `/plan`.

## Context

SL-099 shipped read-path surfaces (relations in show/retrieve, wikilinks, backlinks,
`--expand`, `validate`, `--lifespan` filter/ageing, suggested relations,
`verify --allow-dirty`). The data model is widened with `lifespan`, `review_by`,
`provenance`, `trust`/`severity` flags.

This slice adds the write side: lifecycle verbs (`status`, `edit`, `tag`) and agent
skill updates that guide creation, retrieval, and maintenance with the expanded
vocabulary.

The Status enum already defines 6 variants: `Active`, `Draft`, `Superseded`,
`Retracted`, `Archived`, `Quarantined`. Only `Active` and `Draft` are reachable at
record time. Every other entity surface (knowledge, backlog, ADR, slice) has a
`status` or `edit` verb — memory has neither. Memories are write-once: the only way
to update any field is hand-editing TOML.

Hard `needs` SL-099 — the verbs operate on fields SL-099 introduces.

Full scope: `slice-100.md` (4 objectives).

## Architecture

### Module layout

```
src/tag.rs         NEW   leaf   normalize_tag (extracted from backlog.rs)
src/memory.rs      cmd   widened  run_status, run_edit, run_tag,
                                  apply_memory_tags, memory_status_transition
src/backlog.rs     cmd   import tag::normalize_tag (removes local copy)
src/main.rs        cmd   new CLI variants: MemoryCommand::Status, Edit, Tag
.agents/skills/    cmd   updated record-memory, retrieve-memory;
                         new reviewing-memory, dreaming
```

All new code follows the existing verb patterns: read TOML via `toml_edit`, mutate
in-place, write only if changed. `resolve_memory_toml_path` (existing) handles
`items/` vs `shipped/` — shipped memories rejected for writes.

**`src/tag.rs` extraction:** `normalize_tag` moves from `backlog.rs` to new leaf
module. Backlog imports it; memory imports it. Backlog tag tests stay green
unchanged — behaviour-preservation gate (same pattern as SL-099's `src/links.rs`
extraction from the wikilink surface).

### Behaviour-preservation gate

| Module | Why sensitive | Gate |
|---|---|---|
| `src/backlog.rs` | `normalize_tag` extracted; `run_tag` imports from `src/tag.rs` | Existing backlog tag tests pass unchanged |
| `src/memory.rs` | `resolve_memory_toml_path`, `append_memory_relation`, `MemoryRef` reused | Existing memory tests pass unchanged; no edits to existing functions |
| `src/entity.rs` | Unchanged | Untouched |
| `src/relation.rs` | No new `RELATION_RULES` rows | Untouched — memory uses `[[relation]]` with free-form labels (Tier 3, ADR-010) |

New behaviour → new tests. Existing suites are proof.

## Design decisions

### D1 — `memory status <REF> <STATE> [--by <OTHER>]`

Resolution: `MemoryRef::parse(ref)` → `resolve_memory_toml_path` (rejects shipped/).

**Vocabulary gate:** `Status::parse(state)` — same 6-variant enum. Refuses unknown
states with the known-vocab list (`active, draft, superseded, retracted, archived,
quarantined`).

**Transition:** Uses `dep_seq::set_authored_status(&path, &[("status", state),
("updated", &today)], …)` — identical pattern to `knowledge::run_status`.
Idempotent: re-transitioning to current status is a no-op (no write).

**`--by <OTHER>` (superseded only):**
- Required when `STATE == superseded`, forbidden otherwise
- `<OTHER>` resolved via `MemoryRef::parse` → uid resolution (same as the main REF)
- Appends `[[relation]]` row: `label = "superseded_by"`, `target = "<OTHER>"` via
  existing `append_memory_relation`. This is the ADR-004 §5 carve-out pattern: the
  reverse edge is written on the dead record (which is being rewritten anyway to
  flip status), so a reader can find its successor.
- Then flips status. Order: relation first (so if status-write fails, no orphaned
  status without the successor link).
- Both writes are idempotent — re-running the same supersession is a no-op.
- Self-supersession (`REF == OTHER`) refused.

**Output:** `{ref}: {status_colored_state}` — same format as `knowledge status`.

### D2 — `memory edit <REF>`

```
memory edit <REF> [--title <T>] [--summary <S>] [--status <STATE>]
                  [--lifespan <L>] [--review-by <DATE>]
                  [--trust <LEVEL>] [--severity <LEVEL>]
                  [--key <KEY>]
                  [--path-scope <P>]... [--glob <G>]... [--command <C>]...
```

At least one flag required. Single read→mutate→write transaction via `toml_edit`.
Writes only if any field changed (idempotent).

**Resolution:** `MemoryRef::parse(ref)` → `resolve_memory_toml_path` (rejects shipped/).

**Field mapping:**

| Flag | TOML path | Validation | Behaviour |
|---|---|---|---|
| `--title` | root `title` | non-empty after trim | replace |
| `--summary` | root `summary` | none (free text) | replace |
| `--status` | delegates to status verb logic | `Status::parse`; `--by` not available | same validation path as `memory status` |
| `--lifespan` | root `lifespan` | `Lifespan::from_str` | replace; key absent if unset (remove if present) |
| `--review-by` | `[review].review_by` | `YYYY-MM-DD` or empty `""` to clear | replace |
| `--trust` | `[trust].trust_level` | `low\|medium\|high` | replace |
| `--severity` | `[ranking].severity` | `critical\|high\|medium\|low\|none` | replace |
| `--key` | root `memory_key` | `validate_key`; refused if already set | set once (late-binding) |
| `--path-scope` | `[scope].paths` | non-empty, repeatable | **replace** entire array |
| `--glob` | `[scope].globs` | repeatable | **replace** entire array |
| `--command` | `[scope].commands` | repeatable | **replace** entire array |

**Tags excluded** — routed through `memory tag` (set algebra, not replace).

**`--key` invariance:** If `memory_key` is already set, `--key` is refused with
"key already set; memory_key is immutable once recorded." If no key exists,
`--key` allows late-binding. Enforced before any write.

**`--status` delegation:** Calls the same pure transition logic as `memory status`.
For `superseded`, the transition requires `--by` which `edit` doesn't offer →
fails with "use `memory status superseded --by <OTHER>` to record the successor."
All other states transition normally.

**`--lifespan` unset:** Passing an empty `--lifespan ""` or omitting the flag leaves
the existing value unchanged. Explicit removal is a separate concern (non-goal for
v1 — hand-edit TOML to remove the key).

**Transaction:** The `updated` field is stamped once, on any change — not once per
field.

**Edit core:**

```rust
// Pure — mutates the held DocumentMut in-place, returns true if any field changed.
fn apply_edit(doc: &mut toml_edit::DocumentMut, edits: &EditFields) -> Result<bool>;
```

Each field flag maps to one `table.insert("key", toml_edit::value(v))` or equivalent
nested navigation. The existing `updated` key is unconditionally stamped if any
field changed.

### D3 — `memory tag <REF> [TAGS]... [-d REMOVE]...`

Direct steal from `backlog tag`. Positional args add; `-d` removes. At least one
add or remove required.

**Shared leaf: `src/tag.rs`**

```rust
/// Normalise a tag for WRITE: trim, lowercase, validate charset [a-z0-9_:-].
/// Distinct from filter-fold (lenient, no charset reject).
pub(crate) fn normalize_tag(raw: &str) -> Result<String>;
```

Extracted verbatim from `backlog.rs`. Backlog imports it; memory imports it.
Backlog tag tests stay green — behaviour-preservation gate.

**`apply_memory_tags(doc, adds, removes, today) -> bool`**

Same set-algebra core as `backlog::apply_tags`, adapted for memory's `scope.tags` path:

1. Navigate to `doc["scope"]["tags"]` (F-1: bail if missing — "malformed memory,
   restore seeded scope.tags array")
2. Read current set via `BTreeSet<String>`
3. `new = (current ∪ adds) ∖ removes`
4. Set-compare no-op guard — if `new == current`, return `false` (no write, mtime hold)
5. Replace `scope.tags` with sorted array, stamp `updated` at root
6. Return `true`

The 30-line structural difference from `backlog::apply_tags` is TOML navigation
(`scope.tags` vs root `tags`). The set-algebra core is identical. Extracting
`normalize_tag` eliminates the charset-validation duplication; the navigation
boilerplate is not worth generalizing (same tradeoff SL-099 made: `links.rs` is
shared leaf, callers build their own TOML→string projection).

**Shell: `run_tag`**

```
1. resolve_memory_toml_path(ref) — rejects shipped/
2. Validate adds/removes via normalize_tag
3. Reject overlap (add ∩ remove)
4. Read TOML → apply_memory_tags(doc, adds, removes, today)
5. Write back if changed
6. Print "Tagged {ref}: {tag_list}"
```

**Idempotent behaviour:**
- Re-adding an existing tag → no-op (set-compare unchanged)
- Removing an absent tag → no-op
- Add + remove of same tag → rejected at overlap check (not silent remove-wins)
- Unsorted hand-authored `scope.tags` → sorted on first real change, untouched
  on no-op

### D4 — Skill updates

**Record-memory (§2 — after record):**
- Check suggested relations output on stderr; run `doctrine link` for
  high-confidence matches
- Use `[[relation]]` edges for durable graph structure; `[[mem.…]]` wikilinks
  for contextual "see also" in body prose
- `--lifespan` flag guidance: pick narrowest that fits — `identity` (never
  ages) → `semantic` (10:1) → `procedural` (3:1) → `episodic` (baseline) →
  `working` (fast decay)

**Record-memory (§4 — risk axes):**
- `--trust` and `--severity` are now CLI flags — replace "edit the TOML" with
  flag invocation

**Retrieve-memory (new sections):**
- After §2 (Two surfaces): mention `memory backlinks <REF>` for reverse edges,
  `memory retrieve --expand N` for graph expansion, `--lifespan` filter for
  retrieve/find
- In procedure: connection-making step — after retrieving, check relations
  on key memories and follow edges to related knowledge
- After §3 (Inspect risk): mention `memory validate [REF]` for checking
  dangling relations, stale verification, draft expiry before acting on old
  findings

**New: `reviewing-memory`**

Structured audit for stability gates: before releases, migrations, large
refactors, or when agent confusion detected.

1. Pull highest-impact via `memory validate` corpus-wide
2. Prioritize: scoped + attested + high commit count on scoped paths
3. Checklist: provenance, freshness, metadata efficiency, scope accuracy,
   actionability, duplication
4. Thread hygiene: archive/convert lingering unverified threads
5. Produce outcomes: every reviewed item ends in `verified`, `corrected`,
   `superseded`, `archived`, or `promoted`

**New: `dreaming`**

Unified memory corpus maintenance — covers both reactive (change-triggered:
files move, commands change, invariants shift, duplicates found) and proactive
(periodic/idle-time improvement). One skill, two entry paths.

1. **Validate.** `memory validate` corpus-wide. Triage each finding: fix now,
   capture as backlog item, or note and defer.
2. **Prune.** Memories past `review_by`, unverified threads past expiry,
   `working`-lifespan > N days. Archive or retract stale ones.
3. **Link.** Suggested relations on recent/edited memories. Check orphans
   (no inbound + no outbound edges). Run `doctrine link` for high-confidence
   matches.
4. **Backlog.** Findings that can't be fixed now → backlog items (risks for
   not-yet-surfaced issues, chores for cleanup, improvements for enhancements).
5. **Fact-check.** Spot-check top-N by severity × staleness against current
   code/docs. Correct (`memory edit`), supersede, or quarantine.
6. **Report.** Brief summary of actions, findings, deferred items — handoff
   so the next agent doesn't re-do the same checks.

All new skills follow the existing `SKILL.md` skeleton format (YAML frontmatter
with `name`, `description`, trigger guidance). Placed in `.agents/skills/`.

## Current → Target behaviour

### Objective 1 — `memory status`

| | Current | Target |
|---|---|---|
| Status transitions | Write-once at `record`; hand-edit TOML | `memory status <REF> <STATE>` |
| Vocabulary guard | None (hand-edit can write anything) | `Status::parse` — 6 known states, refused otherwise |
| Superseded tracking | Not tracked | `--by <OTHER>` writes `[[relation]] superseded_by`, flips status |
| Idempotency | N/A | Re-transition to current status = no-op |

### Objective 2 — `memory edit`

| Field | Current | Target |
|---|---|---|
| title/summary | Hand-edit TOML | `memory edit --title "..." --summary "..."` |
| status/lifespan | Hand-edit TOML | `memory edit --status draft --lifespan semantic` |
| trust/severity | Hand-edit TOML (or `--trust`/`--severity` at record time, SL-099) | `memory edit --trust high --severity medium` |
| review_by | Hand-edit TOML | `memory edit --review-by 2026-07-01` or `--review-by ""` to clear |
| key | Set at `record` or absent | `memory edit --key mem.pattern.foo` (late-binding, once only) |
| scope arrays | Hand-edit TOML | `memory edit --path-scope src/a.rs --path-scope src/b.rs` (replace) |

### Objective 3 — `memory tag`

| | Current | Target |
|---|---|---|
| Tags | Set at `record`; hand-edit TOML afterwards | `memory tag <REF> foo bar -d baz` |
| Validation | `validate_tags` at record time only | `normalize_tag` charset gate on every edit |
| Idempotency | N/A | Set-compare no-op guard |

### Objective 4 — Skills

| Skill | Current | Target |
|---|---|---|
| `record-memory` §2 | No relation/lifespan guidance | Suggested relations, `[[relation]]` vs wikilinks, lifespan selection |
| `record-memory` §4 | "edit the TOML" for trust/severity | `--trust` / `--severity` CLI flags |
| `retrieve-memory` | No relation/graph/validate mention | backlinks, `--expand`, `--lifespan`, validate-before-act |
| (new) `reviewing-memory` | Does not exist | Stability-gate audit procedure |
| (new) `dreaming` | Does not exist | Unified maintenance (reactive + proactive) |

## CLI surface

### New verbs

```
doctrine memory status <REF> <STATE> [--by <OTHER>]
doctrine memory edit <REF> [flags...]
doctrine memory tag <REF> [TAGS]... [-d <TAG>]...
```

### New flags

```
doctrine memory status
  --by <OTHER_REF>    Required for superseded, forbidden otherwise

doctrine memory edit
  --title <TITLE>
  --summary <SUMMARY>
  --status <STATUS>
  --lifespan <LIFESPAN>
  --review-by <DATE>
  --trust <LEVEL>
  --severity <LEVEL>
  --key <KEY>
  --path-scope <PATH>
  --glob <GLOB>
  --command <COMMAND>
```

## Verification alignment

| Requirement | Test strategy |
|---|---|
| `status` transitions valid states | Unit: each of 6 states transitions, stamps `updated`. Integration: `memory status <uid> draft` |
| `status` refuses invalid states | Unit: `Status::parse` rejects unknown. Integration: CLI error with known-vocab list |
| `status superseded --by` writes relation | Unit: `append_memory_relation` called with `superseded_by`. Integration: `memory show` shows `[[relation]]` row |
| `status superseded` missing `--by` refused | Integration: CLI error |
| `status --by` on non-superseded refused | Integration: CLI error |
| Self-supersession refused | Integration: `memory status <uid> superseded --by <uid>` error |
| Idempotent re-supersession no-op | Unit: `append_memory_relation` Noop. Integration: re-run, no file change |
| `edit --status` delegates to same logic | Unit: `memory_status_transition` called. Integration: `edit --status draft` = `status draft` |
| `edit` multi-field atomic update | Unit: `apply_edit` changes title+lifespan, stamps `updated` once |
| `edit` no-op when unchanged | Unit: re-apply same values → `apply_edit` returns false |
| `edit --key` late-binding | Unit: sets key on unkeyed memory. Integration: refused on keyed memory |
| `edit --key` refused if already set | Integration: CLI error |
| `edit --status superseded` refused | Integration: "use `memory status superseded --by`" error |
| `tag` add/remove set algebra | Unit: `apply_memory_tags` union/minus, sorted output. Integration: add `foo bar -d baz` |
| `tag` overlap refused | Unit: overlap detection. Integration: CLI error |
| `tag` idempotent no-op | Unit: set-compare returns false. Integration: re-add existing tag, no file change |
| `tag` charset validation | Unit: `normalize_tag` rejects bad chars. Integration: CLI error |
| `tag` shipped memory refused | Integration: CLI error |
| `normalize_tag` extraction from backlog | Unit: backlog `run_tag` tests pass green unchanged |
| Skills committed and discoverable | Integration: `doctrine boot` lists new skills via filesystem scan |
| Behaviour-preservation | Existing test suites in `entity.rs`, `relation.rs`, `memory.rs`, `backlog.rs` pass unchanged |

## Governance alignment

| Authority | Requirement | Alignment |
|---|---|---|
| ADR-001 | Leaf ← engine ← command, no cycles | ✓ `src/tag.rs` at leaf; `memory.rs` at command; imports flow down |
| ADR-004 | Relations outbound-only; `superseded_by` is sanctioned carve-out (§5) | ✓ `superseded_by` written on dead record during status flip — zero marginal coupling |
| ADR-010 | Memory relations are Tier 3 (free-form labels) | ✓ `append_memory_relation` with `"superseded_by"` free-form label |
| Storage rule | Authored vs derived tiers | ✓ All writes to authored TOML; no derived-tier writes |
| Behaviour-preservation gate | Shared machinery suites stay green unchanged | ✓ `backlog.rs` tag tests, `memory.rs` existing tests untouched |
| Pure/imperative split | No clock, rng, git, disk in pure layer | ✓ `normalize_tag`, `apply_memory_tags`, `apply_edit` are pure; clock injected by shell |
| No parallel implementation | Ride existing seams | ✓ `dep_seq::set_authored_status`, `append_memory_relation`, `MemoryRef` reused |

## Risks

- **R1 — `--key` immutability enforcement.** The memory TOML currently carries
  `memory_key` as an optional string. `edit --key` must refuse if the existing
  value is non-empty. A hand-edited TOML with an empty string key (`memory_key
  = ""`) must still accept late-binding — distinguish "never set" from "set to
  empty" (the scaffold writes `memory_key = ""` when no key provided at record).
  `run_edit` checks `memory_key.is_empty()` rather than `Option<>`.
- **R2 — `edit --lifespan` removal not supported.** V1 sets lifespan to a new
  value or leaves it unchanged. Removing a previously-set lifespan (back to
  unset) requires hand-editing the TOML. A follow-up `--lifespan ""` to clear
  is deferred.
- **R3 — Scope array replace semantics.** `--path-scope`, `--glob`, `--command`
  replace the entire array, not append. Users wanting to add one path while
  keeping others must pass all desired values. This matches `backlog edit`
  precedent (no partial-array merge for scope fields). If append semantics are
  needed, that's a separate feature.

## Open questions (non-blocking)

| # | Question | Disposition |
|---|---|---|
| OQ1 | `edit --lifespan ""` to clear/remove lifespan from TOML? | Defer — hand-edit for now |
| OQ2 | Should `edit` support `--review-by ""` to clear? | Yes, already designed — empty string clears the key |
| OQ3 | Scope array append (add one path without replacing)? | Defer — not needed for v1; hand-edit or pass full array |

## Adversarial review findings

| # | Finding | Severity | Resolution |
|---|---|---|---|
| F1 | `edit --key` empty-string detection: scaffold writes `memory_key = ""` when no key provided. `is_empty()` handles both absent key and empty string — but design doc should be explicit | Low | R1 already covers this; clarified in risk text |
| F2 | `--review-by ""` clears the field but `--lifespan ""` is deferred (OQ1) — asymmetry users will hit | Low | Noted as follow-up in scope. Defer to post-v1 |
| F3 | No `--color`/`-p` flags designed for new verbs | Trivial | Clap boilerplate — added at derive level |
| F4 | `edit` scope-array replace is lossy (R3). `--help` should note replace semantics | Low | Accept for v1; append follow-up deferred |

## Governance snapshot

Generated from `doctrine boot` at design-lock. Relevant authorities consumed:
- ADR-001 (module layering)
- ADR-004 (relations, `superseded_by` carve-out)
- ADR-010 (memory relation labels — Tier 3, free-form)
- SL-097 (supersede policy pattern, `StorageTarget::RelationRow` vs `TypedArray`)
- SL-099 design (prior art: leaf extraction pattern, `append_memory_relation`, `resolve_memory_toml_path`)
