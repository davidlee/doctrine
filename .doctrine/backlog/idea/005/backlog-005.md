# IDE-005: Detect harness from ENV inside doctrine commands to shrink the skill decision surface

## Idea

A `doctrine` command can likely detect its running harness directly from the
process environment (at minimum **claude** via `CLAUDECODE`; probably codex/pi
equivalents). If the binary owns harness detection, the per-harness decision
surface now scattered across the `/dispatch-*` skills (the env-marker probe +
self-belief cross-check — SL-056 design, ex-Charge ε/ι) could collapse into a
single CLI surface (e.g. `doctrine harness detect` or an internal probe the
dispatch verbs consult), letting the router skills get dumber.

## Why this is interesting

- **Pushes mechanism into the verb** — exactly SL-056's thesis. The router's
  env-marker logic is currently destined to live in skill prose; a binary detector
  makes it testable and harness-uniform.
- **Shrinks skills.** `/dispatch-subprocess` vs `/dispatch-agent` selection, and
  the cross-check against self-belief, could be one CLI call.

## Caveats / open

- Env markers are **launch-mode-fragile** (headless / cron / nested / IDE vary) —
  the same version-fragility SL-056 spike-gates. A binary detector inherits that;
  it must be spike-backed + named-fallback, not assumed.
- Self-belief vs detection cross-check still matters (a binary detector is *one*
  signal; the spawning agent's self-knowledge is another). Decide whether the
  binary arbitrates or just reports.
- Scope boundary vs SL-056: SL-056 ships the skill-level router; this is the
  *follow-up* that may absorb it. Do not expand SL-056 to cover it.

Surfaced during SL-056 design (8th-inquisition redesign).
