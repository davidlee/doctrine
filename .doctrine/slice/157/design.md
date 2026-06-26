# SL-157 Design — Checkout-independent trunk advance: strip the speculative None-leg resync

> Governed by ADR-012 (dispatch integration topology) — **no ADR-012 decision
> changes** (D4 CAS contract preserved in full; see §5). Successor to SL-121
> (leg-aware advance): SL-121 *introduced* the leg-aware branch; SL-157 *deletes*
> the speculative half of its None leg, **superseding SL-121 design §2.2**. The
> non-FF auto-merge that would reverse ADR-012 D2/D4 is split out to **RFC-006**
> (non-goal, see slice-157.md).
>
> Evidence base: `research.md` (go/no-go: **Go**, thesis CONFIRMED, no
> counter-evidence). `notes.md` carries the onboarding map. This design is the
> deletion map + the verification alignment + the governance/spec impact (§5).

## 1. Problem & root cause

`advance_row` (`src/dispatch.rs:1812`) branches the *mechanism* of a real advance
on the target ref's checkout state:

- **None leg** → `advance_pure_ref` (`dispatch.rs:1822`): `update_ref_cas`, **then
  a speculative post-CAS re-probe + resync** (`1842-1848`). If the ref raced into a
  checkout in the probe→CAS window, it `reset --hard`s a clean racer
  (`AdvancedResynced`) or warns `RacedDesync` on a dirty one.
- **Some leg** → `advance_checked_out` (`dispatch.rs:1859`): atomic `merge
  --ff-only` in the live worktree; refuses non-FF.

**The hazard RFC-005 H2 still carries — R1 (`RacedDesync`, low×high), R3, R4
(IMP-122 F-1/F-2 resync hardenings) — lives *entirely* in the None leg's post-CAS
resync (`1842-1848`).** The resync defends a **None→Some transition**: a ref that
was not-checked-out at the branch point (1812) becoming checked-out by the time the
CAS lands (1842).

**That transition cannot occur under the project's load-bearing invariants**
(`research.md` §1, confirmed against AGENTS.md line 24 + `mem.signpost.project.orientation`):

| Ref | Invariant | `worktree_for_ref` | Leg |
|-----|-----------|--------------------|-----|
| `main` (trunk) | **Never** checked out — buffer ref, advanced via `git fetch . edge:main` | always `None` | pure-ref, always |
| `edge` | **Always** checked out — primary worktree (AGENTS.md mandate) | always `Some` | checked-out, always |

So the re-probe at 1842 can only return `None` (for `main`) — the `Some` arms
(1844-1848) are reachable **only if someone checks out `main`**, which AGENTS.md
forbids. The guard adds no safety; it *is* the R1/R3/R4 hazard. **Delete the
condition, don't harden the window** (RFC-005 OQ-5 steer).

**This is an operating-model invariant, not a Git impossibility.** Git itself
permits the state — manually checking out the delivery ref in another worktree is
mechanically possible. Doctrine's dispatch posture forbids it (AGENTS.md line 24),
so it is **outside SL-157's supported behaviour**. The slice removes a guard
against a transition the operating contract excludes; it does not claim the
transition is physically unrepresentable.

The earlier ISS-038 phantom was the *pre-SL-121* pure-CAS-on-a-checked-out-ref
path — already retired by SL-121's leg-aware advance + the M4 dirty pre-gate. It
is **not** what this resync guards.

## 2. Locked decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | **Strip the post-CAS resync.** On `RefCas::Updated`, disposition is unconditionally `AdvancedPureRef` — no re-probe, no `resync_worktree_hard`, no `RacedDesync`. | The guarded race is impossible (§1). The guard is the sole locus of R1/R3/R4; deleting it dissolves all three at the mechanism. |
| D2 | **Retire `resync_worktree_hard`** (`git.rs:1373`) + its unit test. | Sole production caller is the deleted resync (OQ-D, grep-confirmed `research.md` §3). |
| D3 | **Retire `Disposition::RacedDesync`** (variant `dispatch.rs:2272` + `label()` arm 2284). | Reachable only from the deleted resync. `report_integrate` needs **no** structural change — `RacedDesync` rode the catch-all `disp =>` arm (1906), never a dedicated branch (`research.md` §3, supersedes the notes TODO). |
| D4 | **Keep the checked-out leg unchanged** — `advance_checked_out` + `ff_advance_in_worktree` (`git.rs:1308`) + their tests. | The safe atomic path for the always-checked-out `edge`. Force-CASing `edge` (alt ii) would desync the dev's live tree = the ISS-038 phantom. Load-bearing, not legacy. |
| D5 | **Keep the M4 dirty pre-gate** (`dispatch.rs:1753`) unchanged. | `worktree_for_ref(main)` is always `None`, so it only ever fires for a checked-out target — i.e. edge-dirty protection. Still wanted. |
| D6 | **No ADR-012 Revision; one SPEC-022 prose edit, deferred to reconcile** (§5). | The stripped resync is **SL-121 design** mechanism, *below* the ADR — ADR-012's text never mentions it (grep-confirmed); D4's CAS contract is preserved in full, so no ADR-012 decision changes. The only governance surface that names the resync is SPEC-022's prose body (`spec-022.md:141`); strip it via a `modify` REV at **reconcile** (after code lands), not now (else spec leads code). The SPEC-022 `.toml` responsibility already conforms. |

