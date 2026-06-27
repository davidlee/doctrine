# Wire --columns on relation list/census + tags in --columns + default tags column for all taggable kinds

## Context

IMP-133's catalogue of the `list`/`show`/`inspect` surface surfaced several gaps
in the shared read machinery that SL-025's uniform CLI surface left behind:

1. **`--columns` missing on `relation list`/`relation census`.** Every other
   `list` command supports `--columns` to select/order visible table columns;
   `relation list` and `relation census` do not. SPEC-013 owns the `--columns`
   projection model as a universal `list` contract — these two commands are
   non-conformant.

2. **Tags invisible in `--columns` for 7 taggable kinds.** `slice`, `adr`,
   `policy`, `standard`, `spec`, `rfc`, and `knowledge` all accept
   `doctrine tag set` (SL-136's data tier is wired) but their `list
   --columns` validators omit `tags`. So tags exist at the storage layer but
   cannot be displayed or selected in the human table. SL-136's design D2
   mandates full read-surface parity for every taggable kind: `list --tag`
   filter + `show` row + `--json` field. The `list --columns` gap means
   two of three surfaces are wired but the column projection is not.

3. **Tags absent from default `list` columns for 8 kinds.** `slice`, `revision`,
   `adr`, `policy`, `standard`, `spec`, `rfc`, and `knowledge` all have tag data
   but none shows `tags` in the default column set. Only `backlog`, `memory`,
   and `concept-map` include tags by default. The uniform read-surface contract
   (SPEC-013) implies that any datum wired into `--columns` should appear in
   the default set if it carries useful classification signal — tags qualify.

4. **Header casing inconsistency.** `concept-map list` uses Title Case column headers
   (`ID`, `Status`, `Tags`, `Slug`, `Title`) while every other `list` command uses
   lowercase (`id`, `status`, …). SPEC-013 mandates a uniform table shape.

5. **REC and review are not taggable.** IMP-144 tracks this as deferred work:
   wire tag read surfaces for REC and review, then add them to SL-136's
   taggable set. The write verb already exists (`doctrine tag set`) and
   uniform root storage is in place — only the per-kind read wiring and the
   prefix gating are missing.

## Scope & Objectives

1. **Add `--columns` to `relation list` and `relation census`.** Wire the
   shared `CommonListArgs` column-projection model into both commands so
   they accept `--columns <COLUMNS>` with the same semantics as every
   other `list` command. Available columns: the fields already rendered in
   the human table.

2. **Add `tags` to `--columns` for every taggable kind.** Extend the
   per-kind column-definition tables to include a `Column::tags` entry for
   `slice`, `adr`, `policy`, `standard`, `spec`, `rfc`, and `knowledge`.
   The extractor reads from the root-level `tags` array (SL-136's unified
   storage). `revision` already has `tags` in its column set — no change.

3. **Add `tags` to default columns for every taggable kind that lacks it.**
   `slice`, `revision`, `adr`, `policy`, `standard`, `spec`, `rfc`, and
   `knowledge` each get `tags` in their default `Columns` projection, placed
   immediately before `title` (matching the existing pattern in `backlog`
   and `memory`).

4. **Normalise `concept-map list` column headers** to lowercase
   (`id`, `status`, `tags`, `slug`, `title`), matching every other `list` command.

5. **Wire tag read surfaces for REC and review; add to taggable set.**
   Implement IMP-144's core: add `tags` to the column-definition tables and
   `show`/JSON render for REC and review, then add `REC` and `RV` to
   SL-136's `TAGGABLE` prefix set. The write verb (`doctrine tag set`) and
   root-level storage are already generic — only the read wiring and the
   gating prefix set need extension.

**Affected surface:** `src/listing.rs` (the kind-blind read spine — gains the
shared `default_with_tags` splice helper), `src/commands/relation.rs` (add
`--columns` flatten) threading through `src/relation_query.rs`, the 7 actual
column-definition sites — `src/governance.rs` (serves adr/policy/standard/**rfc**
via `governance::run_list`), `src/{slice,spec,knowledge,revision,rec,review}.rs`;
each gets a `tags` column entry and the conditional default. `slice`/`spec`/`rec`/
`review` row types also need a `tags` field added first. `src/backlog.rs` is
refactored onto the shared helper (its inline splice is the prototype).
`src/tag.rs` extends the `TAGGABLE` prefix set (`src/commands/tag.rs` reads it
at the write gate). REC/review `show`/JSON surfaces. `src/concept_map.rs` header
casing.

**Conformance guard:** `tests/e2e_list_conformance.rs` and
`tests/e2e_list_columns_golden.rs` — the existing matrix/net must be extended
to cover the new columns and the two newly-conformant relation subcommands.

**Design precedent:** SL-136 design §7 D2 (full read-surface parity for
taggable kinds — no write-only metadata); SL-025 (uniform `--columns` across
all kinds); SPEC-013 (the `--columns` projection model is universal).

## Non-Goals

- **New tag write or storage machinery.** The `doctrine tag set`/`clear` verbs
  and root-level `tags` storage are unchanged. Only read-surface wiring and
  the taggable-set gate are touched.
- **Memory, backlog, concept-map.** Their tags surfaces are already complete
  (default columns, `--columns`, `show`, `--json`). No change.
- **IMP-144's full scope beyond REC/review.** IMP-144 also mentions
  concept-map and revision as deferred taggable kinds, but those are already
  taggable (SL-136 included them). The only IMP-144 work still undone is
  REC/review read-surface wiring.
- **`search --json` shorthand.** That is a separate one-line fix tracked
  independently in IMP-133's catalogue (F-2).

## Summary

Five classes of narrow change, all on the read-surface side of the shared
entity engine:

- Wire `--columns` into `relation list` and `relation census` — two commands
  that slipped past SL-025's universal `list` contract.
- Surface `tags` in `--columns` and default columns for the 8 kinds that
  have tag data but don't render it at the column level.
- Wire tag read surfaces for REC and review and open the taggable gate —
  completing IMP-144's deferred read work.
- Normalise `concept-map list` column headers to lowercase.

No new verbs, no storage changes, no spec amendments. Pure read-surface
wiring on the existing shared machinery.

## Follow-Ups

- IMP-144's remaining scope (if any) — concept-map and revision surfaces are
  already wired; verify and close.
- IMP-133's other findings (F-1: `inspect` rollout, F-2: `search --json`,
  F-6: `standard list` slug, F-7: concept-map header casing, F-8: memory
  show format) — tracked separately.
