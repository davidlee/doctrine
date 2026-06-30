# SPEC-002: Requirement Reconciliation Engine

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The mechanism realising **PRD-013** (requirement reconciliation): the engine that
keeps evergreen requirement/spec truth coherent with shipped reality **by explicit
authorship, never by derivation**. It is a set of components inside the doctrine
container — no new crate — composed around one hard line: **observed evidence and
authored truth never touch through a function** (PRD-013 §3; `REQ-105`).

```text
  doctrine CLI   (coverage record/show · reconcile · slice status / close gate)
        │
  ┌─────┴──────────────────────────────────────────────┐  doctrine container
  │                                                      │
  │  coverage substrate (slice-side, git-anchored)       │  observed tier
  │    mode-discriminated entries: (req × change, VT/VA/VH,
  │    status, anchor, [attested])  ── composite = DERIVED view
  │        │ supports (read-only)                         │
  │  drift surfacer (derived read: authored vs composite) │
  │        │ prompt, never a write                        │
  │  reconcile writer (SOLE author of reconciled truth)   │  authored tier
  │    accept → req-status write · revise → spec-truth write
  │    redesign → ADR-009 reconcile→design escalation
  │        │ emits exactly one                            │
  │  REC entity corpus (.doctrine/rec/NNN — REC-NNN)      │  authored tier
  │        │ gates                                        │
  │  closure gate (default-refuse residual drift +        │
  │    recorded override; topology = ADR-009 F12)         │
  └──────────────────────────────────────────────────────┘
```

The **observed tier** (coverage entries) and the **authored tier** (requirement
status, spec truth, REC) are physically separate stores. Evidence flows *up* into
judgement (the reconcile writer reads coverage); authority never flows *down*
(no path writes authored status from coverage). That asymmetry is the spec.

**Relations** (cross-corpus edges are prose — `interactions.toml` is tech→tech
only; the gap is IMP-016). Descends from **PRD-013**. Governed by **ADR-003**
(observe→reconcile→close; explicit-authorship-not-derivation) and **ADR-009** (the
slice FSM `reconcile` state, conduct axis, and the F12 closure-seam topology this
gate sits atop). Composes **PRD-010** (`knowledge_record` — REC reuses its evidence
sub-structure and cites a `DEC`; REC is *not* one). Writes into **PRD-002** entities
(requirement status, spec truth). Reuses the memory git-anchor/staleness seam
(SL-007/008, `src/git.rs`). A **Drift Ledger** (mass-divergence reconciliation) is
out of scope (PRD-013 narrow boundary); the **RV review-ledger** (IMP-001) is a
sibling record family.

## Responsibilities

Own the **REC** reconciliation-record entity (its corpus, schema, and slice
relation); own the **coverage substrate** (mode-discriminated, slice-side,
git-anchored entries and the derived composite view); fix **coverage freshness
physics** and reuse the memory staleness seam for attestation decay; hold the
**two-tier separation** (no derivation path); own the **drift surfacer** (derived
read); own the **reconcile writer** (sole author, accept/revise/redesign, one REC
per act); and own the **closure gate** (default-refuse + recorded override atop the
ADR-009 topology).

## Concerns

- **Two-tier purity (load-bearing).** The acceptance proof is structural: no
  function maps coverage → authored status anywhere (`REQ-105`; PRD-013 forbidden
  derivation). Coverage entries and authored requirement status live in distinct
  stores; the reconcile writer is the *only* code that reads the former and writes
  the latter, and it writes a human/agent-authored value, not a computed one.
- **Determinism of derived reads.** The per-requirement composite coverage view and
  the drift surfacer are pure folds over authored + observed state — same inputs at
  a ref → same view. No clock/RNG/map-iteration order in the fold (pure/imperative
  split; clock/git/disk in the thin shell). Any cache is disposable; correctness is
  recomputation.
- **Freshness physics differ by mode.** VT evidence is re-derivable (re-run the
  suite); VH/VA evidence is irreducible attestation that *decays* as code moves past
  its git-anchor. Staleness is **surfaced, never auto-demoted** (ADR-003 §5 / ADR-009;
  mirrors memory staleness) — a stale `verified` attestation is flagged, not silently
  flipped to `failed`.
