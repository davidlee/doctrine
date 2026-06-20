# Review RV-109 — reconciliation of SL-124

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reviewed surface: **committed `main`** (solo-built, not dispatched) — PHASE-01
`00594ea7`, PHASE-02 `be7f45f8`. No candidate interaction branch.

Lines of attack — does the writer fix close both defects under the single
target invariant, without breaking the shared merge core?

1. **B-path conformance.** All seven `current_exe()` bake sites route through
   `resolve_exec`; `status.rs` keeps its lenient fallback; the bail path is
   unit-reachable via `pick_exec`'s injectable probe. → verify no raw
   `current_exe()` feeds a persisted command (grep), exec pure/shell split holds.
2. **A + B-prune conformance.** `find_owned`/`enum Owned`/`set_command` removed;
   `plan_hook` is the normalize; ownership poison-tolerant via shared
   `is_doctrine_program`. The invariant: one canonical doctrine-sole entry,
   matcher == `DISPATCH_WORKER_AGENT_TYPE`, command clean; idempotent re-run.
3. **Preservation gate (VA-1).** Existing boot/sync/stamp test bodies must be
   byte-unchanged — all deletions confined to removed production symbols.
4. **Drift probes.** Design code-sketch vs impl (`drop_owned_hooks` signature);
   merge-core memory still names the removed `find_owned`.

Out of scope (Non-Goals): no downstream symptom mask; ISS-034's base==B half;
the flaky relation/relation_graph tables (ISS-008, not a SL-124 defect).

## Synthesis

**Closure story.** SL-124 fixes the *writer* (install/merge), as scoped, and
holds the single target invariant: after install, `hooks.<event>` carries
exactly one doctrine-owned, canonical, doctrine-sole entry (clean command,
spec matcher). Both defects close under it:

- **B-path** — `strip_deleted` (byte-level, unix-cfg) + `pick_exec` (injectable
  probe: raw / stripped / bail) + `pub(crate) resolve_exec` land a disk-validated
  exec path. All seven `current_exe()` bake sites route through `resolve_exec`
  (grep confirms no raw reading feeds a persisted command; the remaining hits are
  comments/test/the resolver shell); `status.rs` keeps its lenient
  `unwrap_or_else` fallback so a staleness read never aborts. The pure/imperative
  split holds — strip/pick are pure, `current_exe()` + `Path::exists` stay in the
  shell.
- **A + B-prune** — `enum Owned`/`find_owned`/`set_command` removed; `plan_hook`
  is the normalize (no-write iff a single canonical sole entry exists → else drop
  every owned hook + insert one fresh canonical entry at the first owned hook's
  execution slot, `first + survives`). Ownership is poison-tolerant via the shared
  `is_doctrine_program`. Survival is `entry_has_foreign_hook`, not `!hook_is_sole`
  (the codex round-4 trap — VT-4 guards it). `drop_owned_hooks` clone-rebuilds only
  the `hooks` array, so foreign matcher + unknown keys always survive (D4/m1).

**Evidence.** `just check` green — clippy clean, full suite incl.
`e2e_worktree_verify_worker`. VT-1..8 present and green (VT-5/VT-6 folded into the
foreign-sibling ordered test; VT-8 idempotency asserted as the re-run `→ None` in
each shape; VT-7 proves the shared core stays event-agnostic for boot+sync).
Pure-helper tests cover `strip_deleted`/`pick_exec`/`is_doctrine_program`.
Order-preservation regression (`boot_refresh_keeps_position_before_sync`) guards
the codex M-2 reorder.

**Preservation gate (VA-1).** Confirmed: every PHASE-02 deletion is a removed
production symbol; no existing test body changed. The shared boot/sync/stamp
suites are the proof and stay green unmodified.

**Standing risks / consciously accepted tradeoffs.** The order-preservation bound
(design L228–241) is a deliberate scope line: exact for doctrine-written
single-hook entries (every real file); sub-entry interleave order in a
hand-merged multi-hook entry is *not* guaranteed, though content/matcher/keys
always are. Entry-splitting to chase exact interleave was rejected as gold-plating
on the shared core — accepted, no finding.

Two non-blocking drift findings (F-1 design code-sketch, F-2 merge-core memory),
both `verified` and routed to `/reconcile`; one `aligned` (F-3 preservation
confirmation). No blockers. Ledger done, await=none.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §Code-impact (L191 / L216 / L248) — F-1:** the `drop_owned_hooks`
  code-sketch shows `drop_owned_hooks(arr, &owned)` (positions slice). The
  implementation is `drop_owned_hooks(arr: &mut Vec<Value>, is_ours: fn(&str) ->
  bool)` (re-filters via `retain_mut`). Update the sketch + the `plan_hook`
  normalize snippet (L218 `drop_owned_hooks(arr, &owned)` → `drop_owned_hooks(arr,
  spec.is_ours)`) and the L248 prose to match the shipped signature. No code
  change — behaviourally equivalent and cleaner.

### Governance/spec (memory — knowledge surface, no REV)
- **memory `mem.pattern.distribution.hookspec-merge-core-generalized-event-matcher`
  — F-2:** body names `find_owned` first-match as the ownership mechanism. Update
  to the normalize: `owned_positions` collect → no-write short-circuit on a single
  canonical sole entry → else drop every owned hook + insert one canonical entry.
  Drive via `/record-memory` (update existing), not a REV — this is a knowledge
  artefact, not governance/spec.

_No ADR / REQ / SPEC change: the slice rode SPEC-009 / REQ-289 / ADR-001 without
altering them. No REV required._

## Reconciliation Outcome

### Direct edits applied
- **design.md §Code-impact (L216 + L247–253) — RV-109 F-1:** the `drop_owned_hooks`
  code-sketch call and its prose now match the shipped signature
  `drop_owned_hooks(arr, is_ours: fn(&str)->bool)` (re-filter via `retain_mut`,
  not a consumed positions slice). Behaviourally identical; doc-only sync.
- **memory `mem.pattern.distribution.hookspec-merge-core-generalized-event-matcher`
  (memory.md) — RV-109 F-2:** body updated to describe the SL-124 `normalize`
  mechanism (`owned_positions` + no-write short-circuit + `drop_owned_hooks` +
  insert-one-canonical) and poison-tolerant `is_doctrine_program`, replacing the
  stale `find_owned` first-match reference. Body-guard recomputed clean. Treated as
  a knowledge artefact (direct edit), not governance/spec → no REV.

### REVs completed
- None. No ADR / REQ / SPEC / policy change — SL-124 rode SPEC-009 / REQ-289 /
  ADR-001 unaltered.

### Withdrawn / tolerated
- None. F-3 was `aligned` (no remediation); F-1 / F-2 applied above.

Reconcile pass complete — handoff to /close.
