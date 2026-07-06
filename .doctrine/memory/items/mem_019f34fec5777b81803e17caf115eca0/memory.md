# worker_commit false-reds on a stale PATH doctrine binary — land-not-rewrite

A claude-arm `worker_commit` gate can refuse a **correct** delta as
`commit-gate-red` when its validation binary lags the in-flight dispatch branch.
Empirically hit SL-206 PHASE-04 (2026-07-06, Opus 4.8, this harness).

## The trap (multi-phase dependency + stale gate binary)

- `worker_commit`'s `check commit` belt runs `just validate` → `doctrine doctor`
  via **PATH `~/.cargo/bin/doctrine`** — READONLY in the jail, built from
  **edge/main**, NOT the coordination branch.
- When a later phase's authored content DEPENDS on an earlier phase's
  binary-level change that lives only on `dispatch/<slice>` (unpromoted
  pre-audit), the PATH binary doesn't know it. Concretely: PHASE-04 shipped a
  `doctrine-role: probe` agent-def + 3 new orchestrator MCP tokens; the
  conformance surface that VALIDATES them (the `probe` role, the grown
  `allowed_mcp_tokens`) shipped in PHASE-03 — on `dispatch/206` only. So the
  edge-built PATH doctor rejects `invalid doctrine-role: probe` + `forbidden MCP
  token …` → exit 1 → gate red.
- Tell: the PATH binary and the coord/worktree binary report the **same version**
  (`0.16.0`) yet diverge — the difference is SOURCE, not version. `git diff` the
  trunk fork-point against the coord branch to see the gating source.

## Confirm it's a FALSE-red, not a real defect

Run the SAME check with the **fresh** binary (the worker worktree's
`./target/debug/doctrine`, built from `dispatch/<slice>` source):
`./target/debug/doctrine doctor` → exit 0, zero conformance findings. AGENTS.md
already mandates corpus/doctor verbs run from the coord tree's
`./target/debug/doctrine`, not PATH — the gate violates its own rule.

## Remedy — land-not-rewrite (same mechanism as the RO-`.git` case)

You cannot fix the gate in-jail (`~/.cargo/bin/doctrine` is readonly; promoting
the earlier phase pre-audit is off-limits). The orchestrator lands the worker's
own staged bytes and substitutes fresh-binary verification for the gate's
stale-binary check:

```
WT=.dispatch/SL-<n>/.worktrees/agent-<id>        # HEAD already == B (worker_commit never ran)
git -C "$WT" add -- <in-selector source files>
git -C "$WT" commit --author="<worker author>" -F - -- <files>   # C^==B, single non-merge
# orchestrator verification stands in for the false-red gate:
#   fresh ./target/debug/doctrine doctor  → exit 0 (zero conformance findings)
#   ./target/debug/doctrine check prove   → fmt + clippy clean
#   cargo test --bin doctrine             → full suite green (S-green ⟹ no regression)
dispatch_import{slice,name} → conclude_phase → reap              # import runs check prove, NOT doctor — no false-red
```

`dispatch_import`'s belt is R-5 + selectors + post-import `check prove` (fmt/lint)
— it does NOT run `doctrine doctor`, so the false-red does not recur at import.
Get REAL operator approval before bypassing the gate (SL-206: granted). This is
the SAME land-not-rewrite as [[mem.pattern.dispatch.correct-worker-delta-in-place]]
— there the worker couldn't self-commit due to a RO `.git`; here due to a gate
false-red. Same orchestrator move, different trigger.

## Also watch

- The import belt (`classify_import`, `src/worktree/import.rs`) requires a
  **design-target** selector to land ANY delta path — `scope-relevant` is NOT
  enough. A file the plan calls scope-relevant but the phase actually MODIFIES
  reds as `undeclared-scope` at import; upgrade it to design-target
  (register-before-land). SL-206 PHASE-04 hit this on `src/install.rs`.
- Filed backlog: the `worker_commit` belt's `doctrine doctor` should resolve
  from the build/coord binary, not PATH — else every phase that teaches the
  binary a check its predecessor shipped will false-red until promotion.
