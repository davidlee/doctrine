# Review RV-272 — reconciliation of SL-218

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

SL-218 close-out audit (facet: reconciliation). Reviewed surface: the `edge`
working tree at commit 3844ad69 (not dispatched — inline execution). Lines of
attack:

1. **Conformance algebra** — does what git touched match design §Code-impact?
   (undeclared / undelivered leads.)
2. **VA-1 (the outstanding design VA)** — render wording vs product-critique #3
   (agent authority disclosed) and #6 (no cardinal-from-ordinal overclaim).
   External adversarial pass = codex (GPT-5.x), read-only.
3. **Intentional behaviour changes** — the off-frontier explain golden diff
   (EX-1/INV-1 rescope): intended, not regression?
4. **Coverage faithfulness** — the reachability caveat (needs/Dep tensions
   unreachable on the actionable frontier → unit goldens, not e2e).
5. **Surface completeness** — does the new `next --verbose` knob (D5) leave any
   spec/help-doc surface stale?

Invariants held: gate green (clippy zero-warning); existing suites green
unchanged knob-off (INV-1); one-truth-per-question (grades never disagree with
the elicit queue); wording single-sourced (REQ-072 AC3); no overclaim under T7.

## Synthesis

SL-218 lands the tension narrative (RFC-019 Phase D) clean: `doctrine check gate`
exit 0, 4145 tests pass, tree clean, all three phases `completed`. The audit
opened eight findings; none is a blocker, and every one is terminal.

**Closure story.** The mechanical conformance delta is small and fully explained.
Two source-tree signals: `src/status.rs` (F-1) is a three-line call-site ripple
of the designed `surface::next → NextView` return-type change — real, necessary,
undeclared only because the design named the producer (`surface.rs`) and not the
downstream consumer; it routes to reconcile as a selector-registry + design-mirror
edit. `src/comparison/query.rs` (F-2) is undelivered because the design retained
it as a conscious export fence that never needed to fire (F-5 held: no new
comparison API) — accepted, documented drift.

**VA-1 (the outstanding design VA) passes.** An external adversarial pass (codex,
GPT-5.x) pressed the render wording against critiques #3 and #6 and surfaced five
concerns; on adjudication all resolve in the design's favour. The core objection —
"ranks above" leads before the grade qualifies — is answered by D6's deliberate
three-grade vocabulary: the parenthetical grade clause is adjacent and directly
states the determinacy standing ("agent-proposed … unconfirmed" / "projected
order — no determining evidence"), while "ranks" names only the value_dim
computation. Provenance (critique #3) and projection disclaimer (critique #6) are
both present and golden-pinned (design §3, F-6). Two concerns had real teeth and
were run to ground in the code: the `AgentProposed` render surfaces the agent
share of the full-system counts (F-7) — deliberate and conservative, because the
human-only system is *indeterminate* for such a pair so the present human rows are
non-decisive and naming them would imply an endorsement the grade denies; and the
`Determined + (0,0)` anchor-only case renders "no constraining judgements" (F-8) —
accurate but self-undermining, a rare unfixtured wording edge captured as IMP-288.

**Standing risks / consciously accepted tradeoffs.**
- The needs/Dep structure-tension branch is unreachable on the actionable frontier
  (no non-terminal blocker holds between two actionable members), so it is pinned
  at the pure/render **unit** level rather than e2e (F-4). This is faithful
  coverage of the exact code path, not a gap — recorded durably in
  `mem.pattern.priority.tension-render-reachability`.
- The off-frontier `explain` now discloses "not on the current frontier — no
  tension analysis" (F-3), an intended INV-1 rescope; the one updated golden is
  documented at its test site.
- D6 prose ("counts come from the producing system") reads loosely against the
  deliberate agent-share render for `AgentProposed` — an optional, non-blocking
  prose tightening (F-7), carried to the brief for reconcile to take or leave.

## Reconciliation Brief

### Per-slice (direct edit)
- **F-1 — selector registry + design mirror (load-bearing: the registry).**
  Record the `src/status.rs` touch so conformance reads clean:
  `doctrine slice selector add SL-218 src/status.rs` (intent: call-site ripple of
  the `NextView` return-type change). Mirror it as a row in `design.md`
  §Code impact (`src/status.rs` — "adapt `next_up` caller to `NextView.rows`").
  The registry edit is the load-bearing change (it is what `slice conformance`
  reads); the §Code-impact row is its human mirror.
- **F-7 — optional D6 prose tightening (non-blocking).** In `design.md` D6,
  clarify that for `AgentProposed` the render surfaces the *agent share* of the
  producing (full) system's counts — the human rows are present but non-decisive
  (human-only system indeterminate). Take-or-leave; no behaviour change.

### Governance/spec (REV)
- None. No ADR, policy, standard, PRD, or tech-spec surface diverged. The
  `next --verbose` knob (D5) is clap-self-documenting with no spec enumeration to
  update (F-5).

### Not reconcile surface (recorded elsewhere)
- **F-8 → IMP-288** (backlog improvement): anchor-determined zero-judgement
  callout wording. Out of audit scope; fix sketch recorded on the item.
- **F-2, F-3, F-4** — aligned/tolerated, no write required.
