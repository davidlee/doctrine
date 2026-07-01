# Implementation Plan SL-182: Claude-arm subagent write-confinement hooks

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-182 graduates the RSK-014 probe-h1 apparatus — proven hard write-containment
for a claude `isolation:worktree` subagent — from throwaway shell into installed
doctrine machinery, closing the ADR-006 D2b / ADR-012 OQ-D impersonation gap on
the Linux/bwrap claude arm. Five phases, strictly sequenced. The spine is the
design's own §9 sequencing: **prove the harness first (D7), then build Rust onto a
proven shape, then converge the funnel.**

The five phases map onto the slice objectives:

- **PHASE-01** — the D7 empirical probe (gates everything below).
- **PHASE-02** — pure jail core (`jail.rs`) — objectives 1 + 2's logic.
- **PHASE-03** — the `pretooluse` shell, registration, and fail-closed install —
  objectives 1 + 2's enforcement + objective 4.
- **PHASE-04** — the per-arming policy surface — objective 3.
- **PHASE-05** — funnel convergence via symmetric live-import
  (`worktree import --from-worktree`) — objective 5.
  **AMENDED 2026-07-01** (design §10 PHASE-05 amendment + RV-205 inquisition): the
  `SubagentStop` capture design was retired after a live probe disproved its teardown
  premise. See "Why funnel convergence is last" below.

## Sequencing & Rationale

**Why probe-first (PHASE-01 before any Rust).** The two tallest risks (R1
funnel-teardown, R2 plugin-registration) plus the RV-202 `SubagentStop`
worktree-correlation gap are *harness behaviours* the `docs/claude` cache does not
prove — it documents none of the timing, and SubagentStop carries no
`worktree_path`. Sinking Rust into a refuted premise is the expensive failure mode.
The probe is the RSK-014 idiom (live `settings.local.json` hooks + `rm`-able shell)
and is the cheapest place to refute each premise. Crucially, **its outcomes branch
the rest of the slice**: item 1 picks the PHASE-03 registration path (plugin vs the
`settings.local` fallback); item 2 decides Path L vs the abort to Path C / IDE-024
that would moot PHASE-02/03/04 as written; item 3 decides whether the `|| exit 2`
vanish-guard is even shell-runnable. So PHASE-01 is a true gate, not a warm-up —
its VA/VH exits record the decisions the later phases consume.

**Why the pure core is its own phase (PHASE-02).** ADR-001 layering separates the
pure jail logic (no clock/git/disk/rng) from the impure hook shell. The pure
surface — `resolve_target`, `pathcheck`, `bwrap_argv`, `opaque_wrap`,
`validate_policy`, `JailPolicy`/`load_policy` — is independently testable with
TDD red/green/refactor and carries the load-bearing invariants (INV-3 `.git`/root
rejection, INV-4 validated-allowlist trust, INV-5 shell-quoting, A1 topology
recognition, D5 pi-arm parity). Landing and proving it before the shell means
PHASE-03 wires a *trusted* core into stdin/stdout plumbing rather than debugging
logic and I/O together.

**Why registration + install fail-closed ride together (PHASE-03).** The
`pretooluse` shell is meaningless until an installed hook invokes it, and the F-1
blocker is precisely that the *installer* (`install_hooks_plugin_for_claude`) ships
the plugin `hooks.json` as a verbatim byte-copy with bare `doctrine` commands —
fail-OPEN. So the shell, its two PreToolUse registrations, and the install-time
templating that bakes an absolute `resolve_exec()` path are one coherent unit: the
wall is only real once it is both invoked and invoked through a resolved binary.
The `settings.local` fallback (if PHASE-01 refuted the plugin path) lands here in
the same phase (RV-200 F-5), not as a deferred contingency.

**Why the policy surface follows enforcement (PHASE-04).** The per-arming config
(`extra_rw` + `network`) only matters once `pretooluse` exists to consume a resolved
policy. This phase is the plumbing that flows an orchestrator-declared intent through
the spawn handshake: `arm-spawn` writes `jail.toml` beside `base` in one arming step
(the F-4 pairing, made atomic by the blocking `Agent` call + INV-6), and the
`create-fork` hook gains a net-new provision step (patterned on
`marker.rs:write_marker`) that materializes `jail/<name>.toml` under the name it
learns at spawn. Absence ⇒ the strictest floor, so the wall holds even with no
declared policy.

**Why funnel convergence is last (PHASE-05) and riskiest — AMENDED to live-import.**
Confinement's *consequence* — ro-`.git` kills the worker's self-commit — only bites
once the jail is actually enforcing (PHASE-03/04). The funnel converges onto the
worker's **live** working-tree diff. The original design routed this through a
`SubagentStop` capture hook on the assumption that the harness tears the worker
worktree down on subagent finish; a PHASE-05 live probe (2026-07-01) **disproved
that premise** — with `create-fork` as the `WorktreeCreate` hook and no
`WorktreeRemove` hook, the tree **persists on disk** post-return, diff intact
(INV-6), and the Agent footer hands its `worktreePath` per-return (proven,
`mem_019efe28` P2). So the orchestrator imports the **live** tree directly:
read footer `worktreePath` → `verify-worker --dir` (identity/base belt, unchanged
Rust) → `worktree import --from-worktree` (relocated gather + unchanged
`classify_import` belt + `git apply --index`) → `git worktree remove --force` **iff
import succeeded** (F-3). No capture, no correlator, no teardown race. The capture
apparatus (`capture.rs`, `SubagentStop` command/guard/hook entry, file-based
`--patch`) is **retired** (git history is the archive). The E2E leg (live claude
`/dispatch` on the **Fork** path, one jailed worker, escape vectors denied + tree
persists + funnel green via `--from-worktree`) is the VH acceptance and the slice's
tallest verification — it also confirms the Fork-path persistence + branch-case
footer (§5.5 ASM). OQ-1 is **resolved**: the delta-check is the pure
`classify_import` in `src/worktree/import.rs` (not `src/dispatch.rs`). The RV-205
inquisition hardened this: INV-6 two-boundary enforcement (install-time no-
`WorktreeRemove` assert + runtime `verify-worker` absence-catch), reap gated on
import success, Fork-path ASM grounded in hook-presence.