**Rejected — pure one-leg integrate (alt ii):** force every target ref
not-checked-out so the checked-out leg could retire. Fights AGENTS.md's
primary-on-edge mandate; operationally hostile. Not pursued (slice-157.md
Non-Goals).

## 3. Current vs target behaviour

**`advance_pure_ref` — the whole change.**

Current (`dispatch.rs:1839-1851`):
```rust
RefCas::Updated => {
    row.status = LedgerStatus::Verified;
    planned.clone_into(&mut row.applied_new_oid);
    let disposition = match git::worktree_for_ref(root, &row.target_ref)? {
        None => Disposition::AdvancedPureRef,
        Some(wt) if git::tree_clean(&wt)? => {
            git::resync_worktree_hard(&wt, planned)?;
            Disposition::AdvancedResynced
        }
        Some(_dirty) => Disposition::RacedDesync,
    };
    Ok(RowOutcome::Done { disposition })
}
```

Target:
```rust
RefCas::Updated => {
    // Not-checked-out advances are pure ref CAS only. Do NOT re-probe and
    // resync a worktree after CAS: under Doctrine's dispatch posture the
    // delivery ref is never checked out, and the post-CAS resync was the
    // RacedDesync / IMP-122 hazard (SL-157).
    row.status = LedgerStatus::Verified;
    planned.clone_into(&mut row.applied_new_oid);
    Ok(RowOutcome::Done { disposition: Disposition::AdvancedPureRef })
}
```

CAS-and-done. The `Moved` arm (refusal), the `advance_row` classification
(no-op / moved / advance), and the whole Some leg are untouched. The fn doc
(`1818-1821`) loses its "then the §2.2 None-leg post-CAS re-probe…" clause. The
comment above is **required** — it prevents a future reader from re-adding the
"defensive" resync without the operating-model context.

**Observable behaviour delta: none under the supported Doctrine worktree posture.**
Before: CAS succeeds → probe → (for `main`, always `None`) → `AdvancedPureRef`.
After: CAS succeeds → `AdvancedPureRef`. Identical post-state on every reachable
input; one fewer probe, two dead arms removed (`research.md` §6). (In the
*unsupported* state where `main` is checked out during the old CAS→re-probe
window, behaviour does change — old code resynced or reported `RacedDesync`, new
code reports `AdvancedPureRef`. That state is excluded by the operating contract
(§1); preserving its behaviour would preserve the hazard.)

## 4. Code-impact map (A's whole footprint)

**Remove:**

| Item | Location | Note |
|------|----------|------|
| Post-CAS re-probe + resync `match` | `dispatch.rs:1842-1848` → unconditional `AdvancedPureRef` | The deletion (D1) |
| `advance_pure_ref` doc clause | `dispatch.rs:1818-1821` | Drop the re-probe/resync sentence |
| `resync_worktree_hard` fn | `git.rs:1373-1376` | Sole caller deleted (D2) |
| `resync_worktree_hard` unit test | `git.rs:4022-4046` | Tests the deleted fn; shared helper `main_at_c1_with_descendant_c2` has 3 other callers → not orphaned |
| `Disposition::RacedDesync` variant | `dispatch.rs:2272` | + doc `2268-2271` (D3) |
| `RacedDesync` `label()` arm | `dispatch.rs:2284` (`"raced-checkout-desync"`) | |
| `AdvancedResynced` doc trim | `dispatch.rs:2260-2264` | Drop "or a None-leg … resynced"; it is now *only* the checked-out ff leg |
| `report_integrate` doc trim | `dispatch.rs:1893` | Drop the "`raced-checkout-desync` is a non-fatal warning line" sentence (no code branch exists — §3 D3) |

**Keep unchanged (load-bearing — do NOT touch):**
`advance_checked_out` (1859) · `ff_advance_in_worktree` (`git.rs:1308`) + its
tests · M4 dirty pre-gate (1753) · `worktree_for_ref` · `update_ref_cas` · the
`advance_row` branch point (1812, the None/Some split itself stays — `main` keeps
the None leg, `edge` keeps the Some leg) · `report_integrate` body (1895+).

**Design-target selectors:** `src/dispatch.rs`, `src/git.rs`.

## 5. Governance & spec impact (D6)

The pre-split scope assumed an ADR-012 mechanism Revision. Inspection (grep of
ADR-012 + SL-121 design, 2026-06-26) corrected that: **there is no ADR-012 decision
to revise, and exactly one SPEC-022 prose line to strip.**

### ADR-012 — no change

