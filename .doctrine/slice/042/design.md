# Design SL-042: Reconciliation observe substrate (SPEC-002 A)

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Build the **observe** half of SPEC-002 (Requirement Reconciliation Engine): the
two-tier truth substrate that stores observed coverage evidence, derives a
per-requirement composite and a drift read, and **never derives authored status
from coverage** (the load-bearing NF-001 line). Greenfield only where the spec is;
it rides foundations that already ship.

Scope = four phases: **P1** REC record kind · **P2** coverage substrate · **P3**
composite view + drift surfacer · **P4** VH/VA staleness decay. Realises
REQ-108/109/110/111 (FR) and REQ-114/115 (NF-001/NF-002). The reconcile *writer*,
the requirement-status/spec-truth write seam, and the closure gate are the
dependent **Slice B**, deliberately excluded (§3).

## 2. Current State

Foundations already shipped (SL-007/008, SL-015, SL-028, SL-040 in flight):

- **The two truth enums already exist** in `src/requirement.rs` (SL-028,
  vocabulary-only): `ReqStatus` {Pending, InProgress, Active, Deprecated, Retired,
  Superseded} — authored, already a field on the `Requirement` entity; and
  `CoverageStatus` {Planned, InProgress, Verified, Failed, Blocked} — observed
  evidence, a **stub** behind a self-clearing `#[cfg_attr(not(test),
  expect(dead_code))]`, explicitly awaiting *"the reconcile engine, the coverage
  join"* consumer. **SL-042 is that named consumer.**
- **The NF-001 contract is already written in code** (`src/requirement.rs`
  L83–88): *"deliberately no `ReqStatus = f(CoverageStatus)` mapping… the two enums
  never reference each other… the reconcile writer is a deferred follow-on; the
  absence here is the contract, not an omission."*
- **The numbered-kind seam** — `integrity::KINDS` / `KindRef` (`kind`, `stem`,
  `state_dir`), the manifest + `.gitignore`-negation wiring
  (`mem.pattern.install.authored-entity-wiring`), `NNN-slug` aliases, and (in
  flight under SL-040) `meta::read_id`, the **status-less scan-path reader** a kind
  with no authored `status` field uses in `scan_kind`.
- **The git staleness seam** — `src/git.rs` born-frame/anchor + verify/staleness
  machinery (SL-007/008), the VH/VA decay mechanism (NF-002/H1) reuses unchanged.

Nothing today stores coverage, emits a REC, or surfaces drift.

## 3. Forces & Constraints

- **Governance.** ADR-003 (canonical loop; explicit-authorship-not-derivation),
  ADR-009 (slice FSM, conduct axis, two-enum truth model), ADR-004 (relations
  outbound-only — *why* reverse lookups are scans, §5.1), ADR-001 (leaf←engine←
  command layering). SPEC-002 D1–D9 / H1–H5 are the governing technical decisions —
  not re-decided here; tensions go back through `/consult`.
- **No parallel implementation.** Reuse the SL-028 enums, the SL-040 kind seam +
  `read_id`, and the `src/git.rs` staleness seam verbatim.
- **Pure/imperative split.** The composite and drift folds are pure (no
  clock/RNG/git/disk, no map-iteration-order dependence); I/O lives in the shell.
- **Two-tier purity (load-bearing).** Coverage and authored requirement status
  occupy **distinct stores** and never touch through a function.

## 4. Guiding Principles

- **Surface, never resolve.** This slice only *stores evidence* and *derives
  reads*. It has **no write path** to authored requirement or spec truth — that is
  the structural proof of NF-001 in SL-042, and the reason the reconcile writer is
  Slice B.
- **Conservative derivation.** Derived reads refuse to silently call ambiguous
  cases coherent (§5.2 coherence predicate); ambiguity is surfaced for judgement.
- **Correctness is recomputation.** No composite/drift value is stored; any cache
  is disposable.

## 5. Proposed Design

### 5.1 System Model

```text
.doctrine/rec/NNN/            REC corpus — NEW numbered kind (status-less)
  rec-NNN.toml  rec-NNN.md
.doctrine/rec/NNN-slug -> NNN alias (reused machinery)

.doctrine/slice/NNN/coverage.toml   coverage substrate — keyed table, authored
                                    (committed, diffable), NOT a numbered kind
requirement REQ-X authored status   distinct store ── NF-001 line: never f(coverage)

src/rec.rs       (new)  REC kind: schema, scaffold/show/list; reuses meta::read_id
src/coverage.rs  (new)  entry I/O (shell) + PURE folds: composite(), drift()
src/integrity.rs (+row) REC_KIND in KINDS (stem "rec", state_dir None)
src/git.rs       (reuse) VH/VA staleness — no new fork
install/manifest.toml (+ .doctrine/rec) · .gitignore (+ !.doctrine/rec/)
```

