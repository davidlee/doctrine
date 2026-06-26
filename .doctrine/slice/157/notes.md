# SL-157 notes — Checkout-independent integrate

Status: **PHASE-01 IMPLEMENTED (completed) — next is `/audit` → `/reconcile` →
`/close`.** Solo `/execute` on fork `sl-157-phase-01` (base edge@`42c55624`).
Single atomic delta `da243b3d` (`src/dispatch.rs`, `src/git.rs`; +13/-75). NOT yet
landed on `main` — that is `/close`'s post-audit job. Fork retained.

**Execution record (2026-06-26):**
- EN-2 baseline GREEN before edits (env note: `DOCTRINE_RESERVATION_FALLBACK`
  must NOT leak into `just check`/`just gate` — it flips `reserve::tests::vt3_*`
  red; only the doctrine CLI reservation needs it).
- Delta exactly the keep/remove map (design §4): `advance_pure_ref` `RefCas::
  Updated` → unconditional `AdvancedPureRef` + do-not-re-add comment; retired
  `resync_worktree_hard` (+ unit test), `Disposition::RacedDesync` (+ `label()`
  arm); trimmed 3 stale docs. KEPT `AdvancedResynced`, helper
  `main_at_c1_with_descendant_c2` (3 callers), `ff_advance_in_worktree`, M4 gate,
  `report_integrate` body, D4 CAS contract.
- VT-1/VT-2 GREEN (`just check` + `just gate` rc=0, clippy zero warnings, no new
  test). VA-1 self-review PASS (footprint = resync machinery only, no over-deletion).
- **Reconcile/close obligations (carry forward):** SPEC-022 prose strike
  (`spec-022.md:140-141`) via `revision change add --action modify --target
  SPEC-022` at reconcile; close IMP-122 at slice close. (Detail in D6 below.)

---

Status (design stage, historical): **plan LOCKED — lifecycle `ready`, next is `/phase-plan` → `/execute`.**
Scope re-baselined + premise-corrected; `design.md` locked (`31be77aa` author,
`e53642e5` self-review, `26a867bc` GPT hardening, `66c688e8` D6 correction);
`plan.toml`/`plan.md` authored (`b1e764aa`, single phase PHASE-01); PHASE-01 sheet
materialised; `design-target` selectors recorded. No code yet.

**Execute posture:** code-changing slice → needs isolation (AGENTS.md: main
worktree stays on edge). One phase touching both files → solo `/execute` on a
worktree fork, land on `main`. PHASE-01 = the atomic deletion (keep/remove EX
pairs guard over-deletion). EN-2: verify baseline gate green on the fork BEFORE
edits. Then SPEC-022 strike at reconcile (design §5).

**Remaining:** `/plan` (next). **No design-stage Revision** — see below.

**D6 corrected (`/consult`, 2026-06-26):** the pre-split scope assumed an ADR-012
mechanism Revision. Grep of ADR-012 + SL-121 + SPEC-022 corrected it:
- **ADR-012 — no change.** Its text never names the resync; D4 CAS contract fully
  preserved. No Revision against ADR-012.
- **SL-121 design §2.2 — superseded** (slice-level; the resync mechanism lives
  there, below the ADR). Carried by the `references SL-121` edge + design §5.
