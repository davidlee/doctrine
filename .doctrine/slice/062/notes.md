# SL-062 — implementation notes

Durable cross-phase findings from the dispatch run. Disposable phase sheets live
under `.doctrine/state/`; this file is the authored durable record.

## Dispatch mechanism observation (PHASE-01)

The `/dispatch` claude arm spawned a `dispatch-worker` via the `Agent` tool with
`isolation: worktree`. Observed: the worker's single commit `S` **integrated
directly onto `main`** on completion — no registered worktree remained
(`.git/worktrees/dispatch/HEAD` absent), and `main@{0}` in the reflog is the
worker's commit. The orchestrator did NOT run a separate `import`/one-commit step;
the delta was already on the coordination branch.

Outcome was nonetheless sound — the funnel's *goals* held even though its
sole-writer *mechanism* was bypassed:
- delta = exactly the 3 declared source files (`src/lifecycle.rs`, `src/slice.rs`,
  `src/main.rs`); **no foreign untracked sweep** (review/020, slice/063, memory
  items all still untracked, untouched);
- **R-5 clean** (no `.doctrine/`/`.claude/` touch);
- linear on HEAD (B=32dae47 → 3 foreign commits → S=7e4e071), no divergence;
- combined tree verified GREEN by the orchestrator (`just gate`, clippy `-D
  warnings` clean) AFTER landing — not trusting the worker's self-report.

Consequence for PHASE-02/03: the R-5 belt + verify run **post-landing**, not
pre-commit. Mitigation in the worker brief: stage ONLY declared files, never `git
add -A` (foreign untracked files sit in the shared tree); orchestrator verifies
each delta post-landing and would have to revert on a violation. Single
observation — confirm on PHASE-02 before recording as durable doctrine memory.

## PHASE-01 — re-home pure FSM into src/lifecycle.rs (DONE, verified green)

- New pure leaf `src/lifecycle.rs` (beside `conduct.rs`, ADR-009 axis-A/axis-B
  pairing): `enum Transition`, `fn classify`, `fn is_transition_terminal`,
  `fn crosses_closure_seam` + edge table. Pure `&str`-edge data, imports no kind
  module (ADR-001 no-cycle holds).
- STAYED in `slice.rs`: `transition_label` (P4), `is_terminal_status` (P3, distinct
  from `is_transition_terminal`), `SLICE_STATUSES`/`SliceStatus`/drift canary,
  `is_divergent`/`is_hidden`/`is_drifted`, `run_status`/`set_slice_status` (retarget
  imports to `lifecycle::*`).
- OQ-1 resolved: classify edge-case tests MOVED to `lifecycle.rs` (smaller, cohesive
  diff); the distinct-predicate canary stays in `slice.rs` importing
  `lifecycle::is_transition_terminal`.
- Behaviour-preservation gate held: slice FSM suite assertion text unchanged, only
  import paths shifted (F-E). Commit `7e4e071`.

## PHASE-02 — one authored-TOML mutation seam (DONE, verified green)

Commit `1ea07b3` (parent == B, linear, R-5 clean, `just gate` green verified by
orchestrator). OQ-3 resolved: grew `src/dep_seq.rs` into the authored-TOML mutation
leaf (no new module) — it already hosts the `append` core + the non-destructive F-1
idiom.

- Pure cores on `&mut DocumentMut`: `apply_status(doc, managed, hint)->bool`,
  `apply_string_append(doc, field, value)->bool`. IO wrappers `set_authored_status`
  / `append_string_array` (read→parse→core→write-once).
- DRY: extracted `push_str_if_absent(&mut Array, &str)->bool` — ONE string-membership
  body, called by both `dep_seq::append`'s `Needs` arm and `apply_string_append`. The
  `After {to,rank}` struct path is byte-untouched (R3; SL-060 needs/after suites green).
- Four setters retired onto `set_authored_status`, each keeping its gate in the shell
  (slice classify+RV; backlog validate_transition coupling + D9 res-clear, still
  returns resolution `&str`; gov flat; requirement flat status-only no `updated`).
- EX-4: gov + requirement F-1 hints reworded non-destructive; slice/backlog preserved.

**Load-bearing subtlety — no-op excludes `updated`.** The unified `apply_status`
no-op guard compares all managed keys EXCEPT `updated` (`.filter(|(k,_)| *k !=
"updated")`). The four donors keyed their no-op on `status` (gov/slice/req) or
`status`+`resolution` (backlog), NEVER on `updated` (a derived stamp). Comparing
`updated` would spuriously write on every status-unchanged-but-today-differs call.
Behaviour-preserving by construction; two no-op tests pin it.

**For PHASE-03:** `apply_string_append`/`append_string_array` are gated
`#[cfg_attr(not(test), expect(dead_code, reason=...))]` (staged for the supersede
consumer). PHASE-03 wires `apply_string_append` → it MUST DROP that `expect`, else
the now-fulfilled lint makes the `expect` unfulfilled = compile error.

**Follow-up captured:** IMP-061 — a fifth byte-identical setter
`knowledge::set_record_status` (`src/knowledge.rs:1283`) is out of SL-062 scope; fold
it onto `set_authored_status` to complete the DRY collapse.

## PHASE-03 — transactional ADR-first supersede verb (DONE, verified green)

Commit `a6ed379` (parent == B, linear, R-5 clean, `just gate` green verified by
orchestrator). Top-level `doctrine supersede <NEW> <OLD>`, sibling of link/needs/after.

- `SupersedePolicy` + `supersede_policy(kind)->Option` live in `src/adr.rs` (the kind
  that owns the `supersedes`/`superseded_by`/`superseded` vocab); hardcoded ADR-only
  match (D4 — not GovKind data). POL/STD/slice → None → ADR-first refuse (F2 follow-up).
- `run_supersede` in main.rs: parse-once/hold-both/write-once (§5.4). Composes the
  PHASE-02 cores (`apply_string_append` ×2, `apply_status` ×1) over docs parsed once;
  writes NEW then OLD (ordering makes a torn state detectable, not the verb's job).
  No third write body; `append_string_array` wrapper unused (its dead_code expect kept).
- Guards: pre-flight F-1 (non-destructive) before any write; F-D not-already-superseded
  (both-files no-op require `OLD.superseded_by==[NEW]` AND `NEW.supersedes∋OLD`);
  different-supersessor `<X>` refuse; drift refuse → `doctrine validate` (P5, no self-heal);
  self-edge / cross-kind refuse.
- Refs stored prefixed (`ADR-001`) via `parse_canonical_ref`→`listing::canonical_id`,
  matching `validate`'s derived side. VT-6 partial-write detected by
  `relation_graph::validate_relations` ("supersession drift"), NOT the verb (F-F/codex C5).
- Dropped the `#[cfg_attr(not(test),expect(dead_code))]` on `apply_string_append` (now
  consumed). NEW black-box suite `tests/e2e_supersede.rs` (7 tests, all through run()).
- Unblocks SL-048 OD-3 (F3). F1/F2/F3 minted at CLOSE.

## Dispatch isolation — confirmed across all three phases

All 3 workers ran in the shared `main` working tree (each saw foreign untracked files)
and integrated their single commit directly onto `main` — `isolation: worktree` did not
hold the orchestrator-sole-writer split. Every delta was nonetheless clean (exact
declared files, R-5 clean, parent==B linear) and orchestrator-verified green
post-landing. SL-064 (foreign, in-flight) is scoped to fix this coordination-branch
isolation gap. The funnel's GOALS held; its sole-writer MECHANISM was bypassed.
