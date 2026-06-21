# Dispatch coordination worktree omits gitignored build artifacts — funnel verify fails to compile until provisioned

`dispatch setup`'s coordination worktree is provisioned only via
`.worktreeinclude` (currently just `.doctrine/doctrine.just`). **Gitignored build
artifacts are NOT carried over.** Notably `web/map/dist/` (the `RustEmbed`
`#[folder = "web/map/dist/"]` for `map_server::assets::Assets`, built by `just
web-build`) is absent — so the coordination tree **fails to compile**:

```
error[E0599]: no function or associated item named `get` found for struct
`assets::Assets` ... candidate #1: `Embed`
```

The funnel's **verify** step runs in the coordination tree, so this blocks the
whole gate even though the phase delta is unrelated.

- **Worker forks differ:** `doctrine worktree fork --worker` DID include
  `web/map/dist/`, so the worker compiled fine — the asymmetry hides the gap
  until the orchestrator tries to verify.
- **Fix (in-flight):** copy the prebuilt artifact from the main worktree into the
  coordination tree before verify — `cp -r web/map/dist <coord>/web/map/` (it is
  gitignored, identical at the same commit, never part of the delta). Or run `just
  web-build` if node is available in the jail.
- **Durable fix candidate:** add gitignored build artifacts to coordination-tree
  provisioning, or have the gate build them. Mirrors the
  [[mem_019eacb95c397f63b8a4e4272a96e633]] gitignored-tier partition discussion.

Observed: SL-133 PHASE-02 dispatch (pi arm), base `8e2c9e12`.
