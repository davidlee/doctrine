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

### B-path — sanitize at the source (validated, byte-safe, all bake sites)

`strip_deleted` works on **bytes** (not `to_str`), so a path that is non-UTF-8
*before* the kernel-appended ASCII suffix is still cleaned (codex M2). It returns
`Option` — `Some` only when the suffix was actually present — so `resolve_exec`
can tell a poisoned reading from a clean one and **validate against disk** rather
than silently baking a guessed path (codex M1):

```rust
/// Strip a kernel-appended b" (deleted)" suffix from a /proc/self/exe reading.
/// Pure, byte-level (UTF-8-agnostic). `Some` iff the suffix was present.
#[cfg(unix)]
fn strip_deleted(p: &Path) -> Option<PathBuf> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    p.as_os_str()
        .as_bytes()
        .strip_suffix(b" (deleted)")
        .map(|b| PathBuf::from(std::ffi::OsString::from_vec(b.to_vec())))
}
#[cfg(not(unix))]
fn strip_deleted(_p: &Path) -> Option<PathBuf> { None } // /proc/self/exe poison is Linux-only

/// Pure branch logic, injectable existence probe so all three arms are
/// unit-testable without a real `current_exe()` reading (codex minor). Prefer the
/// raw reading when it exists; on a `(deleted)` reading take the stripped path
/// **only if it exists**; otherwise fail loudly rather than bake a dead command.
fn pick_exec(raw: PathBuf, exists: impl Fn(&Path) -> bool) -> anyhow::Result<PathBuf> {
    if exists(&raw) {
        return Ok(raw);
    }
    if let Some(stripped) = strip_deleted(&raw)
        && exists(&stripped)
    {
        return Ok(stripped);
    }
    anyhow::bail!(
        "doctrine executable path {raw:?} does not resolve to an on-disk binary; \
         reinstall from a stable location"
    )
}

/// The single approved exec resolver — the thin shell over `pick_exec`.
pub(crate) fn resolve_exec() -> anyhow::Result<PathBuf> {
    let raw = std::env::current_exe()
        .context("Failed to resolve the doctrine executable path")?;
    pick_exec(raw, |p| p.exists())
}
```

`resolve_exec` is the **single approved resolver** and replaces **every**
`current_exe()` that feeds a persisted command, extension, MCP entry, or hook spec
— not just the four originally listed (codex C1). The full set:

| Site | What it bakes |
|---|---|
| `boot.rs:316`, `:333` | `boot` run / `--check` exec |
| `boot.rs:1479` (`run_install`) | the `wire`/hook commands |
| `corpus.rs:482` | `memory sync` exec |
| `skills.rs:1069` | the SubagentStart **stamp hook** (`doctrine claude install`) — a real stamp bake site the first draft missed |
| `install.rs:140` (`run_forward_steps`) | forward-step exec (hook/extension wiring) |
| `status.rs:337` | boot-staleness compare — read-only, but a poisoned exec falsely reports `stale`; route through `resolve_exec().unwrap_or_else(|_| PathBuf::from("doctrine"))` to keep its existing lenient fallback |

