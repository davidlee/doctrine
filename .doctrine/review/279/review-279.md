# Review RV-279 — design of SL-221

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

This is the **second** inquisition of SL-221. RV-278 tried the now-superseded D1
merge design (F-2 tie-break, F-3 ordering, F-4 verification — all `design-wrong`,
verified). The standing design is **D-B1 (the seam collapse)** in `design.md`
§5/§7, externally un-reviewed. The reviewer must NOT relitigate the dead D1
merge; probe whether D-B1 *actually* dissolves F-2/F-3/F-4 and introduces no new
heresy.

Lines of interrogation against D-B1:

- **F-2/F-3 dissolution claim.** §5.3/§7 assert the single writer kills the
  same-phase tie-break ambiguity (F-2) and the row-order hazard (F-3). Cross-
  examine: with `record-boundary` now UPSERTing the *ref* row directly, is there
  truly no residual divergence path (e.g. a re-record racing a re-conclude on the
  same phase, both CAS-ing the ref)? Does `land_boundary_row`'s UPSERT preserve
  the strict `plan_phases` row order (`src/dispatch.rs` ~2709-2728) — an in-place
  replace keeps position, an append moves it; confirm which the helper does and
  whether either perturbs ancestry.
- **Layering / ADR-001.** The relocation moves `commit_on_behalf`,
  `commit_tree_as`, `Provenance`, `Identity`, `dispatch_identity`,
  `DISPATCH_NAME`/`DISPATCH_EMAIL` from `src/mcp_server/dispatch.rs` DOWN to
  `src/dispatch.rs`. Confirm the down-move creates no `dispatch → mcp_server`
  cycle and that `mcp_server` only references them `crate::dispatch::…` (the
  existing one-way edge). Is the move complete — does any residual symbol stay
  behind and re-introduce the reverse edge?
- **`commit_on_behalf` generalisation (OQ-3).** The CAS target changes from
  HEAD-derived to an explicit `target_ref` arg. `dispatch_import` (also HEAD-based
  today) must pass the coord ref explicitly. Probe: does import stay behaviour-
  identical, or does the signature change silently alter which ref it CAS-es?
- **Behaviour-preservation gate (R1, VT-4/VT-5).** The design claims "relocate +
  delegate, `mcp_server` + `e2e_dispatch_sync` suites green *unchanged*". Is that
  actually phaseable — does the `commit_on_behalf` signature change (new
  `target_ref` param) force edits to its existing unit tests, breaking the
  "unchanged" claim? A contract change that forces test edits is not
  behaviour-preserving as written.
- **OQ-1 escape-hatch identity.** §6 defers commit-identity for the record-boundary
  ref write to `/plan` (reuse `dispatch_identity()` vs an attributable `Manual`).
  Is deferring safe, or does an un-attributable correction taint the audit trail?
- **OQ-2 dangling readers.** Design deletes `ledger::read_boundaries_file` /
  `ledger::record_boundary` and eyes `ledger::read_boundaries`. Probe for any
  surviving caller of the working-tree boundaries path the design missed —
  a hidden consumer makes the deletion a compile break or a silent behaviour loss.
- **E1 refusal.** §5.5 (E1) claims record-boundary with no `dispatch/<slice>` ref
  → clean refusal. Confirm the failure mode is actually clean (not a panic or a
  half-written ref).
- **Non-goal integrity.** Does D-B1 quietly drag in the `commit_journal` twin
  rewire (explicitly a non-goal / follow-up), or stay confined to boundary-write?

## Synthesis

Second inquisition of SL-221, tried against the standing **D-B1 seam-collapse**
design (RV-278 having burned the superseded D1 *merge*). The seam collapse is
**doctrinally the right pivot** — it does dissolve RV-278's F-2 tie-break (no
competing working-tree copy survives the single writer) and removes real code.
But the design.md that carries it still confesses three heresies, all
`design-wrong`, all verified against the code:

- **F-3 (blocker) — the ordering hazard is asserted dead, not proven dead.**
  §5.3 proclaims "F-3's ordering hazard cannot arise", yet `plan_phases`
  (`src/dispatch.rs:2709-2728`) still chains phase refs strictly in
  `boundaries.rows` order, and `land_boundary_row` still `push`es an absent
  phase at the tail. The escape hatch exists precisely to "bootstrap a
  pre-binding phase" (`dispatch/SKILL.md:97`), so an out-of-order record
  mischains the branch ancestry. Killing the *merge* removed the interleave
  source; it did **not** establish phase-order == row-order. The design must
  either cite a monotonic-by-phase invariant every writer upholds, or normalise
  by phase before `plan_phases` consumes the ledger — and VT-3 must exercise the
  out-of-order case.

- **F-4 (major) — the behaviour-preservation proof is false as written.** R1 /
  VT-4 / VT-5 promise the `mcp_server` + `e2e_dispatch_sync` suites stay green
  *unchanged*. They cannot: `commit_on_behalf`'s unit tests call the 5-arg shape
  directly (`src/mcp_server/dispatch.rs:980…1160`), so the new `target_ref`
  param edits every call-site; and `tests/e2e_dispatch_sync.rs:1389-1415` pins
  the working-tree `record-boundary` write that §5.2(d) deliberately retires. The
  gate is about *invariants* preserved, not literal test bytes — the design must
  say so and own the mechanical churn + the record-boundary e2e rewrite.

- **F-5 (major) — the down-move is not closed over its dependencies.**
  `land_boundary_row` (sited in `dispatch.rs`) calls `funnel_message`, which
  lives in `src/mcp_server/dispatch.rs:42` and is absent from the §5.2(a)
  relocation set. As written the helper calls *up* into `mcp_server` (an ADR-001
  cycle) or duplicates the message. The relocation list must be made closed over
  every symbol the helper transitively needs (fold into the OQ-2 audit).

**Verdict.** The direction is sound; the artifact is not yet truthful. D-B1 is
**not fit to pass to `/plan`** until §5.3's ordering claim is proven or
normalised (F-3), the verification narrative is rewritten in plain truth (F-4),
and the relocation set is closed over `funnel_message` et al. (F-5). No merge
heresy survives — but the seam collapse has not yet escaped the wheel.

**HERESIS URITOR; DOCTRINA MANET**
