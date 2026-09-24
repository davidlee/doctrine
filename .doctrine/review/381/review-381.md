# Review RV-381 — reconciliation of SL-263

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Conformance audit of SL-263 (pi/codex port of `memory surface`), driven from the
primary tree on `edge` at 007a0ec87. Not a dispatched slice — no candidate
branch; the surface reviewed is `edge` itself.

Lines of attack:

1. **Path conformance** — `slice conformance 263` against the five
   `design-target` selectors; every undeclared path traced to its commit and
   slice; the recorded phase spans checked for foreign commits (the handover
   flags PHASE-05's 7-commit span).
2. **Criteria** — every phase's EX/VT/VA against the tree: `slice verify-vt`,
   `doctrine check gate`, and the recorded attestations in `notes.md`.
3. **Neutral core (POL-003 / DEC-280)** — harness vocabulary confined to the
   codecs; the admit/dedup/cap/format pipeline composed once and unchanged
   (SL-205 non-goal: no new query, admission rule or knob).
4. **Governance leg** — POL-003 draft, REV-058/059/060 proposed; the spec drift
   they close, and anything they would open against shipped code (ISS-480).
5. **Scope fidelity** — delivered artefacts against `slice-263.md` scope.

Prior ledgers: RV-377 (design, concluded) and RV-380 (code review of
PHASE-01..04, done; F-1..F-4 fixed at 9d57beb96) — not re-litigated here.

## Synthesis

SL-263 delivers what its locked design declared. All five `design-target`
selectors are conformant and none is undelivered. `doctrine check gate` is green
at 007a0ec87 (fresh build, clippy, eslint, full test suites; GATE_EXIT=0).
`slice verify-vt 263` passes all eleven VTs across PHASE-02..04. PHASE-01 and
PHASE-05 are VA/VH phases whose attestations are recorded in `notes.md`. The
neutral pipeline (`retrieve_rows`, `admits`, `dedup_diff`, `cap`,
`format_block`) is unchanged; the diff since 3dd5e8de4 does not touch those
functions. Harness vocabulary stays in the codecs as named constants, and the pi
adapter consults no `PI_SUBAGENT_CHILD` marker, as POL-003 and DEC-286 require.

The only defect found in the recorded evidence was bookkeeping: PHASE-01's span
included a foreign SL-261 commit. It was narrowed (F-1). After that, every
undeclared path is an authored process artefact or a test under a
scope-relevant selector (F-3). The handover's warning about PHASE-05's
multi-commit span was checked; all seven commits belong to SL-263.

Standing risks and accepted tradeoffs:

- **Governance lags code (F-5).** Three specs describe a system without the
  shipped surface, and the policy this slice is governed by is still a draft.
  This is the main remaining work for `/reconcile` and requires the user's act.
- **REQ-018 pointer rendering (F-6, ISS-480).** SL-263 extends SL-205's
  unquoted, unattributed pointer format to two more harnesses. This is accepted
  because the formatter is outside this slice's scope. The risk is that
  approving REV-058/060 is later read as certifying the formatter. It does not;
  ISS-480 remains open.
- **One missing fixture (F-2).** The via-shell `apply_patch` fixture cannot be
  captured on codex 0.155.1. The README records this absence with evidence.
- Deltas already recorded in the design, not reopened here: retrospective
  path nudges on codex and pi (R-3), inert-until-trusted codex hooks (R-2/R-5),
  and no subagent suppression on pi or codex (DEC-286).

## Reconciliation Brief

### Per-slice (direct edit)

- **F-4** — `slice-263.md` Scope § Governance leg and § Affected surface: add
  REV-060 against SPEC-007 (the owning spec of `src/memory.rs` and
  `src/retrieve.rs`, which gains the pointer-surface contract), beside the
  PRD-004 and SPEC-011 revisions.

### Governance/spec (REV)

- **F-5** — approve and apply REV-058 (PRD-004 §2/§8: tool-call-keyed pointer
  surfacing, distinct from an explicit memory request).
- **F-5** — approve and apply REV-059 (SPEC-011: codex hook registry and
  generated pi extensions, including `surface.ts`).
- **F-5** — approve and apply REV-060 (SPEC-007: memory-side pointer-surface
  contract).
- **F-5** — POL-003 `draft → required`, then `doctrine boot` so it enters the
  boot snapshot. `SL-263 governed_by POL-003` is already recorded.
- **F-6** — no write. When applying REV-058/060, do not state that the shipped
  formatter conforms to REQ-018; ISS-480 remains the open owner.
