# SL-190 — Design: Dispatch orchestrator state-visibility verbs

Governed by ADR-006 (worktree posture / sole-writer), ADR-007 (baton in primary
runtime state), ADR-012 (dispatch topology), ADR-001 (layering), POL-002
(platform independence). Originates from IDE-027 (the "now" half). Related:
IMP-191 (shipped read-only `slice status`), IMP-174 (authored-tier split-brain),
SL-180 (selector-conformance gate), IDE-017 (fork provisioning).

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

## Canonical source of truth (the foundational decision)

Two representations of phase progress coexist:

- **Runtime phase sheets** (`state.rs:290 phase_rollup`) — `planned/in_progress/
  completed/blocked` per phase; gitignored, **per-tree**, disposable; the thing
  that split-brains.
- **`boundaries.toml` conformance registry** — primary-tree-only (`state.rs:614`
  always `primary_worktree`), populated by `record-boundary`/`record-delta`,
  completeness-gated at `prepare-review`. Committed, durable, **canonical for
  which phases landed**.

**Decision: composite source, not registry-only and not cross-tree copy.** Per
phase, truth resolves as:

```
registry LANDED ∧ live coord disagrees (in_progress/blocked) → CONFLICT (surface, never auto-resolve)
registry boundary present                                    → LANDED   (authoritative, durable)
else live dispatch/<slice> worktree                          → IN-FLIGHT (that tree's runtime status)
else                                                         → LOCAL / UNKNOWN
```

**CONFLICT (F-1, rework masking).** `record-boundary` upserts, so a *re-dispatched*
phase shows LANDED in the registry while the live coord tree shows `in_progress`
again. Resolving that to LANDED masks active rework. So when the registry says
landed but a **live** coord runtime disagrees, the phase resolves to `CONFLICT` —
surfaced in the divergence table and treated as divergence, never silently
collapsed. (No live coord tree ⇒ no disagreement possible ⇒ registry LANDED stands.)

Rationale:
- **Registry-only** is durable and grain-aligned but blind to `in_progress` —
  useless mid-drive (the primary locus per the user: avoidable mid-drive fuckups
  cascade into later-phase blowup). Rejected.
- **Cross-tree runtime copy** (coord→primary) fits one mental model but is fragile
  (coord tree is removed at conclude → no source at close), fights ADR-006 (raw
  per-tree runtime copy), and ignores the already-canonical registry. Rejected.
- **Composite** covers both loci: mid-drive the live coord tree supplies in-flight
  truth; at/after close the committed registry supplies landed truth and survives
  coord removal. The single cross-tree read (live coord) is explicit, read-only,
  and confined to phases the registry structurally cannot represent. Grain-fit:
  derive disposable state from canonical truth.

**Cross-machine handoff boundary (documented limit):** on a fresh machine the
coord tree is absent and gitignored in-flight runtime does not travel with git.
Reconcile recovers *landed* truth from the committed registry and marks in-flight
phases `unknown`. This is inherent (never-committed state cannot be recovered),
not a defect.

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
| `slice reconcile-phases <ID>` | write | rewrite the **primary** tree's runtime sheets from composite truth (deliberate cross-tree write, F-5) |

**Pure core** (engine, beside `phase_rollup`):
`resolve_phase_truth(registry_rows, coord_rollup, local_rollup) -> {truth, divergence}`
— total, testable; shell gathers the three inputs (`primary_worktree` registry,
live coord via `live_worktree_for_ref("dispatch/<slice>")`, local `phase_rollup`).

**`--assert` scope (F-2).** Fires **only** on `CONFLICT` or an opinionated
disagreement (two sources hold differing non-`unknown` statuses). It does **not**
fire on `unknown`/silent — otherwise a fresh handoff machine (all in-flight →
`unknown`, no coord) is permanently red. Exit code aligns with `worktree status
--assert`'s convention (F-7).

**Tree-root resolution (F-3).** Each `git worktree list --porcelain` entry path
**is** that tree's root — use it directly for the coord/primary state read. Do
**not** strip `.worktrees/<name>`: coord dirs are also `.dispatch/SL-<n>`, so a
fixed-prefix strip is convention-fragile. `live_worktree_for_ref` *locates* the
coord tree; its porcelain path *is* the root.

**Cross-tree write justification (F-5).** `reconcile-phases` resolves the primary
tree (`primary_worktree`) and writes *its* `.doctrine/state/` from wherever the
verb is invoked (coord tree during drive, session root, or a handoff machine).
Writing another tree's runtime state is within remit: the orchestrator is the
**sole writer** (ADR-006 D2); this is an explicit operator-invoked verb, not
implicit background sync.

