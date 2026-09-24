# Notes SL-263: Ambient memory surfacing for pi and codex

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage

Recorded at the `exploring` runbook step `explore.triage`. Full detail in
`research/research.md` (runtime tier).

### Constraining governance

- **PRD-004 §2 out-of-scope** (*proactive, unsolicited injection ahead of
  demand*) + **§8 Open Question** (blocks pre-emptive surfacing + its trust bar).
  The spec-dark gap; the governance leg closes it via a PRD-004 REV at
  reconcile.
- **PRD-007 §2 out-of-scope** (*per-turn injection of governance content*).
- **ADR-011** — governs by its surviving first theme only (mechanism-in-binary);
  the per-harness-altitude theme is falsified by SL-254 and must not be leaned on.
- **POL-002** — constrains the neutral core against host-*project* coupling;
  does **not** reach the host-*harness* axis.
- **IDE-034 / candidate POL-003** — the rule for adapter placement (opt-in
  supplement on doctrine-owned contracts); this slice is its 4th instance by
  IDE-034's own threshold, hence the governance leg.
- **STD-001** (single-source literals), **STD-003** (a skipped/failed install leg
  must be named, not absorbed), **ADR-001** (layering — `memory` and `boot` are
  command tier; no new module).
- **REQ-018 / REQ-151 / REQ-152** — render-as-data + non-bypassable trust
  holdback; the new arms must not regress them.

### Shaping decisions (agent-proposed, carried into `/design`)

- **D1** A doctrine-owned, harness-neutral **surface envelope** (a tool *class* +
  value) that each adapter normalises *into*; `probe_for` never learns a harness
  tool name or input key. (research `F2`; the concrete reading of IDE-034.)
- **D2** Output form is explicit — `--format claude|plain`. Claude and codex
  require the JSON envelope; pi needs the bare block.
- **D3** The `apply_patch` reader is a neutral *patch text → root-relative paths*
  function; codex's tool dispatch is a thin call onto it.
- **D4** Ports ride the existing neutral pipeline unchanged — one call site; no
  forking of `admits`/`dedup_diff`/`cap`/`format_block`.

### Open questions carried forward

- **OQ-2** pi `--format plain` (bare text) vs TS parsing the shared Claude
  envelope (zero Rust).
- **OQ-3** pi subagent suppression: accept surfacing in pi subagent sessions for
  v1, or find a signal.

### Retrieved memories (design inputs)

- `mem_019f139f1dc071829a885949da8203a3` — **three parallel agent-detection
  resolvers** (`install::detect_agents`, `skills::resolve_agents`,
  `boot::resolve_harnesses`); any agent-detection change must update all three.
  Bears on OQ-4's pi modelling and on any tweak to install harness resolution.
- `mem.pattern.testing.absence-assertion-over-an-unwritten-file` — `doctrine
  install --agent claude` historically discarded the explicit agent, and `pi`
  maps onto the codex arm (`install_harnesses`/`boot_harness_names`). Install
  tests need a **positive control**; an `is_empty()` assertion can pass because
  nothing ran.
- `mem.pattern.rust.fail-soft-ok-defeats-sequencing` — `plan_hook`'s malformed
  inputs fail soft as `PrintedFallback` through `Ok(..)`; ordering safety must
  guard on the **outcome value**, never `?`. Bears on the new install legs and
  the STD-003 disclosure.
- `mem.pattern.distribution.settings-key-rides-beside-hookspec-merge-core` — the
  merge core is owner-locked/shared; unrelated additions are separate planners
  beside it, never threaded through `plan_hook`.
- `mem.pattern.build.jail-binary-for-skill-install` (stale) — run `install`/`boot`
  from the freshly built in-tree binary, not PATH.

## Review

RV-377 (design pass, `human-only`). One hostile pass conducted in-session; the
four findings raised were all fixed in the design before lock:

- **F-1 (major)** — codex's `PreToolUse` carries no `agent_id` (only
  `SubagentStart`/`SubagentStop` do) and its subagent hooks report the *parent*
  session id, so the seen-set is shared across that boundary. The draft
  documented the `INV-3` parity gap only for pi. Fixed in §3, §5.4, §5.7; `R-6`
  added; scope Non-Goals amended.
