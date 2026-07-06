# Implementation Plan SL-206: Workflow-templated slice-driver

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

> **↻ UNJAIL REWORK (2026-07-06). PHASE-01..07 below are FROZEN as the confined-model
> historical record; the live plan is PHASE-08..16 (§ Unjail rework, end of file).**
> The §5 design re-opened from a *confined* orchestrator to a *nominated-unjailed*
> one (`unjail-direction.md` P1/P3/P4); PHASE-06/07's coord-root PLACEMENT fix is
> RETIRED by design §5.4. None of the confined design-targets landed — the Rust
> emitter, `ROLE_PROBE`, `dispatch-probe.md`, and `install/workflows/drive-slice.js`
> do not exist (the POC lived only in gitignored copies). PHASE-08..16 build the
> unjail design from the Rust down. Read 01-07 for the *why* (confined→unjail); read
> 08-16 for the *what*.

The deliverable is Rust-primary (three read-only doctrine tools + a projection
reader + a conformance-authority update + the I1 seam-symmetry check) plus the
nomination/spawn-gate hook config and an install-templated `/drive-slice`
reference workflow. The design (`design.md`, RV-255 + RV-258-hardened) is canon;
this plan sequences it so each phase ends green and the risky premises are proven
before code depends on them.

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

### GPT hostile pass (2026-07-06, post-PHASE-06 authoring)

An external adversarial review (GPT-5.5) over the §5.4 delta + PHASE-06/07 plan +
the authored driver returned four findings; disposition:

- **BLOCKER (padding) — FIXED in-band (`9ba5c0a6`).** The coord-root entry-assert
  interpolated unpadded `SL-${slice}`/`dispatch/${slice}`; the Rust coord authority
  is zero-padded 3-digit everywhere (`.dispatch/SL-{slice:03}`, `dispatch/{slice:03}`
  — `src/dispatch.rs:583`, `src/worktree/coordinate.rs:156`,
  `src/mcp_server/dispatch.rs:136`). Any 1-2 digit slice would false-halt. Now pads.
  Worked for 206/209 only by accident.
- **MAJOR (arming ≠ placement) — FIXED + criterion added.** The assert proves
  cwd/branch, not that the worker forked at the armed base; `classify_create` needs
  `base=Some`, else the same location degrades to `Passthrough` (fresh fork),
  replaying the never-completes defect with a green assert. Driver now instructs an
  arming check (`boundary.code_start == B`, else `halt_reason="funnel:unarmed-fork"`);
  PHASE-07 **EX-5 / VA-2** verify it empirically.
- **MAJOR (forbidden-write probe not in the drive) — scope corrected.** EX-3/VA-1
  read as if the happy-path drive proves the grant boundary; it does not (the loop
  performs no `review_*`/`memory_*` write). PHASE-07 **EX-6** makes the probe an
  explicit standalone step against the installed def — the design's phase-0 grant
  test, run here — not an emergent property of a clean drive.
- **MINOR (VT-1 substring floor is weak) — accepted by design.** Keyword presence
  cannot judge semantics or an absence; that is why VA-1 (isolation absent, meta
  literal, no `run` export) is the real gate. Standard doctrine VT-floor + VA split;
  no change.

## Unjail rework — PHASE-08..16 (the live plan)

The §5 design re-opened from confined to nominated-unjail after PHASE-06/07 proved
a Workflow `agent()` leaf cannot drive committing dispatch (three walls: no `Agent`
tool, RO `.git` + no `DispatchRecord`, worktree-jail deny). The confined *placement*
fix (drop `isolation`, coord-root assert) was the last confined attempt; the
replacement is a durable workflow loop spawning **alternating nominated-unjailed
orchestrators + jailed workers**, escalation closed at the `PreToolUse` spawn seam
(P1/P3/P4). PHASE-01..07 are frozen: they carry the confined criteria and the
executed status, and they are the legible record of *why* unjail.

**Salvage, not rebuild (correction).** An earlier planning pass asserted "nothing
from 01..07 landed as shipped code" — false. The confined PHASE-02/03/04 DID land
~1112 lines on `dispatch/206` via the funnel; it was never integrated (`dispatch
sync` never ran) and the primary phase sheets never flipped, so it read as empty
from the primary tree. It is archived at `SL-206-confined-archive`. Crucially the
**read core is model-agnostic** — reading phase state and the read-only funnel
tools serve a PassThrough orchestrator exactly as a confined one — so PHASE-09/10
**integrate that stranded delta** (`src/dispatch.rs`, `src/mcp_server/dispatch.rs`,
`src/mcp_server/tools.rs`, `src/doctor_checks.rs` allowlist growth) rather than
rebuild it; its tests ride in and the VT mandates go green on arrival. Only the
confined `install.rs` seeding + `install/workflows/drive-slice.js` are genuinely
**superseded** (wrong trust model; the Workflow runtime strips the `Agent` tool) —
those are rebuilt by the unjail defs (PHASE-13) + driver (PHASE-14). So 08, 11, 12,
13, 14, 15, 16 are the real remaining work; 09/10 are integrate-and-conform.

