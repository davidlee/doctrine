# Claude dispatch-agent worker worktree base is uncontrollable — run dependent serial phases inline on the coordination worktree

The `/dispatch-agent` arm spawns the worker via the `Agent` tool with
`isolation: worktree`. **Claude default-creates that worktree off the opaque
session-root `main` HEAD region — NOT a git ref the orchestrator can set.** This is
the concrete bite of the arm's confessed base-pinning residual (M1): "the base is
opaque and not orchestrator-controlled."

Observed across the SL-066 run (three spawns, three different bases, none what a
`git update-ref refs/remotes/origin/main …` predicted):
- spawn 1: forked stale `origin/main` (`7e2bc4b`) — local `main` was 32 ahead;
- spawn 2: forked `6062e28` — but `main == origin/main == 6062e28` then, so this
  "success" was a coincidence, not the tracking ref taking effect;
- spawn 3: after setting `origin/main` to the `dispatch/066` tip, the worker forked
  `main` HEAD (`23a1ce9`) anyway — proving the tracking ref is **not** the handle.

**Consequence.** **Independent** phases that all fork from the same base `B` (=
session HEAD) delegate fine — PHASE-02 did. But **serial DEPENDENT** phases cannot:
phase N+1 needs phase N's source, which the funnel keeps isolated on
`dispatch/<slice>` (never on `main` pre-audit, ADR-012). The worker forks off
`main`, which lacks it, so it correctly refuses (wrong base). There is no reliable
ref to redirect it, and advancing `main` to the dispatch tip would integrate
pre-audit (violates ADR-012).

**What to do.** For a slice whose phases are serial and dependent (the common case),
the claude arm buys nothing over inline execution — there is no parallelism to
exploit and the base is uncontrollable. **Execute the dependent phases inline on the
`dispatch/<slice>` coordination worktree** (doctrine sanctions inline for
non-delegable phases): same isolation (off `main`, pre-audit), reliable base, one
commit + `record-boundary` per phase, then conclude → audit. Reserve claude-arm
worker delegation for **file-disjoint independent** phases that all fork from the
same `B`.

A proper fix would be a harness affordance to pass the worker a base (IDE-004 /
WorktreeCreate `create-fork` path), or the codex/pi arm (`worktree fork --worker`
takes an explicit base `B`). Until then: inline for dependent serial work.
