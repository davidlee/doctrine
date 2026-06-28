# Review RV-188 — reconciliation of SL-170

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Subject.** SL-170 "Dispatch handover trust-gate" — S1 regression baseline-diff
(PHASE-02), S3 VT existence/shape gate (PHASE-01 lift → PHASE-03), S6 VT-status
summary at conclude/handover (PHASE-04). All four phases `completed`, solo
provenance, landed on `edge` (last code `1fef2e2a`). Reviewed the **authored
edge tree** (no dispatch — `boundaries.toml` all `provenance = "solo"`), so the
parent-tree / candidate-branch caveat does not apply: there is no separate
candidate surface, the authored commits *are* the evidence.

**Lines of attack.**
1. *Conformance algebra* — `slice conformance 170`: 14 undeclared, 2 undelivered.
   Triage each cell to a cause (scope creep vs. authoring byproduct vs. shared-
   branch interleave vs. genuine undelivered scope).
2. *Decision currency* — do D1/D2 (notes.md: one-line S1 = orchestrator prose;
   conclude = skill cadence, no `dispatch.rs` verb) hold against what shipped, and
   are the design-target selectors still truthful after those decisions?
3. *Mandate fulfilment* — every PHASE EX/VT against the landed code; dogfood
   `verify-vt 170`; the SL-169 replay acceptance proof (PHASE-02 VT-7, PHASE-03
   VT-4) actually present.
4. *Forward-discipline completeness* — design's R2 mitigation ("the `/plan`
   authoring discipline closes the gap forward") — was the `plan/SKILL.md`
   guidance for `test_file`/`keywords` delivered?
5. *Invariant integrity* — INV-1..8 (esp. INV-5 no-silent-∅, INV-7 changed-halt,
   INV-8 fingerprint) carried by the PASS VTs; POL-002 walk-back (F-3) coherent
   across design/plan/notes/code/memory.

**Out of scope (settled, do not reopen).** D1/D2 settled in-design (notes.md).
EX-3 negative half (mechanical working-fs-waiver rejection) is *declared
deferred* (design §5.4 / Follow-Ups; prepare_review committed-graph reader) — its
absence is not a gap. PHASE-04 VT-3 `UNCHECKABLE` in `verify-vt` is correct (the
INV-6 waiver case is behavioural, not file-greppable). Semantic test-quality
judgement is a declared non-goal.

**Evidence run (2026-06-28).** `just check` exit 0 (full root suite green).
`verify-vt 170` clean — every VT-mode row Pass or correctly Uncheckable, zero
Fail. POL-002 memory `mem.pattern.gate.host-source-no-language-syntax`
(`mem_019f0d16…`) persisted + committed (5e126df8, anchored at 25ef560e).

## Synthesis

**Closure story.** SL-170 set out to move "verify, don't trust the worker
self-report" from convention into a mechanical gate, after SL-169 shipped its own
regressions as "env". It delivers three coherent mechanisms — S1 regression
baseline-diff (`regression.rs` + `check regression capture|diff`), S3 VT
existence/shape gate (`plan.rs` model lift + `vtgate.rs` + `slice verify-vt`), and
S6 the human-readable VT summary at conclude/handover. All four phases are
`completed` (solo), `just check` is green, and the slice dogfoods its own gate:
`verify-vt 170` runs clean over SL-170's own plan, with every Pass genuine and
every Uncheckable correct (no structured mandate — VT-2/6/8 cache-IO and
behavioural rows, PHASE-04 VT-3 the behavioural INV-6 waiver case). The SL-169
replay acceptance proof is present and passing (PHASE-02 VT-7 `new`/`persistent`
partition; PHASE-03 VT-4 missing-keyword Fail over the conformance-matrix fixture).

**Conformance algebra — fully accounted, zero genuine source drift.** The 14
undeclared cells decompose cleanly (F-1): SL-170's own authored slice files, the
layering.toml vtgate=leaf registration, five src incidentals all pre-declared
scope-relevant with rationale notes, and IMP-208 (a legitimate derived follow-up,
co-committed into the PHASE-02 feat — an accepted one-off hygiene blemish). The
slice-172 cells (F-2) are foreign concurrent work on the shared `edge` branch swept
into the oid-range boundary union (PHASE-02 `code_start_oid` is itself an SL-172
commit) — a known shared-branch artifact, not scope creep. The two undelivered
cells split: src/dispatch.rs (F-3) is undelivered *by decision* (D2 — conclude is
skill cadence, not a CLI verb; design §5.4 rejected the prepare_review fold), the
selector merely stale; plan/SKILL.md (F-4) is a *genuine* declared-scope gap — the
/plan authoring discipline for `test_file`/`keywords` was never written and no EX
criterion guarded it.

**Decision currency.** D1 (one-line S1 status = carried orchestrator prose, no
second renderer) and D2 hold against what shipped and are not reopened. The F-3
POL-002 walk-back (raw substring, no host-language stripping; `patterns` as the
opt-in shape escalation) is coherent across design §5.2, plan PHASE-03 EX-2/VT-2,
notes.md, the `vtgate.rs` code, and the harvested memory.

**Standing risks / consciously accepted.** (a) The S3 gate's un-skippability rests
on cadence trust — the `prepare_review` committed-graph hardening (EX-3 negative
half) is a built seam with wiring deferred (design §5.4 / Follow-Ups); accepted, not
a gap. (b) P2 is vacuous on plans that don't populate the structured fields — S6
surfaces UNCHECKABLE so the absence is visible, but the *forward* mitigation (the
/plan discipline, F-4) is the unfilled half and is routed to reconcile. (c) Raw
substring tolerates a keyword present only in a comment (adversarial bait, out of
the omission threat model) — documented accepted weakness. (d) Flaky / renamed-
failing tests can mis-classify in S1 — documented edges, the report aids diagnosis.

## Reconciliation Brief

All findings are observation-confirmed (`verified`). Two carry remediation, both
per-slice direct edits for `/reconcile`; none touch spec or governance, so there is
no REV surface. F-1/F-2/F-5 need no write.

### Per-slice (direct edit)
- **F-3 — src/dispatch.rs design-target selector is stale-by-decision.** Annotate
  the dispatch.rs selector (`doctrine slice selector note 170 src/dispatch.rs …`)
  with the D2 rationale (conclude = `/dispatch` skill cadence, no dispatch.rs verb;
  design §5.4 rejected the prepare_review fold) so the conformance undelivered cell
  reads as decided, not dropped. No code change.
- **F-4 — plan/SKILL.md /plan authoring discipline undelivered.** Add a short
  authoring-discipline note to `plugins/doctrine/skills/plan/SKILL.md`: when
  authoring VT rows, populate `test_file` + `keywords` so `slice verify-vt` can gate
  them; mention `patterns` as the optional language-agnostic shape escalation. This
  completes the declared Affected-surface item and closes design §8 R2's forward-
  adoption mitigation (without it the S3 gate stays vacuous on future plans).
  (Source of truth is `plugins/doctrine/skills/`, NOT the gitignored
  `.doctrine/skills/` projection — see mem.pattern.distribution.skills-source-vs-installed.)

### Governance/spec (REV)
- None.
