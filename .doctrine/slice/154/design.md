# Design SL-154: Reliable conformance-registry capture

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

RFC-004 v0.1 (SL-147) shipped `slice conformance`, which diffs a slice's declared
`design-target` selectors against its **actual** git delta. The actual-side input
is the arm-neutral **conformance registry** —
`.doctrine/state/slice/NNN/boundaries.toml` (runtime tier) — one `[[boundary]]`
row per landed phase. The consumer fail-closes when the registry is incomplete, so
an unpopulated registry makes conformance unavailable **at audit — exactly when it
is wanted**.

Two landing paths feed the registry and both leak in practice:

- **ISS-051 (solo path):** the final phase of a slice can land no row.
- **ISS-052 (funnel path):** a dispatched slice can reach audit with the registry
  empty (SL-153).

This slice closes both leaks so every landed phase, by either path, deposits
exactly one row — no manual `record-delta` bootstrap at audit. Registry-population
only; the conformance consumer and its algebra are untouched.

## 2. Current State

**Empirically root-caused (code is authority; the SL-147/SL-153 registries on disk
were since hand-bootstrapped, so root-cause is from code + topology, not live
forensics).**

**Solo binding** (`state.rs::capture_phase_boundary`, bound to `set_phase_status`,
slice.rs:723):
- `in_progress` flip → stamps `code_start_oid = HEAD` into the **phase sheet**
  (`.../NNN/phases/phase-NN.toml`, runtime tier) once.
- `completed` flip → reads the stamp back, records `(start, HEAD)` via
  `record_source_delta` (F-6 guard: `is_ancestor(start,end)` + non-merge `end`;
  upsert by phase).
- An **empty range** (`start == end`) **records fine** (boundary.rs:14; `is_ancestor`
  admits equal). So the scope's hypothesis (a) — "end read early → start==end →
  dropped row" — is **refuted**. A row is dropped **only when the start-stamp is
  absent** (state.rs:524 named-warning degrade): the phase never entered
  `in_progress` under the current binding (e.g. a stale PATH binary flipped it), or
  the `.doctrine/state` tree was wiped between the two flips. The **final phase** is
  the fragile case — it most often spans a handover / `/next` / audit-prep boundary.
- `init_phases` is **per-file-skip** (writes only missing pairs, prunes only under
  `--prune`), so re-running `slice phases` does **not** clobber the stamp; only a
  full `.doctrine/state` wipe does — which takes `boundaries.toml` with it (same
  tier). Relocating the stamp to be "durable" therefore buys nothing.

**Funnel** (`dispatch.rs::run_record_boundary`, the sole non-test caller of
`ledger::record_boundary`):
- Registry population rides a **skippable, per-arm hand-step at funnel Record beat
  (router step 8)**:
  - **claude** → `dispatch record-boundary` writes the dispatch **ledger**
    (`.doctrine/dispatch/NNN/boundaries.toml`) **and** the registry (`:606` + `:614`).
  - **codex/pi** → `slice record-delta` writes the **registry only**; this arm has
    **no dispatch ledger**.
- No machinery beat guarantees a landed phase deposits a row. SL-153 (claude arm,
  mixed-mode) reached audit empty: the hand-step did not reliably populate the
  registry. The coord worktree is a **linked** worktree (`git worktree add`,
  dispatch.rs:3375), so `primary_worktree` resolves to the session main tree —
  target divergence is **not** the cause; unreliable hand-discipline is.

**Constraint discovered.** `slice conformance` runs at **audit**, and stage-2
`dispatch sync --integrate` is `/close`'s job **post-audit** (dispatch router
Conclude). So the registry must be complete by **`prepare-review`** (the mandatory
pre-audit conclude beat), **not** integrate. `prepare-review` already reads the
boundaries ledger (dispatch.rs:1523) to project per-phase refs.

