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
adapts). Its *packaging* has converged on nothing: three harnesses, three
loading conventions, one of them inert.

## Observed state

Viewed from a confined pi session, `~/` is a persistent share rather than the
operator's host home, so claude's and codex's user dirs are copied in for
inspection — `~/_claude_skills/`, `~/_claude_agents/`, `~/_codex_skills/`; pi's
are live under `~/.pi/`.

| artefact | claude | codex | pi |
|---|---|---|---|
| `super-orchestrator` seat | `~/.claude/skills/…`, 11 317 B / 211 lines | 11 317 B — **byte-identical** | `~/.pi/skills/…`, 7 800 B / 144 lines — *adapted* |
| `pi-orchestrate` | 1 394 B / 23 lines | 1 394 B — **byte-identical** | 3 106 B / 53 lines — *expanded* |
| sub-orchestrator charter | `~/.claude/agents/sub-orchestrator.md`, 5 817 B — **claude-native location** | `super-orchestrator/references/sub-orchestrator.md`, 5 817 B — **byte-identical copy, in a location codex cannot load as an agent** | `~/.pi/agent/agents/sub-orchestrator.md`, 2 274 B, pi-native frontmatter |

Adjacent but not of this tier: `arbiter-review` (7 560 B) is claude-only.

## What that says — content is 2-shape, packaging is 3-shape

1. **The text split already exists.** claude and codex share the seat skill and
   `pi-orchestrate` byte-for-byte; pi is a real adaptation (jail-aware: "run
   inside the existing jail", "never assume they can read host paths mentioned
   in an older handover", "do not require Quotient"). The charter is the same
   file in claude and codex too. Nothing needs inventing — one canonical content
   set plus a pi adaptation, exactly as field-converged.
2. **Packaging is the actual defect.** claude loads the charter from
   `~/.claude/agents/`; pi from `~/.pi/agent/agents/`; codex has **no agents dir
   at all** (`~/.codex/agents` absent, and post-SL-281 doctrine installs no
   codex agent defs — IMP-515), so its copy sits inert inside a skills tree as
   documentation. The charter therefore *appears* present in all three and is
   loadable in two — the same invisible-asymmetry failure as IMP-468 and
   IMP-523, now in the third dimension: location rather than prose.
3. **The seat is defined against two other skills.** Its description reads
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
current state and it is *why* the packaging divergence went unnoticed: a copy no
installer knows about is a copy no installer refreshes, and nothing reports that
codex's charter is inert. It is also not an existing doctrine altitude for
skills or defs — `--global` is the only path to `$HOME`, and it would put the
tier into every project on the machine, doctrine-governed or not. Decide it
explicitly.

## What a fix has to account for

1. **One charter body, three wirings.** The def frontmatter is per-harness (pi:
   `allowNestedSubagents`/`allowedAgents`; claude: `model`/`maxTurns`/`tools`/
   `TaskStop`; codex: `.codex/agents/*.toml`, gated on IMP-515) while the body
   is shared — and the harness's *loading convention* is a third variable the
   shipping mechanism must own, or the codex copy repeats this bug. Same split
   IMP-523 needs for the capsule tier; whichever lands first sets the pattern.
2. **No baked model.** `sub-orchestrator.md` pins `model: opus`, and
   `install/agents/claude/capsule-*.md` pins `opus`/`sonnet`. A shipped version
   must let the operator's configuration decide; the pi skill is already
   explicit about this ("do not automatically switch to GPT, Sonnet or Opus").
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
- Governed by **POL-003**; the opt-in-supplement question it decides was raised
  as IDE-034.
