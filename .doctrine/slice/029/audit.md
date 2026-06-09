# Audit SL-029 — Dispatch worktree creation: detection, creation, guards

Conformance audit (post-implementation). Reconciles the two implemented phases
against `design.md` §2–§5, `plan.toml` (EX/VT/VA/VH), and ADR-006 (worktree
posture). Hand-authored — no audit scaffold yet.

**Mode:** conformance. **Slice rollup:** 2/2 phases complete.
**Gate:** `just check` green; plain `cargo clippy` zero warnings.
**Commits:** PHASE-01 234448c (+ notes cbf7ba5); PHASE-02 511e4f4 (T1),
c2d444f (T2), 1904167/71f32af (notes + memory).

## PHASE-01 — CLI heart (`src/worktree.rs` + `provision`/`check-allowlist`)

Evidence: `cargo test worktree` → **13 pure (lib) + 5 e2e** pass, 0 fail.

| Criterion | Expected | Observed | Disposition |
|---|---|---|---|
| EX-1 | pure core, no disk/git (ADR-001 leaf) | `src/worktree.rs` pure `WITHHELD`/`Tier`/`parse_allowlist`/`is_withheld`/`select_copies`/`allowlist_violations`; IO in the shell | **aligned** |
| EX-2 | `**` cannot leak the tier (skip+warn) | `select_copies` drops WITHHELD matches under `**` — VT-3 load-bearing test | **aligned** |
| EX-3 | `provision` from source root, fail-closed, `-z`-safe, guarded copy | impure shell + `src/fsutil.rs` canonicalize/sibling/symlink guard | **aligned** |
| EX-4 | `check-allowlist` nonzero on tier-naming / unsupported pattern | exit-code test (VT-7) | **aligned** |
| EX-5 | clippy zero, `just check` green, suites unchanged | gate green; behaviour-preservation held | **aligned** |
| VT-1..9 | parse/violations/select/parity + e2e copy/withhold/refuse/fail-closed/absent | all green (13 pure + 5 e2e) | **aligned** |

## PHASE-02 — `/worktree` lifecycle skill + `/execute` solo-isolation thread

Skill-prose phase — conformance is VA read-through + VH acceptance, no suite.
Authored under `plugins/` (source of truth), refreshed into `.doctrine/skills/`
via `doctrine skills install -y` + relinked `.claude/skills/`.

| Criterion | Expected | Observed | Disposition |
|---|---|---|---|
| EX-1 | `worktree/SKILL.md` carries `mode=solo\|worker` + `allow_work_in_place` contract + declared outputs; solo impl, worker declared | present (§"Mode contract"); worker DECLARED-only | **aligned** |
| EX-2 | detection + creation ladder + always-provision rule | present (§Detection w/ submodule guard, §Creation ladder 4 rungs, §Always-provision) | **aligned** |
| EX-3 | three guards (commit-before-spawn `-z`, branch-point pre/post, baseline `just check`) | present (§Guards) | **aligned** |
| EX-4 | `/execute` thin opt-in branch → `/worktree` mode=solo; in-tree default unchanged; worker OFF; `/dispatch` untouched | present (§"Optional: solo isolation"); `git diff` confirms only `execute/SKILL.md` changed, `dispatch` untouched | **aligned** |
| EX-5 | `.worktreeinclude` commented template + squash-orphan record-on-trunk nudge | both present in `worktree/SKILL.md` | **aligned** |
| VA-1 | `/worktree` contract+ladder+provision+guards internally consistent, match design §2/§4/§5 | read-through done — matches in substance (detection, 4-rung ladder, sole-copier invariant + honest F7 caveat, three guards) | **aligned — PASS** |
| VA-2 | `/execute` explicit opt-in (never automatic), delegates mode=solo, in-tree + `/dispatch` untouched | read-through done — matches §5 clause-for-clause | **aligned — PASS** |
| VH-1 | human: prose drives PHASE-01 verbs; split seam reusable by funnel without re-deciding | **NOT self-dischargeable** | **pending human sign-off** (surfaced to user) |

## Findings / dispositions

- **A-1 VH-1 requires human acceptance.** The mode-contract split seam (solo
  IMPLEMENTED / worker DECLARED) is the reuse contract the future funnel slice
  inherits. Both VA read-throughs PASS; the human reusability judgment is
  outstanding. → **pending human sign-off** (the one gate before `/close`).
- **A-2 `.worktrees/` not gitignored in this repo (PD-1).** The `/worktree` skill
  instructs `git check-ignore` + fix-immediately at creation time, so a real fork
  cannot pollute git status. No fork has been created, so nothing to fix now.
  → **aligned** (handled by the skill at creation; not a slice deliverable).
- **A-3 Skill refresh ≠ `doctrine install` (PHASE-02 F-5).** Refreshing installed
  skill content needs `doctrine skills install` (+ `touch src/skills.rs` to force
  RustEmbed re-embed); `doctrine install` only ensures the dir. Knowledge, not a
  defect. → **aligned** — captured as memory
  `[[mem.pattern.distribution.skill-refresh-command]]` (high-trust, verified).
- **A-4 Lifecycle status divergence (⚠).** `slice-029.toml` status is hand-edited
  `proposed` while the rollup is 2/2 complete. This is the known
  no-lifecycle-transition CLI gap, not a slice defect. → **follow-up at `/close`**
  (reconcile terminal status).
- **A-5 Worker-mode behaviour deferred (by design).** `worker` is contract-only;
  full implementation (guard, funnel order, concurrent branch-point) belongs to
  the funnel slice. → **follow-up slice** (design §5 / §6 "Deferred"). The scope
  reconciliation already moved `/dispatch` out of SL-029.

## Closure readiness

Design and code agree across both phases; every finding is dispositioned. The
single blocker to clean closure is **A-1 (VH-1 human sign-off)**. A-4 (status
reconcile) is `/close`'s job; A-5 is owned future work. Ready for `/close` once
the human accepts VH-1.
