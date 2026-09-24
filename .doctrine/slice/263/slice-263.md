# Ambient memory surfacing for pi and codex

## Context

`SL-205` shipped ambient memory surfacing as a binary subcommand (`doctrine
memory surface`) plus per-harness hook glue. It targets **Claude Code only**:
the command reads Claude's `PreToolUse` stdin envelope and emits Claude's
`hookSpecificOutput.additionalContext` line, and only `.claude/settings.json`
gets the two `PreToolUse` entries. `SL-205`'s own Follow-Ups named the gap —
*"Harness ports. This slice targets the Claude Code hook contract. Sibling ports
for other harnesses are follow-up slices"* — with **pi** and **codex** listed
first. This slice is that port.

pi and codex are the other two harnesses `doctrine boot install` wires. Both can
inject model-visible context at tool-call time, by different seams:

- **pi** has no hook-config file. Its extension API exposes `tool_result`
  (post-execution), whose result may carry replacement `content`. There is **no
  pre-execution context channel**: `tool_call` can only `block` or mutate input.
- **codex** has a full hooks engine whose wire protocol is deliberately
  Claude-compatible: `PreToolUse` on stdin (`session_id`, `cwd`, `tool_name`,
  `tool_input`) and `hookSpecificOutput.additionalContext` on stdout. Codex
  shipped `PreToolUse.additionalContext` support in August 2026 (openai/codex
  issue #19385, closed 2026-08-04).

The port is therefore asymmetric. **Codex is mostly configuration** — a new
`PreToolUse` spec in the codex hook registry, reusing the existing handler
verbatim for the command surface. **pi needs a generated TypeScript adapter**
plus a way to obtain the bare block rather than the Claude JSON envelope.

**Authority note (research `F1`).** `ADR-011` governs this change by its
**surviving first theme only** — *mechanism moved into the binary is identical
under every harness by construction*; its second theme (per-harness capability
altitude) is self-declared falsified by `SL-254` and must not be leaned on. The
rule that actually governs adapter placement is `IDE-034` / candidate `POL-003`,
which this slice authors (governance leg below). `POL-002` constrains the neutral
core against host-*project* coupling but does **not** reach the host-*harness*
axis.

## Scope & Objectives

Deliver the pi and codex ports on the SAME neutral core `SL-205` established
(the `retrieve` query + severity/staleness admission + session dedup + cap +
format + seen-set/log IO). No new retrieval logic.

- **`memory surface` output form.** Add `--format <claude|plain>`, default
  `claude` (back-compat for the shipped Claude/`.claude/settings.json` entries).
  `plain` emits the bare block text (or nothing). The input envelope, admission
  gate, caps, dedup and tuning log are unchanged. Codex requires the envelope
  (it ignores plain `stdout` on `PreToolUse`); pi needs the bare block.
- **codex command surface.** Add a `memory surface` `PreToolUse` spec to the
  codex hook registry (`.codex/hooks.json`) with a codex matcher. Codex reports
  shell tools as `tool_name: "Bash"` with `tool_input.command`, which the
  existing handler already discriminates — so this leg is config + registry
  only.
- **codex path surface.** Codex file edits arrive as the single `apply_patch`
  tool with the patch body in `tool_input.command` (no `file_path`). Add a pure
  patch-path extractor (the `*** Update File: <path>` / `*** Add File:` /
  `*** Delete File:` headers) producing one or more path probes, so the path
  surface fires on codex the way it fires on Claude's `Read|Edit|Write`.
- **pi surface adapter.** Generate a `.pi/extensions/doctrine/surface.ts`
  module (sibling to the existing `mcp.ts` bridge, imported by `index.ts`). On
  `tool_result`, map pi's lowercase tool names (`read`/`edit`/`write`/`bash`)
  and input keys (`path` / `command`) onto the surface envelope, invoke
  `memory surface --format plain`, and append the returned block to
  `event.content`. Strictly fail-open: any failure returns `undefined`.
- **Installer wiring.** The codex arm grows the surface spec in its registry
  (with the `/hooks` trust notice already printed); the pi extension install
  grows the `surface.ts` module (generate / regenerate / foreign-skip, mirroring
  `install_mcp_extension`). Both report their outcomes.
- **Neutral surface envelope (design input, research `F2`).** The naive port adds
  codex's `apply_patch` (and, if mapping moves to Rust, pi's lowercase names) as
  arms on `probe_for` (`src/memory.rs:10533`), baking harness seams into the
  neutral core — exactly what `IDE-034` forbids. The design should instead
  introduce a doctrine-owned, harness-neutral surface envelope (a tool *class* +
  value) that each adapter normalises *into*, so `probe_for` never learns a
  harness name and the pipeline is called once, unchanged. The `apply_patch`
  reader's contract is then neutral: *patch text → root-relative paths*.
