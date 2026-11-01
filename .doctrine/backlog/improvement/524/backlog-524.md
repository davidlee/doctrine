# IMP-524: Ship an orchestration tier (super-orchestrator seat + sub-orchestrator charter) for pi, claude and codex

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Doctrine ships no orchestration tier. `plugins/doctrine/skills/` — the
framework-owned canon every harness links into its own skills dir — has no
`super-orchestrator` and no `sub-orchestrator`, and `install/agents/` has no
sub-orchestrator def for any harness. The tier is operator-authored and lives at
user level in all three harnesses.

Its *content* has converged on two shapes (claude and codex share one, pi
adapts). Its *packaging* is three conventions, and the same file form means a
different thing in each.

## Observed state

Viewed from a confined pi session, `~/` is a persistent share rather than the
operator's host home, so claude's and codex's user dirs are copied in for
inspection — `~/_claude_skills/`, `~/_claude_agents/`, `~/_codex_skills/`; pi's
are live under `~/.pi/`.

| artefact | claude | codex | pi |
|---|---|---|---|
| `super-orchestrator` seat | `~/.claude/skills/…`, 11 317 B / 211 lines | 11 317 B — **byte-identical** | `~/.pi/skills/…`, 7 800 B / 144 lines — *adapted* |
| `pi-orchestrate` | 1 394 B / 23 lines | 1 394 B — **byte-identical** | 3 106 B / 53 lines — *expanded* |
| sub-orchestrator charter | `~/.claude/agents/sub-orchestrator.md`, 5 817 B — **spawnable agent** | `super-orchestrator/references/sub-orchestrator.md`, 5 817 B — **byte-identical, but a document** | `~/.pi/agent/agents/sub-orchestrator.md`, 2 274 B, pi-native frontmatter — **spawnable agent** |

Adjacent but not of this tier: `arbiter-review` (7 560 B) is claude-only.

## What that says — content is 2-shape, packaging is 3-shape

