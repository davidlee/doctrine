# Review RV-413 — reconciliation of SL-275

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Conformance audit of SL-275 (PHASE-01..03), reviewed on `edge` at `b889d3b66`
(in-tree; not dispatched). Code of PHASE-01/02 already had a code review (RV-412,
concluded); this pass holds the slice to its design, not re-reviewing code quality.

Lines of attack:
- **Contract vs live behaviour.** Probe the in-tree binary for each design sec-7
  VT: default page 20; `--page N` without `--limit`; unaligned `--offset` hint;
  final page `end of results`; past-end page; floor (`bwrap` 565 → 10); no-match
  notice on search + retrieve; `--json` empty `rows`; punctuation-only browses.
- **Path conformance.** `slice conformance 275` — undeclared/undelivered leads.
- **VT gate.** `slice verify-vt 275` across all three phases; `doctrine check gate`.
- **Supersession duty** (design sec-2): the superseded records are named; the
  clamp comment is gone.
- **Shipped guidance** matches behaviour and carries no repo-private ids (ADR-024).
- **Design truth.** design.md still describes what shipped.


## Synthesis

SL-275 delivered what its design locked. Every design sec-7 behaviour was probed
live on the in-tree binary and matched: an unset `--limit` is a 20-row page;
`--page 2` without `--limit` yields rows 21-40 and names page 3; an unaligned
`--offset 5` hints `--offset 25`; the final page says `end of results`; a past-end
page says so without a false hint. Free text is floored (`bwrap`: 565 → 10 rows);
a no-match query prints the notice on search and retrieve, and `--json` returns
empty `rows`; punctuation-only input browses. `slice verify-vt 275` passes all 10
VT criteria across PHASE-01..03, and `doctrine check gate` is green at `afd1ef32c`
(the only later commit is the status flip). The clamp and its `RV-206`/`RV-207`
comment are gone from `src/retrieve.rs`; the remaining `usize::MAX` uses are test
witnesses. Code quality of PHASE-01/02 was covered by RV-412 (concluded).

Path conformance: 9/9 source paths conformant, 0 undelivered; the two undeclared
paths are the slice's own bookkeeping (F-2, aligned). The one real drift is prose:
design.md sec-6 still names the `.agents/skills` mirror as the guidance target
(F-1, delegated to reconcile).

Standing risks, carried from design sec-8 and consciously accepted there:
quantization could floor a sub-`1/LEX_SCALE` BM25 hit (three orders of margin on
this corpus); `retrieve --query <no-match>` and scope+free-text requests now return
nothing where they used to fall back to severity order — intended, and the notice
names the remedy. Out of scope and tracked: `CHR-171` (spec text still says
`memory find`), `ISS-503` (priority page-offset overflow).

## Reconciliation Brief

### Per-slice (direct edit)
- F-1 — design.md sec-6 "Code impact", guidance row: replace
  `.agents/skills/{retrieve-memory,record-memory,dreaming}/SKILL.md` with
  `plugins/doctrine/skills/{retrieve-memory,record-memory,dreaming}/SKILL.md`
  (shipped masters; `.agents/skills` is a published-repo mirror). No selector
  change — the registry already lists the plugins/ paths.

### Governance/spec (REV)
- None. Spec-text drift (`SPEC-007`/`REQ-378` naming `memory find`) is already
  owned by `CHR-171`, deferred by design sec-2.

## Reconciliation Outcome

### Direct edits applied
- design.md sec-6 "Code impact", guidance row: `.agents/skills/…` →
  `plugins/doctrine/skills/{retrieve-memory,record-memory,dreaming}/SKILL.md`
  (finding F-1). User assent 2026-09-29 ("agreed").

### REVs completed
- None — the brief carried no governance/spec items.

### Withdrawn / tolerated
- F-2: aligned — the slice's own bookkeeping files; no write needed.

Reconcile pass complete — handoff to /close.