- **Governance leg (in scope; drafted after design locks, applied at reconcile).**
  This slice also closes the two governance gaps research `F1`/`F4` surfaced —
  deliberately *after* the port's design locks, so the governance text is drafted
  against a locked design rather than ahead of it. Sequencing (user ruling,
  2026-09-24): `/design` locks → mint a REV placeholder → draft any time → apply
  at `/reconcile`.
  - **POL-003, authored from `IDE-034`** — *harness-specific behaviour ships as an
    opt-in supplement whose correctness rests only on doctrine-owned contracts,
    never baked into the neutral core, never load-bearing on a host harness's
    incidental seams.* `IDE-034`'s own threshold makes this slice its fourth
    instance, i.e. policy-shaped; once authored it is the load-bearing authority
    for adapter placement.
  - **A `PRD-004` revision** reconciling §2's *"Proactive, unsolicited injection
    of memories into a context ahead of demand"* and §8's blocking Open Question
    with the shipped mechanism. The reading to argue: the surface injects
    **concise pointers** (`[triage] title — uid`), never memory bodies, delivered
    at the moment of demand (a tool call keyed on the path or command) rather
    than carrying memory payload *ahead of* demand.

## Non-Goals

- **Not** subagent surfacing on pi or codex. `SL-205`'s `INV-3` (main-thread
  only) rides Claude's `agent_id`; neither pi's `tool_result` nor codex's
  `PreToolUse` supplies an equivalent here. Documented as a delta; widening is a
  follow-up.
- **Not** a new `retrieve` query, a new admission rule, or new tuning knobs —
  the neutral core is `SL-205`'s, untouched.
- **Not** a `memory surface` MCP operation (the zero-subprocess alternative for
  pi). Evaluated and deferred — see Open Questions.
- **Not** Cursor (`IMP-245`) or any other harness.
- **Not** promoting pi to a first-class `Harness` variant, even though the
  installer's current hosting of pi's extensions on the codex arm is a wart this
  slice will work beside (see Open Questions).

## Affected surface

- `src/memory.rs` — `--format` plumbing `MemoryCommand::Surface` →
  `run_surface` → `run_surface_to` → `emit_surface` (envelope vs bare); the
  neutral surface envelope and the adapter mapping into it (replacing the naive
  `probe_for` arm); new pure patch-path extractor beside the existing pure
  helpers.
