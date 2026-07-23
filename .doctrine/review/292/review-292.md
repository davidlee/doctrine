# Review RV-292 — design of SL-225

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This second Inquisition arraigns the post-RV-291 remediation of **SL-225's
design facet**, with DEC-003's self-locating recipe as the principal accused.
It re-tries each prior charge and presses the replacement mechanism at these
seams:

1. Whether `--git-common-dir` locates the coordination worktree on both dispatch
   arms, and where each arm actually runs its verification gate.
2. Whether fork-build-before-coord precedence is sound when `validate` precedes
   `build`, especially for a phase whose own delta changes a validation rule.
3. Whether the coord binary is guaranteed built before any gate, or absence
   merely changes ISS-218 from false-red to hard refusal.
4. Whether `git-dir != git-common-dir` identifies a dispatch fork or wrongly
   captures every solo/human linked worktree.
5. Whether the selected git plumbing is compatible with the project's supported
   Git floor and fails cleanly when unavailable.
6. Whether POL-002's project/platform boundary remains intact.
7. Whether any live `current_exe()` publication or launch-frozen
   `DOCTRINE_BIN` precondition survived the F-2 sweep.
8. Whether VT-1/1b truly discriminate coord-green from stale-PATH-red through
   the real `worker_commit` seam, rather than proving precedence by mock.

Sanctioned authorities: POL-002, ADR-008, ADR-012, SPEC-012, SPEC-021, RFC-016,
the SL-225 scope, DEC-003, ISS-218, RV-291, and current dispatch source. The
trial is confined to design intent; implementation conformance is not
commingled.

## Synthesis

**Judgement.** DEC-003 is not fit to advance from `proposed`, and SL-225 must
not pass from design to plan. The engine remains clean under POL-002, but the
project recipe's load-bearing Git inference is false: a worker fork and its
coordination worktree share the repository's **primary** common directory.
Parenting `--git-common-dir` therefore finds the primary tree, not the coord
tree. The alleged self-location is a painted door upon a stone wall.

### Re-trial of RV-291

- **RV-291 F-1 — STILL-STANDING.** The launch-frozen environment precondition
  has been correctly rejected, but its replacement cannot locate the coord
  build. In a linked worker, rung 3 selects the primary `edge` build when it
  exists or rung 3a refuses when it does not. ISS-218 is therefore restored,
  not dissolved. The claude-arm gate does run in the worker fork as claimed;
  that fact does not change Git common-directory identity. The subprocess arm
  has no `worker_commit` self-commit gate, though a worker-side `just validate`
  in its linked fork suffers the same locator error.
- **RV-291 F-2 — RESOLVED.** DEC-003, the scope, and the design carry no live
  `current_exe()` publication. Every occurrence presents it as the rejected
  redundant-or-harmful alternative, and the design invariant explicitly leaves
  `worker_commit`'s environment untouched. The surviving mandatory
  `DOCTRINE_BIN` prose in governance is a separate lifecycle contradiction
  captured by RV-292 F-3, not a resurrection of `current_exe()`.
- **RV-291 F-3 — RESOLVED.** VT-1 now names a rule-divergent coord binary,
  intentionally stale PATH binary, non-Rust fork, and the real
  `worker_commit` seam; VT-1b names the missing-build refusal, while VT-1c is
  honestly demoted to precedence-only. This is discriminating design intent.
  Faithfully implemented, it should fail against the present locator and expose
  F-1 rather than conceal it.

### New heresies

1. **F-1 (blocker): false Git topology.** Source uses ordinary `git worktree
   add` for coord and worker trees, and `src/git.rs` expressly warns that
   `parent(--git-common-dir)` is not a worktree-root oracle. A live linked tree
   returned primary `.git` as its common directory. The same broad signal also
   mistakes every solo/human linked worktree for a dispatch fork, selecting the
   primary binary or hard-refusing it.
2. **F-2 (major): existence masquerades as freshness.** `just check` reaches
   `validate` before `test` and `build`. A stale fork executable can pre-empt the
   coord rung; an absent one falls to a coord binary that cannot contain the
   current unlanded rule change. The asserted same-phase freshness has no
   lifecycle or enforceable phase constraint.
3. **F-3 (major): the coord build is promised but never established.**
   Dispatch setup creates and provisions the coord worktree but does not build
   it; neither the dispatch cadence nor SL-225's locked surface adds that beat.
   DEC-003 also says the softened governance note already “lives in” two files,
   while operative governance still mandates the rejected frozen
   `DOCTRINE_BIN` precondition and `CLAUDE.md` contains no replacement.
4. **F-4 (minor): undeclared Git floor.** The unconditional
   `--path-format=absolute` invocation assumes Git 2.31+, but no supported
   minimum is declared. Under `set -e`, an older generic host exits before both
   the advertised PATH tail and the tailored refusal.

**Sentence.** Replace the false common-directory inference with an actual,
run-time-reachable coord identity; own the coord fresh-create/resume/rebuild
lifecycle; make current-phase rule changes validate against a source-consistent
binary; narrow or remove the generic linked-worktree refusal; and either own a
Git version floor or use compatible plumbing. Then re-run VT-1/1b through the
real claude-arm gate and state the subprocess arm's distinct seam honestly.
Until these penances are authored, DEC-003 remains `proposed` and SL-225 remains
in design.

> **HERESIS URITOR; DOCTRINA MANET**
