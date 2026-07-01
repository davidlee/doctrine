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
  Darwin)
    # AR-1: emit argv to a FILE, not process-substitution — so the exit status
    # is real (direct command) and NUL bytes survive (file, not $()/var).
    doctrine worktree jail-prefix --dir "$D" --main-root "$ROOT" \
      --out "$D/.tmp/jail.argv" || abort            # fail-closed: no unconfined pi
    mapfile -d '' PREFIX < "$D/.tmp/jail.argv"
    [ "${#PREFIX[@]}" -gt 0 ] || abort              # AR-1: empty prefix ⇒ abort (defence in depth)
    ;;
  *)  PREFIX=( bwrap --ro-bind / / … ) ;;           # (A): Linux inline, UNTOUCHED
esac
timeout "${PREFIX[@]}" pi --mode rpc … < fifo > out
```

macOS branch: `jail-prefix` materializes `$D/.tmp/jail.sb` and writes the
`sandbox-exec -f … -D WT=<realpath> … --` prefix (NUL-delimited, terminating in
`--`) to `--out`. One whole-process wrap; children inherit (SL-183-probed).
**Fail-closed (AR-1):** any resolve/materialize/write error → nonzero exit +
reason on stderr, no `--out` file (or empty) → the script **aborts the spawn**,
never falling through to an unconfined `pi`. The file seam is load-bearing: a
`$(…)` capture would both strip NUL and swallow the child's exit status.

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
// XR-3: probe topology ONCE, thread it — no second git call, no read-A/resolve-B window.
// disk lookup keyed by the SAME topology used to resolve (claude only):
fn acquire_policy(topo: &Topology, env: &dyn ResolveEnv) -> Result<JailPolicy, ResolveDeny>;

// SHARED core: validate_policy + realpaths, over an already-probed topology.
fn resolve_with_policy(
    policy: &JailPolicy, topo: &Topology, main_root: &Path, env: &dyn ResolveEnv,
) -> Result<ResolvedMac, ResolveDeny>;

// behaviour-preserving recomposition (claude arm) — ONE topology probe:
fn resolve_inputs(cwd, main_root, env) = {
    let topo = env.worktree_topology(cwd)?;   // the single probe (was inline in the old body)
    resolve_with_policy(&acquire_policy(&topo, env)?, &topo, main_root, env)
}

// extracted single writer (was inline in pretooluse::materialize_seatbelt_profile):
fn write_seatbelt_profile(resolved: &ResolvedMac) -> io::Result<()>;
```

`validate_policy` moves from `resolve_inputs` into `resolve_with_policy` so both
the disk and inline policy sources are validated (an inline `--extra-rw` cannot
smuggle root/ancestor/`.git`). Pure builders — `seatbelt_profile`,
`sandbox_exec_argv`, `bwrap_argv`, `bwrap_core_argv` — untouched (ADR-001 leaf).

**XR-1 (external, D-canon obligation on the inline source).** `validate_policy`
is a PURE lexical ancestor test whose precondition is that `extra_rw` is
*already* shell-canonicalized (`jail.rs:361-368`, the D-canon cut); `resolve_*`
runs it *before* the realpath step. The claude arm meets this because its
`extra_rw` arrives from a disk policy canonicalized upstream. The subprocess
arm's inline `--extra-rw` is **raw orchestrator-shell input** — a `..`-bearing or
symlinked grant would pass the lexical check, then realpath to `/` / a main-root
ancestor / `.git` and be bound rw (sandbox-widening). So `run_jail_prefix` MUST
`env.realpath` each inline `--extra-rw` (fail-closed if it does not exist) and
build `JailPolicy` from the canonicalized set **before** calling
`resolve_with_policy`. `jail-prefix` *is* the shell seam for this arm — meeting
D-canon is its job, exactly as `pretooluse-pathcheck.sh` meets it for the claude
arm. Do NOT reorder `resolve_with_policy` to realpath-before-validate: that would
break the behaviour-preserved claude ordering; canonicalize at the inline source
instead.

### `src/worktree/mod.rs` — new command-tier consumer

```rust
WorktreeCommand::JailPrefix { dir: PathBuf, main_root: Option<PathBuf>,
                             out: PathBuf,                 // AR-1: NUL-delim argv sink
                             network: bool, extra_rw: Vec<PathBuf> }
```

`run_jail_prefix()` — impure command tier, cfg-split (its OWN backend
resolution — NOT `probe_backend`, whose macOS branch reads the disk policy D3
rejects; AR-4):

- **Canonicalize inline grants first (XR-1):** `env.realpath` each `--extra-rw`
  (nonexistent ⇒ fail-closed), then build `JailPolicy` from the canonicalized set
  (default = `JailPolicy::default()` worktree floor). This meets the D-canon
  precondition `validate_policy` assumes — the inline source is raw shell input.
