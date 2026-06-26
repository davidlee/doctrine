# SL-156 Research

Pre-slice investigation. Sources: memories, code inspection, ADRs, backlog.

## The problem — shared CARGO_TARGET_DIR

`CARGO_TARGET_DIR=/home/david/.cargo/doctrine-target-jail` (set in `flake.nix`).
One cache across every worktree. Cargo's fingerprint reuses a **test artifact**
compiled in tree W when tests run from tree Z → false RED / false GREEN.

Two axes of the footgun (both documented in IMP-004):

- **Axis #1 — path-baking (FIXED, CHR-014).** `env!("CARGO_MANIFEST_DIR")` baked
  the building tree's absolute path. Now resolved via runtime
  `std::env::var("CARGO_MANIFEST_DIR")` in `src/test_support.rs::repo_root()`.
  Guarded by `e2e_no_baked_manifest_dir` test.
  Memory: `mem.fact.testing.runtime-manifest-dir`.

- **Axis #2 — stale test artifact (this slice's domain).** The compiled test
  binary itself is stale — deleted tests still run, old fixtures embedded.
  `just rebuild-stale` (`touch src/main.rs && cargo build`) only forces a BIN
  rebuild, not test binaries. Clearing a stale test needs `cargo clean -p
  doctrine` or `touch tests/<file>.rs`.
  Memory: `mem.fact.build.rebuild-stale-skips-test-binaries`.

### Evidence cluster (ADR-008 §5.1)

- `mem.pattern.build.jail-target-redirect` — jail cargo writes to shared target
- `mem.pattern.testing.shared-cargo-target-stale-binary` — shared target → stale
  integration test binaries
- `mem_019edf8f57d2726281fcddd36d5197b1` — worktrees share target → builds thrash
- `mem.pattern.jail.stale-test-fixture-vocabulary-change` — stale test binary
  embeds old fixture corpus → reads like a logic regression
- `mem.pattern.dispatch.shared-target-false-green-touch-rerun` — touch + re-run
  to confirm fresh compile
- `mem.pattern.dispatch.worktree-removal-stale-manifest-dir-false-red` — removed
  worktree → false RED until recompiled
- `mem.pattern.testing.stale-cargo-bin-exe` — stale CARGO_BIN_EXE makes e2e
  tests spawn-fail

## Design authority — ADR-008

D-B1: Per-worktree `CARGO_TARGET_DIR`, nested under the jail-redirect root, keyed
by `wt/<branch>`, set at worker spawn. Restores parallel builds, kills
cross-branch fingerprint thrash, gives correct per-worktree `CARGO_BIN_EXE`.

D-B2: No in-jail `cargo install` — structural (RO binary), not a policy.

D-B3: Per-worker bwrap confinement — spike-first, codex/pi only. Out of scope.

D-B4: sccache — deferred.

D-B5: Keep the flake minimal — per-worktree env set at spawn, not baked in flake.

## Current implementation state

### Pure layer — done

`src/worktree/shared.rs`:
```rust
pub(crate) fn target_dir_for_branch(branch: &str) -> PathBuf {
    Path::new("wt").join(branch)
}
```
Pure mapping, unit-tested. ✅

### Env contract — done, but only CLI-accessible

`src/worktree/fork.rs`:
```rust
pub(super) fn project_env_contract(fork: &Path, branch: &str) -> Vec<(String, String)> {
    let base = match std::env::var_os("CARGO_TARGET_DIR") {
        Some(v) => PathBuf::from(v),
        None => fork.join("target"),
    };
    let target = base.join(target_dir_for_branch(branch));
    vec![("CARGO_TARGET_DIR".to_owned(), target.to_string_lossy().into_owned())]
}
```
Called ONLY by `run_fork()` (the CLI verb). ✅ for CLI path.

### fork_core — shared, silent

`src/worktree/fork.rs`:
```rust
pub(super) fn fork_core(repo: &Path, base: &str, branch: &str, dir: &Path, worker: bool) -> anyhow::Result<()>
```
Byte-identical creation core: add + provision + mark. SILENT (no stdout).
Called by BOTH `run_fork()` and `create-fork`. ✅ shared.

