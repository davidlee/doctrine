# SL-181 Design — OQ-D reframe: confinement is the close, retract the positive-marker overclaim

Status: reshaped to REV-only (2026-07-03). Governed by ADR-006 (D2a/D2b),
ADR-012 (OQ-D). Originates from IMP-065. Residual tracked in RSK-014.

> **Reshape note (A2).** This slice began as "positive coordination marker"
> (IMP-065), was reframed to "anti-accident ref-corruption guard + REV," and is
> now **REV-only**. The guard is **retired**: RV-199 F-1 proved its mechanism
> unsound *and* confinement (SL-182/183/185) shipped the genuine close, leaving the
> guard's residual empty. The design below records why the guard cannot earn its
> place, and specifies the sole surviving deliverable: the Revision. **No code
> ships.** Prior guard design preserved in git history (`a334a962`, `244d7bc4`).

## 0. Thesis

OQ-D (ADR-012, RV-023 F-2) is the deferred "real close" of the dispatch
impersonation gap (ADR-011 D6/M2): an **unstamped worker** is `is_linked &&
marker_absent`, indistinguishable by absence from the markerless SL-064
coordination tree, so its **Orchestrator verb class** runs fail-open. ADR-012
carried IMP-065's *positive coordination marker* as the intended close.

**Two findings collapse that plan:**

