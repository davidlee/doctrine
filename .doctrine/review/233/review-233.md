# Review RV-233 — reconciliation of SL-180

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation audit of a dispatched slice).

**Reviewed surface (F-2 / dispatched-slice caveat).** The candidate interaction
branch `candidate/180/review-001` (`264c4161`, worktree
`.doctrine/state/dispatch/candidate/cand-180-review-001`), projected from the
`review/180` impl-bundle onto `refs/heads/main`. `main` is the exact merge-base of
`dispatch/180` (`main ⊆ dispatch/180`), so the bundle diff `main...review/180` is
the whole and only slice delta: 5 files, +518/−63. Evidence refs (immutable, R2):
`review/180` `73b7ddd8`, `phase/180-01` `7afe63c7`, `phase/180-02` `f3fd6652`.
Build/test evidence gathered on the code-identical coord tree (warm target).

**Lines of attack / invariants:**
- **Conformance algebra** — is every touched path declared (no scope creep), and
  is every declared selector delivered? Undeclared is the highest-signal cell.
- **Layering (EX-5, ADR-001)** — `worktree::import` (engine) must hold zero
  `crate::slice` (command-tier) references; the selector read is the sole
  command-tier seam in `worktree::mod`.
- **Non-ASCII / F8 (design §4)** — `core.quotePath=false` folds verbatim UTF-8
  keys; `--strict` requires `--against`; a non-`..` value yields the rev-range
  diagnostic.
- **Behaviour preservation** — the existing conformance/registry suites stay green
  unchanged (ASCII paths); no new or changed regression failures.
- **VT existence/shape** — all five VT mandates match their mandated file+token.

**Known drift entering audit (to disposition, not rediscover):**
- `dispatch-subprocess/SKILL.md` reads *undelivered* — flagged at plan time
  (EX-4 reconcile note). Premise to test: does that arm even have an import
  invocation to annotate?
- Worker skipped `cargo fmt`; remediated in-drive (`33599284`).
- VT-2 needed an in-drive reconcile (verify-vt line-anchored pattern vs a
  rustfmt-wrapped assert) — captured as IMP-235.

## Synthesis

SL-180 delivered both phases through the dispatch claude arm and the deliverable
is conformant and correct. `slice conformance` reports **0 undeclared** (no scope
creep), 5 conformant, 1 undelivered — and that lone undelivered
(`dispatch-subprocess/SKILL.md`) is authored-plan truth drift (F-1), not a code
gap. All five VT criteria PASS on the code surface; parent-tree `verify-vt` FAILs
only because the deliverable is not yet on `edge` (expected — landing is `/close`'s
job, post-audit). `check gate` exited 0: `cargo fmt` clean, production `cargo
clippy` clean, every suite green, and the embedded S1 regression diff reported
`✓ no new or changed failures` — behaviour preservation held (ADR-001 gate green,
ASCII conformance suite unchanged).

**EX-5 layering verified directly, not trusted:** `worktree::import` (engine) holds
zero `crate::slice` (command-tier) references; the one new cross-module edge is
engine→leaf `crate::conformance::undeclared_paths`; the sole command-tier selector
read (`crate::slice::selectors(.., DesignTarget)`) lives in `worktree::mod`. The
`worktree → slice` top-level edge already existed via `coordinate.rs`, so no new
`accepted_violation`.

**Standing risks / tradeoffs consciously accepted:**
- **Clippy toolchain drift (environmental, not a finding).** `cargo clippy
  --all-targets -D warnings` is pre-existing-red in the jail (clippy 0.1.97 vs the
  pinned beta.5; `--all-targets` lints test code lacking allow-in-tests keys,
  flooding `memory.rs`). The sanctioned gate (`check gate` / production clippy) is
  green. Pre-existing, out of SL-180 scope.
- **Worker quality (Opus — Sonnet-vs-Opus eval).** One spawn, base-guard passed,
  layering-clean, 10 tests, `import.rs` 0 `crate::slice`. Correct architecture on
  the first pass — but skipped `cargo fmt` (remediated `33599284`) and arranged a
  fixture that tripped VT-2's `café` shape (remediated `380c1403`). Net: two
  orchestrator reconcile rounds for style/VT-shape, none for logic. Signal: Opus
  nailed the hard multi-file layering split; the misses were mechanical hygiene the
  worker prompt could pre-empt (explicit `cargo fmt` + VT-token-on-one-line
  reminders).
- **verify-vt line-anchored patterns (F-2 → IMP-235).** A formatter-wrapped assert
  false-Fails; deferred to the backlog with three remediation options.

Both findings are terminal (verified); no blocker. The slice is ready to reconcile.

## Reconciliation Brief

### Per-slice (direct edit)
- **plan.toml EX-4 (PHASE-02)** [F-1]: rewrite so `--slice` is scoped to arms that
  actually invoke `worktree import`. dispatch-subprocess records via `slice
  record-delta` (no import invocation), so it is not a required-edit site. Drop it
  from EX-4's "both skills" requirement.
- **design.md §6 code-impact, line 254** [F-1]: the design-target line
  `plugins/doctrine/skills/dispatch-subprocess/SKILL.md  # same (subprocess arm)`
  is wrong — the subprocess arm has no import invocation to annotate. Drop or
  annotate it so it is no longer a declared edit target.
- **design.md §3 "Belt firing in practice (F4)", lines 138–144** [F-1]: the "the
  orchestrator *always* passes `--slice` on the coordination-root import" framing
  holds for the agent arm; add a note that the subprocess arm carries scope via
  `record-delta`, and that `--slice` is CLI-threaded to *both* import arms so
  subprocess is covered automatically if it ever gains an import invocation.
- **design-target selector set** [F-1]: after the above, `slice conformance SL-180`
  should no longer report `dispatch-subprocess/SKILL.md` as undelivered (the
  selector is removed/annotated), leaving a fully-delivered registry.

### Governance/spec (REV)
- **(none).** No ADR/REQ/POL/STD change. ADR-001 layering upheld and independently
  verified; behaviour-preserving for ASCII; no governance surface touched.