Cross-slice fan-in (**R2**, Q2): the composite reader walks
`.doctrine/slice/*/coverage.toml` and filters by requirement — a **corpus scan,
no reverse index**, matching SL-040's reverse close-gate (D-C9b) and the ADR-004
outbound-only grain. Scan cost is bounded by a perf spike (§9), not assumed.

### 5.2 Interfaces & Contracts

Two pure folds in `coverage.rs` (signatures illustrative; CLI shapes settle at
build per SPEC-002 D9):

```text
composite(entries: &[CoverageEntry]) -> Composite
    fan-in of one requirement's entries across contributing changes:
    modes present, per-mode statuses, staleness flags. Deterministic.
    NOT persisted (D4). v1 surfaces all; no precedence weighting (OQ-3).

drift(authored: ReqStatus, composite: &Composite) -> Verdict
    read-only. Returns NO ReqStatus. Verdict ∈ {Coherent, Divergent(reason),
    Indeterminate}.
```

**The v1 coherence predicate** (deliberately conservative — not a precedence
engine; honours the OQ-3 deferral while giving FR-004 its verdict):

- **Coherent** — only unambiguous alignment: authored `Active` ↔ a fresh
  `Verified` with no `Failed`; authored `Pending`/`InProgress` ↔ observed
  `Planned`/`InProgress` (forward intent — PRD-013 "not drift when grounded").
- **Divergent(reason)** — unambiguous contradiction: any `Failed`/`Blocked` under
  an in-force authored status; an in-force authored status with zero confirming
  evidence.
- **Indeterminate** — everything else, incl. a **stale** `Verified` (NF-002 —
  flagged, never auto-demoted): surfaced for the writer to judge.

At the Slice-B closure gate, `{Divergent, Indeterminate}` both read as
*unreconciled* — consistent with FR-004's binary (coherent vs drifted); the reason
rides along. Collapsing `Indeterminate` via precedence is the OQ-3 follow-on.

### 5.3 Data, State & Ownership

**REC** `rec-NNN.toml` (status-less — §7 D-Q3): `status_deltas =
[[requirement, from, to]]` (facts already applied), `move ∈ {accept, revise,
redesign}`, `evidence_refs`, `owning_slice?` (optional — its optionality is *why*
a freestanding REC survives slice close), `decision_ref?`. `rec-NNN.md` holds
rationale. No authored `status` field → scanned by `meta::read_id`.

