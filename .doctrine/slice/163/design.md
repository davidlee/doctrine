# Design SL-163: check command proxy verb

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The shipped skill corpus (`plugins/**`, embedded via RustEmbed and materialised
into client projects by `doctrine claude install`) hardcodes *this* repo's
conventions — a POL-002 platform-independence gap. Two couplings:

1. Six skill sites tell agents to run `just check` at phase / commit boundaries.
   A client has no `justfile`; the instruction load-bears on a host convention
   this repo owns (POL-002 facet 1).
2. `dispatch/SKILL.md:19` cites `mem_019ec65ecbc7`, a repo-local memory uid that
   does not exist in a client corpus — the citation dangles on install.

The fix: add a CLI verb that proxy-executes project-declared check commands
sourced from a contract **doctrine owns** (the `doctrine.toml [verification]`
table), rewrite the shipped skills onto the verb, and scrub the dangling uid.
The `just …` strings survive only as *informing defaults*, never carried
correctness (POL-002 Scope explicitly blesses this repo keeping `just gate`).

## 2. Current State

- **Owned contract.** `[verification]` in `.doctrine/doctrine.toml` (const
  `dtoml::DOCTRINE_TOML`). Field `command: Option<Vec<String>>` = the VT-evidence
  base argv, read through ONE reader: `coverage_store::load_config(root)` →
  `dtoml::parse().verification`. Pure argv resolution (`verify::resolve`) is a
  leaf (ADR-001, no IO).
- **Spawn precedent.** `coverage_verify::run_argv` spawns argv but *pipes +
  captures + caps with a timeout* (VT must match output). The check verb wants
  the opposite posture (§5.4).
- **Skill sites.** Six `just check` occurrences (`notes:32`, `audit:109`,
  `close:33`, `execute:50`, `worktree:192`, `worktree:219`); **zero** `just gate`.
  One dangling uid (`dispatch:19`).
- **Wiring.** Verb arms live in `cli.rs::Command` (enum) → `cli.rs::dispatch`
  (exec) → a per-kind handler module; `guard.rs::write_class` classifies each arm
  Read/Write; modules register in `commands/mod.rs`. Root via
  `root::find(path, default_markers())`.

## 3. Forces & Constraints

- **POL-002** (required) — shipped product must not load-bear on host
  conventions; a convention may *inform* a default, never *carry* correctness.
- **Behaviour-preservation gate** — existing `[verification].command` (VT)
  resolution must not change; existing coverage suites stay green unchanged.
- **ADR-001** — pure leaf / impure shell layering; no clock/disk/rng/git/process
  in the pure layer.
- **STD-001** — no magic strings; default argv literals are named constants.
- **DRY / no parallel implementation** — one `dtoml` reader; one shipped-surface
  lint family (`tests/e2e_no_baked_paths.rs` precedent); do not fork the config
  parser or the spawn path.
- **SPEC-013 / SPEC-010** (concerns) — the verb sits beside the verification
  config surface; it must not perturb VT coverage semantics.

## 4. Guiding Principles

- Source correctness from an **owned contract**, not a sniffed host marker.
- Defaults are pure *data* (argv literals) — they inform, they never gate.
- Three cadences, one verb: edit (`quick`) / commit (`commit`) / phase (`gate`).
- Map skill sites by **cadence**, not by preserving the incidental `just check`.

## 5. Proposed Design

### 5.1 System Model

```
doctrine check {quick|commit|gate} [-p PATH]
        │
   cli.rs::dispatch ──► commands/check.rs::dispatch
        │                   │ root::find(cmd.path()) ──► coverage_store::load_config (THE reader)
        │                   │ verify::resolve_check(cfg, kind) ── PURE ──► CheckPlan
        │                   ▼
        │              match plan:
        │                Noop(note)  ─► println + exit 0           (owned; no spawn)
        │                Empty(kind) ─► keyed error
        │                Run(argv)   ─► run_proxy: spawn (inherit stdio) ─► wait ─► exit 128+signo|code
        ▼
   guard.rs::write_class ──► Read   (writes no authored doctrine state)
```

