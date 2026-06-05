# Design SL-011: Cache-friendly session boot context

## 1. Design Problem

Every doctrine session re-pays to learn project governance. ADR list, memory
pointers, the routing/process digest, and project-local "articles of truth" are
**stable governance state** the agent rediscovers via Read/Bash tool calls,
turn-by-turn, uncached, every session. Adapt spec-driver's "preboot" idea: emit
a governance snapshot into the agent's **cacheable session-start prefix** so it
costs zero tool calls and only busts cache when governance changes.

Constraints that shape the whole design:

- **Verified Claude Code behaviour (2026).** Only `CLAUDE.md`/`AGENTS.md` and
  their `@`-imports are inlined into the cached prefix at session start (≤4 hops,
  resolved before turn 1). `.claude/rules/` is **not** a dependable cached loader
  (lazy/path-scoped); skill bodies are lazy; SessionStart-hook `additionalContext`
  is a separate **non-cached** message. → the snapshot must ride an `@`-import,
  and the hook must only **regenerate the file**, never push content via stdout.
- **Content-diff is the cache key.** Regeneration must write only on change.
- **Loose coupling (R2).** Vendor-specific wiring is not *core*. Claude is one
  harness among others, behind a seam — never privileged in core logic.

## 2. Current State

- `doctrine install` (`src/install.rs`): pure `Step` planner + `execute`; copies
  the `install/` embed → `.doctrine/`, **create-if-missing** (`Step::Skip` when a
  dest exists), additive `.gitignore` via the shared `ensure_gitignored` seam.
  **Never touches `.claude/`, root `CLAUDE.md`/`AGENTS.md`, symlinks, or hooks.**
- `doctrine skills install` (`src/skills.rs`): pure planner + `Runner` seam; owns
  `.claude/skills/` relative symlinks by **ownership-by-target-equality +
  never-clobber** (SL-010). Shares `ensure_gitignored`.
- Root `CLAUDE.md` is a **symlink to `AGENTS.md`** (one inode); AGENTS.md hand-
  recites the full CLI, the `just list-memories` nag, and the process — all
  governance an agent re-reads each session.
- No boot/preboot mechanism exists. `doctrine` is **not on PATH** in dev
  (`./target/debug/doctrine`).

## 3. Forces & Constraints

- Cache discipline: ride the `@`-import (cached), not `.claude/rules`; write only
  on content change.
- Pure/imperative split (house rule): no clock/rng/git/disk in the pure layer.
- Never-clobber user-owned files (settings, root MD) — SL-010 ownership ethos
  applied to structured JSON + prose.
- Behaviour-preservation gate: reusing `adr`/`memory` listing logic must keep
  their existing suites green.
- **Concurrency:** slice-012 (`memory-record-symlink-tolerance`) is editing
  `src/skills.rs` and memory-record concurrently. Keep all new logic in
  `src/boot.rs`; any shared-file touch (`adr.rs`/`memory.rs`) must be **additive**.

## 4. Guiding Principles

- One agent-neutral snapshot; per-harness wiring behind a seam.
- Governance lives **once**, in the cached prefix; AGENTS.md shrinks to durable
  orientation the snapshot can't carry.
- The snapshot is a **pure projection** — never an authoritative store.
- Vendor-specific ≠ core. Claude is a `Harness` impl, not a special case.

## 5. Proposed Design

### 5.1 System Model

New module **`src/boot.rs`**. Two verbs; `install.rs`/`skills.rs` untouched:

- `doctrine boot [-p ROOT]` — regenerate `.doctrine/state/boot.md`. The hook
  target: fast, content-diffed, resolves its own `current_exe()`.
- `doctrine boot install [-p ROOT] [--dry-run] [--yes]` — wire the `@`-import and
  the per-harness session refresh.

Core produces the neutral snapshot and iterates **selected harnesses**; it never
names Claude. Pure assembler + declared sequence; impure shell gathers section
bodies, reads/writes disk, resolves `current_exe()`.

### 5.2 Interfaces & Contracts

