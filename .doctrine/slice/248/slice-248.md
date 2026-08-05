# Capsule dispatch v0

## Context

This is the implementation slice RFC-025 § State of play calls for in
next-obvious-action 2: *"Scope the implementation slice from SPEC-030, then
design and plan it without starting implementation."*

The governing target contract already exists and is settled:

- **ADR-020** (accepted) — execution capsules are the dispatch authority
  boundary. A persistent trusted control plane; fresh headless phase and
  verification capsules; bundle-snapshot ingestion; journaled admission;
  capsule-provenance candidate recovery; frozen-source/fresh-repair lifecycle.
- **SPEC-030** (active, tech container under SPEC-003, descends from PRD-015) —
  *Dispatch execution capsules*. Twelve functional and two quality
  requirements, REQ-448–REQ-461, all `pending`. This slice's job is to move
  them to satisfied.
- **REV-046** (proposed, unapproved) — the governance cutover Revision. It is
  explicitly *not* the implementation: "implementation itself — this Revision
  is the governance boundary that a later slice must satisfy, not the slice."

The evidence base is `SL-241`'s completed spike (the capsule spike rig) and the
banked artefacts under `.doctrine/rfc/025/` — `mechanism-census.md`,
`red-team.md`, `probe-specs.md`, `evidence/`, `go-no-go.md`. Its verdict is
**go**, scoped: measured on Linux/bwrap, one client shape, n = 1 for the
real-agent phase, with the seven limits in `evidence/README.md` attached to
every claim. The rig is disposable; its hostile rows and stage assertions
become production acceptance tests, they are not migrated.

The settled epistemic inputs are `DEC-133` (durable admission journal vs
expiring forensic exhibit), `DEC-134` (persistent control-plane orchestration,
fresh headless phase worker), `DEC-135` (bundle ingestion), `DEC-136`
(`[interpretation]` policy in `.doctrine/doctrine.toml`, resolved from the
contracted base), `DEC-137` (same-base candidate admission, frozen source
capsules, fresh repair). Their questions `QUE-200`/`QUE-201`/`QUE-202` are
answered and terminal.

### Why now, and why one slice

REV-046's apply gate 3 requires *"a planned implementation"* that proves
uniform headless worker launch, contracted bases, Linux/bwrap confinement,
trusted-side ingestion, separate-capsule verification, normalization, durable
admission journaling, and crash-safe admission/integration. Gate 4 requires
that implementation to **name the exact compatibility/cutover point**. So the
approved design and plan of *this* slice are an input to REV-046's approval —
not the reverse. Sequencing is therefore:

    SL-248 scope → design → plan  →  REV-046 approve + apply  →  SL-248 execute

Capsule dispatch does not ship in halves: a partial cutover leaves the repo
neither on the incumbent worktree arms nor on capsules. The requirement set is
large, so the decomposition is carried by **phases**, not by sibling slices —
see OQ-1 if that judgement should be revisited at design time.

## Scope & Objectives

Satisfy SPEC-030's requirement set at v0 altitude, on the Linux/bubblewrap
backend, reusing existing seams rather than building parallel machinery.

**In scope**

1. **Authority split and capsule lifecycle** (REQ-448, REQ-450) — the trusted
   control plane as sole canonical mutation authority; the transaction
   lifecycle `resolve → provision → launch → notify → snapshot → harvest/freeze
   → conform → verify → normalize → journal → admit/integrate → close →
   explicit cleanup`, each transition with one trusted writer and a durable
   state. Fresh mutable state per phase worker and per verifier.
2. **Contract and interpretation provenance** (REQ-449) — the required v1
   `[interpretation]` block in `.doctrine/doctrine.toml`: typed parse,
   validation, normalization, canonical hash; resolution once from the
   contracted base; monotonic-restriction-only phase contract refinement.
   Extends the existing `doctrine.toml` parser (`src/dtoml.rs`,
   `src/dispatch_config.rs`), it does not fork one.
3. **Result publication and hostile ingestion** (REQ-451, REQ-452) — worker
   publishes one Git bundle; the control plane makes one bounded immutable
   parent-owned snapshot under path/symlink/quiescence/byte/time/object bounds;
   trusted Git reads only that snapshot through a fresh disposable quarantine
   repository; no fetch-from-capsule path, primary or fallback.
4. **Trusted conformance** (REQ-453) — pin exactly one result identity; check
   contracted-base ancestry, merge shape, actual changed paths, declared phase
   scope, forbidden paths, modes, gitlinks/submodules, and interpretation
   obligations, all from Git objects. Reuses the declared/changed selector
   algebra (`src/conformance.rs`, `slice selector`) rather than restating it.
5. **Separate-capsule verification and normalization** (REQ-454) — construct
   the exact normalized candidate; provision a fresh verification capsule from
   that immutable identity and the bound policy; run the declared verification
   rows in order without trusted-side shell evaluation; the verifier process
   result plus trusted evidence capture is the verdict. Journaled verified
   identity == later admitted identity.
