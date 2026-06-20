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
- **D2 — unified normalize, not layered heal+prune (OQ-1/OQ-3).** Make ownership
  recognition poison-tolerant and enforce the one-canonical-entry invariant inside
  `plan_hook` by **normalize**: drop every owned hook and append one fresh
  canonical entry (with a no-write short-circuit when a single canonical
  doctrine-sole entry already exists). Rejected: heal-in-place — `matcher` is
  entry-level, so rewriting it clobbers a foreign sibling hook (see D4). Also
  rejected: `find_owned` first-match + a separate `(deleted)`-only prune pass — two
  concepts, a parallel path, won't collapse a clean duplicate.
- **D3 — removal stays ownership-bounded.** Only **doctrine-owned** hooks
  (poison-tolerant `is_ours`) are ever removed. A divergent operator entry never
  matches `is_ours`, so it is untouched. Among owned hooks, more than one is never
  legitimate → collapse to one.
- **D4 — non-clobber on shared entries.** `drop_owned_hooks` removes the owned
  **hook** from a shared entry (not the whole entry) and never rewrites an
  entry-level `matcher` in place, so a foreign sibling hook and its matcher always
  survive. The canonical survivor is always a fresh doctrine-sole entry, so the
  shared-matcher hazard cannot arise.

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

`find_owned` / `enum Owned` are replaced by a collector + **normalize** in
`plan_hook`. Heal-in-place is rejected: `matcher` is an **entry-level** key shared
by every hook in `entry.hooks`, so rewriting it in place would clobber a foreign
sibling hook's matcher (the very hand-merged case D4 protects). Normalize instead —
drop every owned hook and append one fresh canonical entry — which never re-keys a
shared entry and needs no index surgery:

```rust
/// All doctrine-owned (entry_idx, hook_idx) positions for this spec, array order.
fn owned_positions(arr: &[Value], is_ours: fn(&str) -> bool) -> Vec<(usize, usize)>

// plan_hook normalize branch (replaces the Current/Stale/Absent match):
let owned = owned_positions(arr, spec.is_ours);
// No-write iff a single canonical doctrine-sole entry already exists.
if let [(ei, hi)] = owned[..]
    && entry_is_canonical(arr, ei, hi, spec)
    && hook_is_sole(arr, ei)
{
    return None;                              // idempotent no-write
}
if owned.is_empty() {
    arr.push(desired_entry(spec));            // Wired
} else {
    drop_owned_hooks(arr, &owned);            // extract every owned hook (F1/F2)
    arr.push(desired_entry(spec));            // one fresh canonical entry
    // Refreshed
}
```

- `entry_is_canonical(arr, ei, hi, spec)` = `arr[ei].matcher == spec.matcher &&
  arr[ei].hooks[hi].command == spec.command`.
- `hook_is_sole(arr, ei)` = the entry's `hooks` array has length 1 (so the entry
  carries no foreign sibling — the canonical entry is doctrine-sole).
- `drop_owned_hooks`: for each owned `(ei, hi)`, remove that **hook** from its
  entry's `hooks` array; afterwards remove any entry whose `hooks` became empty.
  Foreign hooks and their entry-level `matcher` are preserved (D4). Implemented as
  a filter/rebuild (collect owned positions into a set, rebuild each entry's
  `hooks` keeping non-owned hooks, drop emptied entries) — no descending-index
  surgery, no shared-matcher rewrite.

Net effect: at most one doctrine-owned entry survives, always canonical and
doctrine-sole, appended fresh. A messy file's entry moves to the array tail
(cosmetic; SubagentStart matching is order-independent); a hand-merged-but-canonical
owned hook is extracted into its own entry once, then `hook_is_sole` makes re-runs
no-write.

**Blast radius:** `find_owned` + `enum Owned` + `set_command` removed/replaced;
`plan_hook` rewritten as normalize; three one-line predicate edits; add
`is_doctrine_program`, `owned_positions`, `entry_is_canonical`, `hook_is_sole`,
`drop_owned_hooks`, `strip_deleted`, `pub(crate) resolve_exec` (reachable from
`corpus.rs`). Boot/sync ride the new core unchanged (single canonical doctrine-sole
entry → no-write).

## Verification

All tests in the `src/boot.rs` test module (plus pure-helper tests). Existing
boot/sync hook matrix stays green **unmodified** (behaviour-preservation gate).

**Pure helpers**
- `strip_deleted`: `/x/doctrine (deleted)` → `/x/doctrine`; clean unchanged;
  no-suffix unchanged; `(deleted)` mid-string (not trailing) unchanged.
- `is_doctrine_program`: `…/doctrine` true; `…/doctrine (deleted)` true;
  `…/doctrine-helper` false; bare `(deleted)` false.

**Defect A — stale matcher heal**
- Clean stamp command under a wrong matcher → `Refreshed`, the surviving entry's
  `matcher == DISPATCH_WORKER_AGENT_TYPE`, command preserved. Re-run → `None`.

**Defect B-prune — convergence**
- Three owned stamp entries (two poisoned, one clean) under assorted matchers →
  one canonical entry, the rest gone. Re-run → `None`.
- Single poisoned stamp entry → normalized to one clean canonical entry
  (`Refreshed`); re-run → `None`.
- **Two owned stamp hooks in one entry** (same `ei`) → collapse to one canonical
  doctrine-sole entry. Re-run → `None`.

**Safety / non-clobber (D3/D4)**
- Owned hook sharing an entry with a **foreign** sibling hook → the owned hook is
  extracted (removed) and a fresh canonical entry appended; the foreign hook **and
  its entry-level matcher** survive unchanged. (Directly exercises F2.)
- Foreign `SubagentStart` hook under an unrelated matcher → untouched.
- A single canonical doctrine-sole entry → `None` (no spurious reorder/write).

**Preservation**
- Existing boot/sync single-entry installs → canonical → `None` on re-run
  (current idempotency tests pass unmodified).
- `stamp_subagent_matcher_tracks_worktree_const` stays.

**Seam note:** `current_exe()` can't be faked in a unit test; `resolve_exec` is
covered transitively — `strip_deleted` carries the logic, the wrapper is thin.

## Open questions

_None remaining — OQ-1/2/3 resolved (D1/D2/D3)._
