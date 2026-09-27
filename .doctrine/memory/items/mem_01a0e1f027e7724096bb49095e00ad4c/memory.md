`doctrine install --dry-run` prints the plan and returns BEFORE running any forward step, so it never reaches `boot::wire()` and produces no per-leg output (no MCP line, no hook activation notice, no probe).

To exercise `wire(..., dry_run=true)` end-to-end — the codex/hook "would write" wording, the MCP dry-run line — use `doctrine boot install --agent <a> --dry-run -y -p <dir>` instead. That path runs `run_install` → `wire(..., dry_run=true)`.

SL-271 PHASE-02/03 e2e (`tests/e2e_codex_install.rs`) hit this: the dry-run case must be `boot install --dry-run`. The install-level dry-run is a plan-only preview.