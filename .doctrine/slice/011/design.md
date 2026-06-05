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

// ---- freshness/health sentry (Charge II/III) ----
fn boot_check(root: &Path, exec: &Path) -> CheckReport
//   recompute render; diff vs on-disk boot.md (stale?); tally marker-vs-populated
//   sections. DETERMINISTIC — no clock in the body: a generation TIMESTAMP in the
//   cached content would bust the cache every session (defeats §1), so freshness is
//   reported out-of-band, never embedded. `/route` interrogates `doctrine boot --check`.

// ---- the harness seam (R2) — enum+match, NOT trait+Box<dyn> (Charge IV) ----
enum RefreshOutcome { Wired(String), Refreshed(String), PrintedFallback, None }
enum Harness { Claude, Codex }                              // local id for v1
fn import_targets(h: &Harness, root: &Path) -> Vec<PathBuf>;        // CLAUDE.md / AGENTS.md
fn install_refresh(h: &Harness, root: &Path, exec: &Path) -> Result<RefreshOutcome>;
//   match on the harness (Claude = import + hook; Codex = import-only) — mirroring
//   skills.rs's own Claude-vs-Other match. One full impl + one stub do NOT earn a
//   trait + Box<dyn> + a parallel registry/resolver (YAGNI; "no parallel impl").
fn resolve_harnesses(explicit: &[String], root: &Path) -> Result<Vec<Harness>>
//   detection mirrors skills::resolve_agents. DEBT (deferred — Charge IV): once
//   SL-012 lands and skills.rs is uncontended, promote skills::Agent + resolve_agents
//   to pub(crate) and UNIFY — erasing the twin identity. Not done now: the
//   concurrency gate (§3) forbids touching skills.rs while SL-012 is in_progress.

// ---- shared wiring primitives (harness-agnostic) ----
enum RefOutcome { Added(PathBuf), Present(PathBuf), Created(PathBuf) }
fn ensure_boot_import(targets: &[PathBuf], reference: &str) -> Result<Vec<RefOutcome>>
//   idempotent prepend; create-if-missing; dedup targets by `canonicalize` (one
//   inode → one write — the CLAUDE.md→AGENTS.md symlink case).

// ---- reuse (behaviour-preserving) — NOT uniformly "additive" (Charge V) ----
// memory.rs: fn list_rows(root, filter) -> Result<String>
//   genuinely ADDITIVE — wraps existing select_rows()+format_list(); no existing
//   line touched. SL-012-contended file, but an additive touch is the allowed kind.
// adr.rs:    fn list_rows(root, status) -> Result<String>
//   behaviour-preserving EXTRACT — run_list() writes straight to io::stdout(); split
//   the compute (read_metas→sort_and_filter→meta::format_list) from the write!. Small
//   (meta::format_list already exists) but it edits run_list — and adr.rs is the
//   riskier deed though UNCONTENDED by SL-012. Guarded by the e2e suite at adr.rs:331.
```

`boot install` flow: `resolve_harnesses` → **union** their `import_targets(h, …)` →
dedup by `canonicalize` → `ensure_boot_import` **once** → each
`install_refresh(h, root, exec)`.

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

  ## Routing & Process        ← embedded digest asset(s) (Q4-A)        ┐ governance
  ## Governance (project)     ← .doctrine/governance.md body, or marker │ prefix —
  ## Accepted ADRs            ← adr::list_rows (accepted), or "none"    │ cache-stable
  ## Memory                   ← memory::list_rows / pointers           ┘
  ## Invoking doctrine        ← R1: current_exe() — LAST (build-volatile; cache tail)
  ```