```rust
// ---- pure: snapshot assembly (the extensible seam) ----
struct Section { heading: String, body: String }
enum SourceKind { ExecPath, Static(&'static str), Governance, Adrs, Memories }
//   future: Policies, Standards — append to the sequence, no other change.
fn boot_sequence() -> Vec<(&'static str, SourceKind)>   // declared heading+source order
fn render_boot(sections: &[Section]) -> String          // header + concat; deterministic

// ---- impure shell: produce each body ----
fn produce(kind: &SourceKind, root: &Path, exec: &Path) -> Section
//   miss/err → benign `<!-- … -->` marker body, never a crash.
fn write_if_changed(path: &Path, content: &str) -> Result<bool>  // wrote? (cache-key rule)

// ---- the harness seam (R2) ----
enum RefreshOutcome { Wired(String), Refreshed(String), PrintedFallback, None }
trait Harness {
    fn id(&self) -> &str;                                   // "claude", "codex"
    fn import_targets(&self, root: &Path) -> Vec<PathBuf>;  // CLAUDE.md / AGENTS.md
    fn install_refresh(&self, root: &Path, exec: &Path) -> Result<RefreshOutcome>;
}
fn registry() -> Vec<Box<dyn Harness>>                     // static, compile-time
fn resolve_harnesses(explicit: &[String], root: &Path) -> Result<Vec<Box<dyn Harness>>>

// ---- shared wiring primitives (harness-agnostic) ----
enum RefOutcome { Added(PathBuf), Present(PathBuf), Created(PathBuf) }
fn ensure_boot_import(targets: &[PathBuf], reference: &str) -> Result<Vec<RefOutcome>>
//   idempotent prepend; create-if-missing; dedup targets by `canonicalize` (one
//   inode → one write — the CLAUDE.md→AGENTS.md symlink case).

// ---- additive reuse (behaviour-preserving) ----
// adr.rs:    fn list_rows(root) -> Result<String>   (CLI list prints it; tests stay green)
// memory.rs: fn list_rows(root, filter) -> Result<String>   (additive; avoid 012 clash)
```

`boot install` flow: `resolve_harnesses` → **union** their `import_targets` →
dedup by `canonicalize` → `ensure_boot_import` **once** → each
`harness.install_refresh(root, exec)`.

### 5.3 Data, State & Ownership

| Artifact | Tier | Ownership rule |
|---|---|---|
| `.doctrine/state/boot.md` | derived (runtime state) | regenerated; inherits `.doctrine/state/` ignore |
| `@.doctrine/state/boot.md` ref in `CLAUDE.md`/`AGENTS.md` | authored (committed) | idempotent prepend; symlink/inode-dedup; never duplicate |
| SessionStart hook in `.claude/settings.local.json` | personal (gitignored) | auto-merge, own-by-pattern, never-clobber, fail-soft |
| `.doctrine/governance.md` | authored (user-owned) | seed-if-missing via install Skip; never clobbered |

- **Snapshot format:**
  ```
  <!-- Generated by `doctrine boot` — do not edit; regenerated each session -->
  # Doctrine Boot Context

  ## Invoking doctrine        ← R1: absolute current_exe() path (off-PATH reliability)
  ## Routing & Process        ← embedded digest asset(s) (Q4-A)
  ## Governance (project)     ← .doctrine/governance.md body, or marker if absent
  ## Accepted ADRs            ← adr::list_rows (accepted), or "none"
  ## Memory                   ← memory::list_rows / pointers
  ```
- **Exec path (R1).** `doctrine boot` resolves `current_exe()` and emits it as the
  "Invoking doctrine" body, so the agent can call the CLI reliably off-PATH. The
  **same** resolved path is what the Claude harness bakes into the hook (single
  source). Machine-specific path in a gitignored derived file → acceptable; stable
  per machine → no spurious cache bust.
- **`@`-import** → committed root files (portable text). **Symlink-aware:**
  `canonicalize` the targets; same inode → one write.