**Reconcile conflict rule:** *composite wins where it has an opinion; local
survives where composite is silent; `CONFLICT` phases are **skipped and reported**,
never auto-written* (a rework disagreement is not the verb's to resolve). Never
regress a locally-`completed` inline phase because the (dispatched-phases-only)
registry has no row for it. Load-bearing for mixed inline+dispatch slices (the
SL-156 / IMP-174 shape). Write via the edit-preserving `set_phase_status`
(`toml_edit`). Idempotent.

**Degradation for non-dispatch slices (F-6).** A pure solo/inline slice has no
`dispatch/<slice>` coord tree and (typically) no registry boundaries → composite
== local → `--across-trees` shows no divergence and `reconcile-phases` is a no-op.
Single-tree slices are unaffected; no harm.

**Value:** the `--assert` read is the primary mid-drive fuckup-prevention lever
(gate before acting on a stale view); the reconcile keeps `slice list` honest and
de-risks cross-machine / halfway handoffs.

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

**`landed` column (default on).** Role-conditional — a verdict only for
`worker-fork` (target = its `dispatch/<slice>`) and `coordination` (target =
`deliver-to`/trunk) rows; `primary`/`benign` → N/A. Fail-soft: a missing target
ref → `unknown`, never a hard error. The verdict **reuses gc's existing landed
oracle** (`gather_landed`/`classify_gc`, ancestry + patch-id); lifting it to a
shared engine fn is a **behaviour-preserving** refactor of `worktree/gc.rs` — the
gc suite stays green unchanged (shared-machinery gate). No second oracle.
**F-4:** the lift also **parameterizes the target ref** (gc currently implies the
coordination target; coord-rows need `deliver-to`). Behaviour-preservation holds
because gc keeps passing its existing coordination target — only the new
inventory caller supplies a different one.

**Value:** kills the audit "nothing to audit" blindspot; `--slice` answers "where
does SL-X's implementation live, and is it already imported?"

### 3. Selector doctor — `slice selector doctor <ID>`

New `SelectorCommand::Doctor` (beside Add/Note/List/Rm, `slice.rs:68`), read-only.

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

**SL-180 coordination:** one pure predicate, two dispositions. If the doctor lands
first, SL-180's gate reuses this predicate; if SL-180 lands first, the doctor
consumes its predicate. Resolved at plan time via the SL-180 relation. Guard the
predicate with `#[expect(dead_code, reason)]` if it is staged ahead of SL-180's
consumer.

## Layering & code impact (ADR-001, pure/imperative)

Each verb = pure core (engine) + thin shell (command). New modules need a layering
umbrella entry or the gate fails (`mem.pattern.lint.module-split-needs-layering-entry`).

| Path | Change |
|---|---|
| `src/state.rs` | `resolve_phase_truth` (pure) + reconcile write helper |
| `src/slice.rs` | `status --across-trees/--assert`, `reconcile-phases`, `SelectorCommand::Doctor` |
| `src/conformance.rs` | selector-doctor pure predicate (reuse/extend) |
| `src/worktree/inventory.rs` **(new)** | `classify_worktree` + `run_list` |
| `src/worktree/gc.rs` | lift landed-oracle to shared fn (behaviour-preserving) |
| `src/git.rs` | `list_worktrees()` list-all primitive |
| `src/worktree/mod.rs` | `WorktreeCommand::List` |
| `src/commands/cli.rs` | route new subcommands |
| `.doctrine/adr/001/layering.toml` | sub-classification entry for `worktree/inventory.rs` (mirror the `gc.rs` tier; a new module differing from its umbrella tier reds `MixedUmbrella` in `tests/architecture_layering.rs`) |

Clippy: `print_stdout` denied → `writeln!(io::stdout())`; `#[expect(dead_code,
reason)]` for predicates staged ahead of a consumer.

## Verification alignment

Pure cores are the primary evidence (total functions, exhaustive case tables):
- `resolve_phase_truth` — landed / in-flight / silent / divergent / **CONFLICT
  (registry-landed ∧ coord-in_progress)** / mixed inline+dispatch; non-regression
  rule; `--assert` fires on conflict-not-unknown (F-1, F-2).
- `classify_worktree` — each role from (is_primary, branch, marker) combos; landed
  reuse (landed / not / unknown-target).
- `diagnose_selector` — uncompilable / unmatched / redundant / broad, per intent.

Shell / integration:
- `--assert` exit codes (phase-status divergent, selector-doctor findings).
- `reconcile-phases` idempotence + non-regression on a mixed slice.
- `worktree list` over a fixture (primary + live coord + worker-fork): table,
  `--slice`, `--json`, landed column.
- **gc suite green unchanged** after the oracle lift (behaviour-preservation gate).

## Open questions

- **OQ-1** Write-verb name: `slice reconcile-phases` vs `slice phase-sync`
  (`slice phases` is taken by phase authoring). Leaning `reconcile-phases`.
- **OQ-2** Does `--across-trees` become the default for a *bare* `slice status`
  when a live coord tree is detected, or stay opt-in? Leaning opt-in (keep IMP-191's
  cheap local read the default; explicit `--across-trees` for the cross-tree cost).
- **OQ-3** SL-180 sequencing — which of doctor / SL-180-gate lands the shared
  predicate. Resolve at plan time.
