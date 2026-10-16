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


## Synthesis

**Closure story.** SL-281 makes `doctrine install` act on ADR-019's *owned*
axis. The manifest classifies framework roots and per-harness def sets
(PHASE-01). A pure ownership plan (PHASE-02) drives exclusive staging
(PHASE-03) and a convergence executor that re-proves every entry before
mutating it (PHASE-04). A managed `.gitignore` block replaces the hand-kept list
(PHASE-05). The repo dogfoods it (PHASE-06). All 13 VT criteria pass, and
`doctrine check gate` is green at `9dff138fa` and again after the fix-now
commit `6fd47ea0b`. The safety contract held under adversarial probing:
nothing user-owned was deleted in any reproduction, and no finding is a
blocker.

**What the audit changed.**
- Fixed in code (`6fd47ea0b`):
  - F-1: a symlinked canon container no longer freezes its members.
  - F-15: an unreadable entry is disclosed, not classed as a real file.
  - F-9: install prompts say what confirming mutates.
- Fixed in the repo:
  - F-5: the pi dispatch-worker link is untracked.
  - F-2: PHASE-03..06 decisions and the user's assent are harvested into
    `notes.md` (`a62526d64`).

**Standing risks.**
- `.agents/skills` is still written by npx from GitHub, not the local build,
  so a dogfooded repo can carry retired or stale skills (F-6, owned by
  IMP-406). ISS-353 is closed only for doctrine-linked harnesses.
- A canon container that is a dangling symlink, or a symlink to a file,
  records no member states. A write there fails at apply time, as before
  (pre-existing; noted in the fix hand-back).
- The F-9 convergence wording is single-sourced in `canon_convergence()` as
  two match-arm literals rather than named constants. That meets STD-001's
  intent, if not its letter; accepted.

**Tradeoffs consciously accepted.**
- PHASE-03..05 shipped on a substitute gate, forced by the cargo 1.100 bump
  and later covered by CHR-178 and green gates (F-12).
- PHASE-03 never observed red (F-13).
- The `.gitignore` comment rationale now lives only in
  `install/manifest.toml` (F-16).
- User-visible output changes:
  - Already-correct links print nothing (D1).
  - The install prompt and summary lines are reworded; there is a new
    `canon` summary line and an `unreadable` report row.

## Reconciliation Brief

### Per-slice (direct edit)
- F-3 — `design.md` sec-convergence flowchart (`KF` node) and §Links prose:
  a foreign symlink is kept and reported **only at a member link site**;
  elsewhere it is ignored (PHASE-02 OQ-1).
- F-4 — `design.md` §Shape / §Convergence and the report vocabulary. Record:
  - D3: convergence runs only when at least one harness is confirmed; the
    gitignore block is written regardless.
  - D4: abort on the first I/O or bake error (`framework install failed`).
  - D1: an already-correct link emits no line.
  - The new rows from the fix: `skipped … (changed since scan)` and
    `unreadable … (left untouched)`.
- F-7 — `design.md` sec-ownership: replace "nothing else lists a framework
  path" with "the manifest is the source; `canon_root()`'s kind→root map is
  test-pinned to it (`canon_roots_match_manifest_framework_roots`), as with
  `DERIVED_RUNTIME`".
- F-14 — `design.md` sec-verification: move "only `.pi/` wires the codex boot
  adapter" from the real-binary e2e list to unit coverage
  (`detection_and_boot_follow_registry`).
- F-10 — **selector registry** (load-bearing):
  - `doctrine slice selector add 281` design-target `src/fsutil.rs`,
    `tests/e2e_install_ownership.rs`, `.doctrine/adr/001/layering.toml`,
    `.pi/agents/dispatch-worker.md`.
  - scope-relevant `src/memory.rs`.
  - Mirror in `design.md` §6.
- F-8 (design half) — `design.md` DEC-372 text: the full def set projects on
  an explicit `--agent` **or a detected harness the user confirms (prompt or
  `-y`)**.

### Governance/spec (REV)
- F-8 — REV-071 D7 row (still proposed): amend to "explicit `--agent`, or
  detected and confirmed". Check that ADR-019's VA ("nothing projected by
  default") reads as satisfied, since confirmation is a request. Settle the
  DEC-372 knowledge record's wording to match.
- F-6 — ISS-353: on close, resolve as fixed **for doctrine-linked harnesses**
  only, with the npx-written `.agents/skills` remainder pointed at IMP-406.
  This is a backlog transition, not a spec write.

### Governance/spec (REV) — addendum
- F-17 — SPEC-009 and SPEC-010 residual passages (toml responsibilities,
  D5/D4 headings, SPEC-010 overview, dual-path routing, D2, hypotheses) →
  second REV "SL-281 spec residuals", `modify` rows, excerpts aligned to
  REV-071's landed text.