ADR-012's text mentions `ff-only`/CAS **only** in the trunk-projection contract
(D4: every ref update a CAS, no-op-if-planned, refuse-if-moved, never auto-resolve).
It says **nothing** about worktree resync, the None-leg re-probe, the checked-out
leg, or `RacedDesync`. SL-157 **preserves D4 in full** — every advance stays a 3-arg
CAS; no force-push, no auto-resolve; non-FF still refused. So no ADR-012 decision
changes; **no Revision against ADR-012.** (The D2/D4 *reversal* — non-FF auto-merge
— is RFC-006's, gated by external review per ADR-014.)

### SL-121 design — superseded (slice-level)

The stripped mechanism (worktree-aware advance, None-leg post-CAS re-probe/resync,
the `{AdvancedResynced, AdvancedPureRef, RacedDesync}` dispositions) lives in
**SL-121 design §2.2** — *below* the ADR. SL-157's design supersedes that sub-section
of SL-121's; no governance vehicle needed for a per-slice design supersession (the
`references SL-121` edge + this note carry it).

### SPEC-022 — one prose edit, deferred to reconcile

The only durable-governance surface naming the resync is SPEC-022's prose body:

> `spec-022.md:140-141`: *"a not-checked-out target advances by pure `update_ref_cas`
> **(with a post-CAS re-probe that resyncs a newly-checked-out ref)**;"*

**Edit:** strike the parenthetical → "advances by pure `update_ref_cas`". The
SPEC-022 `.toml` responsibility 4 already says only *"pure `update_ref_cas`"* — it
**already conforms**; no `.toml` change. This is a single prose strike.

**Vehicle & timing:** a SPEC change routes through a REV (`revision change add
--action modify --target SPEC-022`, surfaced-for-manual at apply → hand-strike line
141), per ADR-013 / the reconcile model ("REV for governance/spec"). Authored at
**reconcile**, *after* the code lands — not now, else the spec would describe code
that doesn't yet exist (the same governance-ahead-of-code hazard that retired the
ADR-012 Revision). Recorded here as a known reconcile-stage obligation so it is not
lost. Verb shapes confirmed via CLI: `revision new` / `revision change add` /
`revision approve` / `revision apply`.

## 6. Verification alignment (behaviour-preservation)

The change is a **deletion**, not an addition — the existing suites are the proof
(behaviour-preservation gate, AGENTS.md). Expect **no edits** to these; they stay
green unchanged. In `tests/e2e_dispatch_sync.rs`:

| Test | Line | Proves |
|------|------|--------|
| `integrate_trunk_fast_forwards_then_is_idempotent` | 767 | FF advance + idempotent replay |
| `integrate_trunk_refuses_non_fast_forward` | 803 | Non-FF refusal preserved |
| `integrate_refuses_clobbered_prepared_ref` | 897 | CAS refusal on moved target |
| `integrate_trunk_checked_out_ff_leaves_clean_tree` | 962 | **VT-2** — checked-out FF leg atomic, no phantom |
| `integrate_trunk_not_checked_out_advances_ref_leaves_live_checkout_clean` | 1000 | **VT-1** — pure-ref CAS doesn't desync a live checkout; exercises the surviving `None → AdvancedPureRef` path |
| `integrate_report_emits_disposition_and_preserves_stdout_reflist` | 1121 | Asserts `(advanced+resynced)` + `(no-op)` tokens literally. **Its fixture checks `main` out** (1138-1139) → the **checked-out** leg, which `AdvancedResynced` is *kept* for (D4, `dispatch.rs:1872`). NOT a None-leg test — unaffected by the deletion. |

VT-1 is the load-bearing regression: it advances a not-checked-out ref by pure CAS
and asserts the live checkout is untouched — exactly the path SL-157 keeps.

**The `advanced+resynced` label survives** — it is the checked-out leg's
disposition (1872), kept by D4. No test asserts it via the None-leg resync path:
the only None-leg resync coverage is the `git.rs` unit test below, which goes with
the fn. (Adversarial self-review confirmed: no e2e exercises the deleted Some-arms
— consistent with §1, they are unreachable without checking out a not-checked-out
ref mid-run, which a fixture cannot orchestrate honestly.)

**Only removal:** the `resync_worktree_hard` unit test
(`resync_worktree_hard_resyncs_stale_index_after_pure_ref_advance`,
`git.rs:4022-4046`), deleted with its fn. Its fixture helper
`main_at_c1_with_descendant_c2` has 3 other callers (`git.rs:3960/3987/4009`) →
**not orphaned**, stays live. **No new test needed** — no surviving behaviour is
added; the removed arms were unreachable under the invariants.

**We intentionally do not add a regression test for the unsupported
`main`-checked-out race.** Those Some-arms handled a delivery-ref checkout
transition the operating contract excludes (§1); a test pinning their behaviour
would pin the hazard SL-157 removes. The absence of such a test is deliberate, not
a coverage gap.

**Gate:** `just check` (fast inner loop) → `just gate` before commit. Plain
`cargo clippy` (no `--all-targets`), zero warnings.

## 7. Downstream (informational, not this slice)

- **IMP-122** (F-1/F-2 resync hardenings) targets the exact deleted code → closable
  after SL-157 lands (`research.md` §6).
- **RFC-006** extends A at `plan_trunk_row` (plan-time merge-oid), disjoint from
  A's advance-leg edit → no rework if B follows A.
- **R2** (`/close` ISS-030 recovery) — independent skill fix, carried separately.