- `#[cfg(not(target_os = "macos"))]` — reuse the **bwrap-presence helper**
  factored out of `probe_backend` (AR-4, avoid a parallel check); present ⇒
  `select_jailer(Backend::Bwrap)`, absent ⇒ fail-closed. `wrap_argv(realpath(dir),
  &policy)` → bwrap prefix.
- `#[cfg(target_os = "macos")]` — probe `topo = worktree_topology(dir)` once, then
  `resolve_with_policy(&policy, &topo, main_root, RealEnv)`
  (main_root **required**; absent ⇒ fail-closed) → `Seatbelt { resolved }`;
  `write_seatbelt_profile` (io error ⇒ fail-closed `Deny{seatbelt-profile-write-failed}`);
  `wrap_argv` → sandbox-exec prefix.
- Write the prefix **NUL-delimited** to `--out` (AR-1). On any `Deny` → nonzero
  exit + reason on stderr, **no** `--out` written (never a partial/empty file the
  caller could mistake for success).

### `src/worktree/pretooluse.rs` — dedupe

`materialize_seatbelt_profile` calls the extracted `write_seatbelt_profile`
instead of the inline `fs::write(profile_path, seatbelt_profile(resolved))`. The
claude PreToolUse suites are the behaviour-preservation proof.

### `scripts/pi-spawn-confined.sh` — launcher branch

`uname` branch: `Darwin` → the **§1 `--out` file contract** (XR-2 — NOT
process-substitution, which reburies the AR-1 exit-status/NUL hole): `jail-prefix
--out "$D/.tmp/jail.argv" || abort`, then `mapfile -d '' PREFIX < "$D/.tmp/jail.argv"`,
then the `[ "${#PREFIX[@]}" -gt 0 ] || abort` empty-guard. `*` → the existing inline
bwrap array. A single `timeout "${PREFIX[@]}" pi …` exec site. The `--out` path is
truncated/removed before the call so a stale prior-run file cannot be mistaken for
success (jail-prefix writes it only on full success).

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
- **XR-5 (shell-reader test, Linux-runnable):** the `mapfile -d '' … < --out` +
  empty-guard reader is exercised against a real `jail-prefix --out` file on Linux
  (Linux branch emits a bwrap prefix — the reader is OS-agnostic). Covers the
  contract `cargo check --target` cannot: stale/empty/absent `--out` ⇒ the guard
  aborts, no unconfined exec. Also: `--out` is written **only** on full success
  (no partial file on `Deny`).
- **XR-1:** inline `--extra-rw` canonicalization — a `..`/symlink grant resolving
  outside `$D` (or onto `.git`) is **rejected** (realpath-then-validate at the
  inline source), proving the D-canon precondition is met before `validate_policy`.

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

- **OQ-a (RESOLVED, XR-3 — was cost, is correctness).** The naïve
  `acquire_policy ∘ resolve_with_policy` split probes `worktree_topology` twice —
  not just a wasted git call but a read-policy-for-A / resolve-topology-for-B
  window if the FS shifts between calls. **Decision: thread one `Topology`** —
  `resolve_inputs` probes once and passes it to both (signatures in §3). Single
  probe, no drift; behaviour-preserved (the old body probed once too).
- **OQ-b (revised, AR-2)** — does the default subprocess policy need **any**
  `~/.pi` grant at all? Seatbelt is allow-default / deny-`file-write*` — reads are
  open — and the script already redirects pi's session under `$D`
  (`--session-dir "$D/.pi-session"`). So `~/.pi` config is read-open for free; a
  grant is needed **only if pi writes `~/.pi` at runtime** with the session
  redirected. The bwrap `--bind ~/.pi` grants write only because `--ro-bind /`
  made everything ro first — not evidence pi writes there. Plan/probe must
  *establish whether pi writes `~/.pi`*; if yes → `extra_rw` (+ existence/realpath,
  which fails closed if `~/.pi` is absent); if no → no grant. Do not assume the
  bwrap bind implies a needed write grant.
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

