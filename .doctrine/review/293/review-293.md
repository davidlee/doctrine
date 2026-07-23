# Review RV-293 — reconciliation of SL-215

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** SL-215 — Unified harvest surface (IMP-059 + IMP-042). Slice at
`audit`, 5/5 phases complete, dispatched via `/dispatch`.

**Reviewed surface.** This is a **dispatched** slice — the impl bundle lives on
the coordination/evidence refs, **not on edge**. Audit ran against the
impl-bundle ref `review/215 @ 2046c23e8` (verified against `dispatch/215 @
d979f2f6`), read in a detached worktree; the primary tree stayed on `edge`
untouched. Integration to `main` is `/close`'s job (`dispatch sync --integrate`).
Nothing here was gathered from edge, where the slice deliverables do not yet
exist — a `verify-vt` run on edge false-reds all 19 VTs (files absent); the
audit surface is the bundle.

**Lines of attack (invariants the slice is held to):**

1. **Conformance algebra** — declared `design-target` selectors vs recorded
   source-deltas. Clean expected (docs/skills-only slice).
2. **VT/VA per phase** (`plan.toml` authoritative) — keyword substance + the
   agent-mode design invariants.
3. **DEC-004..006 conformance** — the delivered corpus must match the three
   locked decisions: harvest shape (owner doc + `/notes` entry, no new skill),
   single freshness-stamped `## Harvest` output, arm-agnostic code-review cadence
   (adherence bar + four tripwires).
4. **Design VA-1 single-source invariant** — the three-sink routing block
   appears exactly once (harvest.md §2); every other tail cites, never restates.
5. **POL-002** — no repo-local couplings in shipped `install/**` or `plugins/**`
   artifacts (platform independence).
6. **Design §6 residuals** — confirm the two named residuals are intended scope
   boundaries, not gaps.

**Carried items to disposition** (from the conclude handoff): PHASE-05 VT-1
UNATTRIBUTABLE (dogfood notes.md, keyword-substance confirmed, outside the
code-delta registry); the two §6 residuals; the accepted POL-002 VA-2 template
filler in harvest.md §3.

## Synthesis

**Verdict: clean. Reconcile with one cosmetic per-slice edit; no code change, no
governance change.**

SL-215 unifies the end-of-work harvest onto one owner doc (`install/harvest.md`,
ADR-005 PULL tier) with `/notes` as the single routed entry point, and shrinks
every other harvest tail to a citation. The delivered corpus conforms to the
three locked decisions:

- **DEC-004 (shape)** — one owner + `/notes` entry, no new skill. Verified:
  `notes/SKILL.md` is the trigger-form entry point and cites harvest.md rather
  than re-deriving the routing; no competing routing row was minted.
- **DEC-005 (output)** — single freshness-stamped `## Harvest` section,
  pointer-only, no status field. Verified in harvest.md §3, the notes template
  stub, and the SL-215 dogfood.
- **DEC-006 (cadence)** — arm-agnostic gate keyed on model tier + four
  tripwires. Verified: the code-review `## Cadence` keys on the model "never the
  transport… a worker is a worker whether it ran in-process, as a subprocess, or
  in an isolated worktree" — POL-002 spirit honoured (PHASE-01 VA-1).

**Conformance** is clean: 0 undeclared / 0 undelivered / 14 conformant — no scope
creep, nothing declared-but-undelivered. **VT: 18 PASS, 1 UNATTRIBUTABLE** (F-1,
disposed aligned — a tool-attribution artifact, not a substance gap). The
**design VA-1 single-source invariant holds**: the three-sink routing block
(produced→backlog / learned→memory / open→knowledge) co-occurs in exactly one
file, harvest.md §2; every consumer and tail cites it. `handover` and `next`
carry the inline `fresh-as-of` freshness check with an explicit "do not
re-survey", eliminating the independent re-survey prompt (design VA-1). The gate
is unaffected — zero compiled-surface files in the slice delta, so no clippy
regression is reachable; the docs/skills-only change cannot move it off its
green baseline.

**Standing risks / tradeoffs consciously accepted:**

- The harvest freshness contract is **honour-model** (F-2) — no verb enforces
  the consumer check; it rides inline in each consumer skill (design §4). A
  future skill that forgets the check is unguarded. Accepted: enforcing it would
  need a CLI verb, a declared non-goal.
- The code-review **adherence bar** is qualitative orchestrator judgment with no
  model-tier registry (F-2). Conditional follow-up only ("if it needs firming");
  not owed now, no backlog item minted.
- One real repo id rides shipped text as template filler (F-3, tolerated, VH-1
  ratified). Self-evident within a fenced example; a marginal POL-002 hardening
  was consciously declined.

**Dispatch-topology note for `/close`:** the impl bundle — including the dogfood
`notes.md` and the coord journal — lives on `dispatch/215` / `review/215`, not on
edge. `/close` lands it via `dispatch sync --integrate --trunk refs/heads/main`.
An accidental empty `notes.md` stub created on edge during audit (by a `slice
notes` probe) was removed; the tree is clean. Do not hand-author a competing
edge `notes.md` — let integration carry the dogfood version.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md header (F-4)**: line 3 reads `Status: drafted 2026-07-24; pending
  adversarial pass + lock.` The design is locked (slice at audit; adversarial
  F1..F5 folded into the body; no ledgered design RV, permitted). Update the
  status line to reflect the locked state (e.g. `locked 2026-07-24 (adversarial
  F1..F5 folded)`). Cosmetic; the only write this audit delegates.

### Governance/spec (REV)
- **None.** No ADR, policy, standard, spec, or requirement is contradicted by
  the implementation. DEC-004..006 already record the decisions; POL-002 and
  ADR-005 are honoured. Nothing routes to a REV.

### Not on any write surface (recorded, no action)
- **F-1** (aligned): PHASE-05 VT-1 UNATTRIBUTABLE — tool attribution, no edit.
- **F-2** (aligned): design §6 residuals are intended boundaries — no edit.
- **F-3** (tolerated, VH-1): harvest.md template filler — no edit.
