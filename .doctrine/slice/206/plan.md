# Implementation Plan SL-206: Workflow-templated slice-driver

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases. The deliverable is Rust-primary (three read-only doctrine tools +
a projection reader + a conformance-authority update) plus an install-templated
`/drive-slice` reference workflow. The design (`design.md`, RV-255-hardened) is
canon; this plan sequences it so each phase ends green and the risky premises
are proven before code depends on them.

## Sequencing & Rationale

**Why PHASE-01 first (empirical gate, not code).** Two premises the whole design
rests on are still unverified against *this* harness: SQ3 (a workflow's confined
fork mints at the armed base — RV-255 R4, leans on SL-199 prior art) and ISS-216
(a *changed* confined agent-def can be made live under `.claude/agents/` — R6).
Both are cheap to test with shipped SL-199 machinery and a scratch slice, and a
failure in either sends the work back to `/design`, not forward. Investing in the
receipt plumbing before this gate would be building on sand. No new code — pure
evidence.

**Why PHASE-02 is the reader core, alone.** The single genuinely-shared,
genuinely-testable seam is the per-phase projection over {plan, sheet, committed
boundaries}. It carries the behaviour-preservation risk (R5): `run_status` must
keep its slice-global aggregate output byte-identical while delegating only its
per-phase rows. Isolating that refactor in its own phase keeps the gate — the
existing `dispatch status` suite green *unchanged* — a clean pass/fail, unmixed
with new tool surface. `ReceiptStatus` lands here because the projection is what
derives it (Completed from the committed boundary, Blocked/Unknown fail-loud —
RV-255 F4).

**Why the tools (PHASE-03) precede the defs (PHASE-04) — a conformance ordering
proof.** Doctor check #9 is a *ceiling* ("extras rejected; tokens need not all be
present", `src/doctor_checks.rs`). Granting the orchestrator three new MCP tokens
means growing `allowed_mcp_tokens` FIRST: with the allowlist at six and the def
still listing three, `3 ⊆ 6` stays green. Then PHASE-04 adds the tokens to the def
(`6 ⊆ 6`, green). Reverse that order and the def would false-red against a stale
allowlist between phases (RV-255 F1). PHASE-03 also teaches the marker validator
the new `probe` role (it currently accepts only `worker|orchestrator`) — without
it, the read-only `dispatch-probe` def (RV-255 F5) is rejected as an invalid
marker.

**Why the read-only role is its own surface (F5).** The bootstrap planner and
closing divergence probe must be genuinely read-only; reusing `dispatch-orchestrator`
(which carries raw `Edit`/`Write`/`Bash`/`Agent`) would make "read-only" a lie
(CHR-039: containment is by policy, not MCP-unreachability). `dispatch-probe`
holds `Read`/`Grep`/`Glob` + the three reads only.

**Why PHASE-05 is a separate live acceptance.** PHASE-04's script is verified by
inspection + static conformance; but the load-bearing safety claims — halt-on-red
without auto-merge, a forbidden write refused against the *installed* def (F2's
runtime leg), the divergence advisory emitting without gating, the budget loop
pacing — only prove out in a real drive. This phase overlaps PHASE-01's SQ3 demo
but now exercises the full deliverable, not shipped machinery alone.

## Declared touch-set (audit-clean)

The 7 `design-target` selectors are `src/dispatch.rs`, `src/mcp_server/dispatch.rs`,
`src/mcp_server/tools.rs`, `src/doctor_checks.rs`, and the three
`install/agents/claude/*.md` defs (orchestrator, worker, probe). Two accuracy
corrections carried into `plan.toml`:

- **`/drive-slice` home is not yet a design-target** (PHASE-04 EX-6). No selector
  covers `install/workflows/**`; `.claude/workflows/**` is scope-relevant *and
  gitignored* (installed copy only). The authored script's committed home must be
  registered `--intent design-target` the instant OQ-1 fixes it, or audit
  conformance reds it as undeclared. This is the one HIGH gap.
