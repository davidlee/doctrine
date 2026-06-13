# Implementation Plan SL-056: Orchestrator spawn seam: worktree mechanism into CLI verbs

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-056 moves worktree/dispatch **mechanism** out of fail-open skill prose into
fail-closed, golden-testable CLI verbs, with an **orchestrator-owned fork + a disk
marker** as the harness-agnostic worker identity (design §1). The thirteen phases
fall into two bands the design's §10 sequence makes non-negotiable: **decisions
govern, so they land first** (four governance deliverables), then the code that
mechanises them, then the prose carriers that *call* the shipped verbs.

The single ordering law across the whole plan: **the spike precedes what it
validates, the verb precedes its caller, the trust core precedes every verb, and
the prose comes last.** Everything below is that law applied.

## Sequencing & Rationale

### Why governance first (PHASE-01 → PHASE-04)

The slice's central risk is under-delivery: if the design merely *assumed* the
ADR/spec decisions, the slice would ship code against ungoverned premises. So the
governance drafts are **deliverables of this slice**, not inputs to it (slice
§Sequencing). The order inside the band is forced by what depends on what:

- **PHASE-01 (G1+G3)** lands the gate. ADR-008 must reach `accepted` before the
  IMP-004 work it governs (the per-worktree-target premise, D-B1) can proceed; the
  new spawn-interface ADR (G3) must state the agnostic *contract* and the
  per-harness altitude table before any other decision can reference it. The
  fail-closed claude cell is left `proposed` here on purpose — it is **φ
  two-valued and O3-spike-contingent**, and PHASE-01 must not let the headline
  outrun the footnote.
- **PHASE-02 (O3 spike)** sits between G1+G3 and G2 exactly as design §10 dictates:
  "the guard+privilege spike precedes the ADR-006 amend it validates." The claude
  WorktreeCreate path is the one assumption the design cannot settle by reasoning —
  whether a *named* `dispatch-worker` subagent propagates `agent_type`, whether a
  hook **replaces** creation, and (the hard gating outcome, σ) whether a **matcher**
  can scope the hook. G2 and the `create-fork` else-branch both branch on the
  answer, so the answer must exist first.
- **PHASE-03 (G2)** amends the *accepted* ADR-006 only after the spike validates
  the path it encodes, and in the same pass **firms the φ cell** in the
  spawn-interface ADR to the spike outcome. Targeted edits, not a rewrite — the
  withheld-tier D1/D4/D9 invariants survive untouched (behaviour-preserving
  governance).
- **PHASE-04 (G4)** rewrites SPEC-012 last in the band, because it is downstream of
  every ADR decision: the funnel philosophy it must invert ("a discipline, not
  enforced code") only becomes false once the verbs are governed into existence.

### Why this code order (PHASE-05 → PHASE-11)

The trust topology is the spine, so it lands first and everything rides it:

- **PHASE-05 (trust core)** is the keystone: the disk marker, `worker_mode`, the
  `run()` guard, the `WriteClass` enum (now discriminating `Orchestrator`/
  `Hook-mint`), the bespoke `marker --clear`, and the `worktree status [--assert]`
  observability+gate verb. A design-wide subtlety lives here: **`write_class` is
  exhaustive — there is no wildcard arm**, so a new verb-`Command` is a compile
  error until classified. PHASE-05 establishes the enum + guard and classifies the
  *existing* surface behaviour-preservingly plus the marker/status verbs it
  introduces; **each later phase classifies its own new verb** (`fork`/`import`/
  `land`/`gc` → `Orchestrator`; `create-fork`/`marker --stamp-subagent` →
  `Hook-mint`; `claude install` → `write`). This is why the worker-mode refusal
  tests for, e.g., `claude install` live in PHASE-11, not PHASE-05 — the verb does
  not exist until then. Each Orchestrator verb proves its **own** run()-driven
  refusal in its introducing phase (fork/import/land at PHASE-06/07/08), and
  **PHASE-09 owns the EXHAUSTIVE `{fork,import,land,gc}` refusal test** (the
  round-7 Charge π fix, design §12) — PHASE-09 is the first phase where all four
  verbs coexist. The slice's most load-bearing trust guarantee thus has a single
  named owner, not a scatter of EX assertions.
- **PHASE-06 (`fork` + per-wt env)** is the codex/pi orchestrator-owned creator and
  the marker's codex/pi writer. It carries O6's per-worktree env contract because
  the contract is *emitted by fork on stdout* — the same atomic act. Its
  compensating-cleanup rollback core is deliberately built here so **PHASE-10's
  `create-fork` can reuse it** (one cleanup implementation, two callers) — a
  forward dependency that fixes fork before the claude path.
- **PHASE-07 → PHASE-09 (`import` → `land` → `gc`)** are the funnel verbs. `import`
  (dispatch route, ancestry severed) and `land` (solo route, ancestry preserved)
  are the two landing routes; `gc`'s **two-leg oracle** certifies *both*, so gc must
  come after both exist. `land` reuses `import`'s tree-clean precond (no parallel
  implementation), so import precedes land. The belt (R1, "the real protection")
  lands inside `import` and is tested by an invariant that **drives the write seam**,
  not a pure helper.