- **F-2 (major)** — the pi adapter sketch spawned synchronously on pi's event
  loop, where tool calls may run in parallel. Fixed: async spawn plus a
  "never block the event loop" contract bullet.
- **F-3 (minor)** — codex's `additionalContextLimit` unit is asserted, not
  verified. Fixed: the design states the assumption; the unit is confirmed at
  implementation.
- **F-4 (minor)** — the shared `SurfaceInput` tolerates codex's extra fields only
  implicitly. Fixed: §5.2 states the unknown-field tolerance and the open
  `tool_input` map.

## Independent pass

A second, independent adversarial pass (Opus, on the run's own RV-377) raised
F-5..F-15 — 7 major, 4 minor, none blocking. Each was verified against the
source or codex 0.155.1 before disposition; all are answered `fix-now` and none
was contested. It changed more than prose:

- **F-5** moved `additionalContextLimit`/`timeout` to the codex **handler** and
  extended `entry_is_canonical` to handler fields.
- **F-6** resolves every wire's paths against its reported cwd (`probe_for`
  takes `cwd` and `root`; the old "root-relative" claim was wrong).
- **F-7** widened `ScopeProbe`'s path arm to a set so a multi-file patch is one
  query, deleting the per-probe loop, the cross-probe dedup and a second ranking
  rule. This **reopened the "engine untouched" scope commitment** (user
  confirmed); `src/retrieve.rs` is now a design-target selector.
- **F-10** named the reader honestly as codex's `apply_patch` grammar, added
  `*** Move to:`, and dropped the producerless `class: patch` from the neutral
  wire.
- **F-11** added a phase-1 fixture-capture gate and a tolerant string-or-argv
  `command` reader.

Sections revised: 3, 5, 6, 7, 8, 9. The run's `pass_stale` lamp is true by
construction after any such revision (mem_01a0d17f827772b096e836f95a2887c4) and
is not a gate.

**What a further pass would probe**: whether the widened probe's single cap
across many paths is the right shape (a patch touching ten files competes for
three slots); whether the tolerant `command` reader's space-join is the right
normalisation for an argv vector; whether the codex `additionalContextLimit`
unit and value hold under a real block; and whether the Claude two-form
ownership predicate has a third ancestor form the refresh should also own. None
is load-bearing for the design's shape.

## Fresh adversarial pass