- **SPEC-022 — one prose strike, deferred to reconcile.** `spec-022.md:140-141`
  carries the resync parenthetical "(with a post-CAS re-probe that resyncs a
  newly-checked-out ref)"; strip it. The `.toml` responsibility already conforms.
  Routes via `revision change add --action modify --target SPEC-022` at
  **reconcile** (after code lands, so spec doesn't lead code). Known obligation.

GPT verdict (design): no architecture block; two hardened risks were the "cannot
occur" framing (now: operating-model invariant, not Git impossibility) and the
behaviour-delta claim (now: scoped to supported posture).

**Self-review catch (folded into design §6):** `integrate_report_emits_disposition`
(e2e:1121) asserts `(advanced+resynced)` but via the CHECKED-OUT leg (fixture
checks `main` out) → survives. `AdvancedResynced` variant KEPT (D4, dispatch.rs:1872);
only its doc trims. Removed unit test's helper `main_at_c1_with_descendant_c2` has
3 other callers → not orphaned.

---

## Locked decisions (read these first)

1. **A/B split.** The original maximal scope bundled two separable things:
   - **A = this slice (SL-157).** Checkout-independence on the *advance* leg —
     specifically, strip the speculative post-CAS resync. FF-only preserved.
     Mechanism-only ADR-012 Revision.
   - **B = RFC-006** (new, linked). Non-FF trunk auto-merge + conflict surgery
     (absorbs IMP-127). **Reverses ADR-012 D2/D4 FF-only** → external review gates
     it. B edits `plan_trunk_row` (a merge-at-plan oid producer), **disjoint** from
     A's advance-leg edit → B extends A with no rework. (Merge oid MUST be produced
     at plan time, not advance — `commit-tree` committer-date is non-deterministic,
     would break D4 replay.)

2. **Scope premise corrected (the design caught it).** Original scope said "retire
   the checked-out leg, always pure CAS." **Backwards.** The hazard is the
   *not-checked-out* leg's resync, not the checked-out leg:
   - RFC-005 H2 localizes **R1/R3/R4 entirely to `advance_pure_ref`'s post-CAS
     re-probe + resync** (`dispatch.rs:1842-1848`). It names the checked-out leg
     the **safe** one (`ff` syncs ref+index+tree atomically; regression-proven by
     `integrate_trunk_checked_out_ff_leaves_clean_tree`).
   - **Real invariants:** `main` is *never* checked out (buffer ref, `git fetch .
     edge:main`) → pure-ref leg, always. `edge` is *always* checked out (AGENTS.md
     "primary worktree stays on edge") → checked-out leg, always. So the None→Some
     race the resync guards **cannot happen**. Delete the guard ⇒ R1/R3/R4 dissolve.
   - Force-CASing `edge` (the rejected framing) would desync the dev's live tree =
     ISS-038 phantom. User chose **(i) keep the safe atomic edge leg** over (ii)
     pure-one-leg (which fights AGENTS.md).

---

## Exact code-impact (the deletion) — A's whole footprint

**Remove (the hazard):**
- `advance_pure_ref` (`src/dispatch.rs:1822-1853`): on `RefCas::Updated`, replace
  the `worktree_for_ref` re-probe `match` (**1842-1848**) with unconditional
  `Disposition::AdvancedPureRef`. No re-probe, no resync, no `RacedDesync`.
- `Disposition` enum (`dispatch.rs:2255-2273`): drop the **`RacedDesync`** variant
  (2272) + its `label` arm `"raced-checkout-desync"` (2284). Trim the
  `AdvancedResynced` doc (2260-2264) — it's now *only* the checked-out ff leg, not
  "or a None-leg … resynced".
- `report_integrate` (`dispatch.rs:1895+`): **No structural change needed.**
  `grep -rn RacedDesync` confirms zero matches in the `report_integrate` match
  body — the variant was handled by the catch-all `disp =>` arm, identically to
  all other non-NoOp dispositions. Only the stale doc-comment at line 1893
  (mentions `raced-checkout-desync`) needs trimming. See `research.md` §3.
- `git.rs`: remove `resync_worktree_hard` (**1373-1376**) + its unit test
  (`resync_worktree_hard_*`, ~**4023-4037**). Sole production caller is the deleted
  resync (OQ-D grep-confirmed).

**Keep unchanged (load-bearing, do NOT touch):**
- `advance_checked_out` (`dispatch.rs:1859`) + `ff_advance_in_worktree`
  (`git.rs:1308`) + its unit tests — the safe atomic path for the checked-out
  `edge`.
- M4 dirty pre-gate (`dispatch.rs:1753`) — only ever fires for a checked-out
  target (`worktree_for_ref(main)` is always `None`), i.e. edge-dirty protection.