- **PHASE-10 (claude spawn path)** depends on two upstreams: PHASE-02 fixes the
  else-branch disposition (matcher-green deletes it; matcher-red keeps the
  serviceable default) and the fallback rung; PHASE-06 supplies the rollback core it
  reuses. `create-fork` is the verb the install hook will call, so it precedes the
  installer.
- **PHASE-11 (install surface)** wires the WorktreeCreate hook to `create-fork` and
  symlinks the `dispatch-worker` agent def — both must exist first (PHASE-10). The
  `claude install` rename reuses the existing `HookSpec` merge core (no parallel
  merge), and its orphan-reference sweep (SR-3, the hidden `skills install` alias +
  the memory update) is an explicit deliverable so the rename does not strand
  references.

### Why the spike and prose come last (PHASE-12 → PHASE-13)

- **PHASE-12 (D6 bwrap spike)** is codex/pi-only and depends on O6 (it confines rw
  to the per-wt target). Its only real dependency edge is **PHASE-06** — it is
  **parallelizable any time after fork lands** and is slotted at 12 purely to keep
  it *before the prose* (PHASE-13's `/dispatch-subprocess` must describe whether
  bwrap confinement is real). An executor with spare capacity may pull it forward
  to run alongside the funnel verbs. It is timeboxed and **may back out** — the
  design frames both outcomes symmetrically, so the phase's exit is "a clear
  land/back-out decision," never an ambiguous result.
- **PHASE-13 (prose + router)** is genuinely last: prose that *calls* a verb is a
  lie until the verb ships. It also lands the `/dispatch-*` harness-routing split
  (codex/pi subprocess vs claude `Agent`) — distinct from SL-055's audience-homing
  split (Non-Goals) — and closes with the full `--workspace` gate as the slice's
  final conventions check.

## Notes

- **Two spikes, empirical verification.** PHASE-02 (O3, claude WorktreeCreate) and
  PHASE-12 (O7, nested bwrap) are investigations, not constructions — verified by
  recorded result (VA) and gated forward into the decisions/code they feed. A red
  spike is a *recorded* outcome with a confessed fallback, never a silent gap.
- **Governance verified by VA + VH.** The ADR/spec phases assert document content
  (VA) and a governance acceptance gate (VH) where a status reaches `accepted`. The
  spawn-interface ADR is cited by its **minted canonical id** (via `doctrine adr
  new`) — never the pre-guessed "ADR-011".
- **Behaviour-preservation is precise (R2).** The migration legitimately *changes*
  the worker-mode trigger (env → marker), so the old `DOCTRINE_WORKER`-only guard
  tests are **rewritten**, not kept green. What stays green *unchanged* is the
  preservation proof: `select_copies`/provision, `branch-point-check`,
  `is_withheld`/allowlist, and the `git.rs` born-frame capture.
- **Deferred, not dropped.** IMP-043 (moved-HEAD `--allow-reanchor`), IDE-004
  (channels claude env backend), IMP-045 (macOS OS-confinement), and IDE-005
  (harness detection into the binary) are named follow-ups carried by the design's
  §13, out of v1 scope.