- **RISK-1 (gating, AR-3) — pi's fifo/rpc under `sandbox-exec` is potentially
  fatal, not merely VH-4.** SL-183's probe wrapped short-lived *bash* commands;
  it does NOT cover a long-lived `pi --mode rpc` process reading a fifo and
  spawning children. If `sandbox-exec` breaks that, the whole launcher strategy
  fails and the Rust is moot. Falsification-first (SL-183's own posture) says
  probe this **cheap and early**: a disposable
  `sandbox-exec -f <floor.sb> -- pi --mode rpc … < fifo` on a mac, BEFORE heavy
  investment. Tension: dev is Linux-first (the slice's premise), so the probe is
  itself mac-deferred. Resolution: the Linux pure-layer work is low-regret
  (factored/reused regardless), but **RISK-1 is the go/no-go a mac must answer,
  ideally probed before the launcher wiring lands** — sequence it as the first
  mac-side action, not the last. If it fails: reconsider the launcher (e.g. confine
  pi's *children* rather than the rpc host, or a different mac seam) — the Rust
  `jail-prefix`/factoring survives either way.
  **XR-4 (sequencing hardened):** the launcher *wiring* — `run_jail_prefix`'s macOS
  branch and the `pi-spawn-confined.sh` Darwin arm — MUST NOT land before RISK-1
  clears on a mac; it is the only mac-dependent code and it is the code RISK-1 can
  invalidate. The Linux-safe work (the `acquire_policy`/`resolve_with_policy`/
  `write_seatbelt_profile` factoring, the Linux `jail-prefix` branch, all VT) is
  low-regret and proceeds now; the plan sequences the mac-branch phase *behind* the
  RISK-1 probe. Codex would block *all* planning on the probe; the user's premise is
  Linux-first with mac deferred, so we gate the mac-touching phase, not the slice.
- SL-183 probe findings (nesting, canonicalization, child inheritance) otherwise
  hold for a single whole-process wrap.
- cfg-rot: the mac branch is not compiled by a plain Linux `cargo build`;
  mitigated by D2 (command shell Linux-tested — `resolve_with_policy`/
  `write_seatbelt_profile`/`seatbelt_profile` are all exercised on Linux via
  `FakeEnv`; only the macOS `RealEnv` wiring is cfg-gated) + the `--target` guard.
  **XR-5 (honest scope):** `cargo check --target` type-checks the mac *Rust*
  branch but exercises **nothing** at the shell/runtime seam — the `uname` dispatch,
  the `mapfile`/`timeout "${PREFIX[@]}"` wrapper, and the live `sandbox-exec`/pi
  interaction all escape it. Do NOT read "whole plumbing Linux-tested" as covering
  those: the shell reader is covered by the XR-5 Linux test (§4); the `sandbox-exec`
  runtime is covered only by VH + RISK-1. The `--target` guard is a bitrot tripwire,
  not enforcement proof.

## 8. Adversarial review log

Internal hostile pass on the drafted design; material findings integrated above.

- **AR-1 (bug, fixed §1/§3):** `mapfile -d '' < <(jail-prefix)` hides the child's
  exit status and can't carry NUL through `$()`; the `|| abort` never fired →
  unconfined pi on failure. Fixed: `jail-prefix --out <file>` (real exit status,
  NUL survives in a file) + empty-`PREFIX` guard.
- **AR-2 (claim corrected §5 OQ-b):** "needs an `extra_rw` for `~/.pi`" was
  unjustified — Seatbelt reads are open and the session is already under `$D`; a
  write grant is needed only if pi actually writes `~/.pi`. Downgraded to a
  probe/plan question, not a default.
- **AR-3 (gating risk added §7 RISK-1):** the pi-under-`sandbox-exec` question is
  potentially fatal and uncovered by SL-183's probe; front-load it as the mac
  go/no-go rather than final VH.
- **AR-4 (design corrected §3):** `jail-prefix` must not reuse `probe_backend`
  (its macOS branch reads the disk policy D3 rejects); it uses its own backend
  resolution and only reuses a factored bwrap-presence helper.
- **Considered, no change:** moving `validate_policy` into `resolve_with_policy`
  preserves the original from_toml → validate → realpath order (behaviour-preserved);
  the `uname` OS-dispatch is legitimate backend selection, not host-project
  coupling (POL-002 intact — SL-183 set this precedent with its cfg-split).

### External pass (codex / GPT-5.5, read-only, verified against `jail.rs`)

- **XR-1 (major, ACCEPT — was codex-blocker) §3:** the inline `--extra-rw` is raw
  shell input but `validate_policy` is lexical with a "pre-canonicalized"
  precondition (D-canon, `jail.rs:361-368`), run *before* realpath — a `..`/symlink
  grant would widen the sandbox. Fix: `jail-prefix` `env.realpath`s each inline
  grant *before* `validate_policy`. Canonicalize at the source; do not reorder the
  behaviour-preserved core.
- **XR-2 (major, ACCEPT — doc bug) §3:** the launcher snippet contradicted §1,
  regressing to `< <(…)` process-substitution and reviving the AR-1
  exit-status/NUL hole. Fixed to the §1 `--out` file contract + stale-file
  truncation.
- **XR-3 (major, ACCEPT) §5 OQ-a:** the split double-probes topology — not just
  cost but a read-A/resolve-B window. Resolved: thread one `Topology` (signatures
  updated §3); single probe, behaviour-preserved.
- **XR-4 (partial, ACKNOWLEDGE) §7:** codex would block all planning on the pi-rpc
  probe. Kept Linux-first (user's premise); hardened the sequencing so the
  mac-branch wiring lands *behind* the RISK-1 probe while the low-regret factoring
  proceeds.
- **XR-5 (minor, ACCEPT) §4/§7:** `cargo check --target` covers neither the shell
  reader nor the `sandbox-exec` runtime. Added a Linux-runnable `mapfile`-reader
  test; scoped the cfg-rot claim honestly (tripwire, not proof).
