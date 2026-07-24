# Review RV-296 — reconciliation of SL-224

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation audit). **Self/auto** — one hand drives
both roles via `--as`.

**Reviewed surfaces.** This is a **dispatched** slice; the coord tree was removed.
- *Algebra* (conformance, selectors, criteria) — read from the **primary edge
  tree**, with the **canonical selector registry taken from `dispatch/224` tip**
  (`444de78`), not the stale edge copy. The edge `slice-224.toml` lags by design
  until stage-2 integrate, so edge `slice conformance` over-reports.
- *Code* (build / suite / gate) — the **`review/224` impl-bundle** evidence ref
  (tip `e414a7ef28a6`), verified independently in a detached worktree
  (`.worktrees/audit-224`), seeding `web/map/dist` first (RustEmbed derived asset
  absent from a fresh checkout — [[mem_019f4c64e65574238b7026f7301c8a2c]]).

**Invariants held.**
- Faithfulness of impl to design §5.1–§5.5 (both objectives ride the pre-existing
  `conformance::undeclared_paths`; one shared `undeclared_detail` formatter; no
  re-inlined `selector add` hint — the integrated F1/F2 inquisition fixes).
- Behaviour-preservation (AGENTS.md): `check_vt_shape` and the `classify_import`
  pure verdict stay byte-for-byte green — **confirmed IDENTICAL** vs `main`.
- Pure/imperative split (ADR-001): new formatter + coverage predicate pure; the
  selector read, stdout, JSON-RPC impurity stays in the shell.
- Closure intent (§9): obj-1 detail golden + obj-2 lint tests green + memory-blind
  refusal demonstrable (detail names the path; `check plan` flags the plan).

**Evidence (code leg, `review/224`).** `cargo build` clean; `cargo clippy
--workspace` zero warnings; full suite **4566 passed / 0 failed** (the "3778" worker
figure is the lib-unit subset). 23 `undeclared*` tests green; `check_plan` ×2 green;
the EX-4 wiring flip asserts a **non-empty** `detail` that names the path *and*
carries a runnable `doctrine slice selector add SL-199 docs/readme.md …` remediation
(not the vacuous keyword-only test §9/F9 warned against).

**Lines of attack (the five design-vs-impl gaps this audit dispositions).**
1. CLI slice plumbing: design §5.2/F4 said `u32` ("no None hazard"); impl threads
   `Option<u32>` with an honest id-less `None` fallback.
2. MCP arm: design sketched `match Err(Refusal::UndeclaredScope) =>`; impl routes
   through a `Refusal::scope_detail(..)` engine value-method.
3. Mid-dispatch scope widen: `src/worktree/mod.rs` added as a design-target
   selector during dispatch — the exact gap PHASE-02's lint targets; §5.5/A2's "VT
   `test_file` is the only structured touch-target" understated the CLI-threading
   need.
4. PHASE-02 `plan_check_report` pure seam extraction — discretionary ADR-001
   refinement enabling exit-path testability.
5. `verify-vt` ×4 UNATTRIBUTABLE from edge — coord-topology attribution
   false-negative (IMP-228); keywords present + suite green on `review/224`.

Residual conformance cell after the canonical read: only
`.doctrine/slice/224/slice-224.toml` (the slice's own authored registry — never a
code selector; a pure algebra artifact, no action). `src/worktree/mod.rs` is
**conformant** on the canonical tier.

## Synthesis

**Closure story.** SL-224 delivers both objectives faithfully. Objective 1 gives the
MCP `dispatch_import` undeclared-scope refusal a non-empty, runnable `Refused.detail`
that names the offending path(s) and emits a runnable `doctrine slice selector add
<ID> <path> --intent design-target` remediation, via a new pure leaf formatter
`conformance::undeclared_detail(slice, undeclared)` shared with the CLI import arm.
Objective 2 adds `plan::undeclared_test_files` + a `check plan` coverage leg that
flags, at plan time, any VT `test_file` no design-target selector covers — catching
the under-declaration class *before* dispatch. Both ride the pre-existing
`conformance::undeclared_paths` predicate (no new classifier logic), and both route
their remediation through the single `undeclared_detail` formatter (the integrated
F1/F2 inquisition fixes — no re-inlined `selector add` hint, id present so the
command is runnable). The load-bearing wiring flip (EX-4) asserts a non-empty detail
that names the path *and* carries a runnable remediation — not the vacuous
keyword-only test §9/F9 warned against. Closure intent met: obj-1 golden green,
obj-2 lint tests green, and the memory-blind refusal is demonstrable.

