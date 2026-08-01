# DEC-103: Spike worker realism

**Decision.** The SL-241 rig's worker defaults to a deterministic **stub**.
A real agent runs **once**, explicitly opted into with `--agent`.

Everything in P-C2 and P-C3 is scripted by mandate anyway — DQ-2 voids a probe
"contained" by a worker politely declining, so hostile workers are `bash -c`
mutators, not agents. Only P-C1 needs an LLM at all.

## Consequent reordering

`probe-specs.md` § Order and gating puts P-C1 first. Its *agent leg* goes last:

1. **P-C1a** — deterministic: clone, provision, nix, build, test, harvest cost.
   Stub worker. Banks every measurement except tokens.
2. **A2 smoke** — a trivial `claude -p 'print OK'` inside the sandbox. Near-free,
   run early, purely to prove the jail's `~/.claude` credential arrangement
   survives nested bwrap.
3. **P-C2**, then **P-C3**.
4. **P-C1b** — the real agent executing a real red→green phase: the token
   measurement, and "does a phase actually reach green in a capsule".

## Why

More than one full run of the rig will be needed, and most of what the rig
settles does not need a slow, expensive, non-deterministic agent. Settling the
deterministic surface first makes re-runs cheap and regression checks possible;
anyone re-executing the rig later should not have to burn an agent to re-measure
disk.

Splitting the A2 smoke out of P-C1b is the counterweight: if the credential
arrangement does not survive nested bwrap, the capsule model needs a
credential-proxy design, and that is worth learning on day one rather than at
the end. Assumption A2 (slice-241.md) is the target.

## Related

- `probe-specs.md` § P-C1, DQ-2, § Order and gating.
- SL-241 § Risks — assumptions A1 (nested bwrap) and A2 (credentials).
