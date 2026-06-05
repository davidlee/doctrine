# SL-011 audit — cache-friendly session boot context

Hand-authored close-out (no `slice audit` scaffold yet — known CLI gap). Verifies
the shipped boot mechanism against `design.md` and the locked decisions D1–D8 /
the §5 contracts / the two hostile-pass corrections (inquisition Charges I–VII +
codex F1–F5), and records the durable risks, divergences, and follow-ups harvested
from the six phase sheets and `notes.md`. `inquisition.md` already ran (round 2, 7
charges) and the codex MCP pass (round 3, 5 findings) — both dispositioned into
design §10; their findings are carried here, not re-derived. The LATE closure
finding (`55b58c8`) materially changed the codex disposition — represented below as
the AS-SHIPPED reality.

- **Status:** all 6 phases completed. `just check` = **361 unit + 3 e2e green,
  clippy zero (bin/lib)**. The e2e suite first tripped the known stale-
  `CARGO_BIN_EXE` footgun (`mem.pattern.testing.stale-cargo-bin-exe`): the compiled
  `e2e_memory_anchoring` binary embedded a hashed `target/debug/doctrine-<hash>`
  path that no longer existed → 3 spawn-NotFound failures; a `touch` + rebuild of
  the test binary made all 3 green. Test-infra artifact, **not** a product defect.
  Commits: `5cd6dbd` (01) · `1fcc514` (clippy-denies memory) · `1ab633f` (02) ·
  `f133a04` (03) · `2f4b894` (04) · `d67499d` (05) · `edf5ee4` (06) · `ca112e8`
  (jail dual-context fix) · `55b58c8` (codex closure finding). Inquisition+plan:
  `f072fc0`/`b83e365` → `320bfcf`.
