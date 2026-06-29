# Review RV-183 — design of SL-168

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Internal hostile pass (the first of two — external codex/GPT-5.5 follows
integration) on `SL-168` design.md (the unified `doctrine doctor` verb). Pass 1
(self) is already integrated into §10; this tribunal does NOT re-litigate its
verified-OK findings (json_envelope row-generic, item→slice outbound `Slices`
edge, KINDS-includes-record-kinds) unless a fresh fault is found.

Lines of attack (handover mandate — attack hardest):

1. **ProseCite exclusion-set completeness (§5.5)** — verified *empirically*
   against the live corpus prose, not reasoned. The dense, precision-critical
   surface. Probed: the candidate-token grammar (undefined), 3-part cite
   shapes, DEC dual-namespacing, illustrative/example ids in committed prose,
   the disposable-tier skip scope.
2. **Native re-point byte-exactness vs legacy goldens (R1, §5.5, §9)** — #1
   id-integrity + #4 memory. Verified the goldens that R1 names as "the proof"
   actually exist and actually pin byte-exact output.
3. **done-but-open `≥1 linked slice` guard (§5.5)** — the load-bearing
   non-vacuity guard; primitives confirmed present.

Doctrine held to: ADR-001 (layering), STD-001 (no magic strings), the
behaviour-preservation gate (AGENTS.md), no-parallel-implementation (CLAUDE.md),
and the design's own §5.5 invariants / §7 decisions / §8 risks. Evidence is
`file:line` and live `grep` over `.doctrine/**/*.md` + `src/`.

## Synthesis — verdict

**Judgement: the design is sound in bones, but pass 1 confessed three material
heresies under empirical cross-examination — all on the surfaces the handover
named, all now reconciled in the design body.** No blocker; the slice may
advance once the User accepts the corrections (and, per the handover, after the
external codex pass).

The corpus does not lie, though the design nearly did. Verified by `grep` over
the living prose, not by reasoning from the armchair:

- **The verification net was vapour (F-3).** R1 swore "existing goldens are the
  proof" of byte-exact native re-point. They are not: `validate` is asserted by
  *substring* (`tests/e2e_integrity.rs` `.contains`), and `memory validate` has
  **no output golden whatsoever** — its only trace is an MCP tool-registry roll
  call. Had this shipped to `/plan` unchallenged, a native re-point of #1 or #4
  could have silently drifted a shipping command's output with nothing to catch
  it. Penance levied: D12 — byte-exact goldens authored **first, red**, as a
  precondition; else the source stays adapter'd. The gate is now real.
- **ProseCite would have cried wolf ~20+ times a run (F-2).** The exclusion set
  omitted the dominant false-positive class — illustrative ids in committed prose,
  `POL-123` festering in the *shipped* `glossary.md` itself. D11 narrows the scan
  off the process-exhaust tier; R8 owns the residual. Advisory severity was the
  only thing standing between this and a doctor that screams at a clean corpus.
- **The 3-part exclusion rested on a misread (F-1, F-4).** The existing
  whole-token primitive treats `-` as no boundary, so it matches `DEC-005` inside
  `DEC-005-C` — the "skip 3-part" rule would never have fired. And the *reason*
  given (external decision cites) was the minority case; the corpus is ruled by
  `SL-048-style`/`IMP-006-gated` compound adjectives. A new maximal-token scanner
  is now mandated and its tests pinned.

The lesser taints (F-5 scan-scope provenance, F-6 the undefined open-item
predicate) are cauterised in the same passes.

**Standing risks carried forward:** R8 (residual ProseCite example noise in
durable bodies — accepted, advisory); the external reviewer should judge whether
D11's blunt scope-cut is the right instrument or whether a sharper
example-detection heuristic is warranted, and whether D12 is design-canon or mere
plan sequencing.

**Tolerated by conscious choice:** the dangling-3-part false-negative
(`SL-999-style` invisible) — the price of excluding 3-part wholesale on an
advisory check.

> **HERESIS URITOR; DOCTRINA MANET**

### Addendum — external pass (codex / GPT-5.5), F-7..F-11

The external inquisitor earned its keep: five fresh heresies the internal passes
missed, every one verified against source before the design was touched (per the
trust-but-verify writ — two of codex's claims re-specified load-bearing seams, so
blind acceptance was forbidden).

- **The raw-label seam was outright fiction (F-7).** Pass 1 confessed an open
  doubt ("does `outbound_for` expose `Raw` labels?"); codex closed it — `no`. The
  named type `RelationLabel::Raw` does not exist; `outbound_for` *drops* off-table
  rows; a Raw label on a numbered edge *panics* as corruption. The real carrier is
  `CatalogEdgeLabel::Raw` in the catalog graph (IMP-141's 173 edges). Re-specified.
- **My own pass-2 penance bred a contradiction (F-8).** The D12 golden-gate I added
  fought §5.2's "native" v1-path and a sketch that wired native before the goldens.
  Reconciled to adapter-first / native-after-golden, sketch reordered.
- **D11 left a door open (F-9):** it forgot `.doctrine/review/**` — and *this very
  ledger* would have re-imported `POL-123`/`SL-999` example noise through it.
- **#7 double-reported entity TOML already owned by #1 (F-10);** scoped to facets +
  `plan.toml`. **D8 cited the wrong check number (F-11);** corrected.

Eleven charges total across three passes, zero blockers, all terminal. The design
now tells the truth about its seams, its verification basis, and its scope. Fit to
lock on the User's word.

> **HERESIS URITOR; DOCTRINA MANET**
