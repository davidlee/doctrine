# SL-215 — Unified harvest surface: design

Status: drafted 2026-07-24; pending adversarial pass + lock.
Origin: IMP-059 (harvest unification) + IMP-042 (code-review corpus integration),
ordered per the IMP-042 → IMP-059 `after` edge. RFC-011 L2' carry-forward half.

## 1. Current vs target behaviour

**Current.** The end-of-work harvest — surface what was *produced* (work delta),
*learned* (durable knowledge), and what *decisions* remain open — is restated in
eight-plus sites: `/notes`, `/handover`, `/next`, `/audit` step 6, `/close` step 1,
`/code-review` tail, `/inquisition` tail, `/walkthrough`, `/feedback` step 7, and
`review-ledger.md` §5. The review side is half-unified (§5 owns synthesis+harvest
mechanics; skills keep lens + tail); the notes/handover/next side is not unified
at all. No tail routes to the `/knowledge` sink (SL-214, done) — the "open
decisions / assumptions carried" leg exists nowhere. Templates drift; nothing
carries a decision recorded in notes into the handover or the next agent's
orientation; each skill re-surveys the same ground.

**Target.** One shared PULL-tier reference doc (`install/harvest.md`, ADR-005)
owns the harvest once: moments, three legs, sink table, canonical output shape,
consumer contract. `/notes` is the single routed entry point (it already holds
the boot-table row "finished a coherent unit → `/notes`"). The canonical output
is a single maintained, freshness-stamped `## Harvest` section in the governing
slice's `notes.md`. `/handover`, `/next`, and review synthesis become
projections: they check freshness and cite ids from the harvest instead of
re-surveying. Every other harvest tail shrinks to a one-line citation + its
skill-specific lens. The code-review skill additionally gains lifecycle
awareness (a `## Cadence` section) — the IMP-042 leg.

## 2. Decisions

- **D1 — Harvest shape: shared reference doc + `/notes` entry point; no new
  skill.** A dedicated `/harvest` skill would mint a routing row whose trigger
  ("end of a coherent unit") is already `/notes`' — two skills competing for one
  moment is a routing defect (ADR-009 F14 pressure). Matches ADR-005 (skills
  route, reference docs explain) and the `review-ledger.md` precedent exactly.
  Precision (adversarial F1): "one entry point" means one **owner**
  (`harvest.md`) and one **output** (`## Harvest`); `/notes` is the routed entry
  for the bare end-of-unit moment, while review skills execute the same
  procedure through their cited tails against the same manifest — they do not
  route to `/notes`. → DEC record minted at design lock.
- **D2 — Canonical output: single maintained `## Harvest` section, pointer-only,
  freshness-stamped.** Satisfies the RFC-011 L2' properties (single-copy,
  freshness-marked, progressively accreted, scoped to the next stage) in one
  shape. Append-only per-event blocks relocate the staleness problem;
  no-manifest fails "consumed, not re-derived" outright. Drift is bounded by
  pointer-only discipline (ids + one clause, never restated content) and swept
  at each harvest pass. Entries carry **no status field** (adversarial F2): a
  status is queried data, and queried data in authored prose violates the
  storage rule — consumers query status via the CLI when they need it.
  → DEC record.
- **D3 — Code-review cadence gate keys on worker model tier + tripwires,
  arm-agnostic.** The dispatch arm is a transport property, not an adherence
  property; the pi arm can run any model. Default per-phase review **on** when
  the worker model sits below a stated adherence bar (qualitative prose
  heuristic; orchestrator judgment); tripwires escalate to mandatory regardless
  of tier: deleted tests in the import diff, "Deviations: NONE" beside
  design-relevant divergence, waived/uncheckable VT, out-of-scope touches.
  (SL-222 PHASE-09 incident class.) → DEC record.
- **D4 — Single ownership per concern; cite, never restate.** `harvest.md` owns
  moments/legs/output/consumer-contract. `using-doctrine.md` keeps
  work/knowledge/decision home-arbitration; `/knowledge` keeps kind selection;
  `review-ledger.md` keeps synthesis. Four docs, no duplicated boundary text.
- **D5 — All citing sites shrink, not just the four named consumers.** The
  marginal cost is four one-line edits; stopping at four preserves the drift
  failure mode inside the slice that kills it. Behaviour-preservation VA stays
  scoped to the four named projections.

## 3. `install/harvest.md` — content design

Header comment matches `review-ledger.md`'s (shipped reference, ADR-005 PULL
tier; edit the source under `install/`; installed copy inert).

- **§1 The harvest moment.** End of any coherent unit — phase wrap, slice wrap,
  review close, walkthrough/feedback close. Judgment-gated: a clean unit
  harvests nothing; that is a valid outcome, not a skipped step.
- **§2 Three legs, one sink table.**
  - *produced* — work delta: commits/refs into notes prose; follow-up **work** →
    `backlog new`. Home arbitration: `using-doctrine.md` § Which home (cited).
  - *learned* — reusable agent guidance → `/record-memory`; citable epistemic
    observations → EVD/CPT via `/knowledge` (discriminator owned by
    `/knowledge`, cited).
  - *open* — decisions → DEC, questions → QUE, assumptions carried → ASM,
    constraints → CON, via `/knowledge`. The previously-missing leg.
