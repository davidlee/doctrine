# Design SL-189: Pi-arm boundary recording scopes to imported code commit, not funnel span

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Origin **IMP-231** (SL-186 audit, RV-213 F-3). On the pi/subprocess dispatch arm,
`slice conformance` reports `undeclared` paths that were never delivered code —
orchestrator-trailed knowledge (memories, backlog, authored `slice-NNN.toml` /
`adr/**`) and foreign source pulled in by a `refresh-base` trunk merge. SL-186
showed 17 undeclared against a clean 19-file projected candidate, forcing the
auditor to hand-diff the candidate against trunk to separate real scope creep from
noise.

Make the pi arm record a per-phase source-delta scoped to the **single non-merge
imported code commit** so conformance is clean by construction — no read-side
laundering, no audit-time hand-correction.

## 2. Current State

Both dispatch arms record the same per-phase span into one arm-neutral registry
and `slice conformance` reads it two-dot:

- **Recorders** (funnel step 8, `plugins/doctrine/skills/dispatch/SKILL.md`):
  - claude — `dispatch record-boundary --code-start B --code-end B+1`
    (`run_record_boundary`, `src/dispatch.rs:712`) dual-writes the committed
    ledger `boundaries.toml` **and** the arm-neutral registry.
  - pi/codex — `slice record-delta <SL> PHASE-NN --start B --end B+1`
    (`run_record_delta`, `src/slice.rs:2291`) writes the arm-neutral registry only
    (symmetric ledger derive deferred, D6 / IMP-171).
  - both build a `BoundaryRow{code_start_oid, code_end_oid}` and call
    `record_source_delta` (`src/state.rs:668`) — F-6 guard + upsert, resolved
    against the PRIMARY tree so a coord worktree still writes what the integrator
    reads.
- **Reader** — `conformance_outcome` (`src/slice.rs:2215`) for each row runs
  `git diff --name-status <code_start>..<code_end>` (line 2245), folds the events,
  runs the pure three-cell algebra. **No `.doctrine/` filter, no pathspec.**

The two-dot diff `B..B+1` is the tree delta between the two endpoints — it includes
*everything* that differs: the code commit **plus** any trailed-knowledge commit's
paths **plus** foreign `src/` files a `refresh-base` merge incorporated. The read
is clean **iff `code_end` == the phase's one import commit `S`**, because the funnel
Delta-check guarantees each phase lands as exactly one non-merge commit with
`S^ == B` (`plugins/doctrine/skills/dispatch/SKILL.md` step 2). The pollution is
precisely: the stored range brackets more than `S`.

This is **arm-independent** — same registry, same read. The claude arm escapes
*review-side* only because it has a second, cleaner consumer: the `phase/<N>`
projection (`plan_phases`, `src/dispatch.rs:2481`) does a chained single-commit
code-only tree cut (`tree_of(code_end)` minus first-parent, `.doctrine/` stripped).
The pi arm lacks that projection, so its only consumer is the polluted registry.

The **solo** path is the same disease: `capture_phase_boundary` (`src/state.rs:495`,
triggered by `set_phase_status`) stamps `code_start` = HEAD at `in_progress` and
`code_end` = HEAD at `completed` — a `[start-HEAD, completed-HEAD]` span that sweeps
foreign commits landed in between (IMP-175). It is guarded **off** under a live
`dispatch/<slice>` coord worktree (`src/state.rs:514`) — solo and funnel are
mutually exclusive, never both for one phase.

There is **no separate "phase closed" seam.** The recorded boundary row is the only
per-phase span; the sole closing mechanism today is the auditor hand-running
`record-delta --start --end` when conformance looks noisy
(`plugins/doctrine/skills/audit/SKILL.md`). The "last phase has no closing brace,
sweeps commits up to audit" symptom is the identical span disease with an `end` that
resolves to a coord tip which, by audit-time, includes verify-vt / prepare-review /
knowledge commits.

## 3. Forces & Constraints

- **Read-side filtering is insufficient.** Stripping `.doctrine/` at the
  conformance read would remove authored/knowledge noise but **not** foreign
  `src/*.rs` pulled in by a `refresh-base` merge — those survive any `.doctrine/`
  filter. Only tightening the recorded range to `S` excludes them. The fix must be
  at the recorded range, not the read.
