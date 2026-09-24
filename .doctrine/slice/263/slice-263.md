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
format + seen-set/log IO). No new retrieval logic. The engine's `ScopeProbe`
path arm widens to a path *set* so a multi-file patch is one query rather than
N — arity, not a new query, admission rule, ranking or tuning knob.

- **`memory surface` output form.** Add `--format <claude|plain>`, default
  `claude` (back-compat for the shipped Claude/`.claude/settings.json` entries).
  `plain` emits the bare block text (or nothing). The input envelope, admission
  gate, caps and dedup are unchanged; the tuning log gains a `wire` field.
  Codex requires the envelope
  (it ignores plain `stdout` on `PreToolUse`); pi needs the bare block.
- **`memory surface` input wire.** Add `--input <claude|codex|neutral>`, default
  `claude`, selecting the codec that normalises stdin into the doctrine-owned
  neutral surface request. Canonical hook commands become explicit
  (`memory surface --input claude` / `--input codex`); a bare `memory surface`
  stays a parse-time alias for `claude` **and stays owned**, so the Claude
  install refreshes an un-upgraded entry rather than double-wiring it
  (`DEC-281`).
- **codex command surface.** Add a `memory surface --input codex` `PreToolUse`
  spec to the codex hook registry (`.codex/hooks.json`). Codex's hook layer
  canonicalises shell calls to `tool_name: "Bash"` with `tool_input.command`; the
  codex codec maps that to a command request, so the neutral decoder is
  untouched.
- **codex path surface.** Codex file edits arrive as the single `apply_patch`
  tool with the patch body in `tool_input.command` (no `file_path`). Add a pure
  patch-path extractor over codex's `apply_patch` envelope grammar (the
  `*** Update File:` / `*** Add File:` / `*** Delete File:` / `*** Move to:`
  headers) producing a path probe, so codex edits reach the path surface — with
  the delta that codex has no read tool (reads are shell calls) and that
  `apply_patch` fires after the patch is written.
- **pi surface adapter.** Generate a `.pi/extensions/doctrine/surface.ts`
  module (sibling to the existing `mcp.ts` bridge, imported by `index.ts`). On
  `tool_result`, map pi's tool names (`read`/`edit`/`write` → `path`;
  `bash` → `command`) into the neutral envelope, invoke
  `memory surface --input neutral --format plain`, and append the returned block
  to `event.content`, passing `session_id` from `ctx.sessionManager`. Strictly
  fail-open: any failure returns `undefined`. Subagent suppression is accepted
  as a v1 delta — no harness-native signal exists (`DEC-286`).
- **Installer wiring.** The codex arm grows a `codex_hook_specs` registry (two
  `PreToolUse` matcher groups, `Bash` and `apply_patch`, each with an explicit
  `additionalContextLimit`), replacing the single inline call, with the `/hooks`
  trust notice extended; `HookSpec` gains an optional limit field (`DEC-283`).
  The Claude canonical command becomes `memory surface --input claude`, and
  `plugins/doctrine/hooks/hooks.json` moves with it. The pi install grows the
  `surface.ts` module (generate / regenerate / foreign-skip, mirroring
  `install_mcp_extension`). All outcomes are reported (STD-003).
- **Neutral surface envelope (settled; `DEC-280`/`DEC-281`).** A doctrine-owned
  `SurfaceRequest { Path, Command, Patch }` in `src/memory.rs`; `claude_request`,
  `codex_request` and `neutral_request` normalise each wire into it, and
  `probe_for` becomes `probes_for(request, root)` — so the neutral pipeline
  (`retrieve_rows` + `admits`/`dedup_diff`/`cap`/`format_block`) is composed
  once, unchanged, and never learns a harness tool name. `apply_patch`'s reader
  is a neutral pure function: *patch text → root-relative paths* (`DEC-285`).
