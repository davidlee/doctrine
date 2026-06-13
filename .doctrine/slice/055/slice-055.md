# Holistic skills review & token-efficient improvements

## Context

The skill corpus (`plugins/doctrine/skills/*/SKILL.md` — 24 skills — plus
`review/code-review` and `handover/handover`; the partner/memory plugins are
symlinks into `doctrine/skills`, not forks) has grown slice-by-slice. No pass
has yet looked at it *as a whole*: shared vocabulary drift, overlap between
skills, inconsistent structure, redundant recitation of governance already in
the boot snapshot, stale references (e.g. CHR-004: `close` text pre-ADR-009),
and uneven token weight (22→346 lines, no obvious correlation to skill
importance).

This slice runs that holistic critical review and homes the resulting
improvements. The conduct is **interactive `/pair` with Fable** — a human-in-loop
posture, not a hand-off. Fable's context is the scarce resource, so the slice's
first job is to **do the expensive reading up front** with async subagents and
leave Fable a compact, pre-digested evidence base to reason over.

**Division of labour (cost discipline):**
- **DeepSeek (external, user-operated)** — the mechanical research wave
  (inventory, vocab drift, staleness, boot/ref duplication, lifecycle-mention
  extraction). Smarter/faster/cheaper than an in-session Haiku subagent and runs
  off the agent's context entirely.
- **Opus** sub-agents (in-session) — reasoning-heavy synthesis: overlap/boundary
  analysis, the lifecycle seam matrix, and the ranked `00-index` synthesiser.
- **Fable (paired, foreground)** — adjudicates the synthesised findings, authors
  the fixes, and drives `dispatch`/`backlog` calls with the human.

## Scope & Objectives

- **Preparatory research base** — async-subagent-produced artifacts that
  catalogue the corpus and surface candidate problems, so the paired session
  reasons over evidence, not raw files. (The token-optimisation objective.)
- **Holistic critical review** — corpus-wide findings: structural consistency,
  vocabulary/term drift, skill-boundary overlap, governance recitation that
  duplicates the boot snapshot, stale refs, token bloat.
- **Improvements** — apply the agreed changes to the skill corpus. Prose edits;
  re-embed via the skills-refresh ritual
  (`[[mem.pattern.distribution.skill-refresh-command]]`).

## Non-Goals

- New skills / new capabilities (scaffold-skill is IDE-001, separate).
- Rewiring review skills onto the RV kind (IMP-023) — separate slice.
- Redesigning the lifecycle / routing-table *semantics*; this is a
  quality/consistency/cost/hygiene pass, not a redesign.
- CLI / Rust changes are not the *default* — prose edits are. But the disposition
  model (design §1) lets Fable `dispatch` a subagent for a structural fix within
  this slice, or `backlog` anything too involved. So CLI work is permitted but
  opt-in, not assumed.

## Affected surface

- `plugins/doctrine/skills/*/SKILL.md` (source of truth;
  `[[mem.pattern.distribution.skills-source-vs-installed]]`). Edits to the real
  dirs propagate through the partner/memory symlinks automatically.
- Possibly `install/{routing-process,using-doctrine,glossary}.md` if the review
  finds skill text duplicating reference-doc material.
- Re-embed: `src/skills.rs` touch + `doctrine skills install`
  (`[[mem.pattern.distribution.skill-refresh-command]]`).

## Risks / Assumptions / Open Questions

- **R1** — skill `description:` is the auto-trigger surface
  (`[[mem.pattern.skill.description-is-the-trigger]]`); edits there change
  routing behaviour. Treat descriptions as behavioural, not cosmetic.
- **A1** — "install/*" in the request means the skill corpus generally; actual
  source is `plugins/doctrine/skills/`, not `install/` (installer non-skill
  sources). Partner/memory duplicates are symlinks, not forks (confirmed).
- **OQ-1** — how much plan/phase ceremony does a prose-only slice warrant? Lean
  light per the request; design will set the bar.

## Summary

Set the scene for a paired Fable session that critically reviews the whole skill
corpus and lands consistency/cost/quality improvements — front-loading the
expensive reading onto async Opus/Haiku/Sonnet subagents to keep Fable's context
lean.

## Follow-Ups
