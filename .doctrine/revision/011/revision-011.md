# REV REV-011 — ADR-008 D-B1/D-B5: platform exits build-env

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

ADR-008 accepted a per-worktree build-isolation design in which the **platform**
injects a per-worktree `CARGO_TARGET_DIR` at worker spawn. SL-156 found that
mechanism (a) impossible on the claude arm — no Claude hook can inject env into a
spawned worker (`mem.fact.dispatch.claude-worker-no-per-worktree-env`, Probe 2),
and (b) a **POL-002 violation**: the shipped platform hardcodes a *cargo*
convention (`project_env_contract`/`coordinate`/`gc`). The fix inverts the
mechanism: isolation now comes from the **absence** of a shared
`CARGO_TARGET_DIR` (the flake export is retired), so every worktree defaults to
its own in-tree `target/` — correct by construction, both arms, no env channel.

The **intent** of D-B1 (per-worktree isolation, parallel builds, correct
`CARGO_BIN_EXE`) is **preserved and better served**. Only the mechanism changes.
SL-156 (`design.md` D1–D3) is the implementing slice; this revision is **proposed**
and is **applied at SL-156 reconcile** (the ADR edit is real only once the code
lands).

### Amendment 1 — D-B1 mechanism: platform-injects → flake-retires

| Where | Before | After |
|---|---|---|
| ADR-008 § Decision, D-B1 | Per-worktree `CARGO_TARGET_DIR`, nested under the jail-redirect root, keyed `wt/<branch>`, **set at worker spawn** | Per-worktree isolation by **retiring** the shared `CARGO_TARGET_DIR`; each worktree defaults to its own in-tree `<worktree>/target` — no platform env injection, no `wt/<branch>` keying |

The "where per-worktree target is unavailable — claude shares the jail-wide
target" caveat (ADR-008:55) is **resolved**, not mitigated: the claude arm now
isolates by construction.

### Amendment 2 — D-B5: flake-minimal → flake exits build-env

| Where | Before | After |
|---|---|---|
| ADR-008 § Decision, D-B5 | Keep the flake minimal; per-worktree env **set at worker spawn**, not baked; justfile unchanged | The flake loses the `CARGO_TARGET_DIR` export entirely — the **removal is the mechanism**; platform exits the build-env business (POL-002); justfile still unchanged |

### Unchanged

- **D-B2** (no in-jail `cargo install`) and **D-B3** (bwrap confinement) stand.
- **D-B4** (sccache) is **unchanged but gains relevance** — it is now the
  warm-fork-cache lever, since B1 accepts cold fork builds (the persisted warm
  cache the shared target gave is the deliberate trade; SL-156 §5.4, R1).

### Forcing function

**POL-002** (platform independence from host build conventions). The codex arm's
`$fork_env` stdout dance was the existing violation; extending it to the claude
arm would have deepened it. Retiring the coupling is the policy-conformant fix.
