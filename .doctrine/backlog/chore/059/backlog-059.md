# CHR-059: Backfill empty requirement prose (Statement + Rationale) for 335 template-shell REQs

## Scope

335 of 475 requirement `.md` files (71%) are scaffold-only: heading, `## Statement`,
`## Rationale`, and HTML comments — zero human-authored prose. The TOML tier is
materialised (every REQ has a title, slug, kind, and status), but the MD tier is a
shell.

## Detection

Script at `scripts/find-empty-reqs.sh` (`--brief` / `--ids-only` for listing).

## Distribution

Nearly all REQs 001–309 are empty shells. REQ-310 is the inflection: from 310
onward, most have prose. The backfill is ~309 items (REQ-001 through REQ-309,
minus the 3 that were filled: 164, 165, 171, 258).

The TOML `description` field does carry the statement for some of these (e.g.
REQ-353 says "The sister TOML's `description` field is the primary, normative
statement"), which is legitimate per the storage rule — but the MD tier still
needs rationale and elaboration even when `description` is set.

## Root cause: `spec req add` can't carry a statement or acceptance criteria

`spec req add` scaffolds a title and slug only — it has no flags for statement,
rationale, or acceptance criteria. The default outcome of an interrupted
authoring session is precisely the title-only requirement CHR-058 exists to
repair. IMP-410 targets closing this gap at the tool level.

## Relationship to CHR-058

CHR-058 backfills REQ-254 through REQ-257 as an urgent dependency-unblocker for
RFC-027. CHR-059 is the broad campaign and sequences after CHR-058.

## Approach

- For REQs that have a TOML `description`, the MD Statement can reference it
  and elaborate.
- For REQs without either, research the owning spec/slice for context.
- Prefer small batches linked to an active slice rather than one monolithic PR.
