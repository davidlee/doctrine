# IDE-033: Prime Read-first after 'doctrine new' to avoid doubled large-payload Write on scaffolded templates

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`doctrine <kind> new` scaffolds a template file on disk. An agent then authors
into it with `Write`/`Edit`. The Claude harness enforces a read-before-write
gate: `Write`/`Edit` on any file not `Read` this session is rejected. So a raw
write of a large authored payload (e.g. 40k tokens of design prose) fails, and
the payload must be regenerated after a forced `Read` — the generation cost is
paid twice.

## Ruled out

- **Content-injection does not help.** Injecting the template body via a
  `PostToolUse` `additionalContext` block (or `cat`) does NOT flip the harness
  read-bit — confirmed empirically. Only a real `Read` tool-call satisfies the
  gate. So injecting content can't unlock the `Write`.
- **Tweaking `new` stdout to shout "Read first" — rejected.** It bakes
  Claude-specific harness behaviour (the read-before-write gate is a Claude
  harness trait) into an agent-neutral CLI. Violates the platform-independence
  posture (POL-002). A `PostToolUse` hook could do the same priming without
  touching the tool — but priming only *nudges*; it relies on agent adherence
  and still pays one generation.

## Chosen direction

Build a **gate-free MCP authoring verb** that collapses `new` + `edit` into one
call: create the entity and fill its body in a single MCP tool invocation. MCP
tools are not `Write`/`Edit`, so they bypass the harness read-before-write gate
entirely (cf. existing `memory_record` / `memory_edit` / `review_new`, which
already author without the gate). The payload is generated once, as the MCP
tool's argument, and lands directly — no scaffold-then-write round trip, no
doubled generation, no Claude-specific leakage into the CLI.

Scope note: today only memory/review have MCP authoring. Slices, specs, ADRs,
backlog items author through raw `Write`/`Edit` and therefore hit the gate —
those are the kinds that would gain a create-with-body MCP verb.

## Status

Parked (idea). Needs a slice: scope which kinds get MCP authoring, and design
the create-with-body contract. Probe artifacts (`new-readfirst-probe.sh`) were
exploratory only — the hook path is superseded by the MCP direction.

Related: IDE-032 (auto-surface memories via PreToolUse hook — sibling probe).