**ISS-039 (hard constraint on the derive source).** The dispatch ledger
`boundaries.toml` is written to the **live coordination worktree** but is **never
committed** to `dispatch/NNN` (only `journal.toml` is tracked there). `read_ledger`
(dispatch.rs:1991) deliberately sources from the branch **object-db**
(`git::read_path_at(coord_ref, …)`, so it works stage-1 *and* stage-2), so it reads
`Boundaries::default()` (**empty**) — this is why `plan_phases` projects 0 phase
cuts on the claude arm today. ISS-039 is **out of scope** (RFC-005 H3, own track),
so the derive **must not** read the committed ref — it must read the **live coord
worktree's working file** (located via `git::worktree_for_ref`), which prepare-review
(stage-1) can still see because the worktree is removed only *after* this beat.

**Pre-built machinery to reuse:** `state::check_completeness` (state.rs:654) — a
pure F-2 cross-check (every completed phase has exactly one row; no extras /
duplicates), already the gate `slice conformance` uses to refuse an unclean diff.

## 3. Forces & Constraints

- **ADR-001 (layering):** `boundary.rs` is a leaf; `state.rs` / `dispatch.rs` are
  engine. New start-resolution logic must be **pure** (no clock/rng/git/disk) — the
  reads stay in the shell, passed in (the date/uid pattern).
- **POL-002 (platform independence):** recording rides doctrine-owned contracts
  (recorded SHAs, the `dispatch/NNN` branch name), never host commit conventions.
  The solo arm-guard keys on `dispatch/NNN` — keep it.
- **Behaviour-preservation gate:** existing `set_phase_status` and dispatch suites
  must stay green unchanged.
- **Audit-before-integrate** (§2): enforcement point is `prepare-review`.
- **Arm asymmetry:** the dispatch ledger is **claude-only**. A clean *symmetric*
  derive needs codex/pi to also write a ledger — but `plan_phases` (dispatch.rs:2049,
  unconditional on the ledger) couples that to **codex/pi `phase/<N>` ref projection
  turning on**, an untested behaviour change. Out of scope here → **IMP-171**.
- **ISS-039 (out of scope):** the ledger is never committed to `dispatch/NNN`, so the
  derive reads the **live coord worktree working file**, not the committed ref (§2).
  This slice neither depends on nor fixes ISS-039.
- `record-delta` stays the manual escape hatch; this slice removes the *need* for it
  on a normal slice, not the verb.

## 4. Guiding Principles

- **One recording model end-to-end.** One engine writer, one start-resolution rule,
  shared by solo and funnel. The `in_progress` stamp is not jank — it is the
  *precision* input (it excludes inter-phase knowledge/notes commits); keep it,
  fall back when it is absent.
- **Enforce structurally, not by discipline.** A landed phase deposits its row as a
  consequence of machinery (the binding; `prepare-review`), not an orchestrator
  remembering a step.
- **Reach audit complete or halt.** It must be impossible to reach audit with a
  silently-incomplete registry.
- **Build P as Q's foundation** (IMP-171): every seam this slice touches is where
  the symmetric-derive follow-up extends — additive, not rework.

## 5. Proposed Design

### 5.1 System Model

One registry (`state::record_source_delta`, the sole engine writer, unchanged) fed
by three structural sources, cross-checked by one gate:

```
solo completed-flip ─┐
                     ├─> record_source_delta ─> .doctrine/state/slice/NNN/boundaries.toml
prepare-review derive ┤        (upsert by phase)              │
codex/pi record-delta ┘                                       │
                                                              v
prepare-review ──> check_completeness(completed, recorded) ──> Complete | HALT
```

- **Solo:** the `completed` flip resolves start via **`stamp ?? prev_end ?? base`**
  (A′) and records `(start, HEAD)`.
- **Funnel (claude):** `prepare-review` **derives** registry rows from the ledger
  it already reads — the enforced beat. `run_record_boundary` **sheds its registry
  half** (ledger-only); the registry is no longer hand-written on the claude arm.
- **Funnel (codex/pi):** keeps step-8 `record-delta` (no ledger to derive from).
- **Gate (both arms):** `prepare-review` runs `check_completeness` over **all**
  completed phases (solo + funnel) and **bails** on any gap.

