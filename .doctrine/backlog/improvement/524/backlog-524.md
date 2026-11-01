# IMP-524: Ship an orchestration tier (super-orchestrator seat + sub-orchestrator charter) for pi, claude and codex

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Doctrine ships no orchestration tier. `plugins/doctrine/skills/` — the
framework-owned canon that every harness links into its own skills dir — has no
`super-orchestrator` and no `sub-orchestrator`, and
`install/agents/{claude,pi}/` has no sub-orchestrator def. The tier in field use
is operator/agent-authored and lives at user level, unowned by install.

## Observed state (this environment, 2026-11-01)

| harness | super-orchestrator | sub-orchestrator | where |
|---|---|---|---|
| pi | **skill present** | **agent def present** | `~/.pi/skills/super-orchestrator/SKILL.md` (7.8 KB), `~/.pi/skills/pi-orchestrate/SKILL.md`, symlinked from `~/.pi/agent/skills/`; def at `~/.pi/agent/agents/sub-orchestrator.md` (2.3 KB) |
| claude | absent | absent | `~/.claude/skills/` holds only the `synced/` third-party packs; `~/.claude/agents/` does not exist |
| codex | absent | absent | `~/.codex/skills/` holds only the `.system` built-ins |

Two corrections to the working assumption, both load-bearing for scoping:

- **Only pi has them.** claude's `history.jsonl` mentions `super-orchestrator`
  and `sub-orchestrator` twice each — past prompts, not installs.
- **`sub-orchestrator` is not a skill.** It exists only as a pi *agent def*
  (`inheritProjectContext: true`, `inheritSkills: true`, `allowedAgents:
  worker, delegate, scout, planner, researcher, reviewer`). There is no
  sub-orchestrator skill in any harness.

The pi skill states its own provenance: "Adapted from the operator's Claude
super-orchestrator skill" — the claude original is no longer on disk. So the
surviving artefact is a pi-shaped adaptation of a lost source, which is the
drift IMP-523 describes in the capsule tier: hand-authored, user-level, outside
the install surface, no refresh path, no second harness to keep it honest.

## The tier as it stands

    super-orchestrator (seat: interactive, holds the human, one run)
      └── sub-orchestrator (fresh child per charter: 1–3 phases, verifies, commits)
            ├── planner / phase work
            └── worker (bounded tests + implementation)

That is the same shape as the capsule tier (`super` → `capsule-orchestrator` →
`capsule-worker`, IMP-434), with different child names and no
`capsule-phase-planner` leaf. Whether these are two tiers or one tier with two
naming generations is a decision this item should force, not inherit.

## Placement — the open question

Presumed **project-local**, which is where doctrine's install model already
lives: `.doctrine/skills` is the framework-owned canon and each harness links it
(`harness.pi` / `harness.universal` → `.agents/skills`, `harness.claude` →
`.claude/skills`). User-local would be a new altitude: `--global` is the only
existing path to `$HOME`, and it would put the tier in every project on the
machine regardless of whether that project is doctrine-governed. Decide it, and
say why; do not let the current user-level location silently become the answer.

## What a fix has to account for

1. **Defs are per-harness; skills are not.** IMP-523 established the asymmetry
   for the capsule tier: the def carries harness-native spawning keys
   (`allowNestedSubagents`/`allowedAgents` for pi; `Agent`/`TaskStop`/`maxTurns`
   for claude; whatever codex's `.codex/agents/*.toml` schema is — see IMP-515),
   while the skill prose is shared. The sub-orchestrator charter is the def half
   and needs three variants; the seat skill is the prose half and needs either
   one harness-neutral text or conditional arms.
2. **The pi text is jail-shaped.** It already carries "run inside the existing
   jail and main worktree", "never assume they can read host paths mentioned in
   an older handover", "do not require Quotient". Those are correct for a
   confined pi worker and wrong verbatim for a claude session on a host. Whatever
   ships must not carry confinement assumptions into a harness that has none.
3. **Model/provider is configured, never inferred.** The pi skill is explicit
   about this ("do not automatically switch to GPT, Sonnet or Opus"); a shipped
   version must keep that, and the claude/codex variants must not bake
   `model: opus`/`sonnet` the way `install/agents/claude/capsule-*.md` does.
4. **`pi-orchestrate` overlaps.** A second pi skill in the same directory claims
   the same delegation ground ("bounded phase planning, tests and implementation
   through native pi subagents while retaining orchestration, verification,
   commits and Doctrine state"). Shipping either without ruling on the other
   leaves two skills competing for one seat.

## Relations

- **IMP-523** (pi capsule tier) — same class, sibling tier. Whichever is
  designed first should set the def/skill harness-split pattern for the other.
- **IMP-434** (capsule defs + `/capsule-driver`) — the shipped-tier precedent.
- **IMP-515** (codex agent defs) — codex has no agent-def install leg at all
  after SL-281 (`skills_via = "universal"`), so a codex sub-orchestrator def
  depends on that card.
- **IDE-053** (orchestrator/worker role prompts for the clone workflow) —
  adjacent: role prompts were deleted in SL-254 PHASE-10, and the orchestrator
  concept is split across that deletion, this tier, and the capsule tier.