1. **A positive marker cannot close it.** Worker identity is a presence-only file
   + optional `DOCTRINE_WORKER` env — **cooperative flags, not enforced
   boundaries** (RSK-014; ADR-006 D2a concedes "the fence … is the funnel + the
   jail, **not** a fail-closed CLI floor"). A capable worker `cd`s into the coord
   tree or forges any marker trivially.

2. **The genuine close — confinement — has now shipped.** SL-182 (Linux/bwrap
   claude-arm write-confinement, **ro shared `.git`**), SL-183 (macOS Seatbelt
   arm), SL-185 (subprocess-arm parity). Where a backend runs, the worker's writes
   — including **ref mutation** — are blocked at the OS floor, stamped or not.

So OQ-D's close is **enforcement (confinement), landed**, not a marker and not the
interim anti-accident guard this slice previously proposed. The sole remaining act
is to **make ADR-012 honest**: a Revision that retracts the overclaim and records
the shipped close. That Revision is this slice's only deliverable.

## 1. Why the anti-accident guard is retired (not reworked)

The reframed guard (retire target) refused Orchestrator-class verbs in a linked
worktree that is not the coordination tree, catching one honest-failure path: an
**unstamped worker that stays in its own tree and mutates shared refs**
(`gc`/`sync`/`import`/`candidate` — invisible to the `B..S` import file delta, so
the R-5 belt misses them). It never had a configuration where it both *matters*
and *works*:

| arm | worker runs cmds? | reaches shared refs? | guard adds anything? |
|---|---|---|---|
| confined — bwrap/Seatbelt (SL-182/183/185) | yes | **no** — ro `.git` (worker cannot self-commit; CLAUDE.md) | **no** — floor already blocks ref mutation |
| no sandbox backend (e.g. Linux, no `bwrap`) | **no** — SL-182 hook is **fail-closed** (`pretooluse-wrap.sh`: "missing bwrap → deny; never falls through") | n/a | **no** — worker executes nothing |
| standalone clone (IDE-024 "Path C") | yes | own object store only; main `.git` ro | **no** — corruption contained to disposable clone; tree is **not `is_linked`**, guard is blind to it |
| **unconfined** (vanilla pi subagent; hypothetical) | yes | yes | **no** — RV-199 **F-1**: the worker fork rides `dispatch/<name>` (`create.rs:238`), so `is_coordination_worktree` false-**allows** it. Void on the one arm that needed it. |

**There is no configuration where the guard is both load-bearing and correct.**
Where it could matter (unconfined), F-1 voids its mechanism; everywhere else the
confinement floor or clone topology closes the gap first. RV-199's "heresy
confessed" verdict is upheld — the redress is **removal**, not the A1 signal
rework. Reworking the coordination signal (anchor on registered dispatch state)
would be effort spent hardening a guard the OS floor has already made redundant.

## 2. The Revision — the sole deliverable

A **REV against ADR-012** (and the ADR-006 D2a/D2b notes). Content:

- **Retract** "the positive coordination marker (IMP-065) is **the real close** of
  OQ-D." It never could be — cooperative flags are not boundaries (§0.1).
- **Record the genuine close as shipped:** confinement — SL-182 (Linux/bwrap,
  claude arm, ro shared `.git`), SL-183 (macOS Seatbelt), SL-185 (subprocess
  parity). The prior framing ("closable, not-yet-landed"; "unclosable on claude")
  is superseded by **landed** enforcement.
- **Reclassify the residual (RSK-014):** OQ-D is **enforcement-closed on every arm
  that runs a confinement backend**. The bounded residual is only the degenerate
  cases, each self-limiting: no-backend → fail-closed (worker runs nothing);
  standalone clone → contained to a disposable object store. No cooperative-marker
  residual remains, because no marker is claimed.
- **Record the guard disposition:** an anti-accident branch-check was designed,
  inquisited (RV-199 F-1, blocker), and **retired** — the confinement floor
  subsumed it. SL-181 ships **no code**; its value is the honest reframe.
- **Supersede/close IMP-065:** the positive-marker proposal is retired,
  superseded by confinement (recorded at reconcile — §4).

ADR-006 D2a/D2b and ADR-012 are owner-locked (VH); the REV is the sanctioned
amendment path, routed via reconcile — **not** hand-edited.

## 3. Code impact: none

The slice touches **no `src/`**. All prior design-target selectors
(`guard.rs`, `worktree/shared.rs`, `git.rs`, `dispatch.rs`, `worktree/mod.rs`) and
scope-relevant selectors are cleared. `slice conformance` holds SL-181 to a zero
touch-set; any code edit under this slice id would be an undeclared-edit finding.

## 4. Verification / closure alignment

The deliverable is governance prose, not behaviour — verification is
**conformance + coherence**, mode `VA`/`VH`:

- **REV lands** against ADR-012 (+ ADR-006 D2a/D2b notes) via `revision new` →
  approve → apply; its statements match §2. (`VA`: an agent reads the applied
  ADR-012 and confirms the "real close" claim is retracted and points at
  confinement.)
- **No code delta.** `slice conformance SL-181` reports an empty design-target set
  and no undeclared edits. (`VT`-adjacent: the conformance delta is the test.)
- **RV-199 resolved** — all findings terminal (F-1/F-2/F-3 accepted; guard
  retired). No open blocker gates the slice.
- **Reconcile-time effects (not design):** IMP-065 closed as reframed-superseded;
  RSK-014 downgraded to "enforcement-closed on confined arms; bounded residual."
  These are the REV's *effects*, recorded at reconcile/close, not authored now.

## 5. Design decisions

- **D1 — retire, don't rework the guard.** A2. F-1 voided the mechanism;
  confinement voided the need (§1 table). **Locked** (user, 2026-07-03).
- **D2 — REV-only; no code.** The honest reframe is the whole slice. **Locked.**
- **D3 — REV authored at execute, not design.** Design specifies the REV's
  content (§2); the `revision` entity is created during execution (design/execute
  separation; ADR-013). **Locked.**
- **D4 — IMP-065 / RSK-014 dispositions deferred to reconcile.** They are effects
  of the REV landing, not design edits. **Locked.**

## 6. Superseded material

The retired guard design (positive coordination signal, `is_coordination_worktree`
predicate, `worker_guard` clause, truth table, per-harness accounting, OQ-A/B/C)
lived in `design.md` up to commit `244d7bc4` and is preserved in git history. It is
intentionally **not** carried forward: its mechanism was found unsound (RV-199 F-1)
and its purpose subsumed by confinement. Do not resurrect it without first showing
a configuration where the confinement floor is absent *and* the worker both
executes and reaches shared refs — §1 argues none exists.