**Why PHASE-08 first (empirical gate, unjail premises).** The unjail design rests
on premises confined PHASE-01 never tested: a *worker* leaf (not a nested confined
orchestrator) forks at the armed base; P5 — which handoff mode a claude worker leaf
runs (B `worker_commit` vs A import); the **Workflow-seam** escalation is denied
(OQ-4); the revive-base mechanism (OQ-6). All cheap to test with shipped machinery
+ the nomination hooks; a failure in a load-bearing premise (fork-at-base, P5,
seam-denial) sends the work back to `/design`, not forward. No receipt code.

**Why the Rust core (09/10) is salvaged and precedes everything else.** The
projection reader + `ReceiptStatus` (PHASE-09) and the three read-only tools +
doctor allowlist growth (PHASE-10) are *identical* under confined or unjail — they
read committed coord truth and add no write surface. Because they are already
built + tested on `dispatch/206`, these two phases integrate that delta (the
orchestrator, sole writer, applies the salvaged patch onto the fresh coord and
conforms) — not a worker rebuild. They neither require nor wait on the PHASE-08
empirical gate (08 gates the unjail-specific machinery 11+). They carry the
behaviour-preservation gate (R5: `run_status` output byte-identical) and the
conformance-ceiling ordering proof (grow `allowed_mcp_tokens` to 6 first, so the
defs' 3 tokens stay `3 ⊆ 6` green until PHASE-13 lists them — reverse it and the
def false-reds against a stale allowlist, RV-255 F1).

**Why the security machinery splits into PHASE-11 (config) then PHASE-12 (checks).**
The nomination + spawn-gate hook config is *data* (SubagentStart allowlist, a
PreToolUse deny-gate over both the `Agent` and `Workflow` seams, SubagentStop
hygiene, the I2 out-of-jail path). The I1 seam-symmetry doctor check is *code* that
reads that config and reds if nomination ⊄ gate **or** a seam registry entry lacks
a matcher (RV-258 F-1: nomination ⊆ gate is necessary, not sufficient). Config
lands first so the check has a real shipped config to green against — the same
grow-then-conform discipline as the allowlist. PHASE-12 also pins the wrong-base
revive rejection (RV-258 F-5): the guarantee lives in `worker_commit`'s existing
`HEAD==base` ∧ single-parent-`==base` check exercised with a revive-base case, never
in a worker obeying a `git reset` prompt.

**Why the driver (PHASE-14) waits on defs + hooks + tools.** `/drive-slice` spawns
`dispatch-probe` (bootstrap O₀ + closing divergence probe) and `dispatch-orchestrator`
(nominated-unjailed), calls the three read-only tools, and relies on the nomination
hooks to flip the orchestrator to `PassThrough`. So it lands after PHASE-13 (defs),
PHASE-11 (hooks), and PHASE-10 (tools). It is the one authored `.js` deliverable and
its home `install/workflows/drive-slice.js` is registered `--intent design-target`
before commit (else audit reds it undeclared — the drift RV-258 F-4 flagged).

**Why the ADR-008 amendment is its own phase (PHASE-15).** The confinement-posture
change is governance the operator reserves; it carries three clauses (reversible
escape hatch, seam-symmetry price conformance-checked, steered-O blast radius named)
and leans on the PHASE-12 doctor check for clause (b)'s "conformance-checked, not
prose". Isolating it keeps the governance authoring — and its `VH` operator
acceptance — a clean gate, unmixed with the driver's mechanics. Draws on
`mem.concept.dispatch.confinement-posture-cost-trajectory`.

**Why PHASE-16 is a separate live acceptance.** The unjail safety claims —
halt-on-red without auto-merge, a forbidden write refused against the *installed*
def (F2 runtime leg, exercised as an explicit standalone probe, not inferred from a
clean drive — RV-258/GPT residual), the Workflow-seam escalation denied, arming
proven (`boundary.code_start == pre-arm B`), the budget loop pacing — only prove out
in a real drive of the full deliverable.

### Touch-set delta under unjail

- **`install/workflows/drive-slice.js` IS now a design-target** (OQ-1 resolved,
  RV-258 F-4) — registered before the PHASE-14 commit, so conformance reds any
  drift from the alternating-unjail topology.
- **New design-target work in `src/doctor_checks.rs`** beyond the allowlist growth:
  `check_spawn_seam_symmetry` + `SEAM_REGISTRY` (PHASE-12). Already a selector.
- **New non-Rust surface** — the nomination/spawn-gate hook config (PHASE-11) and
  its seeding leg in `src/install.rs`; the hook config's authored home is registered
  when it lands, out of every jail (I2). `src/install.rs` rides the `src/**`
  scope-relevant selector.
- **`dispatch-worker.md` stays scope-relevant** (behaviour-preservation, doctor #9
  green unchanged) — unjail does not touch the worker's confinement.

### Verification modes

- **PHASE-08/16 empirical** (VH/VA) — harness fork mechanics, live grant + seam
  enforcement, no-auto-merge, arming are not unit-testable.
- **PHASE-09/10/12 carry the structured VT floor** (the testable Rust core +
  conformance checks).
- **PHASE-11 is VA/VH** — the hook config is JS/JSON, not cargo-testable; its
  mechanical gate is the PHASE-12 doctor check + PHASE-08's live proof.
- **PHASE-13 mixes** a conformance VT + an install VT + inspection VA.
- **PHASE-14 is VT (source-substring floor) + VA (topology/semantics gate)** — the
  driver is `.js`, judged by inspection over the substring floor.
- **PHASE-15 is VA + VH** — governance authoring, operator-accepted.
