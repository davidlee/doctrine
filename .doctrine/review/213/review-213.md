# Review RV-213 — reconciliation of SL-186

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance-mode audit of SL-186 (prompt cascade: per-context instruction
resolver), all four phases landed via pi/subprocess dispatch. Reviewed surface:
the admitted candidate **candidate/186/review-001** (22b47152), the impl bundle
projected onto `main` — not the raw evidence refs.

Lines of attack / invariants held:
- **Layering (ADR-001):** the PHASE-04 D-B relocation of the impure corpus loader
  into `src/install.rs` must not close an `install→commands` cycle. Held by
  `tests/architecture_layering.rs` (green).
- **Canon truth:** does `design.md` still describe where the code actually lives
  after D-B moved the loader and the def-refresh path?
- **Pure/impure split (INV-5):** engine stays disk/clock/env-free; loader is the
  only impurity. (Held — engine untouched by P04.)
- **Delivered ≠ attributed:** does the conformance registry's source-delta match
  the artifact actually projected onto trunk? (Cross-checked candidate vs main.)
- **VT integrity:** the mid-conclude P02 VT revision must follow the code without
  relaxing verification intent.

Evidence gathered: `slice conformance SL-186`; candidate build (green, 17.9s);
`cargo test` bin + architecture_layering + e2e_prompt_def_expansion +
e2e_prompt_resolve_golden (2940 + 17 + 3 + 5, **0 failed**); `doctrine check gate`
(exit 0); `git diff main..candidate/186/review-001` (19 files, all SL-186).

## Synthesis

SL-186 delivers the prompt-cascade resolver as designed: a pure engine
(`src/hymns.rs`), an impure corpus loader + seal set + embedded→disk projector,
the `doctrine prompt {resolve,model-keys,explain,check}` verbs, a seed corpus
under `install/hymns/**`, and the D7 def↔hymn install-time wiring (role-band-only
marker expansion in `dispatch-worker` defs). The delivered artifact is **correct
and green**: candidate builds, `check gate` passes (exit 0), the layering gate is
green, and every phase VT passes in the projected candidate.

The audit's substance is **canon drift, not code defect**. The one design-shaping
decision taken in-flight — PHASE-04 D-B, relocating the impure loader from
`src/commands/prompt.rs` into `src/install.rs` — was the right call (it dissolves
an `install→commands` layering back-edge that a def-expansion-time loader call
would otherwise create; ADR-001 gate confirms no cycle). But it left `design.md`
describing the *old* module map in two places: the loader location (§5.2 l.206,
§5.3 l.423-424) and the canonical-def refresh path (l.342, l.433, which names a
`src/skills.rs` that does not exist). Both are handed to `/reconcile` as per-slice
design edits (F-1, F-2). The consequential P02 VT criteria were already realigned
under `/consult` (F-5, aligned) so verify-vt tells the truth against the relocated
code.

Two lower-severity observations round it out. The conformance registry
over-attributes SL-186's source-delta (F-3, tolerated): pi-arm `record-delta`
ranges swept orchestrator-trailed knowledge commits and the refresh-base trunk
merge into the delta, so `undeclared` lists authored/knowledge/foreign files that
never reach the projected candidate — a known dispatch-arm characteristic, not
slice scope creep, verified by diffing the candidate against `main`. And the
design-target selectors under-declare the genuinely-designed surface (F-4):
command wiring, the two `dispatch-worker` defs, and the two e2e tests sit outside
the selectors, so conformance keeps flagging designed work — cheap to fix by
widening the selectors at reconcile.

Standing risks accepted: none block ship. The registry over-attribution (F-3) is
a dispatch-arm quirk captured as backlog work; it does not affect the delivered
code, which the candidate diff proves clean.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §5.2 (l.206) + §5.3 Ownership (l.423-424)** [F-1]: the impure corpus
  loader (embedded⊕disk walk, sidecar overlay, seal-twin drop) now lives in
  `src/install.rs`, not `src/commands/prompt.rs`. Update the prose + ownership
  table; note the D-B rationale (avoids `install→commands` layering back-edge;
  `prompt` verbs call `crate::install::load_full_corpus`).
- **design.md l.342 + l.433** [F-2]: the canonical-def refresh path
  (`install_agents_for`) and the D7 marker expander (`install_agent_def`,
  `expand_worker_marker`) live in `src/install.rs`. `src/skills.rs` does not
  exist — retarget both references.
- **SL-186 design-target selectors** (slice-186.toml `[[selector]]`) [F-4]: extend
  to cover `src/commands/{cli,guard,mod}.rs`, `install/agents/**/dispatch-worker.md`,
  and `tests/e2e_prompt_*.rs` so conformance stops flagging designed surface.

### Governance/spec (REV)
- None. No ADR/policy/spec/requirement change is implied; ADR-001 is upheld (gate
  green), and the D-B relocation is consistent with the accepted layering.