6. **Journaled admission and CAS** (REQ-455) — journal intent before any
   canonical mutation; precheck the expected accepted tip before object
   transfer; one expected-old-object compare-and-swap; idempotent replay
   classifying already-applied / still-applicable / diverged. Rides the
   existing CAS and journal-before-mutation substrate (SPEC-022, `src/git.rs`,
   `src/ledger.rs`).
7. **Capsule-provenance candidate recovery** (REQ-456) — stale results enter
   the *existing* object-only candidate engine through an explicit
   capsule-provenance seam (current accepted commit, pinned source commit,
   contracted base, verification attestation) with no incumbent
   coordination-journal consultation. Created / Conflicted are durable; every
   clean, hand-resolved, and fix-on-top candidate is freshly verified at its
   exact immutable commit before admission; a second accepted-tip movement
   records explicit supersession.
8. **Frozen source, fresh repair, cleanup discipline** (REQ-457) — repair is a
   new transaction from the current accepted commit with the frozen result as
   input; cleanup requires mechanically recorded incorporation, integration
   plus formal closure, or explicit operator abandonment. Tree similarity never
   authorizes cleanup.
9. **Journal, live-work, and forensic-exhibit lifecycle** (REQ-458) — durable
   compact admission journal; unresolved source capsules are non-evictable live
   work; post-close exhibits may expire without rewriting journal truth.
10. **Platform backend contract** (REQ-459) — a shared property-conformance
    suite (freshness, explicit inputs, bounded filesystem and network reach,
    process-tree teardown, resource observation, denial of canonical state and
    credentials); Linux/bubblewrap implemented and measured against it.
11. **Non-destructive failure envelope** (REQ-460) — adversarial coverage of
    failed verification, malformed result, stale base, candidate conflict,
    repeated ref movement, crash replay, and low capacity; no force-update, no
    auto-resolution, no capsule resumption, no automated loss of unresolved
    work.
12. **Advisory capacity handling** (REQ-461) — configurable expected capsule
    size, conspicuous structured low-space warning, halt-for-manual-intervention
    on exhaustion; no reservation, backpressure, eviction, or rescue archive.
13. **The named cutover point** (REV-046 gate 4) — the exact commit/flag/verb at
    which no dispatch run can still depend on worktree marker identity,
    `DOCTRINE_WORKER`, the SubagentStart hook stamp, the gated `worker_commit`
    tool, patch/worktree import, or coordination-worktree placement. Naming it
    is in scope; *executing* the governance retirement is REV-046's apply.
14. **Production acceptance tests** derived from SL-241's hostile matrix rows
    and stage assertions — carried across as behaviour, not as rig code.

**Out of scope** — see Non-Goals.

## Non-Goals

Inherited verbatim from SPEC-030 § Overview and REV-046 § Deliberately outside:

- **macOS / Seatbelt backend** — unselected until independently specified and
  measured. No claim of cross-platform parity.
- **Egress allowlisting and non-Git build-input provisioning** — `IMP-397` and
  `QUE-204` own this separately.
- **Retention durations, quota hierarchy, project/slice/machine policy
  layering** beyond the DEC-133/DEC-137 live-work / journal / exhibit
  separation.
- **Capacity reservation, throughput backpressure, capsule eviction, rescue
  archive** — deferred by D7 because guessing early could destroy work.
- **Migrating solo `/execute` worktrees to capsules** — solo worktrees remain a
  supported non-dispatch isolation mode; SPEC-012 keeps that mechanism.
- **Production optimisations** — overlays, snapshots, reflinks, shared caches,
  remote execution.
- **Applying REV-046** — this slice supplies the design/plan its gates need and
  names the cutover point; the Revision's approve/apply is its own act, and it
  precedes phase execution (see Context § Why now).
- **Rewriting RFC-025 prose** or migrating the SL-241 spike rig
  (`scripts/spike-capsule/`) into product.
- **A second conflict system** — DEC-137 forbids it; the candidate engine is
  reused behind a provenance seam.

## Affected surface

Coarse and provisional — `/design` fixes the exact touch-set. Seeded as
`scope-relevant` selectors.

