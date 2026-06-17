# SL-088 — Durable notes

## Audit (RV-064, 2026-06-18)

**2 findings, both terminal.** Clean audit — no blockers, no design defects.

- **F-1 (minor, fix-now):** Missing `--agent pi --dry-run` integration test
  (PHASE-04 VT-1 / EX-4). Added `install_agent_pi_dry_run_prints_delegation_plan`
  to `tests/e2e_claude_install.rs`. The dry-run test avoids the npx dependency
  (unlike `--yes` which would invoke `npx skills` for pi).
- **F-2 (nit, tolerated):** README L153 adds `--yes` beyond design §3 reference
  text. Accepted — better UX in the pip-install context.

### Standing risks

- The wildcard fallback (`_ → pi`) in both `install_agents_for` (embed selection)
  and `install_agent_def` (link_dir selection) works for two agents but will need
  explicit dispatch when a third agent type is added. Not a defect — a natural
  extension point.

### Pre-existing test noise

- `e2e_relation_migration_storage` tests (2/6) fail on `review/088` because the
  branch split from `b109b3ae` which predates migration test content on main.
  Confirmed green on main. Not SL-088's concern.
