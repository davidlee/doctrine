# SL-124 Design — Hook-stamp install reliability

<!-- Reference forms: entity ids padded (SPEC-009, ADR-001); doc-local refs bare
     (D1 decision, OQ-1 open question). -->

## Overview

The `dispatch-worker` `SubagentStart` stamp hook — wired by `doctrine claude
install` through the owner-locked merge core in `src/boot.rs` — silently fails to
fire in two distinct ways, each leaving an **unstamped worker** (downstream:
`verify-worker-refused: unstamped`, or fail-open writes on the no-env-leg/no-bwrap
harness). Source: ISS-011 (RV-016 F-13, folding ISS-034's hook-stamp half).

This design fixes the **writer** (install/merge), not the symptom. No
`verify-worker` self-stamp, no orchestrator hand-stamp change (slice § Non-Goals).

The merge core is the single implementation wiring every doctrine-owned Claude
hook (`mem.pattern.distribution.hookspec-merge-core-generalized-event-matcher`).
It is generic over `event + matcher`, but its ownership/heal logic keys on the
**command** alone, and the command string it writes is built from an exec path
that can carry a `(deleted)` token. Both gaps close here under a single invariant.

## Governing constraints

- **SPEC-009** (Install & distribution) — the `doctrine claude install` refresh
  surface this rides.
- **REQ-289** — the dispatch-worker identity stamp (SL-056); this slice defends
  its reliability.
- **ADR-001** (module layering) — no change; edits stay within `boot.rs`
  (+ `corpus.rs` call-site swap).
- **No parallel implementation** (CLAUDE.md; the merge-core memory) — one merge
  core, one invariant; no bolt-on prune path.
- **Behaviour-preservation gate** (AGENTS.md) — boot + memory-sync `SessionStart`
  hooks ride the same core; their suites stay **green unchanged**.
- **Pure/imperative split** — the strip/recognition/reconcile logic is pure;
  `current_exe()` and disk I/O stay in the thin shell.
- **Drift pin** — the stamp matcher is sourced from `DISPATCH_WORKER_AGENT_TYPE`,
  never re-spelled (`stamp_subagent_matcher_tracks_worktree_const` stays).

## Defects (current behavior)

### Defect A — stale matcher never healed

`plan_hook` → `find_owned` (`boot.rs:805-905`) returns `Owned::Current` (no write)
as soon as a doctrine-owned **command** matches and equals the desired command —
it never inspects the entry's **matcher**. A settings file carrying the stamp
command under a stale/wrong matcher (an old agent-type literal) is never healed on
reinstall → the matcher never matches the dispatch-worker → the stamp never fires.

### Defect B — `(deleted)` exec path + dead duplicates

`run_install` (`boot.rs:1479`) resolves the command path via
`std::env::current_exe()`, which on Linux reads `/proc/self/exe`; when the running
binary was rebuilt/replaced on disk the kernel appends a literal ` (deleted)` to
the readlink target. The install bakes that into the hook command
(`…/doctrine (deleted) worktree marker --stamp-subagent`), which fails to exec.

Compounding: `is_doctrine_stamp_command` strips the ` worktree marker
--stamp-subagent` suffix and checks the program file-name is `doctrine`. For a
poisoned command the residual file-name is `(deleted)` ≠ `doctrine`, so the
poisoned entry is **not recognised as ours** — never healed, never removed, and a
fresh entry is appended beside it. That is the observed three-stamp-hooks
accumulation. The same `current_exe()` poison also reaches the pi extension
(`generate_pi_extension`), the boot-snapshot `ExecPath` section, and `corpus.rs`
memory-sync — every bake site reads the same raw path.

## Target invariant

After any install, `hooks.<event>` holds **exactly one** doctrine-owned entry for
a given spec, and it is canonical: `command == spec.command` (clean, no `(deleted)`)
and `matcher == spec.matcher`. This single invariant subsumes all three failure
modes — stale matcher, poisoned command, duplicate/dead entries.

## Design decisions

- **D1 — root sanitize (OQ-2).** Fix the exec-path poison at the single source: a
  `resolve_exec()` wrapper over `current_exe()` strips a trailing ` (deleted)` via
  the pure `strip_deleted`. Every bake site uses it, repairing the hook, pi
  extension, snapshot, and corpus together. Rejected: sanitizing only the hook
  command — leaves identical latent bugs in the sibling bake paths.
- **D2 — unified reconcile, not layered heal+prune (OQ-1/OQ-3).** Make ownership
  recognition poison-tolerant and enforce the one-canonical-entry invariant inside
  `plan_hook`: collect all owned positions, heal the first to canonical (command +
  matcher), remove the rest. Rejected: keeping `find_owned` first-match plus a
  separate `(deleted)`-only prune pass — two concepts, a parallel path, and it
  won't collapse a clean duplicate.
- **D3 — prune predicate stays ownership-bounded.** Only **doctrine-owned**
  entries (poison-tolerant `is_ours`) are ever removed. A divergent operator entry
  never matches `is_ours`, so it is untouched. Among owned entries, two is never
  legitimate → collapse to one.
- **D4 — non-clobber on shared entries.** If a dropped owned hook shares an entry
  with foreign sibling hooks, remove only the hook, not the entry. Doctrine writes
  single-hook entries, so the common path removes whole entries.

## Code impact

### B-path — sanitize at the source

```rust
/// Strip a kernel-appended " (deleted)" suffix from a /proc/self/exe reading.
/// Pure; non-UTF-8 paths pass through untouched.
fn strip_deleted(p: &Path) -> PathBuf {
    match p.to_str() {
        Some(s) => PathBuf::from(s.strip_suffix(" (deleted)").unwrap_or(s)),
        None => p.to_path_buf(),
    }
}

/// The single exec resolver every bake site uses instead of current_exe().
fn resolve_exec() -> anyhow::Result<PathBuf> {
    let p = std::env::current_exe().context("Failed to resolve the doctrine executable path")?;
    Ok(strip_deleted(&p))
}
```

Replace the four raw `std::env::current_exe()` calls — `boot.rs:316`, `:333`,
`:1479`, `corpus.rs:482` — with `resolve_exec()`.

### A + B-prune — poison-tolerant ownership + reconcile

```rust
/// A program path is doctrine's iff — after dropping a trailing " (deleted)" —
/// its file name is `doctrine`. Shared by all three ownership predicates.
fn is_doctrine_program(program: &str) -> bool {
    let p = program.trim_end();
    let p = p.strip_suffix(" (deleted)").unwrap_or(p);
    Path::new(p.trim_end()).file_name() == Some(OsStr::new("doctrine"))
}
```

`is_doctrine_boot_command` / `_sync_command` / `_stamp_command` keep their
arg-shape split, then defer the program check to `is_doctrine_program` (one-line
edit each).

`find_owned` / `enum Owned` are replaced by a collector + reconcile in
`plan_hook`:

```rust
/// All doctrine-owned (entry_idx, hook_idx) positions for this spec, array order.
fn owned_positions(arr: &[Value], is_ours: fn(&str) -> bool) -> Vec<(usize, usize)>

// plan_hook reconcile branch (replaces the Current/Stale/Absent match):
let owned = owned_positions(arr, spec.is_ours);
match owned.split_first() {
    None => { arr.push(desired_entry(spec)); /* Wired */ }
    Some((&(ei, hi), rest)) => {
        if rest.is_empty() && entry_is_canonical(arr, ei, hi, spec) {
            return None;                      // idempotent no-write
        }
        remove_owned(arr, rest);             // drop duplicates/dead (D4 safety)
        set_canonical(arr, ei2, hi, spec);   // heal survivor: command + matcher
        /* Refreshed */
    }
}
```

- `entry_is_canonical(arr, ei, hi, spec)` = `arr[ei].matcher == spec.matcher &&
  arr[ei].hooks[hi].command == spec.command`.
- `set_canonical` = today's `set_command` plus setting the entry's `matcher` (heal
  for Defect A).
- `remove_owned`: drops non-survivor owned positions in **descending** index order
  (earlier indices stay valid; survivor index `ei2` adjusted for removals before
  it). Per D4: if a dropped owned hook shares an entry with foreign hooks, remove
  only that hook, not the entry.

**Blast radius:** `find_owned` + `enum Owned` removed/replaced; `plan_hook`
rewritten; `set_command` → `set_canonical`; three one-line predicate edits; add
`is_doctrine_program`, `owned_positions`, `entry_is_canonical`, `remove_owned`,
`strip_deleted`, `resolve_exec`. Boot/sync ride the new core unchanged in behavior
(single entry → canonical → `None`).

## Verification

All tests in the `src/boot.rs` test module (plus pure-helper tests). Existing
boot/sync hook matrix stays green **unmodified** (behaviour-preservation gate).

**Pure helpers**
- `strip_deleted`: `/x/doctrine (deleted)` → `/x/doctrine`; clean unchanged;
  no-suffix unchanged; `(deleted)` mid-string (not trailing) unchanged.
- `is_doctrine_program`: `…/doctrine` true; `…/doctrine (deleted)` true;
  `…/doctrine-helper` false; bare `(deleted)` false.

**Defect A — stale matcher heal**
- Clean stamp command under a wrong matcher → `Refreshed`, entry `matcher ==
  DISPATCH_WORKER_AGENT_TYPE`, command unchanged. Re-run → `None`.

**Defect B-prune — convergence**
- Three owned stamp entries (two poisoned, one clean) under assorted matchers →
  one canonical entry, two removed. Re-run → `None`.
- Single poisoned stamp entry → healed to clean command (`Refreshed`), matcher
  canonical; re-run → `None`.

**Safety / non-clobber (D3/D4)**
- Dropped owned hook sharing an entry with a foreign sibling → only the doctrine
  hook removed, foreign hook survives.
- Foreign `SubagentStart` hook under an unrelated matcher → untouched.

**Preservation**
- Existing boot/sync single-entry installs → canonical → `None` on re-run
  (current idempotency tests pass unmodified).
- `stamp_subagent_matcher_tracks_worktree_const` stays.

**Seam note:** `current_exe()` can't be faked in a unit test; `resolve_exec` is
covered transitively — `strip_deleted` carries the logic, the wrapper is thin.

## Open questions

_None remaining — OQ-1/2/3 resolved (D1/D2/D3)._
