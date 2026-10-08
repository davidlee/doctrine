# Backlog kind guidance

## Context

Agents mis-kind backlog items, mostly defaulting to `improvement` (517 IMP vs
304 ISS at slicing). Two causes:

1. **The definitions skew toward improvement.** PRD-009 §6 defines issue as
   "expected behaviour gone wrong" and improvement as "works but should be
   better"; the `backlog` skill narrows issue to "a bug or defect" and says
   improvement is "no new feature". Most concerns read as "works but should be
   better", so they land as improvements.
2. **The guidance arrives at the wrong time.** Most items are filed from inside
   other skills (`/harvest`, `/code-review`, `/consult`) that call
   `doctrine backlog new <kind>` without loading `/backlog`. The only text in
   context then is the boot essentials line, which lists kinds but gives no
   way to choose.

The user's heuristic, adopted as the target **kind ladder** (first match wins):

| order | test | kind |
|---|---|---|
| 1 | hasn't happened, but would be harm if it did | risk |
| 2 | mostly addresses a concern — even if the defect isn't pinned down; the default when debatable | issue |
| 3 | unsure whether to do it, or how | idea |
| 4 | adds a new capability, not dealing with a problem | improvement |
| 5 | upkeep, no behaviour change | chore |

This reorders PRD-009's precedence (`risk > issue > improvement > chore >
idea`) to `risk > issue > idea > improvement > chore`.

## Scope & Objectives

- **Govern the ladder.** A revision (REV) against PRD-009 replaces §6 kind
  boundaries and precedence with the ladder.
- **One owner.** New library doc `lib:reference/backlog.md` owns the ladder and
  the backlog mechanics the skill currently restates. This closes IMP-500's gap
  G3 ("backlog mechanics has no library owner").
- **Right-time delivery.** The boot essentials backlog line carries a short,
  explicit instruction to read the classification guidance in
  `lib:reference/backlog.md` before `backlog new` — not the ladder itself.
- **CLI surface.** `backlog new --help` gives a one-line test per kind,
  consistent with the library doc.
- **Shrink the skill.** `/backlog` cuts its restatements to `lib:` citations
  (ADR-005: skills route, reference docs explain).
- **Governance sufficiency (user direction).** Whatever PRD-009 revision this
  slice makes must also cover the governance the related backlog-surface items
  need, so their later slices need no further governance change: IMP-509
  (body-edit verb), IMP-343 (body filename consistency), IMP-274 (`/triage`
  grooming skill), and any of IMP-500's other gaps that PRD-009 governs.
  Research decides which of those actually need a governance change.

## Non-Goals

- Implementing IMP-509, IMP-343 or IMP-274 — only their governance.
- IMP-500 gaps other than G3 (library docs for memory, knowledge, specs, etc.).
- Reclassifying existing items — changing a kind mints a new id.
- Post-creation nags or a re-kind verb.

## Affected surface

- `.doctrine/spec/product/009/` — via a REV, not direct edit
- `install/backlog.md` (new library doc), `install/essentials.md`,
  `install/using-doctrine.md` (backlog paragraph → citation)
- `plugins/doctrine/skills/backlog/SKILL.md`; other skills that restate kinds
- `src/backlog.rs` — `ItemKind` clap help
- `.doctrine/backlog/improvement/500/` — mark G3 owned

## Risks, assumptions, open questions

- **Boot cost.** The essentials line rides every session; keep the delta to one
  short clause.
- **Shipped corpus.** `install/` and skills ship to client repos: no
  repo-private ids in them (ADR-024).
- **Assumption:** agents do follow an explicit "read X before Y" boot
  instruction more reliably than they load an un-routed skill.
- **OQ-1:** Which of IMP-509 / IMP-343 / IMP-274 need a PRD-009 change at all,
  versus a tech-spec or nothing?
- **OQ-2:** Does a new library doc need registering beyond the `install/` embed
  (library tree, nix `srcWithDist`)?

## Verification / closure intent

- REV applied; PRD-009 §6 matches the ladder.
- `doctrine library show lib:reference/backlog.md` resolves; skill and boot
  line cite it; no surface still defines kinds the old way (grep).
- `backlog new --help` shows the per-kind tests; a test pins them.
- IMP-500 updated (G3 owned); related items cite the REV where it covers them.
- `doctrine check gate` green; `doctrine boot --check` clean.

## Summary

## Follow-Ups