- **Contracts deferred.** No automated observed-truth corpus yet (PRD-013 §2). v1
  rests on audit close-reading + git-anchored attestations; VT entries record the ref
  last observed green and *become* continuously re-derived when a contracts/test-run
  surface lands. The engine must not depend on contracts existing.
- **Failure mode — evidence unobtainable.** Coverage that cannot be established is an
  entry with status `blocked`, surfaced, never defaulted to `verified` (PRD-013).
- **Override attributability.** A closure override is not a flag that suppresses the
  gate — it is a reconciliation act that records *accepted residual drift* with
  rationale and evidence, so "closed with **unreconciled** drift" is unrepresentable.

## Hypotheses

- **H1 — the memory staleness seam is the coverage-attestation staleness mechanism.**
  `src/git.rs` born-frame capture + the SL-007/008 verify/staleness machinery
  (git-anchor, `verified_sha`, staleness surfacing) apply to a VH/VA coverage
  attestation unchanged. *Challenge:* if a coverage anchor needs per-requirement
  granularity the memory anchor lacks, the seam widens at the leaf, not a fork.
- **H2 — REC rides the existing numbered-entity wiring.** A new numbered kind
  (`REC`, 3-char prefix) needs only the standard wiring — `integrity::KINDS` row,
  manifest dir, gitignore negation, prefix→kind resolution — not new engine
  machinery (`Kind` is data, not a trait). *Challenge:* the owning-slice relation may
  need an outbound edge surface the current relation model doesn't expose.
- **H3 — the composite view is a fold, not a graph engine.** Per-requirement
  composite coverage and drift are a straight fan-in over a requirement's entries; no
  `cordage`-scale graph is needed (unlike SPEC-001). *Challenge:* if composition rules
  grow (precedence across modes/changes), revisit — see OQ-3.
- **H4 — the evidence sub-structure is shared with `knowledge_record`.** REC's
  `supports/contradicts/notes` evidence block is the PRD-010 structure. Until
  knowledge_record lands, REC defines it; it lifts to a shared type when the second
  consumer appears (no parallel impl). *Challenge:* sequencing — see OQ-2.
- **H5 — the closure gate rides the slice-status transition seam.** The gate is a
  predicate on the `reconcile → done` edge (ADR-009), not a new lifecycle. *Challenge:*
  the override path needs a recorded artefact (a REC), so the gate reads REC state.

## Decisions

- **D1 — REC is a first-class numbered entity.** Id `REC-NNN`; own corpus
  `.doctrine/rec/NNN/` (`rec-NNN.toml` + `rec-NNN.md`). `rec-NNN.toml`: `status_deltas`
  = `[(requirement, from, to)]`, `move ∈ {accept, revise, redesign}`, `evidence_refs`,
  `owning_slice` (**optional** relation — its optionality is *why* a freestanding REC
  works), optional `decision_ref` → `DEC`. `rec-NNN.md`: rationale scratchpad / the
  approval surface. Resolves PRD-013 **OQ-1**. Citation is by stable id, durable past
  slice close (`REQ-102/107`).
- **D2 — REC composes, does not subclass, `knowledge_record`.** REC is not a PRD-010
  kind (its lifecycle is an act record, not belief/decision/question/rule; PRD-010 §3
  routes such artefacts away). It reuses PRD-010's evidence sub-structure and *cites*
  a `DEC` for a non-obvious call. Both are forward deps (knowledge_record unbuilt).
- **D3 — observed coverage is mode-discriminated, slice-side entries.** Entry =
  `(requirement, contributing_change, mode ∈ {VT,VA,VH}, status ∈ {planned,
  in-progress, verified, failed, blocked}, git_anchor, [attested_date])`. Stored
  **slice-side** — the contributing change owns the evidence it established — so
  several slices touching one requirement compose with **no clobber**. Resolves
  PRD-013 **OQ-2** (granularity = per requirement × contributing change).
- **D4 — the per-requirement composite is a derived view, never a stored scalar.** A
  stored rollup would read as derivation and blur the tiers. Composition is a pure
  fold over a requirement's entries across changes; computed on read.
