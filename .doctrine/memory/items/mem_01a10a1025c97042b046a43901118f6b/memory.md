`tests/mcp-bridge.test.ts` imports `vitest`, but the repo has no
`package.json` or vitest install. `bun test` runs it unchanged — bun maps the
`vitest` import onto its own runner. `just test-bridge` runs it against
`${DOCTRINE_BIN:-./target/debug/doctrine}` and is part of `just gate` (~1s).
Node and bun are present in the jail; "no vitest" is not a blocker.

Its integration half skips when the binary path does not exist, so run it
after a build (gate orders `build` first).
