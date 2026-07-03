# Funnel import scope belt: omit --slice for legitimately-coupled non-selector paths; dispose undeclared aligned at audit

`doctrine worktree import` has TWO scope postures. **With `--slice N`** it refuses the
delta (`undeclared-scope`) if any path matches no `design-target` selector. **Without
`--slice`** it does NO scope check — only the R-5 belt (`.doctrine/`/`.claude/` reject),
byte-for-byte the pre-scope belt. Both fail closed on precond/belt violation.

## The trap (SL-191 P02/P03, 2026-07-03)

A phase legitimately touches a path that is not a declared design-target selector:
- a **golden** in another test binary that a corpus change must reconcile
  (`tests/e2e_prompt_resolve_golden.rs` — P02),
- a **mandated caller migration** forced by a signature widening
  (`src/commands/prompt.rs` — P03, "fix ALL callers").

These are coupled deliverables, not drift. `import --slice` would refuse them, and the
"fix" I nearly ran — add the path as a selector at session root → promote edge→main →
`refresh-base` → re-fork — is a **phantom ceremony** built on a false premise (that the
scope check is an unavoidable import gate). It is not: the selector feeds `slice
conformance` at **audit**, not import.

## How to apply

1. **Import WITHOUT `--slice`** once you have **manually verified** the delta is exactly
   the intended files (`git -C <wt> status --porcelain`; R-5 clean; no untracked). The
   R-5 belt still guards authored state — you only drop the design-target scope check,
   which had nothing legitimate to catch.
2. **At audit**, the coupled non-selector paths report `undeclared` in `slice
   conformance`. Dispose them **`aligned`** ("deliverable-coupled: golden reconciliation
   / mandated caller migration, not drift"), exactly like a REV-only deliverable
   ([[mem.fact.conformance.rev-only-slice-undeclared]]) — that memory is the rev-only
   case; this is the code-slice generalisation.
3. Optionally add the path as a durable `design-target` selector (a session-root authored
   write, any time before audit) if you want `conformance` clean with no disposal — but
   it is cosmetic, never a precondition to landing.

Distinct from the R-5 belt (`.doctrine/`/`.claude/`) which is NON-negotiable and armed in
both postures. See also [[mem.pattern.dispatch.worker-prompt-run-full-suite]] (the P02
golden went red because the worker's suite scope was too narrow — the coupling surfaced
one funnel round late).