- `worktree_for_ref`, `update_ref_cas`, the `advance_row` branch point (1812) — the
  branch stays (edge → Some leg; main → None leg).

---

## Remaining design work (the `/design` to-do)

1. **Write `design.md`** — sections: current-vs-target behaviour; code-impact
   (the map above); verification alignment; the ADR-012 mechanism Revision scope.
   Small slice — design is mostly the deletion map + Revision + verification.
2. **Record `design-target` selectors** when code-impact locks:
   `doctrine slice selector add SL-157 "src/dispatch.rs" "src/git.rs" --intent design-target`
3. **ADR-012 Revision (mechanism-only).** Restate: the not-checked-out advance is
   pure ref CAS with **no** worktree resync (the None→Some resync is removed as
   defending an impossible transition). FF-only (D2/D4) + CAS-replay contract (D4)
   **preserved unchanged**. Route per ADR-013 (`doctrine revision …` — check verb
   shapes). Confirm minimal vs broader (OQ-E → answered minimal/mechanism-only).
4. **Adversarial review** (internal pass, then offer `/inquisition` or external
   codex reviewer), integrate, then `/plan`.

## Verification posture (behaviour-preservation)

Integrate safety semantics stay green **unchanged** — these are the proof, expect
NO edits to them:
- `tests/e2e_dispatch_sync.rs` (PHASE-05 set, ~727-1010): `integrate_trunk_fast_forwards_then_is_idempotent`,
  `integrate_trunk_refuses_non_fast_forward`, `integrate_refuses_clobbered_prepared_ref`,
  `integrate_edge_is_opt_in_*`, **`integrate_trunk_checked_out_ff_leaves_clean_tree`**
  (VT-2, checked-out leg), **`integrate_trunk_not_checked_out_advances_ref_leaves_live_checkout_clean`**
  (VT-1, pure-ref leg).
- Only removal: the `resync_worktree_hard` unit test (goes with the fn).
- Gate: `just check` (fast inner loop) / `just gate` before commit. Not yet run
  (no code).

## OQ resolutions (from preflight OQ-A..E)

- **OQ-A** — no `main` worktree exists to drop; `main` is already a bare ref. Done.
- **OQ-B** — `edge` rides the (safe) checked-out leg; not force-CASed.
- **OQ-C** — N/A: no conflict surgery in A (that's B/RFC-006).
- **OQ-D** — `resync_worktree_hard` → delete (sole caller is the resync);
  `ff_advance_in_worktree` → keep (edge needs it).
- **OQ-E** — ADR-012 Revision is mechanism-only.

## Reading list (governance)

1. **RFC-005** — Current posture #1 + H2 section (R1/R3/R4 localization is the
   spine of the corrected premise) + OQ-5.
2. **RFC-006** — B (the split-out non-FF auto-merge); shows what A is *not*.
3. **ADR-012** — D2/D4 (FF-only + CAS-replay, both preserved), D6 (legitimacy).
4. **ADR-013** — Revision routing for the mechanism change.
5. **AGENTS.md** — "primary worktree stays on edge" (the load-bearing invariant).
6. Memories: `mem.pattern.dispatch.close-integrate-shared-trunk-race`
   (the H2 friction B addresses), `mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree`.

## Commits (this design session)

- `49b18ad3` — split scope to A, create RFC-006, wire relations.
- `8520309d` — correct scope premise (strip None-leg resync; keep checked-out leg).
- All `.doctrine` changes committed (path-limited; other agents' index entries
  untouched). No code touched.

## Out of scope (don't let design absorb)

- B / non-FF auto-merge / IMP-127 / conflict surgery → RFC-006.
- Pure one-leg integrate (alt ii) → rejected (fights AGENTS.md primary-on-edge).
- R2 `/close` ISS-030 recovery → independent skill fix.
- Candidate-flow rewrite; ADR-012 D1/D5; IMP-174 split-brain.