### run_fork — codex/pi arm only

```rust
pub(crate) fn run_fork(...) -> anyhow::Result<()> {
    fork_core(&repo, base, branch, dir, worker)?;
    for (key, value) in project_env_contract(dir, branch) {
        writeln!(io::stdout(), "{key}={value}")?;
    }
    // ...
}
```
Emits `CARGO_TARGET_DIR=...` on stdout. ✅

### create-fork — Claude WorktreeCreate hook

`src/worktree/create.rs`:
```rust
fn act_on_create(root: &Path, action: CreateAction) -> anyhow::Result<PathBuf> {
    match action {
        CreateAction::Fork { base, name } => {
            // ...
            fork_core(root, &base, &branch, &dir, true)?;
            fs::canonicalize(&dir) // returns ONLY the path
        }
        // ...
    }
}
```
Calls `fork_core` — same core. But NEVER calls `project_env_contract`.
Only returns the created path on stdout (the WorktreeCreate hook contract).
Nowhere to emit env vars for the spawned Agent. ❌

### dispatch-subprocess skill — codex/pi

Captures `$fork_env` from `run_fork` stdout, passes to `env -C "$D" $fork_env ...`:
```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)"
env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "..."
```
✅ per-worktree target set.

### dispatch-agent skill — Claude

Uses `doctrine dispatch arm-spawn --base B` + cd into spawn dir + Agent spawn.
The WorktreeCreate hook calls `create-fork` → `fork_core`. Agent inherits
orchestrator's `CARGO_TARGET_DIR`. ❌ per-worktree target NOT set.

## The gap — confirmed

| Arm | Creation path | Env contract injection |
|---|---|---|
| codex/pi | `run_fork` → `fork_core` → stdout env | ✅ `$fork_env` captured and passed |
| Claude Agent | `create-fork` → `fork_core` → stdout path | ❌ No channel to inject env |

Both arms share `fork_core`. The gap is purely at the env injection layer.
`project_env_contract` exists but is never called from the Claude arm path.

## Possible approaches for Claude arm env injection

The WorktreeCreate hook can only print the created path. It cannot set env vars
for the spawned subagent. Options:

1. **Write `.cargo/config.toml` in worktree during provision** — set
   `build.target-dir` to the per-worktree path. Global `CARGO_TARGET_DIR` env
   overrides it, so the worker must ALSO unset or override the env var.

2. **Worker base-guard block sets `CARGO_TARGET_DIR`** — embed an explicit
   `export CARGO_TARGET_DIR=...` in the worker prompt's base-guard section.
   The agent sets it in every Bash call. Low-tech, reliable.

3. **SubagentStart hook injection** — if the SubagentStart hook can set env for
   the spawned subagent (currently it runs as a subprocess, not env-modifying).

4. **Write a `.env` or shell init in the worktree** — rely on agent's shell
   sourcing it (fragile across harnesses).

Design will select and harden the approach.

## Related entities

| Entity | Status | Role |
|---|---|---|
| ADR-008 | accepted | Design authority for per-worktree isolation |
| IMP-004 | open | Backlog item this slice implements |
| ISS-011 | closed/done | Hook stamp reliability (dependency) |
| SL-152 | — | `create-fork` verb (unified creation path) |

## Relevant memories (for design)

- `mem.fact.doctrine.jail-target-dir` — CARGO_TARGET_DIR value
- `mem.pattern.build.jail-target-redirect` — jail target redirect
- `mem.pattern.build.jail-binary-for-skill-install` — which binary is fresh
- `mem.fact.build.rebuild-stale-skips-test-binaries` — rebuild-stale limitation
- `mem.pattern.dispatch.subagentstart-hook-cwd-is-worker-worktree` — ISS-011 Defect C
- `mem.pattern.testing.shared-cargo-target-stale-binary`
- `mem.pattern.testing.stale-cargo-bin-exe`
- `mem.pattern.embed.rust-embed-no-rerun`
