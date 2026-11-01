# IMP-523: Ship a pi capsule tier: agent defs and a harness arm for /capsule-driver

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

The capsule tier exists in doctrine's shipped surface, but only in claude's
shape: three claude agent defs, and one driver skill that ships to *every*
harness while being written entirely in unnamed Claude Code vocabulary. A pi
client installs, receives the skill, and is told to spawn agents it does not
have, using tools it does not have.

## The shipped surface

| artefact | path | lines | reaches pi? |
|---|---|---|---|
| capsule defs (3) | `install/agents/claude/capsule-{orchestrator,phase-planner,worker}.md` | 124 / 70 / 66 | **no** |
| the only pi def | `install/agents/pi/dispatch-worker.md` | 36 | yes |
| driver skill | `plugins/doctrine/skills/capsule-driver/SKILL.md` | 99 | **yes** — `[framework] roots` → `.doctrine/skills`, `harness.pi` → `.agents/skills` |

The install leg is not the problem, and IMP-468 already fixed that axis:
`desired_set` (`src/install.rs:3199-3235`) sweeps every `(harness, file)` pair
under `install/agents/` into `<harness dir>/<file>`, so **adding files to
`install/agents/pi/` needs no engine change**. A harness with no `agents` route
contributes nothing — which is codex's state (IMP-515). `git log --all --
install/agents/pi/capsule*` is empty: no pi capsule def has ever existed.

## Two defects, two policies

### 1. Harness vocabulary in shared canon — POL-003

The driver skill is framework canon linked into every harness's skills dir, and
it never names the harness it means. Six lines of claude-only machinery:

| line | token |
|---|---|
| 39, 41 | `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH`, `Agent` (silently withheld at the depth limit) |
| 47 | `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` |
| 49 | `TaskOutput` |
| 70 | `TaskStop` |
| 81 | `AskUserQuestion` |

POL-003 puts harness vocabulary in that harness's adapter, and a skill linked
into pi's `.agents/skills` is not an adapter. The failure is quiet in the way
POL-003 predicts: a pi reader sees a precondition it cannot evaluate, then a
reap step (`TaskStop`) with no pi equivalent, and has no signal that any of it
was ever claude-specific.

### 2. Seat and children in separate trees, both restating the contract

The driver skill and the three defs live in different trees
(`plugins/doctrine/skills/…` vs `install/agents/claude/…`) and *both* state the
phase-lifecycle contract. That is a drift generator, and it has already fired:

- `install/agents/claude/capsule-orchestrator.md:20` — "Flip the phase to
  `in_progress`" (orchestrator, loop step 2)
- `plugins/doctrine/skills/phase-plan/SKILL.md:45` — step 8, the *planner* flips it
- `capsule-phase-planner.md:22` — "Write the runtime phase sheet, and nothing
  else."

Two owners for one transition, across three files in two trees, adjudicated by
nobody. That is **ISS-519**, open. It is the empirical proof of the packaging
conclusion IMP-524 reached from the orchestration tier: one canonical body
co-located with the seat (a `references/` file), thin per-harness defs carrying
only instantiation keys and a pointer. Under that layout the disagreement above
is not merely detectable, it is unrepresentable.

### Not a quotient problem

The capsule surface contains zero quotient references
(`rg -in quotient plugins/doctrine/skills/capsule-driver/ install/agents/claude/capsule-*`
is empty). POL-002 facet 3 does not reach here — stated explicitly so this card
is not read as a clone of IMP-524's quotient finding.

## What a fix has to account for

1. **Write pi's defs; do not translate claude's.** The field pi tier has two
   rungs, not three (no `capsule-phase-planner`), and its worker **commits its
   own delta** where `capsule-worker.md:29-30` orders "NEVER commit, and NEVER
   discard. Your orchestrator commits." Same tier, inverted commit ownership —
   a semantic divergence, not a formatting one. A translated def would be wrong
   in the one place it matters most.
2. **No baked model.** `install/agents/claude/capsule-*.md` pin `model: opus` /
   `sonnet`; the reference to `capsule-worker` from IMP-524's tier passes
   `model` as an override, which only works if the def does not already decide.
3. **Three shapes, not two.** claude keeps the full tier (3 defs); pi plausibly
   needs 2; universal/codex get no defs at all — so the driver skill's pi arm
   must degrade to "no capsule tier here" rather than to a broken loop. pi also
   has no `TaskStop`/`/tasks` reap, so the serial-supervision discipline is
   re-derived, not translated.
4. **Placement is IMP-524's open question, not a local one.** These artefacts
   land in the same install surface and should be decided once, together.
5. **Instance-agnostic.** The hand-authored pi defs in field use carried
   project-local paths and assumptions and needed a manual cleanup pass; nothing
   in doctrine owned or refreshed that location, so every copy drifts
   independently. The invariant to restore: **doctrine-authored defs are
   instance-agnostic and refreshed by install.**

## Empirical evidence

`~/.pi/agent/agents/{capsule-orchestrator,capsule-worker}.md` exist as
hand-authored pi-native defs (`allowNestedSubagents`, `allowedAgents`,
`acceptanceRole`, `systemPromptMode`, `inheritSkills`) — keys doctrine ships in
*no* def (`rg 'allowNestedSubagents|acceptanceRole|systemPromptMode|inheritSkills'
install/` is empty). Their prose diverges from claude's exactly as item 1
describes. They were then cleaned by hand of baked paths, which is the drift this
item removes the cause of.

## Relations

- **IMP-434** (capsule defs + `/capsule-driver`) — the origin card, scoped
  claude-only.
- **IMP-468** — resolved; made the claude capsule defs actually install. This
  item is the second axis: not "does the directory get swept" but "is there a pi
  def in it at all".
- **IMP-524** (orchestration tier) — sibling tier, and the source of the
  co-location/pointer-stub conclusion and the placement decision. Scope the two
  together.
- **ISS-519** — the live desync this card's packaging fix prevents.
- **IMP-515** (codex agent defs) — codex has no `agents` route at all, so it
  cannot receive a capsule def however many are written.
- **CHR-040** — resolved; the pi agents dir is correctly sourced, it is just
  nearly empty.
- Governed by **POL-003**.