- **D5 — coverage freshness is mode-typed.** VT entries are re-derivable (v1 records
  last-green ref; live re-derivation when contracts/test-run lands). VH/VA entries are
  point-in-time attestations that decay via the memory git-anchor seam (H1).
- **D6 — the drift surfacer is a derived read.** Divergence = authored requirement
  status vs composite observed coverage; emitted as a prompt. It has no write path to
  authored truth (`REQ-105`).
- **D7 — the reconcile writer is the sole author of reconciled truth.** It is the
  only code that writes authored requirement status and reconciled spec truth in the
  loop. Per divergence it applies one move: **accept** (write req status to match
  evidence), **revise** (write corrected spec truth), or **redesign** (escalate
  `reconcile → design`, ADR-009 — no instance write). It emits exactly one REC per
  reconciliation act.
- **D8 — the closure gate default-refuses residual drift; the override is narrowed,
  not uniform (REV-017).** Moving a slice to **`done`** checks its owning specs carry
  no outstanding unreconciled drift (the `abandoned` terminal is a distinct giving-up
  exit and is **not** coverage-gated). Residual drift → refuse, and what may override
  depends on the drift's source:
  - a live **`Failed`** cell (a check that *ran and contradicted*) is **not**
    override-acceptable — no accept-REC discharges it; it must be fixed (the cell
    re-derived to `Verified`) or the requirement withdrawn via a recorded act;
  - a live **`Blocked`** cell (evidence *unobtainable*) is override-acceptable **only**
    when the requirement also carries a fresh **human (VH)** `Verified` cell and the
    override REC cites both keys (stricter than status-lag);
  - **status-lag** (`EvidenceOutrunsAuthored` — authored status trails confirmed
    evidence) keeps the unchanged recorded-override path;
  - **withdrawing** a requirement that still carries a live `Failed`/`Blocked` cell is
    itself a reconciliation act — a slice-owned `revise`/`redesign` REC citing the
    evidence keys, not a bare status flip.

  The override **is** a reconciliation act (a REC recording the accepted residual
  drift + rationale), so closed-with-*unreconciled*-drift is unrepresentable. The
  topology edge (`done` only from `reconcile`) stays ADR-009 F12 hard. Resolves
  PRD-013 **OQ-3**.
- **D9 — CLI surface is named, shapes deferred.** A coverage recorder/reader, the
  `reconcile` writer, and a closure-gate hook on the slice-status / `/close` seam. The
  CLI is the source of truth for exact verb/flag shapes; they settle at build, not in
  this spec.

## Open Questions

- **OQ-1 — REC corpus on-disk convention.** D1 fixes the first-class corpus; the
  precise symlink/alias convention (mirroring `mem.<key>` / `nnn-slug`) settles when
  the entity wiring is built.
- **OQ-2 — shared-evidence type ownership/sequencing.** If reconciliation builds
  before PRD-010's `knowledge_record`, REC owns the evidence sub-structure and it is
  lifted to a shared type later; if knowledge_record lands first, REC consumes it.
  The build order decides; neither forks the type (H4).
- **OQ-3 — composite precedence rules.** When one requirement's entries disagree
  across modes/changes (e.g. a stale VH `verified` vs a fresh VT `failed`), the
  composite-view precedence is unspecified. v1 may surface all and let the writer
  judge; a precedence rule is a later refinement (H3 challenge).

## Appendix A — REC entity wiring checklist (build-time)

A new numbered kind rides the standard seam (no engine change; `Kind` is data):
`integrity::KINDS` row (prefix→kind, id table), manifest directory, gitignore
negation for the authored tier, scaffold/show/list/validate registration. See the
numbered-kind identity table and authored-entity-wiring patterns in memory.

## Appendix B — forward dependencies

- **`knowledge_record` (PRD-010)** — REC's evidence sub-structure and `DEC` citation
  (D2, OQ-2). Not a build prerequisite; REC degrades to inline rationale until it lands.
- **Contracts (deferred, ADR-003 §11)** — turns VT entries from last-green-ref records
  into continuous re-derivation (D5). The engine must not depend on it.
- **Cross-corpus relation surface (IMP-016)** — would let the PRD-013/010/002 edges and
  the REC↔slice relation be structural rather than prose.