### 5.2 Interfaces & Contracts

**Config (owned), `.doctrine/doctrine.toml`** — three optional override keys; the
VT `command` key is untouched:

```toml
[verification]
command = [...]              # VT base argv — FROZEN semantics (not read by `check`)
quick   = ["just","check"]  # doctrine check quick  (override)
commit  = ["just","check"]  # doctrine check commit (override)
gate    = ["just","gate"]   # doctrine check gate   (override)
```

**`verify.rs` (pure leaf):**

```rust
// on VerificationConfig:
quick:  Option<Vec<String>>,
commit: Option<Vec<String>>,
gate:   Option<Vec<String>>,

// named defaults (STD-001). `quick` has NO argv default — its unconfigured path
// is an OWNED no-op (CR-F3), so no host binary is named there.
const DEFAULT_COMMIT:   &[&str] = &["just", "check"];
const DEFAULT_GATE:     &[&str] = &["just", "gate"];
const QUICK_UNSET_NOTE: &str    = "doctrine check quick: no [verification].quick set — skipping";

pub(crate) enum CheckKind { Quick, Commit, Gate }

/// What the shell should do for a kind. The `Noop` arm keeps the `quick`
/// unconfigured path OWNED — doctrine prints + exits 0 itself, never proxies a
/// host `echo` (CR-F3, POL-002). `Empty` carries a configured-but-empty override
/// (CR-F2) so the shell errors toward the key instead of spawning nothing.
pub(crate) enum CheckPlan {
    Run(Vec<String>),   // spawn this argv
    Noop(&'static str), // print note, exit 0 — NO spawn (quick, unconfigured)
    Empty(CheckKind),   // override is `[]` — keyed error, never an empty spawn
}

/// PURE, total over (cfg, kind):
///   override Some(v) non-empty → Run(v)
///   override Some([])          → Empty(kind)            (CR-F2)
///   override None, Quick       → Noop(QUICK_UNSET_NOTE) (CR-F3, owned)
///   override None, Commit      → Run(DEFAULT_COMMIT)
///   override None, Gate        → Run(DEFAULT_GATE)
pub(crate) fn resolve_check(cfg: &VerificationConfig, kind: CheckKind) -> CheckPlan;
```

**`commands/check.rs` (impure shell):**

```rust
pub(crate) fn dispatch(cmd: CheckCommand) -> anyhow::Result<()> {
    let root = crate::root::find(cmd.path(), &crate::root::default_markers())?; // -p/--path (CR-F6)
    let cfg  = crate::coverage_store::load_config(&root)?;   // the ONE reader (DRY)
    match crate::verify::resolve_check(&cfg, cmd.into()) {
        CheckPlan::Noop(note)  => { println!("{note}"); std::process::exit(0) }
        CheckPlan::Empty(kind) => anyhow::bail!(
            "[verification].{} is empty — set a non-empty argv in .doctrine/doctrine.toml",
            kind.key()),
        CheckPlan::Run(argv)   => run_proxy(&root, &argv),  // diverges via process::exit
    }
}
```

**`cli.rs`:** `Command::Check { command: CheckCommand }`; `enum CheckCommand {
Quick { path }, Commit { path }, Gate { path } }` each carrying `-p/--path`
(CR-F6), with `path()` and `From<CheckCommand> for CheckKind` accessors.

### 5.3 Data, State & Ownership

- `check` writes **no authored doctrine state** → `guard::write_class(
  Command::Check{..}) => Read` (pass-through under worker-mode). The guard gates
  *doctrine-mediated authored writes*, not filesystem mutation: a proxied command
  that mutates source (e.g. `cargo fmt`) is a **worker-legal source delta**, not
  an authored write — and a dispatch worker running `doctrine check gate` to
  verify its fork is the intended use, so `Read` is both correct and *necessary*.
- The child process owns its own stdout/stderr (inherited fds). `doctrine` owns
  only the spawn + the forwarded exit code.
- No new config reader — rides `coverage_store::load_config`. The three new
  fields deserialize on the existing `VerificationConfig` (`#[serde(default)]`,
  kebab-case), so an absent `[verification]` → all `None` → defaults.