A third pass (an external reviewer on the run's own RV-377) raised F-16..F-21 —
1 major, 4 minor, 1 nit, none blocking. Each was verified against the source
(the generated-handler transport, `entry_is_canonical`) or this repo's own
toolchain before disposition; all six are answered `fix-now` and none was
contested.

- **F-16 (major)** — Node's async `execFile`/`spawn` accept no `input` option, so
  the pi sketch would have blocked on an unwritten stdin and emitted nothing,
  with fail-open hiding it. Fixed: the sketch writes `child.stdin.end(json)`; a
  new §5.7 contract bullet; §9's grep check replaced with a behavioural node run.
- **F-17 (minor)** — R-3 still claimed the read → edit nudge precedes the edit
  "on both"; false for codex, which has no read surface. Fixed: R-3 split.
- **F-18 (minor)** — `probe_for` must receive the canonicalised anchor, not the
  raw reported cwd, and an absent `cwd` must be specified. Fixed in §5.1; §9
  gains the symlinked-anchor and absent-cwd cases.
- **F-19 (minor)** — under harness-side timeouts "delivered" no longer means the
  model saw it. Fixed: §5.4 states the bounded `INV-6` weakening.
- **F-20 (minor)** — the three design-only §7 rows lacked DEC records; the
  phase-1 capture gate is not confined-worker work; the Claude fixture was
  labelled captured but is not. Fixed: `DEC-287`/`DEC-288`/`DEC-289`;
  §9/§6 mark the gate `VH`; §9 names the `SL-205` fixture.
- **F-21 (nit)** — stale prose left by the F-5..F-15 round. Fixed in §1, §3,
  §5.4, §5.6 and the scope's `--format` bullet.

The raiser verified F-16..F-21 and raised two more on the re-read:

- **F-22 (major)** — the F-16 stdin write had no `'error'` listener, so an
  early-exiting child turns the write into an unhandled EPIPE that can kill the
  pi process. Reproduced (`execFile("true", …).stdin.end(<5 MB>)` throws; the
  same with a no-op listener survives). Fixed: the sketch and bullet attach the
  listener; §9 gains a stub-binary behavioural case.
- **F-23 (minor)** — lexical normalisation never resolves symlinks, so an
  absolute value through a symlinked prefix still failed the root strip, while
  §5.1 claimed the two forms were always compared equal. Fixed: the anchor
  carries both forms and an absolute value under the raw prefix is rebased onto
  the canonical anchor, with the residual (an unrelated symlink fails open)
  stated; §9's case covers an absolute value.

Sections revised: 1, 3, 5, 6, 7, 8, 9 in the first round (rev 37), then 5 and 9
again for F-22/F-23 (rev 39). `pass_stale` stays true by construction after any
revision; the lock edge's `BlockersUndisposed` cause reads only blocker-severity
*open/contested* findings, and none is one.

## Execution

### PHASE-01 — captured codex wire fixtures

Live capture (codex 0.155.1, `gpt-6-luna`, a throwaway `PreToolUse` tee hook,
`codex exec --dangerously-bypass-hook-trust`). Fixtures in
`tests/fixtures/codex/{shell,shell_write,apply_patch_tool}.json` + `README.md`.

- `tool_input.command` is a **string** on both wires, and it is the **model's own
  command** (`echo hi`) — the `bash -lc '…'` wrapper appears only at exec time,
  after the hook, so a token-prefix command match is not defeated.
- `apply_patch` is a **distinct tool** (`tool_name: "apply_patch"`, patch body in
  `tool_input.command`); a shell file write reports `Bash` and reaches the
  command surface. No `apply_patch` executable on PATH, so the via-shell patch
  form is **not reachable** — `apply_patch_via_shell.json` is deliberately absent
  and the README carries the evidence.
- `apply_patch` header paths can be **absolute** (`*** Add File: /abs/path`);
  `probe_for` already anticipates both forms.
- No `agent_id` in the payload; `permission_mode: "bypassPermissions"`.
- `A-1` retired. `EX-1` named three fixtures; the third is unobtainable and is
  recorded as an absence rather than fabricated.
- Model spend: 3 runs, ~3.7k / 7.8k / 7.2k tokens.

### PHASE-02 — neutral surface command and engine arity

`memory surface` now speaks three wires and two forms behind a doctrine-owned
`SurfaceRequest` (commit 48c082392):

- `--input claude|codex|neutral` (default claude) selects `claude_request` /
  `codex_request` / `neutral_request`; `--format claude|plain` (default claude)
  selects the envelope or the bare block.
- `probe_for(request, anchor, root)` resolves lexically — a relative value joins
  the canonical anchor, an absolute value under the raw reported cwd is rebased
  onto it, the result is normalised and stripped against root; out-of-root fails
  open.
- `paths_from_patch` parses codex's `apply_patch` grammar (update/add/delete/
  move-to); `command_text` joins an argv vector; codex's `apply_patch` is read as
  a Patch request.
- `ScopeProbe::Path` → `Paths(Vec<PathBuf>)`; a multi-file patch is ONE ranked
  query (`QueryContext.paths` was already a `Vec` — the engine needed only the
  variant rename).
- The tuning log gains `wire`; a path-set probe's key joins with ` | `.

`doctrine check gate` green; 4571 bin tests + the e2e suites pass.

### PHASE-03 — codex PreToolUse wiring and canonical hook forms

`codex_hook_specs` (commit 1560f1f08): `boot_emit` + a `memory_surface_codex`
spec over the codex matchers `Bash` / `apply_patch`, so the Codex arm loops its
registry and `RefreshReport.hooks` carries both outcomes.

- `HookSpec` gains handler-level `additional_context_limit` / `timeout`;
  `memory_surface_codex` sets `SURFACE_CONTEXT_LIMIT_CODEX` (1200) and
  `SURFACE_TIMEOUT_SEC_CODEX` (5). Both render INSIDE `hooks: […]` beside
  `command` (golden-pinned), never on the matcher group. Claude specs leave them
  `None` and their rendered entries are byte-unchanged.