| Area | Paths | Expected shape |
|---|---|---|
| Capsule transaction engine | `src/capsule/**` (new) | new module: contract, provision, launch, snapshot, quarantine, harvest, verify, normalize |
| Dispatch orchestration | `src/dispatch.rs`, `src/funnel_machine.rs`, `src/dispatch_config.rs` | funnel/state-machine investment transfers; arm routing and altitude retire |
| Worktree machinery | `src/worktree/**` | `marker.rs`, `subagent.rs`, `import.rs`, `fork.rs`, `pretooluse.rs`, `dispatch_record.rs` retire or narrow to solo; `jail.rs`/`jail_prefix.rs` recast as the measured Linux backend; `create.rs`/`provision.rs`/`gc.rs`/`land.rs` keep their solo legs |
| Git / admission substrate | `src/git.rs`, `src/ledger.rs`, `src/conformance.rs` | reused: OIDs, merge-tree, CAS, candidate rows, selector algebra; extended with a capsule-provenance seam and admission journal |
| Config | `src/dtoml.rs`, `.doctrine/doctrine.toml` | required `[interpretation]` v1 block: parse, validate, normalize, hash |
| CLI + MCP surface | `src/commands/**`, `src/mcp_server/**` | new capsule verbs; `worker_commit` retires at the cutover point |
| Skills / docs | `.agents/skills/dispatch*`, `install/**` | dispatch skill arms collapse to one headless launch path |
| Tests | `tests/**` | acceptance tests derived from SL-241's hostile matrix and stage assertions |

## Risks / Assumptions / Open questions

**OQ-1 — one slice or several?** SPEC-030 carries 14 requirements across
provisioning, ingestion, conformance, verification, admission, candidate
recovery, repair, retention, and backend conformance. This scope treats them as
one shippable cutover decomposed by phases, because a half-cutover is a state
the repo cannot sit in. If `/design` finds an independently shippable
sub-boundary — most plausibly the `[interpretation]` config surface (REQ-449),
which is additive and can land before any capsule exists — split it out rather
than carrying it as dead weight.

**OQ-2 — `QUE-207` is open and blocks design.** *Binary and crate topology for
the control plane* (open, 2026-08-05, shapes RFC-025) asks where the authority
boundary is expressed: one `doctrine` binary with environment-derived privilege
(frame option A), one workspace with `doctrine` + `doctrine-control` binaries
over shared crates (option B, the frame's provisional choice), or a separate
control-plane system (option C). This is upstream of nearly every module
decision in the table above. It should be answered — as a `DEC` — during or
immediately before `/design`, not discovered mid-plan.

**OQ-3 — the cutover point's shape.** REV-046 gate 4 wants an exact point.
Whether that is a flag day, a config-gated dual-run window, or an atomic
release is a design decision with real consequences for how many phases can be
green in isolation.

**OQ-4 — what replaces `review/*` and `phase/*` refs.** REV-046 § ADR-012
explicitly leaves the population of these refs "a target-design question, not a
DELETE-by-count conclusion." The admission journal and human-facing evidence
views must supply the same durable guarantees before they change.

**R1 — evidence altitude.** SL-241 is Linux/bwrap, one client shape, n = 1 for
the real-agent leg. It is feasibility evidence, not performance, portability,
or production-readiness evidence. No design or plan claim may exceed it, and
the "16/16" summary is explicitly forbidden (fifteen rows reached model level;
the env-file row is unproven beyond the Rust fixture; structural `n/a` cells
are not omissions; four `fail` rows are successful mutant detections).

**R2 — behaviour-preservation on shared machinery.** The candidate engine, CAS,
selector algebra, and journal are shared with non-dispatch paths. Per AGENTS.md
the existing suites are the proof: they stay green unchanged.

**R3 — bootstrapping.** Doctrine dogfoods its own dispatch. The cutover has to
be executable *by* the machinery being replaced, and the incumbent arms must
stay usable until the named cutover point.

**R4 — census verdicts are target-state, not a delete list.** RFC-025 § State
of play: the mechanism-census verdicts "describe the capsule target state, not
mechanisms already retired." REV-046 gate 5: every requirement gets an explicit
keep / transform / retire / solo-scoped disposition; nothing retires solely
because a census row says DELETE.

**A1** — SPEC-030 and ADR-020 are the authority; where this scope and they
disagree, they win. **A2** — REV-046 stays proposed through design and plan,
and is approved and applied before phase execution begins. **A3** — the
existing `doctrine.toml` parser is extended, not forked (DEC-136).

## Verification / closure intent

Closure is judged by SPEC-030's own acceptance criteria, not by this document.

- All fourteen requirements REQ-448–REQ-461 move `pending → satisfied`, each
  with recorded coverage (`doctrine coverage record`) naming the test or agent
  evidence that discharges it.
- The REQ-459 shared property-conformance suite passes on Linux/bubblewrap and
  is structured so a second backend can be admitted only by passing it
  independently.
- REQ-460's adversarial cases are `VT` tests, not attestations — failed
  verification, malformed result, stale base, candidate conflict, repeated ref
  movement, crash replay, low capacity — each proving the last accepted
  canonical state and all unresolved source work survive.
- Existing dispatch, candidate, CAS, and conformance suites are green
  **unchanged** (R2).
- The cutover point is named in the design and demonstrated: after it, no
  dispatch path reads a worktree marker, `DOCTRINE_WORKER`, the SubagentStart
  stamp, or the `worker_commit` gate.
- REV-046's six apply gates are satisfiable from this slice's artefacts, and
  the Revision is approved and applied before phase execution.
- `doctrine check gate` green; clippy zero warnings.

## Summary

## Follow-Ups
