# Review RV-095 — design of SL-117

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This inquisition interrogates the SL-117 design — a single `bool` config key
(`claude-force-subprocess-dispatch`) added to `DispatchConfig` and a ~3-line
routing prose change in `dispatch/SKILL.md`.

### Lines of attack

1. **Config correctness.** Does the bool default, parse, and round-trip without
   regressing existing tests? Is the absent==false semantic watertight?

2. **Routing prose precision.** Does the step 3 rewrite actually give an
   orchestrator LLM enough to route correctly? Or does it leave ambiguity about
   how to detect the orchestrator's own harness?

3. **Interaction with preferred-subprocess-harness.** The design claims
   orthogonality — does the routing prose wire the two keys together correctly, or
   does "respect preferred_subprocess_harness" point at a dispatch-subprocess skill
   that doesn't yet consume that config?

4. **Doctrinal alignment.** Does the design contradict ADR-011 (Claude's Agent
   tool is first-class, not a degraded rung)? Does it violate any convention or
   storage rule?

5. **Completeness.** Are there missing test cases, unstated invariants, or
   silent assumptions the design assumes but doesn't name?

## Synthesis

*Let the record show the accused confessed under minimal duress — the design's
bones are righteous, but its skin bore eight bleeding wounds where prose failed
to meet the pyre. All wounds are dressed; the design walks free, scarred but
pure.*

### Verdict

**No heresy.** The design's core decisions — bool key, default `false`,
orthogonal to `preferred-subprocess-harness`, config-first routing with
env-marker fallback — are doctrinally sound. ADR-011 is honoured: this is an
opt-out, not a demotion of the native subagent arm.

### Penance (ordered)

1. **F-1/F-5 — Remove the dead parenthetical.** Strike "(respect
   preferred_subprocess_harness)" from the step 3 routing prose.
   `preferred-subprocess-harness` consumption in `dispatch-subprocess` is IMP-101
   scope and not yet wired. Replace with a concrete fallback: default to `pi`
   until IMP-101 lands. (blocker + major)

2. **F-3/F-8 — Add config file path and absent-file handling.** Step 3 prose must
   say: "Check `doctrine.toml` → `[dispatch]` →
   `claude-force-subprocess-dispatch`. If the file is absent, the default is
   `false`." (major + minor)

3. **F-2 — Move env-marker detection into the skill prose.** The
   `.claude/`-presence detection mechanism currently lives only in design.md.
   Step 3 must include it: "If orchestrator has `.claude/` directory → Claude;
   otherwise → codex/pi." (major)

4. **F-4 — Name the dtoml test location.** Verification table: specify
   `src/dtoml.rs` alongside the existing `dispatch_table_roundtrip` test.
   (minor)

5. **F-7 — Update skill frontmatter description.** Append mention of the config
   override to the dispatch SKILL.md description line. (nit)

### Standing risks

- **IMP-101 dependency.** `preferred-subprocess-harness` remains unwired in
  `dispatch-subprocess/SKILL.md`. SL-117's routing prose now names a concrete
  fallback (`pi`), so the system degrades gracefully — but the full codex-vs-pi
  selection awaits IMP-101.

- **Prose-only enforcement.** The config key has no binary consumer — correctness
  rests on orchestrator LLMs faithfully reading and applying skill prose. This is
  the same posture as `preferred-subprocess-harness` and consistent with the
  dispatch framework's design (config is advisory, orchestrator is the consumer).

### Harvest

No durable knowledge to harvest — the findings are specific to this design's
prose imprecision and are remedied here.
