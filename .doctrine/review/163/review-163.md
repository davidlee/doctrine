# Review RV-163 — reconciliation of SL-154

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** SL-154 *Reliable conformance-registry capture* — closes the two
population leaks RFC-004 v0.1 (SL-147) left in `.doctrine/state/slice/NNN/boundaries.toml`
(the conformance registry): ISS-051 (solo final-phase miss), ISS-052 (funnel never
populates), absorbing ISS-039 (dispatch ledger never committed). Six phases, all
`completed`, landed solo on `edge`. Claude-arm-bounded; codex/pi symmetry stays IMP-171.

**Surface reviewed.** In-tree authored source on `edge` (this is a **solo** slice,
not dispatched — no candidate interaction branch; R2 evidence-ref caveat N/A).
Reviewed at code tip `b625bd82` (PHASE-06 feat).

**Lines of attack / invariants held:**
- **Conformance algebra** — every `design-target` selector delivered (undelivered=0);
  every undeclared path explained, not scope creep.
- **Design↔code fidelity** — the locked Rev-6 committed-ref model (D1–D12) matches the
  shipped code at each load-bearing seam: sticky merge keyed on *incoming* provenance
  (D12), D11 projection-source guard set-membership (Funnel∪Unknown), commit-then-
  derive-then-gate ordering before projection (F1), reopen eviction (D8).
- **Plan EX/VT/VA** — each phase's authored criteria satisfied by green tests; the
  PHASE-06 EX-2/VA-1 amendment (Decision A) audited against the *amended* text.
- **Behaviour-preservation** — `set_phase_status` solo path + `worktree_for_ref`
  callers unchanged; `just check` green.
- **Spec/governance** — D10 asserts NO SPEC-022 REV (committing the ledger is
  *conformance*, not a spec change); confirm no governance drift slipped in.
- **Standing risks** — the runtime registry's all-Unknown provenance (stale PATH
  binary footgun); design-prose↔code departure on `forget` error handling; the three
  motivating issues' backlog status; deferred follow-ups (IMP-171, IMP-173/F4).

**Caveats baked in (handover):** use `./target/debug/doctrine` (PATH binary predates
`provenance`, strips it on RMW); all-`unknown` registry provenance is **expected**, not
a defect (solo conformance keys on oids); concurrent SL-155/156 agents — path-limit
every commit.

## Synthesis

**Closure story.** SL-154 lands clean. The conformance algebra is the strongest
single signal and it is green: `undelivered=0` (every `design-target` selector
delivered), `conformant=9` (all six source files + three dispatch `SKILL.md`), and the
five `undeclared` paths are fully accounted for — three test files (VT evidence, no
`tests/` selector by convention) and two authored `.doctrine/` files folded because
conformance does not strip `.doctrine/` (F-2, `aligned`). No undeclared *source* path:
no scope creep, no silent touch.

The Rev-6 committed-ref design (D1–D12, hardened across eight inline codex passes — the
slice carries no separate design RV) is faithfully implemented at every load-bearing
seam, verified directly against the tree, not just the notes: the provenance sticky
merge keyed on the **incoming** row (`state.rs:698/701` — `Solo`/`Funnel` overwrite,
`Manual`/`Unknown` preserve, atomic inside the existing RMW, D12); the D11 projection-
source guard as a pure phase-id set-membership over `{Funnel, Unknown}` excluding
`Solo`/`Manual` (`dispatch.rs:1514/1520`); the funnel `Funnel` stamp on the double-write
(`dispatch.rs:608`); `record-delta`'s `Manual` incoming (`slice.rs:1992`); the
`#[serde(default)] Unknown` back-compat story (`boundary.rs:48`). The guard→derive→gate
ordering sits before ref projection (F1: a halt creates no refs). Every phase's authored
EX/VT/VA is satisfied by green tests — `just check` exit 0, and the three phase-specific
e2e suites pass unchanged (`e2e_dispatch_sync` 38, `e2e_slice_record_delta` 6,
`e2e_skills_dispatch_shrinkage` 3). The PHASE-06 EX-2/VA-1 amendment (Decision A) was
audited against the **amended** text and is internally coherent with design D6/§5.1
(codex/pi `record-delta` is a retained, gate-enforced funnel write, not removed).
Behaviour-preservation held: the named shared seams (`set_phase_status` solo path,
`worktree_for_ref` callers) are unchanged.

**Standing risks (consciously accepted).**
- **Runtime registry all-Unknown provenance (F-3, `tolerated`).** A jail-environment
  footgun, not a code defect: the readonly PATH binary predates the `provenance` field
  and strips it on any read-modify-write. Harmless here because solo conformance keys on
  oids; provenance gates only the dispatch D11 guard, never reached on a solo slice. Left
  surfaced rather than cosmetically masked. Durable memory captured (`mem_019f025e`).
- **F4 — liveness ≠ ownership (design D9, deferred).** A coord worktree left un-pruned
  through the pre-integrate audit window can false-stand-down a post-drive solo phase;
  caught loudly by the gate/conformance, but the precise dispatch-run ownership signal is
  hardening work — filed as **IMP-173**.
- **codex/pi symmetric derive deferred (D6) — IMP-171.** This slice is claude-arm-bounded
  by design; the codex/pi arm's registry write stays `record-delta` (now gate-enforced).

**Tradeoffs.** D10's no-SPEC-022-REV stance holds under audit: committing the dispatch
boundaries ledger is *conformance* to SPEC-022 §run-ledger-sourcing (which already
mandates the committed tip), not a spec change — so the only governance touch this slice
implies is zero. The single design-prose drift (F-1) is a stale `?` in the §5.2 reopen
pseudocode; the shipped degrade-and-warn is the *correct* behaviour (propagating would
break D5 on non-git roots) — prose follows code at reconcile, not the reverse.

**No blocker raised; close-gate clear.** Findings: F-2 `aligned`, F-3 `tolerated`,
F-1/F-4 `verified` → reconciliation brief.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §5.2 (line 388) [F-1]:** the reopen-eviction pseudocode shows
  `state::forget_source_delta(&primary, slice, phase)?;` — the propagating `?` is wrong
  for the shipped contract. Replace with the **degrade-and-warn** form: forget warns like
  the record tail and never propagates, because `?` would make a reopen fail on a
  non-repo/bare cwd and break D5 (the binding must never block a status transition). Note
  the self-healing rationale (re-completion's upsert overwrites a lingering row; a
  never-recompleted reopen surfaces via the completeness gate). No code change — code is
  correct; align the prose. Cross-ref `mem.pattern.state.reopen-evict-degrades-self-heal`.

### Backlog status reconciliation (sibling-entity direct edit)
- **ISS-051 [F-4]:** fixed by PHASE-03 (live-coord guard + `Solo` stamp + absent-stamp
  records-nothing). Resolve against SL-154 closure.
- **ISS-052 [F-4]:** fixed by PHASE-05 (derive + D11 guard + completeness gate). Resolve.
- **ISS-039 [F-4]:** absorbed + fixed by PHASE-04 (`commit_boundaries` splice at
  prepare-review). Resolve.

### Governance/spec (REV)
- **None.** D10 stands under audit: committing the boundaries ledger is SPEC-022
  conformance, not a spec change. No ADR/spec/policy edit is implied by this slice.

### Deferred follow-ups (already filed — no new work, confirm only)
- **IMP-171** — codex/pi symmetric ledger+registry derive (D6, claude-arm-bounded scope).
- **IMP-173** — dispatch-run ownership signal for solo stand-down (F4 hardening, D9).