- **§3 Canonical output.** The maintained `## Harvest` section in the governing
  slice's `notes.md`:

  ```markdown
  ## Harvest
  <!-- single-copy: updated in place each harvest; ids only, never restated content -->
  fresh-as-of: 2026-07-24 · PHASE-03 · a1b2c3d

  ### Produced
  - PHASE-03 done — <one line> (commits a1b2c3d..d4e5f6a)
  - minted: IMP-241 — <one clause>; ISS-102 — <one clause>

  ### Learned
  - mem.pattern.dispatch.import-tripwires
  - EVD-014 — <one clause>

  ### Open
  - DEC-011 — <one clause>
  - QUE-023 — <one clause>
  ```

  Rules: stamp = date · lifecycle position (PHASE-NN **or** stage, for
  non-phase moments — F5) · head commit at harvest time; entries are ids + one
  clause, **never a status** (F2 — query the CLI); settled/superseded entries
  drop at the next pass (git holds history). The section stub seeds from the
  notes template (F3).
- **§4 Consumer contract** (the load-bearing sentence): a consumer checks
  `fresh-as-of` against actual lifecycle position — **fresh → cite ids, never
  re-survey; stale → the harvest is owed, route to `/notes` first**, never
  silently re-derive. ADR-005 conformance rule (F4): this check rides **inline
  in each consumer skill's body** — the doc explains, the skill carries the
  behavioural rule; demoting it to a pulled pointer is the C4 error class.
- **§5 No governing slice.** No manifest — legs route to sinks; for reviews the
  synthesis carries the story; consumers fall back to entity queries.

## 4. Code impact (design-target touch-set)

All skill edits under `plugins/doctrine/skills/`; docs under `install/`.

| Path | Change |
|---|---|
| `install/harvest.md` | **new** — §3 above |
| `install/templates/notes.md` | `## Harvest` section stub with the stamp line + three leg headings (F3) |
| `install/review-ledger.md` | §5 harvest paragraph → two-line defer to `harvest.md`; synthesis ownership stays; "skill-specific tails stay in the owning skill" survives |
| `notes/SKILL.md` | entry-point rewrite: trigger-form description names the harvest role; body restructured around three legs (existing checklist = *produced* detail; sink paragraphs → citation; adds `## Harvest` maintenance + the *open* leg). Keeps storage-rule paragraph + scaffold pointer |
| `handover/SKILL.md` | TODO "record any information worth durably persisting" → "confirm `## Harvest` fresh (else `/notes` first); cite its ids". Reading list points at the Harvest section. SL-170 S6 VT-embed beat untouched |
| `next/SKILL.md` | "confirm durable state in order" → mechanical check (`fresh-as-of` vs lifecycle position); continuation prompt cites open DEC/QUE/ASM ids |
| `code-review/SKILL.md` | tail → citation; new `## Cadence` section (D3): two moments, tier-gated default, tripwires, finding-landing per position (per-phase → RV on the slice, fix while worktree hot; pre-close → the audit's reconciliation RV; ad-hoc → target ladder) |
| `audit/SKILL.md` | step 6 tail → citation + phase-sheet-sweep lens kept |
| `close/SKILL.md` | step 1 harvest bullet → citation + "or consciously rejected" gate wording kept |
| `inquisition/SKILL.md` | tail → citation |
| `walkthrough/SKILL.md` | harvest note → citation |
| `feedback/SKILL.md` | step 7 → citation |
| `dispatch/SKILL.md` | one-line cadence touchpoint (between import and conclude) |
| `execute/SKILL.md` | one-line cadence touchpoint (phase wrap) |

**Not touched:** `install/routing-process.md` (no new skill, existing rows
route correctly), engine/CLI, ADR-007 ledger mechanics, `knowledge/SKILL.md`
(SL-214 artifact — consumed only).

## 5. Verification alignment

- **VA-1** (projections consume, not re-derive) — grep-assertable: no re-survey
  prompt remains in `handover`/`next` bodies and review tails run the shared
  procedure via citation (F1 — review skills don't route to `/notes`); each
  consumer carries the inline freshness check (F4); the three-sink routing
  paragraph appears exactly once in the corpus (in `harvest.md`).
- **VA-2** (code-review routing) — Cadence section states finding-landing per
  lifecycle position; persistent→backlog / session→ledger holds from IMP-023,
  verified by reading.
- **VA-3** (knowledge legs live) — dogfood on SL-215 itself: design DECs minted
  via `/knowledge`; `## Harvest` maintained in SL-215's `notes.md`.
- **VH** (POL-002 reflex) — fresh-client resolve for all edited skills + new
  doc; grep for repo-local couplings. Delivery note: `install/` is an existing
  embed root (no `flake.nix` change); re-embed footgun applies (touch the
  embedding crate → `cargo build` → `doctrine install`).

## 6. Residual (named, not hidden)

- The adherence **bar** is a qualitative prose heuristic; no model-tier registry
  exists and minting one is out of scope. Follow-up if it needs firming.
- `## Harvest` staleness detection is honour-model — no verb enforces the
  consumer contract. Consistent with non-goals (no CLI widening); the audit
  should not treat this as an omission.
