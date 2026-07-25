# ISS-243: dispatch_verify over MCP corrupts the JSON-RPC stream: run_suite inherits stdout

## Symptom

Calling the `dispatch_verify` MCP tool writes the verify suite's raw stdout/stderr
into the MCP server's JSON-RPC channel, corrupting the protocol stream for the rest
of the session. Every real suite prints (cargo, just, make), so this fires on the
happy path, not an edge case.

## Mechanism

- `src/mcp_server/mod.rs:44-45` — the server's transport is stdin/stdout; JSON-RPC
  frames are written to `io::stdout()`.
- `src/verify.rs` `run_suite` spawns with `Command::…​.status()`, which **inherits**
  the parent's stdio. Its doc-comment says so explicitly: "INHERITING stdio (live
  stream, not piped)".
- SL-228 PHASE-05 made `run_suite` reachable from the MCP surface for the first
  time, via `dispatch_verify`. The child therefore writes straight onto the
  server's protocol fd.

**The CLI arm (`doctrine dispatch verify`) is unaffected** — inherited stdio is
correct and desirable there (a dev gate should stream).

## Why it was built this way

Not worker error. SL-228 design §5 step 3 pins the runner as "stdio
inherited/streamed, no exit", and the same design mandates an MCP mirror of the
verb. The two requirements conflict; the conflict was not noticed at design or at
phase-plan, and PHASE-05's task pinned `run_suite` as specified. Surfaced by the
PHASE-05 worker in its hand-back and confirmed by the orchestrator against both
source seams.

Every other MCP path already avoids this: `worker_commit`'s `run_commit_gate` uses
`Command::output()` (piped), and `dispatch_conclude_phase` calls `set_phase_status`
directly rather than `run_phase` with the comment "so no CLI-shell `stdout` print
pollutes the MCP JSON-RPC channel".

## Fix sketch

Give `run_suite` an explicit stdio mode (inherited for the `check` command shell,
piped for engine callers) rather than a second runner — the parallel-implementation
trap. The piped arm's captured output is also what a `VerifyFailed` detail would
want to carry, so the two needs converge. Touches the existing `check` call site and
its tests, which is why it was not folded into PHASE-05 unreviewed.

## Interim workaround

Drive verify through the **CLI** arm, never the MCP tool:

```
doctrine dispatch verify --slice <N> --phase PHASE-NN
```

The engine beneath both surfaces is identical and fully covered; only the transport
differs.

## Related

- SL-228 PHASE-05 (the phase that introduced the reachable path)
- SL-228 design §5 step 3 — reconcile: the inherited-stdio pin and the MCP mirror
  cannot both stand as written
- ISS-219 — the other MCP payload-hygiene defect on the dispatch surface