### 5.4 Lifecycle, Operations & Dynamics

`run_proxy(root, argv)` — only ever reached with a non-empty `argv` (the `Empty`
/ `Noop` arms are handled in `dispatch` before this point):

1. `Command::new(program).args(args).current_dir(root)` — **inherit** stdout /
   stderr / stdin (live stream; *not* piped — opposite of `run_argv`). **No
   timeout** (interactive dev gate, not a capped VT run).
2. `.spawn()`:
   - `Err(ENOENT)` (default `just` absent on a client) → actionable error
     naming the owned key: *"`<program>` not found — set `[verification].<kind>`
     in `.doctrine/doctrine.toml`"* (OQ-3 → D3).
   - other spawn error → propagated with context.
3. `child.wait()` → `std::process::exit(exit_code(status))`. Diverges; never
   returns to `cli::dispatch`. (`process::exit` is safe — stdio is inherited,
   nothing buffered/owned to flush. Proxy precedent: rtk.)

```rust
// True exit forwarding (CR-F5): a signal-killed child re-exits 128+signo (shell
// convention), not a flattened `1`. Unix-only branch; doctrine targets linux/nixos.
fn exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() { return code; }
    #[cfg(unix)] { use std::os::unix::process::ExitStatusExt;
        status.signal().map(|s| 128 + s).unwrap_or(1) }
    #[cfg(not(unix))] { 1 }
}
```

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** `command` (VT base) resolution is byte-for-byte unchanged → VT
  behaviour-preservation. The three new fields are read **only** by
  `resolve_check`, never by `verify::resolve`.
- **INV-2** `resolve_check` is total — every (cfg, kind) yields a `CheckPlan`. A
  `Run` argv is non-empty **by construction** (defaults are non-empty; a
  configured `[]` routes to `Empty(kind)`, never `Run([])` — CR-F2). `run_proxy`
  therefore never sees an empty argv.
- **EDGE** configured override `[]` → `Empty(kind)` → keyed error (not a silent
  no-op, not an empty spawn).
- **EDGE** child killed by signal (`code() == None`) → exit `128 + signo` on unix
  (CR-F5); `1` on non-unix.
- **EDGE** `quick` unconfigured → **owned** no-op: doctrine prints `QUICK_UNSET_
  NOTE` and exits 0 with **no** child spawn (CR-F3); never fails a per-edit hook,
  never load-bears on a host `echo`.
- **ASSUMPTION** `.agents/` is generated, gitignored; only `plugins/**` is
  authored (verified via `git ls-files`).

## 6. Open Questions & Unknowns

All resolved in design conversation:

- **OQ-1 (config key shape)** → **D1**. Three explicit keys under the existing
  `[verification]` table; `command` frozen.
- **OQ-2 (informing defaults)** → yes; defaults are argv literals (POL-002).
- **OQ-3 (absent-command behaviour)** → **D3**. Baked default spawns; spawn
  `ENOENT` → actionable error naming the owned key. No host-marker sniff (a
  sniff would itself be the POL-002 coupling).

## 7. Decisions, Rationale & Alternatives

- **D1 — three keys under `[verification]`, `command` frozen.** The VT-evidence
  base and the dev check altitudes are distinct concerns; conflating them would
  break clients whose test command ≠ commit gate, and risk the
  behaviour-preservation gate. *Alt:* reuse `command` for `gate` (rejected:
  couples concerns); a new `[check]` table (rejected: parallel surface, the
  reader/`[verification]` already owns "how this project runs checks").
- **D2 — three cadences `quick`/`commit`/`gate`.** Matches observed practice
  (edit / commit / phase). Middle named `commit` (not `check`) to avoid the
  `doctrine check check` token collision. *Alt:* two altitudes (rejected: loses
  the per-edit vs per-commit distinction the user runs in practice).
- **D3 — default spawns, `ENOENT` → keyed error.** The default is pure data; on
  absence we error toward the owned key, never limp or sniff. *Alt:* marker-gated
  default (rejected: marker detection is a host-convention sniff — the very
  POL-002 facet-1 coupling); no default at all (rejected: contradicts OQ-2, hurts
  home ergonomics).
