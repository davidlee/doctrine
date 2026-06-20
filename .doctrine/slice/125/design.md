# SL-125 Design — stamp provision source from primary worktree

Governed by ADR-006 (orchestrator-sole-writer dispatch). Mechanism origin SL-056
(`marker --stamp-subagent`). Fixes ISS-011 **Defect C**, proven by the IMP-046
fresh-session probe (2026-06-20). Harness finding:
`mem.pattern.dispatch.subagentstart-hook-cwd-is-worker-worktree`.

## 1. Problem

`run_stamp_subagent` (`src/worktree.rs:2099`) resolves the provision **SOURCE**
via `root::find(path, …)` from the **process cwd** (`worktree.rs:2116`), on the
assumption that the `SubagentStart` hook fires inside the orchestrator tree.

Empirically false. The Claude harness runs the matcher-scoped hook with **process
cwd = the worker's own worktree** (`.claude/worktrees/agent-<id>`). `root::find`
checks `ancestor.join(marker).exists()` (`root.rs:31`); a linked worktree's root
carries its own `.git` *file* (and a checked-out `Cargo.toml`), so the first
ancestor that matches is the worker worktree itself. Hence `source == cwd == fork`.

`classify_stamp` passes (its `cwd_shares_repo(fork, fork)` and `is_linked` probes
are both trivially true), then `run_provision(Some(fork), fork)` reaches
`verify_sibling_worktree` (`worktree.rs:415`) → `source == fork` → bail
`fork path is the source tree itself; refusing to provision` → `run_provision`
aborts → **no marker**. The worker comes up unstamped; operators hand-stamp from
the orchestrator cwd (source ≠ fork) to unblock — the ISS-011 workaround.

## 2. Source decision — primary worktree

SOURCE = the repo's **primary worktree**, derived from the payload `cwd`,
independent of the process cwd.

Rationale (two independent reasons, both decisive):

1. **Only computable source.** The `SubagentStart` payload carries only the worker
   `cwd` (`{session_id, cwd, agent_id, agent_type, hook_event_name}`). The
   orchestrator / coordination-tree location is **not present** — the hook cannot
   address it. The single non-fork source the hook can deterministically derive is
   the primary worktree: `parent(resolve(--git-common-dir of cwd))`.
2. **Byte-correct.** `.worktreeinclude` lists exactly one path,
   `.doctrine/doctrine.just` — a gitignored (`.gitignore:22 .doctrine/*`),
   untracked, **static** include, identical across every worktree (primary,
   coordination, orchestrator-branch). Withheld tiers (`.doctrine/state/**`, the
   `phases` symlink, `handover.md`) are excluded by `select_copies` regardless of
   source. So even if the orchestrator sits on an orchestrator branch, the copied
   bytes do not differ.

The coordination/orchestrator tree was considered and rejected: it is invisible to
the hook (reason 1) and would gain nothing (reason 2).

## 3. Current vs target behavior

| | Current | Target |
|---|---|---|
| SOURCE resolution | `root::find` on **process cwd** → worker worktree | `primary_worktree(payload cwd)` → primary checkout |
| `source` vs `fork` | `source == fork` | `source ≠ fork` |
| provision | bails at `verify_sibling_worktree` | copies `.doctrine/doctrine.just`, then `write_marker` |
| marker | absent → unstamped worker | present at `<worker>/.doctrine/state/dispatch/worker` before first command |

## 4. Code impact (`src/worktree.rs` only)

### 4a. New helper

```rust
/// The repo's PRIMARY worktree root, derived from any worktree path `cwd`.
/// `git rev-parse --git-common-dir` resolves to `<primary>/.git`; its parent is
/// the primary checkout. Independent of the PROCESS cwd — the SubagentStart hook
/// fires inside the worker worktree, so the process cwd is the wrong source
/// (ISS-011 Defect C). Impure (git read).
fn primary_worktree(cwd: &Path) -> anyhow::Result<PathBuf> {
    let common =
        resolve_common_dir(cwd, &git::git_text(cwd, &["rev-parse", "--git-common-dir"])?)?;
    let primary = common
        .parent()
        .ok_or_else(|| anyhow::anyhow!("git-common-dir {} has no parent", common.display()))?;
    Ok(primary.to_path_buf()) // resolve_common_dir already canonicalized
}
```

`resolve_common_dir` (`worktree.rs:387`) already joins-and-canonicalizes, so
`common` is canonical `<primary>/.git` and `parent()` is the canonical primary
root — no second `canonicalize`.

