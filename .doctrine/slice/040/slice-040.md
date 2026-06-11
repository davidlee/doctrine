# RV review-ledger kind and review verb family

## Context

Doctrine has no structured adversarial-review primitive: `audit.md` is hand-made
(a known scaffold gap), inquisition is informal, code review is unstructured.
**ADR-007** (accepted) decides the fix — a first-class `review` kind (`RV-NNN`),
one generic `facet`-parameterized ledger reviewing any subject via the outbound
`reviews` edge, coordinated by a turn-based baton in runtime state with
CLI-mediated turn-taking and a per-review lock. This slice **builds that kind and
its verb family**, realising IMP-001.

The build was de-risked in preflight against predecessor prior art
(`scratch/rv-review-prior-art.md`, spec-driver/autobahn), which produced three
ADR amendments now in force: D-C10 (warm-cache restored), D-C11 (`drift` dropped
from the facet enum → future Drift Ledger kind), and a named reverse-relation
dependency for the close-gate.

Governing decision: **ADR-007** (all D-C0…D-C11). This slice does not re-decide
any of it; tensions with the ADR go back through `/consult`, not local drift.

## Scope & Objectives

Ship the RV kind end-to-end, single piloted integration:

1. **Authored kind** — `review-NNN.toml` schema per ADR-007 §Schema shape, facet
   enum **minus `drift`** (D-C11): `scope|design|plan|phase-plan|implementation|
   code-review|reconciliation`. `review-NNN.md` with `## Brief` + optional
   `## Synthesis` (D-C6). Append-only findings with disjoint field ownership
   (D-C5).
2. **Engine + install wiring** — id allocation, `integrity::KINDS` row, manifest
   dir + `.gitignore` negation (`mem.pattern.install.authored-entity-wiring`),
   render/`show`.
3. **`reviews` edge + reverse close-gate** — outbound edge (ADR-004); the D-C9b
   close-gate as a corpus scan over RV `[target].ref` (no reverse index exists).
4. **Verb family** — `raise / dispose / verify / contest / withdraw / status /
   prime`, CLI-mediated turn guard (D-C4), `--as <role>` role assertion.
5. **Runtime coordination** — baton in runtime state (D-C1/D-C2), authored-first /
   baton-last ordering (D-C3), per-review lock + CAS (D-C4a).
6. **Derived status + lifecycle teeth** — total status function over the status
   enum incl. empty-ledger and all-terminal (D-C8); two-gate lifecycle: review-done
   = all findings terminal (D-C9a), target-close refuses on unresolved `blocker`
   (D-C9b).
7. **D-C10 warm-cache** — uniform, self-scaling reviewer-context cache
   (`domain_map`/invariants/risks) beside the baton; `prime` populates it;
   staleness keyed on **content-hashes of the explored path set** (not
   `{phase, head}`). Worktree-interaction is a **design consideration** to resolve
   in `/design` (per-worktree provisioning copies some gitignored tiers, excludes
   others — ADR-006 — so the hashable path set differs main-tree vs fork).
8. **Pilot one skill** — rewire **`/audit`** (the known scaffold gap) onto the RV
   kind as proof-of-integration. One skill only.

Closure intent: the kind exists, all verbs work under the turn guard, the
concurrent-write test shows no clobber (one writer wins, other aborts/retries),
status/close-gate behave per D-C8/D-C9, warm-cache primes and invalidates on
content drift, and `/audit` produces an RV instead of a hand-made `audit.md`.
ADR-007 §Verification enumerates the test obligations.

## Non-Goals

- **Drift Ledger kind** (multi-artefact normative-vs-observed divergence) — **IMP-022**.
- **Rewiring the remaining review skills** (`/inquisition`, `/code-review`,
  reconciliation) onto RV beyond the one pilot — **IMP-023**.
- **Large-review funnel** (parallel raisers into one ledger; ADR-007 Neutral
  seam) — **IMP-024**.
- `--as` as a security boundary — cooperative only, by ADR-007 (Negative).
- A general reverse-relation index — the close-gate is a scoped corpus scan, not a
  new index subsystem.

## Summary

One coherent capability — the RV review kind — built end to end (schema → engine
wiring → edge + close-gate → verb family → runtime baton/lock → derived status +
lifecycle teeth → warm-cache) with `/audit` rewired as the single proof-of-
integration. Large but not sprawling; phasing absorbs the size, with warm-cache
sequenced late so an RV-without-cache increment is demonstrable mid-slice.

### Affected Surface (provisional — firm up in /design)

- `src/integrity.rs` — `KINDS` row (new numbered kind).
- `install/manifest.toml`, `.gitignore` — authored-tree wiring.
- new `src/review.rs` (likely) — kind, verb family, status fn, turn guard.
- runtime state tree `.doctrine/state/` — baton, lock, warm-cache (gitignored).
- close-gate hook into the `/close` path (`src/slice.rs` / close skill).
- render/`show` seam (cf. `src/adr.rs`, `src/slice.rs`).
- `src/main.rs` — CLI subcommand wiring.
- `.doctrine/skills` / `plugins/` — the `/audit` skill (source-of-truth in
  `plugins/`, not the installed copy).

### Risks / Assumptions / Open Questions

- **R1 — warm-cache content-hash × worktree** (D-C10): the explored path set and
  its hashability differ across worktrees; staleness model must be worktree-aware.
  Largest design unknown. → `/design`.
- **R2 — reverse close-gate scan cost**: corpus scan over all RV per `/close`;
  acceptable at current scale, note if it needs indexing later.
- **A1 — pilot skill = `/audit`** unless `/design` surfaces a better first
  integration.
- **A2 — single slice altitude**: large but one coherent capability; size is
  absorbed by phasing, not by splitting the kind.
- **OQ-1** — does `prime` warrant its own runtime sub-file or fold into the baton
  file? (design)
- **OQ-2** — lock mechanism: advisory file lock vs CAS-on-content; interaction
  with D-C4a re-read. (design)

## Follow-Ups

Tracked as backlog, per `mem.system.lifecycle.defer-needs-backlog-before-close`:
IMP-022 (Drift Ledger), IMP-023 (skill rewiring), IMP-024 (large-review funnel).
