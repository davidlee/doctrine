# SL-041 Design — Resolve branch-point-check --base in the impure shell

Origin: ISS-002 (SL-031 §5.2 re-audit, C-V). Closes the verbatim-trust hole in
`worktree::run_branch_point_check`.

## 1. Current vs target behaviour

`run_branch_point_check(path, base, head)` (`src/worktree.rs`):

| input | current | defect |
|---|---|---|
| `--base <sha> --head <sha>` | string-compare, exit 0/1 | none |
| `--base <sha>` (head absent) | head = `rev-parse HEAD`, base raw | base trusted raw |
| `--base HEAD` (symbolic) vs resolved head | `"HEAD" != <sha>` → exit 1 | **false "moved"** (safe dir, wrong reason) |
| `--base HEAD --head HEAD` | `"HEAD" == "HEAD"` → exit 0 | **false stationary** — guard passes against a base it never resolved (unsafe dir) |

Root cause: only the *absent*-`--head` path resolves a ref; `--base` and any
*passed* `--head` flow to `matches` verbatim. `matches` is raw `base == head`.

**Target.** Both ends are resolved to a canonical commit sha in the impure shell
*before* the compare. `matches` is unchanged — still pure ref-equality. A base
or head that cannot be resolved to a commit makes the verb **bail** (the safe
failure direction for a guard), not silently pass.

## 2. Design — resolution in the shell, leaf stays pure

New impure helper, sibling to the existing git reads:

```rust
/// Peel a base/head ref to its canonical commit sha for the stationarity
/// compare. `--verify <ref>^{commit}` resolves a sha, `HEAD`, a branch, or a
/// (lightweight/annotated) tag down to the commit; an unresolvable ref errors,
/// so the guard bails rather than comparing an unresolved symbol (ISS-002).
fn resolve_commit(root: &Path, reference: &str) -> anyhow::Result<String> {
    Ok(git::git_text(root, &["rev-parse", "--verify", &format!("{reference}^{{commit}}")])?)
}
```

Rewritten shell:

```rust
pub(crate) fn run_branch_point_check(
    path: Option<PathBuf>,
    base: &str,
    head: Option<String>,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let base_sha = resolve_commit(&root, base)?;
    let head_sha = resolve_commit(&root, head.as_deref().unwrap_or("HEAD"))?;
    if matches(&base_sha, &head_sha) {
        writeln!(io::stdout(), "stationary: HEAD == base {base_sha}")?;
        Ok(())
    } else {
        bail!("HEAD moved: base {base_sha} != HEAD {head_sha}");
    }
}
```

Changes vs current:
- `root::find` now runs unconditionally (base always needs a repo to resolve
  against); previously only on the absent-head branch. No new dependency — same
  helper, same markers.
- absent-`--head` folds into `resolve_commit(root, "HEAD")` — behaviour-preserving
  (HEAD is a commit; `rev-parse --verify HEAD^{commit}` == old `rev-parse HEAD`).
- diagnostic messages now print the *resolved* shas, not the raw input — clearer
  on a symbolic base.

### Why peeled `^{commit}`, not plain `rev-parse <ref>`
The guard compares HEAD *positions* = commit identity. Plain `rev-parse
<annotated-tag>` yields the tag-object sha, not the commit → spurious mismatch.
`^{commit}` peels everything to the commit; `<sha>^{commit}` == `<sha>` for a
commit, so zero change on the common sha/HEAD/branch path.

### Why both ends, not base-only
The verbatim-trust defect is symmetric: a passed `--head` is also raw. Base-only
fixes ISS-002's title and leaves the twin (symbolic passed-head → false "moved").
A safety verb must not trust *either* safety input to be pre-resolved.

## 3. Architecture alignment (ADR-001)

`matches` stays a pure leaf (ref-equality, no clock/git/disk). All resolution is
in the impure shell — the date/uid / impurity-in-the-shell split holds. No change
to the `/dispatch` SKILL contract: it already captures `B = rev-parse HEAD` and
passes a resolved sha; `resolve_commit(<sha>)` is the identity on that input.

## 4. Code impact

| path | change |
|---|---|
| `src/worktree.rs` | add `resolve_commit`; rewrite `run_branch_point_check` shell; update verb doc-comment (resolution now both-ended, bails on unresolvable ref) |
| `tests/e2e_worktree_branch_point.rs` | add VT cases (below) |
| `src/worktree.rs` `mod tests` | `matches` unit test unchanged (leaf untouched) |

No change to `src/main.rs` wiring or the CLI arg surface.

## 5. Verification alignment

`matches_is_ref_equality` (leaf) stays green unchanged — the behaviour-preservation
gate on the pure compare.

Existing e2e `stationary_head_exits_zero_moved_head_exits_one` stays green; note
the `--head deadbeef` case (line 95) now exits non-zero by *resolution error*
(`deadbeef^{commit}` unverifiable) rather than string mismatch — the asserted
exit-code contract (`!success`) is unchanged.

New e2e rows (same `init_repo` harness):

- **VT — symbolic base resolves (the ISS-002 fix):** `--base HEAD` with HEAD
  stationary ⇒ exit **0**. (Today: exit 1, false "moved".)
- **VT — symbolic base, moved HEAD:** capture `base=<sha>`; stray commit;
  `--base HEAD --head <old sha>`-style / `--base <old sha>` ⇒ exit **1**.
- **VT — both ends resolved:** `--base HEAD --head HEAD` ⇒ exit **0** because both
  peel to the same sha (not because of a `"HEAD"=="HEAD"` string accident);
  after a stray commit, `--base <old sha> --head HEAD` ⇒ exit **1** (proves the
  passed head is resolved too).
- **VT — unresolvable base bails:** `--base does-not-exist` ⇒ exit non-zero with
  a resolution error (the safe failure direction).

## 6. Invariants & boundary conditions

- I1: after resolution, both compare operands are full commit shas (or the verb
  has already bailed). `matches` never sees a symbolic ref.
- I2: equal valid commit shas ⇒ stationary ⇒ exit 0; any other resolved pair ⇒
  exit 1; any unresolvable operand ⇒ bail (exit ≠ 0).
- I3: `/dispatch`'s resolved-sha input is unaffected (`resolve_commit` is identity
  on a valid commit sha).

## 7. Decisions

- **D1 — both ends, peeled `^{commit}`.** Locked. Rejected: base-only (leaves the
  twin bug); plain `rev-parse` (tag-object mismatch).
- **D2 — `root::find` unconditional.** Base always needs a repo; the prior
  conditional was an artefact of resolving only HEAD.

## 8. Open questions

None. Scope is one surface, one helper, four VT rows.

## 9. Adversarial self-review (integrated)

- A1 — helper param renamed `reff` → `reference` (keyword-dodge). Applied above.
- A2 — resolution-error `bail!` maps to the same non-zero exit as the existing
  "HEAD moved" `bail!` (one path through `main`). Confirmed, no new exit code.
- A3 — `git::git_text` returns `CaptureError`; `Ok(… ?)` converts via `?` to
  `anyhow::Error` exactly as the current shell does. Sound.
- A4 — `--base` is a required CLI arg; `resolve_commit("")` bails where the old
  code reached `matches("", head)` → exit 1. Both non-zero — no contract change.
- A5 — `src/main.rs:1176` read-class comment ("HEAD read + ref compare — no
  authored write") stays valid; the verb now also *reads* the base ref but writes
  nothing, so its mode classification is unchanged.
- A6 — verb doc-comment must state both-ended peeled resolution + bail-on-
  unresolvable; tracked in §4 code impact.