1. **The text split already exists.** claude and codex share the seat skill and
   `pi-orchestrate` byte-for-byte; pi is a real adaptation (jail-aware: "run
   inside the existing jail", "never assume they can read host paths mentioned
   in an older handover", "do not require Quotient"). The charter is the same
   file in claude and codex too. Nothing needs inventing — one canonical content
   set plus a pi adaptation, exactly as field-converged.
2. **Codex's packaging is right, and it is the only one that is.** codex has no
   agent-def surface: `~/.codex/agents/` does not exist, `config.toml` has no
   `[agents]` section, its `.system` packs carry `agents/openai.yaml` as
   `display_name`/icon/`default_prompt` UI metadata rather than spawnable roles,
   and `references/prompting-guide.md:93` defers subagent mechanics outward ("if
   you are implementing a multi-agent system in your harness"). Its own
   precedent for a *role* is `.system/review-agent/SKILL.md` — a role shipped as
   a skill and invoked, not spawned. Carrying the charter as
   `super-orchestrator/references/` is the same shape, and it keeps the seat and
   its child contract in one directory, where they cannot desync. claude and pi
   keep the charter in a separate tree from the seat, so a change to the
   hand-back contract must land twice with no drift detector. **Co-location is
   the better convention; the defect is not codex's.**
3. **But a reference cannot be instantiated, so "present" is not "parity".** In
   claude and pi the same 5 817 bytes become a separate context with its own
   model and turn budget — a *process the harness runs*. On codex they land in
   the seat's context, spending exactly the budget the tier exists to protect —
   a *document the seat follows*. That is honest for the available surface, not
   a bug, but it must not be read as the tier existing on codex in the same
   sense. pi already papers over the same distinction in the other direction:
   its 2 274 B def is pointers to the two skills, not a body.
4. **The seat is defined against two other skills.** Its description reads
   "Combines /capsule-driver's tiering with /pi-orchestrate's delegation."
   `/capsule-driver` is doctrine canon; `/pi-orchestrate` is not shipped at all
   (all three copies user-level, and 1 394 B of it is a pointer to
   `quotient help orchestrate`).

## The constraint that decides the shape: quotient

The claude/codex text assumes **quotient** throughout — `mcp__quotient__pi_agent`
workers, `pi_agent_wait` (~45 s cap per call), `quotient allow` for form
approvals, `quotient help orchestrate` as the real guide, "quotient is assumed"
as a **blocking** preflight item — and spends Sonnet/Opus subagents on top.
Doctrine depends on quotient nowhere. The pi adaptation explicitly refuses the
dependency ("Do not require Quotient or its host-side forms for native pi
delegation").

This is **POL-002 facet 3** (no undeclared dependency on a host tool), not
POL-003: quotient is not a harness, and POL-003's own rationale warns that
stretching it past the harness axis "would let a real gap pass as
already-governed". Facet 3 admits a host tool only at a declared altitude — a
named baseline requirement with a version floor, or a feature-scoped capability
that is opt-in, never acquired by a default path, and whose absence fails with a
message naming what is missing and what would satisfy it.

The field state fails facet 3 on both halves: "quotient is assumed" is a
**blocking** preflight item, so the default path acquires the dependency, and its
absence yields a preflight block rather than a named-missing-capability failure.

So the conclusion is stronger than "needs an opt-in supplement". **The canonical
tier must be quotient-free, with quotient as one conforming delegation adapter** —
and the pi variant is the proof that this is possible: the dependency is not
load-bearing on the capability (delegation), only on one implementation of it.

The assumption is structural rather than careless. Doctrine's own dev jail pulls
quotient as a flake input and builds confined-worker forms on `quotient`'s
`pi_agent` (`flake.nix:14`, `~230-233`; `CLAUDE.md:45-81`), so the tool is
ambient where the product is developed and reads as part of the platform. That
is the leak facet 3 exists to catch, and it explains why the tier came out
quotient-shaped without anyone deciding it should.

And the shared seat is not harness-neutral even setting quotient aside: codex
runs it byte-identically while it instructs `TaskStop`, `SendMessage`,
`PushNotification`, `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` and
`CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` (one each), plus 8 `quotient` and 13
Sonnet/Opus references — Claude Code machinery, on `gpt-6-luna` with
`sandbox_mode = danger-full-access` and `approval_policy = "never"`.
Byte-identity between claude and codex was never a virtue in the seat; only in
the charter. That half is the **POL-003** axis: harness vocabulary belongs in
that harness's adapter, not in shared text.

## Placement — the open question

Presumed **project-local**, which is where doctrine's install model already
lives: `.doctrine/skills` is the framework-owned canon and each harness links it
(`harness.pi` / `harness.universal` → `.agents/skills`, `harness.claude` →
`.claude/skills`).

User-local deserves a real answer rather than a default, because it is the
current state and it is *why* the packaging divergence went unnoticed: a copy no
installer knows about is a copy no installer refreshes, and nothing reports that
codex's charter is inert. It is also not an existing doctrine altitude for
skills or defs — `--global` is the only path to `$HOME`, and it would put the
tier into every project on the machine, doctrine-governed or not. Decide it
explicitly.

## What a fix has to account for

1. **Invert the convention to codex's: one body, three stubs.** Canonical
   charter text lives in the shipped skill's `references/`; each harness gets a
   thin def carrying only harness-native instantiation keys (pi:
   `allowNestedSubagents`/`allowedAgents`; claude: `model`/`maxTurns`/`tools`)
   plus a pointer that says read the reference. Two zero-cost gains: the body has
   exactly one home, and the seat and its contract version together. pi's def is
   already stub-shaped. Codex needs no def at all — which is the argument for
   not shipping it one once IMP-515 lands. Same split IMP-523 needs for the
   capsule tier; whichever lands first sets the pattern.
2. **No baked model.** `sub-orchestrator.md` pins `model: opus`, and
   `install/agents/claude/capsule-*.md` pins `opus`/`sonnet`. A shipped version
   must let the operator's configuration decide; the pi skill is already
   explicit about this ("do not automatically switch to GPT, Sonnet or Opus").
3. **Quotient at a declared altitude, if it ships at all.** Either it drops out
   of the canonical text and survives only as a delegation adapter behind a
   doctrine-owned contract, or the tier declares it as a named requirement with
   a version floor and a clean failure when absent. "Blocking preflight item" is
   neither.
3. **Confinement is not portable.** The pi text is jail-shaped and correct for a
   confined worker; the claude/codex tier runs outside any jail on the host's own
   auto-mode permissions. Neither assumption may leak into the other's copy.
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
  SL-281 (`skills_via = "universal"`), which is why its charter copy is inert.
  A loadable codex sub-orchestrator depends on that card.
- **IDE-053** (orchestrator/worker role prompts for the clone workflow) — the
  orchestrator concept is split across that deletion, the capsule tier and this
  tier.
- Governed by **POL-003** (harness vocabulary in adapters) and **POL-002
  facet 3** (no silently assumed host tool — quotient). The opt-in-supplement
  question POL-003 decides was raised as IDE-034.