### 4b. Shell change in `run_stamp_subagent`

Resolve `cwd_canon` first, then derive `source` from it. Keep `path` as an
explicit override (test seam / operator escape hatch); the hook passes `None`:

```rust
let cwd_canon = if cwd_present { fs::canonicalize(&cwd_str).ok() } else { None };

// SOURCE = primary worktree of the payload cwd's repo — NEVER the process cwd
// (Defect C: the hook fires in the worker worktree == fork). `--path` overrides.
let source = match (path, cwd_canon.as_deref()) {
    (Some(p), _) => fs::canonicalize(&p).ok(),
    (None, Some(cwd)) => primary_worktree(cwd).ok(),
    (None, None) => None,
};

let cwd_valid = match (source.as_deref(), cwd_canon.as_deref()) {
    (Some(src), Some(cwd)) => {
        is_linked_worktree(cwd).unwrap_or(false) && cwd_shares_repo(src, cwd)
    }
    _ => false,
};
```

Downstream is unchanged byte-for-byte: `already_marked`, `classify_stamp(...)`,
the `let (Some(source), Some(cwd)) = (source, cwd_canon) else { bad-dir }` bind,
`run_provision(Some(source), &cwd).and_then(|()| write_marker(&cwd))`, and the M3
`STAMP FAILED … worktree LEFT in place` no-rollback diagnostic.

The defect-site comment at `worktree.rs:2110-2115` (the false "hook fires inside
the orchestrator tree" claim) is rewritten to the truth.

## 5. Refusal / fail-closed paths — all preserved

| Token | Trigger | Preserved how |
|---|---|---|
| `missing-cwd` | empty payload `cwd` | `classify_stamp` order unchanged; `cwd_present` first |
| `bad-dir` | `cwd` not a linked worktree, OR `primary_worktree` git-fails (cwd not in a repo) ⇒ `source = None` ⇒ `cwd_valid = false` | unchanged token |
| `missing-agent-type` | `agent_type ≠ dispatch-worker` | unchanged |
| `already-marked` | re-entrant stamp (marker present) | unchanged; `marker_present(cwd_canon)` |
| M3 no-rollback | provision/mark fail | unchanged diagnostic + `Err` |

`is_linked_worktree(cwd)` still rejects a **primary**-tree cwd (gitdir ==
common-dir ⇒ not linked) → `bad-dir`. The change removes only the spurious
`source==fork` failure on a *valid* worker worktree; it widens no acceptance.

`cwd_shares_repo(source, cwd)` after the change compares the primary's common-dir
to the cwd's common-dir — equal by construction when both resolve, so it no longer
rejects a same-repo worker (correct), and still yields `false`/`bad-dir` when the
git reads fail (fail-closed).

## 6. Verification

- **VT-1** — fixture: a primary repo + a `git worktree add` linked worktree. Call
  `run_stamp_subagent` with payload `cwd` = the linked worktree (process cwd
  irrelevant). Assert: marker exists at `<linked>/.doctrine/state/dispatch/worker`,
  and `.doctrine/doctrine.just` copied. Proves source ≠ fork and provision ran.
- **VT-2** — `primary_worktree` unit: from a linked-worktree path it returns the
  primary root; from the primary it returns the primary (idempotent).
- **VT-3** — regression: the four refusal tokens (`missing-cwd`, `bad-dir` via a
  non-linked cwd, `missing-agent-type`, `already-marked`) stay green; pure
  `classify_stamp` tests untouched (signature unchanged).
- **VH-1** — re-run the IMP-046 fresh-session probe (Agent tool,
  `isolation: worktree`, matcher-scoped hook): worker stamped, no hand-stamp.
  Harness-dependent; not in-suite.
- **Gate** — `just check`, zero clippy warnings. Behaviour-preservation: the
  shared entity engine is untouched; existing worktree suites stay green.

## 7. Non-goals

- ISS-011 Defect A (matcher heal on reinstall) and Defect B (`(deleted)` path
  poison) — SL-124 territory.
- `SubagentStart` wiring / matcher / `/dispatch-agent` skill leg — proven sound.
- The marker-absent fail-closed privilege rule (ADR-006 D2a) — unchanged; this only
  makes the happy path stamp.
- `verify-worker` self-stamp-on-first-use — rejected in ISS-011 (fix the writer).