### 5.2 Interfaces & Contracts

New pure leaf-level helper (lives with the registry logic in `state.rs`; pure,
no IO):

```rust
/// Resolve a phase's code_start for the registry record (A'): the stamped HEAD
/// (precise — excludes inter-phase knowledge commits) wins; else the previous
/// phase's recorded code_end (the contiguous chain); else a known base; else
/// None (no row can be recorded — caller degrades with a named warning).
pub(crate) fn resolve_phase_start(
    stamp: Option<&str>,      // code_start_oid stamped on in_progress (may be lost)
    prev_end: Option<&str>,   // code_end_oid of the previous recorded phase
    base: Option<&str>,       // slice base, when knowable (None for solo today)
) -> Option<String>
```

Solo `completed` arm of `capture_phase_boundary` (rewired): instead of degrading
when the stamp is absent, read `prev_end` from `read_source_deltas` (the immediately
preceding recorded phase) and call `resolve_phase_start`. Stamp-present behaviour is
**unchanged** (precision preserved).

Funnel — `run_prepare_review` (dispatch.rs) gains, after phase planning:

```text
1. derive (claude):  locate the live coord worktree via
                     git::worktree_for_ref(root, "refs/heads/dispatch/NNN");
                     read its WORKING ledger <coord>/.doctrine/dispatch/NNN/boundaries.toml
                     (ISS-039-independent — NOT the committed ref, which is empty);
                     for each boundary row -> record_source_delta(row).
                     Worktree absent (already removed) or empty (codex/pi: no ledger)
                     -> derive is a no-op; the gate then speaks.
2. gate (both arms): completed = completed PHASE-NN set (phase rollup / read_phase_status)
                     recorded  = read_source_deltas(...).map(|r| r.phase)
                     match check_completeness(&completed, &recorded) {
                       Complete            => continue,
                       Incomplete { gaps } => bail!("prepare-review: conformance registry
                                                     incomplete: <gaps>; record-delta the
                                                     missing phase(s) before audit"),
                     }
```

The derive reads the **working** ledger (which `record_boundary` writes) rather than
the committed ref, so it is **strictly more robust than today's inline double-write**
for the SL-153 failure shape: even when `record-boundary`'s inline registry write
(`:614`) errored *after* its ledger write (`:606`) succeeded, the row is recovered
here from the surviving working ledger.

`run_record_boundary`: drop the `record_source_delta` call (`:614`); keep the
`ledger::record_boundary` write. Now ledger-only.

`record-delta` verb (`slice.rs::run_record_delta`): **unchanged** (escape hatch).

### 5.3 Data, State & Ownership

- **Registry** `.doctrine/state/slice/NNN/boundaries.toml` (runtime, primary-tree
  resolved): unchanged shape (`[[boundary]]` rows). Writers: solo binding,
  `prepare-review` derive (new), codex/pi `record-delta`, manual `record-delta`.
  All go through `record_source_delta` (upsert by phase → idempotent, no dupes).
- **Ledger** `.doctrine/dispatch/NNN/boundaries.toml` (claude-only): now the **sole**
  output of `run_record_boundary`; `prepare-review`'s derive source — read from the
  **live coord worktree's working file** (`worktree_for_ref` → `dispatch_dir`), NOT
  the committed ref (empty under ISS-039). Ownership unchanged (`ledger.rs`); ISS-039
  itself (committing the ledger) stays out of scope.
- **Phase sheet** `.../NNN/phases/phase-NN.toml`: still carries the `code_start_oid`
  stamp (precision input). No relocation (§2: futile).

### 5.4 Lifecycle, Operations & Dynamics

**Solo slice.** Each phase: `in_progress` (stamp) → work+commits → `completed`
(record via A′). Final phase with a lost stamp now self-heals from the prior row's
`code_end`. Pure-solo slices never hit `prepare-review`; their safety net stays the
existing conformance-time `check_completeness`.

