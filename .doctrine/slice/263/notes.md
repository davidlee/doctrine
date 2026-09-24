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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-24 · design/reviewing · ee6a0649f

### Produced

- DEC-280..DEC-289 — the seven settled inquiry decisions plus the three design
  resolutions the fresh pass recorded (engine probe arity, path resolution, pi
  transport).
- `.doctrine/slice/263/design.md` — 9 sections, materialised at run rev 39.
- scope reconciled; design-target selectors: `src/memory.rs`, `src/boot.rs`,
  `src/retrieve.rs`, `templates/surface.ts`,
  `plugins/doctrine/hooks/hooks.json`.
- RV-377 — 23 findings; F-1..F-21 verified, F-22..F-23 answered, awaiting raiser
  verification.
- commits ddccae191, e270e886b, 1950d37d6, 0a0a35b38, ee6a0649f.

### Learned

- mem_01a0d17f6b5a795081e684ec41a96d89 — codex hook contract (handler-level
  limit/timeout, canonical tool names, no `agent_id`, parent session id, no read
  tool).
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

### Open

- RV-377 F-22..F-23 raiser verification → pass disposition (conducted) → 9
  section attestations → `design-accepted` → lock. Design is not binding until
  locked.
- F-11/F-20's fixture-capture gate (design §6's three codex unknowns) must become
  a `/plan` phase-1 exit criterion, marked `VH` (orchestrator/human).
- Governance leg at reconcile: POL-003 (from IDE-034), the PRD-004 §2/§8 REV,
  the SPEC-011 REV; re-ground SL-263 `governed_by` on POL-003.
- OQ-4 follow-up: pi as a first-class boot `Harness` variant.
