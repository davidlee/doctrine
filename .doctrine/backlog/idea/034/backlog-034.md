# IDE-034: Policy: harness-specific behaviour ships as opt-in supplement on doctrine-owned contracts (candidate POL-003)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Idea

Promote the principle applied ad hoc in SL-205 into explicit governance:

> Harness-specific behaviour (Claude Code hooks, pi/codex equivalents) ships as
> an **opt-in supplement** whose correctness rests only on **doctrine-owned
> contracts** — never baked into the neutral core, never load-bearing on a host
> harness's incidental seams.

Today this is inferred from ADR-011 (harness-agnostic orchestrator spawn) as a
*precedent* and enforced case-by-case. SL-205 (ambient memory hooks) is the
second instance; the forthcoming pi + codex ports are the third and fourth. A
recurring principle applied across ≥4 sites is policy-shaped.

## Decide when authoring

- **Altitude** — POL (a required rule of the road) vs ADR (a decision record).
  ADR-011 is the decision; a POL would make the *rule* enforceable. Likely POL,
  numbered next in sequence (POL-003 if free).
- **Relation to POL-002** — sibling. POL-002 forbids load-bearing on *host
  project* conventions; this would forbid load-bearing on *host harness* seams
  and baking harness glue into neutral core. Same "strict-and-owned beats
  lenient-and-coupled" ethos, different coupling axis.

Origin: SL-205 Follow-Ups (harness-agnosticism). See also ADR-011, POL-002.