**Dispatched slice (claude).** Per phase the funnel writes the **ledger** (step 8,
`record-boundary`, ledger-only now). At Conclude, `prepare-review` **derives** all
registry rows from the ledger, then **gates**. Registry complete before
`slice status audit`.

**Mixed-mode (SL-153).** Solo P01/P02 → A′ binding at their completed flips; funnel
P03/P04 → `prepare-review` derive. The gate cross-checks **all four** → composition
verified, not assumed. (obj 3)

**Dispatched slice (codex/pi).** Step-8 `record-delta` writes the registry; the
`prepare-review` gate halts on any miss → operator runs `record-delta` → re-run
`prepare-review`. Symmetric auto-derive is IMP-171.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1:** every completed phase has exactly one registry row by the time audit
  runs (binding for solo; derive+gate for funnel; gate enforces).
- **INV-2:** `record_source_delta` upsert-by-phase ⇒ no duplicate rows even if a
  phase is recorded by two sources (e.g. solo binding then a redundant derive).
- **INV-3:** the stamp, when present, is authoritative for solo start (precision);
  the chain is only a fallback.
- **Edge — lost-stamp P01 (solo):** no prev row and no recorded base ⇒
  `resolve_phase_start` returns `None` ⇒ named-warning degrade + escape hatch. The
  residual hole; smallest (P01 flips early, in-session) and caught by the gate /
  conformance. **Accepted** (no base source invented — D2/§7).
- **Edge — empty-code phase:** `start == end` still records (unchanged); contributes
  a row, satisfies the gate, emits no phase ref.
- **Edge — derive idempotency:** `prepare-review` re-run re-derives the same rows
  (upsert) — safe.

## 6. Open Questions & Unknowns

- **OQ-1 (resolved → D1):** drop `run_record_boundary`'s registry half vs keep the
  double-write. **Resolved: drop** (one-writer-per-tier; registry sourced at
  `prepare-review` derive).
- **OQ-2 (resolved → D2):** solo P01 base source. **Resolved: none**; accept the
  residual degrade.
- **OQ-3:** does any other live caller of `record_source_delta` assume the funnel
  also wrote it per-phase? (Audit at impl: only the three sources in §5.2; the
  drive-time consumer was conformance, which runs post-`prepare-review`.)
- **OQ-4:** `dispatch_dir` is private to `ledger.rs` (:375). The derive needs the
  worktree-relative ledger path — expose a `pub(crate)` accessor (or a
  `ledger::read_boundaries_at(worktree_root, slice)` reader) rather than rebuilding
  the path string in `dispatch.rs` (DRY). Resolve at impl.

## 7. Decisions, Rationale & Alternatives

- **D1 — A′ (stamp ?? chain ?? base), not pure-chain (B).** Pure-chain drops the
  stamp and folds inter-phase knowledge/notes commits into the next phase's range →
  spurious `undeclared` conformance noise. The stamp is the precision mechanism;
  keep it, fall back only when absent. A′ is strictly more correct than B and
  cohesive (one rule).
- **D2 — `prepare-review` derive + gate (shape 3 + 1), not integrate, not a bare
  hand-step.** Audit precedes integrate, so integrate is too late. The derive makes
  claude-arm population structural; the gate (reusing `check_completeness`) makes
  *reaching audit incomplete* impossible on both arms.
- **D3 — Drop the claude registry hand-write (OQ-1).** Registry now sourced solely
  at `prepare-review` derive on the claude arm — one writer per tier, no skippable
  double-write.
- **D4 — Defer codex/pi symmetric derive to IMP-171 (P-as-Q-foundation).** A
  codex/pi ledger couples to `phase/<N>` projection (dispatch.rs:2049), an untested
  behaviour change unrelated to registry population; the reproduction (SL-153) is
  claude-arm. Ship the claude fix + arm-symmetric gate now; layer the symmetric
  derive (and codex/pi phase-refs, tested) later.
- **Alternative rejected — read-time fallback** (conformance reads the ledger when
  the registry is absent): papers over the write gap; scope-rejected; and the ledger
  is claude-only.

## 8. Risks & Mitigations

