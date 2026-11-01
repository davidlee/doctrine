# IMP-523: Ship a pi capsule tier: agent defs and a harness arm for /capsule-driver

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

The capsule tier is **claude-only in both halves of its surface**, but neither
half is gated on the claude harness.

**Defs.** `install/agents/pi/` ships exactly one file —
`dispatch-worker.md`. The three capsule defs
(`capsule-orchestrator`, `capsule-phase-planner`, `capsule-worker`) live only
under `install/agents/claude/`, in claude-harness frontmatter
(`model: opus`, `tools: Agent, TaskStop, Read, Grep, Glob, Bash`,
`maxTurns: 80`, `isolation`-free but Agent-tool-bound). There is no pi capsule
def and never has been: `git log --all -- install/agents/pi/capsule*` is empty.

**Skill.** `/capsule-driver` ships to *every* harness — `plugins/doctrine/skills/`
is the framework-owned canon (`[framework] roots` → `.doctrine/skills`), and
each harness links it into its own skills dir (`harness.pi` / `harness.universal`
→ `.agents/skills`). Its 99 lines are written entirely against Claude Code:

- preconditions on `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` and
  `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS`;
- `TaskStop` to reap, `Agent` to spawn, `/tasks` to confirm the reap,
  `TaskOutput` withheld from subagents, `AskUserQuestion` withheld;
- permission modes `bypassPermissions` / `acceptEdits`, and the named-spawn
  teammate semantics ("never pass `name`").

None of that vocabulary or machinery exists in pi. pi's equivalent surface is
`allowNestedSubagents` / `allowedAgents` / `thinking` / `timeoutMs` /
`acceptanceRole` on the def, and `subagent` + `contact_supervisor` as tools.

## Why it matters

The asymmetry is invisible, in the same way IMP-468's was: a pi project installs,
gets `/capsule-driver` in `.agents/skills/`, and the skill names agents to spawn
that do not exist in pi's namespace. The only available move is to fall back to
`general-purpose`, or — the observed empirical outcome — to hand-author pi
adaptations and park them somewhere unmanaged.

Because those adaptations are hand-authored, they bake assumptions. Observed in
this environment: pi capsule defs sitting at `~/.pi/agent/agents/` (pi's own
user-level config dir, entirely outside doctrine's install surface) that had
project-local paths and assumptions in them and needed a manual cleanup pass.
Nothing in doctrine owns or refreshes that location, so every installed copy
drifts independently and no `doctrine install` can correct it.

That is the invariant worth restoring: **doctrine-authored defs are
instance-agnostic and refreshed by install; hand-authored ones are neither.**

## Evidence

- `install/agents/pi/` — one file, `dispatch-worker.md`.
- `install/agents/claude/` — four files, three of them capsule.
- `publication/manifest.toml:73-97` publishes the capsule defs under
  `integration/agents/claude/…` only.
- `plugins/doctrine/skills/capsule-driver/SKILL.md` — claude-only tooling
  throughout (lines 24-48, 58-72, 81-86).
- Empirical: `~/.pi/agent/agents/{capsule-orchestrator,capsule-worker}.md` exist
  with pi-native frontmatter (`allowNestedSubagents`, `allowedAgents`,
  `acceptanceRole`, `systemPromptMode`, `inheritSkills`). Doctrine ships *no*
  def carrying those keys — `rg 'allowNestedSubagents|acceptanceRole|
  systemPromptMode|inheritSkills' install/` is empty. Their prose also diverges
  from the claude defs (pi worker commits its own delta; claude worker hands back
  an uncommitted delta; pi orchestrator spawns workers directly, with no
  `capsule-phase-planner` tier). Agent-authored adaptation, not a copy.

## Shape of a fix (not prescriptive)

1. **pi capsule defs** under `install/agents/pi/`, instance-agnostic — no project
   paths, no `~` paths, no repo-specific slice ids. The plural install leg
   (IMP-468, landed) already sweeps the whole dir per harness, so adding the
   files is sufficient; no engine change should be needed.
2. **A harness arm in `/capsule-driver`** — the skill either splits per harness
   or carries a short "on pi" section that restates the preconditions in pi's
   terms and drops the claude-only steps. pi has no `TaskStop`/`/tasks` reap
   equivalent, so the serial-supervision discipline has to be re-derived rather
   than translated.

Decide (2) deliberately: a split skill duplicates the loop; a conditional
section keeps one loop but puts both vocabularies in every reader's context.

## Relations

- **IMP-434** (capsule defs + `/capsule-driver` skill) — the origin card; scoped
  claude-only.
- **IMP-468** (plural agent-def install) — resolved; made the claude capsule defs
  actually install. This item is the *second* axis: not "does the dir get
  swept" but "does pi have capsule defs in it at all".
- **IMP-515** (codex agent defs) — the sibling gap for codex; that card's framing
  treats pi as covered, which this item falsifies for the capsule tier.
- **CHR-040** (pi dispatch-worker symlink → `pi/`) — resolved; the pi agents dir
  is correctly sourced, it is just nearly empty.
