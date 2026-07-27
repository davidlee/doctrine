# IMP-326: Declare unobservable on the non-contributing corpus entries

Routed out of SL-232 by DEC-081 (RV-314 F-2's disposition). SL-232 ships the
`scope.unobservable` **mechanism**; this item is the **corpus work** of using it.

## Why it is not part of SL-232

The declaration is deliberately authored, not derived. § 5.3's own argument is
that `check-ignore` is a local-state instrument — which is *precisely why* the
boundary must be declared — so a mechanical bulk backfill would be the derived
judgement the field exists to refuse. Each entry is a judgement call about
whether a path is genuinely outside git's view for this project, and 30-odd of
those do not belong inside an implementation slice.

The test matrix does not need it either: RV-314 F-4's disposition re-based T49
onto a constructed `GitScratch` fixture population rather than a live-corpus
absolute, so nothing in § 9 depends on the corpus being backfilled.

## The population, stamped

Measured at HEAD `b4c8eac79` (figures from SL-232 `populations.py`; re-measure
before acting — RV-313 F-1 and RV-314 F-14 both caught design-time absolutes
that failed to reproduce through corpus growth):

- **59** non-contributing scope entries corpus-wide
- **33** of those have a root this checkout ignores — the candidate set
- **26** do not, and are plausibly *real* stale declarations rather than
  unobservable ones

So the expected outcome is roughly 59 undifferentiated reports becoming ~26
actionable findings. That is an **estimate, not a target** — the earlier figure
of 20 declarable / 39 actionable did not reproduce, because it used a fixed root
list omitting `.agents/skills/**`, `.mcp.json`, `.worktrees/**`,
`docs/claude/workflows.md` and `web/map/dist`.

## Intended operating mode

The normal path is one declaration in response to one `validate` finding, by the
agent that hits it. This item exists so the standing backlog is drained
deliberately once, not so the mechanism is bypassed. V1 and V2 police the result
either way: an entry matching nothing in `paths ∪ globs` is a finding, and an
entry git *does* match is a stale declaration and also a finding.

## Blocked on

SL-232 landing — there is no producer until `memory edit --unobservable` exists.