- **Orchestrator is sole git writer** (confinement: `pi-spawn-confined.sh`
  `--ro-bind / /`, rw only on the worker worktree `$D`; a linked worktree's `.git`
  is under the ro root → the worker cannot commit). The orchestrator creates `S`
  at funnel step 7 and holds it exactly — the single-commit oid is always available
  at record time.
- **Behaviour-preservation gate** (AGENTS.md): `record-delta` is shared machinery;
  the existing `--start/--end` escape-hatch contract (F-6 guard, upsert, D12
  Manual-provenance non-clobber) and the conformance suite must stay green.
- **Pure/imperative split** (slices-spec § Architecture): git IO stays in the
  imperative shell; the pure algebra (`conformance::compute`, `BoundaryRow`
  serialization) is untouched.
- **ADR-001 layering**: the shared derivation helper lives in the engine layer
  (`state.rs`) so command modules (`slice.rs`, later `dispatch.rs`) import *down*,
  no cycle.
- **No parallel implementation** (CLAUDE.md): one derivation helper, reused by
  every producer (pi now; claude-funnel + solo later) — do not fork the logic.
- **Governed by ADR-012** (dispatch integration topology): the impl-bundle delta
  routing (code vs orchestrator-written knowledge) is exactly what the scoped
  range must honour.

## 4. Guiding Principles

- **Safe by construction, not by discipline.** The caller records a *commit*, not a
  range it must scope correctly. Tight scope is a property of the primitive, not of
  the orchestrator remembering to pass the right oids.
- **Correct at source, not corrected at audit.** The row is right when written;
  conformance and the auditor consume a clean signal.
- **One seam, all producers.** The single-commit move is the "phase closed" bracket
  that fixes pi (now), the open-tail/last-phase case, the claude funnel, and the
  solo path (IMP-175) — via one shared helper.

## 5. Proposed Design

### 5.1 System Model

Introduce a **single-commit boundary primitive**: given the phase's one import
commit `S`, derive and record `[S^, S]`. `git diff S^..S` is exactly `S`'s own
patch — trailed knowledge, refresh-base merges, and foreign source are excluded
because they are simply not in commit `S`. Wire the pi arm's funnel Record beat to
it. The claude funnel, `record-boundary`, and solo `capture_phase_boundary` are
follow-up adopters of the same helper (out of scope here).

### 5.2 Interfaces & Contracts

**Engine helper** (`src/state.rs`, beside `record_source_delta`):

```rust
/// Derive a single-commit boundary row for `commit` (its own patch): resolve to a
/// full oid, require exactly one parent (reject merges AND root commits — neither
/// has a single "own patch"), and record [parent, commit]. The tight, safe-by-
/// construction counterpart to a hand-passed start..end range.
pub(crate) fn single_commit_boundary(
    root: &Path,
    commit: &str,
    provenance: Provenance,
    phase: &str,
) -> Result<BoundaryRow, ...> {
    let end = git::resolve_ref(root, commit)?.ok_or(/* does not resolve */)?;
    let parents = git::parents(root, &end)?;   // git.rs:1017, existing
    let [start] = parents.as_slice() else { /* merge or root → error */ };
    Ok(BoundaryRow { phase: phase.into(), code_start_oid: start.clone(),
                     code_end_oid: end, provenance })
}
```

**CLI** — extend `slice record-delta` with a `--commit` mode (`src/slice.rs`,
`RecordDelta` variant + `run_record_delta`):

```
doctrine slice record-delta <SL> PHASE-NN --commit <S>      # new: safe default
doctrine slice record-delta <SL> PHASE-NN --start <a> --end <b>   # retained: raw escape hatch
```

- `start`/`end` become `Option<String>`; add `commit: Option<String>`.
- Mutual exclusion: `--commit` conflicts with `--start`/`--end`; exactly one mode
  required (clap `conflicts_with` + a validated "exactly one" check → a clear error,
  never a silent default).
- `--commit` → `single_commit_boundary(.., Provenance::Manual, phase)` →
  `record_source_delta`. Legacy range path unchanged (`Provenance::Manual`, F-6
  guard).