- **Verdict:** **ships (Claude-only v1).** The Claude path is functional end-to-end
  (`@`-import inlines; SessionStart hook regenerates the next session). Two headline
  realities, both correctly handled, not defects:
  1. **The codex `@`-import path is DEAD as shipped** — the live codex run
     (`55b58c8`) proved codex reads `AGENTS.md` verbatim but does **not** expand the
     `@`-import, so the snapshot body never reaches the codex model. The codex arm
     ships as a **staged-but-inert seam**; codex is **cut from v1** and deferred to
     **SL-014** (`codex SessionStart-emit boot wiring`, scoped per `9d24215`) with a
     proven replacement mechanism (a `SessionStart` hook emitting the snapshot on
     stdout — zero-lag, *better* than Claude's `@`-import). Correctly deferred — does
     **not** block closure of the Claude-only v1.
  2. **The Claude `@`-import path is LIVE** — distinct from codex. Claude *does*
     inline `CLAUDE.md` `@`-imports into the cached prefix (design §1/§10 confirmed
     against the live harness; this very session reads the snapshot via the
     `@.doctrine/state/boot.md` ref). The boot mechanism is **not broken**: it ships
     working for its supported v1 harness.

## Coverage vs design

| Area | Design | Shipped | Note |
|------|--------|---------|------|
| Pure assembly seam | §5.2, EX-1 | `Section`/`SourceKind`/`boot_sequence`/`render_boot`/`marker` | no clock/rng/disk; ExecPath ordered LAST (boot.rs:76–88) ✓ |
| Content-diff writer | §5.2/§1, EX-2 | `write_if_changed` → `fsutil::write_atomic` | writes only on byte-change, returns wrote-bool (boot.rs:158) ✓ |
| Marker tolerance | §5.5 fix #3, EX-2 | `produce`/`section_or_marker` | miss/err/empty → benign `<!-- … -->`, never panics (boot.rs:120–152) ✓ |
| Single render path | Charge V, notes | `build_sections` shared by `regenerate` + `boot_check` | no second fork (boot.rs:174) ✓ |
| ADR reuse (extract) | §5.2 Charge V, EX-2/VT-2 | `adr::list_rows` (extract of `run_list`) | `run_list` prints it byte-identically; e2e at adr.rs:331-class green (adr.rs:152,160) ✓ |
| Memory reuse (additive) | §5.2 Charge V, EX-1 | `memory::list_rows` over `select_rows`+`format_list` | no pre-existing line touched (memory.rs:1102) ✓ |
| Exec path (R1/D7) | §5.3/F4, EX-3 | `produce(ExecPath)` = `current_exe()`, section LAST | path change hits only cache tail ✓ (but see D-A1 below) |
| Harness seam (D8/Charge IV) | §5.2 | `enum Harness {Claude,Codex}` + `match` | NOT trait/Box/registry; mirrors skills.rs (boot.rs:304) ✓ |
| `@`-import wiring | §5.3, EX-1/VT-1 | `plan_boot_import`/`ensure_boot_import`/`resolve_target` | idempotent prepend, create-missing, inode-dedup, symlink-through (boot.rs:384–441) ✓ |
| SessionStart hook merge | §5.3/D3, EX-2/VT-2 | `plan_session_hook` (pure) + `install_refresh` | own-by-pattern, refresh-on-change, preserve-foreign, fail-soft (boot.rs:567–638) ✓ |
| Ownership match (Charge VII) | §5.3 | `is_doctrine_boot_command` (last-whitespace rsplit) | survives spaced exec path; rejects foreign hooks (boot.rs:498) ✓ |
| `boot --check` disk sentry | §5.2/§5.4/F2, EX-1 | `boot_check`/`CheckReport`/`run_check` | stale + marker tally; deterministic, no timestamp (boot.rs:217) ✓ |
| `/route` + `/canon` copy | §5.4 Charge I/F2, EX-2 | route/canon SKILL.md | disk-vs-session split + freshen-now ritual; no over-claim ✓ |
| governance.md (D5/Charge VI) | §5.3 | `install/governance.md` seeded by Skip path | remit boundary honoured (no restating CLAUDE.md/doc/ADR) ✓ |
| AGENTS.md de-dup | §5.1, EX-1/VT-1 | `edf5ee4` | dropped `## doctrine cli`, `just list-memories` nag, `## core process`; → 99 lines, pointers only ✓ |
| boot.md tier | §5.3/D2 | gitignored (`.doctrine/state/boot.md`) | derived runtime state, `git check-ignore` confirms ✓ |
| codex `@`-import | §6 open q / D8 | `Harness::Codex` import-only arm | **DEAD as shipped** — see Verdict + D-A2 below |

## Findings — correctness

- **C-1 — pure/impure split honoured (house §4 rule). ALIGNED.** The pure core
  (`boot_sequence`/`render_boot`/`marker`/`plan_boot_import`/`plan_session_hook`/
  `is_doctrine_boot_command`/`find_owned`/`boot_command`/`desired_entry`) reads no
  clock, rng, git, or disk; resolution of `current_exe()` lives only in the `run*`
  shells (boot.rs:235,252,654). `boot_check` is deterministic (a generation
  timestamp was deliberately rejected — it would bust the cache every session,
  defeating §1). Verified by `boot_check_is_deterministic` + the determinism unit
  tests. Cite §4/§5.2/Charge II-refinement.
- **C-2 — `list_rows` reuse is genuine, no parallel impl. ALIGNED.** `adr::list_rows`
  is a behaviour-preserving extract: `run_list` (adr.rs:152) now `write!`s the
  returned string verbatim; `format_list` already carries its trailing newline so
  output is byte-identical. `memory::list_rows` is a thin additive wrapper. The
  adr/memory CLI suites stayed green (behaviour-preservation gate, §4). Cite Charge V
  / EX-2 / VT-2.
- **C-3 — inode-dedup + symlink-through write is correct. ALIGNED.** `resolve_target`
  canonicalizes existing targets so the `CLAUDE.md → AGENTS.md` union collapses to
  one inode/one write, and the write lands on the real file (never `rename`d over the
  link — the symlink survives, proven by `ensure_boot_import_dedups_same_inode_to_one_write`).
  Cite §5.3/§5.5/VT-1.
- **C-4 — hook ownership match is non-clobbering and space-robust. ALIGNED.** The
  last-whitespace `rsplit_once` keeps a spaced program path whole while isolating the
  `boot` arg; foreign hooks (`tool boot`, `/x/doctrine-helper run`, `/x/doctrine
  check`, bare `doctrine`) all correctly reject. JSON merge mutates a `Value` at the
  narrow path — foreign hooks + unrelated keys (`model`) survive (proven by
  `plan_session_hook_refreshes_on_path_change_preserving_foreign`). Malformed JSON →
  `PrintedFallback`, never clobber. Cite Charge VII / D3 / VT-2.
- **C-5 — one-harness-failure isolation works. ALIGNED.** `wire` prints a single
  harness's refresh error and continues; `wire_isolates_one_harness_failure` forces
  Claude's write to fail (dir squatting the settings path) and asserts codex's import
  still lands. Cite §5.5 A9 / EX-3.
- **C-6 — no correctness defect found.** No "fix now" finding. Closure is **not**
  blocked.

## Divergences to fold into design

- **D-A1 — hook command is bare `doctrine boot`, not the absolute `current_exe()`
  the design specifies. TOLERATED DRIFT (operator-resolved), fold into §5.3.** Design
  §5.3/D6/D7 bakes the resolved absolute `current_exe()` into the hook command (single
  source with the snapshot's "Invoking doctrine" body). As shipped, this repo's wired
  hook (`.claude/settings.local.json`) carries the **bare** `doctrine boot`. Rationale
  (notes.md §"jailed-dev hook path"): the nixos+bubblewrap dev jail splits the mount
  namespace — the in-jail agent resolves `/workspace/doctrine/target/debug/doctrine`
  while the host harness runs the hook where that path does not exist → the absolute
  path is *wrong by construction* across the host/jail boundary. The landed fix
  (`ca112e8`) ro-binds the host `~/.cargo/bin/doctrine` into the jail and puts it on
  PATH, so the **bare** invocation resolves identically in both contexts (one binary,
  one currency point) — which is *exactly* the design's own "installed ⇒ emit bare
  `doctrine`" branch (§5.3 R1), just applied to the hook command too. The design's
  §5.3 "on PATH ⇒ emit bare `doctrine`" branch is **still UNIMPLEMENTED in the
  snapshot body** (`produce(ExecPath)` always emits the absolute `current_exe()`), so
  the *snapshot* and the *hook* now diverge in path-shape on this repo. Harmless here
  (the snapshot's exec line is ordered last → only the cache tail; and the ro-bind
  makes even the absolute in-jail path stable), but the design text claims a single
  shared path source that no longer literally holds. → fold the bare-emit branch and
  the jail mount-namespace case into §5.3 R1; the bare-emit snapshot branch is the
  named LOW-priority follow-up (notes.md §5.3 bare-emit follow-up).
- **D-A2 — codex `@`-import arm is DEAD; the design's codex mechanism is wrong.
  DESIGN WAS WRONG → FOLLOW-UP SLICE (SL-014).** Design D8/§5.2/§6 wired codex as
  *import-only* (`@`-import into `AGENTS.md`, no hook) on the §6 open assumption that
  codex inlines `@`-imports like Claude. The live codex run (`55b58c8`, codex-cli
  0.133.0) **disproved** it: `codex debug prompt-input` shows codex reads `AGENTS.md`
  verbatim but renders the `@.doctrine/state/boot.md` line as plain text — the
  snapshot body never reaches the model. The §5.4/§6 premise "codex has no
  SessionStart equivalent" is also **stale**: codex 0.133.0 *does* have a
  `SessionStart` hook whose stdout is injected as developer context (spike-confirmed
  live). → The correct codex path (SessionStart-emit hook, zero-lag, supersedes both
  the import-only arm AND the design's cut-fallback) is scoped to **SL-014**. Design
  §5.4/§6 must be revised when SL-014 is shaped. The SL-011 disposition (codex cut
  from v1, Claude-only) is correct and **does not block closure** — the seam stays
  staged for SL-014 to repurpose.

## Accepted risks / known edges

- **A-1 — Claude bounded ≤2-session in-session lag.** By construction the
  `@`-import inlines into the cached prefix *before* the SessionStart hook runs, so
  the hook freshens the *next* session; an edit is visible at N+1 (best) or N+2
  (worst). Accepted (design §5.4 lag law, Charge I / codex F1) — governance rarely
  changes; value (zero tool calls, warm cache) intact; the freshen-now ritual
  collapses it to zero. `/route` + `/canon` carry the ritual and the disk-vs-session
  warning (verified in the SKILL copy). Recorded, not a defect.
- **A-2 — `boot --check` is a DISK sentry, never a session sentry** (codex F2). It
  proves the *file* fresh while the *current inlined prefix* may be stale until
  `/clear`/restart. CLI wording is disk-scoped on purpose ("on-disk snapshot in
  sync"); `/route`'s in-session lag warning carries the rest. Correctly bounded —
  the route SKILL explicitly states a clean `--check` does **not** prove the inlined
  prefix is current. Accepted by design.
- **A-3 — dead-binary hook freezes boot.md silently, surfaced only by `--check`**
  (Charge III). A hook pointing at a vanished binary fails (errors swallowed) and
  boot.md stops refreshing; `boot --check` (stale report) is the only detector. The
  jail case (notes.md) hit exactly this — the live ordeal witnessed
  `No such file or directory` (non-blocking, session continued). Resolved for this
  repo by `ca112e8`; the general "stale ⇒ detectable, not concealed" posture stands.
  Accepted, detection is the gate.
- **A-4 — ownership match is a dep-free single-arg `rsplit`, not a shell tokeniser**
  (notes.md A4). Correct while the hook command is always `<program> boot` (one
  space-free arg). FOOTGUN: a second/spaced *arg* would break it → switch to
  `shell-words`. Guarded by a NOTE on `is_doctrine_boot_command` + the spaced-path
  test. Accepted v1.
- **A-5 — JSON merge re-sorts object keys (no `preserve_order`).** The `Value`
  round-trip preserves every foreign key/hook (tested) but alphabetises object keys.
  Acceptable: `settings.local.json` is gitignored/regenerable. Accepted.
- **A-6 — `--check` cannot `conflicts_with` the `install` subcommand** (clap limit,
  notes.md). Dispatch precedence handles it: `Some(Install)` wins, else `None if
  check` (main.rs:600–608); the arg help documents `--check` as ignored under
  `install`. Accepted — a no-op, not a defect.
- **A-7 — `Accepted ADRs` renders a marker on this repo.** `boot --check` reports
  `unpopulated sections: Accepted ADRs` — the genuinely-empty case (no accepted ADRs
  exist), not a failure. The marker tolerance behaving as designed. Accepted.

## Doctrine adherence

- Pure/impure split upheld (C-1); no parallel implementation (C-2 — adr extract +
  memory additive wrapper ride existing seams; `fsutil::write_atomic`/`root::find`/
  `install::asset_text`/`install::prompt_confirm` reused, not forked).
- Behaviour-preservation gate green: the adr (incl. the e2e at adr.rs:331-class) and
  memory CLI suites stayed green across the `run_list` extract — the proof the shared
  listing machinery was not disturbed (EX-2/VT-2).
- Concurrency gate honoured: all new logic in `src/boot.rs`; the only shared-file
  touches were the additive `memory::list_rows` and the uncontended `adr` extract
  (§3/§8). The deferred Charge-IV identity-unification of `Harness` with
  `skills::Agent` is named debt — and **now unblocked** (SL-012 closed, `631b746`),
  so it is harvestable, but out of SL-011 scope.
- Storage rule honoured: boot.md is derived/gitignored (D2); governance.md is
  authored/user-owned/seed-if-missing (D5); no queried data in prose.

## second-pass: confirmed (independent /code-review, 2026-06-05)

Adversarial second pass over `5cd6dbd~1..55b58c8 -- src/` plus the live binary.
Tried to REFUTE the ship verdict and the two headline claims; both hold. Gate
re-run: **361 unit + 3 e2e green, clippy zero (bin/lib)** (e2e green after the
documented `CARGO_BIN_EXE` rebuild).

- **The @-import-dead / live-Claude split — CONFIRMED, both directions.** Codex arm:
  `import_targets(Codex)=[AGENTS.md]` + `install_refresh(Codex)=None` (boot.rs:336,623)
  — wires only a ref codex won't expand → dead, as `55b58c8` proves. Claude arm: this
  session reads the snapshot body via the `@`-import (`# Doctrine Boot Context`,
  `Route before you act` present in context) → live. Boot is **not** broken; it ships
  working for Claude. The codex deferral to SL-014 is correct, not a concealed defect.
- **Live boot verbs — CONFIRMED.** `doctrine boot` → `Wrote …/boot.md`; a second run
  no-ops (content-diff cache key). The generated snapshot carries the header, all five
  ordered headings, real routing digest (embed), governance body (disk), memory rows,
  and the exec path LAST. `boot --check` → `unpopulated sections: Accepted ADRs` only
  (the genuinely-empty section), stable across runs (not perpetually stale — the
  ca112e8 ro-bind keeps the exec line consistent within a context).
- **Pure layer scrubbed for impurity — CONFIRMED clean.** No `current_exe()`/`fs::`/
  clock in `boot_sequence`/`render_boot`/`plan_*`/`is_doctrine_boot_command`; all I/O
  and exec-resolution sits in the `run*`/`wire`/`produce`/`ensure_*`/`install_refresh`
  shells. `build_sections` is the single render path shared by write and check — no
  fork (Charge V honoured).
- **AGENTS.md de-dup — CONFIRMED.** `edf5ee4` removed `## doctrine cli`, the
  `just list-memories` nag, and `## core process`; the residual `doctrine memory
  record|find|retrieve` mention is a one-line pointer (orientation remit), not a
  recital. No governance lost — the cut content is projected by the inlined prefix.

No correctness defect found. The slice **ships (Claude-only v1)**; the codex
`@`-import death is real, honestly recorded, and correctly deferred to SL-014 — it
does not block closure. Close-out stands.
