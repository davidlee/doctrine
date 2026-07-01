# SL-185 Design — Subprocess-arm Seatbelt confinement (macOS jail parity)

Governed by ADR-008 (jail build isolation + worker confinement), ADR-006
(worktree posture), ADR-011 (harness-agnostic spawn), ADR-001 (layering);
POL-002 (platform independence), STD-001 (named constants). Discharges the
subprocess (pi) residual of IMP-045 that SL-183 scoped out. Reuses SL-183's pure
Seatbelt machinery unchanged; the change is a new command-tier consumer plus a
launcher branch.

## 1. Current vs target behaviour

### Current (subprocess arm)

`scripts/pi-spawn-confined.sh` forks the worker worktree (`doctrine worktree fork
--worker` → `$D`) then wraps the whole `pi --mode rpc` exec once in **inline
bwrap flags**, hand-written in shell:

```
timeout bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp \
  --bind ~/.pi ~/.pi --bind "$D" "$D" --chdir "$D" --die-with-parent \
  -- pi --mode rpc … < fifo > out
```

The confinement never touches `jail.rs` and reads no policy file. On macOS
`bwrap` is absent, so the worker runs **unconfined** — the parity gap this slice
closes.

### Target (subprocess arm, macOS parity)

```
doctrine worktree fork --worker  → $D
case $(uname) in
  Darwin) mapfile -d '' PREFIX < <(doctrine worktree jail-prefix \
            --dir "$D" --main-root "$ROOT") || abort ;;
  *)      PREFIX=( bwrap --ro-bind / / … ) ;;   # (A): Linux inline, UNTOUCHED
esac
timeout "${PREFIX[@]}" pi --mode rpc … < fifo > out
```

macOS branch: `jail-prefix` materializes `$D/.tmp/jail.sb` and emits the
`sandbox-exec -f … -D WT=<realpath> … --` prefix (NUL-delimited, terminating in
`--`). One whole-process wrap; children inherit (SL-183-probed). Fail-closed: any
resolve/materialize error → nonzero exit + reason on stderr → the script
**aborts the spawn**, never falling through to an unconfined `pi`.

Net: macOS subprocess workers gain a write-floor identical in policy to the
claude arm (deny `file-write*`, re-allow under `$D` + validated `extra_rw`);
Linux workers unaffected; no worker influences its own policy (the prefix is
orchestrator-computed).

**Boundary this design commits to:** the wrap is applied by the orchestrator's
spawn script around the harness exec — not per-command, not by a hook. That is
why one whole-process `sandbox-exec` wrap suffices, versus the claude arm's
per-Bash rewrite.

## 2. Decisions

- **D1 — altitude (A).** A new `jail-prefix` command-tier consumer delivers
  macOS subprocess parity with least disruption. The Linux inline-bwrap path is
  untouched. Unifying the Linux subprocess arm onto `jail-prefix` (and richer
  configurable jails) is the **(B)** follow-up; `jail-prefix` is built so (B) is a
  script swap, not a rewrite.
- **D2 — `jail-prefix` cfg-splits and emits both backends.** macOS →
  `sandbox-exec` prefix; Linux → `bwrap` prefix via the existing `bwrap_argv`.
  Mirrors `probe_backend`'s cfg-split. Consequence: the command's whole
  plumbing (argparse, emit, fail-closed, materialize dispatch) is exercised on
  Linux by real tests, shrinking the cfg-rot surface to just the irreducibly-mac
  `resolve_with_policy`/seatbelt branch. (B) becomes a one-line script swap.
- **D3 — inline policy, factored.** The disk per-worktree policy handshake exists
  to work around a claude-hook limitation (a fresh process per Bash call cannot
  receive policy except via disk); the subprocess arm is orchestrator-invoked and
  does not share that constraint. So `jail-prefix` takes policy from flags
  (default = the permissive worktree floor). `resolve_inputs` is factored into
  `acquire_policy` (disk lookup, claude only) and a shared
  `resolve_with_policy(policy, …)` core; `validate_policy` moves into the shared
  core so an inline `extra_rw` is validated identically. No parallel path; the
  claude arm is behaviour-preserved by recomposition.
