# Review RV-433 — reconciliation of SL-287

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Mode: conformance (self-audit). Surface reviewed: primary tree on `edge` at
e4634e60f (not dispatched; PHASE-01 ae3013d83, PHASE-02 f58fb2d41).

Lines of attack:

- **Path conformance** — `doctrine slice conformance 287` against the
  `design-target` selectors (design sec-7's code-impact table).
- **Content fidelity** — `install/commitment-ledger.md` against PHASE-01
  EX-2..EX-7, design sec-2..sec-6 and the governing commitment-ledger quotes
  (counts, names, sets, conditions). First pass delegated (pi `review`
  profile); every reported discrepancy re-checked against the file.
- **Pointers** — five skills route to `lib:reference/commitment-ledger.md`
  per design sec-5's touchpoint table, without restating it; installed copies
  refreshed, not hand-edited.
- **Shipped-text grounding** (POL-002 / ADR-024) — no repo-private ids or
  `src/` paths; other docs only as `lib:` addresses; "commitment", never
  "obligation" in the new doc.
- **Gate** — `doctrine check gate` green; existing shipped-asset,
  lib-citation and skill-dedup tests unchanged.
- **Open items carried from execution** — the autonomy log's deferred
  "obligation" item; RV-429 F-6 left answered; design.md sec-3 edit artefact.

## Synthesis

SL-287 delivered what design sec-7 declared and nothing else: one new shipped
reference doc (`lib:reference/commitment-ledger.md`) with its manifest entry
and shipping test, five one-step skill pointers, the `/plan` step-4
correction, and glossary / using-doctrine layout rows. No engine change.

Evidence:

- **Conformance** (after F-1's bootstrap): 10 conformant, 0 undelivered,
  1 undeclared (notes.md, F-2 aligned).
- **Gate**: `doctrine check gate` exit 0 at e4634e60f;
  `commitment_ledger_is_shipped`, `every_shipped_lib_citation_resolves`,
  `no_bare_library_mention_in_shipped_text`, `dedup_skills_route_not_restate`
  pass. Re-ran the first two after the F-3 edit: green.
- **Fidelity**: delegated pass over EX-2..EX-7 found one discrepancy (F-3),
  confirmed and fixed; every other clause, count, enum and the five staleness
  conditions match design and the governing commitment-ledger quotes.
- **Grounding**: id sweep `[A-Z]{2,4}-[0-9]` over the new doc has no hits
  (positive control: 37 hits in design.md); no `src/`; no "obligation".
- **Install**: `./target/debug/doctrine library show
  lib:reference/commitment-ledger.md` resolves; all ten installed skill copies
  (`.claude/skills`, `.doctrine/skills`) carry the pointer.
- **Commitment ledger** (dogfooded at plan time): read; no variance recorded
  during execution, consistent with the autonomy log (realisation choices only:
  placeholder ids in the doc example, three wording restorations).
- Carried item closed: RV-429 F-6 verified (scope item 6 present).

Standing risks (accepted at design, unchanged): the procedure is unenforced
guidance; staleness detection is manual; the multi-phase trigger may prove too
ceremonious. All three await the follow-up trial (RFC-029 § 9), which is not a
closure condition.

Process note: the execution seat did not record source deltas as phases
landed (F-1); `/super-orchestrator` should be checked for that step.

## Reconciliation Brief

### Per-slice (direct edit)

- **F-3** — design.md sec-3, Satisfy step, *parallelism* bullet: "The ledger
  names candidates" → "The commitment ledger names candidates" (matches the
  shipped doc and C-11's naming rule). Content-neutral wording fix; commit
  through `doctrine design` if the run requires it.
- **F-4** — design.md sec-3, Extract step: rewrap the paragraph containing
  "(§ The commitment ledger). Stamp an `[[extracted]]` row per section read."
  to the 80-column body width. Whitespace only.

### Governance/spec (REV)

- None. No finding touches an ADR, policy, standard or spec.
