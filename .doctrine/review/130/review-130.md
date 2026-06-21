# Review RV-130 — design of SL-133

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Subject.** SL-133 design (`design.md`) — multi-dimensional priority scoring for
`survey`/`next`: two-pass scoring model (base pre-pass + consequence post-pass),
risk-leaf extraction (`src/risk.rs`), `[priority]` config, sort contract, ADR-015.
A prior internal adversarial pass (§10) already fixed F1 (dep edge direction),
F2 (ref label subset), F3 (BaseScore split). This inquisition presses *external*,
independent scrutiny — assume the §10 self-clearance is the accused's own alibi.

**Lines of interrogation.**

1. **Edge-direction correctness (the slice's whole reason to exist).** §5.4 claims
   dep-class dependents = `out_edges(dep_overlay, B)` (the `needs` B→A flip) and
   ref-class = `in_edges` over the `CONSEQUENCE_LABELS` subset. Verify against
   cordage (`in_edges`/`out_edges` at the cited lib.rs lines) and the *actual* edge
   emission direction in `graph.rs`. A direction error here silently inverts every
   ranking — confess or be cleared.

2. **ADR-001 layering.** Risk types extracted to a leaf `src/risk.rs`; new
   `priority::config`. Does the design require the `layering.toml` tier assignment
   (the binding tier, not just prose)? RV-121 (SL-132) was caught omitting exactly
   this. Where does `priority::config` (engine) sit, and does `base_score` (engine)
   reading `EntityFacets` (leaf) + `PriorityConfig` create any upward edge?

3. **Pure/impure split.** §5.1 says only config load + scan touch disk. Confirm
   `base_score` and the post-pass are pure; confirm config `load` is the only new
   impure entry and that D4 (load at the `build` seam, not `main.rs`) does not
   smuggle disk into a surface fn.

4. **Determinism / NaN-freedom (I1/I2/I6).** `f64` scores rank mint + survey.
   Stress every path to a comparator: clamp of non-finite/negative coeffs, the
   `estimate_midpoint` ε-guard, `total_cmp` everywhere. Is the clamp claim actually
   sufficient, or can an authored `[priority]` value still reach a comparator NaN?

5. **Scope fidelity.** D4 deviates from the scope's "main.rs parses config." Is the
   deviation declared *and* justified, or smuggled? Cross-check `slice-133.md`.

6. **Behaviour-preservation gate.** Risk-type extraction + scan read must keep
   backlog/scan suites green unchanged. Does the design name the proof and the
   re-export path? Any silent behaviour change in `exposure`/`validate_facet`?

7. **Internal contradiction.** OQ-2 (store vs derive consequence) is left open yet
   §5.2 commits to `score − base` derivation — is `explain`'s contract under-specified
   while a field it emits is "leaning derive"? Does `ReasonKind::Score` carry a
   `consequence` it may not store? Any place the design says two things.

8. **No parallel implementation.** Risk extraction must be a verbatim move + re-export
   (D2), not a re-parse; the scan risk read must ride `read_facets`, not a second
   validator. Confirm no second facet/risk parser is introduced.

## Synthesis

**Tribunal.** External adversarial review (codex / GPT-5.5, read-only) over `design.md`
against the curated domain_map and the eight lines of interrogation; every charge
re-verified against source before raising (the ledger is append-only). Eight findings:
**1 blocker, 3 major, 4 minor.**

**Verdict — the design is NOT fit to proceed to `/plan` unreconciled.** The spine is
sound — the edge-direction question that is the slice's whole reason to exist (F1/F2 of
§10) **survives external scrutiny clean**: cordage `out_edges`→dst / `in_edges`→src
(`crates/cordage/src/lib.rs:768,783`) confirm the dep-class `out_edges(dep_overlay)`
flip and the ref-class `in_edges` over `CONSEQUENCE_LABELS`. No layering *direction*
violation, no ADR-004 breach, no parallel risk validator. But the design ships a
**false binding-gate story, a false soundness invariant, an unclosed numerically-bogus
OQ, and an incomplete interface enumeration** — each a heresy that would burn on
contact with `just gate`, a fuzz input, or the compiler.

**Ordered penance (reconcile into `design.md` before `/plan`):**

1. **F-1 (blocker) — make the binding tier-map change part of the slice.** Add to
   `.doctrine/adr/001/layering.toml`: `risk = "leaf"`, a classification for
   `priority::config`, and update the `facet` entry/comment (currently "imports only
   estimate + value") to permit the risk import. Cite ADR-001 as the forcing function,
   exactly as D2 already cites it for the *extraction*. Verify: `just gate` green.
2. **F-2 (major) — make I2 true by construction.** Bound authored coefficients to a
   finite max at load and/or `is_finite`-sanitize every computed dim+total before
   storage/comparison; add a near-`f64::MAX` VT (folds into VT-6/VT-3).
3. **F-3 (major) — close OQ-2 by STORING consequence.** Capture the post-pass Σ as its
   own `BTreeMap<EntityKey,f64>` (the value exists pre-summation, exact); drop the
   `score − base` "exact" claim. This also retires the F-2 `inf−inf` NaN path.
4. **F-4 (major) — enumerate every `build_from` caller.** Thread `&PriorityConfig`
   through the pre-scanned `actionability_block_from` path (`surface.rs:484`) or give it
   a dedicated wrapper; name it in §5.2/D4.
5. **F-5 (minor) — honest impurity boundary.** §5.1 must count `dep_seq_for` (graph.rs:221)
   as an existing impure read, reconciling §5.1 with D4.
6. **F-6 (minor) — own the clamp policy.** State `PriorityConfig::load`'s fatal/default/
   clamp granularity as a deliberate new policy; stop laundering it through the
   `dispatch_config` precedent (which hard-errors malformed values). Folds OQ-1.
7. **F-7 (minor) — pin the scan seam proof.** Add a VT asserting the new `[facet]` read
   preserves per-facet isolation (malformed risk drops only risk; estimate/value intact).
8. **F-8 (minor) — self-consistent render contract.** Reconcile D7 with the view types:
   `next` surfaces score via its `ReasonKind::Score` reason line, not a row column (or add
   `score` to `NextRow` + `NEXT_COLS`).

**Standing risks / consciously held.** None tolerated. The clean spine (edge directions,
layering direction, ADR-004) is recorded so reconciliation need not re-litigate it.

**Disposition route.** All eight are **design-wrong** (no code exists; the artifact is the
defect) — resolved by reconciling `design.md` (+ the `layering.toml` requirement), not by
code. Findings left **raised / await=responder**: the blocker (F-1) correctly gates the
slice's eventual close until reconciliation lands. Reconcile, then `verify` each.

> **HERESIS URITOR; DOCTRINA MANET**
