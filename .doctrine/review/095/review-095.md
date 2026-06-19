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