- **R1 — `prepare-review` derive reads a stale/partial/absent ledger.** The derive
  reads the live coord worktree working ledger (ISS-039-independent). If a phase's
  ledger row is missing, or the coord worktree was already removed (degenerate
  ordering — convention is remove *after* prepare-review), the derive can't produce
  that row → the **gate** catches it and halts with the named gap + `record-delta`
  remedy. Net: fail-closed, never silent. (If ISS-039 is later fixed, the derive can
  switch to the committed ref — additive, no rework.)
- **R2 — gate false-positive blocks a legitimate Conclude.** `check_completeness`
  keys on completed `PHASE-NN` vs recorded phase ids; a blocked-but-not-completed
  phase is excluded. Mitigation: gate uses the **same** completed-set source as
  conformance (parity), and the bail message names the exact gaps + the
  `record-delta` remedy.
- **R3 — behaviour regression in `set_phase_status`.** The rewire touches only the
  `completed` arm's start resolution; stamp-present path byte-identical. Mitigation:
  existing binding suite stays green; add the lost-stamp regression test.
- **R4 — dropping the registry half breaks a non-dispatch caller.** Only
  `run_record_boundary` (funnel) loses the call; solo + manual paths unaffected.
  Mitigation: OQ-3 audit at impl.

## 9. Quality Engineering & Validation

Pure unit (no IO):
- `resolve_phase_start`: stamp wins; stamp-absent→chain; both-absent→base;
  all-absent→None.
- `check_completeness`: already covered (reuse).

Engine / integration:
- **VT — solo lost-stamp final phase:** flip in_progress, **clear the stamp**, flip
  completed → a row lands via chain-fallback (the ISS-051 regression).
- **VT — solo stamp-present unchanged:** behaviour-preservation (precise start).
- **VT — funnel derive:** ledger with N boundaries → `prepare-review` populates N
  registry rows.
- **VT — funnel gate halts:** completed phase with no row → `prepare-review` bails
  with the named gap (the ISS-052 regression).
- **VT — mixed-mode:** solo rows + derived funnel rows together pass the gate.
- **VT — derive idempotent:** second `prepare-review` re-derives, no duplicates.
- **Behaviour-preservation:** `e2e_dispatch_sync` + `set_phase_status` suites green
  unchanged.

`just check` green, clippy plain (no `--all-targets`), per commit.

## 10. Review Notes

### Internal adversarial pass (2026-06-26)

- **F-1 (design-sinking, fixed in-draft) — the derive read an empty source.** The
  first draft derived the registry from the boundaries ledger via the committed coord
  ref (`read_ledger`, dispatch.rs:1523). **ISS-039** means that ref is never written
  (only `journal.toml` is committed), so the derive would have produced **0 rows and
  halted every claude dispatch** — breaking the exact arm this slice targets, on an
  out-of-scope bug. **Fix:** derive from the **live coord worktree working file**
  (`worktree_for_ref` → working `boundaries.toml`), which stage-1 prepare-review can
  still read (worktree removed only after). Bonus: strictly more robust than today's
  inline double-write for the SL-153 shape (ledger written, inline registry write
  errored). Folded into §2, §3, §5.2, §5.3, §8 R1.
- **F-2 (accepted residual) — lost-stamp P01.** A solo first phase with no stamp and
  no prev row and no recorded base cannot record (no base source exists). Smallest
  case (P01 flips in-session), caught by the gate / conformance, escape hatch remains.
  Accepted (D2); not engineered around.
- **F-3 (DRY, deferred to impl) — `dispatch_dir` privacy.** Derive needs the
  worktree-relative ledger path; expose a `ledger` accessor rather than rebuilding the
  string (OQ-4).
- **Checks that held:** `record_source_delta` upsert ⇒ no dupes across sources (INV-2);
  arm-guard keys on `dispatch/NNN` (POL-002); `resolve_phase_start` is leaf-pure
  (ADR-001); behaviour-preservation via unchanged stamp-present path + existing suites.

<!-- Next: external adversarial pass — codex mcp (GPT-5.5). -->

