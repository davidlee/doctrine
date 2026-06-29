# IMP-212: Plan-time re-grep: verify design's named paths/constants against live tree before scaffolding plan

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

Design locks at author-time; workers execute later. The tree moves in between.
RFC-011 case notes record the costliest worker investigation: SL-166 PHASE-05,
where design §5.3 assumed a "dedicated enabling commit" for config that SL-146
had relocated to a gitignored file *the same day the design was authored*
(`a0acf0eb`). The worker burned a large fraction of its budget reconciling the
design against a moved target — the canonical "stale-design premise" shape.

`/plan` is the last agent touchpoint before workers execute, and it already
ingests the full design to author phase sheets. Adding a re-grep there catches
the widest staleness window (lock → execution) at zero extra design-ingestion
cost, before phase sheets are written against wrong paths.

## Change

One bullet added to step 2 ("Confirm planning is not getting ahead of design")
of the `/plan` skill: scan `design.md` for concrete grep-pable references —
file paths, function/type names, constants, config keys — resolve each against
the current tree (`grep`/`ls`/`git show`); a missing path / moved file / renamed
symbol means a stale premise → STOP and `/design` to reconcile before
scaffolding. Self-bounding: a design naming nothing concrete has nothing to
check, so it adds no blanket tax.

## Edit target

Edited the **tracked source** `plugins/doctrine/skills/plan/SKILL.md`. The
`.agents/`, `.claude/`, and `.doctrine/skills/` copies are derived (regenerated
by `just reinstall` → `doctrine install -y` + `npx skills add`); they are not
hand-edited. (The original SL-178 slice scope named `.agents/...`, which is the
untracked derived copy — itself stale, an apt instance of the failure this item
fixes.)

## Scope

Skill-text only — no CLI verbs, no other skill changes. Forward-looking; no
retroactive re-grep of already-planned slices.

## Provenance

Was scoped as SL-178; downgraded to a backlog improvement (single-file,
single-edit — slice ceremony was overkill). Answers RFC-011 OQ-1 (mechanism +
scope). See `.doctrine/rfc/011/case-notes.md`.
