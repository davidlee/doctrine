# IMP-524: Ship an orchestration tier (super-orchestrator seat + sub-orchestrator charter) for pi, claude and codex

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Doctrine ships no orchestration tier. `plugins/doctrine/skills/` — the
framework-owned canon every harness links into its own skills dir — has no
`super-orchestrator` and no `sub-orchestrator`, and `install/agents/` has no
sub-orchestrator def for any harness. The tier is operator-authored, lives at
user level in all three harnesses, and has already diverged three ways.

## Observed state

All three harnesses carry it. Viewed from a confined pi session, `~/` is a
persistent share rather than the operator's host home, so the claude and codex
skill dirs are copied in for inspection as `~/_claude_skills/` and
`~/_codex_skills/`; pi's are live under `~/.pi/skills/`.

| artefact | claude | codex | pi |
|---|---|---|---|
| `super-orchestrator/SKILL.md` | 11 317 B / 211 lines | 11 317 B — **byte-identical** | 7 800 B / 144 lines — *adapted* |
| `pi-orchestrate/SKILL.md` | 1 394 B / 23 lines | 1 394 B — **byte-identical** | 3 106 B / 53 lines — *expanded* |
| sub-orchestrator | **absent** | `super-orchestrator/references/sub-orchestrator.md`, 5 817 B, in **claude agent-def** frontmatter (`model: opus`, `maxTurns: 200`, `skills: pi-orchestrate`, `color: purple`) | `~/.pi/agent/agents/sub-orchestrator.md`, 2 274 B, pi-native frontmatter (`allowNestedSubagents`, `allowedAgents`, `inheritSkills`, `contact_supervisor`) |

Adjacent but not of this tier: `arbiter-review` (7 560 B) is claude-only.

## What that asymmetry says

1. **The split already exists in the field.** Two harnesses share one text
   byte-for-byte; the third is a real adaptation (jail-aware: "run inside the
   existing jail", "never assume they can read host paths mentioned in an older
   handover", "do not require Quotient"). Whatever ships does not need to invent
   a def-per-harness / skill-shared scheme — the operator already converged on
   one. It needs to make it authoritative.
2. **The middle rung exists in three incompatible forms and no canonical one.**
   The only sub-orchestrator written in claude agent-def format is parked under
   *codex's* skills tree, where codex cannot load it as an agent (codex defs are
   `.codex/agents/*.toml` — IMP-515), while *claude's* own copy has no
   `references/` at all. So in codex the charter is documentation, in claude it
   is missing, and in pi it is a differently-shaped def. Three hand-maintained
   variants, no source of truth — the same drift mode as IMP-523, but
   demonstrably three-way, which is the strongest argument for shipping it.
3. **The seat is defined against two other skills.** Its description reads
   "Combines /capsule-driver's tiering with /pi-orchestrate's delegation."
   `/capsule-driver` is doctrine canon; `/pi-orchestrate` is not shipped at all
   (all three copies are user-level, and 1 394 B of it is a pointer to
   `quotient help orchestrate`).

## The constraint that decides the shape: quotient

The claude/codex text assumes **quotient** throughout — `mcp__quotient__pi_agent`
workers, `pi_agent_wait` (~45 s cap per call), `quotient allow` for form
approvals, `quotient help orchestrate` as the real guide, "quotient is assumed"
as a **blocking** preflight item — and spends Sonnet/Opus subagents on top.
Doctrine depends on quotient nowhere. The pi adaptation explicitly refuses the
dependency ("Do not require Quotient or its host-side forms for native pi
delegation").

So the claude/codex text cannot become canon as-is. Under
**POL-003** (harness independence from host-harness seams) it may ship only as
an **opt-in supplement** whose correctness rests on doctrine-owned contracts —
or be rewritten onto a neutral delegation contract with quotient as one
conforming adapter. That rewrite is the real work in this item; copying three
directories into `plugins/doctrine/skills/` is not.

## Placement — the open question

Presumed **project-local**, which is where doctrine's install model already
lives: `.doctrine/skills` is the framework-owned canon and each harness links it
(`harness.pi` / `harness.universal` → `.agents/skills`, `harness.claude` →
`.claude/skills`).

User-local deserves a real answer rather than a default, because it is the
current state and it is *why* nobody noticed the three-way divergence: a copy no
installer knows about is a copy no installer refreshes. It is also not an
existing doctrine altitude for skills — `--global` is the only path to `$HOME`,
and it would put the tier into every project on the machine, doctrine-governed
or not. Decide it explicitly.

## What a fix has to account for

1. **The charter is the def half.** Per-harness frontmatter (pi:
   `allowNestedSubagents`/`allowedAgents`; claude: `model`/`maxTurns`/`tools`/
   `TaskStop`; codex: `.codex/agents/*.toml`, gated on IMP-515) with a shared
   body. Same split IMP-523 needs for the capsule tier — whichever lands first
   sets the pattern.
2. **No baked model.** `references/sub-orchestrator.md` pins `model: opus`, and
   `install/agents/claude/capsule-*.md` pins `opus`/`sonnet`. A shipped version
   must let the operator's configuration decide; the pi skill is already
   explicit about this ("do not automatically switch to GPT, Sonnet or Opus").
3. **Confinement is not portable.** The pi text is jail-shaped and correct for a
   confined worker; the claude/codex tier runs outside any jail on the host's
   own auto-mode permissions. Neither assumption may leak into the other's copy.
4. **`pi-orchestrate` overlaps.** A second skill claiming the same delegation
   ground ("bounded phase planning, tests and implementation through native pi
   subagents while retaining orchestration, verification, commits and Doctrine
   state"). Shipping the seat without ruling on `pi-orchestrate` leaves two
   skills competing for one seat.

## Relations

- **IMP-523** (pi capsule tier) — sibling tier, same def-per-harness /
  skill-shared asymmetry.
- **IMP-434** (capsule defs + `/capsule-driver`) — the shipped-tier precedent.
- **IMP-515** (codex agent defs) — codex has no agent-def install leg after
  SL-281 (`skills_via = "universal"`), so a codex sub-orchestrator def depends
  on it.
- **IDE-053** (orchestrator/worker role prompts for the clone workflow) — the
  orchestrator concept is split across that deletion, the capsule tier and this
  tier.
- Governed by **POL-003**; the opt-in-supplement question it decides was raised
  as IDE-034.
