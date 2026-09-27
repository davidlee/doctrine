`boot::wire()` writes to `io::stdout()` directly and takes no writer or output-capture parameter (SL-271 pre-implementation recon). There is no in-process stdout-capture helper in the repo.

Consequence: when a change alters an install/report *message* (not just a side effect), do NOT promise a unit test — the message is only observable through the real binary. Assert it in an integration test that spawns the built binary via `tests/common`'s `doctrine_cmd(dir)` and reads `out.stdout` (see `tests/e2e_claude_install.rs`).

SL-271 PHASE-01 is the worked example: the payload change (full invocation, no re-appended `serve --mcp`) is unit-tested at the planner (`plan_mcp_carries_the_full_invocation`), while the printed form moved to the e2e.

Also: `doctrine slice verify-vt` attributes a VT to the slice only after the phase's source-delta row exists — i.e. after the phase is flipped `completed`. Before that every VT reads UNATTRIBUTABLE even with a green tree.