**Coverage entry** (keyed, not id'd — Q4) in `.doctrine/slice/NNN/coverage.toml`:
`requirement`, `contributing_change` (default = owning slice), `mode ∈ {VT, VA,
VH}`, `status: CoverageStatus` (the SL-028 enum, reused), `git_anchor`,
`attested_date?` (VH/VA only). Stored **slice-side** so several changes touching one
requirement compose with **no clobber** (D3); stored in a file **distinct** from the
requirement's authored status (NF-001).

Ownership: a slice owns the coverage it establishes; a REC is owned by its
reconciliation act (optionally a slice). The composite/drift views own *no* state.

### 5.4 Lifecycle, Operations & Dynamics

- **Coverage** is written at audit (the change records what it observed) and read
  on demand by the composite fold. Append/update within a slice's own file; never
  cross-writes another slice's file (no clobber).
- **REC** is written once by the reconcile writer (Slice B) and never transitions —
  the commit is the act boundary; approval is a conduct-axis concern (ADR-009),
  case-by-case, not a REC lifecycle. Staged draft/approve of deltas against spec
  prose is a future **Revision** vehicle (IDE-003), deferred.
- **Staleness** (P4): a VH/VA entry's `git_anchor` is compared to HEAD via the
  `src/git.rs` seam; movement past the anchor flags the entry stale — **surfaced,
  never auto-demoted** to another status.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (NF-001).** No function maps coverage → authored requirement status.
  Preserved structurally: `drift()` returns `Verdict` not `ReqStatus`; the two
  enums keep their SL-028 "never reference each other" property; coverage and
  status live in distinct files.
- **INV-2 (D4).** No composite/drift value is persisted; correctness is
  recomputation.
- **INV-3.** Coverage is authored/committed (Q1) — reconstructable from the
  authored tier alone, no recourse to disposable runtime state (NF-003 spirit).
- **Edge cases.** Requirement with zero coverage entries → composite empty →
  drift `Indeterminate` (not silently coherent). Conflicting entries across
  changes (stale VH `Verified` vs fresh VT `Failed`) → `Indeterminate`/`Divergent`,
  surfaced (OQ-3, not resolved). Evidence unobtainable → entry `Blocked`, surfaced.

## 6. Open Questions & Unknowns

- **OQ-1 (closed → Q5).** REC alias convention = `NNN-slug`, identical to every
  numbered kind (reuse the alias machinery).
- **OQ-2 (carried, SPEC-002).** Shared-evidence-type ownership vs PRD-010
  `knowledge_record`: REC owns its evidence sub-structure inline until
  knowledge_record lands; neither forks (H4).
- **OQ-3 (deferred).** Composite precedence rules — v1 surfaces all;
  `Indeterminate` collapses later. Not a v1 blocker.

## 7. Decisions, Rationale & Alternatives

- **D-Q1 — coverage is authored (committed, diffable), slice-side.** NF-003 wants
  reconstruction "from the authored tier alone, no recourse to disposable runtime
  state" → evidence must be durable. NF-001 separation is *file*-separation, not
  *tier*-separation. *Alt rejected:* runtime/gitignored (ephemeral evidence can't
  back a closure gate or audit trail).
- **D-Q2 — cross-slice fan-in = corpus scan, no reverse index.** Matches SL-040
  D-C9b and ADR-004 outbound-only. *Alt rejected:* a reverse index (new machinery,
  invalidation risk, cuts the grain) — deferred behind the perf spike. *Alt
  rejected:* central coverage store (contradicts D3 no-clobber).
- **D-Q3 — REC is status-less / immutable.** Matches the event ontology ("one REC
  per act"); keeps the gate in one place (ADR-009 conduct axis); reuses SL-040's
  `read_id`; cleanest NF-003 ledger; additively upgradable. The drafting/approval
  of deltas against spec prose belongs to a distinct future **Revision** vehicle
  (IDE-003), case-by-case/team-by-team — *not* a REC lifecycle. *Alt rejected:*
  status-bearing draft→approved (bakes a second lifecycle + transition verb +
  filtering into P1 that Slice B may never use; harder to reverse).
- **D-Q4 — coverage is a keyed table, not a numbered kind.** D3 keys by
  (requirement × contributing change); only REC gets numbered-kind wiring (one
  `KINDS` row, minimal SL-040 collision surface).
- **D-Q5 — coherence predicate is conservative tri-state.** Gives FR-004 a verdict
  without an OQ-3 precedence engine; refuses to silently call ambiguous coherent.

## 8. Risks & Mitigations

- **R-a — SL-040 dependency/collision.** P1 reuses SL-040's in-flight
  `meta::read_id` + status-less scan path and adds a row to the same
  `integrity::KINDS` table. **Mitigation:** sequence P1 *after* SL-040 commits;
  fallback — SL-042 lands the small `read_id` seam itself and SL-040 rebases. P2–P4
  are independent of SL-040 and proceed in parallel.
- **R-b — `CoverageStatus` `expect(dead_code)` removal.** P2 (the consumer) must
  **remove** the `not(test) expect(dead_code)` when `CoverageStatus` becomes
  genuinely used in the non-test build, or the expectation goes unfulfilled
  (the cfg(test) subtlety the SL-028 comment flags).
- **R-c — coherence precedence (OQ-3).** Conservative-by-design; `Indeterminate`
  counts as drift at the gate. Accepted for v1.
- **R-d — scan cost (R2).** Bounded by the perf spike (§9), not assumed; a cliff
  below realistic scale triggers the pre-registered reverse-index backlog item.

## 9. Quality Engineering & Validation

Per-requirement evidence (VT unless noted):

- **REQ-108** — scaffold→toml round-trip (deltas/move/evidence_refs, optional
  owning_slice/decision_ref); `show`/`list` render; `NNN-slug` alias resolves;
  `validate` clean with the new `KINDS` row; id-stable after slice close.
- **REQ-109** — write/read entries; **no-clobber** (two slices, same requirement,
  neither overwrites); stored in a file distinct from requirement status.
- **REQ-110** — fold determinism (same entries → same view); **assert nothing
  persisted** (no stored composite scalar on disk).
- **REQ-111** — the three verdict cases incl. FR-004 "matches → coherent";
  **type-level**: `drift()` returns `Verdict`, no truth-write in its signature.
- **REQ-114 / NF-001** — **structural**: SL-028's "two enums never reference each
  other" preserved; `drift()` returns `Verdict` not `ReqStatus`; a guard test
  asserting no `f(coverage) → ReqStatus`.
- **REQ-115 / NF-002** — wire `git_anchor` onto the `src/git.rs` seam; stale
  `Verified` flagged, **not demoted**; reuse asserted (no parallel staleness impl).

**R2 perf spike (VT in P3):** seed N≈500 synthetic slices × coverage entries on
shared requirements; time the composite corpus-scan; assert under budget, **debug
~10× release** (`mem.pattern.testing.debug-vs-release-scale-timing`). Output = the
scan-cost cliff; a cliff below realistic repo scale triggers a reverse-index
`backlog new` (condition recorded now, per defer-needs-backlog).

Lint/format gates per house rules (`cargo clippy` zero-warning bins/lib, `just
check`). New module trips the cargo/pedantic doc lints
(`mem.pattern.lint.new-workspace-member-cargo-metadata`).

## 10. Review Notes

_(adversarial pass pending — §Adversarial review of the design skill)_
