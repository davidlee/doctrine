# Capsule provisioning and Linux backend

## Context

The first implementation slice of RFC-025's capsule programme. It builds the
trusted side of *starting* a capsule transaction: resolving its contract,
provisioning fresh confined state, and proving the backend's properties.

The target contract is settled and this slice does not relitigate it —
**ADR-020** (accepted) makes the execution capsule the dispatch authority
boundary; **SPEC-030** (active, under SPEC-003, from PRD-015) specifies the
container across REQ-448–REQ-461; **SL-241**'s spike supplies the Linux/bwrap
evidence; **DEC-134** and **DEC-136** settle persistent control-plane
orchestration and the interpretation-policy home. **REV-046** is the governance
cutover Revision and stays `proposed` — it is not this slice's to apply.

### Why this slice exists at this size

SL-248 originally scoped all fourteen SPEC-030 requirements as one cutover.
Measured against this project's own history that was a repeat of two known
failure modes at once: SL-233 ran 16 phases on 781 design lines (~49 per phase)
and left holes that CHR-049 and SL-244 had to patch; SL-244 ran 8 phases on
3459 design lines (~430 per phase) and was uncomfortably inefficient. The
comfortable band here is ~1000 design lines over 6–9 phases. Fourteen
requirements extrapolated past both ends of it.

The programme is therefore decomposed into roughly five slices (provisionally;
see OQ-1). **Shipping is not cutting over** — every slice but the last lands as
tested machinery sitting beside the incumbent worktree arms, unused, with no
flag day. Only the final slice flips the switch. This is that first slice.

### Why REQ-449 is here and not a slice of its own

The decomposition first proposed the `[interpretation]` policy surface as a
standalone precursor, on the theory that it is purely additive config work.
Reading REQ-449's acceptance criteria kills that: its first criterion is
*"**Capsule provisioning** refuses a missing block, missing required key,
unknown key, unsupported schema version, invalid normalized value, or empty
verification sequence"*, and its fourth is the phase-contract
monotonic-restriction algebra — both of which are provisioning-time behaviour,
not parser behaviour. SPEC-030 § Transaction authority says the same thing
structurally: the control plane creates a transaction *from* base + resolved
policy + work contract + capsule identity + resource choices.

Only REQ-449's second criterion (typed parse plus canonical hash) is
independently landable. Splitting there would either ship a parser with no
consumer and retrofit the refusals later, or put the schema design in the
provisioning slice and its implementation in the config slice — backwards. So
contract resolution and provisioning land together.

## Scope & Objectives

**REQ-449 — contract and interpretation provenance.** The required v1
`[interpretation]` block in `.doctrine/doctrine.toml`: typed parse; validation
of `trusted_side_forbidden_executables` (normalized basenames, no slash /
whitespace / empty / `.` / `..`), `interpreted_paths` (normalized
repo-relative gitignore-style patterns; absolute, backslash, NUL, and lexical
`..` refused), and `[[interpretation.verification]]` rows (non-empty `argv` of
non-empty UTF-8); duplicate rejection, then byte-sorted set-valued lists with
verification-row and argument order preserved; one canonical hash over the
typed value. Resolution **once** from the contracted base, bound into the work
contract, never re-resolved from a capsule checkout. Phase-contract refinement
that may add forbidden entries and append verification rows but may not remove,
reorder, widen, or replace — subset validation over normalized typed values,
never source text. Missing block, missing key, unknown key, unknown schema
version, or empty verification sequence refuses provisioning. Extends the
existing `doctrine.toml` parser (DEC-136); does not fork one.

**REQ-450 (partial) — fresh mutable state.** Provisioning from the exact
accepted commit and only explicit immutable inputs, with fresh mutable phase
state. This slice discharges criterion 1 — two phase transactions share no
mutable checkout, repository, runtime, process, or temporary state — and builds
the mechanism criteria 2 and 3 later assert against. See OQ-3.

**REQ-459 — platform backend contract.** The shared property-conformance suite:
fresh mutable state, explicit input set, no writable canonical repo / shared
object store / control-plane state / credentials, bounded host filesystem
visibility, explicit network posture, deterministic working directory,
process-tree teardown, and trusted observation of resource limits and
termination. Plus the Linux/bubblewrap backend implemented against it, recast
from SL-241's rig profile and `src/worktree/jail.rs`'s existing bwrap knowledge.
The suite is the admission gate for any future backend.

**REQ-461 — advisory capacity.** Configurable expected capsule size; a
conspicuous structured warning below threshold (an initial default may warn
below twice the expectation, without reserving); exhaustion halts for manual
intervention and never deletes a capsule or result.

**Also in scope:** answering `QUE-207` as a DEC (see OQ-2) — provisioning is the
first trusted-side code written, so the control-plane topology question gets
decided on concrete ground here rather than in the abstract.

## Non-Goals

Everything downstream of a provisioned capsule belongs to later slices and is
explicitly **not** here: result publication, snapshot, and quarantine ingestion
(REQ-451, REQ-452); trusted conformance over the pinned result (REQ-453);
verification-capsule construction and normalization (REQ-454); the admission
journal and CAS (REQ-455); the capsule-provenance candidate seam (REQ-456);
freeze, repair, and cleanup discipline (REQ-457); the journal/exhibit retention
lifecycle (REQ-458); and the named cutover point with its skill and CLI collapse.

Inherited from SPEC-030 and REV-046, and out of the whole programme:

- **macOS / Seatbelt backend** — unselected until independently specified and
  measured against the REQ-459 suite. No cross-platform parity claim.
