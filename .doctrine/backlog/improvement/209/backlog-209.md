## Status

**Partial — instruction surface landed, tooling guardrail remains.**

### Done (2026-07-04)

1. `install/templates/plan.toml` — template comment on the `verification` line
   now shows the full structured shape (`test_file`, `keywords`, `patterns`,
   `waived`) with a warning that bare VTs render `UNCHECKABLE`.
2. `plugins/doctrine/skills/plan/SKILL.md` (master) + installed copy — step 4's
   prose paragraph replaced with a **Structured VT mandate (NON-NEGOTIABLE)**
   sub-step containing a concrete TOML example, field-by-field definitions,
   the `waived` escape valve, and the sharp edge that `test_file`-without-keywords
   passes vacuously. Skill now says: every `VT-n` row MUST carry `test_file` +
   `keywords`.

### Remaining: `doctrine check plan <id>`

A deterministic authoring-time check that reads `plan.toml` and asserts: every
non-waived `VT-n` row carries `test_file.is_some() && !keywords.is_empty()`.
Invoked by the skill after authoring `plan.toml`, before commit.

## Design notes for `doctrine check plan <id>`

### Placement

Not wired into `check commit` / `check gate` — those run on every commit and
would flag every legacy bare-VT plan in the corpus on unrelated commits. A
targeted, explicit verb (`check plan <id>`) invoked at exactly one moment:
after authoring `plan.toml`, before commit.

### Assertions (pure, reads one file)

| VT row shape | Verdict |
|---|---|
| `{ id = "VT-1", expects = "…" }` — no `test_file` | **Bare — flag** |
| `{ id = "VT-1", test_file = "src/foo.rs", keywords = [] }` | **Bare — flag** (vacuous pass at runtime) |
| `{ id = "VT-1", test_file = "src/foo.rs", keywords = ["fn"] }` | Pass |
| `{ id = "VT-1", waived = true, waived_reason = "…" }` | Pass (waiver short-circuits) |
| `{ id = "VT-1", waived = true }` no `waived_reason` | Warn (opaque) |
| `{ id = "VA-1" }` / `{ id = "VH-1" }` | Skip (VA/VH never gated) |

Does NOT assert path existence or keyword presence — those are `verify-vt`'s
job at handover (files may not exist yet at plan-authoring time).

### Implementation path

- Parse `plan.toml` through existing `Plan::parse` (reuse, no new parser).
- Iterate `verification` rows, filter `VT-` prefix + `!waived`.
- Emit a table: per-phase, per-VT finding with a one-liner.
- Exit non-zero if any bare VT exists.
- Wire as `SliceCommand::CheckPlan { id }` or sibling in `src/slice.rs`.
- Skill integration: add one line to plan SKILL.md step 4 —
  `Run `doctrine check plan <id>`. If it reports bare VTs, add `test_file` +
  `keywords` to each. Re-run until clean, then commit.`

### Estimate

~1.0–2.0 espresso_shots — reuse `Plan::parse`, pure iteration, no new parser,
no filesystem beyond the single file read. Main cost is CLI wiring + test.

## Context

Surfaced by the SL-171 audit (RV-190 F-4). `doctrine slice verify-vt` is meant to
give a mechanical S3 coverage signal at conclude/handover time, but it can only
check a VT that carries a **structured mandate** (a `test_file` / `keywords` it
can match against the source tree). The `/plan` skill authors VTs as prose-only
`expects = "..."` strings, so verify-vt reports every VT **UNCHECKABLE** — the
gate is inert project-wide.

On SL-171 this meant all 10 VTs (5×PHASE-01, 5×PHASE-02) came back UNCHECKABLE: the
gate could neither confirm PHASE-01's genuinely-strong coverage nor flag PHASE-02's
real test gap (the missing-pagination-tests finding, RV-190 F-3, which the audit
caught by hand and remediated). A coverage gate that never fires is worse than none
— it reads as a green check while signalling nothing.

## Proposal

Two-phase fix:
1. ~~Update the plan skill SKILL.md + `install/templates/plan.toml` to show
   structured VT mandates with concrete TOML examples~~ **DONE** (2026-07-04).
2. Add `doctrine check plan <id>` — a deterministic authoring-time guardrail
   that flags bare VTs before commit.

## Scope notes

- Project-wide tooling/process gap, not specific to SL-171.
- SL-171's own plan was **not** backfilled retroactively (audit decision: marginal
  post-hoc value; the real coverage was confirmed by the existing tests + the
  audit-added pagination tests).
- Source: RV-190 (SL-171 audit) finding F-4; see also F-3 (the gap F-4's dead
  signal failed to surface).
