# Review RV-422 — reconciliation of SL-282

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Mode: **conformance**, self-audit plus one adversarial external pass (codex-cli,
gpt-6-sol). Surface: primary worktree, `edge`, `a2df6d6fe..71ebbfe86`
(PHASE-01..07, capsule-driven, no dispatch).

Lines of attack:

1. **Design conformance** — each locked decision (DEC-378..385) and the design's
   code-impact table against the delivered code; rulings the capsules made where
   the design was silent or departed (PHASE-03 exit-0 on incomplete install;
   PHASE-07 keep-when-newer on every install; PHASE-05 pluralised bullet;
   PHASE-03 `plan_wire` narrower than design).
2. **RV-421 control findings** F-1, F-3, F-10, F-14, F-16, F-17, F-18 — each EX
   row that transcribes them is backed by a test that fails red without the fix.
3. **Invariants** — stamps written last and only on full success; tracked stamp
   never downgraded; `doctrine.toml` byte-identical outside the key; legacy
   starter removal deletes only byte-identical, unsealed-default-sidecar files;
   commit hint is path-limited and omits gitignored paths.
4. **Governance** — ADR-001 layering (`status` engine calls the currency
   reader); STD-001 magic strings; STD-003 degraded reads disclosed
   (linked-worktree lookup failure reads as "not installed").
5. **Conformance algebra** — 33 undeclared, 1 undelivered (`src/dtoml.rs`).
6. **Spec drift** — SPEC-023 / REQ-322 / REQ-329 / DEC-384 describe the
   retired expose/starter model.

Evidence run: `doctrine check gate` exit 0; `slice verify-vt 282` all PASS;
`slice conformance 282` as above.


## Synthesis

SL-282 delivered all seven phases (PHASE-07 appended for the downgrade gate,
split from PHASE-03). Evidence: `doctrine check gate` exit 0 before and after
the audit fixes; `slice verify-vt 282` all PASS; RV-421 control rows (F-1, F-3,
F-10, F-14, F-16, F-17, F-18) each backed by a test shown red first.

The adversarial pass (codex-cli, gpt-6-sol, high) found four real defects in
the new install machinery, all fixed in `e50b1a131` with red-first tests:
an edit made during the `Proceed?` prompt could be deleted as an "unedited"
starter (F-1, blocker); a marker-write failure advanced the team stamp (F-2);
a malformed `.claude/settings.json` let install stamp "current" with no hooks
wired, breaching RV-421 F-14 (F-7); a failed linked-worktree lookup read as
"not installed" (F-3, STD-003). The user added F-10: the commit hint used `&&`,
which nushell rejects; it now prints one command per line.

Tradeoffs consciously accepted:
- Incomplete install exits 0 (it prints `install incomplete: …` and writes no
  stamps); `-y` downgrade refusal exits non-zero.
- Keep-when-newer applies to the tracked stamp on every install, not only under
  `--allow-downgrade` — it protects the team pin.
- `plan_wire` plans only the `@`-import edits (F-4); harness refresh stays in
  `apply_wire`.
- The commit hint omits paths git ignores (F-5, user ruling, amends DEC-382);
  a failed ignore query is disclosed and the hint falls back to every path.
- With git unavailable, the marker lookup now reports "main worktree not
  located" rather than guessing (F-3).

Standing risks:
- The hint lists files written, not files changed; byte-identical rewrites can
  appear. Harmless to `git add`.
- VH-1 (PHASE-06): the user's interactive scratch-repo check of the overview
  and hint readability is still pending.

## Reconciliation Brief

### Per-slice (direct edit)
- F-4 — `design.md` sec-flow / sec-code-impact: state that `plan_wire` is pure
  over `@`-import edits only and harness refresh (hooks, registrations) stays in
  `apply_wire`. PHASE-03 EX-2's wording is immutable; note the divergence in the
  design rather than touching `plan.toml`.
- F-6 — selector registry (load-bearing): `doctrine slice selector add` for
  `src/git.rs`, `src/commands/guard.rs`, `src/main.rs`, `src/worktree/mod.rs`,
  `src/worktree/shared.rs`, `tests/common/mod.rs`,
  `tests/e2e_prompt_resolve_golden.rs`, `tests/e2e_claude_install.rs`;
  `doctrine slice selector rm` for `src/dtoml.rs`. Mirror in `design.md`
  sec-code-impact. Observations, dogfood files and the shipped-memory doc sweep
  are incidental, not design targets.
- F-5 — `design.md` sec-flow "Commit hint": replace "no git ignore query" with
  the check-ignore filter and its disclosed fallback; one command per line
  (F-10).

### Governance/spec (REV)
- F-5 — DEC-382: amend the hint rule — the hint omits paths git ignores (one
  batched `git check-ignore`); on query failure it discloses and lists every
  path; commands print one per line with shell-neutral quoting. Basis: user
  ruling 2026-10-17 (option A).
- F-8 — SPEC-023 (starter/expose sections, roughly §50-60 and §113-158) and its
  `interactions.toml`, REQ-322, REQ-329, DEC-384: rewrite from install-time
  `[hymns].expose` projection to starters on demand (`doctrine prompt
  customise <slot>`, refusing before install has stamped, sealed and unknown
  slots) plus the one-time legacy removal of byte-identical copies. State that
  install no longer back-fills a missing sidecar beside an edited starter, so
  such a starter appends to the framework text instead of replacing it.
