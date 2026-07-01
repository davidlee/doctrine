# IMP-225: Subprocess (pi/codex) Seatbelt backend reusing seatbelt_profile (macOS jail parity for the subprocess arm)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

The residual of IMP-045 after SL-183. SL-183 delivered the macOS Seatbelt
write-confinement floor for the **claude PreToolUse arm** (the shared
`seatbelt_profile`/`sandbox_exec_argv` builders + `resolve_inputs` shell +
`select_jailer` macOS branch). IMP-045's original framing (from SL-056) also
covers the **subprocess (pi/codex) jail seam** — the dispatch-worker sandboxing
axis — which SL-183 scoped OUT as a Non-Goal + Follow-Up.

## Detail

The subprocess Seatbelt backend can adopt SL-183's `seatbelt_profile` builder
as-is; the fork is the launcher/argv shell (subprocess spawn vs the claude
PreToolUse hook). Reuse the same `Decision`/`Target`/policy/funnel machinery —
ride the existing seam, no parallel implementation.

Refs: SL-183 (Non-Goals + Follow-Ups), IMP-045 (originating item, claude-arm
portion now fulfilled partial), SL-056 (original subprocess sandboxing axis),
mem.pattern.seatbelt.profile-materialization-command-tier.
