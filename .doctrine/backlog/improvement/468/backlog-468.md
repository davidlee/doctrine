# IMP-468: Plural agent-def install: project all of a harness's agent defs (capsule defs are published-only)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

`doctrine install --agent claude` projects **exactly one** agent def per harness.
`install_agents_for` (`src/install.rs:1783`) is a `match` to a single asset, not a
sweep of the harness's def directory:

```rust
let embed_asset = match agent_name {
    "claude" => DISPATCH_WORKER_AGENT_ASSET,      // agents/claude/dispatch-worker.md
    _        => DISPATCH_WORKER_AGENT_ASSET_PI,   // agents/pi/dispatch-worker.md
};
```

So the three capsule defs — `install/agents/claude/{capsule-orchestrator,
capsule-phase-planner,capsule-worker}.md` — are *embedded and published* but never
*installed*. They reached a live session in this repo only because they were
hand-copied into the gitignored `.claude/agents/` (byte-identical, untracked; the
one managed entry there is the `dispatch-worker.md` symlink).

## Why it matters

`/capsule-driver` ships as a skill (via `plugins/doctrine/skills/`, and the claude
skills leg), so an installing repo *appears* to have the capsule tier. It does not
have the agents the skill tells it to spawn, so every spawn falls back to
`general-purpose` — the catch-all with `tools: *`, inherited model, no role
contract. That is precisely the failure IMP-434 was raised to remove.

The asymmetry is invisible: the skill arrives, the agents do not, and no warning
is printed.

## Evidence

- Code: `src/install.rs:30-31` (the two asset constants), `:1783` (the single-asset
  match), `:471` (the claude leg's one call, `canon_subdir: None`).
- Empirical: fresh `git init` + `doctrine install --agent claude -y` leaves
  `.claude/agents/dispatch-worker.md` only (symlink → `../../.doctrine/agents/
  dispatch-worker.md`), and `.doctrine/agents/dispatch-worker.md` alone.
- Publication, not installation: `publication/manifest.toml:73-97` registers the
  three defs under `integration/agents/claude/…` to satisfy ADR-019's reachability
  gate (every embedded `install/` asset must be a base backing or published) — not
  to install them. The claude plugin ships `hooks/` + `skills/` only, no `agents/`.
- Design already settled: IMP-434 `Q8` (`backlog-434.md`) — *"The install path must
  go plural. `install_agents_for` … is not a directory sweep. Change it to
  harness → a slice of assets. The capsule defs are claude-only … so claude's list
  becomes four and pi's stays one."*
- Documented state: IMP-434 (now `resolved`) records the defs as *"published-only
  and must be copied to `.claude/agents/` by hand to go live"*, and lists the
  plural install among the work deferred to a slice. No slice or backlog item was
  ever carved for it — this item is that home.

## Proposed shape

1. `install_agents_for` harness → **slice of assets**; `claude` →
   `[dispatch-worker, capsule-orchestrator, capsule-phase-planner, capsule-worker]`,
   `pi` → `[dispatch-worker]`. Derive each dest filename from the asset basename
   (already done in `install_agent_def`, SL-206 PHASE-13) so the leg stays a loop,
   not four copy-paste variants.
2. Explicit match arms — no `_ => pi` wildcard (folds in IMP-181).
3. Name constants + a drift test mirroring `DISPATCH_WORKER_AGENT_TYPE`
   (`src/worktree/mod.rs`), since the def `name:` is machine-matched by the
   `SubagentStart` hook matcher.
4. Ship alongside the rest of IMP-434's deferral, which is entangled with it:
   the `planner` role in the `doctrine-role` vocabulary (a security-boundary
   change — `src/doctor_checks.rs` gates MCP token allowlists on it) and the
   `SubagentStart` role-band wiring. Sized slice-shaped, not quick-sketch-shaped.

## Consider in the same slice

- **ISS-216** — *doctrine install cannot reseat a changed agent def (skip-if-exists
  + stale pre-subdir-split symlink)*. Touches the same function and the same
  materialize-then-link layout: a plural leg that projects the capsule defs while
  install cannot reseat a changed def will ship stale bodies on the next edit. The
  reseat semantics (`--force` / always-re-expand derived defs) deserve the same
  design fork.
- **CHR-040** — `.pi/agents/dispatch-worker.md` still points at the historical
  `codex/` dir; the pi arm of the same match.
- **IMP-181** — the wildcard-fallback hardening (subsumed by (2)).

## Out of scope

- The capsule def bodies themselves, and `/capsule-driver`'s prose.
- Any change to `.claude/agents/` in this repo by hand — that is the status quo this
  item removes.