- **Governance leg (in scope; drafted after design locks, applied at reconcile).**
  This slice also closes the governance gaps research `F1`/`F4` surfaced —
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
  - **A `SPEC-011` revision** covering the pi/codex boot sector this slice grows.
    `SPEC-011`'s responsibilities stop at *"doctrine's owned **Claude** hook
    SET"*; its members (REQ-185/186/476/477) govern the Claude settings merge
    only, so `install_codex_hook` and the pi extension generators are
    requirement-dark (research `F4`). The revision (a REV of SPEC-011, not a new
    spec) adds the codex hook registry and the generated pi extensions to the
    installer's governed surface. User ruling (2026-09-24): dealt with at
    reconcile with the other two.
  - **A `SPEC-007` revision** (`REV-060`), added at the governance review:
    `SPEC-007` owns `src/memory.rs` and `src/retrieve.rs` but describes only
    `find` / `retrieve`, so the pointer surface had no memory-side contract. It
    adds a pointer-surface responsibility and `FR-008`.

## Non-Goals

- **Not** subagent surfacing on pi or codex. `SL-205`'s `INV-3` (main-thread
  only) rides Claude's `agent_id`; neither pi's `tool_result` nor codex's
  `PreToolUse` supplies an equivalent here. On codex the delta is compounded:
  subagent hooks report the **parent** session id, so the seen-set is shared
  across that boundary. Documented as a delta; widening is a follow-up.
- **Not** a new `retrieve` query, a new admission rule, or new tuning knobs —
  the retrieval logic is `SL-205`'s, unchanged. Only the probe's path arity
  widens (one path → a path set).
- **Not** a `memory surface` MCP operation (the zero-subprocess alternative for
  pi). Evaluated and deferred; `DEC-289` chooses the per-call spawn, with the
  already-live MCP child as R-1's named fallback (§6/§7).
- **Not** Cursor (`IMP-245`) or any other harness.
- **Not** promoting pi to a first-class `Harness` variant, even though the
  installer's current hosting of pi's extensions on the codex arm is a wart this
  slice will work beside (see Open Questions).

## Affected surface

- `src/memory.rs` — `--format` and `--input` plumbing `MemoryCommand::Surface` →
  `run_surface` → `run_surface_to` → `emit_surface` (envelope vs bare); the
  `SurfaceRequest` neutral core and the `claude`/`codex`/`neutral` codecs
  (replacing `probe_for`); the pure `paths_from_patch` extractor beside the
  existing pure helpers.
- `src/retrieve.rs` — `ScopeProbe`'s path arm carries a path set, mapped to the
  `QueryContext.paths` the engine already ranks across. No query, admission rule,
  ranking or tuning change.
- `src/boot.rs` — the `codex_hook_specs` registry and its matcher constants;
  `HookSpec`'s optional **handler** `additional_context_limit` / `timeout` and the
  extended canonicality comparison; the canonical Claude args
  (`memory surface --input claude`) and the ownership predicate that still owns
  the legacy bare form (mirroring `is_doctrine_emit_command`); the generated pi
  surface module's `generate_/plan_/install_` triad beside
  `plan_mcp_extension`/`install_mcp_extension`; the `RefreshReport` third field +
  report leg; the codex-arm registry loop and third installer call.
- `plugins/doctrine/hooks/hooks.json` — the published Claude hook commands move
  to `memory surface --input claude`.
- `templates/**` or a `format!` literal — the pi surface module's source, per the
  `mcp.ts`-template vs `index.ts`-literal precedent (design decides).
- `src/commands/guard.rs` — no new command, but confirm the `MemoryCommand`
  match stays exhaustive.
- `.doctrine/policy/` — **POL-003** (authored from `IDE-034`), after design locks.
- `.doctrine/spec/product/004/` + a REV — the **PRD-004** reconciliation, applied
  at reconcile.
- `.doctrine/spec/tech/011/` + a REV — the **SPEC-011** revision extending its
  responsibilities/members to the codex hook registry and the generated pi
  extensions, applied at reconcile.