- **Per-harness import targets (review fix #1).** Claude `import_targets =
  [CLAUDE.md]` **only**; codex `= [AGENTS.md]`. The guide confirmed CLAUDE.md
  inlines `@`-imports; AGENTS.md inlining is unconfirmed (open question). If an
  agent read *both* files each carrying the ref, the whole snapshot would inline
  **twice** (double prefix cost) — so each harness claims exactly one file. This
  repo's `CLAUDE.md→AGENTS.md` symlink makes the two one inode → the union dedups
  to a single write naturally.
- **Harness detection.** `resolve_harnesses`: explicit `--agent` wins; else
  auto-detect by marker (`.claude/` → claude, `.codex/` or an `AGENTS.md` without
  a `.claude/` → codex). At least one required, else error (mirrors
  `skills::resolve_agents`).
- **SessionStart hook** → gitignored `settings.local.json` (machine-specific exec
  path belongs out of git). `serde_json` merge: find a SessionStart entry **owned
  by doctrine** → refresh its command; else append. **Ownership match (review fix
  #4):** the command's path `file_name` is `doctrine` *and* it ends with ` boot` —
  tighter than a loose substring so a user's unrelated hook is never hijacked.
  Entry: `{matcher:"startup|clear", hooks:[{type:"command", command:"<abs exec>
  boot"}]}` — **no `async` key (review fix #2):** the hook must block so
  regeneration completes before the prefix loads. Malformed JSON → never clobber;
  print the snippet (`PrintedFallback`).
- **`governance.md`** → ship `install/governance.md`; `doctrine install` seeds it
  via the existing Skip-if-exists path (no install.rs change). This repo adds
  `!.doctrine/governance.md` to its `.gitignore` for dogfooding; downstream commits
  it normally (manifest writes additive ignores, not the blanket).

### 5.4 Lifecycle, Operations & Dynamics

Session start (Claude) → hook runs `doctrine boot` → `write_if_changed`:
unchanged governance ⇒ byte-identical ⇒ cache holds; changed ⇒ cache busts (correct).
First session before any generation: `@`-import resolves empty (benign), filled
next start. `/canon` treats `governance.md` as already-in-context (Q5-A); notes
"re-run `doctrine boot` if you just edited it". `/route` gains a one-line
presence check (warn + point at `doctrine boot` if the heading is absent).
**codex**: AGENTS.md `@`-import only, **no hook** → refresh on next `doctrine`
run; staleness window accepted, **verified live as a closure gate**.

### 5.5 Invariants, Assumptions & Edge Cases

- Snapshot is a pure projection — never authoritative.
- `render_boot` is deterministic (no clock/rng) → content-diff is stable.
- `.doctrine/state/` created before write.
- Never double-prepend the ref to two views of one inode; never let two read-files
  each carry the ref for the *same* agent (review fix #1).
- **The boot hook is synchronous/blocking and `doctrine boot` is fast** (review
  fix #2) — regeneration must finish before the prefix loads, else the agent reads
  a one-session-stale snapshot. (Contrast: an artifact-event hook would be async;
  the boot hook must not be.)
- `install_refresh` failure for one harness must not abort the others.
- Foreign SessionStart hooks (user's own) are preserved; only the doctrine-owned
  entry (path basename `doctrine`, ` boot` suffix) is touched.
- `produce()` tolerates a missing `Static` digest asset with a benign marker, so
  `doctrine boot` works before the digest is authored (review fix #3 — breaks the
  phase-2/phase-6 ordering knot).
- Reinstall re-adds a deliberately-removed ref (accepted: it is an install verb).

## 6. Open Questions & Unknowns

- **codex `@`-import semantics** — does codex inline AGENTS.md `@`-imports into
  its system prompt the way Claude does? No SessionStart equivalent → staleness.
  **Resolved by the live codex run** in the closure phase, not by assumption.
- **Hook matcher tokens** — confirm Claude's `SessionStart` matcher accepts
  `startup|clear` as specified (research doc asserts it; verify against the live
  harness when wiring phase 4).
- **`memory::list_rows` shape** — keep additive to avoid the slice-012 clash;
  exact signature confirmed at phase 2 against the then-current `memory.rs`.

## 7. Decisions, Rationale & Alternatives

- **D1 New `src/boot.rs` (not extend install/skills).** Isolates a new concern;
  dodges the concurrent `skills.rs` edits; keeps the two installers stable. (Q1-B)
- **D2 `.doctrine/state/boot.md` + `@`-import.** Snapshot *is* runtime state;
  inherits the existing ignore, zero new rules. `.claude/rules` rejected — not a
  dependable cached loader. (Q2-A)
- **D3 Auto-merge settings, never-clobber, fail-soft to print.** Only posture
  giving automatic per-session regen; bounded JSON merge by command-pattern. (Q3-A→C)
- **D4 Ship an authored boot digest in the embed.** Governance lives once in the
  cached prefix; AGENTS.md genuinely shrinks. (Q4-A)
- **D5 `governance.md`, seed-if-missing, consumed from the snapshot.** Cached,
  zero-cost; rarely changes; regenerated each session. (Q5-A)
- **D6 Bake `current_exe()` into the hook; hook in `settings.local.json`.**
  Machine-specific path → gitignored local settings; committed `@`-import stays
  portable. (Q6-B)
- **D7 (R1) Emit the exec path into the snapshot** so the agent invokes the CLI
  off-PATH. Single source with the hook path.
- **D8 (R2) `Harness` seam; Claude is one adapter.** Vendor-specific wiring is not
  core. Static compile-time registry (Claude full, codex import-only); a third
  harness is a new impl, no core edit. **Not** dynamic plugin loading (YAGNI).

## 8. Risks & Mitigations

- **Concurrency on `skills.rs`/`memory.rs`** → all new code in `boot.rs`;
  `adr`/`memory` touches additive only.
- **codex behaviour unknown** → closure-gate live run; degrade gracefully if the
  import doesn't inline.
- **Malformed user `settings.local.json`** → fail-soft, print, never clobber.
- **Prefix-budget duplication** → AGENTS.md rewrite removes what the snapshot now
  carries; net prefix cost roughly flat, tool-call cost drops.

## 9. Quality Engineering & Validation

Pure: `render_boot` determinism + section order; missing-source marker;
`boot_sequence` carries the extensible kinds. `write_if_changed`: writes on
change, no-op when equal, returns the bool. `ensure_boot_import`: prepend-once
idempotent, create-missing, **symlink/same-inode → exactly one write**, preserves
existing content. Claude `install_refresh`: merge into empty/missing, **refresh an
existing doctrine entry on path change**, **preserve foreign hooks**, **fail-soft
on malformed JSON**. `adr`/`memory` `list_rows`: existing CLI suites stay green.
Integration: `doctrine boot` emits the headings + ADR rows + the exec path.
Closure: **live codex session** confirms the snapshot loads and routing is honoured.

## 10. Review Notes

### Internal adversarial pass (round 1)

- **#1 Double-load / wrong-file.** Guide confirmed CLAUDE.md inlines `@`-imports;
  AGENTS.md unconfirmed. Two ref-bearing files read by one agent ⇒ snapshot
  inlines twice. **Resolved:** Claude `import_targets=[CLAUDE.md]`, codex
  `=[AGENTS.md]`; symlink dedups this repo to one write. (§5.3, §5.5)
- **#2 Async hook = stale read.** A non-blocking hook lets the prefix load before
  regeneration finishes. **Resolved:** boot hook is synchronous (no `async` key),
  `doctrine boot` is fast; verification asserts no `async:true`. (§5.3, §5.5, §9)
- **#3 Phase-2/phase-6 ordering.** `doctrine boot` renders the `Static` digest
  before the digest asset is authored. **Resolved:** `produce()` benign-marker
  tolerance — no hard dependency. (§5.5)
- **#4 Loose ownership match.** `*doctrine*boot*` could hijack a user hook.
  **Resolved:** match path basename `doctrine` + ` boot` suffix. (§5.3)
- **#5 `current_exe()` under wrappers** (cargo/nix) — accepted for v1, flagged. (§6)

### External coordination (SL-012 concurrency)

SL-012 landed mid-design: retired the `doc/memories/` bargain-bin, ported notes
into the `doctrine memory` store, and edited CLAUDE.md/AGENTS.md. Impact on this
slice: (a) the **Memory** snapshot section is now a clean projection of the
canonical store (no bargain-bin duality); (b) the AGENTS.md-rewrite objective's
"reconcile the memory story" is largely **subsumed by SL-012** — the rewrite
narrows to CLI-recital removal + delegating routing/process to the snapshot.

### Next

Offer `/inquisition` (formal hostile pass) or proceed to `/plan`.
