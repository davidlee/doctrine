# SL-190 — Design: Dispatch orchestrator state-visibility verbs

Governed by ADR-006 (worktree posture / sole-writer), ADR-007 (baton in primary
runtime state), ADR-012 (dispatch topology / preserved `phase/*` refs), ADR-001
(layering), POL-002 (platform independence). Originates from IDE-027 (the "now"
half). Related: IMP-191 (shipped read-only `slice status`), IMP-174 (authored-tier
split-brain), SL-180 (selector-conformance gate), IDE-017 (fork provisioning).
Follow-up: IDE-028 (auto primary-sheet-push in the Record beat — deferred; this
slice ships the manual reconcile it would automate).

> **Revision after RV-214 (codex external inquisition).** Seven charges, all
> confirmed against source. F-1/F-2/F-3 were blockers reshaping the foundational
> section and the pure-core signature; F-4/F-5/F-6 majors; F-7 minor. The changes
> are integrated below and flagged inline `(RV-214 F-n)`.

## Problem

The `/dispatch` + `/worktree` orchestrator surface is the largest token sink in
the platform (RFC-011 case notes). The dominant sinks are **coordination-state
visibility**, not command shape:

1. **Phase-status split-brain.** Runtime phase-state lives per-tree in gitignored
   `.doctrine/state/` (`phase_rollup`, `state.rs:290`). The coordination worktree
   advances its sheets; the primary/session tree does not. A reader of the stale
   tree (a fresh `/handover` agent, an `/audit` step, a cross-machine handoff)
   sees `2/5` where the truth is `4/4`, re-plans done phases, and blows tokens in
   later phases. All current reads (`slice status`, `dispatch status/plan-next`)
   are **local-tree only** — no cross-tree reconciliation exists.
2. **Worktree-discovery blindspot.** `/audit` reads the main tree first; the
   implementation lives on a dispatch worktree; the operator nearly concludes
   "nothing to audit." No verb enumerates all worktrees + provenance.
3. **Selector rot.** Stale/dead selector globs are only caught post-hoc by the
   `conformance` gate at audit; no on-demand advisory while authoring.

Excluded on POL-002: binary-freshness, provisioning/artifact checks (host-build
coupling). Excluded per IDE-027 gate: the MCP projection. Not touched: dispatch
topology, fork/import modes, spawn choreography, funnel cadence (churning under
RFC-005 / SL-180/181/185/187/189).

## Canonical source of truth (the foundational decision) — reworked (RV-214 F-1)

Phase progress has **three** representations, each with a different durability
tier. Naming them correctly is load-bearing (the prior draft wrongly called the
gitignored registry "committed"):

| Representation | Tier | Travels? | Role here |
|---|---|---|---|
| Runtime phase sheets (`phase-NN.toml`, `state.rs:290`) | gitignored, **per-tree** | no | the thing that split-brains; in-flight status |
| Source-delta **registry** (`.doctrine/state/slice/<N>/boundaries.toml`, `state.rs:614`) | gitignored, **primary-tree-local, derived** | no | a per-phase landed *cache*, written each funnel batch (`record_source_delta`, `dispatch.rs:746`); conformance input |
| **`phase/<slice>-NN` ref cut** (`PHASE_REF_PREFIX`, `dispatch.rs`) | **committed git ref**, immutable (I9) | **yes (via fetch)** | the durable, cross-machine landed signal; cut at `prepare-review` |

The registry is **not** committed — `.doctrine/state/` is gitignored
(`.gitignore:56`), `write_registry` is annotated "disposable gitignored state"
(`state.rs:642`), and `git ls-files` tracks zero `boundaries.toml`. The durable,
git-native, travels-cross-machine artifact is the **`refs/heads/phase/<slice>-NN`**
per-phase cut (readable via `for-each-ref`, the existing `count_phase_refs`
primitive, `dispatch.rs:3105`). The registry is the *derived local cache* that
`prepare-review` maintains and conformance reads.

**Decision: composite source — durable phase refs ⊕ derived registry cache ⊕
live coord runtime.** Per phase, "landed" resolves refs-first, cache-fallback:

```
landed(phase) := phase/<slice>-NN ref present           (durable, cross-machine)
              OR registry cache row present               (primary-local; covers the
                                                            mid-drive window before the
                                                            refs are cut at prepare-review)
```

Then the composite truth table (see the **total** table below, RV-214 F-4).

