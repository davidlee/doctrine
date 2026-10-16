# Implementation Plan SL-281: Install artefact ownership

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Reference forms: `lib:reference/glossary.md` § reference forms. -->

## Overview

Six phases, ordered so each lands green and the destructive power arrives only
after the pure decision core and the staging safety it depends on are proven.
`design.md` is the reference for every shape named here; the phases cite its
sections rather than restating them.

## Sequencing & Rationale

1. **PHASE-01 — registry first.** The `[framework]` roots and `[harness.*]`
   registry are the vocabulary every later phase reads. Landing the parse and
   the detect / boot / delegate rewiring alone is small, separately observable
   (pi delegates `universal` once; `.pi/` alone wires the codex boot adapter)
   and leaves the agent-def legs untouched, so the cutover phase is not also a
   vocabulary change.
2. **PHASE-02 — pure plan.** `install_ownership::plan` carries nearly all the
   decision logic and most of the verification weight. Built and tested with no
   wiring, it can be reviewed as a function before anything calls it.
3. **PHASE-03 — exclusive staging.** The F-1 / F-10 safety rules (RV-418) on
   `write_link` and `materialise_canonical` are independent of convergence and
   change no observable behaviour on a clean tree. Proving them before pruning
   exists keeps the cutover's blast radius to "which actions", not "how a write
   is staged".
4. **PHASE-04 — cutover.** The one phase with destructive effect: the executor
   replaces four legs, the claude canon moves to `.doctrine/agents/claude/`, the
   constants and `_ => pi` arms go, and the real-binary e2e suite lands with it.
   It is the largest phase; it is not split further because the legs, the
   layout move and the e2e assertions are only coherent together (a half-cutover
   would run two link reconcilers over the same dirs).
5. **PHASE-05 — gitignore block.** Depends on PHASE-04's post-run re-scan for
   its link lines. Kept separate because it changes a different file contract
   (`.gitignore`) and removes `Step::Gitignore` from the base plan.
6. **PHASE-06 — dogfood.** Repo-local cleanup that needs the finished binary:
   the block written into this repo's `.gitignore`, the relic removed, the
   capsule defs linked by install rather than hand-copied.

## Revision

DEC-376: one REV over SPEC-009, REQ-170 and REQ-176, rows per design
§Revision, drafted at plan time against the current spec text and applied at
`/reconcile`, not during a phase. No phase edits a spec.

## Risks carried into execution

- **PHASE-04 size.** Mitigated by PHASE-02/03 carrying the logic and the safety
  primitives; if the phase sheet still exceeds one coherent unit, split at
  phase-plan time (append `PHASE-07`, never renumber).
- **`--dry-run` scope.** `install::run` returns after `print_forward_summary`
  under `--dry-run`, before any forward step (verified). PHASE-04 adds the scan
  and plan print to that path; the executor stays unreached — the design's
  scan / plan / execute split already allows it.
- **Global e2e.** `install_base` reads `$HOME` (verified), so a temp `$HOME`
  is sufficient.
- **Unregistered harness defs.** Until PHASE-04 deletes the `_ => pi` arm, an
  unregistered `--agent` still receives pi's def; PHASE-01 only adds the
  disclosure. Interim, not shipped (one slice).
- **`plan_excludes_marketplace_only_skills_from_members`** sits in
  `src/install.rs`, not `install_ownership.rs` as the design lists it:
  membership is built in the shell from `discover()`, so that is where the
  exclusion is testable.
- **Convergence e2e** gets its own file, `tests/e2e_install_ownership.rs`;
  `e2e_claude_install.rs` keeps the def-set assertions only.
- **Embedded `agents/AGENTS.md`.** A publication asset under the agents embed
  prefix; the desired-set builder must filter to `agents/<harness>/` or it
  becomes a phantom member (PHASE-04 EX-1).

## Notes
