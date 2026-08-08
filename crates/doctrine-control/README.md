# doctrine-control

The capsule control binary — the trusted side of *starting* a capsule
transaction: resolving its contract, provisioning fresh confined state, and
proving the backend's properties on the host it will run on.

It is a second binary in the `doctrine` workspace rather than a second verb on
the existing one. The split line is canonical mutation: `doctrine-control` never
writes authored state, and nothing migrates out of the agent-facing `doctrine`
binary to reach it. It depends on that binary's library target for a curated set
of five items and reaches the root package through nothing else.

**Not released.** `publish = false`, and the workspace's publish recipe is
package-limited. The binary is reachable from the build tree only.

Two verbs, neither present yet:

- `provision` — resolve a capsule contract and provision its transaction.
- `backend verify` — run the conformance suite for an on-host admission
  verdict, exiting nonzero with structured output when the backend is not
  admitted.

See `SL-248` and `ADR-020` for the design, and `SPEC-030` for the container
specification.