**Why refs-first, cache-fallback.** The `phase/<slice>-NN` refs are the authoritative
durable signal but only exist **after `prepare-review`** (Stage-1 cuts them). During
the drive, before prepare-review, the per-phase landed signal is the registry cache
row (`record_source_delta` writes one per funnel batch, `dispatch.rs:746`) — present
on the machine running the drive. So: mid-drive → cache; post-prepare-review and
cross-machine → refs.

Rationale for the composite over the alternatives:
- **Registry-cache-only** (the prior draft) is blind to `in_progress` AND does not
  survive cross-machine (gitignored). Rejected.
- **Cross-tree runtime copy** (coord→primary sheet copy) fits one mental model but
  is fragile (coord tree removed at conclude → no source at close), fights ADR-006
  (raw per-tree runtime copy), and ignores the durable refs. Rejected.
- **Composite** covers every locus: mid-drive the live coord tree supplies in-flight
  truth and the registry cache supplies landed; at/after close the committed
  `phase/*` refs supply landed truth that survives coord removal AND a git fetch to
  another machine. Grain-fit: derive disposable state from durable truth.

**CONFLICT — rework masking (RV-214 F-1 context).** `record_source_delta` upserts
and the refs are immutable, so a *re-dispatched* phase reads LANDED (ref/cache
present from a prior run) while the live coord sheet shows `in_progress` again.
Collapsing that to LANDED masks active rework. So **landed ∧ a live coord sheet
disagreeing (in_progress/blocked) → `CONFLICT`** — surfaced in the divergence
table, never silently resolved. (No live coord tree ⇒ no disagreement possible ⇒
landed stands.)

**Cross-machine handoff boundary (documented limit, now narrower).** On a fresh
machine the gitignored registry cache and gitignored in-flight sheets do **not**
travel; the `phase/<slice>-NN` refs **do**, once `git fetch`ed. So reconcile
recovers *landed* truth from the fetched refs (stronger than the prior draft, which
had no cross-machine landed source at all) and marks never-committed in-flight
phases `unknown`. Precondition stated honestly: cross-machine landed recovery needs
the phase refs fetched; in-flight state is inherently unrecoverable (never
committed).

## Verbs

### 1. Phase-status: cross-tree query + reconcile

Home is **`slice`** (not `dispatch`): the staleness victim is any tree and the
truth is slice-scoped; it must work where there is no dispatch context (fresh
machine). Read and write are **separate verbs** — IMP-191 just removed a
reader/writer overload; not reintroducing one behind a `status --reconcile` flag.

| Verb | Mode | Behaviour |
|---|---|---|
| `slice status <ID>` (bare) | read | unchanged — local per-tree rollup (IMP-191) |
| `slice status <ID> --across-trees [--assert]` | read | composite truth + per-tree divergence table; `--assert` → non-zero exit **on CONFLICT / opinionated-disagreement only** (F-2) |
| `slice reconcile-phases <ID>` | write | rewrite the **primary** tree's runtime sheets from composite truth, via a status-only writer (RV-214 F-3); **refuses when a live `dispatch/<slice>` coord tree exists** (RV-214 F-5) |

**Pure core (RV-214 F-2 — per-phase, not rollups).** `phase_rollup` returns
`PhaseRollup` = aggregate bucket *counts* (`state.rs:74`), which cannot say *which*
phase is in which status. The core therefore takes **per-phase status maps**, not
rollups:

```
resolve_phase_truth(
    landed:  &[(PhaseId, LandedSignal)],   // ref-present | cache-present | absent, per phase
    coord:   &[(PhaseId, StemStatus)],     // live dispatch/<slice> per-phase sheet status
    local:   &[(PhaseId, StemStatus)],     // this tree's per-phase sheet status
) -> Vec<(PhaseId, PhaseTruth)>            // + a Divergence summary
```

`StemStatus` (`state.rs:64`) already models `Toml(&str)` / `MissingToml`; the shell
sources the per-phase maps from a **per-phase reader** (the `read_phase_status`
idiom), NOT `fold_rollup`. Rollups may still feed a one-line summary but never the
resolver. The core stays **pure** — no clock/git/disk (ADR-001); the shell gathers
all three inputs: `for-each-ref` for the `phase/*` refs, `read_source_deltas` for
the registry cache, `live_worktree_for_ref("dispatch/<slice>")` + per-phase read for
coord, per-phase read for local.

**Total truth table (RV-214 F-4 — every state, explicit catch-all).**