All except `status.rs` propagate the `bail!`; `status.rs` keeps its lenient
`unwrap_or_else` since a staleness read must never abort.

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
    arr.push(desired_entry(spec));            // Wired — append at tail
} else {
    // Insert the fresh canonical entry at the first owned hook's execution slot.
    // Entries before the first owned entry are all foreign and unremoved, so its
    // index `first` is stable. The +1 depends on whether that entry *survives the
    // drop* — i.e. retains a foreign hook — NOT on how many hooks it had (codex
    // round-4: two owned hooks in one entry has len>1 yet is fully removed):
    //   • first owned entry fully removed (all its hooks were owned) → insert at
    //     `first` (takes the vacated slot).
    //   • first owned entry survives (kept a foreign hook) → insert at `first + 1`
    //     — after the retained foreign remnant.
    let first = owned[0].0;
    let survives = entry_has_foreign_hook(arr, first, spec.is_ours); // ≥1 non-owned hook
    drop_owned_hooks(arr, &owned);            // extract every owned hook (F1/F2/m1)
    let ins = (first + usize::from(survives)).min(arr.len());
    arr.insert(ins, desired_entry(spec));     // one fresh canonical entry
    // Refreshed
}
```

`entry_has_foreign_hook(arr, ei, is_ours)` = the entry at `ei` has at least one
hook whose command is **not** `is_ours` — the precise "survives the drop"
predicate (distinct from `hook_is_sole`, which counts hooks for the no-write
short-circuit).

**Order-preservation bound (codex rounds 2–4).** Doctrine only ever writes
**single-hook** entries, so in every realistic file each owned hook is sole in its
entry and the execution slot is preserved **exactly** — a stale `boot` before
`sync` stays before `sync`. The only residual is a *hand-merged* entry that
interleaves a doctrine hook with foreign hooks in one `hooks` array: the doctrine
hook is extracted to its own entry placed immediately after the foreign remnant,
so a foreign hook the operator listed *before* the doctrine hook keeps its relative
order, but one listed *after* the doctrine hook in that same entry may end up
before it. Foreign **content, matcher, and all entry-level keys are always
preserved** — only sub-entry interleave order is not guaranteed, for an input
doctrine never produces and where independent SessionStart hooks carry no defined
ordering dependency. Splitting the entry to preserve exact interleave was rejected
as gold-plating on the shared merge core (raises preservation-gate risk for the
real boot/sync paths to no real-world benefit).

- `entry_is_canonical(arr, ei, hi, spec)` = `arr[ei].matcher == spec.matcher &&
  arr[ei].hooks[hi].command == spec.command`.
- `hook_is_sole(arr, ei)` = the entry's `hooks` array has length 1 (so the entry
  carries no foreign sibling — the canonical entry is doctrine-sole).
- `drop_owned_hooks`: for each owned `(ei, hi)`, remove that **hook** from its
  entry's `hooks` array; afterwards remove any entry whose `hooks` became empty.
  Implemented as a filter/rebuild — **the entry object is preserved in full**
  (clone it, replace only its `hooks` array with the retained non-owned hooks,
  keep every other entry-level key including `matcher` and any unknown keys), and
  the entry is dropped only when the retained `hooks` array is empty (codex m1).
  No descending-index surgery, no entry-level `matcher` rewrite (D4).

**Outcome mapping (RefreshOutcome):** `owned.is_empty()` → `Wired`; the no-write
short-circuit → `None`; every other path (stale command, stale matcher, poisoned,
duplicate collapse, foreign-sibling extraction) → `Refreshed`. The
`Wired`/`Refreshed`/`None`/`PrintedFallback` variants and their `"wired"` /
`"refreshed"` / `"already current"` labels (`install.rs:314`, `skills.rs:1085`)
are **unchanged**; only the doc comment on `Refreshed` broadens from "stale
command refreshed" to "an owned hook existed and was normalized to canonical"
(codex M3). Existing outcome assertions hold (verified: the stale-path test
asserts `Refreshed`, the first-install test `Wired`, the reinstall test `None`).

Net effect: at most one doctrine-owned entry survives, always canonical and
doctrine-sole, inserted at the first owned hook's execution slot (see the
order-preservation bound below). For every doctrine-written (single-hook) entry
order is preserved exactly — a stale `boot` before `sync` stays before `sync`. A
hand-merged-but-canonical owned hook is extracted into its own entry once, then the
no-write short-circuit makes re-runs `None`.

**Blast radius:** `find_owned` + `enum Owned` + `set_command` removed/replaced;
`plan_hook` rewritten as normalize; `RefreshOutcome::Refreshed` doc broadened;
three one-line predicate edits; add `is_doctrine_program`, `owned_positions`,
`entry_is_canonical`, `hook_is_sole`, `entry_has_foreign_hook`, `drop_owned_hooks`,
`strip_deleted`,
`pick_exec` + `pub(crate) resolve_exec`. Seven `current_exe()` call sites rerouted
through `resolve_exec` across `boot.rs`, `corpus.rs`, `skills.rs`, `install.rs`,
`status.rs` (table above). Boot/sync ride the new core unchanged (single canonical
doctrine-sole entry → no-write; refresh preserves entry position).

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
  extracted into a fresh entry inserted **after** the retained shared entry; the
  foreign hook **and its entry-level matcher/keys** survive unchanged. (Exercises
  F2 + m1.)
- Owned hook listed **after** a foreign hook in one shared entry → the foreign hook
  still executes **before** the doctrine entry. Asserts via the ordered
  `commands()` list.
- **Two owned hooks in one entry, followed by a separate foreign entry** (codex
  round-4 shape d) → the all-owned entry is removed, fresh entry inserts at `first`
  (not `first + 1`), the foreign entry keeps its position. Asserts ordered.
- Foreign `SubagentStart` hook under an unrelated matcher → untouched.
- A single canonical doctrine-sole entry → `None` (no spurious reorder/write).

**Preservation**
- Existing boot/sync single-entry installs → canonical → `None` on re-run
  (current idempotency tests pass unmodified). Verified order-safe:
  `plan_session_hook_refreshes_on_path_change_preserving_foreign` asserts via
  order-independent `commands()`, and `install_claude_hook_wires_boot_and_sync…`
  exact-checks only the no-op reinstall path — insert-at-execution-slot keeps both
  green without edits.
- `stamp_subagent_matcher_tracks_worktree_const` stays.
- **Shared-core proof (codex m2):** add the foreign-sibling shared-entry case for
  **boot and sync too**, not stamp only — proving the shared merge core did not
  acquire event-specific behaviour.
- **Order preservation (codex M-2):** a stale `boot` entry positioned **before** a
  `sync` entry → after refreshing boot, `boot` is still ordered before `sync`
  (insert-at-first-owned-index, not append-at-tail). Asserts via the ordered
  `commands()` list, not `.contains`.

**`pick_exec` branches (injectable probe — codex minor):**
- raw exists → returns raw (clean path, the normal case).
- raw absent, `…/doctrine (deleted)` → stripped exists → returns the stripped path
  (rebuild-in-place).
- neither exists → `bail!` (binary deleted/moved without replacement) — install
  fails loudly, never bakes a dead command.

**Seam note:** `current_exe()` itself can't be faked, but `pick_exec` takes the raw
reading + an `exists` closure, so all three arms above are unit-tested directly;
`resolve_exec` is the thin shell that supplies `current_exe()` + `Path::exists`.

## Open questions

_None remaining — OQ-1/2/3 resolved (D1/D2/D3)._

## Review log

- **Self (adversarial):** F1 fragile index surgery, F2 entry-level matcher clobber
  on heal-in-place → replaced heal-in-place with **normalize** (drop owned + append
  fresh). F3 `resolve_exec` `pub(crate)`. F4 same-entry/foreign-sibling test cases.
- **External (codex GPT-5.5):** C1 missed stamp bake sites (`skills.rs:1069`,
  `install.rs:140`, `status.rs:337`) → all seven `current_exe()` sites rerouted.
  M1 `strip_deleted` too trusting → `resolve_exec` validates against disk, bails
  loudly. M2 non-UTF-8 poison survived `to_str` → byte-level strip. M3 `Refreshed`
  semantics widened → doc broadened, variant/labels kept, existing assertions
  verified. m1 preserve all entry-level keys in `drop_owned_hooks`. m2 shared-entry
  tests for boot/sync, not stamp only. All integrated.
- **External (codex GPT-5.5, 2nd pass):** prior six confirmed resolved. New Major —
  append-at-tail reorders `SessionStart` boot/sync → insert at first-owned index
  (position-preserving) + order regression test. Minor — `resolve_exec` bail path
  untestable → factored `pick_exec` with injectable existence probe + branch tests.
  Minor — slice scope still said "heal in place" → reconciled to normalize wording.
- **External (codex GPT-5.5, 3rd pass):** round-2 fixes confirmed resolved except a
  residual Major — insert-at-`first` jumps the doctrine entry ahead of a foreign
  hook sharing the first owned entry. Fixed: insert at the execution slot, +1 when
  the entry survives the drop. Stale "append-at-tail" doc line corrected.
- **External (codex GPT-5.5, 4th pass):** found the survival predicate `!hook_is_sole`
  was wrong — two owned hooks in one entry has len>1 yet is fully removed, mis-placing
  the insert (Major). Fixed: survival = `entry_has_foreign_hook` (retains a non-owned
  hook), not hook count. Honest order-preservation **bound** added: exact for
  doctrine-written single-hook entries (all real files); sub-entry interleave order in
  a hand-merged multi-hook entry is not guaranteed (content/matcher/keys always
  preserved). Entry-splitting to chase exact interleave rejected as gold-plating.
