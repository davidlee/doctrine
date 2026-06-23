# Review RV-148 — design of SL-147

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Tribunal: the design of SL-147 (RFC-004 v0.1 audit path-conformance delta),
design.md as it stands committed. Two witnesses: the Inquisitor (in-house) and an
external adversary (codex/GPT-5.5, project-default reviewer), whose testimonies
corroborate.

Lines of attack, and the doctrine each is held to:
- **RFC-004 v0.1 fidelity** — the design must not silently ship against a
  superseded or self-contradictory RFC cut (RFC v0.1 "L2 never stored / no
  writer" vs OQ-11a "record SHA at land via funnel writer").
- **POL-002** — no load-bearing on host commit/branch conventions; compute only
  from doctrine-owned contracts.
- **Conformance soundness** — partial/missing boundary coverage must not produce a
  false-clean diff; the actual-side status must be well-defined; the recorded
  range must be constrained (ancestry / non-merge) so trunk cannot pollute it; the
  `undeclared` cell must stay high-signal against broad globs.
- **domain_map burn** — must not break the RV-generic staleness reader
  (`run_status`) or non-slice RV targets, nor depend on fork-refused authored
  reads (review.rs:1953).
- **Cross-worktree state (R5)** — the registry must be visibly + safely shared;
  reuse the existing primary-worktree resolver (worktree/subagent.rs:33), do not
  reinvent, and justify the cross-worktree write against the gitignored-co-write
  anti-pattern the codebase already shuns.
- **ADR-001 layering** — BoundaryRow reuse is permitted only via a leaf extraction.
- **Verification realism** — the prove-value dogfood must be achievable.

## Synthesis

**Judgement: heresy found, and cleansed.** The design as first committed was *not*
lock-ready — eight charges stood, two of them blockers. Both witnesses (the
Inquisitor and the external adversary, codex/GPT-5.5) concurred on F-1…F-7;
F-8 was the Inquisitor's alone. Every charge is now terminal (verified); the
design artifact carries the reconciled truth.

The grave heresies and their penance:

- **F-1 (blocker) — heresy at the source.** RFC-004 contradicted itself: the v0.1
  "no orchestrator writer / L2 never stored" cut versus its own later OQ-11a
  "record the source-delta SHA at land." SL-147 silently obeyed the later arm.
  Penance: the RFC was reconciled (the "never stored" rule now governs the L2
  *file-set*, not the SHA boundary identity; OQ-11a's thin recording beat is
  consistent; the solo arm depends on an *explicit* recording call). The
  contradiction was struck at its root, not papered over downstream.
- **F-2 (blocker) — the false-clean diff.** Degrading only on an *empty* registry
  let *partial* coverage emit a confident, wrong verdict — the worst sin for an
  audit gate. Penance: fail closed — cross-check recorded rows against the slice's
  completed phases, one row per landed phase (explicit zero-delta, never
  omission), `incomplete` on any mismatch.
- **F-3/F-6 — the under-constrained delta.** A lossy status fold and an
  unenforced "trunk contributes nothing" assumption. Penance: an ordered per-path
  event set with a pure, tested `net()`; and a write-time ancestor + non-merge
  guard so a wide boundary fails loudly, not silently.
- **F-4 — the burned generic reader.** Burning `domain_map` would have broken
  `run_status` for non-slice RV targets. Penance: scope the selector source to
  slice-backed RVs; fail clearly elsewhere.
- **F-5 — reinvention + a misread precedent.** The design reinvented an existing
  `primary_worktree` resolver (real heresy — purged: reuse subagent.rs:33) and was
  charged with the cross-worktree gitignored-co-write anti-pattern (a *contextual*
  ban — rebutted: the writer is the un-jailed sole-writer orchestrator, no baton,
  ADR-006). The User's skepticism of context-free "shunning" was vindicated.
- **F-7 — the diluted signal.** Broad `design-target` globs could swallow
  surprises. Penance: report the matched selector per conformant path; lint
  deferred.
- **F-8 — the circular proof.** The dogfood presupposed its own machinery.
  Penance: an explicit, achievable demo target.

**Standing risks (tolerated, eyes open):** the dispatch landing-beat write (R1)
remains the one touch to a live dispatch path — kept thin, gated on existing
dispatch tests; the glob-breadth *lint* is deferred to follow-up, transparency the
v0.1 guard; post-close auditability is out of scope by design (in-loop registry).

**Verdict: the design may proceed to `/plan`.** No blocker stands. DOCTRINA MANET.