# Review RV-420 — reconciliation of SL-281

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->
Mode: **conformance** (post-implementation self-audit). Surface: the primary
tree on `edge`, SL-281 commits `e523e95c0`..`9dff138fa` (PHASE-01..06), with
CHR-178 (cargo 1.100 adoption, `5512d9a61`) as a pre-PHASE-06 dependency.

Lines of attack:

1. **Design conformance** — `design.md` decisions (DEC-372..377: manifest
   classification, per-harness full def set, canon convergence on project
   install / report-only on global, shared link-dir ownership by symlink target,
   managed gitignore block) against `src/install.rs`, `src/install_ownership.rs`.
2. **Prune safety** — nothing user-owned is ever deleted: regular files in link
   dirs kept; symlinked-parent scan; executor re-check before each mutation
   (SPEC-010 REQ-178); `--dry-run` writes nothing.
3. **Plan criteria** — every phase's EX-/VT- criteria evidenced by a named test
   or recorded agent check; VT test attribution recorded.
4. **Conformance algebra** — 21 undeclared paths (`src/fsutil.rs`,
   `src/memory.rs`, `tests/e2e_install_ownership.rs`, ADR-001 `layering.toml`,
   `.pi/agents/dispatch-worker.md`, observation records): scope creep or missing
   selectors.
5. **Orchestrator-made decisions** taken without the human (PHASE-04 D1–D6,
   PHASE-05 dry-run/malformed-marker/no-harness rules, PHASE-06 OQ-1..6) —
   each consistent with design and spec, or a reconcile item.
6. **Known leads from hand-backs** — install populates `.agents/skills` from
   GitHub not the local build; managed block lists a tracked path
   (`/.pi/agents/dispatch-worker.md`); dry-run inaccuracies ("would write" on
   unchanged files, shadowed link lines omitted); lost `.gitignore` comment on
   observation records; PHASE-03 worker skipped red.
7. **Governance** — ADR-019 (asset policies), ADR-001 layering, SPEC-009 /
   SPEC-010 requirements touched; STD-001 (magic strings), STD-003 (no silent
   skip — every skipped/kept artefact disclosed).

Gate: `doctrine check gate` green at `9dff138fa` (orchestrator re-run).