- **G1 — minimal harness invocation (design goal).** The spawn seam is
  `PREFIX=$(jail-prefix …); "${PREFIX[@]}" <harness-cmd>` — a harness-agnostic
  wrap prefix. Adding a new harness (codex, …) is one spawn line. Aligns ADR-011.

## 3. Code impact

The seam is clean because `Jailer::wrap_argv(wt, policy) -> Vec<OsString>`
already returns the wrap **prefix terminating in `--`** (`jail.rs`, asserted at
the `FLAG_ARG_SEP` last-token test). `jail-prefix` resolves a backend, calls
`wrap_argv`, emits — no new argv logic.

### `src/worktree/jail.rs` — factor, no parallel builder

```rust
// disk lookup: topology → basename → read_policy → from_toml_str  (claude only)
fn acquire_policy(cwd: &Path, env: &dyn ResolveEnv) -> Result<JailPolicy, ResolveDeny>;

// SHARED core: validate_policy + topology(wt/is_linked) + realpaths
fn resolve_with_policy(
    policy: &JailPolicy, cwd: &Path, main_root: &Path, env: &dyn ResolveEnv,
) -> Result<ResolvedMac, ResolveDeny>;

// behaviour-preserving recomposition (claude arm):
fn resolve_inputs(cwd, main_root, env) =
    resolve_with_policy(&acquire_policy(cwd, env)?, cwd, main_root, env)

// extracted single writer (was inline in pretooluse::materialize_seatbelt_profile):
fn write_seatbelt_profile(resolved: &ResolvedMac) -> io::Result<()>;
```

`validate_policy` moves from `resolve_inputs` into `resolve_with_policy` so both
the disk and inline policy sources are validated (an inline `--extra-rw` cannot
smuggle root/ancestor/`.git`). Pure builders — `seatbelt_profile`,
`sandbox_exec_argv`, `bwrap_argv`, `bwrap_core_argv` — untouched (ADR-001 leaf).

### `src/worktree/mod.rs` — new command-tier consumer

```rust
WorktreeCommand::JailPrefix { dir: PathBuf, main_root: Option<PathBuf>,
                             network: bool, extra_rw: Vec<PathBuf> }
```

`run_jail_prefix()` — impure command tier, cfg-split (mirrors `probe_backend`):

- Build `JailPolicy` from flags (default = `JailPolicy::default()` worktree floor).
- `#[cfg(not(target_os = "macos"))]` — `select_jailer(Backend::Bwrap)`; `bwrap`
  absent ⇒ fail-closed. `wrap_argv(realpath(dir), &policy)` → bwrap prefix.
- `#[cfg(target_os = "macos")]` — `resolve_with_policy(&policy, dir, main_root, RealEnv)`
  (main_root **required** here) → `Seatbelt { resolved }`; `write_seatbelt_profile`
  (io error ⇒ fail-closed `Deny{seatbelt-profile-write-failed}`); `wrap_argv` →
  sandbox-exec prefix.
- Emit the prefix **NUL-delimited** to stdout. On any `Deny` → nonzero exit +
  reason on stderr, empty stdout.

### `src/worktree/pretooluse.rs` — dedupe

`materialize_seatbelt_profile` calls the extracted `write_seatbelt_profile`
instead of the inline `fs::write(profile_path, seatbelt_profile(resolved))`. The
claude PreToolUse suites are the behaviour-preservation proof.

### `scripts/pi-spawn-confined.sh` — launcher branch

`uname` branch: `Darwin` → `mapfile -d '' PREFIX < <(doctrine worktree jail-prefix
--dir "$D" --main-root "$ROOT")` with nonzero → abort; `*` → the existing inline
bwrap array. A single `timeout "${PREFIX[@]}" pi …` exec site.

