# IMP-306: Consolidate capture skills into harvest + handoff

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The knot

`/notes`, `/backlog`, `/next`, `/handover` overlap heavily. Reframed by intent:

- **`/notes`** is really just a reminder / sub-step — a prompt for an agent to
  run the durable-capture pass itself. It isn't a distinct capability; it's the
  capture reflex under a name.
- **`/handover`** = *first* ensure any durable notes are captured, *then* write a
  handover for what does **not** belong as durable state (the transient "start
  here" scaffolding).
- **`/next`** = the same shape as handover, just a lighter-weight continuation
  prompt.
- **`/backlog`** is one of the durable sinks the capture pass routes to (work
  intake), alongside memory / knowledge / notes.

So there are really two capabilities hiding behind three skills — `/backlog`
is a **sink** the capture pass routes to and an independent triage surface; it
stays as-is and is out of scope for the collapse.

## Desired shape

Collapse to **two** invocable skills:

1. **`/harvest`** — "do what's necessary" to sweep produced / learned / open
   into their durable sinks. This *is* the harvest reflex (`harvest.md`),
   promoted from a sub-step to the primary capability. Subsumes `/notes`
   (renamed: "notes" named a sub-step; harvest is the capability). The doc
   stays the procedure owner; the skill is the behavioural entry point.
2. **`/handover`** — does (1), then additionally emits a continuation for the
   next agent. Subsumes `/next`: **one skill with a light/full dial**, not two.
   Full = the written, gitignored `handover.md` packet; light = a printed
   continuation prompt. The dial defaults by context (phase boundary / complex
   state → full; simple fresh-context continuation → light) and is user-
   overridable. Keeps the existing `/handover` name (right trigger description;
   `worktree`/`dispatch` already cite it; `/handoff` would be churn for nothing).

The distinction between the two is *only* whether a continuation is emitted;
the durable-capture half is identical and shared, not duplicated.

## Why this wasn't addressed by SL-215

SL-215 authored the harvest reflex (`harvest.md`) and wired the **slice/phase
spine** to cite it (code-review, audit, close, execute, notes, handover's *phase*
branch). It did **not** rationalise the skill surface: `/notes`, `/next`, and
handover's "another artifact" (non-slice, e.g. RFC) branch still carry no harvest
cue — an RFC handover runs no capture pass at all. 215 fixed *where findings
land*; it left *which skills you invoke* untouched. That overlap is this item.

## Settled design (design loop, 2026-07-24)

- **D1 — one continuation skill.** `/handover` absorbs `/next` behind a
  light/full dial (see Desired shape). Two skills would duplicate the
  freshness/cite discipline this item exists to kill.
- **D2 — names: `/harvest` + `/handover`.** Rename `/notes` → `/harvest`; keep
  the `/handover` name (not `/handoff`).
- **D3 — clean cut, no stub skills.** Delete the `notes`/`next` skill dirs and
  migrate every reference in the same change. Stubs would be a permanent
  per-session token tax (each installed skill's description rides the skills
  listing) to soften a one-time migration. Stale invocations fail loudly; the
  regenerated routing table points to the new names.
- `harvest.md` stays the single owner of the procedure and §2 sink table; both
  skills cite, never restate. Its §4 "stale → `/notes`" becomes "stale →
  `/harvest`".
- The merged `/handover` runs the harvest-first step on **both** branches —
  phase and non-slice artifact (RFC etc.), using `harvest.md` §5's no-slice
  fallback. This closes the confirmed RFC-handover gap.
- **Behaviour preservation:** the SL-170 S6 dispatch-conclude requirements
  (VT summary block, S1 regression status line) carry into the full-packet
  branch unchanged.

## Migration surface (enumerated at preflight; re-sweep with `rg` at execution)

- `install/routing-process.md:44` (`/notes`, `/next` routing entries) → boot
  snapshot regeneration (`doctrine boot`).
- Cross-cites: `execute/SKILL.md` (→ `/notes`), `worktree/SKILL.md` and
  `dispatch/SKILL.md` (→ `/handover`, name unchanged — verify only),
  `install/harvest.md` §4, `next`/`handover` mutual cites.
- Refresh mechanics: edit masters under `plugins/doctrine/skills/` → `touch
  src/install.rs` → `cargo build` (re-embed) → `doctrine install` →
  `doctrine boot`. Verify skills resolve in a fresh client (SL-215 VH reflex).
- Relate to SL-215 (`originates_from`) if sliced.