| landed | live coord sheet | local sheet | → PhaseTruth |
|---|---|---|---|
| ref/cache present | *(none)* | — | `LANDED` |
| ref/cache present | `completed` | — | `LANDED` |
| ref/cache present | `in_progress` / `blocked` | — | `CONFLICT` (rework) |
| ref/cache present | `planned` | — | `CONFLICT` (rework-reset) |
| ref/cache present | `unknown` / `missing_toml` | — | `LANDED` + anomaly-noted (anomaly suppresses divergence, mirroring `PhaseRollup::anomalies`, `state.rs:96`) |
| absent | `in_progress` / `blocked` / `completed` | — | `IN-FLIGHT` (that sheet's status) |
| absent | `planned` | — | `PLANNED` |
| absent | *(no coord tree)* | any | `LOCAL` (local sheet's status) |
| absent | `unknown` / `missing_toml` | — | `ANOMALY` (surface, suppress divergence) |
| — | phase absent from one side (plan drift) | — | `UNKNOWN` + phase-set-mismatch noted |

The resolver is **total** by an explicit final catch-all → `UNKNOWN`; no silent
hole. Phase-set drift (a phase present in one source's map but not another's — the
orphaned-runtime-phase case the code already surfaces, `state.rs:206`) resolves to
`UNKNOWN` with the mismatch reported, never a panic or a dropped phase.

**`--assert` scope (F-2).** Fires **only** on `CONFLICT` or an opinionated
disagreement (two sources hold differing non-anomaly, non-`unknown` statuses). It
does **not** fire on `UNKNOWN`/`PLANNED`/anomaly/silent — otherwise a fresh handoff
machine (all in-flight → `unknown`, no coord) is permanently red. Exit code aligns
with `worktree status --assert`'s convention (F-7-prior/exit-code).

**Reconcile writer (RV-214 F-3 — status-only, no side effects).** `set_phase_status`
(`state.rs:365`) is **unsuitable** as the reconcile writer: it appends a `[[progress]]`
audit row every call (`state.rs:426`), restamps `last_updated`/`started`/`completed`,
and — fatally — via `capture_phase_boundary`/`record_source_delta`/`forget_source_delta`
(`state.rs:424,452,461`) **mutates the very registry** reconcile reads as landed
truth (a read-then-clobber-your-own-source cycle; also non-idempotent). Reconcile
therefore uses a **dedicated status-only writer**: edit-preserving (`toml_edit`),
sets only the `status` field (+ `last_updated`), **no** `[[progress]]` append, **no**
boundary capture/eviction. Verification proves zero registry mutation during a
reconcile (F-3 test below). Idempotent by construction.

**Reconcile conflict rule:** *composite wins where it has an opinion; local
survives where composite is silent; `CONFLICT` / `UNKNOWN` / anomaly phases are
**skipped and reported**, never auto-written* (a rework disagreement is not the
verb's to resolve). Never regress a locally-`completed` inline phase because the
(dispatched-phases-only) registry/refs have no row for it. Load-bearing for mixed
inline+dispatch slices (the SL-156 / IMP-174 shape).

**Cross-tree write safety (RV-214 F-5 — refuse-when-live, not "sole-writer-as-lock").**
The prior draft cited ADR-006 sole-writer to bless writing another tree's runtime
state "from wherever invoked". But sole-writer is an orchestrator *role* invariant,
not a mutual-exclusion mechanism, and the write seams are lock-free `fs::write`
(`state.rs:439,644`) — a handoff-machine reconcile could race a live drive. So
`reconcile-phases`:
- writes **only the primary tree** (`primary_worktree`, `git.rs:564`), never a coord
  tree;
- **refuses (non-zero, named)** when a live `dispatch/<slice>` coordination worktree
  exists (`live_worktree_for_ref`) — during an active drive the coord tree is the
  writer; reconcile defers to it. Reconcile is a between-drives / post-conclude /
  cross-machine recovery tool.
- **single-operator precondition** stated: doctrine has no cross-machine lock; two
  operators reconciling one repo concurrently is out of contract (documented, not
  guarded — consistent with the rest of the runtime tier).

This confines the cross-tree hazard to a stated, refusable condition rather than
leaning on a role invariant as if it were a lock.

**Degradation for non-dispatch slices (F-6-prior).** A pure solo/inline slice has no
`dispatch/<slice>` coord tree, no phase refs, no registry rows → landed all-absent,
composite == local → `--across-trees` shows no divergence and `reconcile-phases` is a
no-op. Single-tree slices unaffected; no harm.

**Value:** the `--assert` read is the primary mid-drive fuckup-prevention lever (gate
before acting on a stale view); the reconcile keeps `slice list` honest and de-risks
cross-machine / halfway handoffs (now with a real durable landed source — the refs).

### 2. Worktree inventory / provenance — `worktree list`

New `WorktreeCommand::List`, read-classed (callable anywhere, incl. worker-mode).
Global cross-slice view — distinct from `worktree status` (self/cwd worker-mode)
and `dispatch status` (one slice's one coord tree).

Enumerate `git::list_worktrees()` (new list-all primitive extracted from the
porcelain parse `worktree_for_ref` already does, `git.rs:1337`), classify each via
a **pure** `classify_worktree(is_primary, branch, marker_cause) -> WorktreeRole`
(engine, beside `classify_gc`):

| Role | Derivation |
|---|---|
| `primary` | `primary_worktree()` first porcelain entry |
| `coordination` | branch matches `dispatch/<slice>` |
| `worker-fork` | marker present (`resolve_mode` → `Cause::Marker`/`Both`) |
| `benign` | detached / other, unmarked |

Row: `path · role · slice · branch · head · marker · live? · landed`. Default
lists all; `--slice <ID>` filter; `--json`; `--no-landed` opt-out.

**`landed` column (default on), and the gc-oracle lift (RV-214 F-6 — two claims,
split).** Role-conditional — a verdict only for `worker-fork` (target = its
`dispatch/<slice>`) and `coordination` (target = `deliver-to`/trunk) rows;
`primary`/`benign` → N/A. Fail-soft: a missing target ref → `unknown`, never a hard
error. The verdict reuses gc's landed oracle (`gather_landed`/`classify_gc`,
ancestry + patch-id, `gc.rs:114,188`). The prior draft called the whole lift
"behaviour-preserving" — that conflated two distinct claims, now separated:

1. **Extraction is behaviour-preserving.** gc currently hardcodes coordination
   `HEAD` into its ancestry + `git cherry` legs (`gc.rs:188`); lifting the oracle to
   a shared engine fn while gc keeps passing that same `HEAD` target leaves the gc
   suite green **unchanged** (shared-machinery gate). No second oracle.
2. **The generalized (arbitrary-target) path is a NEW contract needing its own
   tests.** Parameterizing the target ref (coord rows need `deliver-to`/trunk, not
   `HEAD`) is a semantic *expansion*; the gc suite does not exercise it. So the
   verification section adds explicit non-`HEAD`-target cases (landed / not-landed /
   missing-target→unknown). The new behaviour ships proven, not implied by gc's
   green suite.

**Value:** kills the audit "nothing to audit" blindspot; `--slice` answers "where
does SL-X's implementation live, and is it already imported?"

### 3. Selector doctor — `slice selector doctor <ID>`

New `SelectorCommand::Doctor` (beside Add/Note/List/Rm, `slice.rs:70`), read-only.

**Boundary vs SL-180:** conformance (`conformance.rs:96`) and SL-180 both act as
**gates** (block on undelivered / scope-creep). The doctor is the missing
**standalone advisory** the author runs on demand while editing selectors — same
predicates, different disposition (report exit-0, or `--assert` for a gate).

**Pure core** (engine, reusing the `conformance.rs` glob machinery — no new
matcher): `diagnose_selector(sel, matched, others) -> [Finding]`:

| Finding | Meaning |
|---|---|
| `uncompilable` | glob fails to parse |
| `unmatched` | matches nothing in the tree (stale/dead fence) |
| `redundant` | subsumed by another selector |
| `broad` | matches an outsized set (heuristic; suppressed for `scope-relevant` intent) |

Scoped by intent so a loose `scope-relevant` fence isn't nagged as `broad`.
`--assert` → non-zero on any finding (symmetry with §1), so a skill step can gate.

**Shared predicate ownership (RV-214 F-7 — named in design, not deferred).** The
`diagnose_selector` predicate's **home is `conformance.rs`** — it already owns the
glob-match machinery (`conformance.rs:96`); the doctor and SL-180's future gate are
both **downstream consumers** of that one predicate. Only the *land-order* (which
slice merges the predicate first) is a plan-time question (OQ-2); the ownership seam
is fixed here, closing the parallel-implementation window. Guard the predicate with
`#[expect(dead_code, reason)]` if it is staged ahead of SL-180's consumer.

## Layering & code impact (ADR-001, pure/imperative)

Each verb = pure core (engine) + thin shell (command). New modules need a layering
umbrella entry or the gate fails (`mem.pattern.lint.module-split-needs-layering-entry`).

| Path | Change |
|---|---|
| `src/state.rs` | `resolve_phase_truth` (pure, per-phase maps) + **status-only reconcile writer** (F-3) + a per-phase status reader if not already exposed |
| `src/slice.rs` | `status --across-trees/--assert`, `reconcile-phases` (with live-coord refusal, F-5), `SelectorCommand::Doctor` |
| `src/conformance.rs` | **owns** the selector-doctor pure predicate (F-7); reuse/extend the glob machinery |
| `src/worktree/inventory.rs` **(new)** | `classify_worktree` (pure) + `run_list` (shell) |
| `src/worktree/gc.rs` | lift landed-oracle to shared fn — **extraction** behaviour-preserving (F-6.1); the generalized target path is new-contract-tested (F-6.2) |
| `src/git.rs` | `list_worktrees()` list-all primitive; the shell also reads `phase/*` refs via the existing `for-each-ref` idiom (`count_phase_refs`) for the landed signal |
| `src/worktree/mod.rs` | `WorktreeCommand::List` |
| `src/commands/guard.rs` | worker-mode read/write classification for the three new verbs (reconcile-phases Write; selector doctor, worktree list Read) |
| `src/reconcile.rs` | call-site ripple: `run_status` gained the `--across-trees`/`--assert` bool params (passes `false,false`) |
| `.doctrine/adr/001/layering.toml` | sub-classification entry for `worktree/inventory.rs` (mirror the `gc.rs` tier; a new module differing from its umbrella tier reds `MixedUmbrella` in `tests/architecture_layering.rs`) |

Clippy: `print_stdout` denied → `writeln!(io::stdout())`; `#[expect(dead_code,
reason)]` for predicates staged ahead of a consumer.

## Verification alignment

Pure cores are the primary evidence (total functions, exhaustive case tables):
- `resolve_phase_truth` — the **full total table above** (RV-214 F-4): landed
  (ref-present / cache-present / absent) × coord (each `StemStatus` incl.
  `planned`/`unknown`/`missing_toml`/absent) × local; `CONFLICT` (rework +
  rework-reset); anomaly-suppresses-divergence; **phase-set drift → UNKNOWN**;
  catch-all totality; `--assert` fires on conflict-not-unknown.
- `classify_worktree` — each role from (is_primary, branch, marker) combos.
- **gc oracle (F-6):** (1) gc suite green **unchanged** after the extraction
  (behaviour-preservation gate); (2) **new** generalized-target cases —
  landed / not-landed / missing-target→unknown against a **non-`HEAD`** target.
- `diagnose_selector` — uncompilable / unmatched / redundant / broad, per intent.

Shell / integration:
- `--assert` exit codes (phase-status divergent, selector-doctor findings).
- **reconcile writes only status (F-3):** a reconcile over a fixture leaves the
  registry `boundaries.toml` byte-identical and appends **no** `[[progress]]` row —
  the explicit no-side-effect proof.
- `reconcile-phases` idempotence + non-regression on a mixed inline+dispatch slice.
- **reconcile refuses when a live coord tree exists (F-5):** non-zero, named refusal.
- **cross-machine landed recovery (F-1):** a fixture with `phase/*` refs but no
  registry cache resolves those phases `LANDED` (refs-first path).
- `worktree list` over a fixture (primary + live coord + worker-fork): table,
  `--slice`, `--json`, landed column.

## Open questions

- **OQ-1** Write-verb name: `slice reconcile-phases` vs `slice phase-sync`
  (`slice phases` is taken by phase authoring). Leaning `reconcile-phases`.
- **OQ-2** Land-order of the shared selector predicate: does doctor or SL-180's gate
  merge `conformance.rs`'s predicate first (ownership is fixed — `conformance.rs`,
  F-7; only sequencing is open). Resolve at plan time via the SL-180 relation.
- **OQ-3** Does `--across-trees` become the default for a *bare* `slice status`
  when a live coord tree is detected, or stay opt-in? Leaning opt-in (keep IMP-191's
  cheap local read the default; explicit `--across-trees` for the cross-tree cost).