- `src/boot.rs` — codex hook registry (a `memory_surface`-equivalent spec and
  its matcher constant); the generated pi surface module's `generate_/plan_/
  install_` triad beside `plan_mcp_extension`/`install_mcp_extension`; the
  `RefreshReport` third field + report leg; the codex-arm call site.
- `templates/**` or a `format!` literal — the pi surface module's source, per the
  `mcp.ts`-template vs `index.ts`-literal precedent (design decides).
- `src/commands/guard.rs` — no new command, but confirm the `MemoryCommand`
  match stays exhaustive.
- `.doctrine/policy/` — **POL-003** (authored from `IDE-034`), after design locks.
- `.doctrine/spec/product/004/` + a REV — the **PRD-004** reconciliation, applied
  at reconcile.
- `tests/**` — hook envelope emission per format; installer goldens.
- Behaviour-preservation gate: the existing memory + retrieve + boot suites stay
  green unchanged.

## Risks / Assumptions

- **R-1** pi pays one binary spawn per main-thread `read`/`edit`/`write`/`bash`
  (`execSync`). Claude pays the same per hook. Measure; the silent-when-no-hit
  property keeps it cheap.
- **R-2** codex requires `/hooks` trust review for a new non-managed hook; until
  trusted it is skipped, silently. The install's manual-steps notice already
  names this — extend it, do not let the new spec look installed.
- **R-3** pi's `tool_result` injection is *post*-execution, where Claude's and
  codex's `PreToolUse` are *pre*-. For the read→edit workflow the nudge still
  lands before the model composes the edit (it follows the `read`); for a bare
  `edit`/`write` it is retrospective. State this as a capability delta, not a
  defect.
- **R-4** codex's `additionalContextLimit` defaults to ~2500 tokens. Our blocks
  are small (path cap 3, command cap 2), but set it explicitly rather than
  inheriting a threshold whose semantics we did not choose.
- **A-1** Assumes the codex `PreToolUse` wire shape and `apply_patch`
  `tool_input.command` field are stable enough to depend on (same class of
  assumption `SL-205` `A-1` made for Claude).

## Open Questions

- **OQ-1 → RESOLVED (in scope).** Ship the `apply_patch` patch-path extractor
  here. A slice that ships codex surfacing with only the command surface is a
  half-port, and the extractor is the only genuinely new *logic* in the slice —
  everything else is glue.
- **OQ-2** pi `--format plain`: emit bare text (one output form per consumer) or
  have the TS adapter parse the shared Claude envelope (zero Rust)? Codex
  confirms the envelope is a de-facto cross-harness contract, which weakens the
  coupling argument for a second form.
- **OQ-3** pi subagent suppression: accept surfacing inside pi subagent sessions
  for v1 (documented), or find a signal (env marker / session metadata)?
- **OQ-4 → RESOLVED (follow-up).** Left as a follow-up; the modelling is
  functionally sufficient. `doctrine install --agent pi` accepts `pi` and wires
  its boot legs through the **codex** arm — the dry run reports *"boot … session
  hooks for codex"* plus *"skills for pi (delegates to npx)"* — while `doctrine
  boot install --agent pi` refuses (`Unknown harness 'pi'`). NQR: pi is not a
  boot `Harness` variant, so a claude-only repo that auto-detects without
  `--agent pi` receives no pi extensions. Not load-bearing for this port.

## Verification / Closure intent

- Pure helpers (the new patch-path extractor; existing admission/dedup/cap/
  format) unit-tested with synthetic inputs — the `SL-205` helper test shape.
- `memory surface` exercised for both formats with synthetic stdin: `claude`
  emits the envelope, `plain` emits the bare block, empty stays empty, exit 0 on
  every path.
- Installer goldens: the codex `PreToolUse` spec round-trips through the merge
  core (wired / idempotent / foreign-preserving / staleness-refreshed, mirroring
  the existing codex `SessionStart` tests); the generated `surface.ts` is
  ownership-marked, regenerate-on-change, foreign-skipped.
- Behaviour-preservation: existing memory, retrieve and boot suites green
  unchanged.
- Governance leg: POL-003 authored and `doctrine check` clean; the PRD-004 REV
  applied at reconcile; `doctrine link SL-263 governed_by POL-003` once POL-003
  exists.

## Follow-Ups

- **pi as a first-class boot `Harness` variant.** `doctrine boot install --agent
  pi` refuses and `resolve_harnesses` auto-detection never yields pi; the skills
  installer compensates by routing pi's boot legs through the codex arm. Fixing
  the modelling is orthogonal to this port.
- **Subagent surfacing on pi/codex** — the `INV-3` parity gap left open by
  OQ-3, if tightening proves worthwhile.
- **A requirement covering the codex hook registry and the generated pi
  extensions.** SPEC-011's members (REQ-185/186/476/477) stop at the Claude
  settings merge; `install_codex_hook` and the pi extension generators have no
  requirement (research `F4`). Lighter than the two governance gaps this slice
  closes; candidate REV of SPEC-011 rather than a new spec.
- **Cursor** (`IMP-245`).