- `entry_is_canonical` compares the handler fields a spec SETS against the emitted
  value (missing/stale → healed). A field the spec does not configure is left
  alone, so an operator's hand-set `timeout` survives — RV-380 `F-1` corrected the
  first cut, which compared `None` as key-absent and silently stripped it.
- Claude's canonical args are `memory surface --input claude`; the Claude
  predicate owns canonical-claude AND legacy-bare (self-heal in place, never a
  duplicate); the codex predicate owns canonical-codex only.
  `plugins/doctrine/hooks/hooks.json` moved with it, and
  `tests/e2e_claude_install.rs`'s command filter moved to the explicit form.
- The manual `/hooks` notice prints once per codex arm (not per spec) and names
  all three codex hooks — the `SessionStart` hook and both `PreToolUse` groups.
- Gate green after the change. `VA-1` attested live: `boot install --agent codex`
  printed the three-hook notice and wrote both handler-configured groups.
- Boundary: another agent's SL-261 commits interleaved the shared tree between
  the `in_progress` stamp and this commit; the phase span was named explicitly
  (`record-delta --start <own>^ --end <own>`) so the foreign commit is not
  attributed here.

### PHASE-04 — generated pi surface extension

`templates/surface.ts` + the installer triad (commit aa6e4e5ac):

- The template maps pi's `read`/`edit`/`write` (path) and `bash` (command) to
  doctrine's neutral envelope and writes it to `memory surface --input neutral
  --format plain` on an async, bounded, fail-open child; the returned block is
  appended to `event.content`.
- The envelope is written with `child.stdin.end(json)` behind a no-op
  `'error'` listener (async `execFile` has no `input` option; an early-exiting
  child would otherwise raise an unhandled EPIPE).
- The per-file install triplication collapsed into one `PiExtension` descriptor
  + `plan_extension`/`install_extension`; `plan_pi_extension` /
  `plan_mcp_extension` / `plan_surface_extension` remain as `#[cfg(test)]`
  seams. `generate_from_template` single-sources the header + `BIN_PATH` bake.
- `generate_pi_extension` imports `./surface.ts` beside `./mcp.ts`; the
  `RefreshReport.surface_extension` leg is installed by the Codex arm and
  `NotApplicable` on Claude; the three extension report legs share one
  `write_ext_outcome` helper.
- Gate green. `VA-1` attested live: `boot install --agent codex` generated all
  three extensions and printed `generated extension
  .pi/extensions/doctrine/surface.ts`. `EX-6` holds — `flake.nix` already
  copies `templates/`.
- Behavioural VTs run the generated `.ts` under `node` (`package.json`
  `{"type":"module"}` for ESM); delivered-to-stdin and EPIPE-survival both pass.

## Governance draft review

`POL-003` is `draft` pending acceptance. `REV-058` distinguishes an automatic
tool-action pointer from an explicit memory request, and `REV-059` updates
`SPEC-011`'s stale codex import-only claim. `REV-060` adds the missing
`SPEC-007` memory-side contract. The owner confirmed that `REQ-018` applies
to pointer titles, so `REV-058` and `REV-060` explicitly require quoted,
attributed pointer data with identity, trust standing and context.

`ISS-479` captures the already known pi-selection follow-up. `ISS-480` tracks
the shipped formatter discrepancy; the holdback proves admission, not
presentation. These implementation issues do not weaken the proposed rules.
The four governance drafts remain unapproved. This review was landed onto
`edge` from `review/SL-263-governance`; `doctrine spec validate` and
`git diff --check` were clean. No code gate was run for the documentation edit.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-24 · PHASE-05 governance review · 9ec0536f0

### Produced

- DEC-280..DEC-289 — the seven settled inquiry decisions plus the three design
  resolutions the fresh pass recorded (engine probe arity, path resolution, pi
  transport).
- `.doctrine/slice/263/design.md` — 9 sections, materialised at run rev 39.
- `.doctrine/slice/263/plan.toml` + `plan.md` — 5 phases, VT mandates intact.
- `tests/fixtures/codex/` — the PHASE-01 captured wire fixtures + README.
- PHASE-02 — the neutral surface command and the `ScopeProbe` arity change
  (`src/memory.rs`, `src/retrieve.rs`, `src/commands/guard.rs`).
