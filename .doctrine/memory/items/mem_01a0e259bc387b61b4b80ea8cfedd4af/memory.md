# MCP review tools bypass the CLI worker guard

`commands/guard.rs` `worker_guard` refuses Write-classed verbs under
`DOCTRINE_WORKER`, but it runs only in `main.rs` CLI dispatch. The MCP server's
review tools (`src/mcp_server/tools.rs`, ~:682 onward) call `review::run_*`
directly, so for MCP callers the only admission check is the one inside
`review::turn::resolve_review_root`.

**How to apply:** any rule about where review verbs may run must live in
`resolve_review_root` (or below it), not only in `write_class`. Deleting the
review-level check "because worker_guard already covers it" opens the MCP path.
The same shape applies to any kind with both a CLI and an MCP surface: check
whether the MCP handler reaches the command-tier guard before relying on it.

Related: confined workers rw-bind their whole fork (`scripts/spawn-confined.sh`),
so a read-only `.doctrine/` is NOT a usable worker signal — `DOCTRINE_WORKER`
is the only one (DEC-338).