**Verification.** Independent re-run on the `review/224` impl bundle (detached
worktree): `cargo build` clean, `cargo clippy --workspace` zero warnings, full suite
**4566 passed / 0 failed**; 23 `undeclared*` tests green; `check_plan` ×2 green. The
two explicit behaviour-preservation gates — `check_vt_shape` and the `classify_import`
pure verdict — are **byte-for-byte identical** to `main`. Pure/imperative split held:
the new formatter and coverage predicate are pure; the selector read, stdout, and
JSON-RPC impurity stay in the shell.

**Design-vs-impl gaps — all confirmed and benign.** Five gaps surfaced (F-1..F-5).
Three are correct refinements the design prose now trails (Option<u32> threading with
an honest None fallback; the `Refusal::scope_detail` engine method over an inline
shell match; the mid-dispatch `mod.rs` design-target widen — itself a dogfood of
PHASE-02's lint). One is a discretionary ADR-001 seam extraction (`plan_check_report`).
One is a coord-topology reporting artifact (edge `verify-vt` false-negative, IMP-228).
None is a code defect; none blocks.

**Standing risks / tradeoffs consciously accepted.**
- The canonical selector registry (incl. `mod.rs`) lives on `dispatch/224`; the edge
  copy lags until stage-2 integrate, so **edge `slice conformance` and `verify-vt`
  will read dirty/false-negative until the code+authored state land**. This is by
  design (ADR-012 topology) — /close must not mistake it for a delivery gap. The fix
  is integration, not a pre-integrate edit (which would fork `slice-224.toml`).
- No blocker raised; the close-gate is clear.

## Reconciliation Brief

Two design-prose edits and two integration guardrails. No `plan.toml` criterion
changes (immutable-append; none needed). No governance/spec REV — this slice touches
no ADR/policy/standard/spec.

### Per-slice (direct edit)

- **design.md §5.2 / F4 (F-1, F-2)** — update the CLI slice-id prose: the chain
  threads `Option<u32>` (not the "unwrapped `u32`, no None hazard" text), with an
  honest id-less `None` fallback in `undeclared_scope_report` for the
  type-reachable/practically-unreachable no-`--slice` path. Also mark the §5.2 **MCP
  code block illustrative** and note the impl routes the detail through a
  `Refusal::scope_detail(slice, selectors, delta_paths)` engine value-method (avoids
  un-gating the `#[cfg(test)]` `Refusal` re-export in prod; layering engine→leaf
  preserved).
- **design.md §5.5 / A2 (F-3, optional mirror)** — note that the CLI-threading touch-set
  includes `src/worktree/mod.rs` (sole non-test caller of `run_import`), so the "VT
  `test_file` is the only structured touch-target" statement understated the obj-1
  touch surface. Prose mirror only — the selector itself is already recorded (below).

### Selector registry (already recorded on canonical tier — integration-delivered)

- **`src/worktree/mod.rs` design-target selector (F-3)** — ALREADY present on
  `dispatch/224` `slice-224.toml` (with an EX-2 rationale note). **No `slice selector`
  verb to run at reconcile**, and **NO edit on the primary/edge** (would fork the
  file and conflict at landing). Stage-2 integrate carries it; edge `slice
  conformance` goes clean for `mod.rs` post-landing. Recorded here so /reconcile and
  /close know the residual edge over-report is a staleness artifact, not scope creep.

### Close guardrails (no write — advisory to /close)

- Edge `verify-vt SL-224` FAILs ×4 (F-5) and edge `slice conformance` `mod.rs`
  undeclared (F-3) are **pre-integration coord-topology artifacts**. The VTs pass and
  the selector is declared on the delivered surface (`review/224` / `dispatch/224`).
  Do not treat either as a delivery gap. Root cause of the verify-vt case: IMP-228.
