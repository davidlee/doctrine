# Review RV-280 — reconciliation of SL-214

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation conformance audit of SL-214 (knowledge authoring skill),
mode: conformance, self-audit (raiser+responder via `--as`). Surface reviewed:
the primary tree on `edge` (no dispatch; phases executed solo) at commits
a88e35e5 (PHASE-01), 32bf6abc (PHASE-02), a2d10d04 (PHASE-03) + follow-on
notes/backlog/memory commits.

Lines of attack:

1. **Design conformance** — every D1–D5 element present in the shipped
   artifacts; the six design-target paths delivered as declared; no silent
   scope widening (Non-Goals: no engine/CLI change, no harvest logic, no
   IDE-007 depth).
2. **Conformance algebra** — `slice conformance` reports 14 undeclared paths,
   0 undelivered. Each undeclared path dispositioned: dogfood records
   (PHASE-03 deliverables?), SL-221 files (delta contamination?),
   skills-lock.json, case-notes, memory edit, ISS-227.
3. **F14 sequencing invariant** (ADR-009) — routing row landed only after the
   skill was installable; boot regenerated and `--check` clean. Includes the
   mid-phase premise correction: reachability required a GitHub publish, not
   just a rebuilt binary.
4. **POL-002** — shipped skill files resolve in a fresh client: no repo-local
   paths, branch names, `just` invocations, governance-id citations, or
   unresolvable `[[mem.…]]` keys.
5. **VT/VA/VH evidence** — re-run `verify-vt`; re-check VA criteria
   (byte-identical installed copy, well-formed records, no dep/seq authored by
   records); surface the two outstanding VH gates (PHASE-02 VH-1 sequencing,
   PHASE-03 VH-1 fair-first-citizens) for human sign-off.
6. **Working-tree hygiene** — uncommitted changes at audit time
   (slice-214.toml status flip, case-notes, flake.nix) accounted for.

## Synthesis

The slice delivered what the design declared, where it declared it. All six
VT mandates pass; `doctrine check gate` green; the POL-002 sweep over the six
shipped files is clean (only English-word and placeholder-id hits; no
repo-local governance ids, no unresolvable `[[mem.…]]` keys); DEC-002 and
ASM-002 are well-formed, seeded with their kinds' default states, carry
`shapes → SL-215` only, and author no dep/seq edges (ADR-017 honoured). After
the registry corrections, conformance reads 12 conformant / 0 undelivered /
3 undeclared — and all three residual undeclared paths are consciously
dispositioned (F-5 aligned: install lockfile + memory hygiene).

The audit's real work was separating signal from noise in the conformance
algebra and testing whether PHASE-02's exit state *survived*:

- **Registry noise (F-1, F-2, fixed).** The solo phase-binding recorded
  HEAD-ranges that swallowed foreign commits (SL-221 design work, an ISS
  capture, case-notes), and the dogfood records — the whole point of
  PHASE-03 — had no design-target selector. Both fixed at audit via the
  registry verbs (safe `--commit` re-record; `selector add
  '.doctrine/knowledge/**'`).
- **The one durable defect surfaced is environmental, not in the slice
  (F-3).** PHASE-02's EX-4 ("boot snapshot carries the row") held at phase
  close but was silently reverted by the session-start regen: `doctrine boot`
  renders from the *generating binary's embedded assets*, and the PATH
  release binary predates the row — worse, `boot --check` validates against
  that same embed, so the rollback self-reports clean. Restored with the
  fresh-embed dev binary; recurrence captured as ISS-228 + a high-severity
  memory. **Standing risk until a release ships:** any boot regen from the
  stale PATH binary de-routes `/knowledge` again.
- **Distribution lag consciously accepted (F-4, tolerated).** The Claude
  marketplace cache (plugin 0.1.0) doesn't contain the skill — `/knowledge`
  is routed but not invocable in a Claude session until the plugin refreshes.
  The channel the slice governs (doctrine install → `.agents/skills`,
  byte-identical, lock-selected) is correct. Process fix: CHR-045 (bump
  plugin.json version when the skill set changes).

Two human gates remain open as blockers by design — PHASE-02 VH-1 (F-7) and
PHASE-03 VH-1 (F-8). The close-gate holds the slice in audit until both are
signed off; evidence for each is assembled in the finding detail.

Tradeoffs consciously accepted: the stale-census premise stays in canon until
reconcile (F-6, brief item); skills-lock.json and the memory correction remain
undeclared rather than falsely claimed as design targets (F-5).

## Reconciliation Brief

### Per-slice (direct edit)

- **F-2 mirror** — design.md §Code impact: add a row for the PHASE-03 dogfood
  records (`.doctrine/knowledge/**` — DEC-002, ASM-002). The load-bearing
  change (selector registry) is already applied at audit; this is the human
  mirror only.
- **F-6** — design.md "Current vs target behaviour" and slice-214.md Context:
  correct the stale zero-records census premise (five SL-158/ADR-017-era
  records pre-existed the slice; dogfood ids are therefore -002; "first
  records" reads "first records authored through the routed skill").

### Governance/spec (REV)

- None. No finding touches an ADR, policy, standard, or spec; ISS-228,
  ISS-229, CHR-045 are backlog work items, not governance edits.

## Reconciliation Outcome

### Direct edits applied

- design.md §Code impact: added the `.doctrine/knowledge/**` row mirroring the
  design-target selector applied at audit (RV-280 F-2). Registry was the
  load-bearing change; this is the human mirror.
- design.md §Current vs target + slice-214.md §Context: corrected the stale
  zero-records census premise — five SL-158/ADR-017-era records post-dated the
  2026-06-26 census; dogfood ids are -002; "first records" reads "first
  authored through the routed skill" (RV-280 F-6).

### REVs completed

- None required — the brief carried no governance/spec items.

### Withdrawn / tolerated

- RV-280 F-4: tolerated — Claude plugin-cache distribution lag; rationale in
  the finding disposition; process fix tracked as CHR-045.

Reconcile pass complete — handoff to /close.