### design-target selectors

`src/worktree/jail.rs`, `src/worktree/mod.rs`, `src/worktree/pretooluse.rs`,
`scripts/pi-spawn-confined.sh`.

## 4. Verification

### VT — Linux, here (red/green/refactor)

- **Behaviour-preservation:** existing `resolve_inputs` suites stay green
  unchanged after the `acquire_policy ∘ resolve_with_policy` split.
- `resolve_with_policy(inline_policy, …)` resolves from a supplied policy (no
  disk) and **rejects a dangerous inline `extra_rw`** (root/ancestor/`.git`) via
  the moved `validate_policy`.
- `run_jail_prefix` **Linux arm**: emits the bwrap prefix, NUL-delimited,
  terminating in `--`, for a given `--dir`/policy; **fail-closed** (bwrap absent →
  nonzero, empty stdout). This exercises the whole command shell on Linux (the
  cfg-rot mitigation).
- `write_seatbelt_profile(resolved)`: writes exactly `seatbelt_profile(resolved)`
  to `profile_path`; io error → `Err`. Pure `fs::write` of a string — no
  `sandbox-exec` needed.
- Emit format: split-on-`\0` round-trips to `wrap_argv` tokens (non-UTF-8 safe).

### VA — agent

- **No parallel implementation:** the new confinement rides `jail.rs` builders;
  the emitted bwrap flags equal `bwrap_core_argv` (existing parity assertion), not
  re-authored.
- **STD-001:** NUL delimiter, command name, reason strings are named constants.

### VH — mac-only, deferred (enforcement gate; cannot close on Linux)

1. profile materializes at `$D/.tmp/jail.sb`;
2. write outside `$D` denied; write inside `$D` succeeds;
3. pi children (bash/cargo/git) inherit the sandbox;
4. **pi's fifo / rpc / session-dir functions under `sandbox-exec`** (the
   long-lived-process risk absent from the claude per-Bash arm);
5. fail-closed live: corrupt/absent policy → `jail-prefix` nonzero → spawn aborts,
   no unconfined pi.

### cfg-rot guard (dev/CI, distinct from VH)

`cargo check --target aarch64-apple-darwin` type-checks the mac branch without a
mac (needs the target installed). Catches mac-branch bitrot between VH passes.

## 5. Open questions (for `/plan`, not blockers)

- **OQ-a** — `resolve_inputs` recomposition double-calls `worktree_topology`:
  thread topology through vs accept two git calls. Plan decides.
- **OQ-b** — the default subprocess policy needs one `extra_rw` for `~/.pi`
  (session dir), or a `TMPDIR` redirect — mirror what the bwrap arm binds. Plan
  pins the exact grant.
- **OQ-c** — `main_root` sourcing: explicit `--main-root "$ROOT"` (the script has
  it) vs git-derive. Leaning explicit (unambiguous). Plan confirms.

## 6. Follow-ups (out of scope)

- **(B)** unify the Linux subprocess arm onto `jail-prefix` (single jail source,
  both OSes) + configurable jails.
- **Codex** confined-spawn (one spawn line under G1).
- **IDE-025** selector-sourced write-allowlist as a policy input on the same
  seam. The Seatbelt floor can host a per-selector regex allow-rule — see the
  slice OQ-3 note.

## 7. Risks / assumptions

- SL-183 probe findings (nesting, canonicalization, child inheritance) hold for a
  single whole-process `sandbox-exec` wrap. Re-probe only if the launcher changes
  a nesting/exec assumption; the pi-fifo/rpc-under-sandbox question is new and
  rides VH-4.
- cfg-rot: the mac branch is not compiled by a plain Linux `cargo build`;
  mitigated by D2 (command shell Linux-tested) + the `--target` guard.