- PHASE-03 — codex `PreToolUse` wiring + canonical hook forms (`src/boot.rs`,
  `plugins/doctrine/hooks/hooks.json`, `tests/e2e_claude_install.rs`; commit
  1560f1f08).
- PHASE-04 — the generated pi surface extension (`templates/surface.ts`,
  `src/boot.rs`; commit aa6e4e5ac).
- RV-380 review fixes — `F-1` (canonicality compares only fields a spec sets),
  `F-2` (a real neutral-wire round-trip integration test), `F-3` (one
  `BIN_PATH_MARKER`), `F-4` (`discover_surface_anchor` returns a `SurfaceAnchor`).
- POL-003; REV-058 / REV-059 / REV-060 — reviewed governance drafts, unapproved.
- ISS-479 / ISS-480 — known pi-selection and pointer-rendering follow-ups.
- scope reconciled; design-target selectors: `src/memory.rs`, `src/boot.rs`,
  `src/retrieve.rs`, `templates/surface.ts`,
  `plugins/doctrine/hooks/hooks.json`.
- RV-377 — 23 findings; F-1..F-21 verified, F-22..F-23 answered fix-now.
  Disposed `conducted RV-377`; nine section attestations and `design-accepted`
  recorded; run locked at rev 41 on the user's assent.
- commits ddccae191, e270e886b, 1950d37d6, 0a0a35b38, ee6a0649f, 97c0a0a52,
  b9bce77ee, b37a7ce4d, a898b3cd1, 3dd5e8de4, 817b4a179, c7efe9f37,
  3c7b140f6, 14d7b3ebb, 48c082392, 553f6dec8, 1560f1f08, aa6e4e5ac.

### Learned

- mem_01a0d17f6b5a795081e684ec41a96d89 — codex hook contract; **updated** with
  the live wire facts (string `command`, the model's own command, `apply_patch`
  as a distinct tool, the unreachable via-shell form, absolute patch headers,
  handler-level limit/timeout).
- mem_01a0d17f6b2577818ec693b77ba8e3d9 — pi `tool_result` context (no agent
  identity; `ctx.sessionManager`/`ctx.cwd`; `PI_SUBAGENT_CHILD` is
  package-owned).
- mem_01a0d17f82ab73128b4791ef705eb8a9 — `ScopeProbe` → `QueryContext.paths`;
  multi-path is native.
- mem_01a0d17f827772b096e836f95a2887c4 — design-review `pass_stale` is a lamp,
  not a gate.
- mem_01a0d187a9d47883b65f69835c8e80b7 — Node async `execFile`/`spawn` ignore an
  `input` option (only the `*Sync` variants honour it); the envelope must be
  written with `child.stdin.end(...)`.
- mem_01a0d2174ba779a3a121431fe0ed5674 — a shared primary tree lets foreign
  commits land inside a phase's stamped span; name your own span with
  `record-delta --start/--end` rather than spanning or forcing.
- mem_01a0d22ca59c7d71997a028f3ccb7a10 — driving a generated `.ts` pi extension
  behaviourally under node (ESM `package.json`, type-only import, `/bin/sh`
  stub child; an early-exit stub for the EPIPE path).
- mem_01a0d23b988370b38f13c416e8796973 — an idempotency comparator must compare
  exactly what the writer emits, never assert absence of unowned fields.
- mem_01a0d23b98cc7740ba50720d79726b1a — pi's `ExtensionRunner` catches handler
  rejections; an unhandled stream `'error'` event is what can kill the process.

### Open

- PHASE-05 in progress — POL-003 authored as `draft`; REV-058 (PRD-004),
  REV-059 (SPEC-011), and REV-060 (SPEC-007) remain `proposed` and unapproved;
  `SL-263 governed_by POL-003` is recorded. Approval and application occur at
  reconcile (VH-1). `ISS-480` holds implementation conformance separately.
- PHASE-01..PHASE-04 done (RV-380 `fix-now` findings reconciled).
- OQ-4 follow-up: pi as a first-class boot `Harness` variant.