- **Egress allowlisting and non-Git build-input provisioning** — `IMP-397` and
  `QUE-204` own it.
- **Capacity reservation, backpressure, eviction, rescue archive** (D7).
- **Retention durations and quota hierarchy** beyond DEC-133/DEC-137.
- **Migrating solo `/execute` worktrees to capsules** — SPEC-012 keeps that
  mechanism and it survives the cutover.
- **Production optimisations** — overlays, snapshots, reflinks, shared caches,
  remote execution.
- **Retiring any incumbent dispatch mechanism.** Marker identity,
  `DOCTRINE_WORKER`, the SubagentStart stamp, `worker_commit`, worktree import,
  and coordination-worktree placement all keep working. This slice is additive.
- **Applying REV-046**, or rewriting RFC-025 beyond its § State of play entry.
- **Migrating the SL-241 rig** (`scripts/spike-capsule/`) into product — its
  hostile rows and stage assertions carry across as *behaviour*, as production
  acceptance tests.

## Affected surface

Coarse and provisional; `/design` fixes the touch-set, and `QUE-207`'s answer
may relocate most of it into a new crate or binary.

| Area | Paths |
|---|---|
| Capsule contract + provisioning | `src/capsule/**` (new) |
| Interpretation policy parse/normalize/hash | `src/dtoml.rs`, `src/dispatch_config.rs`, `.doctrine/doctrine.toml` |
| Linux backend | `src/worktree/jail.rs`, `src/worktree/jail_prefix.rs` |
| CLI surface | `src/commands/**` |
| Property suite + acceptance tests | `tests/**` |

## Risks / Assumptions / Open questions

**OQ-1 — the decomposition is provisional.** The working shape is: (1) this
slice; (2) ingestion and conformance (REQ-451–453); (3) verification and
admission (REQ-454, 455); (4) recovery — candidate provenance, freeze/repair,
retention (REQ-456–458); (5) cutover. Later slices are deliberately **unminted**
— scoping slice 4 before slice 2 is designed is SL-233's failure at a coarser
grain. RFC-025 § State of play carries this as provisional, not settled.

**OQ-2 — `QUE-207` is open and gates design.** *Binary and crate topology for
the control plane*: one `doctrine` binary with environment-derived privilege
(A), a workspace with `doctrine` + `doctrine-control` over shared crates (B, the
frame's provisional choice), or a separate control-plane system (C). Under A
provisioning is a subcommand; under B it is `doctrine-control`'s; under C it is
a service boundary. It must be answered as a DEC at the top of `/design`.

**OQ-3 — three requirements are cross-cutting and close in no single slice.**
REQ-448 (control plane as sole canonical mutation authority), REQ-450
(freshness, whose criteria 2 and 3 need the candidate identity and harvest that
slices 3 and 4 build), and REQ-460 (the non-destructive failure envelope, whose
adversarial matrix spans stale base, candidate conflict, ref movement, and crash
replay). Coverage records per (slice, requirement, **change**), so each can
carry multiple contributing changes and close at the end — but an invariant
owned by every slice is owned by none unless each slice's closure intent names
its obligation explicitly. This slice's obligations: REQ-448's *denial* half
(the backend proves a capsule cannot reach canonical refs, shared object
storage, control-plane state, or credentials — REQ-459's suite is where that is
proven) and REQ-450 criterion 1.

**R1 — evidence altitude.** SL-241 is Linux/bwrap, one client shape, n = 1 on
the real-agent leg. Feasibility evidence, not performance, portability, or
production-readiness evidence. No design or plan claim may exceed it. The
"16/16" summary is forbidden: fifteen rows reached model level, the env-file row
is unproven beyond the Rust fixture, structural `n/a` cells are not omissions,
and four `fail` rows are successful mutant detections.

**R2 — additive, so incumbent suites stay green unchanged.** This slice touches
`src/worktree/jail.rs`, which incumbent dispatch confinement uses. Per AGENTS.md
the existing suites are the behaviour-preservation proof.

**R3 — a property suite is only as good as its adversary.** REQ-459's suite is
the gate every future backend passes. Written weakly it certifies nothing.
SL-241's confinement matrix (P-C2) is the floor, not the ceiling.

**A1** — SPEC-030 and ADR-020 are the authority; where this scope disagrees with
them, they win. **A2** — REV-046 stays proposed and unapplied throughout; this
slice retires nothing. **A3** — the existing `doctrine.toml` parser is extended,
not forked (DEC-136).

## Verification / closure intent

- REQ-449, REQ-459, REQ-461 move `pending → satisfied` with recorded coverage
  (`doctrine coverage record`) naming the discharging test or agent evidence.
- REQ-450 records this slice as a contributing `--change` against criterion 1
  and stays `pending`; likewise REQ-448's denial half via the REQ-459 suite.
  Both are stated as partial in the reconciliation brief, not quietly claimed.
- The REQ-459 property suite passes on Linux/bubblewrap and is structured so a
  second backend is admissible only by passing it independently — no
  Linux-specific assertion leaks into the shared contract.
- REQ-449's refusal cases are `VT` tests over the real parser: missing block,
  missing key, unknown key, unknown schema version, empty verification
  sequence, invalid normalized values, and each phase-contract widening attempt.
- A capsule-side rewrite of `.doctrine/doctrine.toml` demonstrably cannot change
  the bound policy (REQ-449 criterion 3).
- `QUE-207` is answered by an accepted DEC before the design gate clears.
- Existing dispatch, worktree, and confinement suites green **unchanged** (R2).
- `doctrine check gate` green; clippy zero warnings.

## Summary

## Follow-Ups
