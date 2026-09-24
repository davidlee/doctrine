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

**What a further pass would probe**, were one commissioned: whether the
multi-probe merge in §5.4 orders `admits` and cross-probe dedup correctly
against the cap (a cap applied after the union is the only thing stopping a wide
patch from crowding out a better-scoring hit); whether the codex
`additionalContextLimit` value holds under a real block once its unit is known;
and whether the Claude two-form ownership predicate (canonical + legacy-bare)
has a third ancestor form the refresh should also own. None is load-bearing for
the design's shape, and none is a reason to hold the lock.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