- **`dispatch-worker.md` reclassified to `scope-relevant`.** No phase modifies it;
  it is a behaviour-preservation assertion target (doctor #9 stays green
  unchanged), not a modification target — a `design-target` with no source delta
  invites an under-delivery flag at audit.
- **`src/install.rs`** (probe asset registration + a workflows seeding leg, if
  EX-7 keeps the "seeded by install" claim) rides the broad `src/**`
  scope-relevant selector — touched but not audit-red.
- **No flake change for the new `.js`.** `srcWithDist` does `cp -R ./install/.`
  and RustEmbed `#[folder="install/"]` embeds recursively; a new
  `install/workflows/*.js` ships without grafting (AGENTS.md nix-embed hazard does
  not bite here).

## Trunk authority (divergence tool)

`dispatch_authored_divergence`'s `trunk_ref` resolves from the real authority —
`git::trunk_commit` (`src/git.rs`) or the `[dispatch] deliver_to` ref
(`run_deliver_to`, `src/dispatch.rs`) — **not** `run_show_journal_trunk_oid`
(`src/dispatch.rs:630`), the journal-row printer that design §5.5 loosely cites.
PHASE-03 EX-8 pins this.

## Notes

- **Verification modes.** PHASE-01/05 are inherently empirical (VH/VA) — harness
  fork mechanics, live grant enforcement, and no-auto-merge are not unit-testable.
  PHASE-02/03 carry the structured VT floor (the testable core). PHASE-04 mixes
  a conformance VT with inspection VA.
- **OQ-1 (script home)** stays open into PHASE-04: `install/workflows/` does not
  exist yet; the exact authored home (install-templated vs harness-local) is
  settled there, informed by PHASE-01's ISS-216 finding.
- **ISS-216 dependency.** Out of scope to fix; SL-206 leans on the reseat/manual-
  placement procedure PHASE-01 characterises until ISS-216 lands (R6).
- **No auto-land (IMP-174).** No phase crosses the authored split-brain; landing
  stays `/audit → /reconcile → /close`. The driver reports; the divergence
  advisory is raw signal only.

## PHASE-06/07 — rework (appended after PHASE-05 acceptance)

PHASE-05 drove the shipped `/drive-slice` live and found it never completes a
phase. Root cause (reconciled against SL-199, corrected in notes.md FINDING 3 +
design §5.4 delta `ef23828f`): a **placement** defect, not the orchestration
model. The confined orchestrator is proven-realizable (SL-199 Mode B) — RO shared
`.git` is by design, coord `.git` writes go through the server-side MCP funnel.
§5.4 merely spawned the orchestrator with `isolation:'worktree'`, fresh-forking it
away from coord-root where the SL-199 arming discriminator (`cwd_is_coord_root`)
never fires. A placement spike + `docs/claude/subagents.md:263` confirmed a
no-isolation subagent inherits the driver's coord-root cwd — the fix.

- **Why two phases.** The fix (PHASE-06) is a source correction with a testable
  presence floor (VT) plus absence/correctness inspection (VA) — `agent()` carrying
  no `isolation` is an *absence*, which the VT keyword floor cannot assert, so it is
  a VA. The witness (PHASE-07) is the inherently empirical VH-1 live drive SL-199
  left owed; it carries no VT (a live armed loop is human/agent-judged, and the
  SL-209 test files land on a different coord tree than SL-206's `verify-vt` reads).
- **Immutability.** PHASE-01–05 are untouched; the rework appends. PHASE-06/07
  carry the corrected criteria, including the two internal-review residuals: the
  orchestrator entry-assert wording (PHASE-06) and the coord-root invoke ritual
  (PHASE-07).
- **Landing.** The deliverable is authored on `dispatch/206` (its existing home) —
  one home, no new split-brain (IMP-174) — reseated live for the PHASE-07 drive,
  integrated at `/close`. It is a shipped script, edited by the sole writer, not
  driven through a worker (which would also mean repairing the machinery with the
  machinery under repair). OQ-1 (script home) thereby resolves to `dispatch/206`.
