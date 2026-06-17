# Review RV-064 — reconciliation of SL-088

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Audit surface.** Reviewed the `review/088` branch (impl-bundle candidate) diffed
against `main` — the published candidate interaction surface per the dispatch
funnel. The phase worker branches (`worker/SL-088/PHASE-03/04/05`) and the
coordination branch (`dispatch/088`) are immutable evidence refs (R2), not the
audit surface.

**Lines of attack.**

1. **Agent-def generalization (PHASE-03).** Does `install_agent_def()` correctly
   handle both Claude (flat canonical, `None` subdir) and pi (namespaced,
   `Some("pi")`)? Does it reuse `classify_link`/`write_link`/`relative_target`
   without parallel symlink code?

2. **Pi install wiring (PHASE-03 in install.rs).** Does the non-claude agent
   loop call `install_agents_for` with the right arguments (`agent`,
   `Some(agent)`)? Is failure non-fatal?

3. **Prose consolidation (PHASE-04).** Are all five `claude install` references
   in README.md correctly migrated to the consolidated `doctrine install --agent
   claude` surface? Is the `skills install` alias mention (L85) removed?

4. **E2E migration (PHASE-04).** Do the e2e tests drive `doctrine install
   --agent claude --yes` (not `claude install` or `skills install`)? Are the same
   assertions preserved? Is the `skills_install_alias` test removed?

5. **Plan conformance.** Do the phase exit criteria (EX-1..EX-4) and
   verification criteria (VT-1..VT-4, VA-1..VA-3) for PHASE-03/04/05 hold?

6. **Gate (PHASE-05).** Is `.doctrine/agents/claude/` removed? Is `just gate`
   zero-warning? Are the 2 e2e_relation_migration_storage failures pre-existing
   (confirmed: they fail because review/088 split from b109b3ae, which predates
   the migration test content on main).

## Synthesis

SL-088 consolidates five install commands into a single `doctrine install` with
per-agent opt-in. This audit inspected the PHASE-03/04/05 delta (PHASE-01/02
were pre-existing on main) against `design.md`, `plan.toml`, and the runtime
phase sheets.

**Closure story.** Two findings, both terminal. F-1 (minor) flagged a missing pi
dry-run integration test required by PHASE-04 VT-1 and EX-4; the test was added
(`install_agent_pi_dry_run_prints_delegation_plan`) and committed to `review/088`.
F-2 (nit) noted README L153 adds `--yes` beyond the published design text;
accepted as a reasonable editorial improvement for the pip-install context. The
audit uncovered no blockers, no design-level defects, and no unreconciled drift.

**Invariant checks — all green.**

- `install_agent_def()` handles both Claude (`None` subdir, flat canonical) and
  pi (`Some("pi")`, namespaced canonical). Reuses `classify_link`/
  `write_link`/`relative_target` — no parallel symlink code.
- Pi agent-def install is wired into the non-claude agent loop in
  `run_forward_steps()` with `install_agents_for(root, agent, Some(agent), ...)`.
  Failure is non-fatal (continues to next step).
- All five README.md `claude install` references consolidated. The `skills
  install` alias mention (L85) removed.
- E2E tests drive `doctrine install --agent claude --yes`. The
  `skills_install_alias` test is removed. Same assertions preserved.
- `just gate` zero warnings (confirmed by PHASE-05 commit).
  `.doctrine/agents/claude/` removed from git tracking.
- The 2 `e2e_relation_migration_storage` failures are pre-existing (the
  review/088 branch split from b109b3ae, predating migration test content on
  main). Not SL-088's concern.

**Standing risks.** The wildcard fallback (`_ → pi`) in both `install_agents_for`
(embed selection) and `install_agent_def` (link_dir selection) correctly routes
any non-Claude agent to pi paths. This works for the current two-agent world but
will need explicit dispatch when a third agent type is added — a natural
extension point, not a defect.

**Tradeoffs consciously accepted.** The README L153 `--yes` addition (F-2)
deviates from design §3's exact reference text but improves the user experience
in the pip-install context where interactive prompts are unwanted. The design
text was a reference, not a contract-level specification.

## Reconciliation Brief

No spec or governance edits needed. Both audit findings are terminal:

- **F-1** (minor, `fix-now`): Pi dry-run integration test added to
  `tests/e2e_claude_install.rs` — committed to `review/088`.
- **F-2** (nit, `tolerated`): README L153 `--yes` editorial deviation accepted
  as a user-experience improvement.

No per-slice design edits, no REV-governed spec changes.

## Reconciliation Outcome

All findings were withdrawn or tolerated with rationale. No writes needed.
Reconcile pass complete — handoff to /close.
