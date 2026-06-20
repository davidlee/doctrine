# SL-125 Design — stamp provision source from primary worktree

Governed by ADR-006 (orchestrator-sole-writer dispatch). Mechanism origin SL-056
(`marker --stamp-subagent`). Fixes ISS-011 **Defect C**, proven by the IMP-046
fresh-session probe (2026-06-20). Harness finding:
`mem.pattern.dispatch.subagentstart-hook-cwd-is-worker-worktree`.
Reviewed: internal adversarial pass + codex (GPT-5.5) external pass — findings
folded below.

## 1. Problem

`run_stamp_subagent` (`src/worktree.rs:2099`) resolves a single `repo` value via
`root::find(path, …)` from the **process cwd** (`worktree.rs:2116`) and uses it for
**two distinct roles**:

- **(R1) repo-binding / validation** — `cwd_shares_repo(repo, cwd)` confirms the
  payload `cwd` belongs to the same repo as the anchor, and `is_linked_worktree`
  confirms it is a linked worktree.
- **(R2) provision SOURCE** — `run_provision(Some(repo), &cwd)` copies the
  allowlisted gitignored files FROM `repo` INTO the worker `cwd`.

The defect is in **R2 only**. The Claude harness runs the matcher-scoped hook with
**process cwd = the worker's own worktree** (`.claude/worktrees/agent-<id>`).
`root::find` matches on `ancestor.join(marker).exists()` (`root.rs:31`); a linked
worktree's root carries its own `.git` *file* and a checked-out `Cargo.toml`, so the
first matching ancestor is the worker itself. Hence `repo == cwd`, and as the
SOURCE that means `source == fork`: `run_provision` → `verify_sibling_worktree`
(`worktree.rs:415`) bails `fork path is the source tree itself; refusing to
provision` → no marker. Operators hand-stamp from the orchestrator cwd (where
`repo ≠ fork`) to unblock — the ISS-011 workaround.

R1 is **not** broken: in the hook case `repo == cwd` makes the binding check
trivially true, exactly as it already is today (the anchor and payload coincide).
We must not change R1, or we lose the cross-repo rejection it provides on manual
invocations.

## 2. Fix — split the two roles; change only the SOURCE (R2)

Leave R1 (the binding anchor `repo = root::find(...)` and the `cwd_valid` check)
**byte-for-byte unchanged**. For R2, derive the provision SOURCE as the repo's
**primary worktree**, computed from the validated worker `cwd`:

```
source = first `worktree <path>` line of `git -C <cwd> worktree list --porcelain`
```

The main worktree is always git's first porcelain entry. This is the standard
primary-worktree locator and is correct across separate-git-dir and submodule
layouts (where `parent(--git-common-dir)` is not).

Why **primary** (not orchestrator/coordination tree):

1. **Unaddressable otherwise.** The `SubagentStart` payload carries only
   `{session_id, cwd, agent_id, agent_type, hook_event_name}` — no orchestrator
   location. The hook *cannot* name the orchestrator tree. The primary worktree is
   the one non-fork source derivable from the payload alone.
2. **Sufficient for the current allowlist.** `.worktreeinclude` lists exactly
   `.doctrine/doctrine.just` — a static install artifact (NOT written by any
   doctrine code; `grep` of `src/` is empty), worktree-invariant in practice. The
   withheld tier (`.doctrine/state/**`, `phases`, `handover.md`) is excluded by
   `select_copies` regardless of source. So primary vs orchestrator yields the same
   copied bytes.

Limitation (honest): if `.worktreeinclude` ever grows to include genuinely
per-worktree-divergent untracked state that a worker must inherit *from the
orchestrator*, this hook mechanism cannot supply it from primary — and could not
name the orchestrator tree either. That is a separate design (orchestrator-push or
a payload side-channel), tracked as a follow-up, not in scope here.

## 3. Current vs target behavior

| | Current | Target |
|---|---|---|
| R1 binding (`cwd_shares_repo`, `is_linked_worktree`) | from `repo=root::find(process cwd)` | **unchanged** |
| R2 provision SOURCE | `repo` (== fork in the hook case) | `primary_worktree(cwd)` |
| `source` vs `fork` | `source == fork` → bail | `source ≠ fork` |
| provision | bails at `verify_sibling_worktree` | copies `.doctrine/doctrine.just`, then `write_marker` |
| marker | absent → unstamped worker | present at `<worker>/.doctrine/state/dispatch/worker` before first command |