- **Exec path (R1) — couples the cache key to a build-volatile value** (codex review
  finding 4). `doctrine boot` resolves `current_exe()` and emits it as the "Invoking
  doctrine" body so the agent calls the CLI off-PATH; the **same** path is baked into
  the Claude hook (single source). But the snapshot is the cache key (§1), and the
  exec path is **not** governance: it can change with no governance change. Bound the
  volatility by environment:
  - **dev (cargo):** `current_exe()` = `target/debug/doctrine` — stable across
    rebuilds (cargo overwrites in place) → no spurious bust.
  - **installed:** doctrine is on PATH → R1 is moot; emit the bare `doctrine`, not an
    absolute path → stable.
  - **off-PATH nix store binary:** the store path changes per rebuild → a rebuild is a
    legitimate (if mildly annoying) cache bust. Accepted.
  **Mitigation:** order the "Invoking doctrine" section **last** in `boot_sequence`
  so a path change invalidates only the snapshot tail, leaving the governance prefix
  cache-warm. Never let the exec path sit ahead of governance content.
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
  #4, hardened — Charge VII):** tokenise the command, require the program token's
  `file_name == doctrine` *and* the last argument `== boot` — robust to spaces in a
  resolved store/exec path (a naive ` boot`-suffix test breaks on a spaced nix path).
  A user's unrelated hook is never hijacked. Entry: `{matcher:"startup|clear",
  hooks:[{type:"command", command:"<abs exec> boot"}]}`. The `async` key is a
  **start-up latency choice, not correctness** — Charge I struck "review fix #2"
  (the hook freshens the *next* session regardless; see §5.4). The `startup|clear`
  matcher token is confirmed live at wiring (phase 4). Malformed JSON → never
  clobber; print the snippet (`PrintedFallback`).
- **`governance.md`** → ship `install/governance.md`; `doctrine install` seeds it
  via the existing Skip-if-exists path (no install.rs change). This repo adds
  `!.doctrine/governance.md` to its `.gitignore` for dogfooding; downstream commits
  it normally (manifest writes additive ignores, not the blanket).
- **`governance.md` remit (boundary — Charge VI).** It is a *pointer/digest* layer,
  not a fourth source of truth. It carries **only** the short, stable orientation an
  agent needs every session and that no existing surface already projects: cross-
  references to the authoritative sources and one-line "where things live" notes.
  It does **not** restate them. Delineation against the three existing surfaces:
  - **CLAUDE.md** owns the live build/check commands, layout, CLI surface, storage
    rule, conventions — governance.md never duplicates these (the snapshot already
    carries the routing/process digest).
  - **`doc/*`** owns evergreen authoritative specs — governance.md links, never copies.
  - **ADRs** own decisions with rationale/status — projected by the *Accepted ADRs*
    section, not re-narrated in governance.md.
  If a line belongs in CLAUDE.md, `doc/*`, or an ADR, it goes there and governance.md
  at most points at it. When in doubt, it does **not** go in governance.md.

### 5.4 Lifecycle, Operations & Dynamics

**Load order (verified — Charge I).** At session start the harness inlines
`CLAUDE.md`/`AGENTS.md` and their `@`-imports into the cached prefix **before**
SessionStart hooks run (Claude Code *prompt-caching* + *hooks* docs; confirmed via
claude-code-guide, 2026-06-05). The boot hook therefore **cannot freshen the
session that triggers it** — it regenerates `boot.md` for the *next* session. Sync
vs async is irrelevant to this (the refuted "review fix #2").

**The lag law (steady state) — bounded at TWO sessions, not one** (codex review
finding 1). Every session inlines the snapshot the *previous* session's hook wrote.
An edit's visibility depends on its timing vs the start-hook:
- edit lands **before** session N's start-hook (i.e. during N−1) → N's hook captures
  it → visible at **N+1** (best case, one-session lag);
- edit lands **during/after** session N's start-hook → only N+1's hook captures it →
  visible at **N+2** (worst case, two-session lag).
So absent the ritual, staleness is **≤ 2 sessions** (bounded — every new session's
hook regenerates, so an edit cannot outlive two session-starts). For governance that
*rarely changes* this is **acceptable** — value (zero tool calls, warm cache)
untouched; the freshen-now ritual collapses it to **zero**.

Session start (Claude) → step 2 inlines `@.doctrine/state/boot.md` as written by the
*prior* hook → step 3 hook runs `doctrine boot` → `write_if_changed`: unchanged
governance ⇒ byte-identical ⇒ cache holds; changed ⇒ *next* session busts (correct).
First-ever session: import resolves empty (benign), filled next start.

**Freshen-now ritual (corrected — Charge I).** To see a governance edit in *this*
working context: run `doctrine boot` (writes the fresh snapshot to disk) **then**
`/clear` or restart — the cleared session's step-2 inline reads the just-written
file. *`doctrine boot` alone is impotent* (the prefix already inlined this session);
*`/clear` alone serves the pre-edit snapshot* (its step 2 reads the old file before
its own step-3 hook regenerates). `/canon` carries this ritual (Q5-A: governance.md
is already-in-context; the note is "regenerate **then** clear/restart", not "re-run
boot").

**Health sentry (Charge II) — a DISK sentry, not a session sentry** (codex review
finding 2). `doctrine boot --check` flags **disk staleness** (on-disk boot.md ≠
recompute-from-current-governance) and **partial-population** (marker sections) — the
failures the silent `produce()` markers and a dead-binary hook (Charge III) would
hide. **It cannot see what the *current* session inlined:** after the hook (or a
manual `doctrine boot`) refreshes the file, `--check` reports clean while this
session still reads the stale inlined prefix until `/clear`/restart. So `/route`
does two things: (a) run `--check` for disk health, **and** (b) warn of the inherent
≤2-session in-session lag, pointing at the freshen-now ritual.
*Optional upgrade for true in-session detection:* embed a deterministic
**governance content-hash** marker in the snapshot (cache-safe — it changes only when
governance changes, exactly like the content); `/route` compares the hash in its own
context against `boot --check`'s freshly-recomputed hash → a mismatch means the
inlined prefix is stale. Deferred unless the lag warning proves insufficient.

**codex** (in scope — slice-011.md:75–83, 121–129): AGENTS.md `@`-import only, **no
hook** → regenerates only on the *next `doctrine` invocation*, which for a codex-only
user may be arbitrarily far off. Its staleness is therefore **unbounded**, not a
bounded window (codex review finding 3) — worse than Claude's ≤2-session lag. Accepted
for v1 and **verified live as a closure gate**; if the live run shows the import does
not even inline (or the unbounded staleness is unacceptable), **codex is cut from v1**
and the slice ships Claude-only, the seam kept for a later harness.

### 5.5 Invariants, Assumptions & Edge Cases

- Snapshot is a pure projection — never authoritative.
- `render_boot` is deterministic (no clock/rng) → content-diff is stable.
- `.doctrine/state/` created before write.
- Never double-prepend the ref to two views of one inode; never let two read-files
  each carry the ref for the *same* agent (review fix #1).
- **One-session lag is invariant, not a bug** (Charge I): the hook regenerates for
  the *next* session — the @-import already inlined before it runs. Sync vs async is
  a start-up-latency choice, **not** correctness (the refuted "review fix #2" claimed
  a blocking hook freshens the current read; it does not). `doctrine boot` is fast
  either way; a non-blocking hook is acceptable since it need only finish before the
  next session begins. Freshness is reported by `boot --check`, never embedded as a
  cache-busting timestamp.
- `install_refresh` failure for one harness must not abort the others.
- Foreign SessionStart hooks (user's own) are preserved; only the doctrine-owned
  entry (path basename `doctrine`, ` boot` suffix) is touched.
- `produce()` tolerates a missing `Static` digest asset with a benign marker, so
  `doctrine boot` works before the digest is authored (review fix #3 — breaks the
  phase-2/phase-6 ordering knot).
- Reinstall re-adds a deliberately-removed ref (accepted: it is an install verb).

## 6. Open Questions & Unknowns

- **Session load order — RESOLVED (Charge I).** The @-import inlines into the cached
  prefix *before* SessionStart hooks run (Claude Code *prompt-caching* + *hooks*
  docs). The hook freshens the *next* session → the one-session-lag law (§5.4). The
  prior "synchronous ⇒ fresh" belief is struck. *Remaining live confirmation is the
  codex inline + the matcher token, below — the Claude ordering itself is documented.*
- **codex `@`-import semantics** — does codex inline AGENTS.md `@`-imports into
  its system prompt the way Claude does? No SessionStart equivalent → staleness.
  **Resolved by the live codex run** in the closure phase, not by assumption.
- **Hook matcher tokens** — confirm Claude's `SessionStart` matcher accepts
  `startup|clear` as specified (research doc asserts it; `clear` firing a
  SessionStart hook is *already witnessed* this session; verify the OR-token string
  against the live harness when wiring phase 4).
- **`memory::list_rows` shape** — keep additive to avoid the slice-012 clash (still
  `in_progress`); exact signature confirmed at phase 2 against the then-current
  `memory.rs`.

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
- **D8 (R2) per-harness wiring seam; Claude is one adapter.** Vendor-specific wiring
  is not core (R2 upheld). **Revised by Charge IV:** the seam is `enum Harness` +
  `match`, mirroring `skills.rs`'s Claude-vs-Other shape — **not** `trait Harness` +
  `Box<dyn>` + a parallel registry/resolver, which over-abstracts one full impl + one
  stub. A third harness is a new match arm, no framework. Identity-unification with
  `skills::Agent` is **deferred debt** until SL-012 frees `skills.rs` (§3 gate).
  **Not** dynamic plugin loading (YAGNI).

## 8. Risks & Mitigations

- **Concurrency on `skills.rs`/`memory.rs`** (SL-012 still `in_progress`) → all new
  code in `boot.rs`; **`memory` touch is additive** (new `list_rows` over existing
  `select_rows`/`format_list` — the allowed kind on a contended file); **`adr` is a
  behaviour-preserving extract** (not additive) but `adr.rs` is **uncontended** by
  SL-012, so the riskier deed lands on the safe file (Charge V re-weight).
- **codex behaviour unknown** → closure-gate live run; degrade gracefully if the
  import doesn't inline.
- **Malformed user `settings.local.json`** → fail-soft, print, never clobber.
- **Prefix-budget duplication** → AGENTS.md rewrite removes what the snapshot now
  carries; net prefix cost roughly flat, tool-call cost drops.

## 9. Quality Engineering & Validation

Pure: `render_boot` determinism + section order (no clock/timestamp → cache-stable);
missing-source marker; `boot_sequence` carries the extensible kinds. `write_if_changed`:
writes on change, no-op when equal, returns the bool. `ensure_boot_import`: prepend-once
idempotent, create-missing, **symlink/same-inode → exactly one write**, preserves
existing content. Claude `install_refresh`: merge into empty/missing, **refresh an
existing doctrine entry on path change**, **preserve foreign hooks**, **ownership match
survives spaces in the exec path** (Charge VII), **fail-soft on malformed JSON**.
**`boot_check` (Charge II/III):** a stale on-disk file and a marker (failed/missing)
section each trip a *visible* warning; a fully-populated current file reports clean.
`adr`/`memory` `list_rows`: existing CLI suites stay green (`adr.rs:331` guards the
extract).
Integration: `doctrine boot` emits the headings + ADR rows + the exec path;
`doctrine boot --check` reports freshness/population.
Closure (live ordeal — Charge I/VII): on the live harness, edit governance, run the
ritual, and **witness** that the one-session-lag holds as documented and the
`startup|clear` matcher fires. **live codex session** confirms the snapshot loads and
routing is honoured.
**Struck:** the `assert no async:true` gate (review fix #2) — it tested a property
irrelevant to freshness; replaced by the lag-law ordeal + `boot_check`.

## 10. Review Notes

### Internal adversarial pass (round 1)

- **#1 Double-load / wrong-file.** Guide confirmed CLAUDE.md inlines `@`-imports;
  AGENTS.md unconfirmed. Two ref-bearing files read by one agent ⇒ snapshot
  inlines twice. **Resolved:** Claude `import_targets=[CLAUDE.md]`, codex
  `=[AGENTS.md]`; symlink dedups this repo to one write. (§5.3, §5.5)
- **#2 Async hook = stale read. — REFUTED (see Inquisition Charge I).** This note
  declared the freshness problem "resolved" by a synchronous hook. The premise was
  false: the @-import inlines *before* the hook runs, so blocking gains nothing. The
  real model is the **one-session-lag law** (§5.4). Disposition below.
- **#3 Phase-2/phase-6 ordering.** `doctrine boot` renders the `Static` digest
  before the digest asset is authored. **Resolved:** `produce()` benign-marker
  tolerance — no hard dependency. (§5.5)
- **#4 Loose ownership match.** `*doctrine*boot*` could hijack a user hook.
  **Resolved:** match path basename `doctrine` + ` boot` suffix. (§5.3)
- **#5 `current_exe()` under wrappers** (cargo/nix) — accepted for v1, flagged. (§6)

### External coordination (SL-012 concurrency)

SL-012's **memory-port portion** landed mid-design (the `doc(SL-012)` commits:
retired the `doc/memories/` bargain-bin, ported notes into the `doctrine memory`
store, edited CLAUDE.md/AGENTS.md). **The slice itself remains `in_progress`**
(symlink-tolerance work on `src/skills.rs`/memory-record ongoing) — so the §3
concurrency gate on `skills.rs` still holds (codex review finding 5: reconciles the
earlier loose "SL-012 landed" phrasing). Impact on this slice: (a) the **Memory**
snapshot section is now a clean projection of the canonical store (no bargain-bin
duality); (b) the AGENTS.md-rewrite objective's "reconcile the memory story" is
largely **subsumed** — the rewrite narrows to CLI-recital removal + delegating
routing/process to the snapshot.

### Inquisition disposition (round 2 — `inquisition.md`, 2026-06-05)

All seven charges examined and **accepted**; each verb-correction folded above. The
mechanism survives — only false claims were struck. Verdict-by-charge:

- **I (RED) — hook fires after the prefix is sealed. ACCEPTED.** Confirmed against the
  live harness docs (prompt-caching + hooks; via claude-code-guide): the @-import
  inlines *before* SessionStart hooks. Struck "review fix #2" and the
  synchronicity-equals-freshness doctrine (§5.3/§5.4/§5.5/§9/§10 #2). Inscribed the
  **one-session-lag law** + corrected freshen-now ritual (regenerate **then**
  clear/restart). Replaced the `assert no async:true` gate with the lag-law ordeal +
  `boot_check`. *Acceptable:* governance rarely changes; value (zero tool calls, warm
  cache) is intact.
- **II (HIGH) — fails silent, no sentry. ACCEPTED + refined.** Added `boot_check` /
  `doctrine boot --check` reporting **staleness** and **partial-population**, replacing
  the blind heading grep (§5.2/§5.4/§9). *Refinement beyond the charge:* the signal is
  **out-of-band** — an in-content generation timestamp was rejected because it would
  bust the cache every session, defeating §1.
- **III (MED) — `current_exe()` silent freeze. ACCEPTED, bounded.** Pure/impure
  placement acquitted. A dead-binary hook now surfaces via `boot_check` (stale file)
  → detectable, no longer concealed (§5.2/§5.4). PATH-probe fallback left as a v1
  option; detection is the gate.
- **IV (MED) — twin abstraction. ACCEPTED → collapsed.** Dropped `trait Harness` +
  `Box<dyn>` + `registry` for `enum Harness` + `match` (skills.rs's own shape); no
  third harness exists to earn a trait (§5.2/§7 D8). Full identity-unification with
  `skills::Agent` **deferred** as named debt until SL-012 frees `skills.rs` (§3 gate).
- **V (LOW–MED) — "additive" misattributed. ACCEPTED.** Relabelled: `memory` = additive
  wrapper (contended file, safe touch); `adr` = behaviour-preserving **extract** of
  `run_list` (uncontended file) guarded by `adr.rs:331`. SL-012 risk re-weighted down
  (§5.2/§8).
- **VI (LOW) — `governance.md` boundary undefined. ACCEPTED.** Added the remit
  paragraph delimiting it against CLAUDE.md / `doc/*` / ADRs (§5.3). Gitignore
  mechanics already acquitted by the Inquisition.
- **VII (LOW) — ownership-by-suffix brittle. ACCEPTED.** Hardened the match against
  spaces (program `file_name == doctrine` ∧ last-arg `== boot`); `startup|clear` token
  confirmed live at wiring (§5.3/§6/§9). `clear`-fires-a-hook already witnessed.

**On the inquisition's "ordeal before lock" (penance #1).** That sequencing is
**unsatisfiable by construction** and is corrected here: the live ordeal — edit
governance, restart, observe fresh-vs-stale — requires a *built binary* and a *real
user-driven session*. Nothing analytical substitutes, and there is nothing to test
before the mechanism exists. The ordeal is therefore intrinsically a **post-build,
user-run closure verification**, not a pre-`/plan` gate. What *was* answerable before
lock — the load-order axiom — is settled (docs + codex concur). The live ordeal
(Charge I/VII) and the live codex run ride into closure (§9); their being deferred is
the correct shape, not a skipped penance.

### External adversarial pass (codex MCP, round 3 — 2026-06-05)

Fresh independent review of the *revised* design. Five findings; four accepted, one
premise rejected. Folded above.

- **F1 (RED) — lag understated; worst case is TWO sessions, not one. ACCEPTED.** An
  edit made after session N's start-hook is captured only by N+1's hook → visible at
  N+2. Corrected §5.4 to the **bounded ≤2-session lag** (ritual → zero).
- **F2 (HIGH) — `boot_check` is a disk sentry, not a session sentry. ACCEPTED.** It
  reports the *file* fresh while the *current* inlined prefix is still stale. §5.2/§5.4
  narrowed to disk-freshness + population; `/route` also warns of the in-session lag;
  recorded the optional **governance content-hash** marker for true in-session
  detection (cache-safe).
- **F3 (HIGH) — codex scope creep / unbounded staleness. PREMISE REJECTED, sub-point
  ACCEPTED.** codex is *in scope* (slice-011.md:75–83, 121–129, 170 ship a Claude+codex
  seam; line 16's "Claude-only" qualifies the *hook*, not the harness set) — not creep.
  But its staleness is honestly **unbounded** (no hook), restated in §5.4 with an
  explicit **cut-codex-from-v1** fallback at the closure gate.
- **F4 (MED) — exec path pollutes the cache key. ACCEPTED.** `current_exe()` is
  build-volatile, not governance. §5.3 bounds it by environment (cargo stable /
  installed-on-PATH moot / nix-store = legit bust) and orders the section **last** so
  a path change invalidates only the cache tail.
- **F5 (MED) — SL-012 "landed" vs "in_progress" contradiction. ACCEPTED.** §10
  reconciled: only the memory-*port* landed; the slice is `in_progress`, so the
  `skills.rs` gate (and the deferred Charge-IV unification) stand.
- **Note — adr extract. CONFIRMS Charge V.** Output already centralised in
  `meta::format_list` (`src/meta.rs:46–49,91–103`); `adr.rs:155` is a thin `write!` of
  that string → `list_rows` returning `meta::format_list(&rows)` is byte-identical
  (`write!`, no extra newline). Behaviour-preserving.

### Next

Two independent hostile passes (inquisition + codex) dispositioned; the design now
states its staleness honestly (≤2-session Claude / unbounded codex), bounds the
cache key, and posts a real (disk) sentry. **Remaining before lock are empirical, not
analytical:** the live ordeal (lag-law + `startup|clear` matcher) and the live codex
run — both **closure-phase gates**. Design ready for `/plan`; these gates ride into
execution/closure.
