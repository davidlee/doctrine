# IMP-197: Dispatch worker prompt template — negative contract + home + hermetic + path-anchor

**Source:** SL-168 postmortem §3,§5a,§5c,§5d.1. **Home:** RFC-005.

Workers (deepseek-class) fill underspecified prompts with reasonable-but-wrong
choices, underspecified on two axes — *where* and *what-not-to-do*:
- wrong module home → ADR-001 layering violation (F-1)
- live-corpus golden (F-2); substring (not component-anchored) path matching (F-3)
- unasked `cargo fmt`, `.doctrine/` touches, out-of-set edits, stray git verbs.

**Fix direction:** `dispatch arm-spawn` injects a mandatory template:
- NEGATIVE CONTRACT block (no fmt; no `.doctrine/`/`.claude/`; no out-of-set edits;
  no test edits not authored; only git verb is the final commit)
- explicit home-module + layer rationale for every new fn
- hermetic-fixture directive for goldens
- component-anchored path patterns (`.dispatch/`, `.worktrees/`, `target/`).

Related: RFC-005; governed_by ADR-011 (spawn interface).