## 4. Code impact (`src/worktree.rs` only)

### 4a. New helper

```rust
/// The repo's PRIMARY (main) worktree root, as git reports it: the FIRST
/// `worktree <path>` entry of `git worktree list --porcelain`, run against any
/// path in the repo. Correct across ordinary, separate-git-dir, and submodule
/// layouts (unlike `parent(--git-common-dir)`). Used as the provision SOURCE so it
/// is independent of the process cwd — the SubagentStart hook fires inside the
/// worker worktree, which must never be the source (ISS-011 Defect C). Impure (git
/// read). Bare repos (no main worktree) are out of scope for dispatch.
fn primary_worktree(cwd: &Path) -> anyhow::Result<PathBuf> {
    let listing = git::git_text(cwd, &["worktree", "list", "--porcelain"])?;
    let first = listing
        .lines()
        .find_map(|l| l.strip_prefix("worktree "))
        .ok_or_else(|| anyhow::anyhow!("no main worktree for {}", cwd.display()))?;
    fs::canonicalize(first).with_context(|| format!("canonicalize primary worktree {first}"))
}
```

### 4b. Shell change in `run_stamp_subagent` — minimal, surgical

Everything up to and including `classify_stamp(...)` and the
`let (Some(source), Some(cwd)) = (repo, cwd_canon) else { … bad-dir }` bind stays
as today, except the bind's first slot is the (now binding-only) anchor. Only the
provision call changes its SOURCE:

```rust
// R1 binding anchor unchanged: `repo`/`cwd_valid`/`classify_stamp` as before.
let (Some(_anchor), Some(cwd)) = (repo, cwd_canon) else {
    let token = StampRefusal::BadDir.token();
    writeln!(io::stderr(), "stamp-refused: {token}")?;
    bail!("stamp-refused: {token}");
};

// R2: provision SOURCE is the PRIMARY worktree, NOT the binding anchor — which is
// the fork itself when the hook fires inside the worker worktree (Defect C).
let act = primary_worktree(&cwd)
    .and_then(|source| run_provision(Some(source), &cwd))
    .and_then(|()| write_marker(&cwd));
if let Err(cause) = act {
    writeln!(io::stderr(),
        "STAMP FAILED for {} — worktree LEFT in place (not removed); orchestrator post-spawn check will catch the unstamped worker: {cause:#}",
        cwd.display())?;
    return Err(cause.context(format!("stamp worker worktree {}", cwd.display())));
}
```

