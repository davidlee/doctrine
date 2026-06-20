# Hook-stamp install reliability: heal stale SubagentStart matcher, sanitize (deleted) exec path, prune dead stamp hooks

## Context

Source: ISS-011 (RV-016 F-13, widened 2026-06-20 to fold ISS-034's hook-stamp
half). The `dispatch-worker` `SubagentStart` stamp hook — wired by `doctrine
claude install` through the owner-locked merge core in `src/boot.rs` — silently
fails to fire in two distinct ways, each leaving an **unstamped worker**. The
downstream symptom is `verify-worker-refused: unstamped` (best case, on the
bwrap/env-leg harness) or **fail-open writes by an unauthenticated worker** on the
one harness with no env leg and no bwrap (worst case). Both observed dogfooding
`/dispatch` (claude arm) on SL-121.

The merge core is the single implementation wiring every doctrine-owned Claude
hook (`mem.pattern.distribution.hookspec-merge-core-generalized-event-matcher`);
it is generalized over `event + matcher` but its **ownership/heal logic still keys
on the command alone**, and the command string it writes is built from an exec
path that can carry a `(deleted)` token. This slice closes both gaps in the
writer — masking the symptom downstream (e.g. a `verify-worker` self-stamp) is
explicitly out of scope.

## Scope & Objectives

Two defects, one writer, fixed together (ISS-011 § Fix):

- **Defect A — stale matcher never healed.** `find_owned` / `plan_hook`
  (`src/boot.rs:805-905`) return `Owned::Current` (no write) as soon as a
  doctrine-owned **command** matches, never inspecting the entry's **matcher**. A
  `.claude/settings.local.json` that already carries the stamp hook under a
  stale/wrong matcher (e.g. an old agent-type literal) is never healed on
  reinstall → the matcher never matches the dispatch-worker → the stamp never
  fires.
  - **Objective:** the merge owns and heals identity on `(event, matcher,
    command)`. A reinstall reconciles a stale matcher to the spec's matcher
    (`DISPATCH_WORKER_AGENT_TYPE`) in place, idempotently.

- **Defect B — `(deleted)` exec path + dead duplicates.** `run_install`
  (`src/boot.rs:1479`) resolves the command path via
  `std::env::current_exe()`, which on Linux reads `/proc/self/exe`; when the
  running binary was rebuilt/replaced on disk the kernel appends a literal
  ` (deleted)` to the readlink target. The install bakes that into the hook
  command (`<path> (deleted) worktree marker …`), which fails to exec. The merge
  does not recognise such a dead entry as its own / does not prune it, so dead
  duplicates accumulate.
  - **Objective:** install resolves a stable on-disk binary path — never a
    `(deleted)`-bearing reading — for every wired hook command; and the merge
    prunes provably-dead doctrine-owned stamp duplicates (a `(deleted)` command
    is provably dead) on reinstall, converging on the single live entry.

Closure intent — judged green when:
- A settings file with the stamp command under a stale matcher is healed in place
  on reinstall (matcher == `DISPATCH_WORKER_AGENT_TYPE`), idempotent on re-run.
- A `current_exe()` reading bearing ` (deleted)` yields a sanitized command path
  (no `(deleted)` token) in the written hook.
- A settings file carrying duplicate / `(deleted)` doctrine stamp hooks converges
  to one live entry after reinstall.
- Existing boot/install suites stay green unchanged (behaviour-preservation gate
  on the shared merge core).
- New regression tests cover A (stale matcher), B-path (sanitize), B-prune
  (duplicate/dead collapse).

## Affected Surface

- `src/boot.rs` — the merge core: `find_owned` / `Owned` / `plan_hook` /
  `set_command` → normalize (Defect A + B-prune); poison-tolerant
  `is_doctrine_program`; `strip_deleted` + `pub(crate) resolve_exec` (Defect
  B-path).
- Exec-path resolution rerouted through `resolve_exec` at **all seven**
  `current_exe()` bake sites (design review C1): `boot.rs` (×3), `corpus.rs`,
  `skills.rs` (the stamp-hook install), `install.rs` (forward steps), `status.rs`
  (read-only staleness, lenient fallback retained).
- Tests live in the `src/boot.rs` test module (existing hook-merge matrix; add
  heal / sanitize / prune-converge / shared-entry cases).
- No change to `worktree marker --stamp-subagent`, to the matcher const, or to the
  `verify-worker` refusal — those are correct; the writer is at fault.

## Non-Goals

- No downstream symptom mask: no `verify-worker` self-stamp on first use, no
  orchestrator hand-stamp workflow change. Fix the writer.
- ISS-034's **other** half (claude dispatch arm isolation / `baseRef:"head"`
  base==B defect) stays in ISS-034 — only the hook-stamp half is folded here.
- No widening of the hook-merge ownership vocabulary beyond what A/B require; no
  new hook events.
- No change to the pure/imperative split or module layering (ADR-001).

## Open Questions

- **OQ-1** Defect A heal granularity: extend the ownership *key* to `(matcher,
  command)` so a wrong-matcher entry reads as `Stale` and `set_command` grows a
  sibling matcher reconcile — or reconcile the matcher as a separate heal pass?
  (key the merge identity vs. a targeted matcher reconcile.) → `/design`.
- **OQ-2** Defect B-path: sanitize the ` (deleted)` suffix off the
  `current_exe()` reading, or resolve a stable path by a different means (argv0 /
  install-target lookup)? Sanitize is minimal; confirm it cannot mask a genuinely
  relocated binary. → `/design`.
- **OQ-3** Defect B-prune scope: prune only `(deleted)`-bearing doctrine stamp
  duplicates, or any duplicate doctrine-owned entry under the same event? Keep the
  prune predicate provably-dead-only to avoid clobbering a legitimately divergent
  operator entry. → `/design`.

## Summary

_(filled at close)_

## Follow-Ups

_(filled at close)_