- `.doctrine/spec/tech/007/` + a REV — the **SPEC-007** revision (`REV-060`)
  adding the pointer-surface contract, applied at reconcile.
- `tests/**` — hook envelope emission per format; installer goldens.
- Behaviour-preservation gate: the existing memory + retrieve + boot suites stay
  green unchanged.

## Risks / Assumptions

- **R-1** pi pays one binary spawn per main-thread `read`/`edit`/`write`/`bash`
  (a process spawn). Claude pays the same per hook. Measure; the silent-when-no-hit
  property keeps it cheap.
- **R-2** codex requires `/hooks` trust review for a new non-managed hook; until
  trusted it is skipped, silently. The install's manual-steps notice already
  names this — extend it, do not let the new spec look installed.
- **R-3** pi's `tool_result` injection is *post*-execution, where Claude's and
  codex's `PreToolUse` are *pre*-. For pi's read→edit workflow the nudge still
  lands before the model composes the edit (it follows the `read`); for a bare
  `edit`/`write` it is retrospective. Codex has **no read surface at all** (reads
  are shell calls), and its only path trigger, `apply_patch`, fires after the
  patch is written — so its every path nudge is retrospective. State both as
  capability deltas, not defects.
- **R-4** codex's `additionalContextLimit` default (reported as ~2500 tokens; the
  field's unit is not established) and its handler `timeout` are values we did
  not choose. Both are set explicitly, on the **handler** where codex reads them.
- **R-5** codex trusts a hook by a hash of its handler definition, and doctrine's
  baked absolute exec path changes on every upgrade — so the new groups go inert
  until re-trusted.
  The install discloses the manual step; the runtime skip is a documented delta.
- **R-6** the codex wire shape (string vs argv `command`, wrapped commands,
  `apply_patch` through the shell) is documented rather than observed. Retired by
  a phase-1 payload-capture gate with checked-in fixtures.
- **A-1** Assumes the codex `PreToolUse` wire shape and `apply_patch`
  `tool_input.command` field are stable enough to depend on (same class of
  assumption `SL-205` `A-1` made for Claude).

## Open Questions

- **OQ-1 → RESOLVED (in scope).** Ship the `apply_patch` patch-path extractor
  here. A slice that ships codex surfacing with only the command surface is a
  half-port, and the extractor is the only genuinely new *logic* in the slice —
  everything else is glue.
- **OQ-2 → RESOLVED (`DEC-282`).** `--format plain` emits the bare block as a
  doctrine-owned output form; the generated TS adapter is a dumb consumer and
  does not parse the envelope.
- **OQ-3 → RESOLVED (`DEC-286`).** Accept the `INV-3` parity delta for v1 — pi
  surfaces inside subagent sessions too. No harness-native signal exists;
  `PI_SUBAGENT_CHILD` is a pi-subagents package marker, not a pi seam.
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
- Captured codex `PreToolUse` fixtures (shell, `apply_patch`-as-tool,
  `apply_patch`-via-shell) drive the codec VTs; a phase-1 gate captures them and
  checks them in — an orchestrator/human (`VH`) step before the codec phase, not
  confined-worker work.
- `memory surface` exercised for every input wire and output form: `claude`,
  `codex` and `neutral` each decode to the same request; `plain` emits the bare
  block, `claude` the envelope, empty stays empty, exit 0 on every path. A bare
  `memory surface` decodes as `claude`; a run from a subdirectory cwd resolves
  cwd-relative paths correctly.
- `paths_from_patch` unit-tested with synthetic patches (update / add / delete /
  move-to headers, malformed input); the widened `ScopeProbe` admits a memory
  anchored on any of its paths.
- Installer goldens: the two codex `PreToolUse` specs round-trip through the
  merge core (wired / idempotent / foreign-preserving / staleness-refreshed,
  mirroring the existing codex `SessionStart` tests), including the
  `additionalContextLimit`; a legacy bare Claude entry is refreshed to the
  explicit form rather than double-wired; the generated `surface.ts` is
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
- **Cursor** (`IMP-245`).