**Dependency chain (linear):** PHASE-01 → PHASE-02 → PHASE-03 → PHASE-04 →
PHASE-05. Each phase's EN criteria name its upstream green. PHASE-01 can abort the
whole Path L approach (→ Path C / IDE-024) before any Rust is sunk; that is the
point of putting it first.

## Notes

- **Net-new vs ride-existing.** `jail.rs` and `pretooluse.rs` are new modules;
  `WorktreeCommand::Pretooluse` mirrors `CreateFork`. The `create-fork` provision
  step is net-new beside the existing `run_provision`/`write_marker`
  (`fork.rs`/`marker.rs`). The install templating modifies the existing
  `install_hooks_plugin_for_claude` (`src/skills.rs:1024`). The two PreToolUse
  entries are net-new in `plugins/doctrine/hooks/hooks.json` (today: SessionStart +
  WorktreeCreate only). **AMENDED:** the `SubagentStop` entry is NOT added (capture
  retired); PHASE-05 instead REMOVES it (the T1–T4 landings are reverted) and asserts
  no `WorktreeRemove` entry ever ships (F-2/AF-3).
- **Behaviour-preservation gate.** PHASE-05 EX-4 / VT-2: the existing
  worktree/dispatch suites are the proof that shared machinery (create-fork,
  dispatch funnel) stays green; they must pass unchanged.
- **STD-001.** The `doctrine` token, each subcommand string, the bwrap bind flags,
  and the `ARMING_SUBPATH`/`jail/` path fragments are single-source named constants.
- **SL-183 cross-arm seam (PHASE-02, D8).** The macOS Seatbelt arm (SL-183 /
  IMP-045, `needs SL-182`) reuses `jail.rs` wholesale and forks only the
  argv/profile builder. PHASE-02 therefore factors the `Jailer` seam now (trait +
  `Bwrap` impl), routes capability in as **data** (the shell resolves a `Backend`
  descriptor — `Bwrap | Seatbelt | Deny{reason}` — and `select_jailer(&Backend)` is a
  pure map, so host detection never enters the leaf, RV-202), keeps `opaque_wrap`
  wrapper-agnostic, and locks `validate_policy` platform-agnostic — all zero Linux
  behaviour change (design §10 SL-183 upstream + RV-202 correction; EX-5 / VT-8).
  Building it inline, or as a zero-arg host lookup, would force SL-183 to refactor a
  behaviour-frozen core.
- **Disposability (PHASE-01).** The probe apparatus is throwaway — its EX-4
  requires no committed Rust or installed hooks survive the phase; only the
  recorded findings + decisions persist (notes + a durable memory if novel).

## Critical review (post-author)

Surfaced authoring this plan; resolved here without re-`/design` (none is an
architecture change — they are placement/branch clarifications the design left
loose):

- **`load_policy` pure/impure split.** Design §5.2 lists `load_policy(main_root,
  worktree_name) -> JailPolicy` under the *pure* `jail.rs` core, but a fn that
  defaults on a missing file must read disk — it cannot be pure. §5.1 already
  assigns "policy-file read" to the shell. Resolution: PHASE-02 tests the **pure**
  `JailPolicy` parse / Default / `validate_policy`; the disk read is wired in the
  shell (PHASE-03/04). VT-3 + PHASE-04 EN-2 reworded accordingly.
- **SubagentStop capture module home is unspecified.** ~~Defaulted to
  `src/worktree/capture.rs`.~~ **MOOT @ amendment (2026-07-01):** the capture hook is
  retired; `capture.rs` is deleted in the amended PHASE-05. The live-import gather
  (`gather_worktree_patch`) relocates into `src/worktree/import.rs` behind the
  `--from-worktree` verb (design §5.4 D-import-verb).
- **Path-C abort is a slice exit, not a re-plan.** PHASE-01 EX-2 can refute Path L
  (`SubagentStop` unworkable: not blocking, tree gone, or no correlator). That abort
  **stops this slice and hands the funnel to IDE-024 (Path C)** — it does not spawn
  an alternate plan in-place. PHASE-02..05 as authored are Path L only; if Path L
  falls, they are mooted, not rewritten.
- **Fattest phases: PHASE-03 and PHASE-05.** 03 bundles the shell + two
  registrations + install-time templating + runbook; 05 bundles the capture hook +
  dispatch-skill edits + E2E + OQ-1. Kept whole deliberately (a wall is not real
  until invoked through a resolved binary; the E2E needs capture *and* dispatch
  edits together), but they carry the most risk and should be watched closely at
  phase-plan — split only if execution shows them oversized.