No reorder of the gather block; no new `--path` semantics. `--path` retains its
existing meaning (it feeds `root::find` for the binding anchor, exactly as today) —
it does **not** become a source override, so it introduces no new `source==fork`
footgun. The defect-site comment at `worktree.rs:2110-2115` (the false "hook fires
inside the orchestrator tree" claim) is corrected to describe R1 vs R2.

## 5. Refusal / fail-closed paths — unchanged by construction

Because R1 (the entire gather + `classify_stamp` + the `(Some,Some)` bind) is
untouched, every refusal token is preserved verbatim:

| Token | Trigger | Preserved how |
|---|---|---|
| `missing-cwd` | empty payload `cwd` | classify order unchanged |
| `bad-dir` | `cwd` not under the binding repo, OR not a linked worktree | `cwd_shares_repo(repo, cwd)` / `is_linked_worktree` **unchanged** — incl. cross-repo / outside-repo rejection |
| `missing-agent-type` | `agent_type ≠ dispatch-worker` | unchanged |
| `already-marked` | re-entrant stamp | unchanged |
| M3 no-rollback | provision/mark (or `primary_worktree`) fail | loud stderr + `Err`, worktree left; `primary_worktree` failure folds into the same path |

Crucially, the codex BLOCKER (self-authenticating `cwd_shares_repo`) does **not**
apply: the binding anchor remains `root::find(process cwd)`, an *independent* repo
reference, so a payload `cwd` in a different repo (or outside any repo) is still
rejected `bad-dir` exactly as today. The fix derives only the SOURCE from the
payload, after R1 has already vouched that `cwd` is a linked worktree of the
binding repo.

## 6. Verification

- **VT-1 (strong regression — the defect pin)** — in `tests/e2e_worktree_stamp.rs`,
  spawn the BUILT binary with **`current_dir` set to the worker worktree** (process
  cwd == worker == fork — the exact Defect-C condition) and a payload `cwd` = that
  worktree. Assert: exit 0, marker present at
  `<worker>/.doctrine/state/dispatch/worker`, `.doctrine/doctrine.just` copied. This
  FAILS against today's `root::find` source and passes only with the primary-source
  fix. Subprocess `current_dir` ⇒ no in-process cwd race. No weaker fallback.
- **VT-2 (`primary_worktree` unit)** — from a linked-worktree path returns the main
  worktree; from the main worktree returns itself (idempotent).
- **VT-3 (refusal regression)** — existing `missing-cwd` / `bad-dir` (non-linked
  source tree; outside-repo) / `missing-agent-type` / `already-marked` / Hookmint
  cases stay green unchanged.
- **VT-4 (BLOCKER-closure pin — NEW)** — payload `cwd` = a real linked worktree of a
  **different** repo than the binding anchor (process cwd in repo A; payload cwd in
  repo B). Assert `bad-dir`: the binding anchor `cwd_shares_repo` must still reject
  cross-repo, proving the fix did not turn validation self-authenticating (the
  codex BLOCKER). The existing harness already covers outside-any-repo at
  `tests/e2e_worktree_stamp.rs:281`; this adds the in-a-different-repo case.
- **VH-1** — re-run the IMP-046 fresh-session probe (Agent tool,
  `isolation: worktree`, matcher hook): worker stamped, no hand-stamp.
  Harness-dependent; not in-suite.
- **Gate** — `just check`, zero clippy warnings. Behaviour-preservation: the shared
  entity engine is untouched; existing worktree suites stay green.

## 7. Assumptions (post-review)

- **A1 — standard layout via `git worktree list`.** `primary_worktree` trusts git's
  first porcelain entry as the main worktree. Holds for ordinary, separate-git-dir,
  submodule, and worktree-of-worktree layouts. **Bare** repos have no main worktree
  → first entry could be a linked worktree; out of scope (dispatch never runs bare).
  If it ever did, `verify_sibling_worktree` still bails on `source==fork`
  (fail-closed; no silent corruption).
- **A2 — git env inheritance is pre-existing.** `primary_worktree`,
  `is_linked_worktree`, `cwd_shares_repo`, and `verify_sibling_worktree` all run git
  and inherit `GIT_DIR`/`GIT_COMMON_DIR` from the process env. SL-125 adds one git
  call (`worktree list`); it does not change the trust posture — the stamp path
  already depends on the ambient git env. A hostile `GIT_*` env is out of scope and
  unchanged by this slice.
- **A3 — pure/imperative split.** `primary_worktree` is impure (git read) and sits
  in the shell; `classify_stamp` stays pure (ADR-001 leaf, CLAUDE.md split).
- **A4 — source byte-equivalence is allowlist-scoped.** Justified only for the
  current `.worktreeinclude` (one static file). See §2 limitation + follow-up.

## 8. Non-goals

- ISS-011 Defect A (matcher heal on reinstall) and Defect B (`(deleted)` path
  poison) — SL-124 territory.
- `SubagentStart` wiring / matcher / `/dispatch-agent` skill leg — proven sound.
- The subprocess (codex/pi) and coordination provisioning paths — they resolve
  SOURCE from the orchestrator process and are unaffected (Defect C is claude-arm
  only; single caller `main.rs:4190`).
- The marker-absent fail-closed privilege rule (ADR-006 D2a) — unchanged.
- `verify-worker` self-stamp-on-first-use — rejected in ISS-011 (fix the writer).

## Follow-ups

- **FU-1** — if `.worktreeinclude` later includes per-worktree-divergent untracked
  state that a worker must inherit from the orchestrator tree, design an
  orchestrator-addressable provisioning path (the hook cannot name that tree). File
  against ISS-011's family if/when it arises.
