# Dispose of doc/* as legacy heretical practice; rehome & archive

## Context

`doc/*` is the legacy home of 9 evergreen specifications — `entity-model.md`
(umbrella architecture), plus per-subsystem specs (`slices-spec.md`,
`spec-entity-spec.md`, `memory-spec.md`, `skills-spec.md`, `drift-spec.md`,
`relation-index.md`, `reservation-spec.md`, `install-spec.md`). All are
committed, all are public architectural content; none are gitignored.

This location predates the spec entity system. Doctrine now has first-class
`.doctrine/spec/tech/` entities for technical specifications, and the `doc/`
directory is an unreconciled parallel home — heretical coupling of spec content
to a flat unstructured directory with no entity identity, no lifecycle tracking,
and no relation edges.

**SL-021** (tech-spec-backfill) is lifting durable content from `doc/*` into
`.doctrine/spec/tech/NNN/`. It explicitly leaves `doc/*` retirement **out of
scope**: "Whether `doc/*` is eventually superseded by the tech corpus is out of
scope (flag as a follow-up, do not decide here)." This slice is that follow-up.

**IMP-027** tracks: "Deferred doc/* canon has no tracked home or enforcement
after tech-spec backfill bifurcates the doc."

## Scope & Objectives

Dispose of the `doc/` directory — every file, the directory itself — once its
durable content has been rehomed into proper doctrine entities. Three workstreams
in sequence:

1. **Confirm rehoming complete.** Every `doc/*.md` has its durable architectural
   content lifted into the proper entity surface (tech specs via SL-021, and any
   project-global decisions via ADRs). No orphaned content remains in `doc/`
   that isn't represented in the entity corpus.

2. **Archive (if any private content surfaces).** If any file or content block
   is genuinely internal/private and unsuitable for public entity form, archive
   it privately rather than commit it to the public entity surface. (Currently
   all `doc/*` content is public; this is a forward-looking guard, not an
   observed need.)

3. **Remove `doc/` and update all references.** Delete the directory from the
   repo. Update every reference to `doc/*` across:
   - Source code (`src/boot.rs`, `src/corpus.rs`, `src/coverage.rs`,
     `src/install.rs`, `src/spec.rs`, `src/memory.rs`)
   - Skills (`plugins/doctrine/skills/` — every skill; do not enumerate by name;
     SL-084 adds dispatch, dispatch-subprocess, and dispatch-agent to this tree)
   - Install templates (`install/governance.md`, `install/glossary.md`)
   - Memory records (`.doctrine/memory/items/` entries referencing `doc/*`)
   - Any other docs or config pointing at `doc/*`

   References must point to the canonical entity surface (tech specs, ADRs) or
   be removed if obsolete.

## Non-Goals

- **Authoring tech specs** — that is SL-021.
- **Authoring new ADRs** — content already captured as an ADR stays; new ADRs are
  out of scope.
- **Rewriting content** — lift-and-rehome, not revise. Content improvements are
  their own slices.
- **Changing the `doc/*` → spec mapping decided in SL-021** — this slice
  consumes that mapping, does not re-litigate it.
- **Folder hoist** — moving `.doctrine/spec/{product,tech}/` to
  `.doctrine/*` is a separate concern (noted in SL-021's design).

## Affected surface

- **`doc/` directory** — the 9 `.md` files and the directory itself (removed).
- **Source code** — `src/boot.rs`, `src/corpus.rs`, `src/coverage.rs`,
  `src/install.rs`, `src/spec.rs`, `src/memory.rs` (reference updates).
- **Skills** — every skill under `plugins/doctrine/skills/` (SL-084 adds
  dispatch, dispatch-subprocess, dispatch-agent to this tree; reference
  updates, propagated to `.doctrine/skills/` by installer).
- **Install templates** — `install/governance.md`, `install/glossary.md`.
- **Memory records** — `.doctrine/memory/items/` entries citing `doc/*`.
- **`CLAUDE.md`** — now a separate file with Claude-specific reviewers section
  (SL-084 scope item 4, commit 227c3b0; no longer a symlink to AGENTS.md).
- **`AGENTS.md`** — harness-agnostic shared conventions (SL-084).
- **`install/governance.md`** — governance surface referencing `doc/*`.
- **`.gitignore`** — verify no entry needed; `doc/` has no negation rule.

## Risks, assumptions, open questions

- **Blocked on SL-021.** Cannot dispose of `doc/*` until its content is rehomed
  into tech specs. This slice is downstream of SL-021 completion.
- **Reference sprawl.** `doc/*` is cited in at least 25 files across source,
  skills, memory, and templates. Each reference must be re-pointed or removed,
  and the replacement target must actually exist.
- **Memory record staleness.** Memory records may cite `doc/*` as an evergreen
  anchor; after disposal those records become stale and need update or
  supersession.
- **Assumption:** SL-021's taxonomy covers all 9 `doc/*` files. If any file is
  intentionally left un-rehomed (e.g., `relation-index.md` is a deferred design
  note, not an architectural spec), this slice must confirm that decision is
  intentional and handle the residual.
- **Assumption:** No private content to archive — all 9 files are committed
  public architectural specs.
- **Aware:** SL-084 broke the `CLAUDE.md → AGENTS.md` symlink (commit 227c3b0).
  The reference sweep must search both files independently — they now carry
  different content.
- **Aware:** SL-084 creates `.pi/agents/dispatch-worker.md` — the `rg` sweep
  covers it; no special handling needed.
- **Open:** Should the `doc/` directory be `.gitignore`d post-removal to
  prevent accidental re-creation, or is the removal sufficient?

## Verification / closure intent

- `doc/` does not exist in the working tree.
- `rg 'doc/'` across the repo (excluding `.doctrine/memory/shipped/` and
  `target/`) returns zero hits for legacy `doc/` references — every former
  reference points to a valid entity (tech spec, ADR) or is removed.
- `just check` green; no broken references.
- `doctrine claude install` succeeds — re-run after all skill edits so installed
  copies under `.doctrine/skills/` match source.
- Memory records citing `doc/*` are updated or superseded.
- SL-021 is `done` (content rehomed) before this slice is `done`.

## Summary

SL-021 lifts content from `doc/*` into `.doctrine/spec/tech/`. This slice
finishes the job: remove the legacy directory and clean up every dangling
reference so `doc/*` is erased from the project's vocabulary.

## Follow-Ups

- IMP-027 (doc/* canon enforcement) is resolved by this slice — link as
  `resolves` once the relation vocabulary supports it.
