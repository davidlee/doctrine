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
doctrine check {quick|commit|gate}
        │
   cli.rs::dispatch ──► commands/check.rs::dispatch
        │                   │ root::find ──► coverage_store::load_config (THE reader)
        │                   │ verify::resolve_check(cfg, kind) ── PURE ──► argv
        │                   ▼
        │              run_proxy(root, argv): spawn (inherit stdio) ─► wait ─► process::exit(code)
        ▼
   guard.rs::write_class ──► Read   (writes no doctrine state)
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

// named defaults (STD-001):
const DEFAULT_QUICK:  &[&str] = &["echo", "doctrine check quick: no [verification].quick set — skipping"];
const DEFAULT_COMMIT: &[&str] = &["just", "check"];
const DEFAULT_GATE:   &[&str] = &["just", "gate"];

pub(crate) enum CheckKind { Quick, Commit, Gate }

/// PURE, total: configured override → else the kind's baked default argv.
pub(crate) fn resolve_check(cfg: &VerificationConfig, kind: CheckKind) -> Vec<String>;
```

**`commands/check.rs` (impure shell):**

```rust
pub(crate) fn dispatch(cmd: CheckCommand) -> anyhow::Result<()> {
    let root = crate::root::find(None, &crate::root::default_markers())?;
    let cfg  = crate::coverage_store::load_config(&root)?;   // the ONE reader (DRY)
    let argv = crate::verify::resolve_check(&cfg, cmd.into());
    run_proxy(&root, &argv)                                  // diverges via process::exit
}
```

**`cli.rs`:** `Command::Check { command: CheckCommand }`; `enum CheckCommand {
Quick, Commit, Gate }` (`From<CheckCommand> for CheckKind`).

### 5.3 Data, State & Ownership

- `check` **reads** config; **writes nothing** to doctrine state (authored /
  runtime / derived). `guard::write_class(Command::Check{..}) => Read`.
- The child process owns its own stdout/stderr (inherited fds). `doctrine` owns
  only the spawn + the forwarded exit code.
- No new config reader — rides `coverage_store::load_config`. The three new
  fields deserialize on the existing `VerificationConfig` (`#[serde(default)]`,
  kebab-case), so an absent `[verification]` → all `None` → defaults.

### 5.4 Lifecycle, Operations & Dynamics

`run_proxy(root, argv)`:

1. `argv.split_first()` → `(program, args)`; empty argv is unreachable (defaults
   are non-empty) but guarded → keyed error.
2. `Command::new(program).args(args).current_dir(root)` — **inherit** stdout /
   stderr / stdin (live stream; *not* piped — opposite of `run_argv`). **No
   timeout** (interactive dev gate, not a capped VT run).
3. `.spawn()`:
   - `Err(ENOENT)` (default `just` absent on a client) → actionable error
     naming the owned key: *"`<program>` not found — set `[verification].<kind>`
     in `.doctrine/doctrine.toml`"* (OQ-3 → D3).
   - other spawn error → propagated with context.
4. `child.wait()` → `std::process::exit(status.code().unwrap_or(1))`. Diverges;
   never returns to `cli::dispatch`. (`process::exit` is safe — stdio is
   inherited, nothing buffered/owned to flush. Proxy precedent: rtk.)

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** `command` (VT base) resolution is byte-for-byte unchanged → VT
  behaviour-preservation. The three new fields are read **only** by
  `resolve_check`, never by `verify::resolve`.
- **INV-2** `resolve_check` is total (every kind yields a non-empty argv).
- **EDGE** child killed by signal (`code() == None`) → exit `1`.
- **EDGE** `quick` unconfigured → informative-echo no-op, exit 0 (never fails a
  per-edit hook).
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
- **D6 — sweep maps by cadence → all six sites to `gate`.** They are all
  phase/close-boundary instructions. On *this* repo this swaps `just check` →
  `just gate`; that aligns the skills with this repo's own stated "`just gate`
  before every commit" rule, and the argv is client-configured regardless.
  `quick`/`commit` ship as configured altitudes with no shipped-skill caller
  (they are the inner-loop cadences documented in client `AGENTS.md`).

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

## 9. Quality Engineering & Validation

- **Unit (`verify.rs`, pure):** `resolve_check` — override-present and
  absent-default, × {`quick`, `commit`, `gate`} (6 cases). Asserts the exact argv
  (incl. the informative-echo default).
- **Unit (`verify.rs`):** existing `VerificationConfig` parse tests extended —
  the three new keys deserialize; an absent table still yields all-`None`;
  `command` unchanged (INV-1).
- **E2E (`tests/e2e_check_proxy.rs`, new):** built binary against a temp root.
  `[verification].gate = ["sh","-c","exit 7"]` → assert **exit 7** + streamed
  child output; a bogus program → assert error **names the key**. (Covers
  `process::exit` forwarding, untestable in-process — the `e2e_*` precedent.)
- **Shipped-surface guard (`tests/e2e_no_shipped_couplings.rs`, new):** scans
  `plugins/**` — no `just check` / `just gate`, no bare `mem_…` uid. Rides the
  `e2e_no_baked_paths.rs` pattern (needles assembled from fragments so the guard
  does not self-match).
- **Gate:** `just gate` green on this repo.

## 10. Review Notes

(Internal adversarial pass + external codex review recorded here.)