### 5.3 Data, State & Ownership

No schema change — `BoundaryRow{code_start_oid, code_end_oid, provenance}` and the
arm-neutral registry are unchanged; `--commit` only changes *which oids* land.
`record_source_delta` (upsert, F-6 guard, PRIMARY-tree resolution, D12
Manual-provenance sticky-merge) is reused verbatim.

### 5.4 Lifecycle, Operations & Dynamics

Pi funnel step 8 (`plugins/doctrine/skills/dispatch-subprocess/SKILL.md` +
`plugins/doctrine/skills/dispatch/SKILL.md` router line): replace
`record-delta --start <B> --end <B+1>` with `record-delta --commit <S>`, where `S`
is the step-7 single funnel commit (`HEAD` on `dispatch/<slice>` at record time).
`.doctrine/skills/` is the gitignored installed copy (regenerated from `plugins/`)
— not edited.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV** — `git diff S^..S` == `S`'s own patch, for any non-merge non-root `S`.
- **A1** — funnel Delta-check guarantees exactly one non-merge commit `S` per phase
  (`S^ == B`); `--commit HEAD` at step 8 is well-defined.
- **Edge — merge `S`**: rejected (no single parent / own-patch). A funnel `S` is
  never a merge (Delta-check), so this only guards operator misuse.
- **Edge — root commit `S`**: rejected (no parent). Not a funnel case.
- **Edge — legacy range** still available for the rare bootstrap/odd-history case.

## 6. Open Questions & Unknowns

- **OQ-1** — Expose `--commit` on `dispatch record-boundary` in this slice, or wait
  for the claude-funnel adoption follow-up? Lean: **wait** — adding a dead flag the
  claude skill doesn't call is spec-creep; the shared helper already makes adoption
  a one-liner. (Resolved unless review objects.)

## 7. Decisions, Rationale & Alternatives

- **D1 — record `[S^, S]`, tighten the range (chosen).** vs read-side `.doctrine/`
  strip (rejected §3: refresh-base foreign source survives it) vs prose-only tighten
  (rejected: fragile, leaves the two-dot footgun for every caller). A2 in the design
  loop.
- **D2 — `--commit` flag + shared engine helper (chosen).** Safe-by-construction;
  one implementation reused by all producers (no parallel impl).
- **D3 — scope to pi arm now; claude-funnel + solo (IMP-175) are follow-up
  adopters.** Keeps SL-189 shippable and scoped; the helper is built for reuse so
  adoption is rework-free.
- **D4 — retain `--start/--end`.** The escape hatch keeps a raw form for
  bootstrap/odd-history; `--commit` is the safe default.

## 8. Risks & Mitigations

- **R1** — regressing the escape-hatch contract. *Mitigation*: legacy path
  untouched; behaviour-preservation VTs stay green.
- **R2** — orchestrator passes a wrong `S` (not the funnel commit). *Mitigation*:
  non-merge guard rejects the obvious misuse (a merge/cumulative tip is often a
  merge); the skill pins `S = HEAD` at step 7. Residual: a non-merge wrong `S` is
  still mis-recordable — but strictly better than today's mandatory span.
- **R3** — divergence from the claude arm persists until follow-up. *Mitigation*:
  documented; the helper makes convergence cheap; conformance on the claude arm is
  already masked by the `phase/<N>` projection for review.

## 9. Quality Engineering & Validation

- **VT (helper)** — non-merge `S` → `[S^, S]`; merge `S` → error; root commit →
  error.
- **VT (behavioral, SL-186 regression)** — repo with base `B`, code commit `S`
  (`S^==B`), a trailing `.doctrine/` knowledge commit, and a merge bringing a
  foreign `src/` file; record via `--commit S`; assert conformance `actual` == `S`'s
  paths only (trailing knowledge + foreign source excluded).
- **VT (arg)** — `--commit` + `--start` → error; neither → error.
- **Behaviour-preservation** — legacy `--start/--end` tests, F-6 guard (trivially
  holds for `S^..S`), D12 Manual non-clobber, and the existing conformance suite
  (`src/slice.rs:6181`) stay green unchanged.

## 10. Review Notes

<!-- adversarial pass appended below -->
