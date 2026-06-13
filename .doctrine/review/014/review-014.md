# Review RV-014 — reconciliation of SL-058

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-058 (relation surface tooling) against `design.md`,
ADR-010/ADR-004, and the plan's EX/VT criteria. Three phases, all committed:
PHASE-01 (templates), PHASE-02 (parser hardening + entity migration), PHASE-03
(agent guidance). Self-audit (raiser == responder, `--as` role assertion).

**Lines of attack:**
1. **Template post-cut shape (PHASE-01 EX-1/3).** Each of the six templates at
   the shape the design dictates — slice: NO `[relationships]`; gov: `related`
   dropped, `supersedes`/`superseded_by`/`tags` kept typed; backlog:
   `slices`/`specs`/`drift` dropped, `needs`/`after`/`triggers` kept; all carry a
   `doctrine link` guidance comment. Black-box scaffold test + template-guard.
2. **Parser honesty (PHASE-02 EX-1/4).** `view()` header match is comment-stripped
   exact (no `[relationships.x]` false-match), keys quote-stripped; the `name==056`
   hardcode is gone; the corpus invariant asserts no slice `[relationships]`.
3. **Edge preservation (PHASE-02 EX-2).** IMP-045 **and** IMP-052 (re-scan drift)
   `slices=SL-056` linked before strip; SL-054 stray-key table reconciled
   (`adrs=[1]`→`governed_by ADR-001`, `extends=[53]`→prose). Renders post-strip;
   `validate` clean.
4. **Guidance is pointer-not-copy (PHASE-03 D4).** Every surface points at
   `RELATION_RULES`/`link`; none transcribes the vocabulary (the VA-1 mandate).
5. **Behaviour preservation (R5).** Shared machinery untouched; the prior suites
   green unchanged; `just gate` clean on the final state.

**Where bodies may be buried:** RustEmbed false-green (already hit + cleared);
the EN-2 cutover drift (design count stale on shared `main`); any guidance surface
that drifted toward restating the LinkPolicy taxonomy (VA-1 caught + fixed one).

## Synthesis

SL-058 closes the relation-surface migration ("the cut", SL-048) by stopping the
inflow of malformed `[relationships]` rows at every source and reconciling the
existing fallout. The slice conforms to `design.md`, ADR-010, and ADR-004; the
ledger carries three findings, all verified terminal, none a blocker.

**Conformance is clean across all three phases:**
- **PHASE-01 templates** — all six at the post-cut shape: slice carries no
  `[relationships]` header; adr/policy/standard drop `related`, keep `supersedes`
  + `tags` typed; backlog/backlog-risk drop `slices`/`specs`/`drift`, keep
  `needs`/`after`/`triggers`; every template carries a `doctrine link` guidance
  comment. Black-box scaffold test + kind-specific template guard green.
- **PHASE-02 parser + migration** — the `view()` helper matches the header
  comment-stripped + exact (no `[relationships.x]` false-match, F-C) and
  quote-strips keys (F-H); the `name=="056"` hardcode is gone. Edge preservation
  holds: IMP-045 and IMP-052 both render `slices: SL-056`, SL-054 renders
  `governed_by: ADR-001`; `doctrine validate` reports the corpus clean.
- **PHASE-03 guidance** — pointer-not-copy (D4) is satisfied: the EX-1 memory,
  `using-doctrine.md` § Relating entities, and the slice/design/plan skill
  pointers all route to `RELATION_RULES`/`link` and none transcribes the
  vocabulary. The VA-1 adversarial pass caught the one real drift (two surfaces
  enumerating the `LinkPolicy` variants, already diverged) and it was tightened
  out before commit.

**Behaviour preservation (R5):** `just gate` is green on the final state — the
SL-048/SL-046/relation/cordage suites and the four original storage invariants +
`e2e_link_unlink` pass unchanged.

**Dispositions (the conscious calls):**
- **F-1 (aligned)** — the design's literal fallout count ("10 backlog + SL-056")
  went stale on shared `main`, but the design's own F-G cutover-re-eval gate is
  what caught the drift (a 2nd populated key + the SL-054 stray-key class). Code
  and the design's actual mechanism agree; lesson captured in
  `mem.pattern.migration.rescan-cutover-and-stray-key-table-disposition`.
- **F-2 (tolerated)** — SL-054's `extends=[53]` slice→slice lineage has no legal
  `RELATION_RULES` label; demoted to prose. Consciously accepted: the typed table
  was the defect, not its absence; a vocabulary label is an ADR-010 amendment out
  of scope. Recurrence is the backlog trigger, not this one historical key.
- **F-3 (aligned)** — RustEmbed false-green hit during the gate (stale jail-target
  embed); cleared by touch + rebuild. Process gotcha already covered by memory +
  notes, not a slice defect.

**Standing risk into the future:** the RustEmbed recompile footgun keeps biting
template/doc work — touch the embedding crate, rebuild the bin, re-run before
trusting a red. No undispositioned findings remain; the slice is audit-ready.
