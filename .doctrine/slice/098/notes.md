# SL-098 Notes

## RV-078 — Inquisition verdict

GUILTY on 8 counts (2025-07-22). Redesign in progress.

| # | Severity | Finding | Disposition |
|---|---|---|---|
| F-1 | blocker | Design skill already has requirements pass at state 3 | **Correct that .dirge/skills/ had it — but .dirge is transient. Plugins (authoritative) doesn't. Design now baselines plugins. Both "collect decisions" and "requirements pass" are new inserts.** |
| F-2 | blocker | REQ-DNN structured metadata in prose | **`design-requirements.toml` sidecar. Structured data in TOML; design.md has prose reference only.** |
| F-3 | major | plan.toml [requirements] dead fields | **Route through plan.md prose. Acknowledge plan skill constraint.** |
| F-4 | major | Orphan placement depends on missing altitude framework | **IMP-097 created. Design uses `/consult` guardrail. Home_hint in TOML is advisory.** |
| F-5 | major | /plan skill says [requirements] stays empty — design contradicts | **Same fix as F-3. Acknowledge explicitly in plan skill edits.** |
| F-6 | minor | Orphan section has no defined position in brief | **Nest under Governance/spec (REV) as `#### Orphaned requirements (REV introduce)`** |
| F-7 | minor | Walkthroughs not incremental | **Per-skill scenarios in §11 — each applies one skill edit in isolation.** |
| F-8 | nit | Entity-model line references are rotting | **Removed. Concept cited by name not by line.** |

## Key discovery: plugins vs .dirge

`plugins/doctrine/skills/*/SKILL.md` is the authoritative source. `.doctrine/skills/`
and `.dirge/skills/` are transient install targets. Prior SL-098 work edited
`.dirge/skills/design/SKILL.md` — effective at runtime but not durable. All edits
must target `plugins/doctrine/skills/`.

The plugins design skill has none of the prior SL-098 changes — no "collect
decisions," no "requirements pass," asks questions one at a time. The redesign
proposes both as new inserts against the plugins baseline.

## Harvested

- IMP-096: Requirements capture and refinement skills
- IMP-097: Altitude assessment framework
- Memory: design-staleness — skills at transient paths are not authoritative