- **D4 — `quick` default is an informative no-op echo.** Per-edit cadence must
  never fail unconfigured; the echo tells the dev *why* nothing ran.
- **D5 — inherit stdio, no timeout.** A dev gate streams live and may legitimately
  run long; do **not** ride `run_argv`'s pipe+capture+cap path (wrong posture).
- **D6 — sweep maps by cadence, with TWO treatments (refined per CR-F4).** Not a
  blind grep-replace — the six `just check` sites split:
  - **(a) Instruction rewrites → `doctrine check gate`** (4 sites): `execute:50`,
    `close:33`, `audit:109`, `notes:32`. These genuinely *are* phase/close-boundary
    gate instructions; the agent should run the gate. On this repo this swaps
    `just check` → `just gate`, aligning the skills with this repo's own "`just
    gate` before every commit" rule (argv client-configured regardless).
  - **(b) Illustrative-example updates, semantics PRESERVED** (2 worktree sites):
    `worktree:192` ("this repo: `just check`") and `worktree:219` ("not assumed
    `just check`"). These are **not** fixed-gate instructions — :192 keeps "run
    the **project-provided** regenerate-and-verify command, never a hardcoded
    `cargo …`" and :219 keeps "the worker runs the **orchestrator-supplied**
    verify command." Only the illustrative `just check` token is updated to
    `doctrine check gate` as the *example*; the don't-hardcode / caller-control
    semantics are untouched. (CR-F4 flagged that flattening these to a fixed gate
    would erase intentional dispatch caller-control — rejected.)
  - `quick`/`commit` ship as configured altitudes with no shipped-skill caller
    (the inner-loop cadences documented in client `AGENTS.md`, not these skills).

## 8. Risks & Mitigations

- **R1 — sweep behaviour change** (`just check` → `just gate` on this repo).
  *Mitigation:* explicit sign-off obtained; semantically aligns with this repo's
  commit-gate rule; argv is client-configurable.
- **R2 — `process::exit` skips destructors.** *Mitigation:* nothing owned needs
  flushing (stdio inherited); confined to the verb's terminal step.
- **R3 — slice §3 overstates scope** (names `just gate` skill sites; none exist).
  *Mitigation:* reconcile slice scope to the actual six `just check` sites.
- **R4 — new `[verification]` keys break VT parse.** *Mitigation:* INV-1 +
  existing `VerificationConfig` round-trip unit tests stay green unchanged.
- **R5 — typed keys flip unknown→parse-error for a differently-typed client key**
  (CR-F1). Claiming `quick`/`commit`/`gate` as typed `Option<Vec<String>>` means a
  client that *happened* to carry e.g. `quick = "cargo test"` under `[verification]`
  would flip from silently-ignored to a hard parse error across all config-reading
  commands. *Disposition:* **accepted / moot** — no client projects exist yet
  (single-repo reality), and `[verification]` is a doctrine-**owned** table so
  claiming names is legitimate. Revisit if/when external clients adopt (a
  tolerant-parse or `deny_unknown_fields`-aware migration note).

## 9. Quality Engineering & Validation

- **Unit (`verify.rs`, pure):** `resolve_check` — for each of {`quick`, `commit`,
  `gate`}: override-present → `Run(override)`; override-absent → the kind's plan
  (`Quick`→`Noop`, `Commit`/`Gate`→`Run(default)`); **override `[]` → `Empty(kind)`**
  (CR-F2). Asserts exact plan/argv. `CheckPlan` is pure-returned, so the owned
  no-op and the empty-error are unit-covered without spawning.
- **Unit (`verify.rs`):** existing `VerificationConfig` parse tests extended —
  the three new keys deserialize; an absent table still yields all-`None`;
  `command` unchanged (INV-1).
- **E2E (`tests/e2e_check_proxy.rs`, new):** built binary against a temp root
  (via `-p/--path`). (i) `[verification].gate = ["sh","-c","exit 7"]` → assert
  **exit 7** + streamed child output; (ii) bogus program → assert error **names
  the key**; (iii) **signal case** `["sh","-c","kill -TERM $$"]` → assert exit
  **143** (`128+SIGTERM`, CR-F5); (iv) `quick` unconfigured → assert exit 0 +
  the note on stdout + **no** child spawned. (Covers `process::exit` forwarding,
  untestable in-process — the `e2e_*` precedent.)
- **Shipped-surface guard (`tests/e2e_no_shipped_couplings.rs`, new):** scans
  `plugins/**` — no `just check` / `just gate`, no bare `mem_…` uid. Rides the
  `e2e_no_baked_paths.rs` pattern (needles assembled from fragments so the guard
  does not self-match).
- **Gate:** `just gate` green on this repo.

## 10. Review Notes

### Internal adversarial pass

- **A1 — `Read` classification vs source-mutating proxied commands.** `check` can
  spawn `cargo fmt` (mutates the tree) yet is `Read`. *Resolved:* `write_class`
  guards doctrine-mediated **authored** writes under worker-mode, not filesystem
  mutation. Source mutation is a worker-legal source delta; `Read` is correct and
  necessary (workers run `doctrine check gate` to verify forks). §5.3 reworded.
- **A2 — two enums (`CheckCommand`, `CheckKind`).** Intentional layering, not a
  parallel impl: the `verify` leaf must not depend on `clap` (ADR-001), so `cli`
  owns the clap-derive `CheckCommand` and bridges via `From` to the leaf
  `CheckKind`. Documented at §5.2.
- **A3 — `commit` default `just check` vs this repo's "`just gate` before every
  commit" (AGENTS.md).** Intentional: defaults follow the user's stated three-tier
  cadence (edit/commit/phase), not AGENTS.md's two-tier habit; defaults only
  *inform* (POL-002) and are client-overridable.
- **A4 — `quick`/`commit` have no shipped-skill caller post-sweep.** YAGNI tension
  considered, overridden by explicit user practice ("in practice I often end up
  with 3"). The verb is a general altitude surface invoked directly per client
  `AGENTS.md`, not only skill-internal. All six skill sites are phase-boundary →
  `gate` (D6).
- **A5 — no timeout.** A hung proxied check hangs the agent. Accepted: identical
  to running the command directly; the harness interrupts. A configurable timeout
  is a possible follow-up, out of scope (the VT 300s cap stays VT-only).
- **A6 — `run_proxy` diverges (`process::exit`); `dispatch` returns `Ok` only on
  the never-taken success path, `Err` on spawn failure.** Accepted proxy shape;
  implementer guards against a clippy unreachable lint.

### External review (codex / GPT-5.5, hostile pass)

Six findings; all legitimate, all integrated:

- **CR-F1 (MAJOR) — typed keys flip unknown→parse-error.** → **accepted/moot**:
  no clients yet, owned table. Recorded as R5 (§8).
- **CR-F2 (MAJOR) — INV-2 false; `quick = []` empty override reachable.** → fixed:
  `CheckPlan::Empty(kind)` → keyed error; INV-2 reworded; unit + edge added.
- **CR-F3 (MAJOR) — `echo` no-op still load-bears on host `echo` (POL-002).** →
  fixed: `quick` unconfigured is an **owned** `CheckPlan::Noop` (doctrine prints +
  exits 0, no spawn). No host binary named on the default path.
- **CR-F4 (MAJOR) — D6 sweep grep-driven; worktree:192/:219 aren't fixed gates.**
  → fixed: D6 split into (a) 4 instruction-rewrites and (b) 2 worktree
  illustrative-example updates that preserve project-provided / orchestrator-
  supplied caller-control semantics.
- **CR-F5 (MAJOR) — signal death squashed to `1`, not forwarded.** → fixed:
  `exit_code()` returns `128 + signo` on unix; e2e signal case (exit 143) added.
- **CR-F6 (MINOR) — `-p/--path` unspecified.** → fixed: each `CheckCommand`
  variant carries `-p/--path`, threaded to `root::find`; aids e2e temp-root.
